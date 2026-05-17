//! Port of `src/builder/border-radius.ts`.
//!
//! Generates an SVG `d` path for a rounded rectangle. Returns the empty
//! string when all corners are 0 (so callers fall back to a plain `<rect>`).

use crate::css::style::{ComputedStyle, RadiusValue};

use super::xml::js_number_to_string;

fn resolve_corner(r: Option<RadiusValue>, width: f32, height: f32) -> (bool, Option<[f32; 2]>) {
    match r {
        None => (true, None),
        Some(v) => {
            let h = v.h.resolve(width).min(width);
            let vv = v.v.resolve(height).min(height);
            (v.single, Some([h, vv]))
        }
    }
}

fn resolve_size(a: f32, b: f32, limit: f32) -> (f32, f32) {
    let mut a = a;
    let mut b = b;
    if limit < a + b {
        if limit / 2.0 < a && limit / 2.0 < b {
            a = limit / 2.0;
            b = limit / 2.0;
        } else if limit / 2.0 < a {
            a = limit - b;
        } else if limit / 2.0 < b {
            b = limit - a;
        }
    }
    (a, b)
}

fn make_smaller(arr: &mut [f32; 2]) {
    let m = arr[0].min(arr[1]);
    arr[0] = m;
    arr[1] = m;
}

/// Compute the 45-degree intersection offset used by partial-arc starts/ends.
fn svg_arc_center_offset(r: [f32; 2]) -> f32 {
    let rx = r[0];
    let ry = r[1];
    if (rx * 1000.0).round() == 0.0 && (ry * 1000.0).round() == 0.0 {
        return 0.0;
    }
    let raw = (rx * ry) / (rx * rx + ry * ry).sqrt();
    (raw * 1000.0).round() / 1000.0
}

/// Resolved corner radii after the JS clamp/`makeSmaller` cascade.
struct Corners {
    tl: [f32; 2],
    tr: [f32; 2],
    bl: [f32; 2],
    br: [f32; 2],
}

fn resolve_corners(width: f32, height: f32, style: &ComputedStyle) -> Option<Corners> {
    let (s_tl, tl) = resolve_corner(style.border_top_left_radius, width, height);
    let (s_tr, tr) = resolve_corner(style.border_top_right_radius, width, height);
    let (s_bl, bl) = resolve_corner(style.border_bottom_left_radius, width, height);
    let (s_br, br) = resolve_corner(style.border_bottom_right_radius, width, height);

    if tl.is_none() && tr.is_none() && bl.is_none() && br.is_none() {
        return None;
    }

    let mut tl = tl.unwrap_or([0.0, 0.0]);
    let mut tr = tr.unwrap_or([0.0, 0.0]);
    let mut bl = bl.unwrap_or([0.0, 0.0]);
    let mut br = br.unwrap_or([0.0, 0.0]);

    (tl[0], tr[0]) = resolve_size(tl[0], tr[0], width);
    (bl[0], br[0]) = resolve_size(bl[0], br[0], width);
    (tl[1], bl[1]) = resolve_size(tl[1], bl[1], height);
    (tr[1], br[1]) = resolve_size(tr[1], br[1], height);

    if s_tl {
        make_smaller(&mut tl);
    }
    if s_tr {
        make_smaller(&mut tr);
    }
    if s_bl {
        make_smaller(&mut bl);
    }
    if s_br {
        make_smaller(&mut br);
    }

    Some(Corners { tl, tr, bl, br })
}

/// Returns the SVG path `d` string for the rounded-rect outline,
/// starting at the top-left corner: `M<l+tlR>,<t> h... a... v... a... h... a... v... a...`.
///
/// Returns empty string when no corner has a non-zero radius (callers
/// should emit a plain `<rect>` instead).
pub fn radius_path(left: f32, top: f32, width: f32, height: f32, style: &ComputedStyle) -> String {
    let Some(Corners { tl, tr, bl, br }) = resolve_corners(width, height, style) else {
        return String::new();
    };

    // Do the path arithmetic in f64 so values like `100 - 16.8 - 32`
    // round-trip to `35.2` (not `35.199997`) — JS Number is f64 and
    // we need the printed string to match byte-for-byte.
    let left = left as f64;
    let top = top as f64;
    let width = width as f64;
    let height = height as f64;
    let tl = [tl[0] as f64, tl[1] as f64];
    let tr = [tr[0] as f64, tr[1] as f64];
    let bl = [bl[0] as f64, bl[1] as f64];
    let br = [br[0] as f64, br[1] as f64];

    let mut s = String::new();
    s.push('M');
    s.push_str(&super::xml::js_number_to_string_f64(left + tl[0]));
    s.push(',');
    s.push_str(&super::xml::js_number_to_string_f64(top));
    s.push_str(" h");
    s.push_str(&super::xml::js_number_to_string_f64(width - tl[0] - tr[0]));
    s.push_str(" a");
    s.push_str(&super::xml::js_number_to_string_f64(tr[0])); s.push(','); s.push_str(&super::xml::js_number_to_string_f64(tr[1]));
    s.push_str(" 0 0 1 ");
    s.push_str(&super::xml::js_number_to_string_f64(tr[0])); s.push(','); s.push_str(&super::xml::js_number_to_string_f64(tr[1]));
    s.push_str(" v");
    s.push_str(&super::xml::js_number_to_string_f64(height - tr[1] - br[1]));
    s.push_str(" a");
    s.push_str(&super::xml::js_number_to_string_f64(br[0])); s.push(','); s.push_str(&super::xml::js_number_to_string_f64(br[1]));
    s.push_str(" 0 0 1 ");
    s.push_str(&super::xml::js_number_to_string_f64(-br[0])); s.push(','); s.push_str(&super::xml::js_number_to_string_f64(br[1]));
    s.push_str(" h");
    s.push_str(&super::xml::js_number_to_string_f64(br[0] + bl[0] - width));
    s.push_str(" a");
    s.push_str(&super::xml::js_number_to_string_f64(bl[0])); s.push(','); s.push_str(&super::xml::js_number_to_string_f64(bl[1]));
    s.push_str(" 0 0 1 ");
    s.push_str(&super::xml::js_number_to_string_f64(-bl[0])); s.push(','); s.push_str(&super::xml::js_number_to_string_f64(-bl[1]));
    s.push_str(" v");
    s.push_str(&super::xml::js_number_to_string_f64(bl[1] + tl[1] - height));
    s.push_str(" a");
    s.push_str(&super::xml::js_number_to_string_f64(tl[0])); s.push(','); s.push_str(&super::xml::js_number_to_string_f64(tl[1]));
    s.push_str(" 0 0 1 ");
    s.push_str(&super::xml::js_number_to_string_f64(tl[0])); s.push(','); s.push_str(&super::xml::js_number_to_string_f64(-tl[1]));

    s
}

/// JS `radius(..., partialSides)` port. Generates a path that traces a
/// subset of the rounded-rect outline (typically a single side or a
/// run of consecutive sides), starting and ending with half-arcs at the
/// 45-degree intersection of the adjacent corners. Used by the
/// directional-border builder so that the SVG strokes for differently
/// styled sides meet at the corner midpoints.
///
/// `partial_sides[i]` selects sides in the order Top, Right, Bottom, Left.
pub fn radius_path_partial(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    style: &ComputedStyle,
    partial_sides: [bool; 4],
) -> String {
    let Some(Corners { tl, tr, bl, br }) = resolve_corners(width, height, style) else {
        // Fallback when no radius: build a plain partial-side path.
        return radius_path_partial_no_radius(left, top, width, height, partial_sides);
    };

    // p[i][0] = arc radii for the corner after side i (matches JS
    // index where p[3] is top-left, p[0] is top-right, etc.).
    let corner_radii: [[f32; 2]; 4] = [tr, br, bl, tl];

    // Side path segments T, R, B, L (relative arcs).
    let t = format!(
        "h{} a{},{} 0 0 1 {},{}",
        js_number_to_string(width - tl[0] - tr[0]),
        js_number_to_string(tr[0]),
        js_number_to_string(tr[1]),
        js_number_to_string(tr[0]),
        js_number_to_string(tr[1]),
    );
    let r = format!(
        "v{} a{},{} 0 0 1 {},{}",
        js_number_to_string(height - tr[1] - br[1]),
        js_number_to_string(br[0]),
        js_number_to_string(br[1]),
        js_number_to_string(-br[0]),
        js_number_to_string(br[1]),
    );
    let b = format!(
        "h{} a{},{} 0 0 1 {},{}",
        js_number_to_string(br[0] + bl[0] - width),
        js_number_to_string(bl[0]),
        js_number_to_string(bl[1]),
        js_number_to_string(-bl[0]),
        js_number_to_string(-bl[1]),
    );
    let l = format!(
        "v{} a{},{} 0 0 1 {},{}",
        js_number_to_string(bl[1] + tl[1] - height),
        js_number_to_string(tl[0]),
        js_number_to_string(tl[1]),
        js_number_to_string(tl[0]),
        js_number_to_string(-tl[1]),
    );
    let sides = [t, r, b, l];

    if !partial_sides.iter().any(|x| *x) {
        return String::new();
    }
    let mut start = partial_sides.iter().position(|x| !*x).unwrap_or(0);
    if !partial_sides.iter().any(|x| !*x) {
        start = 0;
    } else {
        while !partial_sides[start] {
            start = (start + 1) % 4;
        }
    }

    let get_arc = |i: usize| -> [[f32; 2]; 2] {
        // `i` is constrained to 0..4 at every call site (it indexes
        // `partial_sides: [bool; 4]` via `(start + n) % 4`), so both
        // matches below are exhaustive over its actual range. Keep
        // variants 0..=3 written out explicitly and the `_` arm as
        // an annotated panic so a future caller passing `i >= 4` is
        // caught immediately rather than producing a wrong-corner
        // arc (audit #5).
        let c0 = svg_arc_center_offset(match i {
            0 => tl,
            1 => tr,
            2 => br,
            3 => bl,
            _ => unreachable!(
                "border-radius corner index {i} out of range \
                 (must be 0..4 - index is `(start + n) % 4`)"
            ),
        });
        match i {
            0 => [
                [left + tl[0] - c0, top + tl[1] - c0],
                [left + tl[0], top],
            ],
            1 => [
                [left + width - tr[0] + c0, top + tr[1] - c0],
                [left + width, top + tr[1]],
            ],
            2 => [
                [left + width - br[0] + c0, top + height - br[1] + c0],
                [left + width - br[0], top + height],
            ],
            3 => [
                [left + bl[0] - c0, top + height - bl[1] + c0],
                [left, top + height - bl[1]],
            ],
            _ => unreachable!(
                "border-radius corner index {i} out of range \
                 (must be 0..4 - index is `(start + n) % 4`)"
            ),
        }
    };

    let mut result = String::new();
    let arc0 = get_arc(start);
    let mut seg = format!(
        "M{},{} A{},{} 0 0 1 {},{}",
        js_number_to_string(arc0[0][0]),
        js_number_to_string(arc0[0][1]),
        js_number_to_string(corner_radii[(start + 3) % 4][0]),
        js_number_to_string(corner_radii[(start + 3) % 4][1]),
        js_number_to_string(arc0[1][0]),
        js_number_to_string(arc0[1][1]),
    );

    let mut len = 0usize;
    while len < 4 && partial_sides[(start + len) % 4] {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&seg);
        seg = sides[(start + len) % 4].clone();
        len += 1;
    }
    let end = (start + len) % 4;

    // For the last segment, take only the leading side instruction
    // (`h…` / `v…`) and replace the trailing relative arc with the
    // half-arc to the end-corner midpoint.
    let leading = seg.split(' ').next().unwrap_or("").to_string();
    if !result.is_empty() {
        result.push(' ');
    }
    result.push_str(&leading);
    let arc1 = get_arc(end);
    result.push_str(&format!(
        " A{},{} 0 0 1 {},{}",
        js_number_to_string(corner_radii[(end + 3) % 4][0]),
        js_number_to_string(corner_radii[(end + 3) % 4][1]),
        js_number_to_string(arc1[0][0]),
        js_number_to_string(arc1[0][1]),
    ));

    result
}

/// No-radius fallback: render the subset of sides as straight segments
/// joined with degenerate arcs (matches JS's "A0,0 0 0 1" half-arcs).
fn radius_path_partial_no_radius(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    partial_sides: [bool; 4],
) -> String {
    let l = left;
    let t = top;
    let w = width;
    let h = height;
    let r = l + w;
    let b = t + h;
    // Per-side full paths (each is the JS "M c0 A0,0 0 0 1 c0 <side> A0,0 0 0 1 c1").
    let per_side = [
        format!(
            "M{},{} A0,0 0 0 1 {},{} h{} A0,0 0 0 1 {},{}",
            js_number_to_string(l), js_number_to_string(t),
            js_number_to_string(l), js_number_to_string(t),
            js_number_to_string(w),
            js_number_to_string(r), js_number_to_string(t),
        ),
        format!(
            "M{},{} A0,0 0 0 1 {},{} v{} A0,0 0 0 1 {},{}",
            js_number_to_string(r), js_number_to_string(t),
            js_number_to_string(r), js_number_to_string(t),
            js_number_to_string(h),
            js_number_to_string(r), js_number_to_string(b),
        ),
        format!(
            "M{},{} A0,0 0 0 1 {},{} h{} A0,0 0 0 1 {},{}",
            js_number_to_string(r), js_number_to_string(b),
            js_number_to_string(r), js_number_to_string(b),
            js_number_to_string(-w),
            js_number_to_string(l), js_number_to_string(b),
        ),
        format!(
            "M{},{} A0,0 0 0 1 {},{} v{} A0,0 0 0 1 {},{}",
            js_number_to_string(l), js_number_to_string(b),
            js_number_to_string(l), js_number_to_string(b),
            js_number_to_string(-h),
            js_number_to_string(l), js_number_to_string(t),
        ),
    ];
    let mut out = String::new();
    for i in 0..4 {
        if partial_sides[i] {
            if !out.is_empty() {
                out.push(' ');
            }
            out.push_str(&per_side[i]);
        }
    }
    out
}
