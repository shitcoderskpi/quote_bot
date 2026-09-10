use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use vello::kurbo::Rect;
use vello::Scene;
use crate::renderer::SceneTask;

mod compressor;
mod config;
mod redis_queue;
mod templater;
mod primitives;
mod renderer;

async fn process_job<'a>(
    raw: &str,
    cfg: &config::Config,
    templater: &mut templater::Templater,
    renderer: &mut primitives::Renderer,
    render_ctx: &mut renderer::RenderContext,
    buf: &'a mut Vec<u8>,
) -> Result<&'a mut Vec<u8>, Box<dyn std::error::Error>> {
    let input_msg: serde_json::Value = serde_json::from_str(raw)?;
    let dpi = input_msg.get("dpi").and_then(|v| v.as_f64()).map(|v| v as f32).unwrap_or(cfg.dpi);

    let theme = input_msg.get("theme").and_then(|v| v.as_str()).unwrap_or("light");
    let template_name = format!("{theme}.scm");
    let template_path = std::path::Path::new(&cfg.templates_dir).join(template_name);
    let template_str = std::fs::read_to_string(&template_path)?;

    let content_opt = input_msg.get("content").and_then(|v| v.as_str()).map(|s| s.to_string());
    let entities_opt = input_msg.get("entities").and_then(|v| v.as_array()).cloned();

    let node_tree = templater.render_template(&template_str, raw, content_opt, entities_opt)?;

    let viewport = primitives::Viewport { width: 1.0, height: 1.0 };

    let (measured_w, measured_h) = renderer.compute_layout(&node_tree, viewport)?;

    let width  = measured_w;
    let height = measured_h;

    let mut scene = Scene::new();
    let root_box = Rect::new(0.0, 0.0, width, height);

    renderer.render(&mut scene, &node_tree, root_box, viewport);

    let scale = dpi as f64 / 96.0;
    let scaled_width = (width * scale).ceil() as u32;
    let scaled_height = (height * scale).ceil() as u32;

    let mut scaled_scene = Scene::new();
    scaled_scene.append(&scene, Some(vello::kurbo::Affine::scale(scale)));

    let (rx, task) = SceneTask::new(scaled_scene, scaled_width, scaled_height);
    render_ctx.send_render_task(task).await;
    let (w, h, pixels) = match rx.await {
        Ok(Ok(data)) => data,
        Ok(Err(e)) => panic!("{}", e),
        Err(e) => panic!("GPU worker died before responding: {}", e),
    };
    info!("Rendered {}x{}", w, h);

    let mut webp_data = std::io::Cursor::new(Vec::new());
    let img_buf = image::RgbaImage::from_raw(w, h, pixels)
        .ok_or("Failed to create RgbaImage from raw pixels")?;
    img_buf.write_to(&mut webp_data, image::ImageFormat::WebP)?;

    use base64::prelude::*;
    let b64_img = BASE64_STANDARD.encode(webp_data.into_inner());

    let header = input_msg.get("header")
        .map(|h| h.to_string())
        .unwrap_or_else(|| "{}".to_string());

    let json_res = format!(r#"{{"header": {}, "image": "{}"}}"#, header, b64_img);

    buf.resize(compressor::max_compressed_size(json_res.len()), 0);

    let n = compressor::compress(&json_res, 9, buf.as_mut_slice())?;
    buf.truncate(n);

    Ok(buf)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("generator=info".parse()?))
        .init();

    let cfg = config::Config::from_env()?;
    let mut templater = templater::Templater::new();
    
    let mut renderer = primitives::Renderer::new();
    
    let mut render_ctx = renderer::RenderContext::new()
        .await
        .expect("Failed to initialize GPU render context");

    info!("Connecting to Redis at {}:{}", cfg.redis_host, cfg.redis_port);
    let mut queue = redis_queue::RedisQueue::connect(&cfg).await?;
    let mut decmpd = String::new();
    let mut cmpd: Vec<u8> = Vec::new();

    loop {
        let payload = match queue.dequeue(cfg.queue_name.clone(), 0.0).await {
            Ok(Some(p)) => p,
            Ok(None) => continue,
            Err(e) => {
                error!("Error dequeuing job: {}", e);
                continue;
            }
        };

        decmpd.clear();
        match compressor::decompress(&payload, &mut decmpd) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to decompress job: {}", e);
                continue;
            }
        };

        match process_job(&decmpd, &cfg, &mut templater, &mut renderer, &mut render_ctx, &mut cmpd).await {
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
