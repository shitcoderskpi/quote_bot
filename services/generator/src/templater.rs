use crate::primitives::node::{Content, Node, Style};
use crate::primitives::paint::{Paint, Stop};
use crate::primitives::shape::ShapeKind;
use crate::primitives::text::{RichText, Span, TextAlign};
use parley::FontWeight;
use steel::steel_vm::engine::Engine;
use steel::rvals::SteelVal;
use std::collections::HashMap;
use steel::steel_vm::register_fn::RegisterFn;
use taffy::prelude::*;

pub struct Templater {
    engine: Engine,
}

impl Templater {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.register_fn("string-byte-length", |s: String| s.len());
        Self { engine }
    }

    pub fn render_template(&mut self, template_src: &str, payload_json: &str) -> Result<Node, Box<dyn std::error::Error>> {
        let payload_val: serde_json::Value = serde_json::from_str(payload_json)?;
        let assoc_str = json_to_scheme(&payload_val);

        let script = format!(
            "(define payload {})\n{}",
            assoc_str, template_src
        );

        let res = self.engine.compile_and_run_raw_program(script)?;
        let last_val = res.last().ok_or("Template returned no value")?;
        
        parse_node(last_val)
    }
}

fn json_to_scheme(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "'()".to_string(),
        serde_json::Value::Bool(b) => if *b { "#t".to_string() } else { "#f".to_string() },
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => format!("\"{}\"", s.replace("\"", "\\\"")),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_to_scheme).collect();
            format!("(list {})", items.join(" "))
        }
        serde_json::Value::Object(obj) => {
            let items: Vec<String> = obj.iter().map(|(k, v)| {
                format!("(cons '{} {})", k, json_to_scheme(v))
            }).collect();
            format!("(list {})", items.join(" "))
        }
    }
}

fn get_field<'a>(val: &'a SteelVal, key: &str) -> Option<&'a SteelVal> {
    if let SteelVal::HashMapV(map) = val {
        for (k, v) in map.iter() {
            if let SteelVal::SymbolV(ks) = k {
                if ks.as_str() == key {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn parse_f64(val: &SteelVal) -> Option<f64> {
    match val {
        SteelVal::IntV(i) => Some(*i as f64),
        SteelVal::NumV(f) => Some(*f),
        _ => None,
    }
}

fn parse_dimension(val: &SteelVal) -> Dimension {
    if let SteelVal::SymbolV(s) = val {
        if s.as_str() == "auto" { return Dimension::auto(); }
    }
    if let SteelVal::ListV(list) = val {
        if let (Some(SteelVal::SymbolV(unit)), Some(num)) = (list.get(0), list.get(1)) {
            if let Some(v) = parse_f64(num) {
                if unit.as_str() == "px" { return Dimension::length(v as f32); }
                if unit.as_str() == "pct" { return Dimension::percent((v / 100.0) as f32); }
            }
        }
    }
    Dimension::auto()
}

fn parse_length_percentage(val: &SteelVal) -> LengthPercentage {
    if let SteelVal::ListV(list) = val {
        if let (Some(SteelVal::SymbolV(unit)), Some(num)) = (list.get(0), list.get(1)) {
            if let Some(v) = parse_f64(num) {
                if unit.as_str() == "px" { return LengthPercentage::length(v as f32); }
                if unit.as_str() == "pct" { return LengthPercentage::percent((v / 100.0) as f32); }
            }
        }
    }
    LengthPercentage::length(0.0)
}

fn parse_length_percentage_auto(val: &SteelVal) -> LengthPercentageAuto {
    if let SteelVal::SymbolV(s) = val {
        if s.as_str() == "auto" { return LengthPercentageAuto::auto(); }
    }
    if let SteelVal::ListV(list) = val {
        if let (Some(SteelVal::SymbolV(unit)), Some(num)) = (list.get(0), list.get(1)) {
            if let Some(v) = parse_f64(num) {
                if unit.as_str() == "px" { return LengthPercentageAuto::length(v as f32); }
                if unit.as_str() == "pct" { return LengthPercentageAuto::percent((v / 100.0) as f32); }
            }
        }
    }
    LengthPercentageAuto::auto()
}

fn parse_color(val: &SteelVal) -> Option<vello::peniko::Color> {
    if let SteelVal::ListV(list) = val {
        if let Some(SteelVal::SymbolV(kind)) = list.get(0) {
            match kind.as_str() {
                "rgb" => {
                    let r = parse_f64(list.get(1)?)? as u8;
                    let g = parse_f64(list.get(2)?)? as u8;
                    let b = parse_f64(list.get(3)?)? as u8;
                    return Some(vello::peniko::Color::from_rgb8(r, g, b))
                }
                "rgba" => {
                    let r = parse_f64(list.get(1)?)? as u8;
                    let g = parse_f64(list.get(2)?)? as u8;
                    let b = parse_f64(list.get(3)?)? as u8;
                    let a = parse_f64(list.get(4)?)? as u8;
                    return Some(vello::peniko::Color::from_rgba8(r, g, b, a));
                }
                "hex" => {
                    if let Some(SteelVal::StringV(hex_str)) = list.get(1) {
                        return parse_hex(hex_str.as_str());
                    }
                }
                _ => {}
            }
        }
    }
    None
}

fn parse_hex(hex: &str) -> Option<vello::peniko::Color> {
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

fn parse_paint(val: &SteelVal) -> Option<Paint> {
    if let SteelVal::ListV(list) = val {
        if let Some(SteelVal::SymbolV(kind)) = list.get(0) {
            if kind.as_str() == "solid" {
                if let Some(color) = parse_color(list.get(1)?) {
                    return Some(Paint::Solid(color));
                }
            } else if kind.as_str() == "linear-gradient" {
                let top_color = parse_color(list.get(1)?)?;
                let bottom_color = parse_color(list.get(2)?)?;
                return Some(Paint::LinearGradient {
                    start: vello::kurbo::Point::new(0.0, 0.0).into(),
                    end: vello::kurbo::Point::new(0.0, 1.0).into(),
                    stops: vec![
                        Stop { offset: 0.0, color: top_color },
                        Stop { offset: 1.0, color: bottom_color },
                    ],
                    extend: Default::default(),
                });
            }
        }
    }
    None
}

fn parse_length_f64(val: &SteelVal) -> Option<f64> {
    if let SteelVal::ListV(list) = val {
        if let (Some(SteelVal::SymbolV(unit)), Some(num)) = (list.get(0), list.get(1)) {
            if unit.as_str() == "px" || unit.as_str() == "pct" {
                return parse_f64(num);
            }
        }
    }
    parse_f64(val)
}

fn parse_corners(val: &SteelVal) -> crate::primitives::shape::Corners {
    if let Some(l_val) = get_field(val, "all") {
        if let Some(l) = parse_length_f64(l_val) {
            return crate::primitives::shape::Corners::all(l);
        }
    }
    let mut corners = crate::primitives::shape::Corners::zero();
    if let Some(v) = get_field(val, "top_left").and_then(parse_length_f64) { corners.top_left = v; }
    if let Some(v) = get_field(val, "top_right").and_then(parse_length_f64) { corners.top_right = v; }
    if let Some(v) = get_field(val, "bottom_right").and_then(parse_length_f64) { corners.bottom_right = v; }
    if let Some(v) = get_field(val, "bottom_left").and_then(parse_length_f64) { corners.bottom_left = v; }
    corners
}

fn parse_shape_kind(val: &SteelVal) -> Option<ShapeKind> {
    if let Some(rect) = get_field(val, "rect") {
        let corners = get_field(rect, "corners").map(parse_corners).unwrap_or(crate::primitives::shape::Corners::zero());
        return Some(ShapeKind::Rect { corners });
    }
    if let Some(_circle) = get_field(val, "circle") {
        return Some(ShapeKind::Circle);
    }
    if let Some(path) = get_field(val, "path") {
        if let Some(SteelVal::StringV(data)) = get_field(path, "data") {
            return Some(ShapeKind::Path { data: data.to_string() });
        }
    }
    None
}

fn parse_style(val: &SteelVal) -> Style {
    let mut style = Style::default();
    
    // Taffy layout mapping
    if let Some(SteelVal::SymbolV(display)) = get_field(val, "display") {
        style.layout.display = match display.as_str() {
            "none" => Display::None,
            _ => Display::Flex,
        };
    }
    
    if let Some(SteelVal::SymbolV(dir)) = get_field(val, "flex_direction") {
        style.layout.flex_direction = match dir.as_str() {
            "row" => FlexDirection::Row,
            "column" => FlexDirection::Column,
            "row-reverse" => FlexDirection::RowReverse,
            "column-reverse" => FlexDirection::ColumnReverse,
            _ => FlexDirection::Row,
        };
    }
    
    if let Some(SteelVal::SymbolV(align)) = get_field(val, "align_items") {
        style.layout.align_items = match align.as_str() {
            "flex-start" | "start" => Some(AlignItems::FLEX_START),
            "flex-end" | "end" => Some(AlignItems::FLEX_END),
            "center" => Some(AlignItems::CENTER),
            "stretch" => Some(AlignItems::STRETCH),
            "baseline" => Some(AlignItems::BASELINE),
            _ => None,
        };
    }
    
    if let Some(SteelVal::SymbolV(justify)) = get_field(val, "justify_content") {
        style.layout.justify_content = match justify.as_str() {
            "flex-start" | "start" => Some(JustifyContent::FLEX_START),
            "flex-end" | "end" => Some(JustifyContent::FLEX_END),
            "center" => Some(JustifyContent::CENTER),
            "space-between" => Some(JustifyContent::SPACE_BETWEEN),
            "space-around" => Some(JustifyContent::SPACE_AROUND),
            "space-evenly" => Some(JustifyContent::SPACE_EVENLY),
            _ => None,
        };
    }
    
    if let Some(pos) = get_field(val, "position") {
        if let Some(SteelVal::SymbolV(pos_type)) = get_field(pos, "type") {
            style.layout.position = match pos_type.as_str() {
                "absolute" => Position::Absolute,
                _ => Position::Relative,
            };
        }
        if let Some(t) = get_field(pos, "top") { style.layout.inset.top = parse_length_percentage_auto(t); }
        if let Some(r) = get_field(pos, "right") { style.layout.inset.right = parse_length_percentage_auto(r); }
        if let Some(b) = get_field(pos, "bottom") { style.layout.inset.bottom = parse_length_percentage_auto(b); }
        if let Some(l) = get_field(pos, "left") { style.layout.inset.left = parse_length_percentage_auto(l); }
    }
    
    if let Some(size) = get_field(val, "size") {
        if let Some(w) = get_field(size, "width") { style.layout.size.width = parse_dimension(w); }
        if let Some(h) = get_field(size, "height") { style.layout.size.height = parse_dimension(h); }
    }
    
    if let Some(size) = get_field(val, "max_size") {
        if let Some(w) = get_field(size, "width") { style.layout.max_size.width = parse_dimension(w); }
        if let Some(h) = get_field(size, "height") { style.layout.max_size.height = parse_dimension(h); }
    }
    
    if let Some(size) = get_field(val, "min_size") {
        if let Some(w) = get_field(size, "width") { style.layout.min_size.width = parse_dimension(w); }
        if let Some(h) = get_field(size, "height") { style.layout.min_size.height = parse_dimension(h); }
    }
    
    if let Some(padding) = get_field(val, "padding") {
        if let Some(all) = get_field(padding, "all") {
            let p = parse_length_percentage(all);
            style.layout.padding = taffy::Rect { top: p, right: p, bottom: p, left: p };
        } else {
            if let Some(t) = get_field(padding, "top") { style.layout.padding.top = parse_length_percentage(t); }
            if let Some(r) = get_field(padding, "right") { style.layout.padding.right = parse_length_percentage(r); }
            if let Some(b) = get_field(padding, "bottom") { style.layout.padding.bottom = parse_length_percentage(b); }
            if let Some(l) = get_field(padding, "left") { style.layout.padding.left = parse_length_percentage(l); }
        }
    }
    
    if let Some(margin) = get_field(val, "margin") {
        if let Some(all) = get_field(margin, "all") {
            let m = parse_length_percentage_auto(all);
            style.layout.margin = taffy::Rect { top: m, right: m, bottom: m, left: m };
        } else {
            if let Some(t) = get_field(margin, "top") { style.layout.margin.top = parse_length_percentage_auto(t); }
            if let Some(r) = get_field(margin, "right") { style.layout.margin.right = parse_length_percentage_auto(r); }
            if let Some(b) = get_field(margin, "bottom") { style.layout.margin.bottom = parse_length_percentage_auto(b); }
            if let Some(l) = get_field(margin, "left") { style.layout.margin.left = parse_length_percentage_auto(l); }
        }
    }
    
    if let Some(gap) = get_field(val, "gap") {
        if let Some(all) = get_field(gap, "all") {
            let g = parse_length_percentage(all);
            style.layout.gap = taffy::Size { width: g, height: g };
        } else {
            if let Some(x) = get_field(gap, "x") { style.layout.gap.width = parse_length_percentage(x); }
            if let Some(y) = get_field(gap, "y") { style.layout.gap.height = parse_length_percentage(y); }
        }
    }

    if let Some(opacity) = get_field(val, "opacity").and_then(parse_f64) {
        style.opacity = opacity as f32;
    }
    if let Some(rotate) = get_field(val, "rotate_deg").and_then(parse_f64) {
        style.rotate_deg = rotate;
    }
    style
}

fn parse_content(val: &SteelVal) -> Content {
    if let Some(group) = get_field(val, "group") {
        let mut children = Vec::new();
        if let SteelVal::ListV(list) = group {
            for item in list.iter() {
                if let Ok(node) = parse_node(item) {
                    children.push(node);
                }
            }
        }
        return Content::Group(children);
    }
    if let Some(shape) = get_field(val, "shape") {
        let kind = get_field(shape, "kind").and_then(parse_shape_kind).unwrap_or(ShapeKind::Circle);
        let fill = get_field(shape, "fill").and_then(parse_paint);
        return Content::Shape { kind, fill, stroke: None };
    }
    if let Some(text) = get_field(val, "text") {
        let text_str = get_field(text, "value")
            .and_then(|v| {
                if let SteelVal::StringV(s) = v { Some(s.to_string()) } else { None }
            })
            .unwrap_or_default();
        let mut rich = RichText::plain(text_str);
        
        if let Some(color_val) = get_field(text, "color") {
            if let Some(c) = parse_color(color_val) {
                rich.default_color = c;
            }
        }
        
        if let Some(size_val) = get_field(text, "size").and_then(parse_f64) {
            rich.default_font_size = size_val;
            rich.root_font_size = size_val;
        }
        if let Some(family_val) = get_field(text, "family") {
            if let SteelVal::StringV(f) = family_val {
                rich.default_family = f.to_string();
            }
        }
        if let Some(weight_val) = get_field(text, "weight").and_then(parse_f64) {
            rich.default_weight = FontWeight::new(weight_val as f32);
        }
        
        if let Some(SteelVal::ListV(spans_list)) = get_field(text, "spans") {
            for span_val in spans_list.iter() {
                if let (Some(start), Some(end)) = (get_field(span_val, "start").and_then(parse_f64), get_field(span_val, "end").and_then(parse_f64)) {
                    if start < end {
                        let mut span = Span::new(start as usize..end as usize);
                        if let Some(color_val) = get_field(span_val, "color") {
                            if let Some(c) = parse_color(color_val) { span.color = c; }
                        }
                        if let Some(weight_val) = get_field(span_val, "weight").and_then(parse_f64) {
                            span.weight = parley::style::FontWeight::new(weight_val as f32);
                        }
                        if let Some(family) = get_field(span_val, "family") {
                            if let SteelVal::StringV(f) = family { span.font_family = Some(f.to_string()); }
                        }
                        if let Some(size) = get_field(span_val, "size").and_then(parse_f64) {
                            span.font_size = size;
                        }
                        rich = rich.with_span(span);
                    }
                }
            }
        }
        
        return Content::Text(rich);
    }
    
    Content::Group(vec![])
}

fn parse_node(val: &SteelVal) -> Result<Node, Box<dyn std::error::Error>> {
    let mut style = Style::default();
    let mut content = Content::Group(vec![]);

    if let Some(s) = get_field(val, "style") {
        style = parse_style(s);
    }
    
    if let Some(c) = get_field(val, "content") {
        content = parse_content(c);
    }

    Ok(Node { style, content })
}
