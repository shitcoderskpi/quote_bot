use crate::primitives::paint::{Paint, Stroke};
use crate::primitives::shape::ShapeKind;
use crate::primitives::text::RichText;
use vello::peniko::ImageBrush;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Style {
    pub layout: taffy::Style,
    pub rotate_deg: f64,
    pub opacity: f32,
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

    pub fn image(image: Arc<ImageBrush>, clip: Option<ShapeKind>) -> Self {
        Self { style: Style::default(), content: Content::Image { image, clip } }
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vello::peniko::ImageData;
    use crate::primitives::paint::Paint;
    use crate::primitives::shape::{Corners, ShapeKind};
    use crate::primitives::text::RichText;

    #[test]
    fn shape_constructor_defaults() {
        let node = Node::shape(
            ShapeKind::Rect { corners: Corners::zero() },
            Some(Paint::solid(vello::peniko::Color::BLACK)),
        );
        assert!(matches!(node.content, Content::Shape { ref kind, ref fill, ref stroke }
            if matches!(kind, ShapeKind::Rect { .. })
            && fill.is_some()
            && stroke.is_none()
        ));
        assert_eq!(node.style.rotate_deg, 0.0);
        assert_eq!(node.style.opacity, 1.0);
        assert!(node.style.clip.is_none());
    }

    #[test]
    fn shape_constructor_no_fill() {
        let node = Node::shape(ShapeKind::Circle, None);
        assert!(matches!(node.content, Content::Shape { ref fill, .. } if fill.is_none()));
    }

    #[test]
    fn text_constructor() {
        let rt = RichText::plain("hello");
        let node = Node::text(rt);
        match &node.content {
            Content::Text(rich) => assert_eq!(rich.text, "hello"),
            other => panic!("Expected Text, got {:?}", other),
        }
    }

    #[test]
    fn group_constructor_with_children() {
        let children = vec![
            Node::shape(ShapeKind::Circle, None),
            Node::text(RichText::plain("a")),
        ];
        let node = Node::group(children);
        match &node.content {
            Content::Group(c) => assert_eq!(c.len(), 2),
            other => panic!("Expected Group, got {:?}", other),
        }
    }

    #[test]
    fn group_constructor_empty() {
        let node = Node::group(vec![]);
        match &node.content {
            Content::Group(c) => assert!(c.is_empty()),
            other => panic!("Expected empty Group, got {:?}", other),
        }
    }

    #[test]
    fn image_constructor() {
        let pixels = vec![255u8; 4];
        let image_data = new_image_data(pixels);
        let brush = Arc::new(ImageBrush::new(image_data));
        let node = Node::image(brush, Some(ShapeKind::Circle));
        assert!(matches!(node.content, Content::Image { ref clip, .. } if clip.is_some()));
    }

    fn new_image_data(pixels: Vec<u8>) -> ImageData {
        let image_data = ImageData {
            data: vello::peniko::Blob::new(Arc::new(pixels)),
            format: vello::peniko::ImageFormat::Rgba8,
            alpha_type: vello::peniko::ImageAlphaType::Alpha,
            width: 1,
            height: 1,
        };
        image_data
    }

    #[test]
    fn image_constructor_no_clip() {
        let pixels = vec![0u8; 4];
        let image_data = new_image_data(pixels);
        let brush = Arc::new(ImageBrush::new(image_data));
        let node = Node::image(brush, None);
        assert!(matches!(node.content, Content::Image { ref clip, .. } if clip.is_none()));
    }

    #[test]
    fn with_style_replaces_style() {
        let node = Node::shape(ShapeKind::Circle, None);
        assert_eq!(node.style.opacity, 1.0);

        let mut custom = Style::default();
        custom.opacity = 0.5;
        custom.rotate_deg = 45.0;
        let node = node.with_style(custom);
        assert_eq!(node.style.opacity, 0.5);
        assert_eq!(node.style.rotate_deg, 45.0);
    }

    #[test]
    fn content_debug_shape() {
        let c = Content::Shape {
            kind: ShapeKind::Circle,
            fill: None,
            stroke: None,
        };
        let dbg = format!("{:?}", c);
        assert!(dbg.contains("Shape"));
        assert!(dbg.contains("Circle"));
    }

    #[test]
    fn content_debug_group() {
        let c = Content::Group(vec![]);
        let dbg = format!("{:?}", c);
        assert!(dbg.contains("Group"));
    }

    #[test]
    fn content_debug_image() {
        let pixels = vec![0u8; 4];
        let image_data = new_image_data(pixels);
        let brush = Arc::new(ImageBrush::new(image_data));
        let c = Content::Image { image: brush, clip: Some(ShapeKind::Circle) };
        let dbg = format!("{:?}", c);
        assert!(dbg.contains("Image"));
        assert!(dbg.contains("clip"));
    }

    #[test]
    fn content_debug_text() {
        let rt = RichText::plain("debug me");
        let c = Content::Text(rt);
        let dbg = format!("{:?}", c);
        assert!(dbg.contains("Text"));
        assert!(dbg.contains("debug me"));
    }
}
