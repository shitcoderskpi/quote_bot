pub mod node;
pub mod paint;
pub mod render;
pub mod shape;
pub mod text;

pub use node::{Content, Node, Style};
pub use paint::{Paint, Stop, Stroke};
pub use render::Renderer;
pub use shape::ShapeKind;
pub use text::{RichText, Span, TextAlign};

#[derive(Clone, Copy, Debug)]
pub struct Viewport {
    pub width: f64,
    pub height: f64,
}
