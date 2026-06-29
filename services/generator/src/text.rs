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

        for ent in &entry.entities {
            let range = ent.offset..(ent.offset + ent.length).min(entry.text.len());
            match ent.ty.as_str() {
                "bold" => {
                    builder.push(StyleProperty::FontWeight(FontWeight::BOLD), range);
                }
                "italic" => {
                    builder.push(StyleProperty::FontStyle(parley::FontStyle::Italic), range);
                }
                "underline" => {
                    builder.push(StyleProperty::Underline(true), range);
                }
                "strikethrough" => {
                    builder.push(StyleProperty::Strikethrough(true), range);
                }
                "code" | "pre" => {
                    builder.push(StyleProperty::FontFamily(FontFamily::Source("monospace".into())), range.clone());
                    if let Some(c) = &entry.monospace_color {
                        let brush = parse_hex_color(c);
                        builder.push(StyleProperty::Brush(brush), range.clone());
                    }
                }
                "text_link" | "url" => {
                    builder.push(StyleProperty::Underline(true), range.clone());
                    if let Some(c) = &entry.link_color {
                        let brush = parse_hex_color(c);
                        builder.push(StyleProperty::Brush(brush), range.clone());
                    }
                }
                _ => {}
            }
        }

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

        let baseline_offset = if entry.valignment == crate::parser::VAlignment::Baseline {
            layout.lines().next().map(|l| l.metrics().baseline).unwrap_or(0.0) as f64
        } else {
            0.0
        };

        let x = entry.x as f64;
        let y = entry.y as f64 - baseline_offset;

        if let Some(bg_color_str) = &entry.bg_color {
            let bg_arr = parse_hex_color(bg_color_str);
            let bg_color = peniko::Color::from_rgba8(bg_arr[0], bg_arr[1], bg_arr[2], bg_arr[3]);
            let mut min_x = f64::MAX;
            let mut max_x = f64::MIN;
            for line in layout.lines() {
                for item in line.items() {
                    if let PositionedLayoutItem::GlyphRun(glyph_run) = item {
                        let start = glyph_run.offset() as f64;
                        let end = start + glyph_run.advance() as f64;
                        if start < min_x { min_x = start; }
                        if end > max_x { max_x = end; }
                    }
                }
            }
            if min_x > max_x {
                min_x = 0.0;
                max_x = layout.width() as f64;
            }

            let text_h = layout.height() as f64;
            
            let pad_x = 6.0;
            let pad_y = -1.0;
            let rx = x + min_x - pad_x;
            let ry = y - pad_y;
            let rw = (max_x - min_x) + pad_x * 2.0;
            let rh = text_h + pad_y * 2.0;
            let radius = rh / 2.0; // Fully rounded

            let rect = vello::kurbo::RoundedRect::new(rx, ry, rx + rw, ry + rh, radius);
            scene.fill(
                Fill::NonZero,
                Affine::IDENTITY,
                bg_color,
                None,
                &rect,
            );
        }

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

                    let peniko_color = peniko::Color::from_rgba8(brush[0], brush[1], brush[2], brush[3]);

                    scene
                        .draw_glyphs(&font)
                        .font_size(font_size)
                        .transform(Affine::translate((x, y)))
                        .normalized_coords(&coords)
                        .brush(peniko_color)
                        .draw(Fill::NonZero, glyphs);

                    let metrics = run.metrics();
                    if style.strikethrough.is_some() {
                        let y_offset = y + glyph_run.baseline() as f64 - metrics.strikethrough_offset as f64;
                        let thickness = metrics.strikethrough_size as f64;
                        let rect = vello::kurbo::Rect::new(
                            x + glyph_run.offset() as f64,
                            y_offset,
                            x + glyph_run.offset() as f64 + glyph_run.advance() as f64,
                            y_offset + thickness,
                        );
                        scene.fill(Fill::NonZero, Affine::IDENTITY, peniko_color, None, &rect);
                    }

                    if style.underline.is_some() {
                        let y_offset = y + glyph_run.baseline() as f64 - metrics.underline_offset as f64;
                        let thickness = metrics.underline_size as f64;
                        let rect = vello::kurbo::Rect::new(
                            x + glyph_run.offset() as f64,
                            y_offset,
                            x + glyph_run.offset() as f64 + glyph_run.advance() as f64,
                            y_offset + thickness,
                        );
                        scene.fill(Fill::NonZero, Affine::IDENTITY, peniko_color, None, &rect);
                    }
                }
            }
        }
    }
}
