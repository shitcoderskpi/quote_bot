//! Opaque wrapper types that Steel Scheme manipulates as values.
//! Each implements `Custom` (via the blanket `Sealed` impl on `Any`)
//! so `register_fn` can accept and return them transparently.

use crate::primitives::node::{Content, Node, Style};
use crate::primitives::paint::Paint;
use crate::primitives::shape::{Corners, ShapeKind};
use crate::primitives::text::RichText;
use steel::rvals::{Custom, FromSteelVal, SteelVal};
use taffy::prelude::*;

// ── Number Helper ────────────────────────────────────────────

/// Transparently parses both Scheme integers and floats into an f64.
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

// ── Dimension ────────────────────────────────────────────────

/// A resolved dimension value (px/pt/pct/auto) that style modifiers consume.
#[derive(Clone, Debug)]
pub enum SchemeDimension {
    Length(f32),   // already in px
    Percent(f32),  // 0.0–1.0
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

// ── Color ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SchemeColor(pub vello::peniko::Color);
impl Custom for SchemeColor {}

// ── Paint ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SchemePaint(pub Paint);
impl Custom for SchemePaint {}

// ── Gradient helpers ─────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SchemeAngle(pub f64);
impl Custom for SchemeAngle {}

#[derive(Clone, Debug)]
pub struct SchemeStop {
    pub offset: f32,
    pub color: vello::peniko::Color,
}
impl Custom for SchemeStop {}

// ── Shape ────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SchemeShapeKind(pub ShapeKind);
impl Custom for SchemeShapeKind {}

// ── Style modifier ───────────────────────────────────────────

/// A modifier that patches one field of a `Style`. Collected by `(style ...)`.
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

// ── Text modifier ────────────────────────────────────────────

/// A modifier that patches one field of a `RichText`. Collected by `(text ...)`.
#[derive(Clone, Debug)]
pub enum TextMod {
    Size(f64),
    Color(vello::peniko::Color),
    Family(String),
    Weight(f32),
    Align(crate::primitives::text::TextAlign),
}

impl Custom for TextMod {}

impl TextMod {
    pub fn apply(&self, rich: &mut RichText) {
        match self {
            TextMod::Size(s) => {
                rich.default_font_size = *s;
                rich.root_font_size = *s;
            }
            TextMod::Color(c) => rich.default_color = *c,
            TextMod::Family(f) => rich.default_family = f.clone(),
            TextMod::Weight(w) => rich.default_weight = parley::FontWeight::new(*w),
            TextMod::Align(a) => rich.align = *a,
        }
    }
}

// ── Fill / Stroke wrappers for shape ─────────────────────────

#[derive(Clone, Debug)]
pub enum ShapeMod {
    Fill(Paint),
    Stroke(crate::primitives::paint::Stroke),
}
impl Custom for ShapeMod {}

// ── Node (final output) ─────────────────────────────────────

#[derive(Clone, Debug)]
pub struct SchemeNode(pub Node);
impl Custom for SchemeNode {}

// ── Style (assembled from mods) ──────────────────────────────

#[derive(Clone, Debug)]
pub struct SchemeStyle(pub Style);
impl Custom for SchemeStyle {}

// ── Rich text (assembled from mods) ──────────────────────────

#[derive(Clone, Debug)]
pub struct SchemeRichText(pub RichText);
impl Custom for SchemeRichText {}
