use async_channel::{Receiver, Sender};
use vello::wgpu;
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene};
use vello::wgpu::Backends;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
}

pub struct RenderContext {
    tx: Sender<SceneTask>,
    handles: Vec<JoinHandle<()>>
}

pub struct SceneTask {
    scene: Scene,
    width: u32,
    height: u32,
    tx: oneshot::Sender<Result<(u32, u32, Vec<u8>), String>>,
}

impl SceneTask {
    pub fn new(scene: Scene, width: u32, height: u32) -> (oneshot::Receiver<Result<(u32, u32, Vec<u8>), String>>, Self) {
        let (tx, rx) = oneshot::channel();
        (rx, Self{
            scene,
            width,
            height,
            tx
        })
    }
}

impl RenderContext {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (tx, rx) = async_channel::unbounded::<SceneTask>();
        let instance = wgpu::Instance::default();
        let mut handles = Vec::new();
        for adapter in instance.enumerate_adapters(Backends::PRIMARY).await {
            let (device, queue) = adapter.request_device(&wgpu::DeviceDescriptor {
                label: Some("generator"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            }).await?;

            let renderer = Renderer::new(
                &device,
                RendererOptions {
                    use_cpu: false,
                    antialiasing_support: AaSupport::area_only(),
                    num_init_threads: None,
                    pipeline_cache: None,
                },
            )?;

            let ctx = GpuContext {
                device,
                queue,
                renderer
            };
            let handle = tokio::spawn(Self::worker(ctx, rx.clone()));

            handles.push(handle);
        }

        Ok(Self { tx, handles })
    }

    pub async fn send_render_task(
        &mut self,
        task: SceneTask
    ) {
        self.tx.send(task).await.unwrap()
    }

    async fn worker(mut ctx: GpuContext, rx: Receiver<SceneTask>) {
        let mut cached_width = 0;
        let mut cached_height = 0;
        let mut texture: Option<wgpu::Texture> = None;
        let mut view: Option<wgpu::TextureView> = None;
        let mut buffer: Option<wgpu::Buffer> = None;
        let mut pixels = Vec::new();

        while let Ok(task) = rx.recv().await {
            let width = task.width;
            let height = task.height;
            let padded_bytes_per_row = (width * 4 + 255) & !255;

            if width != cached_width || height != cached_height || texture.is_none() {
                let tex = ctx.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("render_target"),
                    size: wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
                    view_formats: &[],
                });
                
                view = Some(tex.create_view(&wgpu::TextureViewDescriptor::default()));
                texture = Some(tex);

                buffer = Some(ctx.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("readback"),
                    size: (padded_bytes_per_row * height) as u64,
                    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                    mapped_at_creation: false,
                }));

                cached_width = width;
                cached_height = height;
            }

            let tex = texture.as_ref().unwrap();
            let tex_view = view.as_ref().unwrap();
            let buf = buffer.as_ref().unwrap();

            let params = RenderParams {
                base_color: vello::peniko::Color::TRANSPARENT,
                width,
                height,
                antialiasing_method: AaConfig::Area,
            };

            if let Err(e) = ctx.renderer
                .render_to_texture(&ctx.device, &ctx.queue, &task.scene, tex_view, &params) {
                task.tx.send(Err(format!("Failed to render scene to a texture: {}", e))).ok();
                continue;
            }

            let mut encoder = ctx
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            encoder.copy_texture_to_buffer(
                wgpu::TexelCopyTextureInfo {
                    texture: tex,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::TexelCopyBufferInfo {
                    buffer: buf,
                    layout: wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_bytes_per_row),
                        rows_per_image: Some(height),
                    },
                },
                wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );

            ctx.queue.submit(std::iter::once(encoder.finish()));

            let buffer_slice = buf.slice(..);
            let (tx_gpu, rx_gpu) = std::sync::mpsc::channel();
            buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
                tx_gpu.send(result).unwrap();
            });
            if let Err(e) = ctx.device
                .poll(wgpu::PollType::Wait { submission_index: None, timeout: None }) {
                task.tx.send(Err(format!("Failed to poll on render task: {}", e))).ok();
                continue;
            }

            if let Err(e) = rx_gpu.recv() {
                task.tx.send(Err(format!("Failed to send task to a GPU: {}", e))).ok();
                continue;
            }

            let mapped = match buffer_slice.get_mapped_range(){
                Ok(m) => m,
                Err(_) => continue,
            };

            let unpadded_bytes_per_row = width * 4;
            pixels.clear();
            pixels.reserve((unpadded_bytes_per_row * height) as usize);
            for row in 0..height {
                let start = (row * padded_bytes_per_row) as usize;
                let end = start + unpadded_bytes_per_row as usize;
                pixels.extend_from_slice(&mapped[start..end]);
            }

            drop(mapped);
            buf.unmap();

            task.tx.send(Ok((width, height, std::mem::take(&mut pixels)))).ok();
        }
    }
}

#[cfg(test)]
mod tests {
    use tokio::sync::oneshot::error::RecvError;
    use super::*;

    fn extract(rx: Result<Result<(u32, u32, Vec<u8>), String>, RecvError>) -> (u32, u32, Vec<u8>) {
        let (w, h, pixels) = match rx {
            Ok(Ok(data)) => data,
            Ok(Err(e)) => panic!("{}", e),
            Err(e) => panic!("GPU worker died before responding: {}", e),
        };
        (w, h, pixels)
    }

    #[tokio::test]
    async fn render_context_creation() {
        let ctx = RenderContext::new().await;
        assert!(ctx.is_ok(), "Failed to create RenderContext: {:?}", ctx.err());
    }

    #[tokio::test]
    async fn render_empty_scene() {
        let mut ctx = RenderContext::new().await.expect("No GPU available");
        let scene = Scene::new();
        let (rx, task) = SceneTask::new(scene, 10, 10);
        ctx.send_render_task(task).await;
        let (w, h, pixels) = extract(rx.await);
        assert_eq!(w, 10);
        assert_eq!(h, 10);
        assert_eq!(pixels.len(), (10 * 10 * 4) as usize);
    }

    #[tokio::test]
    async fn render_scene_returns_correct_dimensions() {
        let mut ctx = RenderContext::new().await.expect("No GPU available");
        let scene = Scene::new();
        for &(sw, sh) in &[(1, 1), (64, 64), (100, 200), (300, 150)] {
            let (rx, task) = SceneTask::new(scene.clone(), sw, sh);
            ctx.send_render_task(task).await;
            let (w, h, pixels) = extract(rx.await);
            assert_eq!(w, sw, "width mismatch");
            assert_eq!(h, sh, "height mismatch");
            assert_eq!(pixels.len(), (sw * sh * 4) as usize);
        }
    }

    #[tokio::test]
    async fn render_filled_rect_has_nonzero_pixels() {
        use vello::kurbo::{Affine, Rect};
        use vello::peniko::{Brush, Fill};

        let mut ctx = RenderContext::new().await.expect("No GPU available");
        let mut scene = Scene::new();
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            &Brush::Solid(vello::peniko::Color::from_rgb8(255, 0, 0)),
            None,
            &Rect::new(0.0, 0.0, 10.0, 10.0),
        );

        let (rx, task) = SceneTask::new(scene.clone(), 10, 10);
        ctx.send_render_task(task).await;
        let (w, h, pixels) = extract(rx.await);
        assert_eq!(w, 10);
        assert_eq!(h, 10);
        let has_nonzero = pixels.iter().any(|&p| p != 0);
        assert!(has_nonzero, "Scene with filled rect should have non-zero pixels");
    }

    #[tokio::test]
    async fn render_transparent_scene_is_empty() {
        let mut ctx = RenderContext::new().await.expect("No GPU available");
        let scene = Scene::new();
        let (rx, task) = SceneTask::new(scene.clone(), 4, 4);
        ctx.send_render_task(task).await;
        let (_w, _h, pixels) = extract(rx.await);
        assert!(pixels.iter().all(|&p| p == 0), "Empty scene should be fully transparent");
    }
}
