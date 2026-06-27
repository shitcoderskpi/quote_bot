use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use vello::kurbo::Affine;
use vello::Scene;

mod compressor;
mod config;
mod parser;
mod redis_queue;
mod renderer;
mod text;
mod layout;
mod templater;

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
        match queue.dequeue(cfg.queue_name.clone(), 0.0).await {
            Ok(Some(payload)) => {
                match compressor::decompress(&payload) {
                    Ok(raw) => {
                        let msg_result = if raw.trim_start().starts_with('{') {
                            match serde_json::from_str::<templater::InputMessage>(&raw) {
                                Ok(input_msg) => {
                                    let content = input_msg.content.as_deref().unwrap_or("").trim_end();
                                    let template_str_res = std::fs::read_to_string(&cfg.template_path);
                                    if let Err(e) = template_str_res {
                                        Err(format!("Failed to read template file: {}", e))
                                    } else {
                                        let template_str = template_str_res?;
                                        let font = layout::extract_content_font(&template_str);
                                        let dims = layout::compute_dimensions(content, &mut font_cx, &mut layout_cx, &font);
                                        match templater::render_payload(&cfg, &input_msg, &dims) {
                                            Ok(payload_str) => parser::parse(&payload_str).map_err(|e| format!("{:?}", e)),
                                            Err(e) => Err(e),
                                        }
                                    }
                                }
                                Err(e) => Err(format!("Failed to parse JSON: {}", e)),
                            }
                        } else {
                            parser::parse(&raw).map_err(|e| format!("{:?}", e))
                        };

                        match msg_result {
                            Ok(msg) => {
                                let mut scene = match vello_svg::render(&msg.svg.data) {
                                    Ok(s) => s,
                                    Err(e) => {
                                        error!("Failed to build scene from SVG: {}", e);
                                        continue;
                                    }
                                };

                                let tree = match vello_svg::usvg::Tree::from_str(
                                    &msg.svg.data,
                                    &vello_svg::usvg::Options::default(),
                                ) {
                                    Ok(t) => t,
                                    Err(e) => {
                                        error!("Failed to parse SVG tree: {}", e);
                                        continue;
                                    }
                                };
                                let size = tree.size();
                                let width = size.width().ceil() as u32;
                                let height = size.height().ceil() as u32;

                                text::draw_text_layers(&mut scene, &msg.texts, &mut font_cx, &mut layout_cx);

                                let scale = cfg.dpi as f64 / 96.0;
                                let mut scaled_scene = Scene::new();
                                scaled_scene.append(&scene, Some(Affine::scale(scale)));

                                let scaled_width = (width as f64 * scale).ceil() as u32;
                                let scaled_height = (height as f64 * scale).ceil() as u32;

                                match render_ctx.render_scene_to_pixels(&scaled_scene, scaled_width, scaled_height) {
                                    Ok((w, h, pixels)) => {
                                        info!("Rendered {}x{}", w, h);
                                        let mut webp_data = std::io::Cursor::new(Vec::new());
                                        if let Some(img_buf) = image::RgbaImage::from_raw(w, h, pixels) {
                                            if let Err(e) = img_buf.write_to(&mut webp_data, image::ImageFormat::WebP) {
                                                error!("Failed to encode WebP: {}", e);
                                                continue;
                                            }

                                            use base64::prelude::*;
                                            let b64_img = BASE64_STANDARD.encode(webp_data.into_inner());
                                            let json_res = format!(r#"{{"chat_id": {}, "image": "{}"}}"#, msg.chat_id, b64_img);

                                            match compressor::compress(&json_res, 9) {
                                                Ok(compressed) => {
                                                    if let Err(e) = queue.enqueue(&cfg.results_queue, compressed).await {
                                                        error!("Failed to enqueue result: {}", e);
                                                    } else {
                                                        info!("Pushed result for chat_id {}", msg.chat_id);
                                                    }
                                                }
                                                Err(e) => error!("Failed to compress result: {}", e),
                                            }
                                        } else {
                                            error!("Failed to create RgbaImage from raw pixels");
                                        }
                                    }
                                    Err(e) => error!("Render failed: {}", e),
                                }
                            }
                            Err(e) => error!("Failed to parse job: {}", e),
                        }
                    }
                    Err(e) => error!("Failed to decompress job: {}", e),
                }
            }
            Ok(None) => continue,
            Err(e) => {
                // if e.is_timeout() {
                //     continue;
                // }
                error!("Error dequeuing job: {}", e);
            }
        }
    }
}
