use prost::bytes::Bytes;
use std::collections::HashMap;
use prost::bytes::BytesMut;
use steel::compiler::program::Executable;
use steel::rerrs::ErrorKind;
use steel::SteelErr;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use vello::Scene;
use crate::renderer::SceneTask;
use crate::templater::Templater;

mod compressor;
mod config;
mod nats_queue;
mod templater;
mod primitives;
mod renderer;

pub mod proto;

async fn process_job(
    raw: &[u8],
    cfg: &config::Config,
    templater: &mut Templater,
    themes: &HashMap<String, Executable>,
    renderer: &mut primitives::Renderer,
    render_ctx: &mut renderer::RenderContext,
    buf: &mut BytesMut,
    pb_buf: &mut Vec<u8>,
    webp_buf: &mut Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    use prost::Message;
    let req = proto::quote::QuoteRequest::decode(raw)?;

    let chat_id = req.chat_id;
    let message_id = req.messages.first().map(|m| m.message_id).unwrap_or(0);
    let dpi = req.dpi.unwrap_or(cfg.dpi as u32) as f32;
    let theme = if req.theme.is_empty() { "light" } else { &req.theme };

    let executable = themes.get(theme).ok_or_else(|| format!("Template {} not found", theme))?;

    let node_tree = templater.render_template(executable, req)?;

    let viewport = primitives::Viewport { width: 1.0, height: 1.0 };

    let (width, height, root_id) = renderer.compute_layout(&node_tree, viewport)?;
    let mut scene = Scene::new();

    renderer.render(&mut scene, &node_tree, root_id);

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

    let mut webp_data = std::io::Cursor::new(&mut *webp_buf);
    let img_buf = image::RgbaImage::from_raw(w, h, pixels)
        .ok_or("Failed to create RgbaImage from raw pixels")?;
    img_buf.write_to(&mut webp_data, image::ImageFormat::WebP)?;

    let result_img = proto::quote::ResultImage {
        image: std::mem::take(webp_buf),
        message_id,
        chat_id,
    };
    
    pb_buf.clear();
    result_img.encode(pb_buf)?;

    buf.resize(compressor::max_compressed_size(pb_buf.len()), 0);

    let n = compressor::compress(pb_buf, 4, buf.as_mut())?;
    buf.truncate(n);

    Ok(())
}

async fn read_and_compile_templates<'a>(dir: &String, templater: &mut Templater) -> Result<HashMap<String, Executable>, SteelErr> {
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
            match templater.compile(content){
                Ok(exec) => {
                    map.insert(name, exec);
                },
                Err(e) => {
                    return Err(SteelErr::new(
                        ErrorKind::Parse,
                        format!("Failed to compile template {}: {}", name, e)
                    ));
                }
            }
        }
    }

    Ok(map)
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
    ).await?;

    info!("Connecting to NATS at {}", cfg.nats_url);
    let mut queue = nats_queue::NatsQueue::connect(&cfg.nats_url).await?;
    let mut decmpd: Vec<u8> = Vec::new();
    let mut pb_buf: Vec<u8> = Vec::new();
    let mut webp_buf: Vec<u8> = Vec::new();
    let mut cmpd = BytesMut::with_capacity(64 * 1024);

    loop {
        cmpd.clear();
        decmpd.clear();
        webp_buf.clear();

        let payload = tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl+C, shutting down gracefully...");
                break;
            }
            res = queue.dequeue(&cfg.queue_name) => {
                match res {
                    Ok(Some(p)) => p,
                    Ok(None) => continue,
                    Err(e) => {
                        error!("Error getting job: {}", e);
                        continue;
                    }
                }
            }
        };
        match compressor::decompress(payload.as_ref(), &mut decmpd) {
            Ok(r) => r,
            Err(e) => {
                error!("Failed to decompress job: {}", e);
                continue;
            }
        };

        match process_job(&decmpd,
                          &cfg,
                          &mut templater,
                          &themes,
                          &mut renderer,
                          &mut render_ctx,
                          &mut cmpd,
                          &mut pb_buf,
                          &mut webp_buf,
        ).await {
            Ok(()) => {
                let out: Bytes = cmpd.split().freeze();
                if let Err(e) = queue.enqueue(&cfg.results_queue, out).await {
                    error!("Failed to enqueue result: {}", e);
                } else {
                    info!("Pushed result for message");
                }
            }
            Err(e) => error!("Job failed: {}", e),
        }
    }
    Ok(())
}
