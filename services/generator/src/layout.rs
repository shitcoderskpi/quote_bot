use parley::{FontContext, LayoutContext, StyleProperty, FontFamily, FontWeight};

pub struct ContentFont {
    pub family: String,
    pub size: f32,
    pub weight: f32,
}

pub fn extract_content_font(template_str: &str) -> ContentFont {
    for raw_block in template_str.split(",\n") {
        let raw_block = raw_block.trim_end_matches(',');
        if raw_block.is_empty() { continue; }
        let parts: Vec<&str> = raw_block.splitn(11, ';').collect();
        if parts.len() < 11 || parts[0] != "1" { continue; }
        if parts[10].contains("content") {
            let family = parts[6].trim().to_string();
            let size = parts[7].trim().parse::<f32>().unwrap_or(15.0);
            let weight = parts[8].trim().parse::<f32>().unwrap_or(400.0);
            return ContentFont { family, size, weight };
        }
    }
    ContentFont {
        family: "sans-serif".to_string(),
        size: 15.0,
        weight: 400.0,
    }
}

const BUBBLE_X: i32 = 47;
const TEXT_X: i32 = 56;
const TEXT_LEFT_PAD: i32 = TEXT_X - BUBBLE_X;
const TEXT_RIGHT_PAD: i32 = 16;
const SVG_RIGHT_MARGIN: i32 = 53;

const MIN_WRAP: i32 = 100;
const MAX_WRAP: i32 = 450;
const MAX_SVG: i32 = 550;
const STATUS_RIGHT_MARGIN: i32 = 60;

pub struct LayoutDimensions {
    pub wrap_width: i32,
    pub bubble_width: i32,
    pub svg_width: i32,
    pub bubble_height: i32,
    pub svg_height: i32,
    pub status_x: i32,
}

pub fn compute_dimensions(
    content: &str,
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext,
    font: &ContentFont,
) -> LayoutDimensions {
    let mut builder = layout_cx.ranged_builder(font_cx, content, 1.0, true);
    builder.push_default(StyleProperty::FontFamily(FontFamily::Source(font.family.clone().into())));
    builder.push_default(StyleProperty::FontSize(font.size * 96.0 / 72.0));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(font.weight)));
    let mut layout: parley::Layout<[u8; 4]> = builder.build(content);

    layout.break_all_lines(None);
    
    let natural_w = layout.width().ceil() as i32;
    let wrap_width = natural_w.max(MIN_WRAP).min(MAX_WRAP);

    let mut builder = layout_cx.ranged_builder(font_cx, content, 1.0, true);
    builder.push_default(StyleProperty::FontFamily(FontFamily::Source(font.family.clone().into())));
    builder.push_default(StyleProperty::FontSize(font.size * 96.0 / 72.0));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(font.weight)));
    let mut layout: parley::Layout<[u8; 4]> = builder.build(content);
    
    layout.break_all_lines(Some(wrap_width as f32));
    
    let text_h = layout.height().ceil() as i32;

    let bubble_width = wrap_width + TEXT_LEFT_PAD + TEXT_RIGHT_PAD;
    let svg_width = (BUBBLE_X + bubble_width + SVG_RIGHT_MARGIN).min(MAX_SVG);
    let bubble_height = 35 + text_h;
    let svg_height = 50 + text_h;
    let status_x = svg_width - STATUS_RIGHT_MARGIN;

    LayoutDimensions {
        wrap_width,
        bubble_width,
        svg_width,
        bubble_height,
        svg_height,
        status_x,
    }
}
