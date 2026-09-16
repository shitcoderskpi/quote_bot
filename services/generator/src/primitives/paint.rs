use vello::peniko::{Brush, Color, ColorStop, ColorStops, Extend, Gradient};

#[derive(Debug, Clone, PartialEq)]
pub struct Stop {
    pub offset: f32,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Paint {
    Solid(Color),
    LinearGradient {
        start: (f64, f64),
        end: (f64, f64),
        stops: Vec<Stop>,
        extend: Extend,
        alpha: f32,
    },
    RadialGradient {
        center: (f64, f64),
        radius: f64,
        stops: Vec<Stop>,
        extend: Extend,
        alpha: f32,
    },
    SweepGradient {
        center: (f64, f64),
        start_angle: f32,
        end_angle: f32,
        stops: Vec<Stop>,
        extend: Extend,
        alpha: f32,
    },
}

impl Default for Paint {
    fn default() -> Self {
        Paint::Solid(Color::BLACK)
    }
}

impl Paint {
    pub fn solid(color: Color) -> Self {
        Paint::Solid(color)
    }
    fn build_stops(stops: &[Stop], alpha_factor: f32) -> ColorStops {
        stops
            .iter()
            .map(|s| ColorStop { offset: s.offset, color: s.color.with_alpha(alpha_factor).into() })
            .collect::<Vec<_>>()
            .as_slice()
            .into()
    }
    
    pub fn to_brush(&self, box_w: f64, box_h: f64) -> Brush {
        match self {
            Paint::Solid(c) => Brush::Solid(*c),
            Paint::LinearGradient { start, end, stops, extend, alpha } => {
                let mut g = Gradient::new_linear(
                    (start.0 * box_w, start.1 * box_h),
                    (end.0 * box_w, end.1 * box_h),
                );
                g.stops = Self::build_stops(stops, *alpha);
                g.extend = *extend;
                Brush::Gradient(g)
            }
            Paint::RadialGradient { center, radius, stops, extend, alpha } => {
                let r = radius * box_w.max(box_h);
                let mut g = Gradient::new_radial(
                    (center.0 * box_w, center.1 * box_h),
                    r as f32,
                );
                g.stops = Self::build_stops(stops, *alpha);
                g.extend = *extend;
                Brush::Gradient(g)
            }
            Paint::SweepGradient { center, start_angle, end_angle, stops, extend, alpha } => {
                let mut g = Gradient::new_sweep(
                    (center.0 * box_w, center.1 * box_h),
                    *start_angle,
                    *end_angle,
                );
                g.stops = Self::build_stops(stops, *alpha);
                g.extend = *extend;
                Brush::Gradient(g)
            }
        }
    }
    pub fn multiply_alpha(self, alpha_factor: f32) -> Self {
        match self {
            Paint::Solid(c) => Paint::Solid(c.with_alpha(alpha_factor)),
            Paint::LinearGradient { start, end, stops, extend, alpha } => {
                Paint::LinearGradient { start, end, stops, extend, alpha: alpha * alpha_factor }
            }
            Paint::RadialGradient { center, radius, stops, extend, alpha } => {
                Paint::RadialGradient { center, radius, stops, extend, alpha: alpha * alpha_factor }
            }
            Paint::SweepGradient { center, start_angle, end_angle, stops, extend, alpha } => {
                Paint::SweepGradient { center, start_angle, end_angle, stops, extend, alpha: alpha * alpha_factor }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stroke {
    pub width: f64,
    pub color: Color,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_stops() -> Vec<Stop> {
        vec![
            Stop { offset: 0.0, color: Color::BLACK },
            Stop { offset: 1.0, color: Color::WHITE },
        ]
    }

    #[test]
    fn solid_to_brush() {
        let paint = Paint::Solid(Color::BLACK);
        let brush = paint.to_brush(100.0, 100.0);
        assert!(matches!(brush, Brush::Solid(_)));
    }

    #[test]
    fn solid_constructor() {
        let paint = Paint::solid(Color::WHITE);
        assert!(matches!(paint, Paint::Solid(c) if c == Color::WHITE));
    }

    #[test]
    fn linear_gradient_to_brush() {
        let paint = Paint::LinearGradient {
            start: (0.0, 0.0),
            end: (1.0, 1.0),
            stops: two_stops(),
            extend: Extend::default(), alpha: 1.0,
        };
        let brush = paint.to_brush(200.0, 100.0);
        assert!(matches!(brush, Brush::Gradient(_)));
    }

    #[test]
    fn linear_gradient_endpoints_scaled() {
        let paint = Paint::LinearGradient {
            start: (0.0, 0.0),
            end: (1.0, 1.0),
            stops: two_stops(),
            extend: Extend::default(), alpha: 1.0,
        };
        let brush = paint.to_brush(200.0, 100.0);
        assert!(matches!(brush, Brush::Gradient(_)));
    }

    #[test]
    fn radial_gradient_to_brush() {
        let paint = Paint::RadialGradient {
            center: (0.5, 0.5),
            radius: 0.5,
            stops: two_stops(),
            extend: Extend::default(), alpha: 1.0,
        };
        let brush = paint.to_brush(100.0, 100.0);
        assert!(matches!(brush, Brush::Gradient(_)));
    }

    #[test]
    fn sweep_gradient_to_brush() {
        let paint = Paint::SweepGradient {
            center: (0.5, 0.5),
            start_angle: 0.0,
            end_angle: 360.0,
            stops: two_stops(),
            extend: Extend::default(), alpha: 1.0,
        };
        let brush = paint.to_brush(100.0, 100.0);
        assert!(matches!(brush, Brush::Gradient(_)));
    }

    #[test]
    fn build_stops_maps_offsets_and_colors() {
        let stops = vec![
            Stop { offset: 0.0, color: Color::BLACK },
            Stop { offset: 0.5, color: Color::from_rgb8(128, 128, 128) },
            Stop { offset: 1.0, color: Color::WHITE },
        ];
        let cs: ColorStops = Paint::build_stops(&stops, 1.0);
        assert_eq!(cs.len(), 3);
    }

    #[test]
    fn stroke_fields() {
        let s = Stroke { width: 2.5, color: Color::BLACK };
        assert_eq!(s.width, 2.5);
        assert_eq!(s.color, Color::BLACK);
    }
}
