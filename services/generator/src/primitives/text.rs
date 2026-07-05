//! Rich text: a plain string plus a list of styled spans (font family,
//! size, weight, italic, color/paint, line height), laid out with `parley`
//! and re-wrapped to whatever width the box resolves to. Height is
//! intrinsic -- it comes out of the layout, which is what lets a text node
//! act like an "auto" box on the cross axis.

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
    pub font_size: f64,
    pub weight: FontWeight,
    pub italic: bool,
    pub color: vello::peniko::Color,
    pub line_height: Option<f32>,
}

impl Span {
    pub fn new(range: Range<usize>) -> Self {
        Self {
            range,
            font_family: None,
            font_size: 16.0,
            weight: FontWeight::NORMAL,
            italic: false,
            color: vello::peniko::Color::BLACK,
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
    pub default_font_size: f64,
    pub default_family: String,
    pub default_color: vello::peniko::Color,
    pub default_weight: FontWeight,
    /// Root font size, used to resolve `Em` lengths inside this block.
    pub root_font_size: f64,
}

impl RichText {
    pub fn plain(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            spans: Vec::new(),
            align: TextAlign::Start,
            default_font_size: 16.0,
            default_family: "system-ui".to_string(),
            default_color: vello::peniko::Color::BLACK,
            default_weight: FontWeight::NORMAL,
            root_font_size: 16.0,
        }
    }

    pub fn with_span(mut self, span: Span) -> Self {
        self.spans.push(span);
        self
    }

    /// Build and word-wrap a parley layout for this text at a given
    /// resolved box width. Pass `None` for `max_width` to measure at
    /// intrinsic (unwrapped) width instead -- useful for "auto" width boxes.
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

        for span in &self.spans {
            builder.push(StyleProperty::FontSize(span.font_size as f32), span.range.clone());
            builder.push(StyleProperty::FontWeight(span.weight), span.range.clone());
            if span.italic {
                builder.push(StyleProperty::FontStyle(parley::FontStyle::Italic), span.range.clone());
            }
            builder.push(
                StyleProperty::Brush(Brush::Solid(span.color)),
                span.range.clone(),
            );
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
        layout.break_all_lines(max_width.map(|w| w as f32));
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
