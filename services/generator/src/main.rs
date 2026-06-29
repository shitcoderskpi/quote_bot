use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use vello::kurbo::Affine;
use vello::Scene;

mod compressor;
mod config;
mod layout;
mod parser;
mod redis_queue;
mod renderer;
mod templater;
mod text;

fn process_job(
    raw: &str,
    cfg: &config::Config,
    render_ctx: &mut renderer::RenderContext,
    font_cx: &mut parley::FontContext,
    layout_cx: &mut parley::LayoutContext,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // ── Stage 1: Parse input and produce wire-format message ──
    let (msg, dpi) = if raw.trim_start().starts_with('{') {
        // JSON input → template pipeline
        let input_msg: templater::InputMessage = serde_json::from_str(raw)?;
        let msg_dpi = input_msg.dpi;

        let theme = input_msg.theme.as_deref().unwrap_or("light");
        let template_name = if theme == "dark" { "dark.tem" } else { "light.tem" };
        let template_path = std::path::Path::new(&cfg.templates_dir).join(template_name);
        let template_str = std::fs::read_to_string(&template_path)?;

        // Parse template once — used for both font lookups and rendering
        let template = templater::ParsedTemplate::parse(&template_str);
        let quote_layout = layout::compute_layout(&input_msg, &template, font_cx, layout_cx);
        let payload_str = template.render(&input_msg, &quote_layout)?;
        let msg = parser::parse(&payload_str)?;

        (msg, msg_dpi)
    } else {
        // Raw wire-format input (no template)
        let msg = parser::parse(raw)?;
        (msg, None)
    };

    // ── Stage 2: Build vello scene from SVG ──
    let mut scene = vello_svg::render(&msg.svg.data)?;

    let tree = vello_svg::usvg::Tree::from_str(
        &msg.svg.data,
        &vello_svg::usvg::Options::default(),
    )?;
    let size = tree.size();
    let width = size.width().ceil() as u32;
    let height = size.height().ceil() as u32;

    // ── Stage 3: Draw text layers onto scene ──
    text::draw_text_layers(&mut scene, &msg.texts, font_cx, layout_cx);

    // ── Stage 4: Scale for DPI and rasterize ──
    let dpi = dpi.unwrap_or(cfg.dpi);
    let scale = dpi as f64 / 96.0;
    let mut scaled_scene = Scene::new();
    scaled_scene.append(&scene, Some(Affine::scale(scale)));

    let scaled_width = (width as f64 * scale).ceil() as u32;
    let scaled_height = (height as f64 * scale).ceil() as u32;

    let (w, h, pixels) =
        render_ctx.render_scene_to_pixels(&scaled_scene, scaled_width, scaled_height)?;
    info!("Rendered {}x{}", w, h);

    // ── Stage 5: Encode to WebP ──
    let mut webp_data = std::io::Cursor::new(Vec::new());
    let img_buf = image::RgbaImage::from_raw(w, h, pixels)
        .ok_or("Failed to create RgbaImage from raw pixels")?;
    img_buf.write_to(&mut webp_data, image::ImageFormat::WebP)?;

    // ── Stage 6: Build result JSON, compress, return ──
    use base64::prelude::*;
    let b64_img = BASE64_STANDARD.encode(webp_data.into_inner());
    let json_res = format!(r#"{{"header": {}, "image": "{}"}}"#, msg.header, b64_img);

    let compressed = compressor::compress(&json_res, 9)?;
    Ok(compressed)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("generator=info".parse()?))
        .init();

    let cfg = match config::Config::from_env() {
        Ok(c) => c,
        Err(e) => {
            error!("Configuration Error: {}", e);
            std::process::exit(1);
        }
    };

    info!("Connecting to Redis at {}:{}", cfg.redis_host, cfg.redis_port);
    let mut queue = match redis_queue::RedisQueue::connect(&cfg).await {
        Ok(q) => q,
        Err(e) => {
            error!("Failed to connect to Redis: {}", e);
            std::process::exit(1);
        }
    };

    let mut render_ctx = renderer::RenderContext::new()
        .await
        .expect("Failed to initialize GPU render context");

    let mut font_cx = parley::FontContext::new();
    let mut layout_cx = parley::LayoutContext::new();

    loop {
        let payload = match queue.dequeue(cfg.queue_name.clone(), 0.0).await {
            Ok(Some(p)) => p,
            Ok(None) => continue,
            Err(e) => {
                error!("Error dequeuing job: {}", e);
                continue;
            }
        };

        let raw = match compressor::decompress(&payload) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to decompress job: {}", e);
                continue;
            }
        };

        match process_job(&raw, &cfg, &mut render_ctx, &mut font_cx, &mut layout_cx) {
            Ok(result) => {
                if let Err(e) = queue.enqueue(&cfg.results_queue, result).await {
                    error!("Failed to enqueue result: {}", e);
                } else {
                    info!("Pushed result for message");
                }
            }
            Err(e) => error!("Job failed: {}", e),
        }
    }
}
