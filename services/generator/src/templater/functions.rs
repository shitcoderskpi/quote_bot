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

pub fn register_all(engine: &mut Engine) {
    engine.register_fn("px", fn_px);
    engine.register_fn("pt", fn_pt);
    engine.register_fn("pct", fn_pct);
    engine.register_fn("auto", fn_auto);
    engine.register_fn("hex", fn_hex);
    engine.register_fn("rgb", fn_rgb);
    engine.register_fn("rgba", fn_rgba);
    engine.register_fn("solid", fn_solid);
    engine.register_fn("angle", fn_angle);
    engine.register_fn("stop", fn_stop);
    engine.register_fn("%make-circle", fn_make_circle);
    engine.register_fn("svg-path", fn_svg_path);
    engine.register_fn("fill", fn_fill);
    engine.register_fn("stroke", fn_stroke);
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
    engine.register_fn("wrap", fn_wrap);
    engine.register_fn("align", fn_text_align);
    engine.register_fn("string-byte-length", |s: String| s.len());
    engine.register_fn("%make-style", fn_make_style);
    engine.register_fn("%make-node", fn_make_node);
    engine.register_fn("%make-shape", fn_make_shape);
    engine.register_fn("%make-linear-gradient", fn_make_linear_gradient);
    engine.register_fn("%make-radial-gradient", fn_make_radial_gradient);
    engine.register_fn("%make-sweep-gradient", fn_make_sweep_gradient);
    engine.register_fn("%make-rounded-rect", fn_make_rounded_rect);
    engine.register_fn("%make-rect", fn_make_rect);
    engine.register_fn("%make-padding", fn_make_padding);
    engine.register_fn("%make-image", fn_make_image);
    engine
        .compile_and_run_raw_program(
            r#"
            (define (style . mods) (%make-style mods))
            (define (circle . args) (%make-circle args))
            (define (rect . args) (%make-rect args))
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
fn fn_solid(c: SchemeColor) -> SchemePaint {
    SchemePaint(Paint::Solid(c.0))
}

fn fn_angle(deg: SchemeNumber) -> SchemeAngle {
    SchemeAngle(deg.0)
}

fn fn_stop(pct: SchemeNumber, color: SchemeColor) -> SchemeStop {
    SchemeStop {
        offset: (pct.0 / 100.0) as f32,
        color: color.0,
    }
}

fn fn_make_linear_gradient(args: SteelVal) -> Result<SchemePaint, String> {
    let list = steel_list_to_vec(&args)?;

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

    let mut angle_deg: f64 = 0.0;
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

    let rad = (angle_deg - 90.0_f64).to_radians();
    let (dx, dy) = (rad.cos(), rad.sin());
    let start = (0.5 - dx / 2.0, 0.5 - dy / 2.0);
    let end = (0.5 + dx / 2.0, 0.5 + dy / 2.0);

    Ok(SchemePaint(Paint::LinearGradient {
        start,
        end,
        stops,
        extend: Default::default(),
    }))
}

fn fn_make_circle(args: SteelVal) -> Result<SchemeShapeKind, String> {
    let list = steel_list_to_vec(&args)?;
    if list.is_empty() {
        Ok(SchemeShapeKind::new(ShapeKind::Circle))
    } else if list.len() == 1 {
        let d = SchemeDimension::from_steelval(&list[0])
            .map_err(|_| format!("circle: expected dimension, got {:?}", list[0]))?;
        let v = d.to_f64() as f32;
        Ok(SchemeShapeKind::with_size(ShapeKind::Circle, v, v))
    } else if list.len() == 2 {
        let w = SchemeDimension::from_steelval(&list[0]).map_err(|_| "circle: expected 1st arg to be dimension")?;
        let h = SchemeDimension::from_steelval(&list[1]).map_err(|_| "circle: expected 2nd arg to be dimension")?;
        Ok(SchemeShapeKind::with_size(ShapeKind::Circle, w.to_f64() as f32, h.to_f64() as f32))
    } else {
        Err(format!("circle: expected 0, 1, or 2 arguments, got {}", list.len()))
    }
}

fn fn_make_rect(args: SteelVal) -> Result<SchemeShapeKind, String> {
    let list = steel_list_to_vec(&args)?;
    if list.is_empty() {
        Ok(SchemeShapeKind::new(ShapeKind::Rect { corners: Corners::zero() }))
    } else if list.len() == 1 {
        let d = SchemeDimension::from_steelval(&list[0])
            .map_err(|_| format!("rect: expected dimension, got {:?}", list[0]))?;
        let v = d.to_f64() as f32;
        Ok(SchemeShapeKind::with_size(ShapeKind::Rect { corners: Corners::zero() }, v, v))
    } else if list.len() == 2 {
        let w = SchemeDimension::from_steelval(&list[0]).map_err(|_| "rect: expected 1st arg to be dimension")?;
        let h = SchemeDimension::from_steelval(&list[1]).map_err(|_| "rect: expected 2nd arg to be dimension")?;
        Ok(SchemeShapeKind::with_size(ShapeKind::Rect { corners: Corners::zero() }, w.to_f64() as f32, h.to_f64() as f32))
    } else {
        Err(format!("rect: expected 0, 1, or 2 arguments, got {}", list.len()))
    }
}

fn fn_make_rounded_rect(args: SteelVal) -> Result<SchemeShapeKind, String> {
    let list = steel_list_to_vec(&args)?;
    
    if list.len() == 1 {
        let r = SchemeDimension::from_steelval(&list[0])
            .map_err(|_| format!("rounded-rect: expected dimension, got {:?}", list[0]))?;
        let v = r.to_f64();
        Ok(SchemeShapeKind::new(ShapeKind::Rect { corners: Corners::all(v) }))
    } else if list.len() == 4 {
        let tl = SchemeDimension::from_steelval(&list[0]).map_err(|_| "rounded-rect: expected 1st arg to be dimension")?;
        let tr = SchemeDimension::from_steelval(&list[1]).map_err(|_| "rounded-rect: expected 2nd arg to be dimension")?;
        let br = SchemeDimension::from_steelval(&list[2]).map_err(|_| "rounded-rect: expected 3rd arg to be dimension")?;
        let bl = SchemeDimension::from_steelval(&list[3]).map_err(|_| "rounded-rect: expected 4th arg to be dimension")?;
        Ok(SchemeShapeKind::new(ShapeKind::Rect {
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
    SchemeShapeKind::new(ShapeKind::Path { data: d })
}

fn fn_fill(paint: SchemePaint) -> ShapeMod {
    ShapeMod::Fill(paint.0)
}

fn fn_stroke(paint: SchemePaint, width: SchemeNumber) -> ShapeMod {
    ShapeMod::Stroke(Stroke { width: width.0, color: match paint.0 {
        Paint::Solid(c) => c,
        _ => vello::peniko::Color::BLACK,
    }})
}

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

fn fn_size(d: SchemeDimension) -> TextMod { TextMod::Size(d) }
fn fn_color(c: SchemeColor) -> TextMod { TextMod::Color(c.0) }
fn fn_family(s: String) -> TextMod { TextMod::Family(s) }
fn fn_weight(v: SchemeNumber) -> TextMod { TextMod::Weight(parley::FontWeight::new(v.0 as f32)) }
fn fn_italic() -> TextMod { TextMod::Italic }
fn fn_link_color(c: SchemeColor) -> TextMod { TextMod::LinkColor(c.0) }
fn fn_code_family(s: String) -> TextMod { TextMod::CodeFamily(s) }
fn fn_line_height(v: SchemeNumber) -> TextMod { TextMod::LineHeight(v.0 as f32) }
fn fn_wrap(b: bool) -> TextMod { TextMod::Wrap(b) }

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
            continue;
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
                    "text_link" | "url" | "mention" | "hashtag" | "cashtag" | "email" |
                    "phone_number" | "text_mention" => {
                        span.color = Some(link_color);
                        span.underline = Some(true);
                    },
                    "bot_command" => {
                        span.color = Some(link_color);
                    }
                    _ => {}
                }
                rich.spans.push(span);
            }
        }
    }

    Ok(SchemeRichText(rich))
}

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
        content: Content::Shape { kind: kind.kind, fill, stroke },
    }))
}

fn fn_make_image(data: SteelVal, args: SteelVal) -> Result<SchemeNode, String> {
    let b64_str = match &data {
        SteelVal::StringV(s) => s.to_string(),
        other => return Err(format!("image: first argument must be a base64 string, got {:?}", other)),
    };

    let bytes = BASE64_STANDARD.decode(&b64_str)
        .map_err(|e| format!("image: invalid base64: {}", e))?;

    let img = image::load_from_memory(&bytes)
        .map_err(|e| format!("image: failed to decode image: {}", e))?;
    let rgba = img.to_rgba8();
    let (w, h) = (rgba.width(), rgba.height());

    let vello_image = vello::peniko::ImageData {
        data: vello::peniko::Blob::new(Arc::new(rgba.into_raw())),
        format: vello::peniko::ImageFormat::Rgba8,
        alpha_type: vello::peniko::ImageAlphaType::Alpha,
        width: w,
        height: h,
    };
    let image_brush = Arc::new(ImageBrush::new(vello_image));

    let arg_list = steel_list_to_vec(&args)?;
    let clip_shape = if let Some(item) = arg_list.first() {
        Some(SchemeShapeKind::from_steelval(item)
            .map_err(|e| format!("image: expected clip shape, got {:?} ({})", item, e))?)
    } else {
        None
    };

    let mut node = Node::image(image_brush, clip_shape.as_ref().map(|s| s.kind.clone()));
    if let Some(ref shape) = clip_shape {
        if let Some(w) = shape.width {
            node.style.layout.size.width = Dimension::length(w);
        }
        if let Some(h) = shape.height {
            node.style.layout.size.height = Dimension::length(h);
        }
    }
    Ok(SchemeNode(node))
}

fn fn_make_radial_gradient(args: SteelVal) -> Result<SchemePaint, String> {
    let list = steel_list_to_vec(&args)?;

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

fn symbol_str(val: &SteelVal, fn_name: &str) -> Result<String, String> {
    match val {
        SteelVal::SymbolV(s) => Ok(s.to_string()),
        SteelVal::StringV(s) => Ok(s.to_string()),
        _ => Err(format!("{}: expected a symbol (e.g., 'row), got {:?}", fn_name, val)),
    }
}

fn steel_list_to_vec(val: &SteelVal) -> Result<Vec<SteelVal>, String> {
    match val {
        SteelVal::ListV(l) => Ok(l.iter().cloned().collect()),
        SteelVal::Void => Ok(Vec::new()),
        _ => {
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

#[cfg(test)]
mod tests {
    use super::*;
    use steel::rvals::IntoSteelVal;

    fn epsilon() -> f32 {
        0.0001
    }

    #[test]
    fn hex_6_char_no_hash() {
        let c = parse_hex_color("FF0000").unwrap();
        assert_eq!(c, vello::peniko::Color::from_rgba8(255, 0, 0, 255));
    }

    #[test]
    fn hex_6_char_with_hash() {
        let c = parse_hex_color("#00FF00").unwrap();
        assert_eq!(c, vello::peniko::Color::from_rgba8(0, 255, 0, 255));
    }

    #[test]
    fn hex_8_char_with_alpha() {
        let c = parse_hex_color("#0000FF80").unwrap();
        assert_eq!(c, vello::peniko::Color::from_rgba8(0, 0, 255, 128));
    }

    #[test]
    fn hex_8_char_no_hash() {
        let c = parse_hex_color("AABBCCDD").unwrap();
        assert_eq!(c, vello::peniko::Color::from_rgba8(0xAA, 0xBB, 0xCC, 0xDD));
    }

    #[test]
    fn hex_invalid_too_short() {
        assert!(parse_hex_color("FFF").is_none());
    }

    #[test]
    fn hex_invalid_too_long() {
        assert!(parse_hex_color("AABBCCDDEEFF").is_none());
    }

    #[test]
    fn hex_invalid_chars() {
        assert!(parse_hex_color("ZZZZZZ").is_none());
    }

    #[test]
    fn hex_empty() {
        assert!(parse_hex_color("").is_none());
    }

    #[test]
    fn hex_case_insensitive() {
        let upper = parse_hex_color("AABB00").unwrap();
        let lower = parse_hex_color("aabb00").unwrap();
        assert_eq!(upper, lower);
    }

    #[test]
    fn void_is_void_or_empty() {
        assert!(is_void_or_empty(&SteelVal::Void));
    }

    #[test]
    fn false_is_void_or_empty() {
        assert!(is_void_or_empty(&SteelVal::BoolV(false)));
    }

    #[test]
    fn true_is_not_void_or_empty() {
        assert!(!is_void_or_empty(&SteelVal::BoolV(true)));
    }

    #[test]
    fn int_is_not_void_or_empty() {
        assert!(!is_void_or_empty(&SteelVal::IntV(42)));
    }

    #[test]
    fn px_converts() {
        let d = fn_px(SchemeNumber(100.0));
        assert!(matches!(d, SchemeDimension::Length(v) if (v - 100.0).abs() < 0.01));
    }

    #[test]
    fn px_zero() {
        let d = fn_px(SchemeNumber(0.0));
        assert!(matches!(d, SchemeDimension::Length(v) if v == 0.0));
    }

    #[test]
    fn pt_converts() {
        let d = fn_pt(SchemeNumber(12.0));
        match d {
            SchemeDimension::Length(v) => assert!((v - 16.0).abs() < 0.01, "12pt = {}px", v),
            _ => panic!("Expected Length"),
        }
    }

    #[test]
    fn pct_converts() {
        let d = fn_pct(SchemeNumber(50.0));
        assert!(matches!(d, SchemeDimension::Percent(v) if (v - 0.5).abs() < epsilon()));
    }

    #[test]
    fn pct_100() {
        let d = fn_pct(SchemeNumber(100.0));
        assert!(matches!(d, SchemeDimension::Percent(v) if (v - 1.0).abs() < epsilon()));
    }

    #[test]
    fn auto_dim() {
        let d = fn_auto();
        assert!(matches!(d, SchemeDimension::Auto));
    }

    #[test]
    fn solid_wraps_color() {
        let c = SchemeColor(vello::peniko::Color::BLACK);
        let p = fn_solid(c);
        assert!(matches!(p.0, Paint::Solid(_)));
    }

    #[test]
    fn angle_wraps_degrees() {
        let a = fn_angle(SchemeNumber(45.0));
        assert_eq!(a.0, 45.0);
    }

    #[test]
    fn stop_normalizes_offset() {
        let s = fn_stop(SchemeNumber(50.0), SchemeColor(vello::peniko::Color::BLACK));
        assert!((s.offset - 0.5).abs() < epsilon());
    }

    #[test]
    fn fill_creates_shape_mod() {
        let p = SchemePaint(Paint::solid(vello::peniko::Color::BLACK));
        let m = fn_fill(p);
        assert!(matches!(m, ShapeMod::Fill(_)));
    }

    #[test]
    fn stroke_creates_shape_mod() {
        let p = SchemePaint(Paint::solid(vello::peniko::Color::WHITE));
        let m = fn_stroke(p, SchemeNumber(3.0));
        match m {
            ShapeMod::Stroke(s) => {
                assert_eq!(s.width, 3.0);
                assert_eq!(s.color, vello::peniko::Color::WHITE);
            }
            _ => panic!("Expected Stroke"),
        }
    }

    #[test]
    fn stroke_gradient_paint_falls_back_to_black() {
        let p = SchemePaint(Paint::LinearGradient {
            start: (0.0, 0.0),
            end: (1.0, 1.0),
            stops: vec![],
            extend: Default::default(),
        });
        let m = fn_stroke(p, SchemeNumber(1.0));
        match m {
            ShapeMod::Stroke(s) => assert_eq!(s.color, vello::peniko::Color::BLACK),
            _ => panic!("Expected Stroke"),
        }
    }

    #[test]
    fn width_height_fns() {
        let w = fn_width(SchemeDimension::Length(100.0));
        assert!(matches!(w, StyleMod::Width(_)));
        let h = fn_height(SchemeDimension::Length(50.0));
        assert!(matches!(h, StyleMod::Height(_)));
    }

    #[test]
    fn opacity_fn() {
        let o = fn_opacity(SchemeNumber(0.5));
        assert!(matches!(o, StyleMod::Opacity(v) if (v - 0.5).abs() < epsilon()));
    }

    #[test]
    fn rotate_fn() {
        let r = fn_rotate(SchemeNumber(90.0));
        assert!(matches!(r, StyleMod::Rotate(v) if v == 90.0));
    }

    #[test]
    fn size_fn() {
        let m = fn_size(SchemeDimension::Length(20.0));
        assert!(matches!(m, TextMod::Size(_)));
    }

    #[test]
    fn color_fn() {
        let m = fn_color(SchemeColor(vello::peniko::Color::WHITE));
        assert!(matches!(m, TextMod::Color(_)));
    }

    #[test]
    fn family_fn() {
        let m = fn_family("serif".to_string());
        assert!(matches!(m, TextMod::Family(f) if f == "serif"));
    }

    #[test]
    fn weight_fn() {
        let m = fn_weight(SchemeNumber(700.0));
        match m {
            TextMod::Weight(w) => assert_eq!(w, parley::FontWeight::new(700.0)),
            _ => panic!("Expected Weight"),
        }
    }

    #[test]
    fn italic_fn() {
        let m = fn_italic();
        assert!(matches!(m, TextMod::Italic));
    }

    #[test]
    fn line_height_fn() {
        let m = fn_line_height(SchemeNumber(1.5));
        assert!(matches!(m, TextMod::LineHeight(v) if (v - 1.5).abs() < epsilon()));
    }

    #[test]
    fn wrap_fn() {
        let m = fn_wrap(false);
        match m {
            TextMod::Wrap(b) => assert!(!b),
            _ => panic!("Expected Wrap"),
        }
    }

    #[test]
    fn svg_path_creates_shape_kind() {
        let s = fn_svg_path("M 0 0 L 10 10".to_string());
        assert!(matches!(s.kind, ShapeKind::Path { ref data } if data == "M 0 0 L 10 10"));
    }

    #[test]
    fn rgb_creates_color() {
        let c = fn_rgb(255, 128, 0);
        assert_eq!(c.0, vello::peniko::Color::from_rgb8(255, 128, 0));
    }

    #[test]
    fn rgba_creates_color() {
        let c = fn_rgba(10, 20, 30, 128);
        assert_eq!(c.0, vello::peniko::Color::from_rgba8(10, 20, 30, 128));
    }

    #[test]
    fn fn_hex_valid() {
        let c = fn_hex("#FF0000".to_string()).unwrap();
        assert_eq!(c.0, vello::peniko::Color::from_rgba8(255, 0, 0, 255));
    }

    #[test]
    fn fn_hex_invalid() {
        let err = fn_hex("ZZZ".to_string());
        assert!(err.is_err());
    }

    fn make_steel_list(items: Vec<SteelVal>) -> SteelVal {
        SteelVal::ListV(items.into_iter().collect())
    }

    fn steel_stop(pct: f64, r: u8, g: u8, b: u8) -> SteelVal {
        let stop = fn_stop(SchemeNumber(pct), SchemeColor(vello::peniko::Color::from_rgb8(r, g, b)));
        stop.into_steelval().unwrap()
    }

    fn steel_angle(deg: f64) -> SteelVal {
        let a = fn_angle(SchemeNumber(deg));
        a.into_steelval().unwrap()
    }

    #[test]
    fn linear_gradient_with_angle_and_stops() {
        let args = make_steel_list(vec![
            steel_angle(90.0),
            steel_stop(0.0, 255, 0, 0),
            steel_stop(100.0, 0, 0, 255),
        ]);
        let result = fn_make_linear_gradient(args).unwrap();
        assert!(matches!(result.0, Paint::LinearGradient { .. }));
    }

    #[test]
    fn linear_gradient_error_too_few_stops() {
        let args = make_steel_list(vec![
            steel_angle(45.0),
            steel_stop(0.0, 255, 0, 0),
        ]);
        let result = fn_make_linear_gradient(args);
        assert!(result.is_err());
    }

    #[test]
    fn linear_gradient_error_bad_argument() {
        let args = make_steel_list(vec![
            SteelVal::StringV("bad".into()),
        ]);
        let result = fn_make_linear_gradient(args);
        assert!(result.is_err());
    }

    #[test]
    fn radial_gradient_with_stops() {
        let args = make_steel_list(vec![
            steel_stop(0.0, 255, 0, 0),
            steel_stop(50.0, 0, 255, 0),
            steel_stop(100.0, 0, 0, 255),
        ]);
        let result = fn_make_radial_gradient(args).unwrap();
        assert!(matches!(result.0, Paint::RadialGradient { .. }));
    }

    #[test]
    fn radial_gradient_error_too_few_stops() {
        let args = make_steel_list(vec![
            steel_stop(0.0, 255, 0, 0),
        ]);
        let result = fn_make_radial_gradient(args);
        assert!(result.is_err());
    }

    #[test]
    fn radial_gradient_error_bad_argument() {
        let args = make_steel_list(vec![
            SteelVal::IntV(42),
        ]);
        let result = fn_make_radial_gradient(args);
        assert!(result.is_err());
    }

    #[test]
    fn sweep_gradient_with_angles_and_stops() {
        let args = make_steel_list(vec![
            steel_angle(0.0),
            steel_angle(180.0),
            steel_stop(0.0, 255, 0, 0),
            steel_stop(100.0, 0, 0, 255),
        ]);
        let result = fn_make_sweep_gradient(args).unwrap();
        assert!(matches!(result.0, Paint::SweepGradient { start_angle, end_angle, .. }
            if start_angle == 0.0 && end_angle == 180.0));
    }

    #[test]
    fn sweep_gradient_error_too_few_stops() {
        let args = make_steel_list(vec![
            steel_stop(0.0, 255, 0, 0),
        ]);
        assert!(fn_make_sweep_gradient(args).is_err());
    }

    #[test]
    fn sweep_gradient_error_bad_argument() {
        let args = make_steel_list(vec![
            SteelVal::StringV("nope".into()),
        ]);
        assert!(fn_make_sweep_gradient(args).is_err());
    }

    fn steel_dim(px: f64) -> SteelVal {
        SchemeDimension::Length(px as f32).into_steelval().unwrap()
    }

    #[test]
    fn make_circle_no_args() {
        let args = make_steel_list(vec![]);
        let result = fn_make_circle(args).unwrap();
        assert!(matches!(result.kind, ShapeKind::Circle));
        assert!(result.width.is_none());
    }

    #[test]
    fn make_circle_one_arg() {
        let args = make_steel_list(vec![steel_dim(50.0)]);
        let result = fn_make_circle(args).unwrap();
        assert!(matches!(result.kind, ShapeKind::Circle));
        assert_eq!(result.width, Some(50.0));
        assert_eq!(result.height, Some(50.0));
    }

    #[test]
    fn make_circle_two_args() {
        let args = make_steel_list(vec![steel_dim(80.0), steel_dim(60.0)]);
        let result = fn_make_circle(args).unwrap();
        assert_eq!(result.width, Some(80.0));
        assert_eq!(result.height, Some(60.0));
    }

    #[test]
    fn make_circle_error_too_many_args() {
        let args = make_steel_list(vec![steel_dim(1.0), steel_dim(2.0), steel_dim(3.0)]);
        assert!(fn_make_circle(args).is_err());
    }

    #[test]
    fn make_rect_no_args() {
        let args = make_steel_list(vec![]);
        let result = fn_make_rect(args).unwrap();
        assert!(matches!(result.kind, ShapeKind::Rect { .. }));
    }

    #[test]
    fn make_rect_one_arg() {
        let args = make_steel_list(vec![steel_dim(100.0)]);
        let result = fn_make_rect(args).unwrap();
        assert_eq!(result.width, Some(100.0));
    }

    #[test]
    fn make_rect_two_args() {
        let args = make_steel_list(vec![steel_dim(100.0), steel_dim(50.0)]);
        let result = fn_make_rect(args).unwrap();
        assert_eq!(result.width, Some(100.0));
        assert_eq!(result.height, Some(50.0));
    }

    #[test]
    fn make_rect_error_too_many() {
        let args = make_steel_list(vec![steel_dim(1.0), steel_dim(2.0), steel_dim(3.0)]);
        assert!(fn_make_rect(args).is_err());
    }

    #[test]
    fn make_rounded_rect_four_args() {
        let args = make_steel_list(vec![
            steel_dim(5.0), steel_dim(10.0), steel_dim(15.0), steel_dim(20.0),
        ]);
        let result = fn_make_rounded_rect(args).unwrap();
        match result.kind {
            ShapeKind::Rect { corners } => {
                assert_eq!(corners.top_left, 5.0);
                assert_eq!(corners.top_right, 10.0);
                assert_eq!(corners.bottom_right, 15.0);
                assert_eq!(corners.bottom_left, 20.0);
            }
            _ => panic!("Expected Rect"),
        }
    }

    #[test]
    fn make_rounded_rect_error_bad_count() {
        let args = make_steel_list(vec![steel_dim(1.0), steel_dim(2.0)]);
        assert!(fn_make_rounded_rect(args).is_err());
    }

    #[test]
    fn direction_all_variants() {
        for (sym, expected) in [
            ("row", FlexDirection::Row),
            ("column", FlexDirection::Column),
            ("row-reverse", FlexDirection::RowReverse),
            ("column-reverse", FlexDirection::ColumnReverse),
        ] {
            let val = SteelVal::SymbolV(sym.into());
            let result = fn_direction(val).unwrap();
            assert!(matches!(result, StyleMod::Direction(d) if d == expected));
        }
    }

    #[test]
    fn direction_error() {
        let val = SteelVal::SymbolV("diagonal".into());
        assert!(fn_direction(val).is_err());
    }

    #[test]
    fn align_items_all_variants() {
        for sym in ["start", "flex-start", "end", "flex-end", "center", "stretch", "baseline"] {
            let val = SteelVal::SymbolV(sym.into());
            assert!(fn_align_items(val).is_ok());
        }
    }

    #[test]
    fn align_items_error() {
        let val = SteelVal::SymbolV("middle".into());
        assert!(fn_align_items(val).is_err());
    }

    #[test]
    fn justify_content_all_variants() {
        for sym in ["start", "flex-start", "end", "flex-end", "center",
                     "space-between", "space-around", "space-evenly"] {
            let val = SteelVal::SymbolV(sym.into());
            assert!(fn_justify_content(val).is_ok());
        }
    }

    #[test]
    fn justify_content_error() {
        let val = SteelVal::SymbolV("spread".into());
        assert!(fn_justify_content(val).is_err());
    }

    #[test]
    fn display_flex() {
        let val = SteelVal::SymbolV("flex".into());
        assert!(matches!(fn_display(val).unwrap(), StyleMod::Display(Display::Flex)));
    }

    #[test]
    fn display_none() {
        let val = SteelVal::SymbolV("none".into());
        assert!(matches!(fn_display(val).unwrap(), StyleMod::Display(Display::None)));
    }

    #[test]
    fn display_error() {
        let val = SteelVal::SymbolV("grid".into());
        assert!(fn_display(val).is_err());
    }

    #[test]
    fn position_absolute() {
        let val = SteelVal::SymbolV("absolute".into());
        assert!(matches!(fn_position(val).unwrap(), StyleMod::Position(Position::Absolute)));
    }

    #[test]
    fn position_relative() {
        let val = SteelVal::SymbolV("relative".into());
        assert!(matches!(fn_position(val).unwrap(), StyleMod::Position(Position::Relative)));
    }

    #[test]
    fn position_error() {
        let val = SteelVal::SymbolV("fixed".into());
        assert!(fn_position(val).is_err());
    }

    #[test]
    fn make_padding_uniform() {
        let args = make_steel_list(vec![steel_dim(10.0)]);
        let result = fn_make_padding(args).unwrap();
        assert!(matches!(result, StyleMod::PaddingUniform(_)));
    }

    #[test]
    fn make_padding_trbl() {
        let args = make_steel_list(vec![
            steel_dim(1.0), steel_dim(2.0), steel_dim(3.0), steel_dim(4.0),
        ]);
        let result = fn_make_padding(args).unwrap();
        assert!(matches!(result, StyleMod::PaddingTRBL(..)));
    }

    #[test]
    fn make_padding_error_bad_count() {
        let args = make_steel_list(vec![steel_dim(1.0), steel_dim(2.0)]);
        assert!(fn_make_padding(args).is_err());
    }

    #[test]
    fn padding_xy_fn() {
        let result = fn_padding_xy(SchemeDimension::Length(10.0), SchemeDimension::Length(5.0));
        assert!(matches!(result, StyleMod::PaddingXY(..)));
    }

    #[test]
    fn margin_fn() {
        let result = fn_margin(SchemeDimension::Length(8.0));
        assert!(matches!(result, StyleMod::MarginUniform(_)));
    }

    #[test]
    fn gap_fn() {
        let result = fn_gap(SchemeDimension::Length(4.0));
        assert!(matches!(result, StyleMod::GapUniform(_)));
    }

    #[test]
    fn inset_fn() {
        let result = fn_inset(
            SchemeDimension::Length(1.0),
            SchemeDimension::Length(2.0),
            SchemeDimension::Length(3.0),
            SchemeDimension::Length(4.0),
        );
        assert!(matches!(result, StyleMod::InsetAll(..)));
    }

    #[test]
    fn max_min_width_height_fns() {
        assert!(matches!(fn_max_width(SchemeDimension::Length(500.0)), StyleMod::MaxWidth(_)));
        assert!(matches!(fn_max_height(SchemeDimension::Length(300.0)), StyleMod::MaxHeight(_)));
        assert!(matches!(fn_min_width(SchemeDimension::Length(10.0)), StyleMod::MinWidth(_)));
        assert!(matches!(fn_min_height(SchemeDimension::Length(5.0)), StyleMod::MinHeight(_)));
    }

    #[test]
    fn link_color_fn() {
        let m = fn_link_color(SchemeColor(vello::peniko::Color::WHITE));
        assert!(matches!(m, TextMod::LinkColor(_)));
    }

    #[test]
    fn code_family_fn() {
        let m = fn_code_family("Fira Code".to_string());
        assert!(matches!(m, TextMod::CodeFamily(f) if f == "Fira Code"));
    }

    #[test]
    fn text_align_all_variants() {
        for (sym, expected) in [
            ("start", TextAlign::Start),
            ("left", TextAlign::Start),
            ("center", TextAlign::Center),
            ("end", TextAlign::End),
            ("right", TextAlign::End),
            ("justify", TextAlign::Justify),
        ] {
            let val = SteelVal::SymbolV(sym.into());
            let result = fn_text_align(val).unwrap();
            assert!(matches!(result, TextMod::Align(a) if a == expected));
        }
    }

    #[test]
    fn text_align_error() {
        let val = SteelVal::SymbolV("middle".into());
        assert!(fn_text_align(val).is_err());
    }

    #[test]
    fn make_style_empty() {
        let mods = make_steel_list(vec![]);
        let result = fn_make_style(mods).unwrap();
        assert_eq!(result.0.opacity, 1.0);
    }

    #[test]
    fn make_style_with_mods() {
        let opacity_mod = StyleMod::Opacity(0.5);
        let rotate_mod = StyleMod::Rotate(90.0);
        let mods = make_steel_list(vec![
            opacity_mod.into_steelval().unwrap(),
            rotate_mod.into_steelval().unwrap(),
        ]);
        let result = fn_make_style(mods).unwrap();
        assert!((result.0.opacity - 0.5).abs() < epsilon());
        assert_eq!(result.0.rotate_deg, 90.0);
    }

    #[test]
    fn make_node_empty() {
        let args = make_steel_list(vec![]);
        let result = fn_make_node(args).unwrap();
        assert!(matches!(result.0.content, Content::Group(ref c) if c.is_empty()));
    }

    #[test]
    fn make_node_with_style_and_child() {
        let style = SchemeStyle(Style::default()).into_steelval().unwrap();
        let child = SchemeNode(Node::shape(ShapeKind::Circle, None)).into_steelval().unwrap();
        let args = make_steel_list(vec![style, child]);
        let result = fn_make_node(args).unwrap();
        assert!(matches!(result.0.content, Content::Shape { .. }));
    }

    #[test]
    fn make_node_with_multiple_children() {
        let c1 = SchemeNode(Node::shape(ShapeKind::Circle, None)).into_steelval().unwrap();
        let c2 = SchemeNode(Node::shape(ShapeKind::Circle, None)).into_steelval().unwrap();
        let args = make_steel_list(vec![c1, c2]);
        let result = fn_make_node(args).unwrap();
        assert!(matches!(result.0.content, Content::Group(ref c) if c.len() == 2));
    }

    #[test]
    fn make_node_skips_void() {
        let c1 = SchemeNode(Node::shape(ShapeKind::Circle, None)).into_steelval().unwrap();
        let args = make_steel_list(vec![SteelVal::Void, c1]);
        let result = fn_make_node(args).unwrap();
        assert!(matches!(result.0.content, Content::Shape { .. }));
    }

    #[test]
    fn make_node_error_bad_arg() {
        let args = make_steel_list(vec![SteelVal::IntV(42)]);
        assert!(fn_make_node(args).is_err());
    }

    #[test]
    fn make_text_basic() {
        let content = SteelVal::StringV("hello".into());
        let mods = make_steel_list(vec![]);
        let result = fn_make_text(content, mods, &None, &None).unwrap();
        assert_eq!(result.0.text, "hello");
    }

    #[test]
    fn make_text_error_not_string() {
        let content = SteelVal::IntV(42);
        let mods = make_steel_list(vec![]);
        assert!(fn_make_text(content, mods, &None, &None).is_err());
    }

    #[test]
    fn make_text_with_mods() {
        let content = SteelVal::StringV("styled".into());
        let bold = TextMod::Weight(parley::FontWeight::BOLD);
        let mods = make_steel_list(vec![bold.into_steelval().unwrap()]);
        let result = fn_make_text(content, mods, &None, &None).unwrap();
        assert_eq!(result.0.default_weight, parley::FontWeight::BOLD);
    }

    #[test]
    fn make_text_with_link_color_and_code_family() {
        let content = SteelVal::StringV("links".into());
        let lc = TextMod::LinkColor(vello::peniko::Color::WHITE);
        let cf = TextMod::CodeFamily("Fira Code".into());
        let mods = make_steel_list(vec![
            lc.into_steelval().unwrap(),
            cf.into_steelval().unwrap(),
        ]);
        let result = fn_make_text(content, mods, &None, &None).unwrap();
        assert_eq!(result.0.text, "links");
    }

    #[test]
    fn make_text_with_entities() {
        let text = "Hello bold world";
        let content = SteelVal::StringV(text.into());
        let mods = make_steel_list(vec![]);
        let entities = vec![
            serde_json::json!({"type": "bold", "offset": 6, "length": 4}),
            serde_json::json!({"type": "italic", "offset": 0, "length": 5}),
            serde_json::json!({"type": "underline", "offset": 0, "length": 5}),
            serde_json::json!({"type": "strikethrough", "offset": 0, "length": 5}),
            serde_json::json!({"type": "code", "offset": 0, "length": 5}),
            serde_json::json!({"type": "text_link", "offset": 0, "length": 5}),
            serde_json::json!({"type": "bot_command", "offset": 0, "length": 5}),
            serde_json::json!({"type": "unknown_type", "offset": 0, "length": 5}),
        ];
        let result = fn_make_text(
            content, mods,
            &Some(text.to_string()),
            &Some(entities),
        ).unwrap();
        assert!(!result.0.spans.is_empty());
    }

    #[test]
    fn make_shape_with_fill() {
        let kind = SchemeShapeKind::new(ShapeKind::Circle);
        let fill_mod = ShapeMod::Fill(Paint::solid(vello::peniko::Color::BLACK));
        let mods = make_steel_list(vec![fill_mod.into_steelval().unwrap()]);
        let result = fn_make_shape(kind, mods).unwrap();
        assert!(matches!(result.0.content, Content::Shape { ref fill, .. } if fill.is_some()));
    }

    #[test]
    fn make_shape_with_stroke() {
        let kind = SchemeShapeKind::new(ShapeKind::Circle);
        let stroke_mod = ShapeMod::Stroke(Stroke {
            width: 3.0,
            color: vello::peniko::Color::WHITE,
        });
        let mods = make_steel_list(vec![stroke_mod.into_steelval().unwrap()]);
        let result = fn_make_shape(kind, mods).unwrap();
        assert!(matches!(result.0.content, Content::Shape { ref stroke, .. } if stroke.is_some()));
    }

    #[test]
    fn make_image_valid() {
        let mut buf = std::io::Cursor::new(Vec::new());
        let img = image::RgbaImage::from_pixel(1, 1, image::Rgba([255, 0, 0, 255]));
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let b64 = BASE64_STANDARD.encode(buf.into_inner());

        let data = SteelVal::StringV(b64.into());
        let args = make_steel_list(vec![]);
        let result = fn_make_image(data, args).unwrap();
        assert!(matches!(result.0.content, Content::Image { .. }));
    }

    #[test]
    fn make_image_with_clip() {
        let mut buf = std::io::Cursor::new(Vec::new());
        let img = image::RgbaImage::from_pixel(2, 2, image::Rgba([0, 0, 255, 255]));
        img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
        let b64 = BASE64_STANDARD.encode(buf.into_inner());

        let clip = SchemeShapeKind::with_size(ShapeKind::Circle, 50.0, 50.0);
        let data = SteelVal::StringV(b64.into());
        let args = make_steel_list(vec![clip.into_steelval().unwrap()]);
        let result = fn_make_image(data, args).unwrap();
        assert!(matches!(result.0.content, Content::Image { ref clip, .. } if clip.is_some()));
    }

    #[test]
    fn make_image_error_not_string() {
        let data = SteelVal::IntV(42);
        let args = make_steel_list(vec![]);
        assert!(fn_make_image(data, args).is_err());
    }

    #[test]
    fn make_image_error_invalid_base64() {
        let data = SteelVal::StringV("not-base64!!!".into());
        let args = make_steel_list(vec![]);
        assert!(fn_make_image(data, args).is_err());
    }

    #[test]
    fn symbol_str_from_symbol() {
        let val = SteelVal::SymbolV("test".into());
        assert_eq!(symbol_str(&val, "fn").unwrap(), "test");
    }

    #[test]
    fn symbol_str_from_string() {
        let val = SteelVal::StringV("test".into());
        assert_eq!(symbol_str(&val, "fn").unwrap(), "test");
    }

    #[test]
    fn symbol_str_error() {
        let val = SteelVal::IntV(42);
        assert!(symbol_str(&val, "fn").is_err());
    }

    #[test]
    fn steel_list_to_vec_from_list() {
        let list = make_steel_list(vec![SteelVal::IntV(1), SteelVal::IntV(2)]);
        let result = steel_list_to_vec(&list).unwrap();
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn steel_list_to_vec_from_void() {
        let result = steel_list_to_vec(&SteelVal::Void).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn steel_list_to_vec_error() {
        let val = SteelVal::IntV(42);
        assert!(steel_list_to_vec(&val).is_err());
    }

    #[test]
    fn steel_list_to_vec_from_false() {
        let result = steel_list_to_vec(&SteelVal::BoolV(false)).unwrap();
        assert!(result.is_empty());
    }
}
