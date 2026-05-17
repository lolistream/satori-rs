//! Port of `src/builder/text.ts` — both the `embedFont: false`
//! `<text>` emitter and the `embedFont: true` `<path>` emitter.
//!
//! For `embedFont: true`, JS satori extracts each glyph's outline via
//! `opentype.js`, accumulates the path data via `Path.toPathData(1)`,
//! and wraps the concatenated string in a single `<path d="…"
//! fill="…"/>` per text segment, then wraps THAT in an opacity/clip
//! `<g>` to match the styling rules. We mirror the same shape here
//! using `satori-font::ParsedFont::run_path_d`.
//!
//! For `embedFont: false`, we emit a single `<text>` element per
//! segment and let the downstream renderer shape the glyphs.
//!
//! Both branches share the same `escape_html` substitution set
//! (`&`, `<`, `>`, `"`, `'`) for byte parity with upstream.

use satori_css::style::{ComputedStyle, FontStyle};

use crate::xml::{build_xml, js_number_to_string, AttrValue};

pub struct TextArgs<'a> {
    pub content: &'a str,
    /// Absolute x of the baseline.
    pub x: f32,
    /// Absolute y of the baseline.
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub matrix: Option<&'a str>,
    pub clip_path_id: Option<&'a str>,
    pub opacity: f32,
}

pub struct TextPathArgs<'a> {
    /// Pre-built `d` attribute value (concatenation of one or more
    /// glyph run paths). Mirrors JS satori's `mergedPath`.
    pub d: &'a str,
    pub matrix: Option<&'a str>,
    pub opacity: f32,
    /// `_inheritedMaskId` from a parent that has `overflow: hidden` or
    /// `mask-image` — applied as `mask="url(#<id>)"` on the `<g>` wrapper.
    pub overflow_mask_id: Option<&'a str>,
    /// `_inheritedClipPathId` from a parent that has `overflow: hidden`
    /// or an explicit `clip-path` — applied as `clip-path="url(#<id>)"`
    /// on the `<g>` wrapper.
    pub clip_path_id: Option<&'a str>,
    /// `True` if the parent text color is fully transparent. JS:
    /// `cssColorParse(color).alpha === 0`. Causes:
    ///   * the inner `<g><path/></g>` to be omitted when there's no
    ///     `filter` (no point rendering invisible glyphs),
    ///   * the path's `fill` to be `"black"` so the shadow filter has
    ///     opaque source alpha to blur (JS path).
    pub transparent_text: bool,
    /// `True` if the parent has any `text-shadow` (we'll emit a
    /// `<defs><filter/></defs>` and wrap the inner `<g>` with
    /// `filter="url(#satori_s-{id})"`). Required because the JS path's
    /// `fill` decision depends on whether a filter is present.
    pub has_text_shadow: bool,
}

/// JS satori's `embedFont: true` branch — emits a `<path d="..." />`
/// for the glyph run, wrapped in a `<g>` element that matches the JS
/// template `<g ${maskAttr} ${clipPathAttr}>...</g>` byte-for-byte.
///
/// Even when neither attribute is present, JS still emits the literal
/// string `<g  >` (the two empty placeholders collapse to two
/// spaces). `resvg`'s rasterizer composites a `<g>`-wrapped path
/// slightly differently from a bare `<path>`, so we replicate this
/// structure exactly to match JS satori's pixel output.
pub fn render_text_path(args: &TextPathArgs<'_>, style: &ComputedStyle) -> String {
    if args.d.is_empty() {
        return String::new();
    }
    // JS satori's gate: omit the entire `<g>` (and its inner `<path>`)
    // when `(!isTransparentText || filter) && opacity !== 0` evaluates
    // false. The `opacity !== 0` check is implicit (we never see an
    // opacity-0 glyph because the layout already skipped it), but we
    // still need to honor the transparent-text path: invisible glyphs
    // only render when a text-shadow filter is present (so the shadow
    // has source to render from).
    if args.transparent_text && !args.has_text_shadow {
        return String::new();
    }
    // `fill: filter && isTransparentText ? 'black' : textFillColor || color`.
    let fill = if args.has_text_shadow && args.transparent_text {
        "black".to_string()
    } else {
        style
            ._webkit_text_fill_color
            .clone()
            .or_else(|| style.color.clone())
            .unwrap_or_else(|| "black".to_string())
    };
    let opacity_attr = if (args.opacity - 1.0).abs() > f32::EPSILON {
        AttrValue::Number(args.opacity)
    } else {
        AttrValue::Skip
    };
    let matrix_attr = match args.matrix {
        Some(m) => AttrValue::Str(m),
        None => AttrValue::Skip,
    };

    // `-webkit-text-stroke` (when set) becomes `stroke`+`stroke-width`+
    // `stroke-linejoin="round"`+`paint-order="stroke"` on the glyph path.
    // Mirrors JS satori `text/index.ts` lines 884-895.
    let has_stroke = style
        ._webkit_text_stroke_width
        .map(|w| w > 0.0)
        .unwrap_or(false);
    let (stroke_attr, stroke_w_attr, stroke_lj_attr, paint_order_attr) = if has_stroke {
        let w = style._webkit_text_stroke_width.unwrap_or(0.0);
        let c = style
            ._webkit_text_stroke_color
            .clone()
            .unwrap_or_else(|| "black".to_string());
        (
            AttrValue::Owned(c),
            AttrValue::Owned(format!("{}px", js_number_to_string(w))),
            AttrValue::Str("round"),
            AttrValue::Str("stroke"),
        )
    } else {
        (
            AttrValue::Skip,
            AttrValue::Skip,
            AttrValue::Skip,
            AttrValue::Skip,
        )
    };

    let path = build_xml(
        "path",
        &[
            ("fill", AttrValue::Owned(fill)),
            ("d", AttrValue::Str(args.d)),
            ("transform", matrix_attr),
            ("opacity", opacity_attr),
            ("stroke-width", stroke_w_attr),
            ("stroke", stroke_attr),
            ("stroke-linejoin", stroke_lj_attr),
            ("paint-order", paint_order_attr),
        ],
        None,
    );
    // Mirror JS's `<g ${maskAttr} ${clipPathAttr}>` template.
    // When neither attr is set we still emit the literal two-space
    // placeholder (`<g  >`) for byte-for-byte parity with JS satori.
    let mask_attr = args
        .overflow_mask_id
        .map(|id| format!("mask=\"url(#{id})\""))
        .unwrap_or_default();
    let clip_attr = args
        .clip_path_id
        .map(|id| format!("clip-path=\"url(#{id})\""))
        .unwrap_or_default();
    format!("<g {mask_attr} {clip_attr}>{path}</g>")
}

pub fn render_text(args: &TextArgs<'_>, style: &ComputedStyle) -> String {
    if args.content.is_empty() {
        return String::new();
    }

    let fill = style
        .color
        .clone()
        .unwrap_or_else(|| "black".to_string());

    let font_weight = style.font_weight.unwrap_or(400);
    let font_style_str = match style.font_style.unwrap_or(FontStyle::Normal) {
        FontStyle::Normal => "normal",
        FontStyle::Italic => "italic",
    };
    let font_size = style.font_size.unwrap_or(16.0);
    let font_family = style
        .font_family
        .clone()
        .unwrap_or_else(|| "serif".to_string());

    let escaped = escape_html(args.content);

    let opacity_attr = if (args.opacity - 1.0).abs() > f32::EPSILON {
        AttrValue::Number(args.opacity)
    } else {
        AttrValue::Skip
    };

    let clip_path_attr = args
        .clip_path_id
        .map(|id| AttrValue::Owned(format!("url(#{id})")))
        .unwrap_or(AttrValue::Skip);

    let matrix_attr = match args.matrix {
        Some(m) => AttrValue::Str(m),
        None => AttrValue::Skip,
    };

    // Inline `style="..."` carries CSS that has no direct SVG-attribute
    // analog (`filter`, `-webkit-text-stroke`, `text-shadow` -> filter).
    let mut style_parts: Vec<String> = Vec::new();
    let mut filter_parts: Vec<String> = Vec::new();
    if let Some(f) = style.filter.as_deref() {
        filter_parts.push(f.to_string());
    }
    if let Some(shadows) = style.text_shadow.as_ref() {
        for sh in shadows {
            filter_parts.push(format!(
                "drop-shadow({}px {}px {}px {})",
                sh.offset_x, sh.offset_y, sh.blur, sh.color
            ));
        }
    }
    if !filter_parts.is_empty() {
        style_parts.push(format!("filter:{}", filter_parts.join(" ")));
    }
    if let (Some(w), Some(c)) = (
        style._webkit_text_stroke_width,
        style._webkit_text_stroke_color.as_deref(),
    ) {
        if w > 0.0 {
            style_parts.push(format!("-webkit-text-stroke:{w}px {c}"));
        }
    }
    let style_attr = if style_parts.is_empty() {
        AttrValue::Skip
    } else {
        AttrValue::Owned(style_parts.join(";"))
    };

    // Mirror the embedFont:true `<path>` path: when `-webkit-text-stroke`
    // is set, emit it as SVG `stroke`/`stroke-width`/`paint-order`
    // attributes on the `<text>` element too.
    let has_stroke = style
        ._webkit_text_stroke_width
        .map(|w| w > 0.0)
        .unwrap_or(false);
    let (stroke_attr, stroke_w_attr, stroke_lj_attr, paint_order_attr) = if has_stroke {
        let w = style._webkit_text_stroke_width.unwrap_or(0.0);
        let c = style
            ._webkit_text_stroke_color
            .clone()
            .unwrap_or_else(|| "black".to_string());
        (
            AttrValue::Owned(c),
            AttrValue::Owned(format!("{}px", js_number_to_string(w))),
            AttrValue::Str("round"),
            AttrValue::Str("stroke"),
        )
    } else {
        (
            AttrValue::Skip,
            AttrValue::Skip,
            AttrValue::Skip,
            AttrValue::Skip,
        )
    };

    build_xml(
        "text",
        &[
            ("x", AttrValue::Number(args.x)),
            ("y", AttrValue::Number(args.y)),
            ("width", AttrValue::Number(args.width)),
            ("height", AttrValue::Number(args.height)),
            ("font-weight", AttrValue::Int(font_weight as i64)),
            ("font-style", AttrValue::Str(font_style_str)),
            ("font-size", AttrValue::Number(font_size)),
            ("font-family", AttrValue::Owned(font_family)),
            ("transform", matrix_attr),
            ("clip-path", clip_path_attr),
            ("fill", AttrValue::Owned(fill)),
            ("stroke-width", stroke_w_attr),
            ("stroke", stroke_attr),
            ("stroke-linejoin", stroke_lj_attr),
            ("paint-order", paint_order_attr),
            ("opacity", opacity_attr),
            ("style", style_attr),
        ],
        Some(&escaped),
    )
}

/// Mirror of the npm `escape-html` package: replaces `&`, `<`, `>`, `"`,
/// `'`. We must match this set exactly because the JS reference output
/// uses it verbatim in `<text>` content.
pub fn escape_html(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}
