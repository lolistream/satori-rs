//! Port of `src/builder/text-decoration.ts`.

use crate::css::style::{ComputedStyle, TextDecorationLine, TextDecorationStyle};

use super::xml::{build_xml, AttrValue};

#[derive(Debug, Clone, Copy)]
pub struct GlyphBox {
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
}

pub struct DecorationArgs<'a> {
    pub width: f32,
    pub left: f32,
    /// f64 to keep `top + ascender * 1.1` (underline) and
    /// `top + ascender * 0.7` (line-through) bit-exact with JS satori
    /// — the f32 round-trip on `top` flipped the `<line>` y by ~1 ULP
    /// (e.g. `65.32000007629395` vs `65.32`).
    pub top: f64,
    pub ascender: f64,
    pub clip_path_id: Option<&'a str>,
    pub matrix: Option<&'a str>,
    pub glyph_boxes: &'a [GlyphBox],
}

pub fn build_decoration(args: &DecorationArgs<'_>, style: &ComputedStyle) -> String {
    let Some(line) = style.text_decoration_line else { return String::new() };
    if matches!(line, TextDecorationLine::None) {
        return String::new();
    }

    // JS satori does all decoration math in f64. We promote here so
    // values like `top + ascender * 1.1` keep the full mantissa
    // (otherwise `96.7109375f32` would format as `"96.71094"`, not
    // the byte-identical `"96.7109375"`).
    let font_size = style.font_size.unwrap_or(16.0) as f64;
    let height = (1.0_f64).max(font_size * 0.1);

    let top = args.top;
    let ascender = args.ascender;
    let y = match line {
        TextDecorationLine::LineThrough => top + ascender * 0.7,
        TextDecorationLine::Underline => top + ascender * 1.1,
        _ => top,
    };

    let decoration_style = style
        .text_decoration_style
        .unwrap_or(TextDecorationStyle::Solid);
    let dasharray = match decoration_style {
        TextDecorationStyle::Dashed => Some(format!(
            "{} {}",
            js_num_f64(height * 1.2),
            js_num_f64(height * 2.0)
        )),
        TextDecorationStyle::Dotted => Some(format!("0 {}", js_num_f64(height * 2.0))),
        _ => None,
    };

    let stroke_str = style
        .text_decoration_color
        .clone()
        .or_else(|| style.color.clone())
        .unwrap_or_else(|| "black".to_string());

    let skip_ink_kw = style
        .text_decoration_skip_ink
        .as_deref()
        .unwrap_or("auto");
    let apply_skip_ink = matches!(line, TextDecorationLine::Underline)
        && skip_ink_kw != "none"
        && !args.glyph_boxes.is_empty();

    let baseline = top + ascender;
    let segments: Vec<(f64, f64)> = if apply_skip_ink {
        build_skip_ink_segments_f64(
            args.left as f64,
            (args.left + args.width) as f64,
            args.glyph_boxes,
            y,
            height,
            baseline,
        )
    } else {
        vec![(args.left as f64, (args.left + args.width) as f64)]
    };

    let linecap = if matches!(decoration_style, TextDecorationStyle::Dotted) {
        "round"
    } else {
        "square"
    };

    let mut body = String::new();
    if let Some(id) = args.clip_path_id {
        body.push_str(&format!("<g clip-path=\"url(#{id})\">"));
    }

    let line_one = |x1: f64, x2: f64, y_pos: f64| -> String {
        let mut attrs: Vec<(&str, AttrValue)> = vec![
            ("x1", AttrValue::NumberF64(x1)),
            ("y1", AttrValue::NumberF64(y_pos)),
            ("x2", AttrValue::NumberF64(x2)),
            ("y2", AttrValue::NumberF64(y_pos)),
            ("stroke", AttrValue::Owned(stroke_str.clone())),
            ("stroke-width", AttrValue::NumberF64(height)),
        ];
        if let Some(d) = &dasharray {
            attrs.push(("stroke-dasharray", AttrValue::Owned(d.clone())));
        }
        attrs.push(("stroke-linecap", AttrValue::Str(linecap)));
        // JS satori's `container()` returns matrix as `""` (empty
        // string) when there's no transform, and buildXMLString only
        // skips `undefined`; the result is a literal `transform=""`
        // attribute on every decoration line. We emit that explicitly
        // to keep the SVG byte-identical.
        attrs.push((
            "transform",
            match args.matrix {
                Some(m) => AttrValue::Str(m),
                None => AttrValue::Str(""),
            },
        ));
        build_xml("line", &attrs, None)
    };

    for (x1, x2) in &segments {
        body.push_str(&line_one(*x1, *x2, y));
    }

    if matches!(decoration_style, TextDecorationStyle::Double) {
        for (x1, x2) in &segments {
            body.push_str(&line_one(*x1, *x2, y + height + 1.0));
        }
    }

    if args.clip_path_id.is_some() {
        body.push_str("</g>");
    }
    body
}

fn js_num_f64(f: f64) -> String {
    if f == f.trunc() && f.is_finite() {
        format!("{}", f as i64)
    } else {
        format!("{}", f)
    }
}

/// `build_skip_ink_segments` adapted to f64 segments. The glyph boxes
/// stay f32 (sourced from `satori-font::GlyphBox`); only the segment
/// arithmetic is f64 so the emitted attribute strings match JS satori's
/// double-precision number formatting.
fn build_skip_ink_segments_f64(
    start: f64,
    end: f64,
    glyph_boxes: &[GlyphBox],
    y: f64,
    stroke_width: f64,
    baseline: f64,
) -> Vec<(f64, f64)> {
    let half_stroke = stroke_width / 2.0;
    let bleed = half_stroke.max(stroke_width * 1.25);
    let mut skip_ranges: Vec<(f64, f64)> = Vec::new();

    for b in glyph_boxes {
        let y1 = b.y1 as f64;
        let y2 = b.y2 as f64;
        let x1 = b.x1 as f64;
        let x2 = b.x2 as f64;
        if y2 < baseline + half_stroke || y1 > y + half_stroke {
            continue;
        }
        let from = start.max(x1 - bleed);
        let to = end.min(x2 + bleed);
        if from >= to {
            continue;
        }
        if skip_ranges.is_empty() {
            skip_ranges.push((from, to));
            continue;
        }
        let last = skip_ranges.last_mut().unwrap();
        if from <= last.1 {
            last.1 = last.1.max(to);
        } else {
            skip_ranges.push((from, to));
        }
    }

    if skip_ranges.is_empty() {
        return vec![(start, end)];
    }

    let mut segments: Vec<(f64, f64)> = Vec::new();
    let mut cursor = start;
    for (from, to) in &skip_ranges {
        if *from > cursor {
            segments.push((cursor, *from));
        }
        cursor = cursor.max(*to);
        if cursor >= end {
            break;
        }
    }
    if cursor < end {
        segments.push((cursor, end));
    }
    segments
}
