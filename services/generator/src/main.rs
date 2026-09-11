use std::collections::HashMap;
use steel::compiler::program::Executable;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use vello::kurbo::Rect;
use vello::Scene;
use crate::renderer::SceneTask;
use crate::templater::Templater;

mod compressor;
mod config;
mod redis_queue;
mod templater;
mod primitives;
mod renderer;

async fn process_job<'a>(
    raw: &str,
    cfg: &config::Config,
    templater: &mut Templater,
    themes: &HashMap<String, Executable>,
    renderer: &mut primitives::Renderer,
    render_ctx: &mut renderer::RenderContext,
    buf: &'a mut Vec<u8>,
) -> Result<&'a mut Vec<u8>, Box<dyn std::error::Error>> {
    let input_msg: serde_json::Value = serde_json::from_str(raw)?;
    let dpi = input_msg.get("dpi")
        .and_then(|v| v.as_f64())
        .map(|v| v as f32)
        .unwrap_or(cfg.dpi);
    let theme = input_msg.get("theme")
        .and_then(|v| v.as_str())
        .unwrap_or("light");

    let executable = themes.get(theme).ok_or_else(|| format!("Template {} not found", theme))?;

    let content = input_msg.get("content")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or("".to_string());
    let entities = input_msg.get("entities")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or(vec![]);


    let node_tree = templater.render_template(executable, raw, content, entities)?;

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

async fn read_and_compile_templates<'a>(dir: &String, templater: &mut Templater) -> HashMap<String, Executable> {
    let mut map = HashMap::new();
    let mut tasks = Vec::new();
    
    if let Ok(mut entries) = tokio::fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "scm" {
                        let name = path.file_stem()
                            .unwrap()
                            .to_string_lossy()
                            .to_string();
                        tasks.push(tokio::spawn(async move {
                            let content = tokio::fs::read_to_string(&path)
                                .await
                                .unwrap_or_default();
                            (name, content)
                        }));
                    }
                }
            }
        }
    }
    
    for task in tasks {
        if let Ok((name, content)) = task.await {
            if let Ok(exec) = templater.compile(content) {
                map.insert(name, exec);
            } else {
                error!("Failed to compile template: {}", name);
            }
        }
    }
    
    map
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("generator=info".parse()?))
        .init();

    let cfg = config::Config::from_env()?;
    let mut templater = Templater::new();
    
    let mut renderer = primitives::Renderer::new();
    
    let mut render_ctx = renderer::RenderContext::new()
        .await
        .expect("Failed to initialize GPU render context");

    info!("Compiling template schemes...");
    let themes = read_and_compile_templates(
        &cfg.templates_dir,
        &mut templater
    ).await;

    info!("Connecting to Redis at {}:{}", cfg.redis_host, cfg.redis_port);
    let mut queue = redis_queue::RedisQueue::connect(&cfg).await?;
    let mut decmpd = String::new();
    let mut cmpd: Vec<u8> = Vec::new();

    loop {
        let payload = match queue.dequeue(&cfg.queue_name, 0.0).await {
            Ok(p) => p,
            Err(e) => {
                if !e.is_timeout() {
                    error!("Error getting job: {}", e);
                    continue;
                }
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

        match process_job(&decmpd, &cfg, &mut templater, &themes, &mut renderer, &mut render_ctx, &mut cmpd).await {
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
