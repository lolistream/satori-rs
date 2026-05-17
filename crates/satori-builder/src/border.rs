//! Port of `src/builder/border.ts`.
//!
//! Renders CSS borders as SVG stroke paths. To approximate CSS borders
//! (which are inside the element) using SVG strokes (which are centered),
//! the JS satori doubles the stroke width and applies a clip-path that
//! constrains the stroke to the inside of the element.
//!
//! When all four sides share width/style/color the JS emits a single
//! stroked path (or rect). When they differ, JS emits one `<path>` per
//! side using the `partialSides` path generator — we do the same here.

use satori_css::style::{BorderStyle, ComputedStyle};

use crate::border_radius::{radius_path, radius_path_partial};
use crate::xml::{build_xml, js_number_to_string, AttrValue};

pub struct BorderArgs<'a> {
    pub id: &'a str,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    /// Existing transform attribute to copy onto stroke path / clip path.
    pub matrix: Option<String>,
    pub current_clip_path: Option<String>,
}

/// Return `(defs_xml, body_xml)`. `defs_xml` includes the clip path used
/// to constrain the doubled-stroke; `body_xml` is the actual stroke shape.
///
/// When the element has no border, returns `("", "")`.
pub fn render_border(args: &BorderArgs<'_>, style: &ComputedStyle) -> (String, String) {
    let tw = style.border_top_width.unwrap_or(0.0);
    let rw = style.border_right_width.unwrap_or(0.0);
    let bw = style.border_bottom_width.unwrap_or(0.0);
    let lw = style.border_left_width.unwrap_or(0.0);
    if tw == 0.0 && rw == 0.0 && bw == 0.0 && lw == 0.0 {
        return (String::new(), String::new());
    }

    // JS path: walk the 4 directions, group consecutive equal
    // (width, style, color) tuples into a single stroked path; otherwise
    // emit per-side paths.
    let directions = [
        (tw, style.border_top_style, style.border_top_color.clone()),
        (rw, style.border_right_style, style.border_right_color.clone()),
        (bw, style.border_bottom_style, style.border_bottom_color.clone()),
        (lw, style.border_left_style, style.border_left_color.clone()),
    ];
    let all_equal = directions.iter().all(|d| d == &directions[0]);
    if !all_equal {
        return render_directional_border(args, style, &directions);
    }

    // For the minimal port, treat all 4 sides as a single uniform border.
    // Use the top side's width/color/style as the canonical values.
    // Empty color falls back to currentColor → element's color.
    let color = style
        .border_top_color
        .clone()
        .or(style.border_right_color.clone())
        .or(style.border_bottom_color.clone())
        .or(style.border_left_color.clone())
        .or_else(|| style.color.clone())
        .unwrap_or_else(|| "#000000".to_string());

    let bstyle = style
        .border_top_style
        .or(style.border_right_style)
        .or(style.border_bottom_style)
        .or(style.border_left_style)
        .unwrap_or(BorderStyle::Solid);

    if matches!(bstyle, BorderStyle::None | BorderStyle::Hidden) {
        return (String::new(), String::new());
    }

    // Use the widest side as the stroke width approximation. JS satori
    // generates per-side strokes; we approximate using a uniform stroke.
    let stroke_width = tw.max(rw).max(bw).max(lw);

    // Clip path that constrains the doubled stroke to the element shape.
    let rect_clip_id = format!("satori_bc-{}", args.id);
    let path_d = radius_path(args.left, args.top, args.width, args.height, style);
    let inner_shape = if path_d.is_empty() {
        build_xml(
            "rect",
            &[
                ("x", AttrValue::Number(args.left)),
                ("y", AttrValue::Number(args.top)),
                ("width", AttrValue::Number(args.width)),
                ("height", AttrValue::Number(args.height)),
            ],
            None,
        )
    } else {
        build_xml(
            "path",
            &[
                ("x", AttrValue::Number(args.left)),
                ("y", AttrValue::Number(args.top)),
                ("width", AttrValue::Number(args.width)),
                ("height", AttrValue::Number(args.height)),
                ("d", AttrValue::Owned(path_d.clone())),
            ],
            None,
        )
    };
    let defs = build_xml(
        "clipPath",
        &[("id", AttrValue::Str(rect_clip_id.as_str()))],
        Some(&inner_shape),
    );

    // Stroke shape — doubled stroke clipped to the element interior.
    let stroke_dasharray = match bstyle {
        BorderStyle::Dashed => Some(format!(
            "{} {}",
            js_number_to_string(stroke_width * 2.0),
            js_number_to_string(stroke_width)
        )),
        BorderStyle::Dotted => Some(format!(
            "{} {}",
            js_number_to_string(stroke_width),
            js_number_to_string(stroke_width)
        )),
        _ => None,
    };

    let stroke_path = if path_d.is_empty() {
        build_xml(
            "rect",
            &[
                ("x", AttrValue::Number(args.left)),
                ("y", AttrValue::Number(args.top)),
                ("width", AttrValue::Number(args.width)),
                ("height", AttrValue::Number(args.height)),
                ("fill", AttrValue::Str("none")),
                ("stroke", AttrValue::Str(color.as_str())),
                ("stroke-width", AttrValue::Number(stroke_width * 2.0)),
                ("stroke-dasharray", AttrValue::from(stroke_dasharray.clone())),
                ("clip-path", AttrValue::Owned(format!("url(#{rect_clip_id})"))),
                ("transform", AttrValue::from(args.matrix.clone())),
            ],
            None,
        )
    } else {
        build_xml(
            "path",
            &[
                ("width", AttrValue::Number(args.width)),
                ("height", AttrValue::Number(args.height)),
                ("fill", AttrValue::Str("none")),
                ("stroke", AttrValue::Str(color.as_str())),
                ("stroke-width", AttrValue::Number(stroke_width * 2.0)),
                ("stroke-dasharray", AttrValue::from(stroke_dasharray)),
                ("clip-path", AttrValue::Owned(format!("url(#{rect_clip_id})"))),
                ("transform", AttrValue::from(args.matrix.clone())),
                ("d", AttrValue::Owned(path_d)),
            ],
            None,
        )
    };

    (defs, stroke_path)
}

/// Per-side `<path>` emission for the directional border case. Mirrors
/// JS `src/builder/border.ts`'s `partialSides` walk: consecutive sides
/// with identical (width, style, color) tuples are merged into a single
/// stroked path generated by `radius_path_partial`.
fn render_directional_border(
    args: &BorderArgs<'_>,
    style: &ComputedStyle,
    directions: &[(f32, Option<BorderStyle>, Option<String>); 4],
) -> (String, String) {
    let path_d = radius_path(args.left, args.top, args.width, args.height, style);
    let has_radius = !path_d.is_empty();
    let rect_clip_id = format!("satori_bc-{}", args.id);
    let inner_shape = if has_radius {
        build_xml(
            "path",
            &[
                ("x", AttrValue::Number(args.left)),
                ("y", AttrValue::Number(args.top)),
                ("width", AttrValue::Number(args.width)),
                ("height", AttrValue::Number(args.height)),
                ("d", AttrValue::Str(path_d.as_str())),
            ],
            None,
        )
    } else {
        build_xml(
            "rect",
            &[
                ("x", AttrValue::Number(args.left)),
                ("y", AttrValue::Number(args.top)),
                ("width", AttrValue::Number(args.width)),
                ("height", AttrValue::Number(args.height)),
            ],
            None,
        )
    };
    let defs = build_xml(
        "clipPath",
        &[("id", AttrValue::Str(rect_clip_id.as_str()))],
        Some(&inner_shape),
    );

    let same = |i: usize, j: usize| directions[i] == directions[j];

    let mut body = String::new();
    let mut partial = [false; 4];
    let mut current_idx: usize = 0;
    for _i in 0..4 {
        let i = _i;
        let ni = (_i + 1) % 4;
        partial[i] = true;
        current_idx = i;
        if !same(i, ni) {
            emit_segment(args, style, directions, &rect_clip_id, partial, current_idx, &mut body);
            partial = [false; 4];
        }
    }
    if partial.iter().any(|x| *x) {
        emit_segment(args, style, directions, &rect_clip_id, partial, current_idx, &mut body);
    }

    (defs, body)
}

fn emit_segment(
    args: &BorderArgs<'_>,
    style: &ComputedStyle,
    directions: &[(f32, Option<BorderStyle>, Option<String>); 4],
    rect_clip_id: &str,
    partial: [bool; 4],
    current_idx: usize,
    body: &mut String,
) {
    let (w_val, st_val, c_val) = &directions[current_idx];
    let width_val = *w_val;
    if width_val == 0.0 {
        return;
    }
    let bstyle = st_val.unwrap_or(BorderStyle::Solid);
    if matches!(bstyle, BorderStyle::None | BorderStyle::Hidden) {
        return;
    }
    let color = c_val
        .clone()
        .or_else(|| style.color.clone())
        .unwrap_or_else(|| "#000000".to_string());
    let stroke_w = width_val * 2.0;
    let stroke_dasharray = match bstyle {
        BorderStyle::Dashed => Some(format!(
            "{} {}",
            js_number_to_string(stroke_w),
            js_number_to_string(width_val),
        )),
        BorderStyle::Dotted => Some(format!(
            "{} {}",
            js_number_to_string(width_val),
            js_number_to_string(width_val),
        )),
        _ => None,
    };
    let d_attr = radius_path_partial(args.left, args.top, args.width, args.height, style, partial);
    body.push_str(&build_xml(
        "path",
        &[
            ("width", AttrValue::Number(args.width)),
            ("height", AttrValue::Number(args.height)),
            ("clip-path", AttrValue::Owned(format!("url(#{rect_clip_id})"))),
            ("transform", AttrValue::from(args.matrix.clone())),
            ("fill", AttrValue::Str("none")),
            ("stroke", AttrValue::Str(color.as_str())),
            ("stroke-width", AttrValue::Number(stroke_w)),
            ("stroke-dasharray", AttrValue::from(stroke_dasharray)),
            ("d", AttrValue::Owned(d_attr)),
        ],
        None,
    ));
}

/// JS contentMask emits the directional border paths inside the mask
/// with `stroke="#000"` (carving the border area out so children don't
/// paint over the border). Mirrors `border(... asContentMask: true)`.
/// `mask_border_only=true` means only borders are excluded;
/// `mask_border_only=false` also adds padding to the stroke widths
/// (used when there's an image so the content-area mask excludes the
/// padding too).
pub fn content_mask_border(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    style: &ComputedStyle,
    mask_border_only: bool,
) -> String {
    let tw = style.border_top_width.unwrap_or(0.0);
    let rw = style.border_right_width.unwrap_or(0.0);
    let bw = style.border_bottom_width.unwrap_or(0.0);
    let lw = style.border_left_width.unwrap_or(0.0);
    let p_top = if !mask_border_only { dim_px(style.padding_top) } else { 0.0 };
    let p_right = if !mask_border_only { dim_px(style.padding_right) } else { 0.0 };
    let p_bottom = if !mask_border_only { dim_px(style.padding_bottom) } else { 0.0 };
    let p_left = if !mask_border_only { dim_px(style.padding_left) } else { 0.0 };
    let widths = [tw + p_top, rw + p_right, bw + p_bottom, lw + p_left];
    if widths.iter().all(|w| *w == 0.0) {
        return String::new();
    }
    // Group consecutive sides that share width/style/color so they
    // become a single stroked path inside the mask (matches JS
    // `partialSides` walk in `border.ts` with `asContentMask=true`).
    let directions = [
        (tw, style.border_top_style, style.border_top_color.clone()),
        (rw, style.border_right_style, style.border_right_color.clone()),
        (bw, style.border_bottom_style, style.border_bottom_color.clone()),
        (lw, style.border_left_style, style.border_left_color.clone()),
    ];
    let same = |i: usize, j: usize| directions[i] == directions[j];

    let mut out = String::new();
    let mut partial = [false; 4];
    let mut current_idx: usize = 0;
    for _i in 0..4 {
        partial[_i] = true;
        current_idx = _i;
        let ni = (_i + 1) % 4;
        if !same(_i, ni) {
            emit_mask_segment(left, top, width, height, style, partial, current_idx, &widths, &mut out);
            partial = [false; 4];
        }
    }
    if partial.iter().any(|x| *x) {
        emit_mask_segment(left, top, width, height, style, partial, current_idx, &widths, &mut out);
    }
    out
}

fn emit_mask_segment(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    style: &ComputedStyle,
    partial: [bool; 4],
    current_idx: usize,
    widths: &[f32; 4],
    out: &mut String,
) {
    let w_val = widths[current_idx];
    if w_val == 0.0 {
        return;
    }
    let d_attr = radius_path_partial(left, top, width, height, style, partial);
    if d_attr.is_empty() {
        return;
    }
    out.push_str(&build_xml(
        "path",
        &[
            ("width", AttrValue::Number(width)),
            ("height", AttrValue::Number(height)),
            ("fill", AttrValue::Str("none")),
            ("stroke", AttrValue::Str("#000")),
            ("stroke-width", AttrValue::Number(w_val * 2.0)),
            ("d", AttrValue::Owned(d_attr)),
        ],
        None,
    ));
}

fn dim_px(d: Option<satori_css::dimension::Dim>) -> f32 {
    match d {
        Some(satori_css::dimension::Dim::Px(n)) => n,
        _ => 0.0,
    }
}
