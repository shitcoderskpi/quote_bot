//! Shape kinds. A `ShapeKind` is unit-aware where it makes sense (corner
//! radii, circle radius) and turns into a `kurbo` geometry the vello `Scene`
//! can fill/stroke/clip with, once given a concrete box to sit in.

use crate::primitives::Viewport;
use vello::kurbo::{BezPath, Circle, Ellipse, Point, Rect, RoundedRect, RoundedRectRadii, Shape};

#[derive(Clone, Copy, Debug, Default)]
pub struct Corners {
    pub top_left: f64,
    pub top_right: f64,
    pub bottom_right: f64,
    pub bottom_left: f64,
}

impl Corners {
    pub fn zero() -> Self {
        Self { top_left: 0.0, top_right: 0.0, bottom_right: 0.0, bottom_left: 0.0 }
    }
    pub fn all(v: f64) -> Self {
        Self { top_left: v, top_right: v, bottom_right: v, bottom_left: v }
    }
}

#[derive(Clone, Debug)]
pub enum ShapeKind {
    /// A rounded rectangle; radii can be 0.
    Rect { corners: Corners },
    /// A circle.
    Circle,
    /// Ellipse filling the box (rx = w/2, ry = h/2 by default).
    Ellipse,
    /// An arbitrary SVG-style path data string (e.g. "M 0 0 L 10 10 Z").
    Path { data: String },
    /// Intersect with another shape.
    Clip(Box<ShapeKind>),
}

impl ShapeKind {
    pub fn to_kurbo(&self, w: f64, h: f64, _viewport: Viewport, _font_size: f64) -> BezPath {
        match self {
            ShapeKind::Rect { corners } => {
                let rect = Rect::new(0.0, 0.0, w, h);
                let radii = RoundedRectRadii::new(
                    corners.top_left,
                    corners.top_right,
                    corners.bottom_right,
                    corners.bottom_left,
                );
                RoundedRect::from_rect(rect, radii).to_path(0.1)
            }
            ShapeKind::Circle => {
                let r = w.min(h) / 2.0;
                Circle::new((w / 2.0, h / 2.0), r).to_path(0.1)
            }
            ShapeKind::Ellipse => {
                Ellipse::new((w / 2.0, h / 2.0), (w / 2.0, h / 2.0), 0.0).to_path(0.1)
            }
            ShapeKind::Path { data } => {
                if let Ok(path) = BezPath::from_svg(data) {
                    path
                } else {
                    Rect::new(0.0, 0.0, w, h).to_path(0.1)
                }
            }
            ShapeKind::Clip(inner) => inner.to_kurbo(w, h, _viewport, _font_size),
        }
    }
}
