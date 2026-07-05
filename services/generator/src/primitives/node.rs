use crate::primitives::paint::{Paint, Stroke};
use crate::primitives::shape::ShapeKind;
use crate::primitives::text::RichText;
use vello::peniko::ImageBrush;
use std::sync::Arc;

/// Layout/paint properties shared by every node, all unit-flexible.
#[derive(Clone, Debug)]
pub struct Style {
    pub layout: taffy::Style,
    pub rotate_deg: f64,
    pub opacity: f32,
    /// Optional clip mask applied to this node and its children, resolved
    /// against this node's own box.
    pub clip: Option<ShapeKind>,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            layout: taffy::Style::default(),
            rotate_deg: 0.0,
            opacity: 1.0,
            clip: None,
        }
    }
}

#[derive(Clone)]
pub enum Content {
    Shape { kind: ShapeKind, fill: Option<Paint>, stroke: Option<Stroke> },
    Text(RichText),
    /// An image clipped into `clip` (defaults to the box rect if `None`),
    /// scaled to fill the resolved box -- this is the "images in various
    /// shapes" case (circular avatar, rounded thumbnail, arbitrary path).
    Image { image: Arc<ImageBrush>, clip: Option<ShapeKind> },
    Group(Vec<Node>),
}

impl std::fmt::Debug for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Group(c) => f.debug_tuple("Group").field(c).finish(),
            Content::Shape { kind, fill, stroke } => f
                .debug_struct("Shape")
                .field("kind", kind)
                .field("fill", fill)
                .field("stroke", stroke)
                .finish(),
            Content::Image { clip, .. } => f.debug_struct("Image").field("clip", clip).finish(),
            Content::Text(rich) => f
                .debug_struct("Text")
                .field("text", &rich.text)
                .field("spans", &rich.spans)
                .field("default_color", &rich.default_color)
                .finish(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    pub style: Style,
    pub content: Content,
}

impl Node {
    pub fn shape(kind: ShapeKind, fill: Option<Paint>) -> Self {
        Self { style: Style::default(), content: Content::Shape { kind, fill, stroke: None } }
    }

    pub fn text(rich: RichText) -> Self {
        Self { style: Style::default(), content: Content::Text(rich) }
    }

    pub fn group(children: Vec<Node>) -> Self {
        Self { style: Style::default(), content: Content::Group(children) }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}
