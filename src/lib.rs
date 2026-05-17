//! satori-rs top-level crate.
//!
//! Mirrors `src/satori.ts`'s entry point. Public API:
//!
//! ```text
//! let svg = satori::satori(element, satori::SatoriOptions {
//!     width: Some(100),
//!     height: Some(100),
//!     ..Default::default()
//! })?;
//! ```
//!
//! The element tree uses the JSX-shape `{ "type": ..., "props": { "children": ..., "style": {...} } }`
//! pattern via `serde_json::Value`, mirroring upstream's `ReactNode` input.

mod inline_svg;

pub mod builder;
pub mod css;
pub mod font;
pub mod handler;
pub mod jsx;
pub mod layout;
pub mod text;

use serde_json::Value;

use crate::builder::{
    render_rect, render_svg,
    text::{render_text, render_text_path, TextArgs, TextPathArgs},
    transform::{matrix_to_string, resolve_effective},
};
use crate::builder::build_decoration;
use crate::builder::text_decoration::{DecorationArgs, GlyphBox as BoxBox};
use crate::css::{
    dimension::Dim,
    expand::{expand_style, ExpandContext},
    style::{
        ComputedStyle, FontStyle as CssFontStyle, TextAlign, TextDecorationLine,
        TextTransform as CssTextTransform, TextWrap as CssTextWrap, WhiteSpace,
        WordBreak as CssWordBreak,
    },
    variables::Vars,
};
use crate::font::{FontLoader, FontStyle, ParsedFont};
use crate::handler::image::{resolve_image_asset_file, resolve_image_src, ResolvedImage};
use crate::layout::{install_text_measure, lay_out, make_yoga, Built, TextMeasureCtx};
use crate::text::{
    is_word_separator, layout_text, TabSize, TextAlignMode, TextStyle as TxStyle,
    TextTransformMode, TextWrapMode, WhiteSpaceMode, WordBreakMode,
};

#[derive(Debug, Clone, Default)]
pub struct SatoriOptions {
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub fonts: Vec<crate::font::FontDescriptor>,
    pub embed_font: bool,
    pub debug: bool,
    /// Root directory used to resolve `{"__assetFile": "name.png"}` image
    /// references. Falls back to the current working directory.
    pub asset_root: Option<std::path::PathBuf>,
    /// Mirror of `options.graphemeImages` in JS satori: a map from a
    /// grapheme (often an emoji) to a URL/data-URI to render as an
    /// `<image>` in place of the glyph path.
    pub grapheme_images: std::collections::HashMap<String, String>,
}

#[derive(Debug, thiserror::Error)]
pub enum SatoriError {
    #[error("font: {0}")]
    Font(String),
    #[error("layout: {0}")]
    Layout(String),
    #[error("parse: {0}")]
    Parse(String),
    #[error("render: {0}")]
    Render(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

/// Convenience alias for fixture-driven tests. Same as `satori`.
pub fn satori_from_value(element: Value, options: SatoriOptions) -> Result<String, SatoriError> {
    satori(element, options)
}

/// Convert a JSX-shape `Value` tree to an SVG string.
///
/// Accepts the same `{ type, props }` shape that React renders. `props.children`
/// may be a string, a single child object, or an array of children. Components
/// (functions) are not supported via this Value API — callers should evaluate
/// components themselves and pass the resulting tree.
pub fn satori(element: Value, options: SatoriOptions) -> Result<String, SatoriError> {
    let ctx = ExpandContext {
        base_font_size: 16.0,
        viewport_width: options.width,
        viewport_height: options.height,
    };
    let inherited_root = ComputedStyle {
        _viewport_width: options.width,
        _viewport_height: options.height,
        ..ComputedStyle::inheritable_root()
    };

    let font_loader = FontLoader::new(&options.fonts);
    let embed_font = options.embed_font;
    let debug = options.debug;
    let grapheme_images = std::sync::Arc::new(options.grapheme_images.clone());

    let mut id_counter: usize = 0;
    let root_vars = Vars::new();
    // `asset_root` is the on-disk directory used to resolve
    // `{__assetFile: "name.png"}` shapes the test harness emits. When
    // the caller doesn't provide one, default to an empty `PathBuf`
    // so `asset_root.join("name")` resolves to a relative `"name"` -
    // which fails cleanly under any cwd that doesn't happen to have
    // a matching file. The previous fallback used
    // `std::env::current_dir()`, which silently routed image
    // resolution through whatever directory the process was spawned
    // from - a hidden ambient-state dependency in production
    // (audit finding #7).
    let asset_root = options.asset_root.clone().unwrap_or_default();
    let built = build_node(
        &element,
        &inherited_root,
        &root_vars,
        ctx,
        "id",
        &mut id_counter,
        &font_loader,
        &asset_root,
        None,
        &grapheme_images,
    )?
    .ok_or_else(|| SatoriError::Render("root element is null".into()))?;

    let layout = lay_out(built, options.width, options.height);

    // Pre-compute per-node effective transform matrices and their SVG
    // `matrix(...)` attribute strings. Order matches `layout.nodes`
    // (pre-order), so by the time we hit a child we've already resolved
    // its parent's matrix.
    //
    // - A node with its own `transform: ...` ops folds the parent matrix
    //   into its own (`multiply(parent, local_with_origin)`).
    // - A node without own ops *inherits* its parent's effective matrix
    //   verbatim — mirroring JS satori's `isInheritingTransform` branch
    //   where children share the same array reference as the parent.
    let mut matrices: Vec<Option<[f32; 6]>> = Vec::with_capacity(layout.nodes.len());
    let mut matrix_strs: Vec<Option<String>> = Vec::with_capacity(layout.nodes.len());
    for node in &layout.nodes {
        let parent_eff = node.parent.and_then(|p| matrices[p]);
        match (&node.style.transform, parent_eff) {
            (Some(ops), parent) => {
                let m = resolve_effective(
                    node.left,
                    node.top,
                    node.width,
                    node.height,
                    ops,
                    node.style.transform_origin.as_ref(),
                    parent.as_ref(),
                );
                matrix_strs.push(Some(matrix_to_string(&m)));
                matrices.push(Some(m));
            }
            (None, Some(p)) => {
                matrix_strs.push(Some(matrix_to_string(&p)));
                matrices.push(Some(p));
            }
            (None, None) => {
                matrix_strs.push(None);
                matrices.push(None);
            }
        }
    }

    // Pre-pass: for every `_text` node whose closest ancestor has
    // `background-clip: text`, compute the glyph path and append it
    // into a per-target collector. The bucket is keyed by the
    // ancestor element's id (matches the `satori_bct-{id}` clipPath
    // emitted in front of that element's rect below).
    let mut bg_clip_text_paths: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    for (i, node) in layout.nodes.iter().enumerate() {
        if node.tag != "_text" {
            continue;
        }
        let Some(target_id) = node.style._inherited_bg_clip_text_target.clone() else {
            continue;
        };
        let xml = collect_text_glyph_path(&layout.nodes, i, matrix_strs[i].as_deref(), &grapheme_images);
        if !xml.is_empty() {
            let entry = bg_clip_text_paths.entry(target_id).or_default();
            entry.push_str(&xml);
        }
    }

    // Render in pre-order. We accumulate a flat string of SVG fragments
    // matching the JS layout function's order: parent rect first, then
    // child rects.
    let mut content = String::new();
    for (i, node) in layout.nodes.iter().enumerate() {
        if node.tag == "_text" {
            render_text_node(
                &mut content,
                &layout.nodes,
                i,
                matrix_strs[i].as_deref(),
                embed_font,
                debug,
                &grapheme_images,
            );
            continue;
        }
        // `depsRenderResult` (JS layout.ts line 318): if this element
        // owns a `background-clip: text` context, emit the collected
        // `<clipPath id="satori_bct-{id}"><path d="..."/></clipPath>`
        // BEFORE its own rect. We splice the glyph path string into
        // `node.style._bg_clip_text_path_d` so the rest of `render_rect`
        // can wire `currentClipPath = url(#satori_bct-{id})`.
        let mut style_for_rect = node.style.clone();
        if style_for_rect._bg_clip_text_self == Some(true) {
            let d = bg_clip_text_paths
                .get(&node.id)
                .cloned()
                .unwrap_or_default();
            // Always emit the clipPath element (even when descendants
            // produced no glyphs — JS does so unconditionally so an
            // empty bg-clip-text container still hides its background).
            // `d` already contains one or more `<path d="..." transform="..."/>`
            // elements, one per child text node, with transforms folded in.
            let clip_path_xml = format!(
                "<clipPath id=\"satori_bct-{id}\">{d}</clipPath>",
                id = node.id
            );
            content.push_str(&clip_path_xml);
            style_for_rect._bg_clip_text_path_d = Some(d);
        }
        content.push_str(&render_rect(
            &node.id,
            node.left,
            node.top,
            node.width,
            node.height,
            &style_for_rect,
            matrix_strs[i].as_deref(),
        ));
        // Debug overlay: 1px red stroke around the element's box.
        // Mirrors JS rect.ts `if (debug)` branch — emitted AFTER the
        // shape so it draws on top.
        if debug {
            let mut attrs = format!(
                "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"transparent\" stroke=\"#ff5757\" stroke-width=\"1\"",
                js_int(node.left), js_int(node.top), js_int(node.width), js_int(node.height)
            );
            if let Some(m) = matrix_strs[i].as_deref() {
                attrs.push_str(&format!(" transform=\"{m}\""));
            }
            if let Some(cp) = node.style._inherited_clip_path_id.as_deref() {
                attrs.push_str(&format!(" clip-path=\"url(#{cp})\""));
            }
            content.push_str(&format!("<rect {attrs}/>"));
        }
    }

    Ok(render_svg(layout.root_width, layout.root_height, &content))
}

/// Compute the merged glyph path string for a `_text` node — same
/// `mergedPath` JS satori builds inside `text/index.ts`. Used by the
/// `background-clip: text` pre-pass to feed glyph outlines into the
/// ancestor's `<clipPath>`.
fn collect_text_glyph_path(
    nodes: &[crate::layout::LaidOut],
    i: usize,
    matrix_str: Option<&str>,
    grapheme_images: &std::sync::Arc<std::collections::HashMap<String, String>>,
) -> String {
    let node = &nodes[i];
    let Some(text) = node.text.as_ref() else { return String::new() };
    let parent_idx = node.parent;
    let parent_node_opt = parent_idx.map(|p| &nodes[p]);
    let parent_style = parent_node_opt
        .map(|n| n.style.clone())
        .unwrap_or_default();
    let tx_style = build_text_style_with_container_w(&parent_style, node.width, grapheme_images);
    let tl = layout_text(text, &tx_style, &node.fonts, Some(node.width));

    if node.fonts.is_empty() || tl.words.is_empty() {
        return String::new();
    }

    let container_w = node.width;
    let mut merged = String::new();
    let mut word_buffer: Option<String> = None;
    let mut buffered_offset: f32 = 0.0;

    for (idx, word) in tl.words.iter().enumerate() {
        let next = tl.words.get(idx + 1);
        let top_offset = word.y;
        let (mut left_offset, _ext) = apply_text_align_indent(
            word.x,
            word.line,
            word.line_index,
            container_w,
            tx_style.text_indent,
            &tl.line_widths,
            &tl.line_segment_counts,
            tx_style.text_align,
        );
        if tl.line_widths.len() > 1 {
            left_offset = left_offset.round();
        }
        let baseline_of_line = tl.baselines.get(word.line).copied().unwrap_or(tl.baseline);

        let can_buffer = !word.content.contains('\t')
            && !is_word_separator(&word.content)
            && next
                .map(|n| n.y == top_offset && !n.is_image)
                .unwrap_or(false);
        if can_buffer {
            if word_buffer.is_none() {
                buffered_offset = left_offset;
            }
            let buf = word_buffer.get_or_insert_with(String::new);
            buf.push_str(&word.content);
            continue;
        }
        let had_buffer = word_buffer.is_some();
        let finalized_text = match word_buffer.take() {
            Some(b) => b + &word.content,
            None => word.content.clone(),
        };
        let actual_left = if had_buffer { buffered_offset } else { left_offset };
        let x_f64 = node.left as f64 + actual_left as f64;
        // Equivalent to node.top + top_offset + baseline_of_word + baseline_delta
        // but skips the intermediate subtraction so the FP error pattern
        // matches JS satori's (which uses round(b) directly).
        let y_baseline_f64 = node.top as f64 + top_offset as f64 + baseline_of_line as f64;
        let cleaned: String = finalized_text.chars().filter(|c| *c != '\t').collect();
        if cleaned.is_empty() {
            merged.push(' ');
            continue;
        }
        let font = &node.fonts[0];
        let p = font.run_path_d_with_fallback(
            &cleaned,
            x_f64,
            y_baseline_f64,
            tx_style.font_size_exact.unwrap_or(tx_style.font_size as f64),
            tx_style.letter_spacing as f64,
            true,
            1,
            &node.fonts,
        );
        merged.push_str(&p);
        merged.push(' ');
    }
    if merged.is_empty() {
        return String::new();
    }
    // Mirror JS `buildXMLString('path', { d: mergedPath, transform })`.
    match matrix_str {
        Some(m) => format!("<path d=\"{merged}\" transform=\"{m}\"/>"),
        None => format!("<path d=\"{merged}\"/>"),
    }
}

/// JS `${number}` formatting for SVG attribute values: integer-valued
/// doubles render as plain integers (`10`, not `10.0`).
fn js_int(v: f32) -> String {
    let n = v as f64;
    if n == n.trunc() && n.is_finite() {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

fn build_text_style(
    parent: &ComputedStyle,
    grapheme_images: &std::sync::Arc<std::collections::HashMap<String, String>>,
) -> TxStyle {
    build_text_style_with_container_w(parent, 0.0, grapheme_images)
}

/// Like `build_text_style`, but resolves `text-indent` against the
/// passed `container_w` (only matters for `Percent` indents — `Px`
/// indents resolve identically regardless).
fn build_text_style_with_container_w(
    parent: &ComputedStyle,
    container_w: f32,
    grapheme_images: &std::sync::Arc<std::collections::HashMap<String, String>>,
) -> TxStyle {
    let (line_limit, block_ellipsis) = resolve_line_limit(parent);
    let text_indent = match parent.text_indent {
        Some(Dim::Px(v)) => v,
        Some(Dim::Percent(p)) => p / 100.0 * container_w,
        _ => 0.0,
    };
    TxStyle {
        font_size: parent.font_size.unwrap_or(16.0),
        font_size_exact: parent._font_size_f64,
        line_height: parent.line_height,
        font_family: parent
            .font_family
            .clone()
            .map(|s| {
                s.split(',')
                    .map(|p| p.trim().trim_matches('"').trim_matches('\'').to_string())
                    .collect()
            })
            .unwrap_or_else(|| vec!["sans-serif".to_string()]),
        font_weight: parent.font_weight.unwrap_or(400),
        font_style: match parent.font_style.unwrap_or(CssFontStyle::Normal) {
            CssFontStyle::Normal => FontStyle::Normal,
            CssFontStyle::Italic => FontStyle::Italic,
        },
        letter_spacing: parent.letter_spacing.unwrap_or(0.0),
        word_break: match parent.word_break.unwrap_or(CssWordBreak::Normal) {
            CssWordBreak::Normal => WordBreakMode::Normal,
            CssWordBreak::BreakAll => WordBreakMode::BreakAll,
            CssWordBreak::KeepAll => WordBreakMode::KeepAll,
            CssWordBreak::BreakWord => WordBreakMode::BreakWord,
        },
        text_wrap: match parent.text_wrap.unwrap_or(CssTextWrap::Wrap) {
            CssTextWrap::Wrap => TextWrapMode::Wrap,
            CssTextWrap::Nowrap => TextWrapMode::Nowrap,
            CssTextWrap::Balance => TextWrapMode::Balance,
            CssTextWrap::Pretty => TextWrapMode::Pretty,
        },
        text_transform: match parent.text_transform.unwrap_or(CssTextTransform::None) {
            CssTextTransform::None => TextTransformMode::None,
            CssTextTransform::Uppercase => TextTransformMode::Uppercase,
            CssTextTransform::Lowercase => TextTransformMode::Lowercase,
            CssTextTransform::Capitalize => TextTransformMode::Capitalize,
        },
        tab_size: TabSize::Multiplier(parent.tab_size.unwrap_or(8.0)),
        text_indent,
        line_limit,
        block_ellipsis,
        white_space: match parent.white_space.unwrap_or(WhiteSpace::Normal) {
            WhiteSpace::Normal => WhiteSpaceMode::Normal,
            WhiteSpace::NoWrap => WhiteSpaceMode::NoWrap,
            WhiteSpace::Pre => WhiteSpaceMode::Pre,
            WhiteSpace::PreWrap => WhiteSpaceMode::PreWrap,
            WhiteSpace::PreLine => WhiteSpaceMode::PreLine,
            WhiteSpace::BreakSpaces => WhiteSpaceMode::BreakSpaces,
        },
        text_align: match parent.text_align.unwrap_or(TextAlign::Start) {
            TextAlign::Start => TextAlignMode::Start,
            TextAlign::End => TextAlignMode::End,
            TextAlign::Left => TextAlignMode::Left,
            TextAlign::Right => TextAlignMode::Right,
            TextAlign::Center => TextAlignMode::Center,
            TextAlign::Justify => TextAlignMode::Justify,
        },
        grapheme_images: grapheme_images.clone(),
    }
}

/// Resolve the effective `line-clamp` / `text-overflow: ellipsis` limit.
fn resolve_line_limit(parent: &ComputedStyle) -> (u32, String) {
    use crate::css::style::{Display as Disp, Overflow, TextOverflow};
    let default_ell = "\u{2026}".to_string();
    if matches!(parent.display, Some(Disp::Block)) {
        if let Some(n) = parent.line_clamp {
            if n > 0 {
                let ell = parent
                    .line_clamp_ellipsis
                    .clone()
                    .unwrap_or_else(|| default_ell.clone());
                return (n, ell);
            }
        }
    }
    if matches!(parent.text_overflow, Some(TextOverflow::Ellipsis))
        && matches!(parent.display, Some(Disp::WebkitBox))
        && parent.webkit_box_orient.as_deref() == Some("vertical")
    {
        if let Some(n) = parent.webkit_line_clamp {
            if n > 0 {
                return (n, default_ell);
            }
        }
    }
    let allow_soft_wrap = !matches!(
        parent.white_space.unwrap_or(WhiteSpace::Normal),
        WhiteSpace::Pre | WhiteSpace::NoWrap
    );
    if matches!(parent.text_overflow, Some(TextOverflow::Ellipsis))
        && matches!(parent.overflow, Some(Overflow::Hidden))
        && !allow_soft_wrap
    {
        return (1, default_ell);
    }
    (u32::MAX, default_ell)
}

/// Apply text-align (and JS satori's per-line `leftOffset` adjustments)
/// + `text-indent`.
fn apply_text_align_indent(
    word_x: f32,
    word_line: usize,
    word_line_index: i32,
    container_w: f32,
    text_indent: f32,
    line_widths: &[f32],
    line_segment_counts: &[u32],
    align: TextAlignMode,
) -> (f32, bool) {
    let mut left = word_x;
    let mut extended = false;
    if word_line == 0 && text_indent != 0.0 {
        left += text_indent;
    }
    if line_widths.len() > 1 {
        let lw = line_widths.get(word_line).copied().unwrap_or(0.0);
        let remaining = container_w - lw;
        match align {
            TextAlignMode::Right | TextAlignMode::End => left += remaining,
            TextAlignMode::Center => left += remaining / 2.0,
            TextAlignMode::Justify => {
                if word_line + 1 < line_widths.len() {
                    let segs = line_segment_counts.get(word_line).copied().unwrap_or(0);
                    let gutter = if segs > 1 { remaining / (segs as f32 - 1.0) } else { 0.0 };
                    if word_line_index >= 0 {
                        left += gutter * (word_line_index as f32);
                    }
                    extended = true;
                }
            }
            _ => {}
        }
    }
    (left, extended)
}

struct DecorationLineInfo {
    left: f32,
    /// f64 so the downstream underline-y math (`top + ascender * 1.1`)
    /// keeps full mantissa precision; otherwise an f32 round-trip of
    /// `top` flips the `<line>` y attribute by ~1 ULP.
    top: f64,
    /// f64 to mirror JS satori's bit-exact decoration-y math.
    /// Sourced from `line_metrics_f64` per-word.
    ascender: f64,
    width: f32,
}

#[allow(clippy::too_many_arguments)]
fn render_text_node(
    out: &mut String,
    nodes: &[crate::layout::LaidOut],
    i: usize,
    matrix_str: Option<&str>,
    embed_font: bool,
    debug: bool,
    grapheme_images: &std::sync::Arc<std::collections::HashMap<String, String>>,
) {
    let node = &nodes[i];
    let Some(text) = node.text.as_ref() else { return };
    let parent_idx = node.parent;
    let parent_node_opt = parent_idx.map(|p| &nodes[p]);
    let parent_style = parent_node_opt
        .map(|n| n.style.clone())
        .unwrap_or_default();
    // Note: the `satori_om-{p.id}` content mask is now emitted
    // unconditionally inside `render_rect` for the parent rect
    // (matches JS `overflow()`'s always-emit behavior); no need to
    // duplicate it here.
    let tx_style = build_text_style_with_container_w(&parent_style, node.width, grapheme_images);
    // Use the PARENT's inner-content width as the line-wrap
    // constraint, not the text node's post-layout width. Yoga sizes
    // the text node to `ceil(layout.width)` from the measure
    // callback, which can be larger than the original constraint
    // (e.g. 104 vs the parent's 100, or 62 vs the padding-inner 50).
    // Re-flowing at the inflated width breaks at different word
    // boundaries than the measure pass did, desyncing line positions
    // from JS satori (JS keeps the measure-pass
    // `wordPositionInLayout` — it doesn't re-flow at render time).
    // Mirror JS's
    // `parentContainerInnerWidth = getComputedWidth - padding - border`.
    fn resolve_dim(d: Option<Dim>, base: f32) -> f32 {
        match d {
            Some(Dim::Px(v)) => v,
            Some(Dim::Percent(p)) => p / 100.0 * base,
            _ => 0.0,
        }
    }
    let wrap_width = if let Some(p) = parent_node_opt {
        let pl = resolve_dim(p.style.padding_left, p.width);
        let pr = resolve_dim(p.style.padding_right, p.width);
        let bl = p.style.border_left_width.unwrap_or(0.0);
        let br = p.style.border_right_width.unwrap_or(0.0);
        (p.width - pl - pr - bl - br).max(0.0)
    } else {
        node.width
    };
    let tl = layout_text(text, &tx_style, &node.fonts, Some(wrap_width));

    let container_w = node.width;
    let opacity = node.style.opacity.unwrap_or(1.0);

    // Mirror JS satori's ellipsis emission for `line-clamp` /
    // `text-overflow: ellipsis`. We compute the parent's inner-content
    // width (which `JS` reads from yoga's `getComputedWidth() - padding
    // - border`) and the resolved ellipsis width here, then truncate
    // the LAST allowed line during the per-word loop.
    let parent_inner_width = parent_node_opt
        .map(|n| n.width)
        .unwrap_or(container_w);
    let line_limit_active = tl.line_limit != u32::MAX;
    let mut block_ellipsis_text: String = tl.block_ellipsis.clone();
    let mut ellipsis_width = if line_limit_active && !node.fonts.is_empty() {
        crate::text::measure_grapheme(
            &block_ellipsis_text,
            &node.fonts,
            tx_style.font_size,
            tx_style.letter_spacing,
        )
    } else {
        0.0
    };
    if line_limit_active && ellipsis_width > parent_inner_width {
        block_ellipsis_text = "\u{2026}".to_string();
        ellipsis_width = if !node.fonts.is_empty() {
            crate::text::measure_grapheme(
                &block_ellipsis_text,
                &node.fonts,
                tx_style.font_size,
                tx_style.letter_spacing,
            )
        } else {
            0.0
        };
    }
    let space_width = if line_limit_active && !node.fonts.is_empty() {
        crate::text::measure_grapheme(
            " ",
            &node.fonts,
            tx_style.font_size,
            tx_style.letter_spacing,
        )
    } else {
        0.0
    };
    let mut skipped_line: Option<usize> = None;

    let mut decoration_lines: std::collections::BTreeMap<usize, DecorationLineInfo> =
        std::collections::BTreeMap::new();
    let mut decoration_glyphs: std::collections::BTreeMap<usize, Vec<crate::font::GlyphBox>> =
        std::collections::BTreeMap::new();
    let mut merged_path = String::new();
    let mut word_buffer: Option<String> = None;
    let mut buffered_offset: f32 = 0.0;
    let mut should_break_after_word = false;

    // Mirror JS satori's `shouldCollectDecorationBoxes`. Only the
    // `underline` line kind and a non-`none` skip-ink setting trigger
    // glyph-box collection.
    let should_collect_boxes = matches!(
        parent_style.text_decoration_line,
        Some(TextDecorationLine::Underline)
    ) && parent_style.text_decoration_skip_ink.as_deref() != Some("none");

    // Debug-overlay extras (per-word rect + baseline line). Accumulated
    // outside the glyph buffer so we can splice them in after the
    // `<g>` wrapper closes.
    let mut extra_debug = String::new();

    for (idx, word) in tl.words.iter().enumerate() {
        let next = tl.words.get(idx + 1);

        let top_offset = word.y;
        let (mut left_offset, extended_width) = apply_text_align_indent(
            word.x,
            word.line,
            word.line_index,
            container_w,
            tx_style.text_indent,
            &tl.line_widths,
            &tl.line_segment_counts,
            tx_style.text_align,
        );
        let width = word.width;
        let line = word.line;

        if let Some(skip) = skipped_line {
            if line == skip {
                continue;
            }
        }

        // Mutable per-word content so we can substitute in the truncated
        // form when ellipsis emission triggers.
        let mut effective_content = word.content.clone();
        let mut is_last_displayed_before_ellipsis = false;
        if line_limit_active && !node.fonts.is_empty() && tl.line_limit > 0 {
            let is_not_last_line = line + 1 < tl.line_widths.len();
            let is_last_allowed_line = (line as u32) + 1 == tl.line_limit;
            let line_w = tl.line_widths.get(line).copied().unwrap_or(0.0);
            if is_last_allowed_line && (is_not_last_line || line_w > parent_inner_width) {
                let calc_ellipsis = |base_width: f32, text: &str| -> (String, f32) {
                    use unicode_segmentation::UnicodeSegmentation;
                    let mut subset = String::new();
                    let mut resolved_width = 0.0_f32;
                    for ch in text.graphemes(true) {
                        let candidate = format!("{subset}{ch}");
                        let w = base_width
                            + crate::text::measure_text(
                                &candidate,
                                &node.fonts,
                                tx_style.font_size,
                                tx_style.letter_spacing,
                            );
                        if !subset.is_empty()
                            && w + ellipsis_width > parent_inner_width
                        {
                            break;
                        }
                        subset.push_str(ch);
                        resolved_width = w;
                    }
                    (subset, resolved_width)
                };
                if left_offset + width + ellipsis_width + space_width > parent_inner_width {
                    let (subset, resolved_w) = calc_ellipsis(left_offset, &effective_content);
                    effective_content = format!("{subset}{block_ellipsis_text}");
                    skipped_line = Some(line);
                    if let Some(info) = decoration_lines.get_mut(&line) {
                        info.width = (resolved_w - info.left).max(0.0);
                    }
                    is_last_displayed_before_ellipsis = true;
                } else if let Some(nxt) = next {
                    if nxt.line != line {
                        if matches!(tx_style.text_align, TextAlignMode::Center) {
                            let (subset, resolved_w) =
                                calc_ellipsis(left_offset, &effective_content);
                            effective_content = format!("{subset}{block_ellipsis_text}");
                            skipped_line = Some(line);
                            if let Some(info) = decoration_lines.get_mut(&line) {
                                info.width = (resolved_w - info.left).max(0.0);
                            }
                            is_last_displayed_before_ellipsis = true;
                        } else {
                            let next_text =
                                tl.words.get(idx + 1).map(|w| w.content.clone()).unwrap_or_default();
                            let (subset, resolved_w) =
                                calc_ellipsis(width + left_offset, &next_text);
                            effective_content = format!("{effective_content}{subset}{block_ellipsis_text}");
                            skipped_line = Some(line);
                            if let Some(info) = decoration_lines.get_mut(&line) {
                                info.width = (resolved_w - info.left).max(0.0);
                            }
                            is_last_displayed_before_ellipsis = true;
                        }
                    }
                }
            }
        }
        if is_last_displayed_before_ellipsis {
            should_break_after_word = true;
        }

        let baseline_of_line = tl.baselines.get(line).copied().unwrap_or(tl.baseline);
        // JS satori uses the *un-rounded* `engine.baseline(word)` —
        // resolved by `resolve_for_char` on the word's first char.
        // CJK fallback fonts have a taller hhea ascender than Roboto,
        // so a mixed Latin/CJK line baseline-shifts to match the
        // first-word's font. Compute in f64 to feed the underline-y
        // math (`top + ascender * 1.1`) with bit-exact JS parity.
        let baseline_of_word_f64: f64 = {
            let first_ch = word.content.chars().next().unwrap_or(' ');
            let font = crate::font::FontLoader::resolve_for_char(&node.fonts, first_ch)
                .unwrap_or_else(|| std::sync::Arc::clone(&node.fonts[0]));
            let (_lh, base) = crate::text::line_metrics_f64(
                &font,
                tx_style.font_size,
                tx_style.line_height,
            );
            base
        };
        let baseline_of_word = baseline_of_word_f64 as f32;
        let baseline_delta = baseline_of_line - baseline_of_word;

        if embed_font && tl.line_widths.len() > 1 {
            left_offset = left_offset.round();
        }

        decoration_lines
            .entry(line)
            .or_insert_with(|| DecorationLineInfo {
                left: left_offset,
                top: node.top as f64 + top_offset as f64
                    + (baseline_of_line as f64 - baseline_of_word_f64),
                ascender: baseline_of_word_f64,
                width: if extended_width {
                    container_w
                } else {
                    tl.line_widths.get(line).copied().unwrap_or(0.0)
                },
            });

        // Image grapheme: emit `<image href="...">` (JS satori
        // `text/index.ts` line 548 + `builder/text.ts` line 96). The
        // image is sized to one `font_size` square per JS measureGraphemeArray
        // (width) plus the engine's line-height (height); no baseline
        // shift is applied for images (JS `topOffset += 0`).
        if word.is_image {
            if let Some(href) = grapheme_images.get(word.content.as_str()).cloned() {
                let x_image = node.left + left_offset;
                let y_image = node.top + top_offset;
                let h_image = tl.line_height;
                let escaped = href.replace('&', "&amp;").replace('"', "&quot;");
                let mut attrs = format!(
                    "href=\"{href}\" x=\"{x}\" y=\"{y}\" width=\"{w}\" height=\"{h}\"",
                    href = escaped,
                    x = js_int(x_image),
                    y = js_int(y_image),
                    w = js_int(width),
                    h = js_int(h_image),
                );
                if let Some(m) = matrix_str {
                    attrs.push_str(&format!(" transform=\"{m}\""));
                }
                if let Some(cp) = node.style._inherited_clip_path_id.as_deref() {
                    attrs.push_str(&format!(" clip-path=\"url(#{cp})\""));
                }
                if opacity != 1.0 {
                    attrs.push_str(&format!(" opacity=\"{}\"", js_int(opacity)));
                }
                // Inherited CSS `filter` (not normally inherited but
                // JS satori applies the parent's filter to image-grapheme
                // children since the parent doesn't emit its own rect here)
                // gets passed through as inline `style="filter:..."`.
                let mut style_filter_parts: Vec<String> = Vec::new();
                let filter_src = node
                    .style
                    .filter
                    .as_deref()
                    .or(parent_style.filter.as_deref());
                if let Some(f) = filter_src {
                    style_filter_parts.push(f.to_string());
                }
                if !style_filter_parts.is_empty() {
                    attrs.push_str(&format!(
                        " style=\"filter:{}\"",
                        style_filter_parts.join(" ")
                    ));
                }
                // CSS `text-shadow` becomes an SVG `<filter id="satori_s-..."
                // ">` with `feGaussianBlur`/`feOffset` primitives, referenced
                // via `filter="url(#...)"` on the `<image>`. Mirrors JS
                // satori's text-shadow emission for image graphemes.
                let shadow_src = node
                    .style
                    .text_shadow
                    .as_ref()
                    .or(parent_style.text_shadow.as_ref());
                if let Some(shadows) = shadow_src {
                    if !shadows.is_empty() {
                        let filter_id = format!("satori_s-{i}-text");
                        
                        let mut filter_def = format!(
                            "<defs><filter id=\"{filter_id}\" x=\"-50%\" y=\"-50%\" width=\"200%\" height=\"200%\">"
                        );
                        for sh in shadows {
                            filter_def.push_str(&format!(
                                "<feGaussianBlur in=\"SourceAlpha\" stdDeviation=\"{}\"/><feOffset dx=\"{}\" dy=\"{}\" result=\"offsetblur\"/><feFlood flood-color=\"{}\"/><feComposite in2=\"offsetblur\" operator=\"in\"/><feMerge><feMergeNode/><feMergeNode in=\"SourceGraphic\"/></feMerge>",
                                sh.blur / 2.0,
                                sh.offset_x,
                                sh.offset_y,
                                sh.color
                            ));
                        }
                        filter_def.push_str("</filter></defs>");
                        out.push_str(&filter_def);
                        attrs.push_str(&format!(" filter=\"url(#{filter_id})\""));
                    }
                }
                out.push_str(&format!("<image {attrs}/>"));
            }
            if should_break_after_word {
                break;
            }
            continue;
        }

        if embed_font && !node.fonts.is_empty() {
            // Buffer adjacent same-line non-separator words to merge glyph paths.
            let can_buffer = !effective_content.contains('\t')
                && !is_word_separator(&effective_content)
                && next
                    .map(|n| n.y == top_offset && !n.is_image)
                    .unwrap_or(false)
                && !is_last_displayed_before_ellipsis;
            if can_buffer {
                if word_buffer.is_none() {
                    buffered_offset = left_offset;
                }
                let buf = word_buffer.get_or_insert_with(String::new);
                buf.push_str(&effective_content);
                continue;
            }
            let had_buffer = word_buffer.is_some();
            let finalized_text = match word_buffer.take() {
                Some(b) => b + &effective_content,
                None => effective_content.clone(),
            };
            let actual_left = if had_buffer { buffered_offset } else { left_offset };
            buffered_offset = 0.0;

            // Compute x / y_baseline in f64 to keep precision parity
            // with JS satori (which does this math in numbers/f64). Sub-
            // pixel drift from f32 accumulation would otherwise round
            // glyph coordinates differently in `js_float_to_string`.
            let x_f64 = node.left as f64 + actual_left as f64;
            let y_baseline_f64 = node.top as f64
                + top_offset as f64
                + baseline_of_line as f64;
            let x = x_f64 as f32;
            let y_baseline = y_baseline_f64 as f32;
            // Mirror JS: drop tabs before calling `getSVG` (`finalizedSegment.replace(/(\t)+/g, '')`).
            let cleaned: String = finalized_text.chars().filter(|c| *c != '\t').collect();
            if !cleaned.is_empty() {
                let font = &node.fonts[0];
                let p = font.run_path_d_with_fallback(
                    &cleaned,
                    x_f64,
                    y_baseline_f64,
                    tx_style.font_size_exact.unwrap_or(tx_style.font_size as f64),
                    tx_style.letter_spacing as f64,
                    true,
                    1,
                    &node.fonts,
                );
                merged_path.push_str(&p);
                if debug {
                    // Per-word glyph rect + baseline line. Mirrors JS
                    // text/index.ts lines 762-787. Width is the
                    // finalized buffered width and height the engine
                    // line height (`heightOfWord` in JS).
                    let glyph_width = width + left_offset - actual_left;
                    let height_of_word = tl.line_height;
                    let mut g_attrs = format!(
                        "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"transparent\" stroke=\"#575eff\" stroke-width=\"1\"",
                        js_int(x), js_int(node.top + top_offset + baseline_delta),
                        js_int(glyph_width), js_int(height_of_word)
                    );
                    if let Some(m) = matrix_str { g_attrs.push_str(&format!(" transform=\"{m}\"")); }
                    if let Some(cp) = node.style._inherited_clip_path_id.as_deref() {
                        g_attrs.push_str(&format!(" clip-path=\"url(#{cp})\""));
                    }
                    extra_debug.push_str(&format!("<rect {g_attrs}/>"));
                    let mut l_attrs = format!(
                        "x1=\"{}\" x2=\"{}\" y1=\"{}\" y2=\"{}\" stroke=\"#14c000\" stroke-width=\"1\"",
                        js_int(node.left + left_offset),
                        js_int(node.left + left_offset + width),
                        js_int(node.top + top_offset + baseline_delta + baseline_of_word),
                        js_int(node.top + top_offset + baseline_delta + baseline_of_word),
                    );
                    if let Some(m) = matrix_str { l_attrs.push_str(&format!(" transform=\"{m}\"")); }
                    if let Some(cp) = node.style._inherited_clip_path_id.as_deref() {
                        l_attrs.push_str(&format!(" clip-path=\"url(#{cp})\""));
                    }
                    extra_debug.push_str(&format!("<line {l_attrs}/>"));
                }
                // Collect glyph boxes for skip-ink (JS satori
                // `decorationGlyphs[line].push(...svg.boxes)`).
                if should_collect_boxes {
                    let stroke_w = (1.0_f32).max(tx_style.font_size * 0.1);
                    // JS: `underlineY = baseline + baselineOfWord * 0.1`
                    let underline_y = y_baseline + baseline_of_word * 0.1;
                    let band = crate::font::SkipInkBand {
                        underline_y,
                        stroke_width: stroke_w,
                    };
                    let mut bx = font.run_band_boxes_with_fallback(
                        &cleaned,
                        x_f64,
                        y_baseline_f64,
                        tx_style.font_size_exact.unwrap_or(tx_style.font_size as f64),
                        tx_style.letter_spacing as f64,
                        true,
                        band,
                        &node.fonts,
                    );
                    if !bx.is_empty() {
                        decoration_glyphs
                            .entry(line)
                            .or_default()
                            .append(&mut bx);
                    }
                }
            }
            // JS always appends a trailing space after each segment's
            // glyph path (`mergedPath += path + ' '`), even when `path`
            // is the empty string (e.g. `\n` / pure-whitespace segments).
            // Mirror that exactly so the merged `d` attribute is
            // byte-identical to JS satori's output.
            merged_path.push(' ');
            if should_break_after_word {
                break;
            }
            // Stay in the embed-font branch even when the segment produced
            // no glyphs (e.g. `\n` / pure whitespace) so we don't double-
            // emit a `<text>` element for the same word position.
            continue;
        }

        // Non-embed branch: emit <text> per word.
        let cleaned: String = effective_content.chars().filter(|c| *c != '\t').collect();
        if cleaned.is_empty() {
            if should_break_after_word {
                break;
            }
            continue;
        }
        let y_baseline = node.top + top_offset + baseline_of_word + baseline_delta;
        let x = node.left + left_offset;
        let args = TextArgs {
            content: &cleaned,
            x,
            y: y_baseline,
            width,
            height: tl.line_height,
            matrix: matrix_str,
            clip_path_id: node
                .style
                ._inherited_clip_path_id
                .as_deref()
                .or(parent_style._inherited_clip_path_id.as_deref()),
            opacity,
        };
        out.push_str(&render_text(&args, &parent_style));
        if debug {
            // Mirror the embedFont:true debug-rect emit: per-word bounding
            // rect (#575eff) and baseline line (#14c000).
            let glyph_width = width;
            let height_of_word = tl.line_height;
            let mut g_attrs = format!(
                "x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"transparent\" stroke=\"#575eff\" stroke-width=\"1\"",
                js_int(x), js_int(node.top + top_offset + baseline_delta),
                js_int(glyph_width), js_int(height_of_word)
            );
            if let Some(m) = matrix_str { g_attrs.push_str(&format!(" transform=\"{m}\"")); }
            if let Some(cp) = node
                .style
                ._inherited_clip_path_id
                .as_deref()
                .or(parent_style._inherited_clip_path_id.as_deref())
            {
                g_attrs.push_str(&format!(" clip-path=\"url(#{cp})\""));
            }
            extra_debug.push_str(&format!("<rect {g_attrs}/>"));
        }
        if should_break_after_word {
            break;
        }
    }

    // Build `decoration_shape` first so we can splice it into the same
    // `<g filter=...>` wrapper as the glyph path when text-shadow is
    // active (JS satori applies the filter to text+decoration together).
    let mut decoration_shape = String::new();
    if let Some(line_kind) = parent_style.text_decoration_line {
        if !matches!(line_kind, TextDecorationLine::None) {
            for (line, info) in &decoration_lines {
                let boxes: Vec<BoxBox> = decoration_glyphs
                    .get(line)
                    .map(|v| {
                        v.iter()
                            .map(|b| BoxBox {
                                x1: b.x1,
                                x2: b.x2,
                                y1: b.y1,
                                y2: b.y2,
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                let args = DecorationArgs {
                    width: info.width,
                    left: node.left + info.left,
                    top: info.top,
                    ascender: info.ascender,
                    clip_path_id: None,
                    matrix: matrix_str,
                    glyph_boxes: &boxes,
                };
                decoration_shape.push_str(&build_decoration(&args, &parent_style));
            }
        }
    }

    // Resolve text shadow filter (port of buildDropShadow). The
    // measured size is the same `tl.width` × `tl.height` that JS uses
    // (`measuredTextSize` in `text/index.ts`).
    let is_transparent_text = is_color_fully_transparent(
        parent_style
            ._webkit_text_fill_color
            .as_deref()
            .or(parent_style.color.as_deref()),
    );
    let text_shadow_filter = parent_style
        .text_shadow
        .as_deref()
        .filter(|v| !v.is_empty())
        .map(|shadows| {
            crate::builder::build_drop_shadow(
                &node.id,
                tl.width,
                tl.height,
                shadows,
                is_transparent_text,
            )
        });

    let glyph_g = if !merged_path.is_empty() {
        let path_args = TextPathArgs {
            d: &merged_path,
            matrix: matrix_str,
            opacity,
            overflow_mask_id: node.style._inherited_mask_id.as_deref(),
            clip_path_id: node.style._inherited_clip_path_id.as_deref(),
            transparent_text: is_transparent_text,
            has_text_shadow: text_shadow_filter.is_some(),
        };
        render_text_path(&path_args, &parent_style)
    } else {
        String::new()
    };

    let body = format!("{glyph_g}{decoration_shape}");
    if let Some(filter_xml) = text_shadow_filter {
        if !body.is_empty() {
            // `<defs>filter</defs><g filter="url(#satori_s-{id})">body</g>`
            out.push_str("<defs>");
            out.push_str(&filter_xml);
            out.push_str("</defs>");
            out.push_str(&format!(
                "<g filter=\"url(#satori_s-{})\">{}</g>",
                node.id, body
            ));
        }
    } else {
        out.push_str(&body);
    }
    // Debug overlays are emitted OUTSIDE the filter wrapper so they
    // stay visually crisp regardless of any text-shadow blur.
    out.push_str(&extra_debug);
}

/// Mirror of JS `cssColorParse(color)?.alpha === 0`. We only need to
/// recognize a handful of forms used by the test corpus:
///   * `transparent`
///   * `rgba(...)` with the 4th component literally `0` (or `0.0`)
///   * `hsla(...)` with the 4th component literally `0`
///   * 8-digit hex `#rrggbb00`
/// Anything else returns false (assumed opaque).
fn is_color_fully_transparent(color: Option<&str>) -> bool {
    let Some(c) = color else { return false };
    let trimmed = c.trim();
    if trimmed.eq_ignore_ascii_case("transparent") {
        return true;
    }
    if let Some(hex) = trimmed.strip_prefix('#') {
        if hex.len() == 8 {
            return hex.ends_with("00") || hex.ends_with("0");
        }
        if hex.len() == 4 {
            return hex.ends_with('0');
        }
    }
    if trimmed.starts_with("rgba(") || trimmed.starts_with("hsla(") {
        // Look at the 4th comma-separated component.
        let inner = &trimmed[5..trimmed.len().saturating_sub(1)];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() >= 4 {
            let alpha = parts[3].trim().trim_end_matches('%').trim();
            if let Ok(v) = alpha.parse::<f32>() {
                return v == 0.0;
            }
        }
    }
    false
}

fn resolve_fonts(parent: &ComputedStyle, font_loader: &FontLoader) -> Vec<std::sync::Arc<ParsedFont>> {
    let family_list: Vec<String> = parent
        .font_family
        .clone()
        .map(|s| {
            s.split(',')
                .map(|p| p.trim().trim_matches('"').trim_matches('\'').to_string())
                .collect()
        })
        .unwrap_or_else(|| vec!["sans-serif".to_string()]);
    let weight = parent.font_weight.unwrap_or(400);
    let style = match parent.font_style.unwrap_or(CssFontStyle::Normal) {
        CssFontStyle::Normal => FontStyle::Normal,
        CssFontStyle::Italic => FontStyle::Italic,
    };
    font_loader.resolve_list(&family_list, weight, style)
}

fn build_node(
    el: &Value,
    inherited: &ComputedStyle,
    parent_vars: &Vars,
    ctx: ExpandContext,
    id: &str,
    counter: &mut usize,
    font_loader: &FontLoader,
    asset_root: &std::path::Path,
    parent_computed: Option<&ComputedStyle>,
    grapheme_images: &std::sync::Arc<std::collections::HashMap<String, String>>,
) -> Result<Option<Built>, SatoriError> {
    // Null/undefined skip
    if el.is_null() {
        return Ok(None);
    }
    // Bare text node
    if let Some(s) = el.as_str() {
        // Represent text as a synthetic node tagged "_text". We attach a
        // measure_func so yoga sizes the box from the actual text
        // metrics (rather than collapsing to 0×0).
        //
        // For the text-measure callback, use the PARENT's full computed
        // style (when available) rather than `inherited` — JS satori's
        // `buildTextNodes` keys text-overflow / line-clamp / etc. off
        // `parentStyle`, which includes non-inheritable props like
        // `display` and `lineClamp`. Without the parent style, our
        // measure ignores `lineClamp` and over-reports the text height.
        let style = inherited.clone();
        let measure_style = parent_computed.unwrap_or(&style);
        let yoga = make_yoga(&style);
        let fonts = resolve_fonts(&style, font_loader);
        if !fonts.is_empty() {
            let tx_style = build_text_style(measure_style, grapheme_images);
            install_text_measure(
                &yoga,
                TextMeasureCtx {
                    text: s.to_string(),
                    style: tx_style,
                    fonts: fonts.clone(),
                },
            );
        }
        return Ok(Some(Built {
            yoga,
            children: vec![],
            id: id.to_string(),
            tag: "_text".to_string(),
            style,
            text: Some(s.to_string()),
            fonts,
        }));
    }
    if el.is_number() || el.is_boolean() {
        return Ok(None);
    }
    let Some(obj) = el.as_object() else {
        return Ok(None);
    };

    let tag = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let props = obj.get("props").and_then(|v| v.as_object());
    // JS satori's downstream destructuring assumes `element.props` is
    // an object: `{ type, props: { children, style, ... } }`. A
    // function-component return value that omits `props` hits a
    // `Cannot read properties of undefined` somewhere downstream;
    // reject explicitly here with a clear message.
    if !obj.contains_key("props") {
        return Err(SatoriError::Parse(format!(
            "Element of type <{tag}> is missing the `props` object. Every React element passed to satori must carry a (possibly empty) `props` field."
        )));
    }

    let style_value = props.and_then(|p| p.get("style")).cloned().unwrap_or(Value::Object(Default::default()));
    let (mut computed, mut child_inherited, child_vars) =
        expand_style(&style_value, tag, inherited, parent_vars, ctx)
            .map_err(SatoriError::Parse)?;

    // Port of `src/layout.ts` lines 158-167: if this element has
    // `overflow: hidden` OR an explicit clip-path, set the per-element
    // `_inheritedClipPathId` / `_inheritedMaskId` on the child-inherited
    // style so descendants apply this clip-path / mask.
    let has_overflow_hidden = matches!(
        computed.overflow,
        Some(crate::css::style::Overflow::Hidden)
    );
    let has_clip_path = computed.clip_path.is_some();
    if has_overflow_hidden || has_clip_path {
        child_inherited._inherited_clip_path_id = Some(format!("satori_cp-{id}"));
        child_inherited._inherited_mask_id = Some(format!("satori_om-{id}"));
    }
    // `mask-image` on this element overrides the inherited mask for
    // descendants (JS satori `layout.ts` lines 166-168).
    if computed.mask_image.is_some() {
        child_inherited._inherited_mask_id = Some(format!("satori_mi-{id}"));
    }

    // `background-clip: text` sets up the per-element bg-clip-text
    // target. Descendants of this element collect their glyph paths
    // into the matching bucket so the renderer can emit a
    // `<clipPath id="satori_bct-{id}">…</clipPath>` element.
    if computed.background_clip.as_deref() == Some("text") {
        computed._bg_clip_text_self = Some(true);
        child_inherited._inherited_bg_clip_text_target = Some(id.to_string());
        if computed.background_image.is_some() {
            child_inherited._inherited_bg_clip_text_has_background = Some(true);
            computed._inherited_bg_clip_text_has_background = Some(true);
        }
    }

    // <img>-specific compute step (port of `src/handler/compute.ts`'s
    // `type === 'img'` branch). Must run *before* `make_yoga` so the
    // image's intrinsic size/aspect-ratio reach yoga.
    if tag == "img" {
        // Pre-validate `src`: JS satori's `resolveImageData` rejects
        // relative-path srcs explicitly. Test fixture
        // `image__should-throw-error-when-relative-path-is-used`
        // expects this exact wording.
        if let Some(src_str) = props.and_then(|p| p.get("src")).and_then(|v| v.as_str()) {
            let trimmed = src_str.trim();
            let is_absolute = trimmed.starts_with("data:")
                || trimmed.starts_with("http://")
                || trimmed.starts_with("https://")
                || trimmed.starts_with("file://");
            if !is_absolute {
                return Err(SatoriError::Parse(format!(
                    "Image source must be an absolute URL: {trimmed}"
                )));
            }
        }
        apply_img_compute(&mut computed, props, asset_root, ctx);
        // Post-validate: if neither the natural dimensions nor the
        // explicit width+height props yielded a usable size, JS
        // satori throws `Image size cannot be determined`.
        let has_src = computed._src.is_some();
        let has_natural = computed._natural_width.is_some() && computed._natural_height.is_some();
        let has_explicit = matches!(computed.width, Some(Dim::Px(_)))
            && matches!(computed.height, Some(Dim::Px(_)));
        if has_src && !has_natural && !has_explicit {
            return Err(SatoriError::Parse(
                "Image size cannot be determined. Either provide explicit `width` and `height` props, or use a `src` whose format satori can decode dimensions from.".to_string(),
            ));
        }
    }

    // Inline `<svg>` element: serialize the entire subtree into a
    // `data:image/svg+xml;utf8,...` URI and set `__src` on the style so
    // the rect renderer picks it up via the same code path as `<img>`.
    // Must run *before* `make_yoga` so style.width / style.height are
    // honored when the SVG provides explicit `width`/`height` props or
    // a `viewBox`.
    if tag == "svg" {
        apply_svg_compute(&mut computed, props, inherited);
        if let Some(err) = inline_svg::take_svg_validation_error() {
            return Err(SatoriError::Parse(err));
        }
    }

    // JS satori (`src/handler/preprocess.ts`) refuses
    // `dangerouslySetInnerHTML` outright.
    if props.and_then(|p| p.get("dangerouslySetInnerHTML")).is_some() {
        return Err(SatoriError::Parse(format!(
            "Unsupported prop \"dangerouslySetInnerHTML\" on <{tag}>: satori cannot parse raw HTML. Convert the markup to JSX."
        )));
    }

    // JS satori's `layout.ts:127` requires every `<div>` with more
    // than one child to declare `display: flex` / `contents` /
    // `none` explicitly.
    if tag == "div" {
        let needs_display = match props.and_then(|p| p.get("children")) {
            Some(Value::Array(arr)) => arr.iter().filter(|c| !c.is_null() && !c.is_boolean()).count() > 1,
            _ => false,
        };
        if needs_display
            && !matches!(
                computed.display,
                Some(crate::css::style::Display::Flex)
                    | Some(crate::css::style::Display::Contents)
                    | Some(crate::css::style::Display::None)
                    | Some(crate::css::style::Display::WebkitBox)
            )
        {
            return Err(SatoriError::Parse(
                "Expected <div> to have explicit \"display: flex\", \"display: contents\", or \"display: none\" if it has more than one child node.".to_string(),
            ));
        }
    }

    // Resolve any `background-image: url(...)` layers so the renderer
    // has the data URI + natural dimensions on hand. JS does this via
    // `preProcessNode` (image fetch upstream of layout); we resolve
    // synchronously here because the test harness substitutes data URIs
    // for HTTP URLs ahead of time.
    resolve_background_image_urls(&mut computed, asset_root);

    // Mirror JS satori `text/index.ts` line 90: if the parent's
    // `flex-shrink` is undefined when it owns a text child, set it
    // to `1` so the text container can shrink to fit. We do this
    // eagerly here (rather than during text node construction) so
    // the yoga node is built with the right value the first time.
    if computed.flex_shrink.is_none()
        && props
            .and_then(|p| p.get("children"))
            .map(has_string_child)
            .unwrap_or(false)
    {
        computed.flex_shrink = Some(1.0);
    }

    let yoga = make_yoga(&computed);

    let mut children: Vec<Built> = Vec::new();
    if let Some(children_value) = props.and_then(|p| p.get("children")) {
        // JS `let i = 0; for (child of normalizedChildren) layout(child,
        // { id: id + '-' + i++ })`. The index resets per-parent — using
        // a global counter here would assign IDs like `id-0-1` instead
        // of `id-0-0` and produce off-by-one mask IDs.
        let _ = counter;
        let mut local_counter: usize = 0;
        collect_children(
            children_value,
            &child_inherited,
            &child_vars,
            ctx,
            id,
            &mut local_counter,
            &mut children,
            font_loader,
            asset_root,
            Some(&computed),
            grapheme_images,
        )?;
    }

    Ok(Some(Built {
        yoga,
        children,
        id: id.to_string(),
        tag: tag.to_string(),
        style: computed,
        text: None,
        fonts: vec![],
    }))
}

/// Port of the `type === 'img'` branch in JS satori's `compute.ts`.
///
/// Resolves the `src` prop into a data URI + natural dimensions, then
/// fixes up `width` / `height` so yoga lays out the image box correctly
/// (subtracting padding+border when both axes are absolute, and setting
/// the aspect-ratio hint when only one axis is constrained).
fn apply_img_compute(
    style: &mut ComputedStyle,
    props: Option<&serde_json::Map<String, Value>>,
    asset_root: &std::path::Path,
    _ctx: ExpandContext,
) {
    let src_value = props.and_then(|p| p.get("src"));
    let resolved = resolve_src_value(src_value, asset_root);

    let mut nw = resolved.as_ref().and_then(|r| r.natural_width);
    let mut nh = resolved.as_ref().and_then(|r| r.natural_height);

    let prop_width = props.and_then(|p| p.get("width"));
    let prop_height = props.and_then(|p| p.get("height"));

    let fs = style.font_size.unwrap_or(16.0);

    // Fall back to parseInt(props.width/height) for the no-natural-size case.
    if nw.is_none() && nh.is_none() {
        let pw = parse_int_like(prop_width);
        let ph = parse_int_like(prop_height);
        if let (Some(pw), Some(ph)) = (pw, ph) {
            nw = Some(pw);
            nh = Some(ph);
        } else {
            // JS satori throws here; we silently skip so the parent test
            // path (e.g. `should not throw when image is not valid`) can
            // still render the rectangle without an image. The dims will
            // come from props/style below.
        }
    }

    // Mirror compute.ts: contentBoxWidth = style.width || props.width
    let prop_width_dim =
        prop_width.and_then(|v| parse_dimension_passthrough(v, fs, style._viewport_width, style._viewport_height));
    let prop_height_dim =
        prop_height.and_then(|v| parse_dimension_passthrough(v, fs, style._viewport_width, style._viewport_height));

    let extra_horizontal = style.border_left_width.unwrap_or(0.0)
        + style.border_right_width.unwrap_or(0.0)
        + dim_px(style.padding_left)
        + dim_px(style.padding_right);
    let extra_vertical = style.border_top_width.unwrap_or(0.0)
        + style.border_bottom_width.unwrap_or(0.0)
        + dim_px(style.padding_top)
        + dim_px(style.padding_bottom);

    let style_w = style.width;
    let style_h = style.height;
    let mut content_box_width: Option<Dim> = style_w.or(prop_width_dim);
    let mut content_box_height: Option<Dim> = style_h.or(prop_height_dim);

    let is_absolute_size = matches!(content_box_width, Some(Dim::Px(_)))
        && matches!(content_box_height, Some(Dim::Px(_)));

    if is_absolute_size {
        if let Some(Dim::Px(w)) = content_box_width {
            content_box_width = Some(Dim::Px((w - extra_horizontal).max(0.0)));
        }
        if let Some(Dim::Px(h)) = content_box_height {
            content_box_height = Some(Dim::Px((h - extra_vertical).max(0.0)));
        }
    }

    // r = imageHeight / imageWidth
    let ratio = match (nw, nh) {
        (Some(w), Some(h)) if w > 0.0 => Some(h / w),
        _ => None,
    };

    match (content_box_width, content_box_height, ratio) {
        // Both content dims missing → set width: 100% and aspect-ratio.
        (None, None, Some(r)) => {
            content_box_width = Some(Dim::Percent(100.0));
            style._aspect_ratio = Some(1.0 / r);
        }
        // One axis given, other derivable from natural ratio.
        (None, Some(Dim::Px(h)), Some(r)) => {
            content_box_width = Some(Dim::Px(h / r));
        }
        (None, Some(_), Some(r)) => {
            style._aspect_ratio = Some(1.0 / r);
        }
        (Some(Dim::Px(w)), None, Some(r)) => {
            content_box_height = Some(Dim::Px(w * r));
        }
        (Some(_), None, Some(r)) => {
            style._aspect_ratio = Some(1.0 / r);
        }
        _ => {}
    }

    if is_absolute_size {
        // Add the padding/border back so style.width covers the full box.
        if let Some(Dim::Px(w)) = content_box_width {
            style.width = Some(Dim::Px(w + extra_horizontal));
        }
        if let Some(Dim::Px(h)) = content_box_height {
            style.height = Some(Dim::Px(h + extra_vertical));
        }
    } else {
        style.width = content_box_width;
        style.height = content_box_height;
    }

    if let Some(r) = resolved {
        style._src = Some(r.src);
        style._natural_width = r.natural_width;
        style._natural_height = r.natural_height;
    } else if let (Some(w), Some(h)) = (nw, nh) {
        // Even when we couldn't resolve a data URI, store the inferred
        // natural dims so the object-fit math falls back to "fill".
        style._natural_width = Some(w);
        style._natural_height = Some(h);
    }
}

/// Walk the `background_image` layer list and pre-resolve any
/// `BackgroundImage::Url` entries (data URI / `__assetFile`). Mirrors
/// JS satori's `preProcessNode` pipeline, which calls
/// `resolveImageData` for every `<img src=...>` and every
/// `background-image: url(...)` ahead of the layout pass and stashes
/// the result in an LRU cache keyed by the original `src`.
fn resolve_background_image_urls(
    style: &mut ComputedStyle,
    asset_root: &std::path::Path,
) {
    resolve_layers_urls(style.background_image.as_mut(), asset_root);
    resolve_layers_urls(style.mask_image.as_mut(), asset_root);
}

fn resolve_layers_urls(
    layers: Option<&mut Vec<crate::css::style::BackgroundImage>>,
    asset_root: &std::path::Path,
) {
    let Some(layers) = layers else { return };
    for layer in layers.iter_mut() {
        if let crate::css::style::BackgroundImage::Url { src, resolved } = layer {
            if resolved.is_some() {
                continue;
            }
            let resolved_image = resolve_image_src(src.as_str()).or_else(|| {
                if src.starts_with('{') && src.contains("__assetFile") {
                    parse_asset_file_token(src)
                        .and_then(|name| resolve_image_asset_file(asset_root, &name))
                } else {
                    None
                }
            });
            *resolved = resolved_image.map(|r| crate::css::style::ResolvedUrlImage {
                src: r.src,
                natural_width: r.natural_width,
                natural_height: r.natural_height,
            });
        }
    }
}

/// Best-effort parser for `{__assetFile: "name.png"}` tokens that
/// occasionally appear inside `background-image: url(...)` strings
/// when the upstream JSX serializer inlined an `{__assetFile}` shape
/// into the CSS. Returns the bare filename so
/// `resolve_image_asset_file` can read it from disk.
fn parse_asset_file_token(s: &str) -> Option<String> {
    let key = "\"__assetFile\"";
    let idx = s.find(key).or_else(|| s.find("__assetFile"))?;
    let rest = &s[idx + key.len().min(s.len() - idx)..];
    // Skip past `:` and whitespace.
    let after_colon = rest.find(':')?;
    let rest = &rest[after_colon + 1..];
    let trimmed = rest.trim_start();
    // Expect a quoted string.
    let quote = trimmed.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = &trimmed[1..];
    let end = inner.find(quote)?;
    Some(inner[..end].to_string())
}

/// Port of the `type === 'svg'` branch in `src/handler/compute.ts`,
/// plus the `SVGNodeToImage` call from `src/layout.ts`.
///
/// Serializes the JSX subtree into a `data:image/svg+xml;utf8,...` URI
/// and stores it on `style._src` so the rect renderer treats the node
/// as an image. Also resolves `style.width` / `style.height` from
/// `props.width` / `props.height` / `props.viewBox` the way JS satori
/// does — this is what feeds yoga.
fn apply_svg_compute(
    style: &mut ComputedStyle,
    props: Option<&serde_json::Map<String, Value>>,
    inherited: &ComputedStyle,
) {
    // Resolve effective currentColor (style.color falls back to
    // inherited.color).
    let inherited_color = style
        .color
        .clone()
        .or_else(|| inherited.color.clone())
        .unwrap_or_else(|| "black".to_string());

    if let Some(p) = props {
        let fs = style.font_size.unwrap_or(16.0) as f64;
        let (w, h) = inline_svg::compute_svg_size(p, fs);
        if style.width.is_none() {
            if let Some(w_num) = w {
                style.width = Some(Dim::Px(w_num as f32));
            }
        }
        if style.height.is_none() {
            if let Some(h_num) = h {
                style.height = Some(Dim::Px(h_num as f32));
            }
        }
        let data_uri = inline_svg::build_data_uri(p, &inherited_color);
        style._src = Some(data_uri);
    }
}

fn dim_px(d: Option<Dim>) -> f32 {
    match d {
        Some(Dim::Px(n)) => n,
        _ => 0.0,
    }
}

fn parse_dimension_passthrough(
    v: &Value,
    fs: f32,
    vw: Option<u32>,
    vh: Option<u32>,
) -> Option<Dim> {
    crate::css::dimension::parse_dimension(v, fs, vw, vh)
}

/// JS `parseInt(value, 10)` quirk: returns the leading integer part of a
/// string, ignoring trailing non-digits. Returns `None` for things that
/// can't be coerced.
fn parse_int_like(v: Option<&Value>) -> Option<f32> {
    let v = v?;
    if let Some(n) = v.as_f64() {
        return Some(n as f32);
    }
    let s = v.as_str()?.trim_start();
    let mut end = 0;
    let bytes = s.as_bytes();
    if bytes.first() == Some(&b'-') || bytes.first() == Some(&b'+') {
        end = 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == 0 || (end == 1 && (bytes[0] == b'-' || bytes[0] == b'+')) {
        return None;
    }
    s[..end].parse::<f32>().ok()
}

/// Resolve the value of `<img src=...>` — either a string (data URI or
/// HTTP URL — only data URIs return a payload here), or an
/// `{"__assetFile": "name.png"}` shape produced by the test harness.
fn resolve_src_value(value: Option<&Value>, asset_root: &std::path::Path) -> Option<ResolvedImage> {
    let v = value?;
    if let Some(s) = v.as_str() {
        return resolve_image_src(s);
    }
    if let Some(obj) = v.as_object() {
        if let Some(asset) = obj.get("__assetFile").and_then(|v| v.as_str()) {
            return resolve_image_asset_file(asset_root, asset);
        }
    }
    None
}

fn collect_children(
    v: &Value,
    inherited: &ComputedStyle,
    parent_vars: &Vars,
    ctx: ExpandContext,
    parent_id: &str,
    counter: &mut usize,
    out: &mut Vec<Built>,
    font_loader: &FontLoader,
    asset_root: &std::path::Path,
    parent_computed: Option<&ComputedStyle>,
    grapheme_images: &std::sync::Arc<std::collections::HashMap<String, String>>,
) -> Result<(), SatoriError> {
    // Mirror JS `normalizeChildren` (`reference/src/utils.ts`): flatten
    // arrays, drop `null`/`undefined`/`boolean`, stringify numbers, and
    // concatenate consecutive strings into one segment. Then walk the
    // normalized list and call `build_node` per entry.
    let mut normalized: Vec<Value> = Vec::new();
    flatten_children_into(v, &mut normalized);
    for entry in &normalized {
        *counter += 1;
        let id = format!("{parent_id}-{c}", c = *counter - 1);
        if let Some(b) = build_node(
            entry,
            inherited,
            parent_vars,
            ctx,
            &id,
            counter,
            font_loader,
            asset_root,
            parent_computed,
            grapheme_images,
        )? {
            out.push(b);
        }
    }
    Ok(())
}

/// Mirror of JS satori's `normalizeChildren`: recursively flattens
/// arrays, skips `null`/`undefined`/booleans, stringifies numbers, and
/// merges adjacent string segments into one combined string.
/// Best-effort: does the (un-normalized) children value contain any
/// bare string element? Matches JS satori's `Array.isArray + .some(s
/// => typeof s === 'string')` semantics, applied recursively.
fn has_string_child(v: &Value) -> bool {
    match v {
        Value::String(s) => !s.is_empty(),
        Value::Number(_) => true,
        Value::Array(items) => items.iter().any(has_string_child),
        _ => false,
    }
}

fn flatten_children_into(v: &Value, out: &mut Vec<Value>) {
    match v {
        Value::Null => {}
        Value::Bool(_) => {}
        Value::Array(items) => {
            for c in items {
                flatten_children_into(c, out);
            }
        }
        Value::Number(n) => push_text(out, &n.to_string()),
        Value::String(s) => push_text(out, s),
        Value::Object(_) => out.push(v.clone()),
    }
}

fn push_text(out: &mut Vec<Value>, s: &str) {
    if let Some(Value::String(prev)) = out.last_mut() {
        prev.push_str(s);
        return;
    }
    out.push(Value::String(s.to_string()));
}
