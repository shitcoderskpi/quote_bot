use parley::*;
use vello::kurbo::Affine;
use vello::peniko::{self, Fill};
use vello::Scene;

use crate::parser::{Alignment, TextMessage};

fn parse_hex_color(s: &str) -> [u8; 4] {
    let s = s.trim_start_matches('#');
    let r = u8::from_str_radix(s.get(0..2).unwrap_or("ff"), 16).unwrap_or(255);
    let g = u8::from_str_radix(s.get(2..4).unwrap_or("ff"), 16).unwrap_or(255);
    let b = u8::from_str_radix(s.get(4..6).unwrap_or("ff"), 16).unwrap_or(255);
    let a = u8::from_str_radix(s.get(6..8).unwrap_or("ff"), 16).unwrap_or(255);
    [r, g, b, a]
}

pub fn draw_text_layers(
    scene: &mut Scene,
    texts: &[TextMessage],
    font_cx: &mut FontContext,
    layout_cx: &mut LayoutContext,
) {
    for entry in texts {
        let color = parse_hex_color(&entry.color);

        let mut builder = layout_cx.ranged_builder(font_cx, &entry.text, 1.0, true);
        builder.push_default(StyleProperty::FontFamily(FontFamily::Source(
            entry.font_family.as_str().into(),
        )));
        builder.push_default(StyleProperty::FontSize(entry.font_size * 96.0 / 72.0));
        builder.push_default(StyleProperty::FontWeight(FontWeight::new(entry.font_weight as f32)));
        builder.push_default(StyleProperty::Brush(color));

        let mut layout: Layout<[u8; 4]> = builder.build(&entry.text);

        let max_width = if entry.wrap_width > 0 {
            Some(entry.wrap_width as f32)
        } else {
            None
        };
        layout.break_all_lines(max_width);

        let alignment = match entry.alignment {
            Alignment::Left => parley::Alignment::Start,
            Alignment::Center => parley::Alignment::Center,
            Alignment::Right => parley::Alignment::End,
        };
        layout.align(alignment, AlignmentOptions::default());

        let x = entry.x as f64;
        let y = entry.y as f64;

        for line in layout.lines() {
            for item in line.items() {
                if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                    let run = glyph_run.run();
                    let font = run.font().clone();
                    let font_size = run.font_size();
                    let style = glyph_run.style();
                    let brush = &style.brush;

                    let coords: Vec<_> = run
                        .normalized_coords()
                        .iter()
                        .map(|c| *c)
                        .collect();

                    let glyphs = glyph_run.positioned_glyphs().map(|g| vello::Glyph {
                        id: g.id as u32,
                        x: g.x,
                        y: g.y,
                    });

                    let peniko_color = vello::peniko::Color::from_rgba8(brush[0], brush[1], brush[2], brush[3]);

                    scene
                        .draw_glyphs(&font)
                        .font_size(font_size)
                        .transform(Affine::translate((x, y)))
                        .normalized_coords(&coords)
                        .brush(peniko_color)
                        .draw(Fill::NonZero, glyphs);
                }
            }
        }
    }
}
