use crate::primitives::Viewport;
use parley::{
    Alignment, FontContext, FontWeight, GenericFamily, Layout, LayoutContext,
    StyleProperty,
};
use vello::peniko::{self, Brush};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct Span {
    pub range: Range<usize>,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub weight: Option<FontWeight>,
    pub italic: Option<bool>,
    pub underline: Option<bool>,
    pub strikethrough: Option<bool>,
    pub color: Option<peniko::Color>,
    pub line_height: Option<f32>,
}

impl Span {
    pub fn new(range: Range<usize>) -> Self {
        Self {
            range,
            font_family: None,
            font_size: None,
            weight: None,
            italic: None,
            underline: None,
            strikethrough: None,
            color: None,
            line_height: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    Start,
    Center,
    End,
    Justify,
}

#[derive(Debug, Clone)]
pub struct RichText {
    pub text: String,
    pub spans: Vec<Span>,
    pub align: TextAlign,
    pub wrap: bool,
    pub default_font_size: f64,
    pub default_family: String,
    pub default_color: peniko::Color,
    pub default_weight: FontWeight,
    pub default_italic: bool,
    pub default_underline: bool,
    pub default_strikethrough: bool,
    pub default_line_height: Option<f32>,
    pub root_font_size: f64,
}

impl RichText {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            spans: Vec::new(),
            align: TextAlign::Start,
            wrap: true,
            default_font_size: 16.0,
            default_family: "sans-serif".to_string(),
            default_color: peniko::Color::BLACK,
            default_weight: FontWeight::NORMAL,
            default_italic: false,
            default_underline: false,
            default_strikethrough: false,
            default_line_height: None,
            root_font_size: 16.0,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.spans.push(span);
        self
    }

    pub fn layout(
        &self,
        font_cx: &mut FontContext,
        layout_cx: &mut LayoutContext<Brush>,
        max_width: Option<f64>,
        _viewport: Viewport,
    ) -> Layout<Brush> {
        let mut builder = layout_cx.ranged_builder(font_cx, &self.text, 1.0, true);

        builder.push_default(StyleProperty::FontFamily(
            parley::style::FontFamily::named(&self.default_family),
        ));
        builder.push_default(StyleProperty::FontSize(self.default_font_size as f32));
        builder.push_default(StyleProperty::Brush(Brush::Solid(self.default_color)));
        builder.push_default(StyleProperty::FontWeight(self.default_weight));
        if self.default_italic {
            builder.push_default(StyleProperty::FontStyle(parley::FontStyle::Italic));
        }
        if self.default_underline {
            builder.push_default(StyleProperty::Underline(true));
        }
        if self.default_strikethrough {
            builder.push_default(StyleProperty::Strikethrough(true));
        }
        if let Some(lh) = self.default_line_height {
            builder.push_default(StyleProperty::LineHeight(parley::style::LineHeight::MetricsRelative(lh)));
        }

        for span in &self.spans {
            if let Some(fs) = span.font_size {
                builder.push(StyleProperty::FontSize(fs as f32), span.range.clone());
            }
            if let Some(w) = span.weight {
                builder.push(StyleProperty::FontWeight(w), span.range.clone());
            }
            if let Some(italic) = span.italic {
                if italic {
                    builder.push(StyleProperty::FontStyle(parley::FontStyle::Italic), span.range.clone());
                } else {
                    builder.push(StyleProperty::FontStyle(parley::FontStyle::Normal), span.range.clone());
                }
            }
            if let Some(underline) = span.underline {
                builder.push(StyleProperty::Underline(underline), span.range.clone());
            }
            if let Some(strikethrough) = span.strikethrough {
                builder.push(StyleProperty::Strikethrough(strikethrough), span.range.clone());
            }
            if let Some(c) = span.color {
                builder.push(StyleProperty::Brush(Brush::Solid(c)), span.range.clone());
            }
            if let Some(lh) = span.line_height {
                builder.push(StyleProperty::LineHeight(parley::style::LineHeight::MetricsRelative(lh)), span.range.clone());
            }
            if let Some(family) = &span.font_family {
                builder.push(
                    StyleProperty::FontFamily(parley::style::FontFamily::named(family)),
                    span.range.clone(),
                );
            }
        }

        let mut layout = builder.build(&self.text);
        if self.wrap {
            layout.break_all_lines(max_width.map(|w| w as f32));
        } else {
            layout.break_all_lines(None);
        }

        let alignment = match self.align {
            TextAlign::Start => Alignment::Start,
            TextAlign::Center => Alignment::Center,
            TextAlign::End => Alignment::End,
            TextAlign::Justify => Alignment::Justify,
        };
        layout.align(alignment, parley::layout::AlignmentOptions::default());
        layout
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_defaults() {
        let rt = RichText::plain("hello");
        assert_eq!(rt.text, "hello");
        assert!(rt.spans.is_empty());
        assert_eq!(rt.align, TextAlign::Start);
        assert!(rt.wrap);
        assert_eq!(rt.default_font_size, 16.0);
        assert_eq!(rt.default_family, "sans-serif");
        assert_eq!(rt.default_color, peniko::Color::BLACK);
        assert_eq!(rt.default_weight, FontWeight::NORMAL);
        assert!(!rt.default_italic);
        assert!(!rt.default_underline);
        assert!(!rt.default_strikethrough);
        assert!(rt.default_line_height.is_none());
        assert_eq!(rt.root_font_size, 16.0);
    }

    #[test]
    fn plain_from_string_type() {
        let rt = RichText::plain(String::from("owned"));
        assert_eq!(rt.text, "owned");
    }

    #[test]
    fn plain_empty_string() {
        let rt = RichText::plain("");
        assert_eq!(rt.text, "");
    }

    #[test]
    fn with_span_pushes_span() {
        let span = Span::new(0..5);
        let rt = RichText::plain("hello").with_span(span);
        assert_eq!(rt.spans.len(), 1);
        assert_eq!(rt.spans[0].range, 0..5);
    }

    #[test]
    fn with_span_chaining() {
        let rt = RichText::plain("hello world")
            .with_span(Span::new(0..5))
            .with_span(Span::new(6..11));
        assert_eq!(rt.spans.len(), 2);
    }

    #[test]
    fn span_new_defaults() {
        let s = Span::new(3..7);
        assert_eq!(s.range, 3..7);
        assert!(s.font_family.is_none());
        assert!(s.font_size.is_none());
        assert!(s.weight.is_none());
        assert!(s.italic.is_none());
        assert!(s.underline.is_none());
        assert!(s.strikethrough.is_none());
        assert!(s.color.is_none());
        assert!(s.line_height.is_none());
    }

    #[test]
    fn span_with_all_fields() {
        let mut s = Span::new(0..1);
        s.font_family = Some("monospace".into());
        s.font_size = Some(24.0);
        s.weight = Some(FontWeight::BOLD);
        s.italic = Some(true);
        s.underline = Some(true);
        s.strikethrough = Some(true);
        s.color = Some(peniko::Color::WHITE);
        s.line_height = Some(1.5);
        assert_eq!(s.font_family.as_deref(), Some("monospace"));
        assert_eq!(s.font_size, Some(24.0));
    }

    #[test]
    fn text_align_equality() {
        assert_eq!(TextAlign::Start, TextAlign::Start);
        assert_eq!(TextAlign::Center, TextAlign::Center);
        assert_eq!(TextAlign::End, TextAlign::End);
        assert_eq!(TextAlign::Justify, TextAlign::Justify);
        assert_ne!(TextAlign::Start, TextAlign::End);
    }

    #[test]
    fn layout_nonempty_text_has_positive_dims() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let rt = RichText::plain("Hello, world!");
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
        assert!(layout.width() > 0.0, "width should be > 0, got {}", layout.width());
        assert!(layout.height() > 0.0, "height should be > 0, got {}", layout.height());
    }

    #[test]
    fn layout_empty_text() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let rt = RichText::plain("");
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
        assert_eq!(layout.width(), 0.0);
    }

    #[test]
    fn layout_respects_max_width() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let long_text = "a ".repeat(200);
        let rt = RichText::plain(long_text);
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(100.0), viewport);
        assert!(layout.width() <= 101.0, "width {} should be <= 100", layout.width());
    }

    #[test]
    fn layout_with_spans() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let mut span = Span::new(0..5);
        span.weight = Some(FontWeight::BOLD);
        span.font_size = Some(32.0);
        let rt = RichText::plain("Hello World").with_span(span);
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
        assert!(layout.width() > 0.0);
        assert!(layout.height() > 0.0);
    }

    #[test]
    fn layout_with_default_italic() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let mut rt = RichText::plain("italic text");
        rt.default_italic = true;
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn layout_with_default_underline() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let mut rt = RichText::plain("underlined");
        rt.default_underline = true;
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn layout_with_default_strikethrough() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let mut rt = RichText::plain("struck");
        rt.default_strikethrough = true;
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn layout_with_default_line_height() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let mut rt = RichText::plain("line height");
        rt.default_line_height = Some(2.0);
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
        assert!(layout.height() > 0.0);
    }

    #[test]
    fn layout_with_all_span_properties() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let mut span = Span::new(0..5);
        span.font_family = Some("serif".into());
        span.font_size = Some(20.0);
        span.weight = Some(FontWeight::BOLD);
        span.italic = Some(true);
        span.underline = Some(true);
        span.strikethrough = Some(true);
        span.color = Some(peniko::Color::WHITE);
        span.line_height = Some(1.5);
        let rt = RichText::plain("Hello World").with_span(span);
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn layout_with_italic_false_span() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let mut span = Span::new(0..5);
        span.italic = Some(false);
        let rt = RichText::plain("Hello World").with_span(span);
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn layout_no_max_width() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let rt = RichText::plain("unconstrained");
        let viewport = Viewport { width: 1.0, height: 1.0 };
        let layout = rt.layout(&mut font_cx, &mut layout_cx, None, viewport);
        assert!(layout.width() > 0.0);
    }

    #[test]
    fn layout_all_alignments() {
        let mut font_cx = FontContext::new();
        let mut layout_cx = LayoutContext::new();
        let viewport = Viewport { width: 1.0, height: 1.0 };
        for align in [TextAlign::Start, TextAlign::Center, TextAlign::End, TextAlign::Justify] {
            let mut rt = RichText::plain("aligned");
            rt.align = align;
            let layout = rt.layout(&mut font_cx, &mut layout_cx, Some(500.0), viewport);
            assert!(layout.width() > 0.0);
        }
    }
}
