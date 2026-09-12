pub mod node;
pub mod paint;
pub mod render;
pub mod shape;
pub mod text;

pub use render::Renderer;

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub width: f64,
    pub height: f64,
}
