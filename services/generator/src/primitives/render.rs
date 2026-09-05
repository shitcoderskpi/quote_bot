use crate::primitives::node::{Content, Node};
use crate::primitives::Viewport;
use parley::{FontContext, LayoutContext};
use vello::kurbo::{Affine, Rect as KRect, Shape, Stroke as KStroke};
use vello::peniko::{BlendMode, Brush, Fill, ImageBrush};
use vello::Scene;
use taffy::prelude::*;
use parley::layout::{PositionedLayoutItem, Glyph as ParleyGlyph};
use vello::Glyph;

pub struct Renderer {
    font_cx: FontContext,
    layout_cx: LayoutContext<Brush>,
    taffy: TaffyTree<Content>,
    root_id: Option<NodeId>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            font_cx: FontContext::new(),
            layout_cx: LayoutContext::new(),
            taffy: TaffyTree::new(),
            root_id: None,
        }
    }

    pub fn compute_layout(&mut self, node: &Node, viewport: Viewport, _font_size: f64) -> (f64, f64) {
        self.taffy.clear();
        let root_id = self.build_taffy_tree(node, viewport);
        self.root_id = Some(root_id);
        
        let font_cx = &mut self.font_cx;
        let layout_cx = &mut self.layout_cx;
        
        self.taffy.compute_layout_with_measure(
            root_id,
            Size { width: AvailableSpace::Definite(1000.0), height: AvailableSpace::Definite(1000.0) },
            |known_dims, avail_space, _id, ctx, _tree| {
                if let Some(Content::Text(rich)) = ctx {
                    let max_width = known_dims.width.map(|w| w as f64).or_else(|| {
                        if let AvailableSpace::Definite(w) = avail_space.width {
                            Some(w as f64)
                        } else {
                            None
                        }
                    });
                    
                    let layout = rich.layout(font_cx, layout_cx, max_width, viewport);
                    Size { width: layout.width(), height: layout.height() }
                } else if let Some(Content::Image { image, .. }) = ctx {
                    let iw = image.image.width as f32;
                    let ih = image.image.height as f32;
                    Size { width: iw, height: ih }
                } else {
                    Size::ZERO
                }
            }
        ).unwrap();
        
        let layout = self.taffy.layout(root_id).unwrap();
        (layout.size.width as f64, layout.size.height as f64)
    }

    pub fn render(&mut self, scene: &mut Scene, node: &Node, _root_box: KRect, viewport: Viewport) {
        if let Some(root_id) = self.root_id {
            self.render_taffy_tree(scene, root_id, node, Affine::IDENTITY, viewport, 16.0);
        }
    }

    fn build_taffy_tree(&mut self, node: &Node, viewport: Viewport) -> NodeId {
        let style = node.style.layout.clone();
        
        match &node.content {
            Content::Group(children) => {
                let mut child_ids = Vec::with_capacity(children.len());
                for c in children {
                    child_ids.push(self.build_taffy_tree(c, viewport));
                }
                self.taffy.new_with_children(style, &child_ids).unwrap()
            }
            Content::Text(_) => {
                self.taffy.new_leaf_with_context(style, node.content.clone()).unwrap()
            }
            Content::Image { .. } => {
                self.taffy.new_leaf_with_context(style, node.content.clone()).unwrap()
            }
            _ => {
                self.taffy.new_leaf(style).unwrap()
            }
        }
    }

    fn render_taffy_tree(&mut self, scene: &mut Scene, id: NodeId, node: &Node, parent_transform: Affine, viewport: Viewport, font_size: f64) {
        let layout = self.taffy.layout(id).unwrap();
        
        let x = layout.location.x as f64;
        let y = layout.location.y as f64;
        let w = layout.size.width as f64;
        let h = layout.size.height as f64;
        
        let center = Affine::translate((x + w / 2.0, y + h / 2.0));
        let rotate = Affine::rotate(node.style.rotate_deg.to_radians());
        let local = center * rotate * Affine::translate((-w / 2.0, -h / 2.0));
        let transform = parent_transform * local;
        
        let needs_layer = node.style.clip.is_some() || node.style.opacity < 1.0;
        if needs_layer {
            let clip_shape = node.style.clip.as_ref().map(|c| c.to_kurbo(w, h, viewport, font_size)).unwrap_or_else(|| KRect::new(0.0, 0.0, w, h).into_path(0.1));
            scene.push_layer(Fill::NonZero, BlendMode::default(), node.style.opacity, transform, &clip_shape);
        }
        
        match &node.content {
            Content::Shape { kind, fill, stroke } => {
                let path = kind.to_kurbo(w, h, viewport, font_size);
                if let Some(paint) = fill {
                    let brush = paint.to_brush(w, h, viewport, font_size);
                    scene.fill(Fill::NonZero, transform, &brush, None, &path);
                }
                if let Some(s) = stroke {
                    let kstroke = KStroke::new(s.width);
                    scene.stroke(&kstroke, transform, s.color, None, &path);
                }
            }
            Content::Text(rich) => {
                let text_layout = rich.layout(&mut self.font_cx, &mut self.layout_cx, Some(w), viewport);
                draw_text_layout(scene, transform, &text_layout);
            }
            Content::Image { image, clip } => {
                let clip_shape = clip.as_ref().map(|c| c.to_kurbo(w, h, viewport, font_size)).unwrap_or_else(|| KRect::new(0.0, 0.0, w, h).into_path(0.1));
                let brush_transform = cover_fit_transform(image, w, h);
                scene.fill(Fill::NonZero, transform, &Brush::Image(image.as_ref().clone()), Some(brush_transform), &clip_shape);
            }
            Content::Group(children) => {
                let child_ids = self.taffy.children(id).unwrap();
                for (child, child_id) in children.iter().zip(child_ids.iter()) {
                    self.render_taffy_tree(scene, *child_id, child, transform, viewport, font_size);
                }
            }
        }
        
        if needs_layer {
            scene.pop_layer();
        }
    }
}

fn cover_fit_transform(image: &ImageBrush, box_w: f64, box_h: f64) -> Affine {
    let iw = image.image.width.max(1) as f64;
    let ih = image.image.height.max(1) as f64;
    let scale = (box_w / iw).max(box_h / ih);
    let scaled_w = iw * scale;
    let scaled_h = ih * scale;
    let dx = (box_w - scaled_w) / 2.0;
    let dy = (box_h - scaled_h) / 2.0;
    Affine::translate((dx, dy)) * Affine::scale(scale)
}

fn draw_text_layout(scene: &mut Scene, transform: Affine, layout: &parley::Layout<Brush>) {
    for line in layout.lines() {
        for item in line.items() {
            if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                let mut x = glyph_run.offset() as f64;
                let y = glyph_run.baseline() as f64;
                let run = glyph_run.run();
                let font = run.font();
                let font_size = run.font_size();
                let coords = run.normalized_coords();

                scene
                    .draw_glyphs(font)
                    .brush(&glyph_run.style().brush)
                    .hint(true)
                    .transform(transform)
                    .font_size(font_size)
                    .normalized_coords(coords)
                    .draw(
                        Fill::NonZero,
                        glyph_run.glyphs().map(|g: ParleyGlyph| {
                            let gx = x + g.x as f64;
                            let gy = y - g.y as f64;
                            x += g.advance as f64;
                            Glyph { id: g.id, x: gx as f32, y: gy as f32 }
                        }),
                    );
                    
                let style = glyph_run.style();
                let run_width = glyph_run.advance();
                
                if style.underline.is_some() {
                    let underline = style.underline.as_ref().unwrap();
                    let offset = underline.offset.unwrap_or(run.metrics().underline_offset) as f64;
                    let size = underline.size.unwrap_or(run.metrics().underline_size) as f64;
                    let y_pos = y - offset;
                    let start_x = glyph_run.offset() as f64;
                    let rect = vello::kurbo::Rect::new(start_x, y_pos - size / 2.0, start_x + run_width as f64, y_pos + size / 2.0);
                    let brush = &underline.brush;
                    scene.fill(Fill::NonZero, transform, brush, None, &rect);
                }
                
                if style.strikethrough.is_some() {
                    let strikethrough = style.strikethrough.as_ref().unwrap();
                    let offset = strikethrough.offset.unwrap_or(run.metrics().strikethrough_offset) as f64;
                    let size = strikethrough.size.unwrap_or(run.metrics().strikethrough_size) as f64;
                    let y_pos = y - offset;
                    let start_x = glyph_run.offset() as f64;
                    let rect = vello::kurbo::Rect::new(start_x, y_pos - size / 2.0, start_x + run_width as f64, y_pos + size / 2.0);
                    let brush = &strikethrough.brush;
                    scene.fill(Fill::NonZero, transform, brush, None, &rect);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::node::{Content, Node, Style};
    use crate::primitives::paint::Paint;
    use crate::primitives::shape::{Corners, ShapeKind};
    use crate::primitives::text::RichText;
    use crate::primitives::Viewport;
    use std::sync::Arc;
    use vello::kurbo::Affine;

    fn vp() -> Viewport {
        Viewport { width: 1.0, height: 1.0 }
    }

    fn make_image_brush(w: u32, h: u32) -> ImageBrush {
        let pixels = vec![0u8; (w * h * 4) as usize];
        ImageBrush::new(vello::peniko::ImageData {
            data: vello::peniko::Blob::new(Arc::new(pixels)),
            format: vello::peniko::ImageFormat::Rgba8,
            alpha_type: vello::peniko::ImageAlphaType::Alpha,
            width: w,
            height: h,
        })
    }

    fn shaped_node(w: f32, h: f32) -> Node {
        let mut style = Style::default();
        style.layout.size.width = taffy::Dimension::length(w);
        style.layout.size.height = taffy::Dimension::length(h);
        Node {
            style,
            content: Content::Shape {
                kind: ShapeKind::Rect { corners: Corners::zero() },
                fill: Some(Paint::solid(vello::peniko::Color::BLACK)),
                stroke: None,
            },
        }
    }

    #[test]
    fn cover_fit_exact() {
        let img = make_image_brush(100, 100);
        let t = cover_fit_transform(&img, 100.0, 100.0);
        let c = t.as_coeffs();
        assert!((c[0] - 1.0).abs() < 0.001, "scale_x = {}", c[0]);
        assert!((c[4]).abs() < 0.001, "dx = {}", c[4]);
        assert!((c[5]).abs() < 0.001, "dy = {}", c[5]);
    }

    #[test]
    fn cover_fit_wide_image() {
        let img = make_image_brush(200, 100);
        let t = cover_fit_transform(&img, 100.0, 100.0);
        let c = t.as_coeffs();
        assert!((c[0] - 1.0).abs() < 0.001, "scale = {}", c[0]);
        assert!((c[4] - (-50.0)).abs() < 0.001, "dx = {}", c[4]);
        assert!((c[5]).abs() < 0.001, "dy = {}", c[5]);
    }

    #[test]
    fn cover_fit_tall_image() {
        let img = make_image_brush(100, 200);
        let t = cover_fit_transform(&img, 100.0, 100.0);
        let c = t.as_coeffs();
        assert!((c[0] - 1.0).abs() < 0.001, "scale = {}", c[0]);
        assert!((c[4]).abs() < 0.001, "dx = {}", c[4]);
        assert!((c[5] - (-50.0)).abs() < 0.001, "dy = {}", c[5]);
    }

    #[test]
    fn cover_fit_small_image_upscales() {
        let img = make_image_brush(50, 50);
        let t = cover_fit_transform(&img, 100.0, 100.0);
        let c = t.as_coeffs();
        assert!((c[0] - 2.0).abs() < 0.001, "scale = {}", c[0]);
        assert!((c[4]).abs() < 0.001, "dx = {}", c[4]);
        assert!((c[5]).abs() < 0.001, "dy = {}", c[5]);
    }

    #[test]
    fn cover_fit_zero_image_uses_max1() {
        let img = make_image_brush(0, 0);
        let t = cover_fit_transform(&img, 100.0, 100.0);
        let c = t.as_coeffs();
        assert!((c[0] - 100.0).abs() < 0.001, "scale = {}", c[0]);
    }

    #[test]
    fn compute_layout_explicit_size() {
        let node = shaped_node(120.0, 80.0);
        let mut renderer = Renderer::new();
        let (w, h) = renderer.compute_layout(&node, vp(), 16.0);
        assert!((w - 120.0).abs() < 0.1, "w = {}", w);
        assert!((h - 80.0).abs() < 0.1, "h = {}", h);
    }

    #[test]
    fn compute_layout_group_column() {
        let mut group_style = Style::default();
        group_style.layout.flex_direction = taffy::FlexDirection::Column;

        let group = Node {
            style: group_style,
            content: Content::Group(vec![
                shaped_node(100.0, 50.0),
                shaped_node(100.0, 50.0),
            ]),
        };
        let mut renderer = Renderer::new();
        let (w, h) = renderer.compute_layout(&group, vp(), 16.0);
        assert!((w - 100.0).abs() < 1.0, "w = {}", w);
        assert!((h - 100.0).abs() < 1.0, "h = {}", h);
    }

    #[test]
    fn compute_layout_group_row() {
        let mut group_style = Style::default();
        group_style.layout.flex_direction = taffy::FlexDirection::Row;

        let group = Node {
            style: group_style,
            content: Content::Group(vec![
                shaped_node(50.0, 100.0),
                shaped_node(50.0, 100.0),
            ]),
        };
        let mut renderer = Renderer::new();
        let (w, h) = renderer.compute_layout(&group, vp(), 16.0);
        assert!((w - 100.0).abs() < 1.0, "w = {}", w);
        assert!((h - 100.0).abs() < 1.0, "h = {}", h);
    }

    #[test]
    fn compute_layout_nested_groups() {
        let inner = Node {
            style: Style::default(),
            content: Content::Group(vec![shaped_node(60.0, 40.0)]),
        };
        let outer = Node {
            style: Style::default(),
            content: Content::Group(vec![inner]),
        };
        let mut renderer = Renderer::new();
        let (w, h) = renderer.compute_layout(&outer, vp(), 16.0);
        assert!((w - 60.0).abs() < 1.0, "w = {}", w);
        assert!((h - 40.0).abs() < 1.0, "h = {}", h);
    }

    #[test]
    fn compute_layout_with_padding() {
        let mut style = Style::default();
        style.layout.size.width = taffy::Dimension::length(100.0);
        style.layout.size.height = taffy::Dimension::length(100.0);
        style.layout.padding = taffy::Rect {
            top: taffy::LengthPercentage::length(10.0),
            right: taffy::LengthPercentage::length(10.0),
            bottom: taffy::LengthPercentage::length(10.0),
            left: taffy::LengthPercentage::length(10.0),
        };
        let node = Node {
            style,
            content: Content::Group(vec![]),
        };
        let mut renderer = Renderer::new();
        let (w, h) = renderer.compute_layout(&node, vp(), 16.0);
        assert!((w - 100.0).abs() < 0.1, "w = {}", w);
        assert!((h - 100.0).abs() < 0.1, "h = {}", h);
    }

    #[test]
    fn compute_layout_text_node() {
        let rt = RichText::plain("Hello World");
        let node = Node::text(rt);
        let mut renderer = Renderer::new();
        let (w, h) = renderer.compute_layout(&node, vp(), 16.0);
        assert!(w > 0.0, "text width should be > 0, got {}", w);
        assert!(h > 0.0, "text height should be > 0, got {}", h);
    }

    #[test]
    fn render_does_not_panic_with_valid_tree() {
        let node = shaped_node(100.0, 50.0);
        let mut renderer = Renderer::new();
        let (w, h) = renderer.compute_layout(&node, vp(), 16.0);

        let mut scene = vello::Scene::new();
        let root_box = KRect::new(0.0, 0.0, w, h);
        renderer.render(&mut scene, &node, root_box, vp());
    }

    #[test]
    fn render_group_with_mixed_content() {
        let mut style = Style::default();
        style.layout.flex_direction = taffy::FlexDirection::Column;
        let group = Node {
            style,
            content: Content::Group(vec![
                shaped_node(100.0, 30.0),
                Node::text(RichText::plain("test")),
                shaped_node(100.0, 30.0),
            ]),
        };
        let mut renderer = Renderer::new();
        let (w, h) = renderer.compute_layout(&group, vp(), 16.0);
        let mut scene = vello::Scene::new();
        let root_box = KRect::new(0.0, 0.0, w, h);
        renderer.render(&mut scene, &group, root_box, vp());
    }
}
