use parley::{FontContext, FontFamily, FontWeight, LayoutContext, StyleProperty};

use crate::parser::{parse_entities, TextEntity};
use crate::templater::{FontSpec, InputMessage, ParsedTemplate};

/// Axis-aligned bounding box for a visual element.
#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl BBox {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn right(&self) -> i32 {
        self.x + self.width
    }

    pub fn bottom(&self) -> i32 {
        self.y + self.height
    }
}

/// Computed layout for all visual elements in the quote image.
pub struct QuoteLayout {
    /// Full SVG/image canvas.
    pub canvas: BBox,
    /// Chat bubble background shape.
    pub bubble: BBox,
    /// Circular avatar image area.
    pub avatar: BBox,
    /// Username text element.
    pub username: BBox,
    /// User status badge (present only when user has a custom title).
    pub user_status: Option<BBox>,
    /// Main message content text area.
    pub content: BBox,
    /// Text wrap width for content (text constraint, may differ from content.width).
    pub wrap_width: i32,
}

// ── Spatial constants (empirically tuned for Telegram sticker rendering) ─────

/// X position where the bubble starts.
const BUBBLE_X: i32 = 47;
/// Y position where the bubble starts.
const BUBBLE_Y: i32 = 1;

/// X position for text elements inside the bubble.
const TEXT_X: i32 = 56;
/// Left padding from bubble edge to text: TEXT_X - BUBBLE_X.
const TEXT_LEFT_PAD: i32 = TEXT_X - BUBBLE_X;
/// Right padding from text to bubble edge.
const TEXT_RIGHT_PAD: i32 = 16;
/// Right margin from bubble to SVG canvas edge.
const SVG_RIGHT_MARGIN: i32 = 53;

/// Y position of the username/status baseline.
const HEADER_BASELINE_Y: i32 = 27;
/// Y position of content text top.
const CONTENT_TOP_Y: i32 = 32;
/// Height from bubble top to content bottom including padding.
/// bubble_height = HEADER_HEIGHT + content_text_height
/// This accounts for 31px above content + 10px bottom padding.
const HEADER_HEIGHT: i32 = 41;

/// Minimum text wrap width.
const MIN_WRAP: i32 = 100;
/// Maximum text wrap width.
const MAX_WRAP: i32 = 450;
/// Maximum SVG canvas width.
const MAX_SVG_WIDTH: i32 = 550;

/// Horizontal gap between username and status text.
const HEADER_GAP: i32 = 20;

/// Avatar image size (width and height).
const AVATAR_SIZE: i32 = 36;
/// Avatar X position.
const AVATAR_X: i32 = 1;
/// Avatar vertical offset from bubble bottom edge.
const AVATAR_BOTTOM_OFFSET: i32 = 35;

// ── Bottom padding interpolation (compensates for Telegram tall-sticker downscaling) ──

const PAD_MIN: f64 = 15.0;
const PAD_MAX: f64 = 116.0;
/// Bubble height at which minimum padding applies.
const PAD_HEIGHT_MIN: f64 = 297.0;
/// Bubble height at which maximum padding applies.
const PAD_HEIGHT_MAX: f64 = 890.0;

fn get_text_width(
    text: &str,
    font: &FontSpec,
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext,
) -> i32 {
    if text.is_empty() {
        return 0;
    }
    let mut builder = layout_cx.ranged_builder(font_cx, text, 1.0, true);
    builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
        font.family.clone().into(),
    )));
    builder.push_default(StyleProperty::FontSize(font.size * 96.0 / 72.0));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(font.weight)));
    let mut layout: parley::Layout<[u8; 4]> = builder.build(text);
    layout.break_all_lines(None);
    layout.width().ceil() as i32
}

/// Apply metric-affecting entities (bold, italic, monospace) to a parley builder.
/// Only entities that change text shaping / font metrics are applied here;
/// purely visual entities (underline, strikethrough, color) are handled at render time.
fn apply_layout_entities(
    builder: &mut parley::RangedBuilder<'_, [u8; 4]>,
    entities: &[TextEntity],
    text_len: usize,
) {
    for ent in entities {
        let range = ent.offset..(ent.offset + ent.length).min(text_len);
        match ent.ty.as_str() {
            "bold" => {
                builder.push(StyleProperty::FontWeight(FontWeight::BOLD), range);
            }
            "italic" => {
                builder.push(
                    StyleProperty::FontStyle(parley::FontStyle::Italic),
                    range,
                );
            }
            "code" | "pre" => {
                builder.push(
                    StyleProperty::FontFamily(FontFamily::Source("monospace".into())),
                    range,
                );
            }
            _ => {}
        }
    }
}

/// Compute the full quote layout from an input message and parsed template.
///
/// The pipeline:
/// 1. Get font specs from the parsed template (no reparsing).
/// 2. Measure header text widths (username, status).
/// 3. Build a single content text layout — measure natural width, then re-break
///    with the computed wrap width to get the wrapped height.
/// 4. Derive all bounding boxes from the measurements.
pub fn compute_layout(
    msg: &InputMessage,
    template: &ParsedTemplate,
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext,
) -> QuoteLayout {
    let content_font = template.font_for("content");
    let username_font = template.font_for("username");
    let status_font = template.font_for("user_status");

    let content = msg.content.as_deref().unwrap_or("").trim_end();
    let username = msg.username.as_deref().unwrap_or("");
    let status = msg.user_status.as_deref().unwrap_or("");

    // Parse entities once from the input message JSON
    let entities: Vec<TextEntity> = msg
        .entities
        .as_ref()
        .map(|e| parse_entities(e))
        .unwrap_or_default();

    // ── Measure header text widths ──
    let username_w = get_text_width(username, &username_font, font_cx, layout_cx);
    let status_w = get_text_width(status, &status_font, font_cx, layout_cx);
    let min_header_w = if username_w > 0 || status_w > 0 {
        username_w + status_w + HEADER_GAP
    } else {
        0
    };

    // ── Build content layout ONCE, re-break for wrapped height ──
    let mut builder = layout_cx.ranged_builder(font_cx, content, 1.0, true);
    builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
        content_font.family.clone().into(),
    )));
    builder.push_default(StyleProperty::FontSize(content_font.size * 96.0 / 72.0));
    builder.push_default(StyleProperty::FontWeight(FontWeight::new(
        content_font.weight,
    )));
    apply_layout_entities(&mut builder, &entities, content.len());
    let mut text_layout: parley::Layout<[u8; 4]> = builder.build(content);

    // First pass: natural width (no wrapping constraint)
    text_layout.break_all_lines(None);
    let natural_w = text_layout.width().ceil() as i32;
    let required_min = MIN_WRAP.max(min_header_w);
    let wrap_width = natural_w.max(required_min).min(MAX_WRAP);

    // Second pass: re-break with wrap width (same layout object, no rebuild)
    text_layout.break_all_lines(Some(wrap_width as f32));
    let text_h = text_layout.height().ceil() as i32;

    // ── Compute bounding boxes ──

    let bubble_width = wrap_width + TEXT_LEFT_PAD + TEXT_RIGHT_PAD;
    let bubble_height = HEADER_HEIGHT + text_h;
    let bubble = BBox::new(BUBBLE_X, BUBBLE_Y, bubble_width, bubble_height);

    let svg_width = (BUBBLE_X + bubble_width + SVG_RIGHT_MARGIN).min(MAX_SVG_WIDTH);

    // Bottom padding: linearly interpolated to compensate for Telegram's
    // downscaling of tall stickers.
    let mut bottom_padding = PAD_MIN
        + (bubble_height as f64 - PAD_HEIGHT_MIN) * (PAD_MAX - PAD_MIN)
            / (PAD_HEIGHT_MAX - PAD_HEIGHT_MIN);
    if bottom_padding < PAD_MIN {
        bottom_padding = PAD_MIN;
    }
    let svg_height = bubble_height + bottom_padding.round() as i32;

    let canvas = BBox::new(0, 0, svg_width, svg_height);

    let avatar = BBox::new(
        AVATAR_X,
        bubble.bottom() - AVATAR_BOTTOM_OFFSET,
        AVATAR_SIZE,
        AVATAR_SIZE,
    );

    let username_bbox = BBox::new(TEXT_X, HEADER_BASELINE_Y, username_w, 0);

    let user_status_bbox = if !status.is_empty() {
        Some(BBox::new(TEXT_X, HEADER_BASELINE_Y, status_w, 0))
    } else {
        None
    };

    let content_bbox = BBox::new(TEXT_X, CONTENT_TOP_Y, wrap_width, text_h);

    QuoteLayout {
        canvas,
        bubble,
        avatar,
        username: username_bbox,
        user_status: user_status_bbox,
        content: content_bbox,
        wrap_width,
    }
}
