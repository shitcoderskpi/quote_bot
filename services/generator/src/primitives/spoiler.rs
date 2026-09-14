use std::ops::Range;
use parley::{GlyphRun, Run};
use vello::peniko::Brush;
use crate::primitives::text::Span;

pub struct SpoilerSegment {
    pub start: f64,
    pub end: f64,
    pub text_range: Range<usize>,
}

fn is_overlapping(span: &Span, cluster_range: &Range<usize>) -> bool {
    cluster_range.start < span.range.end &&
        span.range.start < cluster_range.end
}


pub fn spoiler_segments<'a>(
    glyph_run: &GlyphRun<Brush>,
    run: &Run<Brush>,
    spans: impl Iterator<Item = &'a Span>,
) -> Vec<SpoilerSegment> {
    let run_start_x = glyph_run.offset() as f64;
    let mut segments = Vec::new();

    for span in spans {
        let mut cx = run_start_x;
        let mut seg: Option<(f64, usize)> = None;
        let mut seg_end_x = cx;
        let mut seg_end_byte = 0usize;

        for cluster in run.visual_clusters() {
            let cluster_range = cluster.text_range();
            let cw = cluster.advance() as f64;

            if is_overlapping(span, &cluster_range) {
                if seg.is_none() {
                    seg = Some((cx, cluster_range.start));
                }
                seg_end_x = cx + cw;
                seg_end_byte = cluster_range.end;
            } else if seg.is_some() {
                break;
            }

            cx += cw;
        }

        if let Some((x_start, byte_start)) = seg {
            segments.push(SpoilerSegment { start: x_start, end: seg_end_x, text_range: byte_start..seg_end_byte });
        }
    }

    segments
}