use crate::primitives::node::{Content, Node, Style};
use crate::primitives::paint::Paint;
use crate::primitives::shape::{Corners, ShapeKind};
use crate::primitives::text::RichText;
use steel::rvals::{Custom, FromSteelVal, SteelVal};
use taffy::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct SchemeNumber(pub f64);

impl FromSteelVal for SchemeNumber {
    fn from_steelval(val: &SteelVal) -> steel::rvals::Result<Self> {
        match val {
            SteelVal::IntV(i) => Ok(SchemeNumber(*i as f64)),
            SteelVal::NumV(f) => Ok(SchemeNumber(*f)),
            _ => Err(steel::SteelErr::new(
                steel::rerrs::ErrorKind::ConversionError,
                format!("expected number, found: {:?}", val)
            )),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SchemeDimension {
    Length(f32),
    Percent(f32),
    Auto,
}

impl Custom for SchemeDimension {}

impl SchemeDimension {
    pub fn to_dimension(&self) -> Dimension {
        match self {
            SchemeDimension::Length(v) => Dimension::length(*v),
            SchemeDimension::Percent(v) => Dimension::percent(*v),
            SchemeDimension::Auto => Dimension::auto(),
        }
    }

    pub fn to_length_percentage(&self) -> LengthPercentage {
        match self {
            SchemeDimension::Length(v) => LengthPercentage::length(*v),
            SchemeDimension::Percent(v) => LengthPercentage::percent(*v),
            SchemeDimension::Auto => LengthPercentage::length(0.0),
        }
    }

    pub fn to_length_percentage_auto(&self) -> LengthPercentageAuto {
        match self {
            SchemeDimension::Length(v) => LengthPercentageAuto::length(*v),
            SchemeDimension::Percent(v) => LengthPercentageAuto::percent(*v),
            SchemeDimension::Auto => LengthPercentageAuto::auto(),
        }
    }

    pub fn to_f64(&self) -> f64 {
        match self {
            SchemeDimension::Length(v) => *v as f64,
            SchemeDimension::Percent(v) => *v as f64 * 100.0,
            SchemeDimension::Auto => 0.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SchemeColor(pub vello::peniko::Color);
impl Custom for SchemeColor {}

#[derive(Clone, Debug)]
pub struct SchemePaint(pub Paint);
impl Custom for SchemePaint {}

#[derive(Clone, Debug)]
pub struct SchemeAngle(pub f64);
impl Custom for SchemeAngle {}

#[derive(Clone, Debug)]
pub struct SchemeStop {
    pub offset: f32,
    pub color: vello::peniko::Color,
}
impl Custom for SchemeStop {}

#[derive(Clone, Debug)]
pub struct SchemeShapeKind {
    pub kind: ShapeKind,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl SchemeShapeKind {
    pub fn new(kind: ShapeKind) -> Self {
        Self { kind, width: None, height: None }
    }

    pub fn with_size(kind: ShapeKind, w: f32, h: f32) -> Self {
        Self { kind, width: Some(w), height: Some(h) }
    }
}

impl Custom for SchemeShapeKind {}

#[derive(Clone, Debug)]
pub enum StyleMod {
    Direction(FlexDirection),
    AlignItems(Option<AlignItems>),
    JustifyContent(Option<JustifyContent>),
    Display(Display),
    Position(Position),
    Width(SchemeDimension),
    Height(SchemeDimension),
    MaxWidth(SchemeDimension),
    MaxHeight(SchemeDimension),
    MinWidth(SchemeDimension),
    MinHeight(SchemeDimension),
    PaddingUniform(SchemeDimension),
    PaddingXY(SchemeDimension, SchemeDimension),
    PaddingTRBL(SchemeDimension, SchemeDimension, SchemeDimension, SchemeDimension),
    MarginUniform(SchemeDimension),
    MarginSide { side: Side, dim: SchemeDimension },
    GapUniform(SchemeDimension),
    InsetAll(SchemeDimension, SchemeDimension, SchemeDimension, SchemeDimension),
    InsetSide { side: Side, dim: SchemeDimension },
    Opacity(f32),
    Rotate(f64),
}

#[derive(Clone, Debug)]
pub enum Side {
    Top,
    Right,
    Bottom,
    Left,
}

impl Custom for StyleMod {}

impl StyleMod {
    pub fn apply(&self, style: &mut Style) {
        match self {
            StyleMod::Direction(d) => style.layout.flex_direction = *d,
            StyleMod::AlignItems(a) => style.layout.align_items = *a,
            StyleMod::JustifyContent(j) => style.layout.justify_content = *j,
            StyleMod::Display(d) => style.layout.display = *d,
            StyleMod::Position(p) => style.layout.position = *p,
            StyleMod::Width(d) => style.layout.size.width = d.to_dimension(),
            StyleMod::Height(d) => style.layout.size.height = d.to_dimension(),
            StyleMod::MaxWidth(d) => style.layout.max_size.width = d.to_dimension(),
            StyleMod::MaxHeight(d) => style.layout.max_size.height = d.to_dimension(),
            StyleMod::MinWidth(d) => style.layout.min_size.width = d.to_dimension(),
            StyleMod::MinHeight(d) => style.layout.min_size.height = d.to_dimension(),
            StyleMod::PaddingUniform(d) => {
                let p = d.to_length_percentage();
                style.layout.padding = Rect { top: p, right: p, bottom: p, left: p };
            }
            StyleMod::PaddingXY(h, v) => {
                let hp = h.to_length_percentage();
                let vp = v.to_length_percentage();
                style.layout.padding = Rect { top: vp, right: hp, bottom: vp, left: hp };
            }
            StyleMod::PaddingTRBL(t, r, b, l) => {
                style.layout.padding = Rect {
                    top: t.to_length_percentage(),
                    right: r.to_length_percentage(),
                    bottom: b.to_length_percentage(),
                    left: l.to_length_percentage(),
                };
            }
            StyleMod::MarginUniform(d) => {
                let m = d.to_length_percentage_auto();
                style.layout.margin = Rect { top: m, right: m, bottom: m, left: m };
            }
            StyleMod::MarginSide { side, dim } => {
                let m = dim.to_length_percentage_auto();
                match side {
                    Side::Top => style.layout.margin.top = m,
                    Side::Right => style.layout.margin.right = m,
                    Side::Bottom => style.layout.margin.bottom = m,
                    Side::Left => style.layout.margin.left = m,
                }
            }
            StyleMod::GapUniform(d) => {
                let g = d.to_length_percentage();
                style.layout.gap = Size { width: g, height: g };
            }
            StyleMod::InsetAll(t, r, b, l) => {
                style.layout.inset = Rect {
                    top: t.to_length_percentage_auto(),
                    right: r.to_length_percentage_auto(),
                    bottom: b.to_length_percentage_auto(),
                    left: l.to_length_percentage_auto(),
                };
            }
            StyleMod::InsetSide { side, dim } => {
                let v = dim.to_length_percentage_auto();
                match side {
                    Side::Top => style.layout.inset.top = v,
                    Side::Right => style.layout.inset.right = v,
                    Side::Bottom => style.layout.inset.bottom = v,
                    Side::Left => style.layout.inset.left = v,
                }
            }
            StyleMod::Opacity(o) => style.opacity = *o,
            StyleMod::Rotate(r) => style.rotate_deg = *r,
        }
    }
}

#[derive(Clone, Debug)]
pub enum TextMod {
    Size(SchemeDimension),
    Color(vello::peniko::Color),
    Family(String),
    Weight(parley::FontWeight),
    Italic,
    Underline,
    Strikethrough,
    LineHeight(f32),
    Align(crate::primitives::text::TextAlign),
    LinkColor(vello::peniko::Color),
    CodeFamily(String),
}

impl Custom for TextMod {}

impl TextMod {
    pub fn apply(&self, rich: &mut RichText) {
        match self {
            TextMod::Size(d) => {
                let s = match d {
                    SchemeDimension::Length(l) => *l as f64,
                    SchemeDimension::Percent(p) => rich.root_font_size * (*p as f64),
                    SchemeDimension::Auto => rich.root_font_size,
                };
                rich.default_font_size = s;
                rich.root_font_size = s;
            }
            TextMod::Color(c) => rich.default_color = *c,
            TextMod::Family(f) => rich.default_family = f.clone(),
            TextMod::Weight(w) => rich.default_weight = *w,
            TextMod::Italic => rich.default_italic = true,
            TextMod::Underline => rich.default_underline = true,
            TextMod::Strikethrough => rich.default_strikethrough = true,
            TextMod::LineHeight(lh) => rich.default_line_height = Some(*lh),
            TextMod::Align(a) => rich.align = *a,
            TextMod::LinkColor(_) | TextMod::CodeFamily(_) => {}
        }
    }
}

#[derive(Clone, Debug)]
pub enum ShapeMod {
    Fill(Paint),
    Stroke(crate::primitives::paint::Stroke),
}
impl Custom for ShapeMod {}

#[derive(Clone, Debug)]
pub struct SchemeNode(pub Node);
impl Custom for SchemeNode {}

#[derive(Clone, Debug)]
pub struct SchemeStyle(pub Style);
impl Custom for SchemeStyle {}

#[derive(Clone, Debug)]
pub struct SchemeRichText(pub RichText);
impl Custom for SchemeRichText {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dimension_length_to_dimension() {
        let d = SchemeDimension::Length(42.0);
        assert_eq!(d.to_dimension(), Dimension::length(42.0));
    }

    #[test]
    fn dimension_percent_to_dimension() {
        let d = SchemeDimension::Percent(0.5);
        assert_eq!(d.to_dimension(), Dimension::percent(0.5));
    }

    #[test]
    fn dimension_auto_to_dimension() {
        let d = SchemeDimension::Auto;
        assert_eq!(d.to_dimension(), Dimension::auto());
    }

    #[test]
    fn dimension_length_to_length_percentage() {
        let d = SchemeDimension::Length(10.0);
        assert_eq!(d.to_length_percentage(), LengthPercentage::length(10.0));
    }

    #[test]
    fn dimension_percent_to_length_percentage() {
        let d = SchemeDimension::Percent(0.75);
        assert_eq!(d.to_length_percentage(), LengthPercentage::percent(0.75));
    }

    #[test]
    fn dimension_auto_to_length_percentage_is_zero() {
        let d = SchemeDimension::Auto;
        assert_eq!(d.to_length_percentage(), LengthPercentage::length(0.0));
    }

    #[test]
    fn dimension_to_length_percentage_auto() {
        assert_eq!(
            SchemeDimension::Length(5.0).to_length_percentage_auto(),
            LengthPercentageAuto::length(5.0)
        );
        assert_eq!(
            SchemeDimension::Percent(0.3).to_length_percentage_auto(),
            LengthPercentageAuto::percent(0.3)
        );
        assert_eq!(
            SchemeDimension::Auto.to_length_percentage_auto(),
            LengthPercentageAuto::auto()
        );
    }

    #[test]
    fn dimension_to_f64() {
        assert_eq!(SchemeDimension::Length(42.0).to_f64(), 42.0);
        assert_eq!(SchemeDimension::Percent(0.5).to_f64(), 50.0);
        assert_eq!(SchemeDimension::Auto.to_f64(), 0.0);
    }

    #[test]fn style_mod_direction() {
        let mut style = Style::default();
        StyleMod::Direction(FlexDirection::Column).apply(&mut style);
        assert_eq!(style.layout.flex_direction, FlexDirection::Column);
    }

    #[test]
    fn style_mod_align_items() {
        let mut style = Style::default();
        StyleMod::AlignItems(Some(AlignItems::CENTER)).apply(&mut style);
        assert_eq!(style.layout.align_items, Some(AlignItems::CENTER));
    }

    #[test]
    fn style_mod_justify_content() {
        let mut style = Style::default();
        StyleMod::JustifyContent(Some(JustifyContent::SPACE_BETWEEN)).apply(&mut style);
        assert_eq!(style.layout.justify_content, Some(JustifyContent::SPACE_BETWEEN));
    }

    #[test]
    fn style_mod_display() {
        let mut style = Style::default();
        StyleMod::Display(Display::None).apply(&mut style);
        assert_eq!(style.layout.display, Display::None);
    }

    #[test]
    fn style_mod_position() {
        let mut style = Style::default();
        StyleMod::Position(Position::Absolute).apply(&mut style);
        assert_eq!(style.layout.position, Position::Absolute);
    }

    #[test]
    fn style_mod_width_height() {
        let mut style = Style::default();
        StyleMod::Width(SchemeDimension::Length(100.0)).apply(&mut style);
        StyleMod::Height(SchemeDimension::Length(50.0)).apply(&mut style);
        assert_eq!(style.layout.size.width, Dimension::length(100.0));
        assert_eq!(style.layout.size.height, Dimension::length(50.0));
    }

    #[test]
    fn style_mod_max_min_size() {
        let mut style = Style::default();
        StyleMod::MaxWidth(SchemeDimension::Length(500.0)).apply(&mut style);
        StyleMod::MaxHeight(SchemeDimension::Length(300.0)).apply(&mut style);
        StyleMod::MinWidth(SchemeDimension::Length(10.0)).apply(&mut style);
        StyleMod::MinHeight(SchemeDimension::Length(5.0)).apply(&mut style);
        assert_eq!(style.layout.max_size.width, Dimension::length(500.0));
        assert_eq!(style.layout.max_size.height, Dimension::length(300.0));
        assert_eq!(style.layout.min_size.width, Dimension::length(10.0));
        assert_eq!(style.layout.min_size.height, Dimension::length(5.0));
    }

    #[test]
    fn style_mod_padding_uniform() {
        let mut style = Style::default();
        StyleMod::PaddingUniform(SchemeDimension::Length(8.0)).apply(&mut style);
        let p = LengthPercentage::length(8.0);
        assert_eq!(style.layout.padding.top, p);
        assert_eq!(style.layout.padding.right, p);
        assert_eq!(style.layout.padding.bottom, p);
        assert_eq!(style.layout.padding.left, p);
    }

    #[test]
    fn style_mod_padding_xy() {
        let mut style = Style::default();
        StyleMod::PaddingXY(
            SchemeDimension::Length(10.0),
            SchemeDimension::Length(5.0),
        ).apply(&mut style);
        assert_eq!(style.layout.padding.left, LengthPercentage::length(10.0));
        assert_eq!(style.layout.padding.right, LengthPercentage::length(10.0));
        assert_eq!(style.layout.padding.top, LengthPercentage::length(5.0));
        assert_eq!(style.layout.padding.bottom, LengthPercentage::length(5.0));
    }

    #[test]
    fn style_mod_padding_trbl() {
        let mut style = Style::default();
        StyleMod::PaddingTRBL(
            SchemeDimension::Length(1.0),
            SchemeDimension::Length(2.0),
            SchemeDimension::Length(3.0),
            SchemeDimension::Length(4.0),
        ).apply(&mut style);
        assert_eq!(style.layout.padding.top, LengthPercentage::length(1.0));
        assert_eq!(style.layout.padding.right, LengthPercentage::length(2.0));
        assert_eq!(style.layout.padding.bottom, LengthPercentage::length(3.0));
        assert_eq!(style.layout.padding.left, LengthPercentage::length(4.0));
    }

    #[test]
    fn style_mod_margin_uniform() {
        let mut style = Style::default();
        StyleMod::MarginUniform(SchemeDimension::Length(12.0)).apply(&mut style);
        let m = LengthPercentageAuto::length(12.0);
        assert_eq!(style.layout.margin.top, m);
        assert_eq!(style.layout.margin.right, m);
    }

    #[test]
    fn style_mod_margin_sides() {
        let mut style = Style::default();
        StyleMod::MarginSide { side: Side::Top, dim: SchemeDimension::Length(1.0) }.apply(&mut style);
        StyleMod::MarginSide { side: Side::Right, dim: SchemeDimension::Length(2.0) }.apply(&mut style);
        StyleMod::MarginSide { side: Side::Bottom, dim: SchemeDimension::Length(3.0) }.apply(&mut style);
        StyleMod::MarginSide { side: Side::Left, dim: SchemeDimension::Length(4.0) }.apply(&mut style);
        assert_eq!(style.layout.margin.top, LengthPercentageAuto::length(1.0));
        assert_eq!(style.layout.margin.right, LengthPercentageAuto::length(2.0));
        assert_eq!(style.layout.margin.bottom, LengthPercentageAuto::length(3.0));
        assert_eq!(style.layout.margin.left, LengthPercentageAuto::length(4.0));
    }

    #[test]
    fn style_mod_gap() {
        let mut style = Style::default();
        StyleMod::GapUniform(SchemeDimension::Length(6.0)).apply(&mut style);
        let g = LengthPercentage::length(6.0);
        assert_eq!(style.layout.gap.width, g);
        assert_eq!(style.layout.gap.height, g);
    }

    #[test]
    fn style_mod_inset_all() {
        let mut style = Style::default();
        StyleMod::InsetAll(
            SchemeDimension::Length(1.0),
            SchemeDimension::Length(2.0),
            SchemeDimension::Length(3.0),
            SchemeDimension::Length(4.0),
        ).apply(&mut style);
        assert_eq!(style.layout.inset.top, LengthPercentageAuto::length(1.0));
        assert_eq!(style.layout.inset.right, LengthPercentageAuto::length(2.0));
    }

    #[test]
    fn style_mod_inset_sides() {
        let mut style = Style::default();
        StyleMod::InsetSide { side: Side::Top, dim: SchemeDimension::Length(10.0) }.apply(&mut style);
        assert_eq!(style.layout.inset.top, LengthPercentageAuto::length(10.0));
    }

    #[test]
    fn style_mod_opacity() {
        let mut style = Style::default();
        StyleMod::Opacity(0.42).apply(&mut style);
        assert!((style.opacity - 0.42).abs() < 0.001);
    }

    #[test]
    fn style_mod_rotate() {
        let mut style = Style::default();
        StyleMod::Rotate(90.0).apply(&mut style);
        assert_eq!(style.rotate_deg, 90.0);
    }

    #[test]
    fn text_mod_size_length() {
        let mut rt = RichText::plain("x");
        TextMod::Size(SchemeDimension::Length(24.0)).apply(&mut rt);
        assert_eq!(rt.default_font_size, 24.0);
        assert_eq!(rt.root_font_size, 24.0);
    }

    #[test]
    fn text_mod_size_percent() {
        let mut rt = RichText::plain("x");
        rt.root_font_size = 20.0;
        TextMod::Size(SchemeDimension::Percent(1.5)).apply(&mut rt);
        assert_eq!(rt.default_font_size, 30.0);
    }

    #[test]
    fn text_mod_size_auto() {
        let mut rt = RichText::plain("x");
        rt.root_font_size = 20.0;
        TextMod::Size(SchemeDimension::Auto).apply(&mut rt);
        assert_eq!(rt.default_font_size, 20.0);
    }

    #[test]
    fn text_mod_color() {
        let mut rt = RichText::plain("x");
        TextMod::Color(vello::peniko::Color::WHITE).apply(&mut rt);
        assert_eq!(rt.default_color, vello::peniko::Color::WHITE);
    }

    #[test]
    fn text_mod_family() {
        let mut rt = RichText::plain("x");
        TextMod::Family("monospace".into()).apply(&mut rt);
        assert_eq!(rt.default_family, "monospace");
    }

    #[test]
    fn text_mod_weight() {
        let mut rt = RichText::plain("x");
        TextMod::Weight(parley::FontWeight::BOLD).apply(&mut rt);
        assert_eq!(rt.default_weight, parley::FontWeight::BOLD);
    }

    #[test]
    fn text_mod_italic() {
        let mut rt = RichText::plain("x");
        assert!(!rt.default_italic);
        TextMod::Italic.apply(&mut rt);
        assert!(rt.default_italic);
    }

    #[test]
    fn text_mod_underline() {
        let mut rt = RichText::plain("x");
        TextMod::Underline.apply(&mut rt);
        assert!(rt.default_underline);
    }

    #[test]
    fn text_mod_strikethrough() {
        let mut rt = RichText::plain("x");
        TextMod::Strikethrough.apply(&mut rt);
        assert!(rt.default_strikethrough);
    }

    #[test]
    fn text_mod_line_height() {
        let mut rt = RichText::plain("x");
        TextMod::LineHeight(1.5).apply(&mut rt);
        assert_eq!(rt.default_line_height, Some(1.5));
    }

    #[test]
    fn text_mod_align() {
        let mut rt = RichText::plain("x");
        TextMod::Align(crate::primitives::text::TextAlign::Center).apply(&mut rt);
        assert_eq!(rt.align, crate::primitives::text::TextAlign::Center);
    }

    #[test]
    fn text_mod_link_color_and_code_family_are_noops() {
        let mut rt = RichText::plain("x");
        let before_color = rt.default_color;
        let before_family = rt.default_family.clone();
        TextMod::LinkColor(vello::peniko::Color::WHITE).apply(&mut rt);
        TextMod::CodeFamily("Fira Code".into()).apply(&mut rt);
        assert_eq!(rt.default_color, before_color);
        assert_eq!(rt.default_family, before_family);
    }

    #[test]
    fn scheme_shape_kind_new() {
        let s = SchemeShapeKind::new(ShapeKind::Circle);
        assert!(s.width.is_none());
        assert!(s.height.is_none());
    }

    #[test]
    fn scheme_shape_kind_with_size() {
        let s = SchemeShapeKind::with_size(ShapeKind::Circle, 50.0, 30.0);
        assert_eq!(s.width, Some(50.0));
        assert_eq!(s.height, Some(30.0));
    }

    #[test]
    fn shape_mod_fill() {
        let m = ShapeMod::Fill(Paint::solid(vello::peniko::Color::BLACK));
        assert!(matches!(m, ShapeMod::Fill(_)));
    }

    #[test]
    fn shape_mod_stroke() {
        let m = ShapeMod::Stroke(crate::primitives::paint::Stroke {
            width: 2.0,
            color: vello::peniko::Color::BLACK,
        });
        assert!(matches!(m, ShapeMod::Stroke(_)));
    }
}
