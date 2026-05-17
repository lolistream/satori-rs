//! Font loading & resolution (port of `src/font.ts`).
//!
//! Pragmatic slice: parses font binaries with `ttf-parser` to expose the
//! metrics & per-codepoint advance widths needed for the `<text>` emitter,
//! plus a `FontLoader` that mirrors the JS family / weight / style match
//! used during text shaping. Glyph-path extraction is intentionally NOT
//! implemented here — the renderer emits SVG `<text>` elements and lets
//! the downstream rasterizer shape them.

use serde::{Deserialize, Serialize};
use std::sync::Arc;

mod parsed;
mod path_d;
mod ttf_outline;
pub use parsed::{GlyphBox, ParsedFont, SkipInkBand};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontDescriptor {
    pub name: String,
    #[serde(default)]
    pub weight: Option<u16>,
    #[serde(default)]
    pub style: Option<FontStyle>,
    #[serde(default)]
    pub lang: Option<String>,
    #[serde(skip)]
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

/// CSS `font-weight` resolved to a numeric value.
pub type Weight = u16;

/// Loaded font collection. Mirrors the JS `FontLoader` map of
/// `name -> [(font, weight?, style?)]` entries.
#[derive(Clone)]
pub struct FontLoader {
    pub entries: Vec<FontEntry>,
}

#[derive(Clone)]
pub struct FontEntry {
    pub name_lc: String,
    pub weight: Option<Weight>,
    pub style: Option<FontStyle>,
    pub lang: Option<String>,
    pub font: Arc<ParsedFont>,
}

impl FontLoader {
    pub fn new(descriptors: &[FontDescriptor]) -> Self {
        let mut entries = Vec::with_capacity(descriptors.len());
        for d in descriptors {
            if let Some(parsed) = ParsedFont::parse(d.data.clone()) {
                entries.push(FontEntry {
                    name_lc: d.name.to_lowercase(),
                    weight: d.weight,
                    style: d.style,
                    lang: d.lang.clone(),
                    font: Arc::new(parsed),
                });
            }
        }
        Self { entries }
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// JS `FontLoader.get`: best-effort match by family name + weight/style.
    /// Falls back to the first entry of that name. Returns `None` if no font
    /// of that family exists.
    fn get(
        &self,
        name_lc: &str,
        weight: Weight,
        style: FontStyle,
    ) -> Option<&FontEntry> {
        let candidates: Vec<&FontEntry> =
            self.entries.iter().filter(|e| e.name_lc == name_lc).collect();
        if candidates.is_empty() {
            return None;
        }
        let mut best = candidates[0];
        for c in candidates.iter().skip(1) {
            if compare_font(weight, style, best, c) > 0 {
                best = *c;
            }
        }
        Some(best)
    }

    /// Build an ordered "resolution list" of font candidates for a given
    /// (family-list, weight, style). Mirrors `FontLoader.getEngine`'s
    /// `_fonts = [...fonts, ...additionalFonts, ...specifiedLangFonts]`.
    ///
    /// The first family that matches is preferred; the rest of the loaded
    /// fonts act as fallbacks (used when the primary font is missing a
    /// glyph for some codepoint).
    pub fn resolve_list(
        &self,
        family_list: &[String],
        weight: Weight,
        style: FontStyle,
    ) -> Vec<Arc<ParsedFont>> {
        let mut out: Vec<Arc<ParsedFont>> = Vec::new();
        let mut seen_names: Vec<String> = Vec::new();
        for face in family_list {
            let lc = face.to_lowercase();
            if let Some(e) = self.get(&lc, weight, style) {
                out.push(Arc::clone(&e.font));
                seen_names.push(lc);
            }
        }
        // Additional fonts as fallback (everything not already in
        // family_list, preferring name-distinct entries).
        let mut added: Vec<String> = Vec::new();
        for e in &self.entries {
            if seen_names.contains(&e.name_lc) || added.contains(&e.name_lc) {
                continue;
            }
            if let Some(best) = self.get(&e.name_lc, weight, style) {
                out.push(Arc::clone(&best.font));
                added.push(e.name_lc.clone());
            }
        }
        out
    }

    /// Pick the first font that contains a glyph for `ch`. Falls back to
    /// the last font in the list if none match (mirroring JS
    /// `resolveFont(word, fallback=true)`).
    pub fn resolve_for_char(
        list: &[Arc<ParsedFont>],
        ch: char,
    ) -> Option<Arc<ParsedFont>> {
        for f in list {
            if f.has_char(ch) {
                return Some(Arc::clone(f));
            }
        }
        list.last().cloned()
    }
}

/// Port of `compareFont` in `src/font.ts`. Returns >0 if `b` is preferred
/// over `a`, <0 if `a` is preferred. Encoded as a difference for parity.
fn compare_font(weight: Weight, style: FontStyle, a: &FontEntry, b: &FontEntry) -> i32 {
    if a.weight != b.weight {
        if a.weight.is_none() {
            return 1;
        }
        if b.weight.is_none() {
            return -1;
        }
        let aw = a.weight.unwrap() as i32;
        let bw = b.weight.unwrap() as i32;
        let w = weight as i32;
        if aw == w {
            return -1;
        }
        if bw == w {
            return 1;
        }
        if w == 400 && aw == 500 {
            return -1;
        }
        if w == 500 && aw == 400 {
            return -1;
        }
        if w == 400 && bw == 500 {
            return 1;
        }
        if w == 500 && bw == 400 {
            return 1;
        }
        if w < 400 {
            if aw < w && bw < w {
                return bw - aw;
            }
            if aw < w {
                return -1;
            }
            if bw < w {
                return 1;
            }
            return aw - bw;
        }
        if w < aw && w < bw {
            return aw - bw;
        }
        if w < aw {
            return -1;
        }
        if w < bw {
            return 1;
        }
        return bw - aw;
    }
    if a.style != b.style {
        if a.style == Some(style) {
            return -1;
        }
        if b.style == Some(style) {
            return 1;
        }
    }
    -1
}
