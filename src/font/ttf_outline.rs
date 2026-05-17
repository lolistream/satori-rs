//! Minimal TrueType glyf-table outline extractor that mirrors
//! `@shuding/opentype.js`'s `Glyph.getPath` byte-for-byte.
//!
//! Why a hand-rolled parser instead of `ttf-parser::Face::outline_glyph`?
//! `ttf-parser` produces a "minimal" outline — it skips redundant
//! zero-length `L`s after a `Q` ending on an on-curve point, and it
//! starts each contour at the first on-curve point of the contour.
//! `opentype.js` produces a "raw" outline — it iterates the contour
//! point list as-is, starts the `M` at `contour[contour.length - 1]`,
//! and emits exactly one drawing command per point (`L` for on-curve,
//! `Q` for off-curve). The two outputs describe the same closed
//! polygon but with different command sequences, and `resvg`'s
//! anti-aliasing of those slightly-different inputs differs at the
//! ULP level — enough to break our zero-tolerance image snapshot
//! tests. Re-implementing the JS traversal lets us byte-match.
//!
//! Only **simple** TrueType glyphs are handled here — composite
//! glyphs (used by `Roboto` for diacritic marks like `ä`, etc.) fall
//! back to `ttf-parser`'s outline. The cost of that fallback is a
//! handful of currently-failing diacritic tests; basic Latin and the
//! whole core Roman alphabet sit in simple-glyph land.

use ttf_parser::{Face, GlyphId, Tag};

/// One point from a simple glyph's contour array.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) on_curve: bool,
}

/// Per-glyph point list grouped by contour (TTF stores them as a flat
/// array with end-of-contour indices; we split them up here for ease
/// of consumption in `emit_simple_path`).
fn parse_simple_glyph(data: &[u8]) -> Option<Vec<Vec<Point>>> {
    if data.len() < 10 {
        return None;
    }
    let number_of_contours = i16::from_be_bytes([data[0], data[1]]);
    if number_of_contours <= 0 {
        return None; // composite or empty; caller falls back
    }
    let n_contours = number_of_contours as usize;
    // Skip bbox (4 × i16 = 8 bytes); points start at offset 10.
    let mut pos = 10usize;
    if data.len() < pos + 2 * n_contours + 2 {
        return None;
    }

    // End-of-contour indices (one u16 per contour, last value =
    // total-points - 1).
    let mut end_pts = Vec::with_capacity(n_contours);
    for _ in 0..n_contours {
        end_pts.push(u16::from_be_bytes([data[pos], data[pos + 1]]) as usize);
        pos += 2;
    }
    let n_points = *end_pts.last()? + 1;

    // Skip the (variable-length) hinting instructions.
    let instr_len = u16::from_be_bytes([data[pos], data[pos + 1]]) as usize;
    pos += 2 + instr_len;
    if pos > data.len() {
        return None;
    }

    // Decode the flags array — `repeat` flag (bit 3) can expand one
    // byte into several entries, so we loop until we've materialized
    // `n_points` flags.
    let mut flags = Vec::with_capacity(n_points);
    while flags.len() < n_points {
        if pos >= data.len() {
            return None;
        }
        let f = data[pos];
        pos += 1;
        flags.push(f);
        if f & 0x08 != 0 {
            // Next byte is the repeat count.
            if pos >= data.len() {
                return None;
            }
            let repeat = data[pos] as usize;
            pos += 1;
            for _ in 0..repeat {
                flags.push(f);
            }
        }
    }
    flags.truncate(n_points);

    // X-coordinates: each is delta-encoded; flag bits 1 & 4 control
    // whether the value is u8 vs i16 and whether it's same-as-previous.
    let mut xs = Vec::with_capacity(n_points);
    let mut prev: i32 = 0;
    for f in &flags {
        let x_short = f & 0x02 != 0;
        let x_same_or_pos = f & 0x10 != 0;
        let delta: i32 = if x_short {
            if pos >= data.len() {
                return None;
            }
            let v = data[pos] as i32;
            pos += 1;
            if x_same_or_pos { v } else { -v }
        } else if x_same_or_pos {
            0
        } else {
            if pos + 1 >= data.len() {
                return None;
            }
            let v = i16::from_be_bytes([data[pos], data[pos + 1]]) as i32;
            pos += 2;
            v
        };
        prev = prev.wrapping_add(delta);
        xs.push(prev);
    }

    // Y-coordinates: same encoding as X.
    let mut ys = Vec::with_capacity(n_points);
    let mut prev: i32 = 0;
    for f in &flags {
        let y_short = f & 0x04 != 0;
        let y_same_or_pos = f & 0x20 != 0;
        let delta: i32 = if y_short {
            if pos >= data.len() {
                return None;
            }
            let v = data[pos] as i32;
            pos += 1;
            if y_same_or_pos { v } else { -v }
        } else if y_same_or_pos {
            0
        } else {
            if pos + 1 >= data.len() {
                return None;
            }
            let v = i16::from_be_bytes([data[pos], data[pos + 1]]) as i32;
            pos += 2;
            v
        };
        prev = prev.wrapping_add(delta);
        ys.push(prev);
    }

    // Split flat point list into contours using `end_pts`.
    let mut contours: Vec<Vec<Point>> = Vec::with_capacity(n_contours);
    let mut start = 0usize;
    for &end in &end_pts {
        let mut contour = Vec::with_capacity(end - start + 1);
        for i in start..=end {
            contour.push(Point {
                x: xs[i],
                y: ys[i],
                on_curve: flags[i] & 0x01 != 0,
            });
        }
        contours.push(contour);
        start = end + 1;
    }
    Some(contours)
}

/// Look up the glyph data offset/length via the `loca` table.
///
/// `index_to_loc_format`: 0 → short (u16, half-offsets), 1 → long
/// (u32, byte-offsets). Read from the `head` table.
fn loca_range(loca: &[u8], head: &[u8], gid: u16) -> Option<(usize, usize)> {
    if head.len() < 52 {
        return None;
    }
    let format = i16::from_be_bytes([head[50], head[51]]);
    if format == 0 {
        // Short: offsets are u16 * 2 (stored as half-offsets).
        let i = gid as usize * 2;
        if i + 4 > loca.len() {
            return None;
        }
        let start = u16::from_be_bytes([loca[i], loca[i + 1]]) as usize * 2;
        let end = u16::from_be_bytes([loca[i + 2], loca[i + 3]]) as usize * 2;
        if end < start {
            return None;
        }
        Some((start, end))
    } else if format == 1 {
        let i = gid as usize * 4;
        if i + 8 > loca.len() {
            return None;
        }
        let start = u32::from_be_bytes([loca[i], loca[i + 1], loca[i + 2], loca[i + 3]]) as usize;
        let end =
            u32::from_be_bytes([loca[i + 4], loca[i + 5], loca[i + 6], loca[i + 7]]) as usize;
        if end < start {
            return None;
        }
        Some((start, end))
    } else {
        None
    }
}

/// Fetch the raw glyph bytes for `gid` and parse them as either a
/// simple glyph or a composite that expands into one (by transforming
/// each component glyph's points and concatenating). Returns `None`
/// only for byte-level decoding errors — composites that reference
/// missing glyphs return whatever they could expand. Callers fall
/// back to `ttf-parser`'s `outline_glyph` only when this returns
/// `None`.
pub(crate) fn get_simple_contours(
    face: &Face<'_>,
    gid: GlyphId,
) -> Option<Vec<Vec<Point>>> {
    let glyf = face.raw_face().table(Tag::from_bytes(b"glyf"))?;
    let loca = face.raw_face().table(Tag::from_bytes(b"loca"))?;
    let head = face.raw_face().table(Tag::from_bytes(b"head"))?;
    let (start, end) = loca_range(loca, head, gid.0)?;
    if start == end {
        return Some(Vec::new());
    }
    let data = glyf.get(start..end)?;
    if data.len() < 2 {
        return None;
    }
    let n = i16::from_be_bytes([data[0], data[1]]);
    if n >= 0 {
        return parse_simple_glyph(data);
    }
    // Composite glyph — recurse via `get_simple_contours` on each
    // component and concatenate the (transformed) point arrays. The
    // `lastPointOfContour` flag is preserved across the concat so
    // the downstream contour-splitting works unchanged.
    parse_composite_glyph(data, face, glyf, loca, head)
}

/// Parse a composite glyph and expand each component by recursively
/// fetching its referenced glyph's points + applying the per-component
/// 2×2 affine + offset. Mirror's opentype.js's `parseGlyph` composite
/// branch + `transformPoints`.
fn parse_composite_glyph(
    data: &[u8],
    face: &Face<'_>,
    glyf: &[u8],
    loca: &[u8],
    head: &[u8],
) -> Option<Vec<Vec<Point>>> {
    let mut all_points: Vec<Vec<Point>> = Vec::new();
    let mut pos = 10usize; // skip n_contours (2) + bbox (8)

    loop {
        if pos + 4 > data.len() {
            return None;
        }
        let flags = u16::from_be_bytes([data[pos], data[pos + 1]]);
        let child_gid = u16::from_be_bytes([data[pos + 2], data[pos + 3]]);
        pos += 4;

        // Parse offset arguments. Skip matched-point form (we don't
        // support it — Roboto's composites don't use it).
        let (dx, dy) = if (flags & 0x0001) != 0 {
            // ARGS_ARE_WORDS
            if pos + 4 > data.len() {
                return None;
            }
            let dx = i16::from_be_bytes([data[pos], data[pos + 1]]);
            let dy = i16::from_be_bytes([data[pos + 2], data[pos + 3]]);
            pos += 4;
            if (flags & 0x0002) == 0 {
                // matched points (uint16 × 2) — unsupported
                return None;
            }
            (dx as i32, dy as i32)
        } else {
            if pos + 2 > data.len() {
                return None;
            }
            let dx = data[pos] as i8;
            let dy = data[pos + 1] as i8;
            pos += 2;
            if (flags & 0x0002) == 0 {
                // matched points (byte × 2) — unsupported
                return None;
            }
            (dx as i32, dy as i32)
        };

        // 2×2 transform (F2DOT14 = signed fixed-point /16384).
        let (mut x_scale, mut scale01, mut scale10, mut y_scale) =
            (1.0f64, 0.0f64, 0.0f64, 1.0f64);
        let read_f2d14 = |p: usize| -> Option<f64> {
            if p + 2 > data.len() {
                return None;
            }
            let v = i16::from_be_bytes([data[p], data[p + 1]]);
            Some(v as f64 / 16384.0)
        };
        if (flags & 0x0008) != 0 {
            // WE_HAVE_A_SCALE
            x_scale = read_f2d14(pos)?;
            y_scale = x_scale;
            pos += 2;
        } else if (flags & 0x0040) != 0 {
            // WE_HAVE_AN_X_AND_Y_SCALE
            x_scale = read_f2d14(pos)?;
            y_scale = read_f2d14(pos + 2)?;
            pos += 4;
        } else if (flags & 0x0080) != 0 {
            // WE_HAVE_A_TWO_BY_TWO
            x_scale = read_f2d14(pos)?;
            scale01 = read_f2d14(pos + 2)?;
            scale10 = read_f2d14(pos + 4)?;
            y_scale = read_f2d14(pos + 6)?;
            pos += 8;
        }

        // Recurse to get the child glyph's contours.
        if let Some(child_contours) = get_simple_contours(face, GlyphId(child_gid)) {
            for contour in child_contours {
                let transformed: Vec<Point> = contour
                    .iter()
                    .map(|p| Point {
                        x: (x_scale * p.x as f64 + scale01 * p.y as f64 + dx as f64).round() as i32,
                        y: (scale10 * p.x as f64 + y_scale * p.y as f64 + dy as f64).round() as i32,
                        on_curve: p.on_curve,
                    })
                    .collect();
                all_points.push(transformed);
            }
        }

        if (flags & 0x0020) == 0 {
            // No MORE_COMPONENTS — done.
            break;
        }
    }

    // Suppress unused-warning lint for the table references we only
    // pass through to the recursive call.
    let _ = (glyf, loca, head);
    Some(all_points)
}

/// Emit `Glyph.getPath(x, y, fontSize).toPathData(1)`-equivalent SVG
/// path-data into `out` for one parsed simple-glyph point set.
///
/// Mirrors `opentype.js/src/tables/glyf.js:getPath`:
///   1. For each contour, set `M` at `contour[contour.length - 1]`
///      (or at the midpoint of last+first when both are off-curve).
///   2. Iterate `i = 0..contour.length`:
///       - on-curve → `lineTo(curr)`
///       - off-curve → `quadraticCurveTo(curr, next_anchor)` where
///         `next_anchor = next` if `next` is on-curve, else
///         `midpoint(curr, next)`.
///   3. Emit `Z`.
///
/// The transform `(font_size_px / units_per_em, baseline-relative,
/// Y-flipped)` is applied via the `PathDataBuilder` helpers so the
/// numeric formatting (rounding, sign-as-separator packing) matches
/// the JS side bit for bit.
pub(crate) fn emit_simple_path(
    contours: &[Vec<Point>],
    pen_x: f64,
    baseline_y: f64,
    scale: f64,
    dp: u32,
    out: &mut String,
) {
    for contour in contours {
        if contour.is_empty() {
            continue;
        }

        let last_idx = contour.len() - 1;
        let curr0 = contour[last_idx];
        let next0 = contour[0];

        let (m_x, m_y) = if curr0.on_curve {
            (curr0.x as f64, curr0.y as f64)
        } else if next0.on_curve {
            (next0.x as f64, next0.y as f64)
        } else {
            (
                (curr0.x as f64 + next0.x as f64) * 0.5,
                (curr0.y as f64 + next0.y as f64) * 0.5,
            )
        };
        emit_move(out, pen_x + m_x * scale, baseline_y - m_y * scale, dp);

        for i in 0..contour.len() {
            let prev = if i == 0 { contour[last_idx] } else { contour[i - 1] };
            let curr = contour[i];
            let next = contour[(i + 1) % contour.len()];
            if curr.on_curve {
                emit_line(out, pen_x + curr.x as f64 * scale, baseline_y - curr.y as f64 * scale, dp);
            } else {
                // Off-curve control. `next2` is the anchor point of
                // the quadratic segment: if `next` is on-curve use it
                // directly, otherwise use the midpoint of `curr` and
                // `next` (this is how a chain of consecutive
                // off-curve points decomposes into multiple Qs).
                let _ = prev;
                let (nx, ny) = if next.on_curve {
                    (next.x as f64, next.y as f64)
                } else {
                    ((curr.x as f64 + next.x as f64) * 0.5,
                     (curr.y as f64 + next.y as f64) * 0.5)
                };
                emit_quad(
                    out,
                    pen_x + curr.x as f64 * scale,
                    baseline_y - curr.y as f64 * scale,
                    pen_x + nx * scale,
                    baseline_y - ny * scale,
                    dp,
                );
            }
        }
        out.push('Z');
    }
}

fn emit_move(out: &mut String, x: f64, y: f64, dp: u32) {
    out.push('M');
    pack(out, &[x, y], dp);
}

fn emit_line(out: &mut String, x: f64, y: f64, dp: u32) {
    out.push('L');
    pack(out, &[x, y], dp);
}

fn emit_quad(out: &mut String, x1: f64, y1: f64, x: f64, y: f64, dp: u32) {
    out.push('Q');
    pack(out, &[x1, y1, x, y], dp);
}

fn pack(out: &mut String, values: &[f64], dp: u32) {
    for (i, &v) in values.iter().enumerate() {
        if v >= 0.0 && i > 0 {
            out.push(' ');
        }
        out.push_str(&super::path_d::js_float_to_string(v, dp));
    }
}
