use crate::primitives::node::{Content, Node};
use crate::primitives::Viewport;
use parley::{FontContext, LayoutContext};
use vello::kurbo::{Affine, Rect as KRect, Shape, Stroke as KStroke};
use vello::peniko::{BlendMode, Brush, Fill, ImageBrush};
use vello::Scene;
use std::sync::{Arc, Mutex};
use taffy::prelude::*;
use parley::layout::{PositionedLayoutItem, Glyph as ParleyGlyph};
use vello::Glyph;

pub struct Renderer {
    font_cx: Arc<Mutex<FontContext>>,
    layout_cx: Arc<Mutex<LayoutContext<Brush>>>,
    taffy: TaffyTree<Content>,
    root_id: Option<NodeId>,
}

impl Renderer {
    pub fn new() -> Self {
        Self {
            font_cx: Arc::new(Mutex::new(FontContext::new())),
            layout_cx: Arc::new(Mutex::new(LayoutContext::new())),
            taffy: TaffyTree::new(),
            root_id: None,
        }
    }

    pub fn compute_layout(&mut self, node: &Node, viewport: Viewport, _font_size: f64) -> (f64, f64) {
        self.taffy.clear();
        let root_id = self.build_taffy_tree(node, viewport);
        self.root_id = Some(root_id);
        
        let font_cx = self.font_cx.clone();
        let layout_cx = self.layout_cx.clone();
        
        self.taffy.compute_layout_with_measure(
            root_id,
            Size { width: AvailableSpace::Definite(1000.0), height: AvailableSpace::Definite(1000.0) },
            |known_dims, avail_space, _id, ctx, _tree| {
                if let Some(Content::Text(rich)) = ctx {
                    let mut fcx = font_cx.lock().unwrap();
                    let mut lcx = layout_cx.lock().unwrap();
                    let max_width = known_dims.width.map(|w| w as f64).or_else(|| {
                        if let AvailableSpace::Definite(w) = avail_space.width {
                            Some(w as f64)
                        } else {
                            None
                        }
                    });
                    
                    let layout = rich.layout(&mut fcx, &mut lcx, max_width, viewport);
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

    fn render_taffy_tree(&self, scene: &mut Scene, id: NodeId, node: &Node, parent_transform: Affine, viewport: Viewport, font_size: f64) {
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
                let mut fcx = self.font_cx.lock().unwrap();
                let mut lcx = self.layout_cx.lock().unwrap();
                let text_layout = rich.layout(&mut fcx, &mut lcx, Some(w), viewport);
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
