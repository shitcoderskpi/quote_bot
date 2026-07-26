//! All Rust functions registered into the Steel engine.
//! Organized by domain: units, colors, paints, shapes, style mods, text mods,
//! node constructors.

use crate::primitives::node::{Content, Node, Style};
use crate::primitives::paint::{Paint, Stop, Stroke};
use crate::primitives::shape::{Corners, ShapeKind};
use crate::primitives::text::{RichText, TextAlign};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as Base64Engine};
use steel::rvals::{FromSteelVal, SteelVal};
use steel::steel_vm::engine::Engine;
use steel::steel_vm::register_fn::RegisterFn;
use std::sync::Arc;
use taffy::prelude::*;
use vello::peniko::ImageBrush;

use super::types::*;

// ═══════════════════════════════════════════════════════════════
//  Registration entry point
// ═══════════════════════════════════════════════════════════════

pub fn register_all(engine: &mut Engine) {
    // ── Units ────────────────────────────────────────────────
    engine.register_fn("px", fn_px);
    engine.register_fn("pt", fn_pt);
    engine.register_fn("pct", fn_pct);
    engine.register_fn("auto", fn_auto);

    // ── Colors ───────────────────────────────────────────────
    engine.register_fn("hex", fn_hex);
    engine.register_fn("rgb", fn_rgb);
    engine.register_fn("rgba", fn_rgba);

    // ── Paints ───────────────────────────────────────────────
    engine.register_fn("solid", fn_solid);

    // ── Gradient helpers ─────────────────────────────────────
    engine.register_fn("angle", fn_angle);
    engine.register_fn("stop", fn_stop);

    // ── Shapes ───────────────────────────────────────────────
    engine.register_fn("circle", fn_circle);
    engine.register_fn("rect", fn_rect);
    engine.register_fn("svg-path", fn_svg_path);

    // ── Fill / Stroke wrappers ───────────────────────────────
    engine.register_fn("fill", fn_fill);
    engine.register_fn("stroke", fn_stroke);

    // ── Style property functions (value args) ────────────────
    engine.register_fn("direction", fn_direction);
    engine.register_fn("align-items", fn_align_items);
    engine.register_fn("justify-content", fn_justify_content);
    engine.register_fn("display", fn_display);
    engine.register_fn("position", fn_position);
    engine.register_fn("width", fn_width);
    engine.register_fn("height", fn_height);
    engine.register_fn("max-width", fn_max_width);
    engine.register_fn("max-height", fn_max_height);
    engine.register_fn("min-width", fn_min_width);
    engine.register_fn("min-height", fn_min_height);
    engine.register_fn("padding-xy", fn_padding_xy);
    engine.register_fn("margin", fn_margin);
    engine.register_fn("margin-top", |d: SchemeDimension| StyleMod::MarginSide { side: Side::Top, dim: d });
    engine.register_fn("margin-right", |d: SchemeDimension| StyleMod::MarginSide { side: Side::Right, dim: d });
    engine.register_fn("margin-bottom", |d: SchemeDimension| StyleMod::MarginSide { side: Side::Bottom, dim: d });
    engine.register_fn("margin-left", |d: SchemeDimension| StyleMod::MarginSide { side: Side::Left, dim: d });
    engine.register_fn("gap", fn_gap);
    engine.register_fn("inset", fn_inset);
    engine.register_fn("top", |d: SchemeDimension| StyleMod::InsetSide { side: Side::Top, dim: d });
    engine.register_fn("right", |d: SchemeDimension| StyleMod::InsetSide { side: Side::Right, dim: d });
    engine.register_fn("bottom", |d: SchemeDimension| StyleMod::InsetSide { side: Side::Bottom, dim: d });
    engine.register_fn("left", |d: SchemeDimension| StyleMod::InsetSide { side: Side::Left, dim: d });
    engine.register_fn("opacity", fn_opacity);
    engine.register_fn("rotate", fn_rotate);

    // ── Convenience aliases ──────────────────────────────────
    engine.register_fn("flex-row", || StyleMod::Direction(FlexDirection::Row));
    engine.register_fn("flex-column", || StyleMod::Direction(FlexDirection::Column));
    engine.register_fn("align-start", || StyleMod::AlignItems(Some(AlignItems::FLEX_START)));
    engine.register_fn("align-center", || StyleMod::AlignItems(Some(AlignItems::CENTER)));
    engine.register_fn("align-end", || StyleMod::AlignItems(Some(AlignItems::FLEX_END)));
    engine.register_fn("justify-center", || StyleMod::JustifyContent(Some(JustifyContent::CENTER)));
    engine.register_fn("justify-between", || StyleMod::JustifyContent(Some(JustifyContent::SPACE_BETWEEN)));
    engine.register_fn("justify-evenly", || StyleMod::JustifyContent(Some(JustifyContent::SPACE_EVENLY)));
    engine.register_fn("hidden", || StyleMod::Display(Display::None));
    engine.register_fn("absolute", || StyleMod::Position(Position::Absolute));
    engine.register_fn("relative", || StyleMod::Position(Position::Relative));

    // ── Text modifiers ───────────────────────────────────────
    engine.register_fn("size", fn_size);
    engine.register_fn("color", fn_color);
    engine.register_fn("family", fn_family);
    engine.register_fn("weight", fn_weight);
    engine.register_fn("italic", fn_italic);
    engine.register_fn("underline", || TextMod::Underline);
    engine.register_fn("strikethrough", || TextMod::Strikethrough);
    engine.register_fn("link-color", fn_link_color);
    engine.register_fn("code-family", fn_code_family);
    engine.register_fn("line-height", fn_line_height);
    engine.register_fn("align", fn_text_align);

    // ── Utility ──────────────────────────────────────────────
    engine.register_fn("string-byte-length", |s: String| s.len());

    // ── Varargs constructors (Scheme wrappers around Rust impls) ──
    engine.register_fn("%make-style", fn_make_style);
    engine.register_fn("%make-node", fn_make_node);

    engine.register_fn("%make-shape", fn_make_shape);
    engine.register_fn("%make-linear-gradient", fn_make_linear_gradient);
    engine.register_fn("%make-radial-gradient", fn_make_radial_gradient);
    engine.register_fn("%make-sweep-gradient", fn_make_sweep_gradient);
    engine.register_fn("%make-rounded-rect", fn_make_rounded_rect);
    engine.register_fn("%make-padding", fn_make_padding);
    engine.register_fn("%make-image", fn_make_image);

    // Inject thin Scheme wrappers that collect rest-args into lists
    engine
        .compile_and_run_raw_program(
            r#"
            (define (style . mods) (%make-style mods))
            (define (node . args) (%make-node args))
            (define (shape kind . mods) (%make-shape kind mods))
            (define (linear-gradient . args) (%make-linear-gradient args))
            (define (radial-gradient . args) (%make-radial-gradient args))
            (define (sweep-gradient . args) (%make-sweep-gradient args))
            (define (rounded-rect . args) (%make-rounded-rect args))
            (define (padding . args) (%make-padding args))
            (define (image data . args) (%make-image data args))
            "#,
        )
        .expect("Failed to register Scheme wrapper functions");
}

// ═══════════════════════════════════════════════════════════════
//  Unit / Dimension
// ═══════════════════════════════════════════════════════════════

fn fn_px(v: SchemeNumber) -> SchemeDimension {
    SchemeDimension::Length(v.0 as f32)
}

fn fn_pt(v: SchemeNumber) -> SchemeDimension {
    SchemeDimension::Length((v.0 * 1.333333) as f32)
}

fn fn_pct(v: SchemeNumber) -> SchemeDimension {
    SchemeDimension::Percent(v.0 as f32 / 100.0)
}

fn fn_auto() -> SchemeDimension {
    SchemeDimension::Auto
}

// ═══════════════════════════════════════════════════════════════
//  Colors
// ═══════════════════════════════════════════════════════════════

fn fn_hex(s: String) -> Result<SchemeColor, String> {
    parse_hex_color(&s)
        .map(SchemeColor)
        .ok_or_else(|| format!("hex: invalid hex color \"{}\"", s))
}

fn fn_rgb(r: isize, g: isize, b: isize) -> SchemeColor {
    SchemeColor(vello::peniko::Color::from_rgb8(r as u8, g as u8, b as u8))
}

fn fn_rgba(r: isize, g: isize, b: isize, a: isize) -> SchemeColor {
    SchemeColor(vello::peniko::Color::from_rgba8(r as u8, g as u8, b as u8, a as u8))
}

fn parse_hex_color(hex: &str) -> Option<vello::peniko::Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 || hex.len() == 8 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        };
        Some(vello::peniko::Color::from_rgba8(r, g, b, a))
    } else {
        None
    }
}

// ═══════════════════════════════════════════════════════════════
//  Paints
// ═══════════════════════════════════════════════════════════════

fn fn_solid(c: SchemeColor) -> SchemePaint {
    SchemePaint(Paint::Solid(c.0))
}

// ── Gradient helpers ─────────────────────────────────────────

fn fn_angle(deg: SchemeNumber) -> SchemeAngle {
    SchemeAngle(deg.0)
}

fn fn_stop(pct: SchemeNumber, color: SchemeColor) -> SchemeStop {
    SchemeStop {
        offset: (pct.0 / 100.0) as f32,
        color: color.0,
    }
}

/// Shorthand: `(linear-gradient COLOR COLOR)`
/// Advanced: `(linear-gradient (angle 45) (stop 0 (hex "#f00")) (stop 100 (hex "#00f")))`
fn fn_make_linear_gradient(args: SteelVal) -> Result<SchemePaint, String> {
    let list = steel_list_to_vec(&args)?;

    // Check for shorthand (exactly two colors)
    if list.len() == 2 && SchemeColor::from_steelval(&list[0]).is_ok() && SchemeColor::from_steelval(&list[1]).is_ok() {
        let top = SchemeColor::from_steelval(&list[0]).unwrap();
        let bottom = SchemeColor::from_steelval(&list[1]).unwrap();
        return Ok(SchemePaint(Paint::LinearGradient {
            start: (0.0, 0.0),
            end: (0.0, 1.0),
            stops: vec![
                Stop { offset: 0.0, color: top.0 },
                Stop { offset: 1.0, color: bottom.0 },
            ],
            extend: Default::default(),
        }));
    }

    let mut angle_deg: f64 = 0.0; // default: top→bottom
    let mut stops: Vec<Stop> = Vec::new();

    for item in &list {
        if let Ok(a) = SchemeAngle::from_steelval(item) {
            angle_deg = a.0;
        } else if let Ok(s) = SchemeStop::from_steelval(item) {
            stops.push(Stop { offset: s.offset, color: s.color });
        } else {
            return Err(format!("linear-gradient: unexpected argument: {:?}", item));
        }
    }

    if stops.len() < 2 {
        return Err("linear-gradient: need exactly 2 colors OR at least 2 stops".to_string());
    }

    // Convert angle (CSS convention: 0° = bottom→top, 90° = left→right)
    let rad = (angle_deg - 90.0_f64).to_radians();
    let (dx, dy) = (rad.cos(), rad.sin());
    // Map direction vector to 0..1 coordinates
    let start = (0.5 - dx / 2.0, 0.5 - dy / 2.0);
    let end = (0.5 + dx / 2.0, 0.5 + dy / 2.0);

    Ok(SchemePaint(Paint::LinearGradient {
        start,
        end,
        stops,
        extend: Default::default(),
    }))
}

// ═══════════════════════════════════════════════════════════════
//  Shapes
// ═══════════════════════════════════════════════════════════════

fn fn_circle() -> SchemeShapeKind {
    SchemeShapeKind(ShapeKind::Circle)
}

fn fn_rect() -> SchemeShapeKind {
    SchemeShapeKind(ShapeKind::Rect { corners: Corners::zero() })
}

/// `(rounded-rect DIM)` or `(rounded-rect TL TR BR BL)`
fn fn_make_rounded_rect(args: SteelVal) -> Result<SchemeShapeKind, String> {
    let list = steel_list_to_vec(&args)?;
    
    if list.len() == 1 {
        let r = SchemeDimension::from_steelval(&list[0])
            .map_err(|_| format!("rounded-rect: expected dimension, got {:?}", list[0]))?;
        let v = r.to_f64();
        Ok(SchemeShapeKind(ShapeKind::Rect { corners: Corners::all(v) }))
    } else if list.len() == 4 {
        let tl = SchemeDimension::from_steelval(&list[0]).map_err(|_| "rounded-rect: expected 1st arg to be dimension")?;
        let tr = SchemeDimension::from_steelval(&list[1]).map_err(|_| "rounded-rect: expected 2nd arg to be dimension")?;
        let br = SchemeDimension::from_steelval(&list[2]).map_err(|_| "rounded-rect: expected 3rd arg to be dimension")?;
        let bl = SchemeDimension::from_steelval(&list[3]).map_err(|_| "rounded-rect: expected 4th arg to be dimension")?;
        Ok(SchemeShapeKind(ShapeKind::Rect {
            corners: Corners {
                top_left: tl.to_f64(),
                top_right: tr.to_f64(),
                bottom_right: br.to_f64(),
                bottom_left: bl.to_f64(),
            },
        }))
    } else {
        Err(format!("rounded-rect: expected 1 or 4 arguments, got {}", list.len()))
    }
}

fn fn_svg_path(d: String) -> SchemeShapeKind {
    SchemeShapeKind(ShapeKind::Path { data: d })
}

// ── Fill / Stroke ────────────────────────────────────────────

fn fn_fill(paint: SchemePaint) -> ShapeMod {
    ShapeMod::Fill(paint.0)
}

fn fn_stroke(paint: SchemePaint, width: SchemeNumber) -> ShapeMod {
    ShapeMod::Stroke(Stroke { width: width.0, color: match paint.0 {
        Paint::Solid(c) => c,
        _ => vello::peniko::Color::BLACK,
    }})
}

// ═══════════════════════════════════════════════════════════════
//  Style property functions (accept value arg, return StyleMod)
// ═══════════════════════════════════════════════════════════════

fn fn_direction(val: SteelVal) -> Result<StyleMod, String> {
    let s = symbol_str(&val, "direction")?;
    match s.as_str() {
        "row" => Ok(StyleMod::Direction(FlexDirection::Row)),
        "column" => Ok(StyleMod::Direction(FlexDirection::Column)),
        "row-reverse" => Ok(StyleMod::Direction(FlexDirection::RowReverse)),
        "column-reverse" => Ok(StyleMod::Direction(FlexDirection::ColumnReverse)),
        _ => Err(format!(
            "direction: unknown value '{}', expected one of: row, column, row-reverse, column-reverse",
            s
        )),
    }
}

fn fn_align_items(val: SteelVal) -> Result<StyleMod, String> {
    let s = symbol_str(&val, "align-items")?;
    let ai = match s.as_str() {
        "start" | "flex-start" => Some(AlignItems::FLEX_START),
        "end" | "flex-end" => Some(AlignItems::FLEX_END),
        "center" => Some(AlignItems::CENTER),
        "stretch" => Some(AlignItems::STRETCH),
        "baseline" => Some(AlignItems::BASELINE),
        _ => return Err(format!(
            "align-items: unknown value '{}', expected one of: start, end, center, stretch, baseline",
            s
        )),
    };
    Ok(StyleMod::AlignItems(ai))
}

fn fn_justify_content(val: SteelVal) -> Result<StyleMod, String> {
    let s = symbol_str(&val, "justify-content")?;
    let jc = match s.as_str() {
        "start" | "flex-start" => Some(JustifyContent::FLEX_START),
        "end" | "flex-end" => Some(JustifyContent::FLEX_END),
        "center" => Some(JustifyContent::CENTER),
        "space-between" => Some(JustifyContent::SPACE_BETWEEN),
        "space-around" => Some(JustifyContent::SPACE_AROUND),
        "space-evenly" => Some(JustifyContent::SPACE_EVENLY),
        _ => return Err(format!(
            "justify-content: unknown value '{}', expected one of: start, end, center, space-between, space-around, space-evenly",
            s
        )),
    };
    Ok(StyleMod::JustifyContent(jc))
}

fn fn_display(val: SteelVal) -> Result<StyleMod, String> {
    let s = symbol_str(&val, "display")?;
    match s.as_str() {
        "flex" => Ok(StyleMod::Display(Display::Flex)),
        "none" => Ok(StyleMod::Display(Display::None)),
        _ => Err(format!("display: unknown value '{}', expected: flex, none", s)),
    }
}

fn fn_position(val: SteelVal) -> Result<StyleMod, String> {
    let s = symbol_str(&val, "position")?;
    match s.as_str() {
        "absolute" => Ok(StyleMod::Position(Position::Absolute)),
        "relative" => Ok(StyleMod::Position(Position::Relative)),
        _ => Err(format!("position: unknown value '{}', expected: absolute, relative", s)),
    }
}

fn fn_width(d: SchemeDimension) -> StyleMod { StyleMod::Width(d) }
fn fn_height(d: SchemeDimension) -> StyleMod { StyleMod::Height(d) }
fn fn_max_width(d: SchemeDimension) -> StyleMod { StyleMod::MaxWidth(d) }
fn fn_max_height(d: SchemeDimension) -> StyleMod { StyleMod::MaxHeight(d) }
fn fn_min_width(d: SchemeDimension) -> StyleMod { StyleMod::MinWidth(d) }
fn fn_min_height(d: SchemeDimension) -> StyleMod { StyleMod::MinHeight(d) }

fn fn_make_padding(args: SteelVal) -> Result<StyleMod, String> {
    let list = steel_list_to_vec(&args)?;
    if list.len() == 1 {
        let d = SchemeDimension::from_steelval(&list[0])
            .map_err(|_| format!("padding: expected dimension, got {:?}", list[0]))?;
        Ok(StyleMod::PaddingUniform(d))
    } else if list.len() == 4 {
        let t = SchemeDimension::from_steelval(&list[0]).map_err(|_| "padding: expected 1st arg to be dimension")?;
        let r = SchemeDimension::from_steelval(&list[1]).map_err(|_| "padding: expected 2nd arg to be dimension")?;
        let b = SchemeDimension::from_steelval(&list[2]).map_err(|_| "padding: expected 3rd arg to be dimension")?;
        let l = SchemeDimension::from_steelval(&list[3]).map_err(|_| "padding: expected 4th arg to be dimension")?;
        Ok(StyleMod::PaddingTRBL(t, r, b, l))
    } else {
        Err(format!("padding: expected 1 or 4 arguments, got {}", list.len()))
    }
}
fn fn_padding_xy(h: SchemeDimension, v: SchemeDimension) -> StyleMod { StyleMod::PaddingXY(h, v) }

fn fn_margin(d: SchemeDimension) -> StyleMod { StyleMod::MarginUniform(d) }
fn fn_gap(d: SchemeDimension) -> StyleMod { StyleMod::GapUniform(d) }

fn fn_inset(
    t: SchemeDimension, r: SchemeDimension,
    b: SchemeDimension, l: SchemeDimension,
) -> StyleMod {
    StyleMod::InsetAll(t, r, b, l)
}

fn fn_opacity(v: SchemeNumber) -> StyleMod { StyleMod::Opacity(v.0 as f32) }
fn fn_rotate(v: SchemeNumber) -> StyleMod { StyleMod::Rotate(v.0) }

// ═══════════════════════════════════════════════════════════════
//  Text modifiers
// ═══════════════════════════════════════════════════════════════

fn fn_size(d: SchemeDimension) -> TextMod { TextMod::Size(d) }
fn fn_color(c: SchemeColor) -> TextMod { TextMod::Color(c.0) }
fn fn_family(s: String) -> TextMod { TextMod::Family(s) }
fn fn_weight(v: SchemeNumber) -> TextMod { TextMod::Weight(parley::FontWeight::new(v.0 as f32)) }
fn fn_italic() -> TextMod { TextMod::Italic }
fn fn_link_color(c: SchemeColor) -> TextMod { TextMod::LinkColor(c.0) }
fn fn_code_family(s: String) -> TextMod { TextMod::CodeFamily(s) }
fn fn_line_height(v: SchemeNumber) -> TextMod { TextMod::LineHeight(v.0 as f32) }

fn fn_text_align(val: SteelVal) -> Result<TextMod, String> {
    let s = symbol_str(&val, "align")?;
    match s.as_str() {
        "start" | "left" => Ok(TextMod::Align(TextAlign::Start)),
        "center" => Ok(TextMod::Align(TextAlign::Center)),
        "end" | "right" => Ok(TextMod::Align(TextAlign::End)),
        "justify" => Ok(TextMod::Align(TextAlign::Justify)),
        _ => Err(format!("align: unknown value '{}', expected: start, center, end, justify", s)),
    }
}

// ═══════════════════════════════════════════════════════════════
//  Varargs constructors (called from Scheme wrappers)
// ═══════════════════════════════════════════════════════════════

/// `(style (direction 'row) (padding (px 10)) ...)`
fn fn_make_style(mods: SteelVal) -> Result<SchemeStyle, String> {
    let list = steel_list_to_vec(&mods)?;
    let mut style = Style::default();
    for item in &list {
        let m = StyleMod::from_steelval(item)
            .map_err(|e| format!("style: expected style modifier, got {:?} ({})", item, e))?;
        m.apply(&mut style);
    }
    Ok(SchemeStyle(style))
}

/// `(node [style] children...)`
fn fn_make_node(args: SteelVal) -> Result<SchemeNode, String> {
    let list = steel_list_to_vec(&args)?;
    let mut style = Style::default();
    let mut children: Vec<Node> = Vec::new();

    for item in &list {
        if let Ok(s) = SchemeStyle::from_steelval(item) {
            style = s.0;
        } else if let Ok(n) = SchemeNode::from_steelval(item) {
            children.push(n.0);
        } else if let Ok(t) = SchemeRichText::from_steelval(item) {
            children.push(Node::text(t.0));
        } else if is_void_or_empty(item) {
            continue; // skip '() from conditional (if ...) that returned false
        } else {
            return Err(format!("node: unexpected argument: {:?}", item));
        }
    }

    let content = match children.len() {
        0 => Content::Group(vec![]),
        1 => children.into_iter().next().unwrap().content,
        _ => Content::Group(children),
    };

    Ok(SchemeNode(Node { style, content }))
}

/// `(text "Hello" (size 15) (color (hex "#000")) ...)`
pub(crate) fn fn_make_text(
    content: SteelVal,
    mods: SteelVal,
    content_opt: &Option<String>,
    entities_opt: &Option<Vec<serde_json::Value>>,
) -> Result<SchemeRichText, String> {
    let text_str = match &content {
        SteelVal::StringV(s) => s.to_string(),
        other => return Err(format!("text: first argument must be a string, got {:?}", other)),
    };

    let mod_list = steel_list_to_vec(&mods)?;
    let mut rich = RichText::plain(&text_str);

    let mut link_color = vello::peniko::Color::from_rgb8(80, 150, 240);
    let mut code_family = "monospace".to_string();

    for item in &mod_list {
        let m = TextMod::from_steelval(item)
            .map_err(|e| format!("text: expected text modifier, got {:?} ({})", item, e))?;
            
        if let TextMod::LinkColor(c) = m {
            link_color = c;
        } else if let TextMod::CodeFamily(f) = &m {
            code_family = f.clone();
        }
        
        m.apply(&mut rich);
    }
    
    // Eagerly inject entities if this text matches the payload content
    if let (Some(content_str), Some(entities)) = (content_opt, entities_opt) {
        if let Some(base_offset) = rich.text.find(content_str) {
            for ent in entities {
                let t = ent.get("type").and_then(|v| v.as_str()).unwrap_or("");
                let offset = base_offset + ent.get("offset").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let length = ent.get("length").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                
                let mut span = crate::primitives::text::Span::new(offset..(offset + length));
                match t {
                    "bold" => span.weight = Some(parley::FontWeight::BOLD),
                    "italic" => span.italic = Some(true),
                    "underline" => span.underline = Some(true),
                    "strikethrough" => span.strikethrough = Some(true),
                    "code" | "pre" => span.font_family = Some(code_family.clone()),
                    "text_link" | "url" => {
                        span.color = Some(link_color);
                        span.underline = Some(true);
                    },
                    _ => {}
                }
                rich.spans.push(span);
            }
        }
    }

    Ok(SchemeRichText(rich))
}

/// `(shape (circle) (fill (solid (hex "#FFF"))))`
fn fn_make_shape(kind: SchemeShapeKind, mods: SteelVal) -> Result<SchemeNode, String> {
    let mod_list = steel_list_to_vec(&mods)?;

    let mut fill: Option<Paint> = None;
    let mut stroke: Option<Stroke> = None;

    for item in &mod_list {
        let m = ShapeMod::from_steelval(item)
            .map_err(|e| format!("shape: expected fill or stroke modifier, got {:?} ({})", item, e))?;
        match m {
            ShapeMod::Fill(p) => fill = Some(p),
            ShapeMod::Stroke(s) => stroke = Some(s),
        }
    }

    Ok(SchemeNode(Node {
        style: Style::default(),
        content: Content::Shape { kind: kind.0, fill, stroke },
    }))
}

/// `(image "base64data..." (circle))` or `(image "base64data...")`
fn fn_make_image(data: SteelVal, args: SteelVal) -> Result<SchemeNode, String> {
    let b64_str = match &data {
        SteelVal::StringV(s) => s.to_string(),
        other => return Err(format!("image: first argument must be a base64 string, got {:?}", other)),
    };

    // Base64 decode
    let bytes = BASE64_STANDARD.decode(&b64_str)
        .map_err(|e| format!("image: invalid base64: {}", e))?;

    // Decode image (JPEG, PNG, WebP, etc.)
    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("image: failed to decode image: {}", e))?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());

    // Create vello ImageData + ImageBrush
    let vello_image = vello::peniko::ImageData {
        data: vello::peniko::Blob::new(Arc::new(rgba.into_raw())),
        format: vello::peniko::ImageFormat::Rgba8,
        alpha_type: vello::peniko::ImageAlphaType::Alpha,
        width: w,
        height: h,
    };
    let image_brush = Arc::new(ImageBrush::new(vello_image));

    // Parse optional clip shape from args
    let arg_list = steel_list_to_vec(&args)?;
    let clip = if let Some(item) = arg_list.first() {
        Some(SchemeShapeKind::from_steelval(item)
            .map_err(|e| format!("image: expected clip shape, got {:?} ({})", item, e))?.0)
    } else {
        None
    };

    Ok(SchemeNode(Node::image(image_brush, clip)))
}

/// Radial gradient: `(radial-gradient (stop 0 (hex "#fff")) (stop 100 (hex "#000")))`
/// Optionally with center/radius overrides via number pairs.
fn fn_make_radial_gradient(args: SteelVal) -> Result<SchemePaint, String> {
    let list = steel_list_to_vec(&args)?;

    // Shorthand: two colors
    if list.len() == 2
        && SchemeColor::from_steelval(&list[0]).is_ok()
        && SchemeColor::from_steelval(&list[1]).is_ok()
    {
        let inner = SchemeColor::from_steelval(&list[0]).unwrap();
        let outer = SchemeColor::from_steelval(&list[1]).unwrap();
        return Ok(SchemePaint(Paint::RadialGradient {
            center: (0.5, 0.5),
            radius: 0.5,
            stops: vec![
                Stop { offset: 0.0, color: inner.0 },
                Stop { offset: 1.0, color: outer.0 },
            ],
            extend: Default::default(),
        }));
    }

    let mut stops: Vec<Stop> = Vec::new();
    for item in &list {
        if let Ok(s) = SchemeStop::from_steelval(item) {
            stops.push(Stop { offset: s.offset, color: s.color });
        } else {
            return Err(format!("radial-gradient: unexpected argument: {:?}", item));
        }
    }

    if stops.len() < 2 {
        return Err("radial-gradient: need exactly 2 colors OR at least 2 stops".to_string());
    }

    Ok(SchemePaint(Paint::RadialGradient {
        center: (0.5, 0.5),
        radius: 0.5,
        stops,
        extend: Default::default(),
    }))
}

/// Sweep gradient: `(sweep-gradient (angle 0) (angle 360) (stop 0 ...) (stop 100 ...))`
/// First two `angle` args are start and end angles.
fn fn_make_sweep_gradient(args: SteelVal) -> Result<SchemePaint, String> {
    let list = steel_list_to_vec(&args)?;

    let mut angles: Vec<f32> = Vec::new();
    let mut stops: Vec<Stop> = Vec::new();

    for item in &list {
        if let Ok(a) = SchemeAngle::from_steelval(item) {
            angles.push(a.0 as f32);
        } else if let Ok(s) = SchemeStop::from_steelval(item) {
            stops.push(Stop { offset: s.offset, color: s.color });
        } else {
            return Err(format!("sweep-gradient: unexpected argument: {:?}", item));
        }
    }

    let start_angle = angles.first().copied().unwrap_or(0.0);
    let end_angle = angles.get(1).copied().unwrap_or(360.0);

    if stops.len() < 2 {
        return Err("sweep-gradient: need at least 2 stops".to_string());
    }

    Ok(SchemePaint(Paint::SweepGradient {
        center: (0.5, 0.5),
        start_angle,
        end_angle,
        stops,
        extend: Default::default(),
    }))
}

// ═══════════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════════

/// Extract a symbol name from a SteelVal, with a nice error on wrong type.
fn symbol_str(val: &SteelVal, fn_name: &str) -> Result<String, String> {
    match val {
        SteelVal::SymbolV(s) => Ok(s.to_string()),
        SteelVal::StringV(s) => Ok(s.to_string()),
        _ => Err(format!("{}: expected a symbol (e.g., 'row), got {:?}", fn_name, val)),
    }
}

/// Convert a Steel list (or empty/void) to a Vec of SteelVal.
fn steel_list_to_vec(val: &SteelVal) -> Result<Vec<SteelVal>, String> {
    match val {
        SteelVal::ListV(l) => Ok(l.iter().cloned().collect()),
        SteelVal::Void => Ok(Vec::new()),
        _ => {
            // Try treating as a null/empty list
            if is_void_or_empty(val) {
                Ok(Vec::new())
            } else {
                Err(format!("expected a list, got {:?}", val))
            }
        }
    }
}

fn is_void_or_empty(val: &SteelVal) -> bool {
    matches!(val, SteelVal::Void)
        || matches!(val, SteelVal::ListV(l) if l.is_empty())
        || matches!(val, SteelVal::BoolV(false))
}
