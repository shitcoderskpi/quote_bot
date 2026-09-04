use crate::primitives::Viewport;
use vello::peniko::{Brush, Color, ColorStop, ColorStops, Extend, Gradient};

#[derive(Debug, Clone)]
pub struct Stop {
    pub offset: f32,
    pub color: Color,
}

#[derive(Debug, Clone)]
pub enum Paint {
    Solid(Color),
    LinearGradient {
        start: (f64, f64),
        end: (f64, f64),
        stops: Vec<Stop>,
        extend: Extend,
    },
    RadialGradient {
        center: (f64, f64),
        radius: f64,
        stops: Vec<Stop>,
        extend: Extend,
    },
    SweepGradient {
        center: (f64, f64),
        start_angle: f32,
        end_angle: f32,
        stops: Vec<Stop>,
        extend: Extend,
    },
}

impl Paint {
    pub fn solid(color: Color) -> Self {
        Paint::Solid(color)
    }
    fn build_stops(stops: &[Stop]) -> ColorStops {
        stops
            .iter()
            .map(|s| ColorStop { offset: s.offset, color: s.color.into() })
            .collect::<Vec<_>>()
            .as_slice()
            .into()
    }
    
    pub fn to_brush(&self, box_w: f64, box_h: f64, _viewport: Viewport, _font_size: f64) -> Brush {
        match self {
            Paint::Solid(c) => Brush::Solid(*c),
            Paint::LinearGradient { start, end, stops, extend } => {
                let (x0, y0) = (start.0 * box_w, start.1 * box_h);
                let (x1, y1) = (end.0 * box_w, end.1 * box_h);
                let mut g = Gradient::new_linear((x0, y0), (x1, y1));
                g.stops = Self::build_stops(stops);
                g.extend = *extend;
                Brush::Gradient(g)
            }
            Paint::RadialGradient { center, radius, stops, extend } => {
                let (cx, cy) = (center.0 * box_w, center.1 * box_h);
                let r = (radius * box_w.min(box_h)) as f32;
                let mut g = Gradient::new_radial((cx, cy), r);
                g.stops = Self::build_stops(stops);
                g.extend = *extend;
                Brush::Gradient(g)
            }
            Paint::SweepGradient { center, start_angle, end_angle, stops, extend } => {
                let (cx, cy) = (center.0 * box_w, center.1 * box_h);
                let mut g = Gradient::new_sweep((cx, cy), *start_angle, *end_angle);
                g.stops = Self::build_stops(stops);
                g.extend = *extend;
                Brush::Gradient(g)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Stroke {
    pub width: f64,
    pub color: Color,
}
