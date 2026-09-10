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
    Rect { corners: Corners },
    Circle,
    Ellipse,
    Path { data: String },
    Clip(Box<ShapeKind>),
}

impl ShapeKind {
    pub fn to_kurbo(&self, w: f64, h: f64) -> BezPath {
        const EPSILON: f64 = 0.1;
        match self {
            ShapeKind::Rect { corners } => {
                let rect = Rect::new(0.0, 0.0, w, h);
                let radii = RoundedRectRadii::new(
                    corners.top_left,
                    corners.top_right,
                    corners.bottom_right,
                    corners.bottom_left,
                );
                RoundedRect::from_rect(rect, radii).to_path(EPSILON)
            }
            ShapeKind::Circle => {
                let r = w.min(h) / 2.0;
                Circle::new((w / 2.0, h / 2.0), r).to_path(EPSILON)
            }
            ShapeKind::Ellipse => {
                Ellipse::new((w / 2.0, h / 2.0), (w / 2.0, h / 2.0), 0.0).to_path(EPSILON)
            }
            ShapeKind::Path { data } => {
                if let Ok(path) = BezPath::from_svg(data) {
                    path
                } else {
                    Rect::new(0.0, 0.0, w, h).to_path(EPSILON)
                }
            }
            ShapeKind::Clip(inner) => inner.to_kurbo(w, h),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn epsilon() -> f64 {
        0.0001
    }

    #[test]
    fn corners_zero() {
        let c = Corners::zero();
        assert_eq!(c.top_left, 0.0);
        assert_eq!(c.top_right, 0.0);
        assert_eq!(c.bottom_right, 0.0);
        assert_eq!(c.bottom_left, 0.0);
    }

    #[test]
    fn corners_all() {
        let c = Corners::all(12.5);
        assert_eq!(c.top_left, 12.5);
        assert_eq!(c.top_right, 12.5);
        assert_eq!(c.bottom_right, 12.5);
        assert_eq!(c.bottom_left, 12.5);
    }

    #[test]
    fn corners_default_is_zero() {
        let c = Corners::default();
        assert_eq!(c.top_left, 0.0);
        assert_eq!(c.bottom_right, 0.0);
    }

    #[test]
    fn rect_zero_corners_bounding_box() {
        let shape = ShapeKind::Rect { corners: Corners::zero() };
        let path = shape.to_kurbo(100.0, 50.0);
        let bb = path.bounding_box();
        assert!((bb.width() - 100.0).abs() < epsilon());
        assert!((bb.height() - 50.0).abs() < epsilon());
    }

    #[test]
    fn rect_rounded_corners_bounding_box() {
        let shape = ShapeKind::Rect { corners: Corners::all(10.0) };
        let path = shape.to_kurbo(200.0, 80.0);
        let bb = path.bounding_box();
        assert!((bb.width() - 200.0).abs() < epsilon());
        assert!((bb.height() - 80.0).abs() < epsilon());
    }

    #[test]
    fn rect_asymmetric_corners() {
        let corners = Corners {
            top_left: 5.0,
            top_right: 10.0,
            bottom_right: 15.0,
            bottom_left: 0.0,
        };
        let shape = ShapeKind::Rect { corners };
        let path = shape.to_kurbo(100.0, 100.0);
        let bb = path.bounding_box();
        assert!((bb.width() - 100.0).abs() < epsilon());
        assert!((bb.height() - 100.0).abs() < epsilon());
    }

    #[test]
    fn circle_square_box() {
        let path = ShapeKind::Circle.to_kurbo(100.0, 100.0);
        let bb = path.bounding_box();
        assert!((bb.width() - 100.0).abs() < epsilon(), "width={}", bb.width());
        assert!((bb.height() - 100.0).abs() < epsilon(), "height={}", bb.height());
    }

    #[test]
    fn circle_rectangular_box_uses_min_dim() {
        let path = ShapeKind::Circle.to_kurbo(200.0, 100.0);
        let bb = path.bounding_box();
        assert!((bb.width() - 100.0).abs() < epsilon());
        assert!((bb.height() - 100.0).abs() < epsilon());
    }

    #[test]
    fn ellipse_fills_box() {
        let path = ShapeKind::Ellipse.to_kurbo(200.0, 100.0);
        let bb = path.bounding_box();
        assert!((bb.width() - 200.0).abs() < epsilon());
        assert!((bb.height() - 100.0).abs() < epsilon());
    }

    #[test]
    fn svg_path_valid() {
        let shape = ShapeKind::Path {
            data: "M 0 0 L 80 0 L 80 40 L 0 40 Z".to_string(),
        };
        let path = shape.to_kurbo(200.0, 200.0);
        let bb = path.bounding_box();
        assert!((bb.width() - 80.0).abs() < epsilon());
        assert!((bb.height() - 40.0).abs() < epsilon());
    }

    #[test]
    fn svg_path_invalid_falls_back_to_rect() {
        let shape = ShapeKind::Path {
            data: "not-a-path".to_string(),
        };
        let path = shape.to_kurbo(100.0, 50.0);
        let bb = path.bounding_box();
        assert!((bb.width() - 100.0).abs() < epsilon());
        assert!((bb.height() - 50.0).abs() < epsilon());
    }

    #[test]
    fn clip_delegates_to_inner() {
        let inner = ShapeKind::Circle;
        let clip = ShapeKind::Clip(Box::new(inner.clone()));
        let direct = inner.to_kurbo(100.0, 100.0);
        let via_clip = clip.to_kurbo(100.0, 100.0);
        let bb1 = direct.bounding_box();
        let bb2 = via_clip.bounding_box();
        assert!((bb1.width() - bb2.width()).abs() < epsilon());
        assert!((bb1.height() - bb2.height()).abs() < epsilon());
    }
}
