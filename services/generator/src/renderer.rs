use vello::wgpu;
use vello::{AaConfig, AaSupport, RenderParams, Renderer, RendererOptions, Scene};

pub struct RenderContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: Renderer,
}

// Possibly can, and should be used per GPU
impl RenderContext {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            })
            .await?;

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("generator"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await?;

        let renderer = Renderer::new(
            &device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: AaSupport::area_only(),
                num_init_threads: None,
                pipeline_cache: None,
            },
        )?;

        Ok(Self { device, queue, renderer })
    }

    pub fn render_scene_to_pixels(
        &mut self,
        scene: &Scene,
        width: u32,
        height: u32,
    ) -> Result<(u32, u32, Vec<u8>), Box<dyn std::error::Error>> {
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
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
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let params = RenderParams {
            base_color: vello::peniko::Color::TRANSPARENT,
            width,
            height,
            antialiasing_method: AaConfig::Area,
        };

        self.renderer
            .render_to_texture(&self.device, &self.queue, scene, &view, &params)?;

        let padded_bytes_per_row = (width * 4 + 255) & !255;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("readback"),
            size: (padded_bytes_per_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
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

        self.queue.submit(std::iter::once(encoder.finish()));

        let buffer_slice = buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        self.device.poll(wgpu::PollType::Wait { submission_index: None, timeout: None })
            .expect("ERROR: failed to wait for render task");
        rx.recv()??;

        let mapped = buffer_slice.get_mapped_range();

        let unpadded_bytes_per_row = width * 4;
        let mut pixels = Vec::with_capacity((unpadded_bytes_per_row * height) as usize);
        for row in 0..height {
            let start = (row * padded_bytes_per_row) as usize;
            let end = start + unpadded_bytes_per_row as usize;
            pixels.extend_from_slice(&mapped[start..end]);
        }

        drop(mapped);
        buffer.unmap();

        Ok((width, height, pixels))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn render_context_creation() {
        let ctx = RenderContext::new().await;
        assert!(ctx.is_ok(), "Failed to create RenderContext: {:?}", ctx.err());
    }

    #[tokio::test]
    async fn render_empty_scene() {
        let mut ctx = RenderContext::new().await.expect("No GPU available");
        let scene = Scene::new();
        let result = ctx.render_scene_to_pixels(&scene, 10, 10);
        assert!(result.is_ok(), "render failed: {:?}", result.err());
        let (w, h, pixels) = result.unwrap();
        assert_eq!(w, 10);
        assert_eq!(h, 10);
        assert_eq!(pixels.len(), (10 * 10 * 4) as usize);
    }

    #[tokio::test]
    async fn render_scene_returns_correct_dimensions() {
        let mut ctx = RenderContext::new().await.expect("No GPU available");
        let scene = Scene::new();
        for &(sw, sh) in &[(1, 1), (64, 64), (100, 200), (300, 150)] {
            let (w, h, pixels) = ctx.render_scene_to_pixels(&scene, sw, sh).unwrap();
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

        let (w, h, pixels) = ctx.render_scene_to_pixels(&scene, 10, 10).unwrap();
        assert_eq!(w, 10);
        assert_eq!(h, 10);
        let has_nonzero = pixels.iter().any(|&p| p != 0);
        assert!(has_nonzero, "Scene with filled rect should have non-zero pixels");
    }

    #[tokio::test]
    async fn render_transparent_scene_is_empty() {
        let mut ctx = RenderContext::new().await.expect("No GPU available");
        let scene = Scene::new();
        let (_w, _h, pixels) = ctx.render_scene_to_pixels(&scene, 4, 4).unwrap();
        assert!(pixels.iter().all(|&p| p == 0), "Empty scene should be fully transparent");
    }
}
