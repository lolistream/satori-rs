//! Parsed TTF/OTF binary backed by a self-owned `Vec<u8>`.
//!
//! `ttf_parser::Face` borrows its byte slice for the entire lifetime of
//! the face. We want a single owned struct that downstream crates can
//! pass around freely, so we store the bytes inside an `Arc<Vec<u8>>`
//! and expose helpers that re-create a short-lived `Face` for each
//! query. The cost of re-parsing a face header on each call is
//! negligible compared to the rasterization downstream.

use std::sync::Arc;

/// Mirror of JS satori's `GlyphBox` type. Represents a subset of a
/// glyph's bounding box that crosses the underline band; the
/// rasterizer uses these to break the underline around descenders
/// ("skip ink").
#[derive(Debug, Clone, Copy)]
pub struct GlyphBox {
    pub x1: f32,
    pub x2: f32,
    pub y1: f32,
    pub y2: f32,
}

/// Mirror of JS satori's `SkipInkBand` type — the Y range plus stroke
/// width used to slice glyph outlines into ink / non-ink regions.
#[derive(Debug, Clone, Copy)]
pub struct SkipInkBand {
    pub underline_y: f32,
    pub stroke_width: f32,
}

#[derive(Clone)]
pub struct ParsedFont {
    bytes: Arc<Vec<u8>>,
    units_per_em: u16,
    ascender: i16,
    descender: i16,
    line_gap: i16,
    /// OS/2 typo metrics (preferred when available).
    typo_ascender: Option<i16>,
    typo_descender: Option<i16>,
    typo_line_gap: Option<i16>,
    /// Lowercased Postscript-style family name; useful for diagnostics.
    pub family_name: String,
}

impl ParsedFont {
    pub fn parse(bytes: Vec<u8>) -> Option<Self> {
        let arc = Arc::new(bytes);
        let face = ttf_parser::Face::parse(&arc, 0).ok()?;
        let units_per_em = face.units_per_em();
        let ascender = face.ascender();
        let descender = face.descender();
        let line_gap = face.line_gap();
        let typo_ascender = face.typographic_ascender();
        let typo_descender = face.typographic_descender();
        let typo_line_gap = face.typographic_line_gap();
        let family_name = face
            .names()
            .into_iter()
            .find_map(|n| {
                if n.name_id == ttf_parser::name_id::FAMILY && n.is_unicode() {
                    n.to_string()
                } else {
                    None
                }
            })
            .unwrap_or_default();
        Some(Self {
            bytes: arc,
            units_per_em,
            ascender,
            descender,
            line_gap,
            typo_ascender,
            typo_descender,
            typo_line_gap,
            family_name,
        })
    }

    pub fn units_per_em(&self) -> u16 {
        self.units_per_em
    }

    /// Resolved "hhea" ascender in font units. Mirrors JS satori's
    /// `_ascender = (useOS2Table ? os2?.sTypoAscender : 0) || font.ascender`
    /// — we currently always use the hhea ascender (matches the JS
    /// default `useOS2Table = false`).
    pub fn ascender_units(&self) -> i16 {
        self.ascender
    }
    pub fn descender_units(&self) -> i16 {
        self.descender
    }
    pub fn line_gap_units(&self) -> i16 {
        self.line_gap
    }

    pub fn typo_ascender_units(&self) -> Option<i16> {
        self.typo_ascender
    }
    pub fn typo_descender_units(&self) -> Option<i16> {
        self.typo_descender
    }
    pub fn typo_line_gap_units(&self) -> Option<i16> {
        self.typo_line_gap
    }

    /// Returns `true` if the font has a glyph for the given codepoint.
    pub fn has_char(&self, ch: char) -> bool {
        let Ok(face) = ttf_parser::Face::parse(&self.bytes, 0) else {
            return false;
        };
        face.glyph_index(ch).is_some()
    }

    /// Horizontal advance width for `ch` in font units. Returns 0 for
    /// missing glyphs (mirrors `getAdvanceWidth` ignoring missing
    /// glyphs in JS).
    ///
    /// Mirrors opentype.js's `charToGlyph` fall-back: when the font has
    /// no `cmap` entry for `ch`, return the `.notdef` glyph's advance
    /// width (glyph index 0) rather than `0`. Without this, control
    /// characters like `\n` get measured as 0px in Rust but as the
    /// `.notdef` width in JS, which throws line-height accumulation
    /// off by ~0.1px on every multi-line `white-space: pre` fixture.
    pub fn h_advance_units(&self, ch: char) -> u16 {
        let Ok(face) = ttf_parser::Face::parse(&self.bytes, 0) else {
            return 0;
        };
        let gid = face.glyph_index(ch).unwrap_or(ttf_parser::GlyphId(0));
        face.glyph_hor_advance(gid).unwrap_or(0)
    }

    /// Sum of horizontal advances for the string, in font units.
    /// (Naive — no kerning, no shaping. Matches the JS `<text>` emitter's
    /// width accumulation, which is also a naive sum of advances.)
    pub fn measure_units(&self, s: &str) -> f32 {
        let Ok(face) = ttf_parser::Face::parse(&self.bytes, 0) else {
            return 0.0;
        };
        let mut total: u32 = 0;
        for ch in s.chars() {
            if let Some(gid) = face.glyph_index(ch) {
                total += face.glyph_hor_advance(gid).unwrap_or(0) as u32;
            }
        }
        total as f32
    }

    pub fn bytes(&self) -> Arc<Vec<u8>> {
        Arc::clone(&self.bytes)
    }

    /// Build the SVG path-data string for a single glyph at baseline
    /// origin `(x, y)`, drawn at `font_size` px.
    ///
    /// Returns `None` if the font has no glyph for the given codepoint.
    /// The output is byte-identical to
    /// `font.charToGlyph(ch).getPath(x, y, font_size).toPathData(dp)`
    /// from `@shuding/opentype.js`.
    pub fn glyph_path_d(&self, ch: char, x: f64, y: f64, font_size: f64, dp: u32) -> Option<String> {
        let face = ttf_parser::Face::parse(&self.bytes, 0).ok()?;
        let gid = face.glyph_index(ch)?;
        let scale = (1.0 / self.units_per_em as f64) * font_size;
        if let Some(contours) = crate::ttf_outline::get_simple_contours(&face, gid) {
            let mut out = String::new();
            crate::ttf_outline::emit_simple_path(&contours, x, y, scale, dp, &mut out);
            return Some(out);
        }
        let mut b = crate::path_d::PathDataBuilder::new(x, y, scale, dp);
        let _ = face.outline_glyph(gid, &mut b);
        Some(b.out)
    }

    /// Mirror of `font.forEachGlyph(text, x, y, fontSize, options, cb)`
    /// + `glyph.getPath(...).toPathData(1)` from `opentype.js`. Returns
    /// the concatenated SVG path-data string for the run, applying:
    ///   - per-glyph outline scaled by `font_size / unitsPerEm`
    ///   - Y-flip to SVG conventions
    ///   - inter-glyph horizontal advance (kerning **not** yet applied)
    ///   - `letterSpacing` added to the pen after every glyph
    ///   - characters with no glyph in this font are skipped (the
    ///     caller is responsible for font fallback)
    pub fn run_path_d(
        &self,
        text: &str,
        x: f64,
        y: f64,
        font_size: f64,
        letter_spacing: f64,
        apply_kerning: bool,
        dp: u32,
    ) -> String {
        self.run_path_d_with_fallback(text, x, y, font_size, letter_spacing, apply_kerning, dp, &[])
    }

    /// Like `run_path_d`, but characters missing from this font are
    /// resolved against `fallback` (in order) and rendered using the
    /// first font that has them. Mirror's JS satori's
    /// `patchFontFallbackResolver` in `font.ts`: the fallback glyph's
    /// path is scaled by `primary.unitsPerEm / fallback.unitsPerEm`
    /// so the final path-data stays in the primary font's
    /// coordinate system (uses the primary's `scale` everywhere).
    /// The advance also comes from the fallback, scaled the same way.
    pub fn run_path_d_with_fallback(
        &self,
        text: &str,
        x: f64,
        y: f64,
        font_size: f64,
        letter_spacing: f64,
        apply_kerning: bool,
        dp: u32,
        fallback: &[Arc<ParsedFont>],
    ) -> String {
        let Ok(face) = ttf_parser::Face::parse(&self.bytes, 0) else {
            return String::new();
        };
        let scale = (1.0 / self.units_per_em as f64) * font_size;
        let mut out = String::new();
        let mut pen_x = x;
        let mut prev: Option<ttf_parser::GlyphId> = None;
        for ch in text.chars() {
            if (ch as u32) < 0x20 {
                continue;
            }
            // Try the primary first; fall back to the first font in
            // `fallback` that has a glyph for `ch`.
            if let Some(gid) = face.glyph_index(ch) {
                if apply_kerning {
                    if let Some(p) = prev {
                        let k = kerning_units(&face, p, gid);
                        pen_x += (k as f64) * scale;
                    }
                }
                if let Some(contours) = crate::ttf_outline::get_simple_contours(&face, gid) {
                    crate::ttf_outline::emit_simple_path(&contours, pen_x, y, scale, dp, &mut out);
                } else {
                    let mut b = crate::path_d::PathDataBuilder::new(pen_x, y, scale, dp);
                    let _ = face.outline_glyph(gid, &mut b);
                    out.push_str(&b.out);
                }
                let adv = face.glyph_hor_advance(gid).unwrap_or(0);
                pen_x += (adv as f64) * scale;
                if letter_spacing != 0.0 {
                    pen_x += letter_spacing;
                }
                prev = Some(gid);
                continue;
            }
            // Primary doesn't have it. Try fallbacks (skip self).
            let mut resolved = None;
            for f in fallback {
                if std::ptr::eq(f.as_ref(), self) {
                    continue;
                }
                if f.has_char(ch) {
                    resolved = Some(Arc::clone(f));
                    break;
                }
            }
            let Some(fb) = resolved else {
                // No font has it — emit notdef (gid 0) from primary.
                let gid = ttf_parser::GlyphId(0);
                if let Some(contours) = crate::ttf_outline::get_simple_contours(&face, gid) {
                    crate::ttf_outline::emit_simple_path(&contours, pen_x, y, scale, dp, &mut out);
                } else {
                    let mut b = crate::path_d::PathDataBuilder::new(pen_x, y, scale, dp);
                    let _ = face.outline_glyph(gid, &mut b);
                    out.push_str(&b.out);
                }
                let adv = face.glyph_hor_advance(gid).unwrap_or(0);
                pen_x += (adv as f64) * scale;
                if letter_spacing != 0.0 {
                    pen_x += letter_spacing;
                }
                prev = None; // notdef glyphs don't kern across boundaries
                continue;
            };
            // Render the fallback glyph at the primary's scale, and
            // advance the pen by the FALLBACK's own advance (its
            // glyph_advance × font_size / fallback.upm). JS satori's
            // `patchFontFallbackResolver` does:
            //   advanceWidth = fallback.advance × (primary.upm /
            //                                     fallback.upm)
            // which `forEachGlyph` then multiplies by `1 / primary.upm
            // × font_size`, collapsing to `fallback.advance ×
            // font_size / fallback.upm`. Mirror that here.
            let Ok(fb_face) = ttf_parser::Face::parse(&fb.bytes, 0) else {
                prev = None;
                continue;
            };
            let Some(fb_gid) = fb_face.glyph_index(ch) else {
                prev = None;
                continue;
            };
            let upm_ratio = self.units_per_em as f64 / fb.units_per_em as f64;
            let fb_scale = scale * upm_ratio;
            if let Some(contours) = crate::ttf_outline::get_simple_contours(&fb_face, fb_gid) {
                crate::ttf_outline::emit_simple_path(&contours, pen_x, y, fb_scale, dp, &mut out);
            } else {
                let mut b = crate::path_d::PathDataBuilder::new(pen_x, y, fb_scale, dp);
                let _ = fb_face.outline_glyph(fb_gid, &mut b);
                out.push_str(&b.out);
            }
            let fb_adv = fb_face.glyph_hor_advance(fb_gid).unwrap_or(0);
            pen_x += (fb_adv as f64) * fb_scale;
            if letter_spacing != 0.0 {
                pen_x += letter_spacing;
            }
            prev = None; // kerning across font boundaries is not tracked
        }
        out
    }

    /// Mirror of `font.getAdvanceWidth(text, fontSize, { letterSpacing })`
    /// — sum of per-glyph advances + GPOS/legacy kerning between
    /// adjacent glyphs + `letterSpacing` after every glyph.
    pub fn measure_advance(
        &self,
        text: &str,
        font_size: f64,
        letter_spacing: f64,
    ) -> f64 {
        let Ok(face) = ttf_parser::Face::parse(&self.bytes, 0) else {
            return 0.0;
        };
        let scale = (1.0 / self.units_per_em as f64) * font_size;
        let mut pen_x = 0.0_f64;
        let mut prev: Option<ttf_parser::GlyphId> = None;
        for ch in text.chars() {
            let gid = face
                .glyph_index(ch)
                .unwrap_or(ttf_parser::GlyphId(0));
            if let Some(p) = prev {
                let k = kerning_units(&face, p, gid);
                pen_x += (k as f64) * scale;
            }
            let adv = face.glyph_hor_advance(gid).unwrap_or(0);
            pen_x += (adv as f64) * scale;
            pen_x += letter_spacing;
            prev = Some(gid);
        }
        pen_x
    }

    /// Mirror of JS satori's `getSVG` band-box collection (`font.ts`'s
    /// `forEachGlyph` + `computeBandBox`). Walks each glyph in `text`
    /// the same way `run_path_d` does, flattens its outline into line
    /// segments at device coordinates, and returns the
    /// underline-skip-ink boxes that intersect the supplied band.
    pub fn run_band_boxes(
        &self,
        text: &str,
        x: f64,
        y: f64,
        font_size: f64,
        letter_spacing: f64,
        apply_kerning: bool,
        band: SkipInkBand,
    ) -> Vec<GlyphBox> {
        self.run_band_boxes_with_fallback(text, x, y, font_size, letter_spacing, apply_kerning, band, &[])
    }

    /// Like `run_band_boxes`, but resolves each char against the
    /// `fallback` list when missing from the primary, so a mixed
    /// Latin/CJK line still produces skip-ink boxes for descenders
    /// in the fallback font (e.g. `你` in a CJK fallback font next
    /// to Roboto-rendered Latin). Mirror's `run_path_d_with_fallback`
    /// — uses the primary's scale for path coordinates by scaling
    /// the fallback outline through `upm_ratio = primary.upm /
    /// fallback.upm`, and advances the pen by `fallback.advance ×
    /// font_size / fallback.upm` so positions align with the render
    /// pass.
    pub fn run_band_boxes_with_fallback(
        &self,
        text: &str,
        x: f64,
        y: f64,
        font_size: f64,
        letter_spacing: f64,
        apply_kerning: bool,
        band: SkipInkBand,
        fallback: &[Arc<ParsedFont>],
    ) -> Vec<GlyphBox> {
        let Ok(face) = ttf_parser::Face::parse(&self.bytes, 0) else {
            return Vec::new();
        };
        let scale = (1.0 / self.units_per_em as f64) * font_size;
        let mut boxes: Vec<GlyphBox> = Vec::new();
        let mut pen_x = x;
        let mut prev: Option<ttf_parser::GlyphId> = None;
        for ch in text.chars() {
            if (ch as u32) < 0x20 {
                continue;
            }
            if let Some(gid) = face.glyph_index(ch) {
                if apply_kerning {
                    if let Some(p) = prev {
                        let k = kerning_units(&face, p, gid);
                        pen_x += (k as f64) * scale;
                    }
                }
                if let Some(contours) = crate::ttf_outline::get_simple_contours(&face, gid) {
                    let segments = flatten_simple_contours(&contours, pen_x, y, scale);
                    if !segments.is_empty() {
                        let mut sub = compute_band_boxes(&segments, &band);
                        boxes.append(&mut sub);
                    }
                }
                let adv = face.glyph_hor_advance(gid).unwrap_or(0);
                pen_x += (adv as f64) * scale;
                if letter_spacing != 0.0 {
                    pen_x += letter_spacing;
                }
                prev = Some(gid);
                continue;
            }
            // Fallback path: pick the first fallback that has `ch`.
            let mut resolved = None;
            for f in fallback {
                if std::ptr::eq(f.as_ref(), self) {
                    continue;
                }
                if f.has_char(ch) {
                    resolved = Some(Arc::clone(f));
                    break;
                }
            }
            let Some(fb) = resolved else {
                prev = None;
                continue;
            };
            let Ok(fb_face) = ttf_parser::Face::parse(&fb.bytes, 0) else {
                prev = None;
                continue;
            };
            let Some(fb_gid) = fb_face.glyph_index(ch) else {
                prev = None;
                continue;
            };
            let upm_ratio = self.units_per_em as f64 / fb.units_per_em as f64;
            let fb_scale = scale * upm_ratio;
            if let Some(contours) = crate::ttf_outline::get_simple_contours(&fb_face, fb_gid) {
                let segments = flatten_simple_contours(&contours, pen_x, y, fb_scale);
                if !segments.is_empty() {
                    let mut sub = compute_band_boxes(&segments, &band);
                    boxes.append(&mut sub);
                }
            }
            let fb_adv = fb_face.glyph_hor_advance(fb_gid).unwrap_or(0);
            pen_x += (fb_adv as f64) * fb_scale;
            if letter_spacing != 0.0 {
                pen_x += letter_spacing;
            }
            prev = None;
            continue;
        }
        let _ = prev;
        boxes
    }
}

/// Mirror of `flattenPath` in `reference/src/font.ts` — converts a
/// simple-glyph contour list into a flat list of line segments at the
/// final device coordinates `(pen_x + cmd.x * scale, baseline_y -
/// cmd.y * scale)`. Quadratic on-curve / off-curve interleaving is
/// handled the same way as `emit_simple_path`, with each Q segment
/// sampled into 12 sub-segments to mirror `addCurve(..., 12)`.
fn flatten_simple_contours(
    contours: &[Vec<crate::ttf_outline::Point>],
    pen_x: f64,
    baseline_y: f64,
    scale: f64,
) -> Vec<(f64, f64, f64, f64)> {
    let mut segs: Vec<(f64, f64, f64, f64)> = Vec::new();
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
        let start = (pen_x + m_x * scale, baseline_y - m_y * scale);
        let mut current = start;
        for i in 0..contour.len() {
            let curr = contour[i];
            let next = contour[(i + 1) % contour.len()];
            if curr.on_curve {
                let to = (
                    pen_x + curr.x as f64 * scale,
                    baseline_y - curr.y as f64 * scale,
                );
                segs.push((current.0, current.1, to.0, to.1));
                current = to;
            } else {
                let (nx, ny) = if next.on_curve {
                    (next.x as f64, next.y as f64)
                } else {
                    (
                        (curr.x as f64 + next.x as f64) * 0.5,
                        (curr.y as f64 + next.y as f64) * 0.5,
                    )
                };
                let ctrl = (
                    pen_x + curr.x as f64 * scale,
                    baseline_y - curr.y as f64 * scale,
                );
                let end = (pen_x + nx * scale, baseline_y - ny * scale);
                let steps = 12;
                let mut prev_pt = current;
                for s in 1..=steps {
                    let t = s as f64 / steps as f64;
                    let mt = 1.0 - t;
                    let px = mt * mt * current.0
                        + 2.0 * mt * t * ctrl.0
                        + t * t * end.0;
                    let py = mt * mt * current.1
                        + 2.0 * mt * t * ctrl.1
                        + t * t * end.1;
                    segs.push((prev_pt.0, prev_pt.1, px, py));
                    prev_pt = (px, py);
                }
                current = end;
            }
        }
        segs.push((current.0, current.1, start.0, start.1));
    }
    segs
}

/// Mirror of `computeBandBox` in `reference/src/font.ts` — samples the
/// flattened path at fixed Y intervals across the supplied band, marks
/// the column ranges that the path's interior covers, and turns the
/// resulting clusters into `GlyphBox`es. Includes the "bleed" pad and
/// `min(x1, minX) - bleed` / `max(x2, maxX) + bleed` widening that JS
/// satori applies before returning.
fn compute_band_boxes(segments: &[(f64, f64, f64, f64)], band: &SkipInkBand) -> Vec<GlyphBox> {
    let stroke_width = band.stroke_width as f64;
    let band_min = band.underline_y as f64 - stroke_width * 0.25;
    let band_max = band.underline_y as f64 + stroke_width * 2.5;
    let band_height = band_max - band_min;
    if band_height <= 0.0 {
        return Vec::new();
    }
    let y_samples = ((band_height / 0.25).ceil() as usize).max(12);
    let y_step = band_height / y_samples as f64;
    let y_start = band_min + y_step / 2.0;
    let mut column_hits: std::collections::BTreeSet<i64> = std::collections::BTreeSet::new();
    let mut intersections: Vec<f64> = Vec::with_capacity(16);
    for i in 0..y_samples {
        let y = y_start + y_step * i as f64;
        intersections.clear();
        for &(x1, y1, x2, y2) in segments {
            if (y1 - y2).abs() < f64::EPSILON {
                continue;
            }
            let y_min = y1.min(y2);
            let y_max = y1.max(y2);
            if y < y_min || y >= y_max {
                continue;
            }
            let t = (y - y1) / (y2 - y1);
            let x = x1 + (x2 - x1) * t;
            intersections.push(x);
        }
        if intersections.is_empty() {
            continue;
        }
        intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut j = 0;
        while j + 1 < intersections.len() {
            let from = intersections[j].min(intersections[j + 1]);
            let to = intersections[j].max(intersections[j + 1]);
            let start = from.floor() as i64;
            let end = to.ceil() as i64;
            for col in start..end {
                column_hits.insert(col);
            }
            j += 2;
        }
    }
    if column_hits.is_empty() {
        return Vec::new();
    }
    let columns: Vec<i64> = column_hits.into_iter().collect();
    let mut ink_ranges: Vec<(i64, i64)> = Vec::new();
    let mut range_start = columns[0];
    let mut prev_c = columns[0];
    for &col in columns.iter().skip(1) {
        if col > prev_c + 1 {
            ink_ranges.push((range_start, prev_c + 1));
            range_start = col;
        }
        prev_c = col;
    }
    ink_ranges.push((range_start, prev_c + 1));
    let bleed = stroke_width * 0.6;
    let min_x = ink_ranges[0].0;
    let max_x = ink_ranges.last().unwrap().1;
    let mut boxes: Vec<GlyphBox> = Vec::new();
    for &(x1, x2) in &ink_ranges {
        let left = (x1.min(min_x) as f64) - bleed;
        let right = (x2.max(max_x) as f64) + bleed;
        boxes.push(GlyphBox {
            x1: left as f32,
            x2: right as f32,
            y1: band_min as f32,
            y2: band_max as f32,
        });
    }
    boxes
}

/// Lookup the kerning value (in font design units) between two
/// adjacent glyphs. Mirrors `opentype.js`'s `Font.getKerningValue`:
/// prefers GPOS pair-positioning kerning, falls back to the legacy
/// `kern` table.
///
/// Returns `0` when no kerning is defined.
pub(crate) fn kerning_units(
    face: &ttf_parser::Face<'_>,
    left: ttf_parser::GlyphId,
    right: ttf_parser::GlyphId,
) -> i32 {
    // 1. GPOS pair-positioning lookups (modern fonts including Roboto).
    if let Some(gpos) = face.tables().gpos {
        if let Some(k) = gpos_kern_value(gpos, left, right) {
            return k;
        }
    }
    // 2. Legacy `kern` table (older Latin fonts).
    if let Some(kern) = face.tables().kern {
        for sub in kern.subtables {
            if sub.horizontal && !sub.variable {
                if let Some(v) = sub.glyphs_kerning(left, right) {
                    return v as i32;
                }
            }
        }
    }
    0
}

/// Walk GPOS lookups looking for a pair-positioning rule matching
/// `(left, right)`. Returns the x-advance adjustment on the left glyph
/// (i.e. the value to add to the pen position before drawing `right`).
///
/// Matches `opentype.js`'s default behavior: only the kern lookup
/// (script: default, language: default, feature: kern) is used, and
/// only the first matching subtable wins. We approximate this by
/// scanning all pair-pos subtables in lookup order — Roboto's only
/// pair-pos lookups are the kern lookup, so the behavior matches.
fn gpos_kern_value(
    gpos: ttf_parser::opentype_layout::LayoutTable<'_>,
    left: ttf_parser::GlyphId,
    right: ttf_parser::GlyphId,
) -> Option<i32> {
    use ttf_parser::gpos::{PairAdjustment, PositioningSubtable};

    for lookup in gpos.lookups {
        for sub in lookup.subtables.into_iter::<PositioningSubtable>() {
            let PositioningSubtable::Pair(pa) = sub else { continue };
            match pa {
                PairAdjustment::Format1 { coverage, sets } => {
                    let Some(idx) = coverage.get(left) else { continue };
                    let Some(set) = sets.get(idx) else { continue };
                    if let Some((first, _second)) = set.get(right) {
                        // Match opentype.js: only consider the left
                        // glyph's x-advance adjustment.
                        if first.x_advance != 0 {
                            return Some(first.x_advance as i32);
                        }
                    }
                }
                PairAdjustment::Format2 {
                    coverage,
                    classes,
                    matrix,
                } => {
                    if coverage.get(left).is_none() {
                        continue;
                    }
                    let c1 = classes.0.get(left);
                    let c2 = classes.1.get(right);
                    if let Some((first, _second)) = matrix.get((c1, c2)) {
                        if first.x_advance != 0 {
                            return Some(first.x_advance as i32);
                        }
                    }
                }
            }
        }
    }
    None
}

impl std::fmt::Debug for ParsedFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsedFont")
            .field("family_name", &self.family_name)
            .field("units_per_em", &self.units_per_em)
            .finish()
    }
}

#[cfg(test)]
mod path_tests {
    //! Path-emission correctness tests for the `ttf-parser` outline
    //! pipeline. The path strings emitted here describe the SAME
    //! polygon as `@shuding/opentype.js` but with a different
    //! traversal — `ttf-parser` starts each contour at `contour[0]`
    //! and skips redundant zero-length `L`s after a `Q` ending on an
    //! on-curve point, while `opentype.js` starts at `contour[last]`
    //! and emits one command per contour point. Both describe the
    //! same closed shape and rasterize identically; the integration
    //! suite in `crates/satori-tests` covers pixel-level parity end
    //! to end.
    use super::*;

    fn roboto() -> ParsedFont {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("satori-tests")
            .join("assets")
            .join("Roboto-Regular.ttf");
        let bytes = std::fs::read(&path).expect("Roboto-Regular.ttf");
        ParsedFont::parse(bytes).expect("parse Roboto")
    }

    #[test]
    fn h_path_matches_opentype_js() {
        let font = roboto();
        let got = font.glyph_path_d('H', 0.0, 0.0, 16.0, 1).unwrap();
        // Byte-identical to `@shuding/opentype.js`'s
        // `font.charToGlyph('H').getPath(0, 0, 16).toPathData(1)`.
        let want = "M10.1-11.4L10.1 0L8.6 0L8.6-5.3L2.8-5.3L2.8 0L1.3 0L1.3-11.4L2.8-11.4L2.8-6.5L8.6-6.5L8.6-11.4L10.1-11.4Z";
        assert_eq!(got, want);
    }

    #[test]
    fn o_path_matches_opentype_js() {
        let font = roboto();
        let got = font.glyph_path_d('o', 0.0, 0.0, 16.0, 1).unwrap();
        let want = "M0.7-4.2L0.7-4.3Q0.7-5.5 1.2-6.5Q1.7-7.5 2.6-8.1Q3.4-8.6 4.5-8.6L4.5-8.6Q6.3-8.6 7.3-7.4Q8.4-6.2 8.4-4.2L8.4-4.2L8.4-4.1Q8.4-2.9 7.9-1.9Q7.5-0.9 6.6-0.4Q5.7 0.2 4.6 0.2L4.6 0.2Q2.8 0.2 1.8-1.0Q0.7-2.2 0.7-4.2L0.7-4.2ZM2.2-4.1L2.2-4.1Q2.2-2.7 2.8-1.9Q3.5-1.0 4.6-1.0L4.6-1.0Q5.7-1.0 6.3-1.9Q7.0-2.8 7.0-4.3L7.0-4.3Q7.0-5.7 6.3-6.6Q5.6-7.4 4.5-7.4L4.5-7.4Q3.5-7.4 2.8-6.6Q2.2-5.7 2.2-4.1Z";
        assert_eq!(got, want);
    }

    #[test]
    fn e_path_matches_opentype_js() {
        let font = roboto();
        let got = font.glyph_path_d('e', 0.0, 0.0, 16.0, 1).unwrap();
        let want = "M4.6 0.2L4.6 0.2Q2.9 0.2 1.8-1.0Q0.7-2.1 0.7-4.0L0.7-4.0L0.7-4.3Q0.7-5.5 1.2-6.5Q1.7-7.5 2.6-8.1Q3.4-8.6 4.4-8.6L4.4-8.6Q6.1-8.6 7.0-7.5Q7.9-6.4 7.9-4.4L7.9-4.4L7.9-3.8L2.2-3.8Q2.2-2.6 2.9-1.8Q3.6-1.0 4.7-1.0L4.7-1.0Q5.4-1.0 6.0-1.3Q6.5-1.6 6.9-2.2L6.9-2.2L7.8-1.5Q6.7 0.2 4.6 0.2ZM4.4-7.4L4.4-7.4Q3.5-7.4 3.0-6.8Q2.4-6.1 2.2-5L2.2-5L6.5-5L6.5-5.1Q6.4-6.2 5.9-6.8Q5.3-7.4 4.4-7.4Z";
        assert_eq!(got, want);
    }

    #[test]
    fn hello_full_path_matches_opentype_js() {
        let font = roboto();
        let got = font.run_path_d("Hello", 0.0, 15.0, 16.0, 0.0, true, 1);
        let want = "M10.1 3.6L10.1 15L8.6 15L8.6 9.7L2.8 9.7L2.8 15L1.3 15L1.3 3.6L2.8 3.6L2.8 8.5L8.6 8.5L8.6 3.6L10.1 3.6ZM16.0 15.2L16.0 15.2Q14.3 15.2 13.2 14.0Q12.1 12.9 12.1 11.0L12.1 11.0L12.1 10.7Q12.1 9.5 12.6 8.5Q13.1 7.5 14.0 6.9Q14.8 6.4 15.8 6.4L15.8 6.4Q17.5 6.4 18.4 7.5Q19.3 8.6 19.3 10.6L19.3 10.6L19.3 11.2L13.6 11.2Q13.6 12.4 14.3 13.2Q15.0 14.0 16.1 14.0L16.1 14.0Q16.9 14.0 17.4 13.7Q17.9 13.4 18.3 12.8L18.3 12.8L19.2 13.5Q18.1 15.2 16.0 15.2ZM15.8 7.6L15.8 7.6Q15.0 7.6 14.4 8.2Q13.8 8.9 13.6 10L13.6 10L17.9 10L17.9 9.9Q17.8 8.8 17.3 8.2Q16.7 7.6 15.8 7.6ZM22.5 3L22.5 15L21.1 15L21.1 3L22.5 3ZM26.4 3L26.4 15L25.0 15L25.0 3L26.4 3ZM28.4 10.8L28.4 10.7Q28.4 9.5 28.8 8.5Q29.3 7.5 30.2 6.9Q31.1 6.4 32.2 6.4L32.2 6.4Q33.9 6.4 35.0 7.6Q36.1 8.8 36.1 10.8L36.1 10.8L36.1 10.9Q36.1 12.1 35.6 13.1Q35.1 14.1 34.2 14.6Q33.4 15.2 32.2 15.2L32.2 15.2Q30.5 15.2 29.4 14.0Q28.4 12.8 28.4 10.8L28.4 10.8ZM29.8 10.9L29.8 10.9Q29.8 12.3 30.5 13.1Q31.1 14.0 32.2 14.0L32.2 14.0Q33.3 14.0 34.0 13.1Q34.6 12.3 34.6 10.7L34.6 10.7Q34.6 9.3 33.9 8.4Q33.3 7.6 32.2 7.6L32.2 7.6Q31.1 7.6 30.5 8.4Q29.8 9.3 29.8 10.9Z";
        assert_eq!(got, want);
    }

    #[test]
    fn ava_full_path_matches_opentype_js() {
        let font = roboto();
        let got = font.run_path_d("AVA", 0.0, 0.0, 16.0, 0.0, true, 1);
        // Note: GPOS kerning shifts V left from x=12.3 (unkerned) to x=11.6.
        let want = "M8.7 0L7.6-3.0L2.8-3.0L1.8 0L0.2 0L4.6-11.4L5.9-11.4L10.2 0L8.7 0ZM5.2-9.5L3.3-4.2L7.2-4.2L5.2-9.5ZM11.6-11.4L14.8-2.0L18.1-11.4L19.7-11.4L15.5 0L14.2 0L10.0-11.4L11.6-11.4ZM28.0 0L27.0-3.0L22.2-3.0L21.1 0L19.6 0L23.9-11.4L25.2-11.4L29.6 0L28.0 0ZM24.6-9.5L22.6-4.2L26.5-4.2L24.6-9.5Z";
        assert_eq!(got, want);
    }

    #[test]
    fn ava_uses_gpos_kerning() {
        let font = roboto();
        // Roboto's GPOS gives A→V = -87 units, V→A = -87 units.
        // Without kerning V starts ~0.68px later, so the paths must
        // differ.
        let kerned = font.run_path_d("AVA", 0.0, 0.0, 16.0, 0.0, true, 1);
        let unkerned = font.run_path_d("AVA", 0.0, 0.0, 16.0, 0.0, false, 1);
        assert_ne!(kerned, unkerned, "kerning must change AVA's path");
        // Specifically: V's first M command — kerned at x=11.6,
        // unkerned at x=12.3 (matches opentype.js values).
        assert!(kerned.contains("M11.6-11.4"), "expected kerned V-start M11.6-11.4 in {kerned}");
        assert!(unkerned.contains("M12.3-11.4"), "expected unkerned V-start M12.3-11.4 in {unkerned}");
    }
}
