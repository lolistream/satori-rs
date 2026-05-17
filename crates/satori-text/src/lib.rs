//! Text layout (port of `src/text/index.ts` + `src/text/processor.ts` +
//! `src/text/measurer.ts`).
//!
//! The JS satori loop is preserved closely: this module returns a
//! per-word position list (line, x, width, content, lineIndex) that the
//! renderer turns into `<text>` or `<path>` elements, plus per-line
//! widths/baselines/segment-counts so the renderer can apply text-align
//! center/right/justify and line-clamp ellipses.

use std::sync::Arc;

use satori_font::{FontLoader, FontStyle, ParsedFont, Weight};
use unicode_segmentation::UnicodeSegmentation;

const SPACE_CHAR: char = ' ';
const TAB_CHAR: char = '\t';
const ELLIPSIS: &str = "\u{2026}";

/// One emittable run (word / grapheme).
#[derive(Debug, Clone)]
pub struct WordPos {
    pub content: String,
    /// X offset relative to the text container's content box (before
    /// applying `text-align`, `text-indent`).
    pub x: f32,
    /// Y offset of the line's top, relative to the container.
    pub y: f32,
    pub width: f32,
    pub line: usize,
    /// Index of this segment within its line (only filled for
    /// `text-align: justify` so the renderer can compute per-segment
    /// `leftOffset += gutter * lineIndex`).
    pub line_index: i32,
    pub is_image: bool,
}

#[derive(Debug, Clone)]
pub struct TextLayout {
    /// Per-word positions (rendering order).
    pub words: Vec<WordPos>,
    /// Per-line measured widths (after the line-ending-space trim that
    /// JS satori applies).
    pub line_widths: Vec<f32>,
    /// `Math.round(engine.baseline(word))` for each line.
    pub baselines: Vec<f32>,
    /// Number of "word" segments on each line — used by `text-align:
    /// justify` to compute the per-gutter spacing.
    pub line_segment_counts: Vec<u32>,
    /// Total measured width (ceil'd, mirroring JS `Math.ceil`).
    pub width: f32,
    /// Total measured height (sum of per-line heights).
    pub height: f32,
    /// Per-line height (used to render decoration positions).
    pub line_height: f32,
    /// Per-line baseline offset (= `Math.round(engine.baseline(line[0]))`).
    pub baseline: f32,
    /// Unrounded engine.baseline value (JS satori uses this as
    /// `baselineOfWord` for text-decoration y math).
    pub baseline_raw: f32,
    pub ascender: f32,
    /// `True` if the layout was forced past `line_limit` (renderer should
    /// truncate with ellipsis).
    pub line_limit: u32,
    /// Block-ellipsis character / string for `line-clamp`.
    pub block_ellipsis: String,
}

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub font_size: f32,
    /// f64 mirror of `font_size` propagated through the em/rem chain.
    /// `0.8 * 16 * 1.5` in f64 is `19.20000000000000284217`; storing
    /// only as f32 collapses to `19.2 → 19.200000762939453` after
    /// widening, which makes `1/2048 * fontSize` differ from JS by
    /// ~1 ULP and shifts glyph coordinates after `toFixed(1)`.
    pub font_size_exact: Option<f64>,
    pub line_height: Option<f32>,
    pub font_family: Vec<String>,
    pub font_weight: Weight,
    pub font_style: FontStyle,
    pub letter_spacing: f32,
    pub white_space: WhiteSpaceMode,
    pub text_align: TextAlignMode,
    pub word_break: WordBreakMode,
    pub text_wrap: TextWrapMode,
    pub text_transform: TextTransformMode,
    /// `tab-size` resolved to either a multiplier of space-width
    /// (`Multiplier(n)`) or an absolute pixel width (`Pixels(px)`).
    pub tab_size: TabSize,
    pub text_indent: f32,
    pub line_limit: u32,
    pub block_ellipsis: String,
    /// Mirror of `options.graphemeImages` plumbed down from
    /// `SatoriOptions`. Image graphemes count as `font_size` wide in
    /// the measurer and become `<image href="...">` (rendered upstream)
    /// instead of glyph paths.
    pub grapheme_images: std::sync::Arc<std::collections::HashMap<String, String>>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            font_size_exact: None,
            line_height: None,
            font_family: vec!["sans-serif".to_string()],
            font_weight: 400,
            font_style: FontStyle::Normal,
            letter_spacing: 0.0,
            white_space: WhiteSpaceMode::Normal,
            text_align: TextAlignMode::Start,
            word_break: WordBreakMode::Normal,
            text_wrap: TextWrapMode::Wrap,
            text_transform: TextTransformMode::None,
            tab_size: TabSize::Multiplier(8.0),
            text_indent: 0.0,
            line_limit: u32::MAX,
            block_ellipsis: ELLIPSIS.to_string(),
            grapheme_images: std::sync::Arc::new(std::collections::HashMap::new()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WhiteSpaceMode {
    #[default]
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextAlignMode {
    #[default]
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WordBreakMode {
    #[default]
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextWrapMode {
    #[default]
    Wrap,
    Nowrap,
    Balance,
    Pretty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextTransformMode {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TabSize {
    Multiplier(f32),
    Pixels(f32),
}

impl Default for TabSize {
    fn default() -> Self {
        TabSize::Multiplier(8.0)
    }
}

/// Mirror of `processTextTransform` in `text/processor.ts`.
fn process_text_transform(content: &str, mode: TextTransformMode) -> String {
    match mode {
        TextTransformMode::None => content.to_string(),
        TextTransformMode::Uppercase => content.to_uppercase(),
        TextTransformMode::Lowercase => content.to_lowercase(),
        TextTransformMode::Capitalize => {
            let mut out = String::with_capacity(content.len());
            for word in segment_words(content) {
                let mut graphemes = word.graphemes(true);
                if let Some(first) = graphemes.next() {
                    out.push_str(&first.to_uppercase());
                }
                for g in graphemes {
                    out.push_str(g);
                }
            }
            out
        }
    }
}

/// Mirror of `processWhiteSpace` in `text/processor.ts`. Returns
/// `(processed_content, shouldCollapseTabsAndSpaces, allowSoftWrap)`.
pub fn process_white_space(text: &str, ws: WhiteSpaceMode) -> (String, bool, bool) {
    let keep_linebreak = matches!(
        ws,
        WhiteSpaceMode::Pre | WhiteSpaceMode::PreWrap | WhiteSpaceMode::PreLine
    );
    let collapse_tabs_spaces = matches!(
        ws,
        WhiteSpaceMode::Normal | WhiteSpaceMode::NoWrap | WhiteSpaceMode::PreLine
    );
    let allow_soft_wrap = !matches!(ws, WhiteSpaceMode::Pre | WhiteSpaceMode::NoWrap);

    let mut s: String = if keep_linebreak {
        text.to_string()
    } else {
        text.replace('\n', " ")
    };

    if collapse_tabs_spaces {
        let mut out = String::with_capacity(s.len());
        let mut prev_space = false;
        for c in s.chars() {
            if c == SPACE_CHAR || c == TAB_CHAR {
                if !prev_space {
                    out.push(SPACE_CHAR);
                    prev_space = true;
                }
            } else {
                out.push(c);
                prev_space = false;
            }
        }
        if out.starts_with(SPACE_CHAR) {
            out.remove(0);
        }
        if out.ends_with(SPACE_CHAR) {
            out.pop();
        }
        s = out;
    }
    (s, collapse_tabs_spaces, allow_soft_wrap)
}

/// Mirror of `splitByBreakOpportunities` in `src/utils.ts`. Returns
/// `(words, requiredBreaks)`. For each word in `words`, the boolean at
/// the same index in `requiredBreaks` is `true` IFF a break before that
/// word is mandatory (e.g. after `\n`).
pub fn split_by_break_opportunities(content: &str, word_break: WordBreakMode) -> (Vec<String>, Vec<bool>) {
    if word_break == WordBreakMode::BreakAll {
        let words: Vec<String> = content.graphemes(true).map(String::from).collect();
        return (words, vec![false; 0]);
    }
    if word_break == WordBreakMode::KeepAll {
        // `Intl.Segmenter('word')` in JS groups consecutive Han /
        // Hiragana / Katakana into single word units, but
        // `unicode-segmentation`'s `split_word_bounds` (UAX#29)
        // breaks each CJK char as its own word. Post-merge so
        // `你好` stays one word, matching JS `keep-all`.
        let raw: Vec<String> = segment_words(content);
        let mut merged: Vec<String> = Vec::with_capacity(raw.len());
        for w in raw {
            let is_cjk = !w.is_empty() && w.chars().all(is_cjk_char);
            if is_cjk {
                if let Some(last) = merged.last_mut() {
                    if !last.is_empty() && last.chars().all(is_cjk_char) {
                        last.push_str(&w);
                        continue;
                    }
                }
            }
            merged.push(w);
        }
        return (merged, vec![false; 0]);
    }

    use unicode_linebreak::{linebreaks, BreakOpportunity};
    let mut words: Vec<String> = Vec::new();
    // Mirror JS: `requiredBreaks` starts with `[false]` (one extra entry
    // for the implicit "before word 0").
    let mut required_breaks: Vec<bool> = vec![false];
    let mut last = 0usize;
    for (idx, op) in linebreaks(content) {
        let slice = &content[last..idx];
        if !slice.is_empty() {
            words.push(slice.to_string());
            required_breaks.push(matches!(op, BreakOpportunity::Mandatory));
        }
        last = idx;
    }
    // JS keeps an extra "trailing" entry, but we only need indices up to
    // `words.len() - 1`. Trim/pad to match.
    if required_breaks.len() > words.len() {
        required_breaks.truncate(words.len());
    }
    while required_breaks.len() < words.len() {
        required_breaks.push(false);
    }
    (words, required_breaks)
}

/// Mirror of `wordSeparators` in `src/utils.ts`.
pub fn is_word_separator(s: &str) -> bool {
    matches!(
        s,
        " " | "\u{00a0}"
            | "\u{1361}"
            | "\u{10100}"
            | "\u{10101}"
            | "\u{1039}"
            | "\u{1091}"
            | "\n"
    )
}

/// Segment by JS-`Intl.Segmenter`-style word granularity.
///
/// Also applies JS satori's `segment(content, 'word')` post-pass:
/// when an `Intl.Segmenter` boundary lands on a non-breaking space
/// (U+00A0), the adjacent words on either side are JOINED back into
/// a single segment with the NBSP in between. Without this, our
/// renderer breaks `"50\u{00a0}kg"` into three separate emit calls
/// instead of the one JS satori does.
pub fn segment_words(text: &str) -> Vec<String> {
    let raw: Vec<&str> = text.split_word_bounds().collect();
    let mut out: Vec<String> = Vec::with_capacity(raw.len());
    let mut i = 0;
    while i < raw.len() {
        let s = raw[i];
        if s == "\u{00a0}" {
            let prev = if out.is_empty() { String::new() } else { out.pop().unwrap() };
            let next = if i + 1 < raw.len() { raw[i + 1].to_string() } else { String::new() };
            let mut merged = prev;
            merged.push('\u{00a0}');
            merged.push_str(&next);
            out.push(merged);
            i += 2;
        } else {
            out.push(s.to_string());
            i += 1;
        }
    }
    out
}

/// `true` when every char of `s` is in the Han / Hiragana / Katakana
/// blocks that `Intl.Segmenter('word')` keeps as a single word for
/// CSS `word-break: keep-all` purposes. Hangul is excluded — it
/// already comes through `split_word_bounds` as a single segment.
fn is_cjk_char(ch: char) -> bool {
    matches!(ch as u32,
        0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0xF900..=0xFAFF |
        0x3040..=0x309F | 0x30A0..=0x30FF | 0x31F0..=0x31FF
    )
}

/// Per-grapheme advance summation. Mirrors JS satori's `measureText`
/// (sum of `measureGrapheme` over `Intl.Segmenter('grapheme')`).
pub fn measure_text(
    s: &str,
    fonts: &[Arc<ParsedFont>],
    font_size: f32,
    letter_spacing: f32,
) -> f32 {
    if fonts.is_empty() || s.is_empty() {
        return 0.0;
    }
    let mut total = 0.0f32;
    for grapheme in s.graphemes(true) {
        total += measure_grapheme(grapheme, fonts, font_size, letter_spacing);
    }
    total
}

/// Image-aware variant of `measure_text`. Mirrors
/// `measureGraphemeArray` in `text/measurer.ts`: graphemes that are
/// keys in `images` count as `font_size` wide (the rendered `<image>`
/// is sized to the line font-size).
pub fn measure_text_with_images(
    s: &str,
    fonts: &[Arc<ParsedFont>],
    font_size: f32,
    letter_spacing: f32,
    images: &std::collections::HashMap<String, String>,
) -> f32 {
    if fonts.is_empty() || s.is_empty() {
        return 0.0;
    }
    if images.is_empty() {
        return measure_text(s, fonts, font_size, letter_spacing);
    }
    let mut total = 0.0f32;
    for grapheme in s.graphemes(true) {
        if images.contains_key(grapheme) {
            total += font_size;
        } else {
            total += measure_grapheme(grapheme, fonts, font_size, letter_spacing);
        }
    }
    total
}

/// Mirror of JS satori's `measureGrapheme`, which calls
/// `font.getAdvanceWidth(grapheme, fontSize, { letterSpacing })`.
/// `opentype.js`'s `forEachGlyph` adds `letterSpacing * fontSize` to
/// the pen after EVERY glyph (including the last one in the run) AND
/// applies kerning between adjacent glyphs.
///
/// For multi-character input (e.g. a per-segment-word measure), this
/// matters: `measureGrapheme("transformed")` in JS returns the kerned
/// advance, not the sum of per-character advances. Without kerning,
/// the rendered glyph path's bounding box drifts from JS satori's
/// debug rect.
///
/// For single-character input, kerning has no effect, so
/// `measureGrapheme("H")` is identical regardless.
pub fn measure_grapheme(
    g: &str,
    fonts: &[Arc<ParsedFont>],
    font_size: f32,
    letter_spacing: f32,
) -> f32 {
    if fonts.is_empty() || g.is_empty() {
        return 0.0;
    }
    // Resolve the primary font for the first char and let
    // `measure_advance` handle GPOS/legacy kerning + letter-spacing
    // application across the run. (JS satori always uses the resolved
    // font for the WHOLE grapheme; we approximate by using the font
    // resolved for the first char.)
    // JS satori always measures via `font.getAdvanceWidth(content,...)`
    // on the PRIMARY resolved font (`fonts[0]` for unbiased
    // mixed-script segments). Inside opentype.js, unknown chars fall
    // back to the primary's `.notdef` advance — NOT the fallback
    // font's advance. Use `fonts[0]` so per-grapheme widths match
    // JS for mixed Latin/CJK segments.
    let first_ch = g.chars().next().unwrap();
    let font = FontLoader::resolve_for_char(fonts, first_ch)
        .unwrap_or_else(|| Arc::clone(&fonts[0]));
    font.measure_advance(g, font_size as f64, letter_spacing as f64) as f32
}

/// Compute the line height & baseline for a font + style. Mirrors JS
/// satori's `FontEngine.height` / `baseline`.
pub fn line_metrics(font: &ParsedFont, font_size: f32, line_height: Option<f32>) -> (f32, f32) {
    // f64 internally so the result mirrors JS satori bit-for-bit (JS
    // does this math in numbers/f64). An f32 chain accumulates enough
    // error that the rounded baseline lands a sub-pixel below where JS
    // puts it; that propagates into glyph coordinates like `32` vs
    // `32.0000004`, which then disagree in `js_float_to_string`'s
    // integer check (`Math.round(v) === v`).
    let upm = font.units_per_em().max(1) as f64;
    let fs = font_size as f64;
    let asc_px = font.ascender_units() as f64 / upm * fs;
    let desc_px = font.descender_units() as f64 / upm * fs;
    let lg_px = font.line_gap_units() as f64 / upm * fs;
    let content_h = asc_px - desc_px;
    let total_h = match line_height {
        Some(m) => fs * m as f64,
        None => content_h + lg_px,
    };
    let baseline = asc_px + (total_h - content_h) / 2.0;
    (total_h as f32, baseline as f32)
}

/// f64 mirror of `line_metrics` for callers that need full mantissa
/// precision on the returned `(line_height, baseline)`. JS satori
/// does decoration-y math in f64; rounding to f32 here would flip
/// `<line>` y attributes by ~1 ULP after `js_float_to_string`.
pub fn line_metrics_f64(font: &ParsedFont, font_size: f32, line_height: Option<f32>) -> (f64, f64) {
    let upm = font.units_per_em().max(1) as f64;
    let fs = font_size as f64;
    let asc_px = font.ascender_units() as f64 / upm * fs;
    let desc_px = font.descender_units() as f64 / upm * fs;
    let lg_px = font.line_gap_units() as f64 / upm * fs;
    let content_h = asc_px - desc_px;
    let total_h = match line_height {
        Some(m) => fs * m as f64,
        None => content_h + lg_px,
    };
    let baseline = asc_px + (total_h - content_h) / 2.0;
    (total_h, baseline)
}

/// Detect a run of tab characters in `text`. Returns `(start_index, tab_count)`.
fn detect_tabs(text: &str) -> (Option<usize>, usize) {
    let Some(start) = text.find('\t') else {
        return (None, 0);
    };
    let mut count = 0usize;
    for ch in text[start..].chars() {
        if ch == '\t' {
            count += 1;
        } else {
            break;
        }
    }
    (Some(start), count)
}

/// Mirror of `calc(text, currentWidth)` in `src/text/index.ts`.
fn calc_text_width(
    text: &str,
    current_width: f32,
    tab_width: f32,
    fonts: &[Arc<ParsedFont>],
    font_size: f32,
    letter_spacing: f32,
    images: &std::collections::HashMap<String, String>,
) -> (f32, f32) {
    if text.is_empty() {
        return (0.0, 0.0);
    }
    let (idx, tab_count) = detect_tabs(text);
    let origin_width = if let (Some(idx), tc) = (idx, tab_count) {
        if tc > 0 {
            let before = &text[..idx];
            let after = &text[idx + tc..];
            let before_w = measure_text_with_images(before, fonts, font_size, letter_spacing, images);
            let offset_before = before_w + current_width;
            let tab_move = if tab_width == 0.0 {
                before_w
            } else {
                ((offset_before / tab_width).floor() + tc as f32) * tab_width - current_width
            };
            tab_move + measure_text_with_images(after, fonts, font_size, letter_spacing, images)
        } else {
            measure_text_with_images(text, fonts, font_size, letter_spacing, images)
        }
    } else {
        measure_text_with_images(text, fonts, font_size, letter_spacing, images)
    };

    let trim_end = text.trim_end_matches([' ', '\t']);
    let trim_w = if trim_end.len() == text.len() {
        origin_width
    } else if trim_end.is_empty() {
        0.0
    } else {
        let (idx2, tab2) = detect_tabs(trim_end);
        if let (Some(idx2), tc2) = (idx2, tab2) {
            if tc2 > 0 {
                let before = &trim_end[..idx2];
                let after = &trim_end[idx2 + tc2..];
                let before_w = measure_text_with_images(before, fonts, font_size, letter_spacing, images);
                let offset_before = before_w + current_width;
                let tab_move = if tab_width == 0.0 {
                    before_w
                } else {
                    ((offset_before / tab_width).floor() + tc2 as f32) * tab_width - current_width
                };
                tab_move + measure_text_with_images(after, fonts, font_size, letter_spacing, images)
            } else {
                measure_text_with_images(trim_end, fonts, font_size, letter_spacing, images)
            }
        } else {
            measure_text_with_images(trim_end, fonts, font_size, letter_spacing, images)
        }
    };
    (origin_width, origin_width - trim_w)
}

/// Lay out `text` into per-word positions. Mirrors `flow(width)` in
/// `src/text/index.ts`. Also implements `text-wrap: balance` via a
/// binary search over the available width (JS satori
/// `text/index.ts` lines 391-408).
pub fn layout_text(
    text: &str,
    style: &TextStyle,
    fonts: &[Arc<ParsedFont>],
    max_width: Option<f32>,
) -> TextLayout {
    let layout = layout_text_inner(text, style, fonts, max_width);
    let Some(width) = max_width else { return layout };
    match style.text_wrap {
        TextWrapMode::Balance => {
            let initial_height = layout.height;
            let initial_width = layout.width;
            if initial_width <= 0.0 {
                return layout;
            }
            let mut l = initial_width / 2.0;
            let mut r = initial_width;
            while l + 1.0 < r {
                let m = (l + r) / 2.0;
                let candidate = layout_text_inner(text, style, fonts, Some(m));
                if candidate.height > initial_height {
                    l = m;
                } else {
                    r = m;
                }
            }
            let mut best = layout_text_inner(text, style, fonts, Some(r));
            best.width = r.ceil();
            best
        }
        TextWrapMode::Pretty => {
            // Mirror JS satori `text/index.ts:415` — when the last
            // line is shorter than 1/3 of the container, reflow at
            // 90% width and accept the result if it doesn't add too
            // many lines (height ≤ 1.3×).
            let last_line_w = layout.line_widths.last().copied().unwrap_or(0.0);
            let is_short = last_line_w < width / 3.0;
            if !is_short {
                return layout;
            }
            let adjusted = width * 0.9;
            let candidate = layout_text_inner(text, style, fonts, Some(adjusted));
            if candidate.height <= layout.height * 1.3 {
                // JS keeps the original `width` (not the adjusted one)
                // as the reported measurement.
                let mut out = candidate;
                out.width = width.ceil();
                out
            } else {
                layout
            }
        }
        _ => layout,
    }
}

fn layout_text_inner(
    text: &str,
    style: &TextStyle,
    fonts: &[Arc<ParsedFont>],
    max_width: Option<f32>,
) -> TextLayout {
    let TextStyle {
        font_size,
        line_height,
        letter_spacing,
        white_space,
        text_align,
        word_break,
        tab_size,
        text_transform,
        line_limit,
        block_ellipsis,
        grapheme_images,
        ..
    } = style.clone();
    let images: &std::collections::HashMap<String, String> = &grapheme_images;

    let transformed = process_text_transform(text, text_transform);
    let (processed, should_collapse_tabs_and_spaces, allow_soft_wrap) =
        process_white_space(&transformed, white_space);
    let allow_break_word = matches!(word_break, WordBreakMode::BreakAll | WordBreakMode::BreakWord);

    if fonts.is_empty() {
        let lh = line_height.unwrap_or(1.2) * font_size;
        return TextLayout {
            words: vec![],
            line_widths: vec![],
            baselines: vec![],
            line_segment_counts: vec![],
            width: 0.0,
            height: 0.0,
            line_height: lh,
            baseline: lh * 0.8,
            baseline_raw: lh * 0.8,
            ascender: font_size * 0.8,
            line_limit,
            block_ellipsis,
        };
    }

    let primary = Arc::clone(&fonts[0]);
    let (engine_line_h, engine_baseline) = line_metrics(&primary, font_size, line_height);
    let upm = primary.units_per_em().max(1) as f32;
    let ascender = primary.ascender_units() as f32 / upm * font_size;

    let space_w = measure_grapheme(" ", fonts, font_size, letter_spacing);
    let tab_w = match tab_size {
        TabSize::Multiplier(n) => space_w * n,
        TabSize::Pixels(px) => px,
    };

    if processed.is_empty() {
        return TextLayout {
            words: vec![],
            line_widths: vec![],
            baselines: vec![],
            line_segment_counts: vec![],
            width: 0.0,
            height: 0.0,
            line_height: engine_line_h,
            baseline: engine_baseline.round(),
            baseline_raw: engine_baseline,
            ascender,
            line_limit,
            block_ellipsis,
        };
    }

    let (mut words, mut required_breaks) =
        split_by_break_opportunities(&processed, word_break);
    while required_breaks.len() < words.len() {
        required_breaks.push(false);
    }

    // === Flow loop (mirrors JS satori `flow(width)`) ===
    let mut texts: Vec<WordPos> = Vec::new();
    let mut line_widths: Vec<f32> = Vec::new();
    let mut baselines: Vec<f32> = Vec::new();
    let mut line_segment_number: Vec<u32> = vec![0];
    let mut lines = 0u32;
    let mut max_w_seen: f32 = 0.0;
    let mut height: f32 = 0.0;
    let mut current_width: f32 = 0.0;
    let mut current_line_height: f32 = 0.0;
    let mut current_baseline_offset: f32 = 0.0;
    let mut prev_line_ending_spaces_w: f32 = 0.0;
    let mut line_index: i32 = -1;

    let allowed_to_justify = matches!(text_align, TextAlignMode::Justify);

    let mut i = 0usize;
    while i < words.len() && lines < line_limit {
        let word = words[i].clone();
        let force_break = required_breaks.get(i).copied().unwrap_or(false);

        let (origin_w, ending_spaces_w) = calc_text_width(
            &word,
            current_width,
            tab_w,
            fonts,
            font_size,
            letter_spacing,
            images,
        );
        let mut w = origin_w;
        let line_ending_spaces_w = ending_spaces_w;

        if force_break && current_line_height == 0.0 {
            current_line_height = engine_line_h;
        }

        let will_wrap = match max_width {
            Some(mw) => {
                i > 0
                    && current_width + w > mw + line_ending_spaces_w
                    && allow_soft_wrap
            }
            None => false,
        };

        let max_w_constraint = max_width.unwrap_or(0.0);

        let need_to_break_word = allow_break_word
            && max_width.is_some()
            && w > max_w_constraint
            && (current_width == 0.0 || will_wrap || force_break);

        if need_to_break_word {
            let chars: Vec<String> = word.graphemes(true).map(String::from).collect();
            if !chars.is_empty() {
                words.splice(i..i + 1, chars.iter().cloned());
                let extra = chars.len().saturating_sub(1);
                if extra > 0 {
                    required_breaks.splice(i + 1..i + 1, std::iter::repeat_n(false, extra));
                }
            }
            if current_width > 0.0 {
                line_widths.push(current_width - prev_line_ending_spaces_w);
                baselines.push(current_baseline_offset);
                lines += 1;
                height += current_line_height;
                current_width = 0.0;
                current_line_height = 0.0;
                current_baseline_offset = 0.0;
                line_segment_number.push(1);
                line_index = -1;
            }
            prev_line_ending_spaces_w = line_ending_spaces_w;
            continue;
        }

        // Per-word line metrics: JS satori uses `engine.height(word)`
        // / `engine.baseline(word)` which resolve the font based on
        // the word's first char. CJK / Korean / Japanese fallback
        // fonts typically have a taller ascender than Roboto, so a
        // line that mixes Latin + CJK should baseline-shift to the
        // taller segment. Compute per-word instead of always using
        // the primary font.
        let (word_line_h, word_baseline) = {
            let first_ch = word.chars().next().unwrap_or(' ');
            let font = FontLoader::resolve_for_char(fonts, first_ch)
                .unwrap_or_else(|| Arc::clone(&primary));
            line_metrics(&font, font_size, line_height)
        };

        if force_break || will_wrap {
            if should_collapse_tabs_and_spaces && word == " " {
                w = 0.0;
            }
            line_widths.push(current_width - prev_line_ending_spaces_w);
            baselines.push(current_baseline_offset);
            lines += 1;
            height += current_line_height;
            current_width = w;
            current_line_height = if w > 0.0 { word_line_h.round() } else { 0.0 };
            current_baseline_offset = if w > 0.0 { word_baseline.round() } else { 0.0 };
            line_segment_number.push(1);
            line_index = -1;
            if !force_break {
                max_w_seen = max_w_seen.max(max_w_constraint);
            }
        } else {
            current_width += w;
            let glyph_h = word_line_h.round();
            if glyph_h > current_line_height {
                current_line_height = glyph_h;
                current_baseline_offset = word_baseline.round();
            }
            if allowed_to_justify {
                if let Some(last) = line_segment_number.last_mut() {
                    *last += 1;
                }
            }
        }

        if allowed_to_justify {
            line_index += 1;
        }

        max_w_seen = max_w_seen.max(current_width);

        let x_start = current_width - w;

        if w == 0.0 {
            texts.push(WordPos {
                content: word.clone(),
                x: x_start,
                y: height,
                width: 0.0,
                line: lines as usize,
                line_index,
                is_image: false,
            });
        } else {
            let mut x_acc = x_start;
            let sub_segments: Vec<String> = segment_words(&word);
            for sub in &sub_segments {
                let sub: &str = sub.as_str();
                if sub.is_empty() {
                    continue;
                }
                let is_image = images.contains_key(sub);
                let sub_w = if is_image {
                    font_size
                } else if sub.contains('\t') {
                    let local_current = x_acc;
                    let (sw, _) = calc_text_width(
                        sub,
                        local_current,
                        tab_w,
                        fonts,
                        font_size,
                        letter_spacing,
                        images,
                    );
                    sw
                } else {
                    // JS: per-segment width is `measureGrapheme(_text)`
                    // (a kerned `getAdvanceWidth` call for the whole
                    // segment) when `embedFont` is true. A per-grapheme
                    // sum drops inter-glyph kerning and drifts the
                    // rendered glyph path from the recorded
                    // `WordPos.width`.
                    measure_grapheme(sub, fonts, font_size, letter_spacing)
                };
                texts.push(WordPos {
                    content: sub.to_string(),
                    x: x_acc,
                    y: height,
                    width: sub_w,
                    line: lines as usize,
                    line_index,
                    is_image,
                });
                x_acc += sub_w;
            }
        }

        i += 1;
        prev_line_ending_spaces_w = line_ending_spaces_w;
    }

    if current_width > 0.0 {
        if lines < line_limit {
            height += current_line_height;
        }
        line_widths.push(current_width);
        baselines.push(current_baseline_offset);
    }

    let total_w = max_w_seen.ceil();
    TextLayout {
        words: texts,
        line_widths,
        baselines,
        line_segment_counts: line_segment_number,
        width: total_w,
        height,
        line_height: engine_line_h,
        baseline: engine_baseline.round(),
        baseline_raw: engine_baseline,
        ascender,
        line_limit,
        block_ellipsis,
    }
}
