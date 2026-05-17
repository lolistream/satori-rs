//! JS-style camelCase props → `ComputedStyle`.
//!
//! Port of `src/handler/expand.ts`. The full upstream pass:
//!   1. Calls `preprocess` on raw values (currentColor, calc(), etc).
//!   2. Calls `css-to-react-native` to expand shorthand props
//!      (`padding`, `margin`, `border`, `flex`, `background`...).
//!   3. Calls `presets()` to apply per-tag defaults (`h1`, `p`, …).
//!   4. Calls `inheritable()` to filter out non-inheritable properties.
//!
//! For the initial slice we implement (1) and a minimal hand-rolled
//! version of (2) for the props the test suite touches. We grow this file
//! as we port more tests.

use serde_json::Value;

use super::color::parse_color;
use super::dimension::{parse_dimension, Dim};
use super::gradient::{parse_conic_gradient, parse_linear_gradient, parse_radial_gradient};
use super::style::*;
use super::variables::{extract_vars, merge_vars, substitute, Vars};

#[derive(Debug, Clone, Copy)]
pub struct ExpandContext {
    pub base_font_size: f32,
    pub viewport_width: Option<u32>,
    pub viewport_height: Option<u32>,
}

impl Default for ExpandContext {
    fn default() -> Self {
        Self { base_font_size: 16.0, viewport_width: None, viewport_height: None }
    }
}

/// Expand a raw JS-style `style` object plus an inherited style into a
/// `(ComputedStyle, NewInheritableStyle, Vars)` triple, mirroring the JS
/// shape where the second element is what should be passed to children.
///
/// The third element is the merged CSS-variable scope that should be
/// threaded to children: the parent's incoming vars plus any new
/// `--custom-property` declarations on this node (child overrides parent,
/// per the CSS cascade).
/// Error produced by `expand_style` when a CSS property has an
/// unrecognised value. The message is structurally compatible with
/// the wording JS satori's `v()` helper uses (`Invalid value for CSS
/// property "..."`. Allowed values: ... . Received: ...`).
pub type ExpandError = String;

thread_local! {
    /// Stash for the first validation error from inside the current
    /// `expand_style` invocation. Threaded via TLS rather than as a
    /// `Result` everywhere because the surface is wide and the
    /// failure is always fatal (JS satori throws synchronously and
    /// propagates).
    static EXPAND_VALIDATION_ERROR: std::cell::RefCell<Option<ExpandError>> =
        const { std::cell::RefCell::new(None) };
}

/// Record the first validation error from inside `apply_prop` (or any
/// helper called by it). Subsequent calls in the same `expand_style`
/// invocation are dropped; the first wins.
pub fn record_validation_error(msg: impl Into<String>) {
    EXPAND_VALIDATION_ERROR.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            *slot = Some(msg.into());
        }
    });
}

pub fn expand_style(
    raw: &Value,
    tag: &str,
    inherited: &ComputedStyle,
    parent_vars: &Vars,
    ctx: ExpandContext,
) -> Result<(ComputedStyle, ComputedStyle, Vars), ExpandError> {
    // Defensive reset in case a previous `expand_style` left a stale
    // entry (a panic mid-evaluation would skip the read+clear below).
    EXPAND_VALIDATION_ERROR.with(|c| c.borrow_mut().take());

    // Start from inherited so children inherit font, color, etc. Opacity
    // mirrors JS satori behavior: it's stored on `inheritedStyle` and
    // multiplied through on every expand pass.
    let mut s = ComputedStyle {
        font_size: inherited.font_size,
        _font_size_f64: inherited._font_size_f64,
        font_family: inherited.font_family.clone(),
        font_weight: inherited.font_weight,
        font_style: inherited.font_style,
        color: inherited.color.clone(),
        opacity: inherited.opacity.or(Some(1.0)),
        line_height: inherited.line_height,
        white_space: inherited.white_space,
        text_align: inherited.text_align,
        letter_spacing: inherited.letter_spacing,
        text_indent: inherited.text_indent,
        tab_size: inherited.tab_size,
        word_break: inherited.word_break,
        text_wrap: inherited.text_wrap,
        text_overflow: inherited.text_overflow,
        // `overflow` is NOT in JS satori's inheritable list — each
        // element gets its own. (Listed here only to make this explicit.)
        overflow: None,
        line_clamp: inherited.line_clamp,
        line_clamp_ellipsis: inherited.line_clamp_ellipsis.clone(),
        webkit_line_clamp: inherited.webkit_line_clamp,
        webkit_box_orient: inherited.webkit_box_orient.clone(),
        text_decoration_line: inherited.text_decoration_line,
        text_decoration_color: inherited.text_decoration_color.clone(),
        text_decoration_style: inherited.text_decoration_style,
        text_decoration_skip_ink: inherited.text_decoration_skip_ink.clone(),
        text_transform: inherited.text_transform,
        _webkit_text_fill_color: inherited._webkit_text_fill_color.clone(),
        _webkit_text_stroke_width: inherited._webkit_text_stroke_width,
        _webkit_text_stroke_color: inherited._webkit_text_stroke_color.clone(),
        _viewport_width: inherited._viewport_width.or(ctx.viewport_width),
        _viewport_height: inherited._viewport_height.or(ctx.viewport_height),
        _inherited_clip_path_id: inherited._inherited_clip_path_id.clone(),
        _inherited_mask_id: inherited._inherited_mask_id.clone(),
        _inherited_bg_clip_text_target: inherited._inherited_bg_clip_text_target.clone(),
        _inherited_bg_clip_text_has_background: inherited._inherited_bg_clip_text_has_background,
        ..Default::default()
    };

    apply_tag_preset(tag, &mut s);

    // Pull `--foo` declarations out of the raw style and merge with the
    // scope inherited from the parent. Self-declared vars override the
    // parent's, matching JS satori's `mergeVariables(inherited, current)`.
    let own_vars = extract_vars(raw);
    let merged_vars = merge_vars(parent_vars, &own_vars);

    let Some(obj) = raw.as_object() else {
        return Ok((s.clone(), s, merged_vars));
    };

    // Determine the "current color" for this element so `currentcolor`
    // can be substituted in any other prop's value (matches JS
    // `getCurrentColor` + `convertCurrentColorToActualValue`).
    // Self color (if specified) wins; otherwise fall back to inherited.
    let raw_color = obj
        .get("color")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| inherited.color.clone())
        .unwrap_or_else(|| "black".to_string());
    let current_color = if raw_color.eq_ignore_ascii_case("currentcolor") {
        inherited.color.clone().unwrap_or_else(|| "black".to_string())
    } else {
        raw_color
    };

    // Pre-pass: process `fontSize` first so any em/rem dimensions
    // declared on this element resolve against this element's own
    // font-size (matching JS satori's `calcBaseFontSize` + the
    // `lengthToNumber` post-pass in `handler/expand.ts`).
    if let Some(fs_val) = obj.get("fontSize") {
        apply_prop(&mut s, "fontSize", fs_val, ctx);
    }

    for (key, value) in obj {
        // Custom properties never reach `apply_prop`; they live only in
        // the variable scope.
        if key.starts_with("--") {
            continue;
        }
        if key == "fontSize" {
            // Already handled in the pre-pass.
            continue;
        }
        // Resolve any `var(...)` references in string values using the
        // merged scope before handing the value off to the property
        // dispatcher, then substitute `currentcolor` with the resolved
        // current color (matches JS `preprocess`).
        match value.as_str() {
            Some(raw_str) => {
                let mut s_val = if raw_str.contains("var(") {
                    substitute(raw_str, &merged_vars)
                } else {
                    raw_str.to_string()
                };
                if s_val.to_lowercase().contains("currentcolor") {
                    s_val = convert_current_color(&s_val, &current_color);
                }
                if s_val == raw_str {
                    apply_prop(&mut s, key, value, ctx);
                } else {
                    apply_prop(&mut s, key, &Value::String(s_val), ctx);
                }
            }
            _ => apply_prop(&mut s, key, value, ctx),
        }
    }

    // post-pass: compute font_size effective for em conversions of *this*
    // node's own dimensions if they were set as strings with em units.
    // (The first pass uses inherited font_size which is correct for em
    // relative to parent, matching JS satori behavior.)

    // Compute the clip-path / mask IDs that descendants should inherit.
    // Port of `layout.ts` lines 158-167:
    //   * overflow:hidden OR style.clipPath !== 'none' →
    //       _inheritedClipPathId = `satori_cp-{id}` (set by the layout
    //       step, which knows the per-element id).
    // For now, we propagate by reusing the parent's `_inherited_clip_path_id`
    // — the per-node id assignment is done in the layout pass.

    // Build child-inheritable copy from the computed style. Opacity is
    // forwarded so children multiply on top.
    let child = ComputedStyle {
        font_size: s.font_size,
        _font_size_f64: s._font_size_f64,
        font_family: s.font_family.clone(),
        font_weight: s.font_weight,
        font_style: s.font_style,
        color: s.color.clone(),
        opacity: s.opacity,
        line_height: s.line_height,
        white_space: s.white_space,
        text_align: s.text_align,
        letter_spacing: s.letter_spacing,
        text_indent: s.text_indent,
        tab_size: s.tab_size,
        word_break: s.word_break,
        text_wrap: s.text_wrap,
        text_overflow: s.text_overflow,
        // `overflow` is NOT inheritable per JS satori.
        overflow: None,
        line_clamp: s.line_clamp,
        line_clamp_ellipsis: s.line_clamp_ellipsis.clone(),
        webkit_line_clamp: s.webkit_line_clamp,
        webkit_box_orient: s.webkit_box_orient.clone(),
        text_decoration_line: s.text_decoration_line,
        text_decoration_color: s.text_decoration_color.clone(),
        text_decoration_style: s.text_decoration_style,
        text_decoration_skip_ink: s.text_decoration_skip_ink.clone(),
        text_transform: s.text_transform,
        _webkit_text_fill_color: s._webkit_text_fill_color.clone(),
        _webkit_text_stroke_width: s._webkit_text_stroke_width,
        _webkit_text_stroke_color: s._webkit_text_stroke_color.clone(),
        _viewport_width: s._viewport_width,
        _viewport_height: s._viewport_height,
        _inherited_clip_path_id: s._inherited_clip_path_id.clone(),
        _inherited_mask_id: s._inherited_mask_id.clone(),
        _inherited_bg_clip_text_target: s._inherited_bg_clip_text_target.clone(),
        _inherited_bg_clip_text_has_background: s._inherited_bg_clip_text_has_background,
        ..Default::default()
    };

    if let Some(err) = EXPAND_VALIDATION_ERROR.with(|c| c.borrow_mut().take()) {
        return Err(err);
    }
    Ok((s, child, merged_vars))
}

/// Mirror of JS satori's `parseLineClamp` (`reference/text/processor.ts`).
/// Returns `(line_count, optional_ellipsis_text)` for inputs of the form
/// `"<n>"`, `"<n> \"...\""`, or `"<n> '...'"`. Returns `None` when the
/// input doesn't match any of these shapes; callers may fall back to a
/// raw integer parse.
fn parse_line_clamp_with_ellipsis(s: &str) -> Option<(u32, Option<String>)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == 0 {
        return None;
    }
    let n: u32 = std::str::from_utf8(&bytes[..i]).ok()?.parse().ok()?;
    let rest = s[i..].trim_start();
    if rest.is_empty() {
        return Some((n, None));
    }
    let (open, close) = if rest.starts_with('"') {
        ('"', '"')
    } else if rest.starts_with('\'') {
        ('\'', '\'')
    } else {
        return None;
    };
    let after_open = &rest[open.len_utf8()..];
    let close_idx = after_open.rfind(close)?;
    if !after_open[close_idx + close.len_utf8()..].trim().is_empty() {
        return None;
    }
    let ell = after_open[..close_idx].to_string();
    Some((n, Some(ell)))
}

fn apply_tag_preset(tag: &str, s: &mut ComputedStyle) {
    // Port of `src/handler/presets.ts`. Note: in JS satori, presets are
    // expanded against the inherited fontSize FIRST (so `h1`'s `2em`
    // resolves to `32px` when inherited fontSize is `16px`), and
    // `marginTop: '0.67em'` is then resolved against the preset's *own*
    // computed fontSize (`32px`). User-defined `fontSize: 16` later
    // overrides the preset's fontSize but leaves the margin alone, so
    // an `<h1 fontSize: 16>` ends up at fontSize=16 with margin=21.44.
    let inherited_fs = s.font_size.unwrap_or(16.0);
    // f64 inherited fs for em-resolution precision parity with JS satori
    // (`0.8em * 1.5em` in f64 gives 19.20000000000000284217; doing it in
    // f32 gives 19.2 → widens to 19.200000762939453, a different f64).
    let inherited_fs_f64 = s._font_size_f64.unwrap_or(inherited_fs as f64);
    match tag {
        "p" => {
            s.margin_top = Some(super::dimension::Dim::Px(inherited_fs));
            s.margin_bottom = Some(super::dimension::Dim::Px(inherited_fs));
        }
        "blockquote" => {
            s.margin_top = Some(super::dimension::Dim::Px(inherited_fs));
            s.margin_bottom = Some(super::dimension::Dim::Px(inherited_fs));
            s.margin_left = Some(super::dimension::Dim::Px(40.0));
            s.margin_right = Some(super::dimension::Dim::Px(40.0));
        }
        "center" => {
            s.text_align = Some(TextAlign::Center);
        }
        "h1" => {
            let fs = 2.0 * inherited_fs;
            s.font_size = Some(fs);
            s._font_size_f64 = Some(2.0 * inherited_fs_f64);
            s.font_weight = Some(700);
            s.margin_top = Some(super::dimension::Dim::Px(0.67 * fs));
            s.margin_bottom = Some(super::dimension::Dim::Px(0.67 * fs));
            s.margin_left = Some(super::dimension::Dim::Px(0.0));
            s.margin_right = Some(super::dimension::Dim::Px(0.0));
        }
        "h2" => {
            let fs = 1.5 * inherited_fs;
            s.font_size = Some(fs);
            s._font_size_f64 = Some(1.5 * inherited_fs_f64);
            s.font_weight = Some(700);
            s.margin_top = Some(super::dimension::Dim::Px(0.83 * fs));
            s.margin_bottom = Some(super::dimension::Dim::Px(0.83 * fs));
            s.margin_left = Some(super::dimension::Dim::Px(0.0));
            s.margin_right = Some(super::dimension::Dim::Px(0.0));
        }
        "h3" => {
            let fs = 1.17 * inherited_fs;
            s.font_size = Some(fs);
            s._font_size_f64 = Some(1.17 * inherited_fs_f64);
            s.font_weight = Some(700);
            s.margin_top = Some(super::dimension::Dim::Px(fs));
            s.margin_bottom = Some(super::dimension::Dim::Px(fs));
            s.margin_left = Some(super::dimension::Dim::Px(0.0));
            s.margin_right = Some(super::dimension::Dim::Px(0.0));
        }
        "h4" => {
            let fs = inherited_fs;
            s.font_weight = Some(700);
            s.margin_top = Some(super::dimension::Dim::Px(1.33 * fs));
            s.margin_bottom = Some(super::dimension::Dim::Px(1.33 * fs));
            s.margin_left = Some(super::dimension::Dim::Px(0.0));
            s.margin_right = Some(super::dimension::Dim::Px(0.0));
        }
        "h5" => {
            let fs = 0.83 * inherited_fs;
            s.font_size = Some(fs);
            s._font_size_f64 = Some(0.83 * inherited_fs_f64);
            s.font_weight = Some(700);
            s.margin_top = Some(super::dimension::Dim::Px(1.67 * fs));
            s.margin_bottom = Some(super::dimension::Dim::Px(1.67 * fs));
            s.margin_left = Some(super::dimension::Dim::Px(0.0));
            s.margin_right = Some(super::dimension::Dim::Px(0.0));
        }
        "h6" => {
            let fs = 0.67 * inherited_fs;
            s.font_size = Some(fs);
            s._font_size_f64 = Some(0.67 * inherited_fs_f64);
            s.font_weight = Some(700);
            s.margin_top = Some(super::dimension::Dim::Px(2.33 * fs));
            s.margin_bottom = Some(super::dimension::Dim::Px(2.33 * fs));
            s.margin_left = Some(super::dimension::Dim::Px(0.0));
            s.margin_right = Some(super::dimension::Dim::Px(0.0));
        }
        "u" => {
            s.text_decoration_line = Some(TextDecorationLine::Underline);
        }
        "b" | "strong" => {
            s.font_weight = Some(700);
        }
        "i" | "em" => {
            s.font_style = Some(FontStyle::Italic);
        }
        "s" => {
            s.text_decoration_line = Some(TextDecorationLine::LineThrough);
        }
        "code" | "kbd" => {
            s.font_family = Some("monospace".to_string());
        }
        "pre" => {
            s.font_family = Some("monospace".to_string());
            s.white_space = Some(WhiteSpace::Pre);
            s.margin_top = Some(super::dimension::Dim::Px(inherited_fs));
            s.margin_bottom = Some(super::dimension::Dim::Px(inherited_fs));
        }
        "mark" => {
            s.background_color = Some("yellow".to_string());
            s.color = Some("black".to_string());
        }
        "big" => {
            s.font_size = Some(1.2 * inherited_fs);
            s._font_size_f64 = Some(1.2 * inherited_fs_f64);
        }
        "small" => {
            s.font_size = Some(0.8333 * inherited_fs);
            s._font_size_f64 = Some(0.8333 * inherited_fs_f64);
        }
        _ => {}
    }
}

fn apply_prop(s: &mut ComputedStyle, key: &str, v: &Value, ctx: ExpandContext) {
    let fs = s.font_size.unwrap_or(16.0);
    match key {
        // Display / position
        "display" => {
            // JS satori's `v()` lookup throws on unrecognised values
            // (`Invalid value for CSS property "display"`).
            if let Some(raw_str) = v.as_str() {
                let parsed = parse_display(raw_str.trim());
                if parsed.is_some() {
                    s.display = parsed;
                } else {
                    record_validation_error(format!(
                        "Invalid value for CSS property \"display\". Allowed values: \
                         \"flex\" | \"block\" | \"contents\" | \"none\" | \
                         \"-webkit-box\". Received: \"{}\".",
                        raw_str.trim()
                    ));
                }
            }
        }
        "position" => {
            if let Some(raw_str) = v.as_str() {
                let parsed = parse_position(raw_str.trim());
                if parsed.is_some() {
                    s.position = parsed;
                } else {
                    record_validation_error(format!(
                        "Invalid value for CSS property \"position\". Allowed values: \
                         \"absolute\" | \"relative\" | \"static\". Received: \"{}\".",
                        raw_str.trim()
                    ));
                }
            }
        }

        // Size
        "width" => s.width = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "height" => s.height = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "minWidth" => s.min_width = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "minHeight" => s.min_height = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "maxWidth" => s.max_width = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "maxHeight" => s.max_height = parse_dimension(v, fs, s._viewport_width, s._viewport_height),

        // Position offsets
        "top" => s.top = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "right" => s.right = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "bottom" => s.bottom = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "left" => s.left = parse_dimension(v, fs, s._viewport_width, s._viewport_height),

        // Margin shorthand + longhand
        "margin" => apply_box_shorthand(v, fs, ctx, |i, d| {
            match i {
                0 => s.margin_top = Some(d),
                1 => s.margin_right = Some(d),
                2 => s.margin_bottom = Some(d),
                3 => s.margin_left = Some(d),
                _ => {}
            }
        }),
        "marginTop" => s.margin_top = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "marginRight" => s.margin_right = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "marginBottom" => s.margin_bottom = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "marginLeft" => s.margin_left = parse_dimension(v, fs, s._viewport_width, s._viewport_height),

        // Padding
        "padding" => apply_box_shorthand(v, fs, ctx, |i, d| {
            match i {
                0 => s.padding_top = Some(d),
                1 => s.padding_right = Some(d),
                2 => s.padding_bottom = Some(d),
                3 => s.padding_left = Some(d),
                _ => {}
            }
        }),
        "paddingTop" => s.padding_top = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "paddingRight" => s.padding_right = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "paddingBottom" => s.padding_bottom = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "paddingLeft" => s.padding_left = parse_dimension(v, fs, s._viewport_width, s._viewport_height),

        // Border
        "borderWidth" => {
            let n = px_number(v, fs);
            s.border_top_width = Some(n);
            s.border_right_width = Some(n);
            s.border_bottom_width = Some(n);
            s.border_left_width = Some(n);
        }
        "borderTopWidth" => s.border_top_width = Some(px_number(v, fs)),
        "borderRightWidth" => s.border_right_width = Some(px_number(v, fs)),
        "borderBottomWidth" => s.border_bottom_width = Some(px_number(v, fs)),
        "borderLeftWidth" => s.border_left_width = Some(px_number(v, fs)),
        "borderColor" => apply_border_color_all(s, v),
        "borderTopColor" => s.border_top_color = parse_color_str(v),
        "borderRightColor" => s.border_right_color = parse_color_str(v),
        "borderBottomColor" => s.border_bottom_color = parse_color_str(v),
        "borderLeftColor" => s.border_left_color = parse_color_str(v),
        "borderStyle" => apply_border_style_all(s, v),
        "borderTopStyle" => s.border_top_style = parse_border_style(v),
        "borderRightStyle" => s.border_right_style = parse_border_style(v),
        "borderBottomStyle" => s.border_bottom_style = parse_border_style(v),
        "borderLeftStyle" => s.border_left_style = parse_border_style(v),

        // Border shorthand: "border: 1px solid red", "border: 1px", "border: 1px solid"
        "border" => apply_border_shorthand(s, v, fs),
        "borderTop" => apply_border_side_shorthand(s, v, fs, 0),
        "borderRight" => apply_border_side_shorthand(s, v, fs, 1),
        "borderBottom" => apply_border_side_shorthand(s, v, fs, 2),
        "borderLeft" => apply_border_side_shorthand(s, v, fs, 3),

        // Border radius
        "borderRadius" => {
            apply_border_radius_shorthand(s, v, fs, ctx);
        }
        "borderTopLeftRadius" => s.border_top_left_radius = parse_radius_value(v, fs, ctx),
        "borderTopRightRadius" => s.border_top_right_radius = parse_radius_value(v, fs, ctx),
        "borderBottomLeftRadius" => s.border_bottom_left_radius = parse_radius_value(v, fs, ctx),
        "borderBottomRightRadius" => s.border_bottom_right_radius = parse_radius_value(v, fs, ctx),

        // Flexbox
        "flexDirection" => s.flex_direction = v.as_str().and_then(parse_flex_direction),
        "flexGrow" => s.flex_grow = v.as_f64().map(|x| x as f32),
        "flexShrink" => s.flex_shrink = v.as_f64().map(|x| x as f32),
        "flexBasis" => s.flex_basis = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "flex" => apply_flex_shorthand(s, v, fs),
        "boxSizing" => {
            s.box_sizing = match v.as_str() {
                Some("content-box") => Some(super::style::BoxSizing::ContentBox),
                Some("border-box") => Some(super::style::BoxSizing::BorderBox),
                _ => s.box_sizing,
            };
        }
        "flexWrap" => s.flex_wrap = Some(v.as_str() == Some("wrap")),
        "justifyContent" => s.justify_content = v.as_str().and_then(parse_justify),
        "alignItems" => s.align_items = v.as_str().and_then(parse_align_items),
        "alignSelf" => s.align_self = v.as_str().and_then(parse_align_self),
        "alignContent" => s.align_content = v.as_str().and_then(parse_align_content),
        "clipPath" => {
            // Stash the raw string; the renderer resolves it against the
            // element's box (width, height, fontSize) at render-time
            // via `ClipPathShape::parse`. `none` short-circuits.
            if let Some(raw) = v.as_str() {
                let raw = raw.trim();
                if !raw.is_empty() && raw != "none" {
                    s.clip_path = Some(raw.to_string());
                }
            }
        }
        "overflow" => {
            if let Some(raw) = v.as_str() {
                use super::style::Overflow;
                s.overflow = match raw {
                    "visible" => Some(Overflow::Visible),
                    "hidden" => Some(Overflow::Hidden),
                    "scroll" => Some(Overflow::Scroll),
                    "auto" => Some(Overflow::Auto),
                    _ => s.overflow,
                };
            }
        }
        "gap" => s.gap = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "rowGap" => s.row_gap = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "columnGap" => s.column_gap = parse_dimension(v, fs, s._viewport_width, s._viewport_height),

        // Visual — preserve the original CSS color token verbatim
        // (`red` stays `red`, not `#ff0000`) to match JS satori byte-for-byte,
        // except for colors with alpha != 1 which JS re-emits via
        // `normalizeColor` as `rgba(r, g, b, alpha)`.
        "backgroundColor" => {
            if let Some(s_val) = v.as_str() {
                s.background_color = Some(normalize_color_token(s_val));
            } else if let Some(n) = v.as_f64() {
                s.background_color = Some(n.to_string());
            }
        }
        // The `background` shorthand may contain a plain color, a list of
        // gradient layers, or a `<gradient>, <color>` combo. Split on
        // top-level commas, parse each layer as a gradient first, and
        // fall back to color for whatever remains.
        "background" => apply_background_shorthand(s, v),
        "backgroundImage" => apply_background_image(s, v),
        "backgroundSize" => s.background_size = v.as_str().map(|x| x.to_string()),
        "backgroundPosition" => s.background_position = v.as_str().map(|x| x.to_string()),
        "backgroundRepeat" => s.background_repeat = v.as_str().map(|x| x.to_string()),
        // `backgroundClip` + `-webkit-background-clip` (we accept the
        // CSS-cased alias too, but our JSX input is always camelCase).
        // Stored as-is; only `"text"` triggers special handling
        // downstream. Note: assignment order matters when both are
        // set — the later one wins, but JS satori's preprocess
        // applies them in iteration order, so this is fine.
        "backgroundClip" | "WebkitBackgroundClip" => {
            s.background_clip = v.as_str().map(|x| x.to_string());
        }
        "maskImage" | "WebkitMaskImage" => {
            apply_mask_image(s, v);
        }
        "maskSize" | "WebkitMaskSize" => {
            s.mask_size = v.as_str().map(|x| x.to_string());
        }
        "maskPosition" | "WebkitMaskPosition" => {
            s.mask_position = v.as_str().map(|x| x.to_string());
        }
        "maskRepeat" | "WebkitMaskRepeat" => {
            s.mask_repeat = v.as_str().map(|x| x.to_string());
        }
        "color" => {
            if let Some(s_val) = v.as_str() {
                // Preserve the original token (`color: red` stays `red`)
                // except when alpha != 1, in which case JS `normalizeColor`
                // re-emits as `rgba(...)`.
                s.color = Some(normalize_color_token(s_val));
            }
        }
        "opacity" => {
            // JS: serializedStyle.opacity = value * inheritedStyle.opacity
            let inherited = s.opacity.unwrap_or(1.0);
            s.opacity = v.as_f64().map(|x| (x as f32) * inherited);
        }

        // Text
        "fontSize" => {
            let fs_new = px_number(v, fs);
            s.font_size = Some(fs_new);
            // Mirror JS satori: when the input is an em/rem string, the
            // f64 cascade is `multiplier * inherited_f64`; for a bare
            // px/number it's just the literal. Compute the f64-precise
            // version here so downstream `run_path_d` matches JS's
            // `(1 / unitsPerEm) * fontSize`.
            s._font_size_f64 = Some(px_number_f64(v, s._font_size_f64.unwrap_or(fs as f64)));
        }
        "fontFamily" => s.font_family = v.as_str().map(|s| s.to_string()),
        "fontWeight" => {
            s.font_weight = v.as_u64().map(|x| x as u16).or_else(|| {
                v.as_str().and_then(|s| match s {
                    "normal" => Some(400),
                    "bold" => Some(700),
                    _ => s.parse().ok(),
                })
            });
        }
        "fontStyle" => {
            s.font_style = match v.as_str() {
                Some("italic") => Some(FontStyle::Italic),
                Some("normal") => Some(FontStyle::Normal),
                _ => s.font_style,
            };
        }
        "lineHeight" => {
            // `normal` keyword maps to `None`, which the renderer treats
            // as "compute from the font's hhea metrics" (matches the JS
            // satori `'normal' === lineHeight` branch).
            if v.as_str() == Some("normal") {
                s.line_height = None;
            } else {
                s.line_height = v.as_f64().map(|x| x as f32).or_else(|| {
                    v.as_str().and_then(|str| {
                        if let Some(p) = str.strip_suffix("px") {
                            p.parse::<f32>().ok().map(|n| n / fs)
                        } else {
                            str.parse().ok()
                        }
                    })
                });
            }
        }
        "textAlign" => {
            s.text_align = match v.as_str() {
                Some("left") => Some(TextAlign::Left),
                Some("right") => Some(TextAlign::Right),
                Some("center") => Some(TextAlign::Center),
                Some("justify") => Some(TextAlign::Justify),
                Some("start") => Some(TextAlign::Start),
                Some("end") => Some(TextAlign::End),
                _ => s.text_align,
            };
        }
        "transform" => {
            // JS satori only accepts strings; numeric/null inputs throw
            // a contextualised "Invalid `transform` value". A
            // non-empty string that parses to zero transform functions
            // is also a parse failure ("Failed to parse style property
            // \"transform\" ... Only absolute lengths such as `10px`
            // are supported.").
            if v.is_null() || v.as_object().is_some() || v.as_array().is_some() || v.is_number() {
                record_validation_error(format!(
                    "Invalid `transform` value: expected a string, got {v}"
                ));
            } else if let Some(raw) = v.as_str() {
                let parsed = parse_transform_value(v, fs);
                if parsed.as_ref().is_none_or(|ts| ts.is_empty()) && !raw.trim().is_empty() {
                    record_validation_error(format!(
                        "Failed to parse style property \"transform\" with value \"{raw}\". Only absolute lengths such as `10px` are supported."
                    ));
                } else {
                    s.transform = parsed;
                }
            } else {
                s.transform = parse_transform_value(v, fs);
            }
        }
        "transformOrigin" => s.transform_origin = parse_transform_origin_value(v, fs),

        "boxShadow" => {
            // JS satori throws "Invalid `boxShadow` value" on the
            // empty-string case or any input the parser can't
            // tokenise into at least one shadow tuple.
            if let Some(raw) = v.as_str() {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    record_validation_error(
                        "Invalid `boxShadow` value: empty string.",
                    );
                } else {
                    let parsed = parse_box_shadow_value(v, fs);
                    if parsed.is_none() {
                        record_validation_error(format!(
                            "Invalid `boxShadow` value: \"{raw}\""
                        ));
                    } else {
                        s.box_shadow = parsed;
                    }
                }
            } else {
                s.box_shadow = parse_box_shadow_value(v, fs);
            }
        }
        "textShadow" => s.text_shadow = parse_text_shadow_value(v, fs),

        "letterSpacing" => s.letter_spacing = Some(px_number(v, fs)),
        "textIndent" => s.text_indent = parse_dimension(v, fs, s._viewport_width, s._viewport_height),
        "tabSize" => {
            // JS handler/expand.ts converts every string-typed prop to a
            // number via `lengthToNumber` (em/rem/px → px). After that
            // post-pass `tabSize` is always a number. Then text/index.ts
            // does `isString(tabSize) ? lengthToNumber(...) : measureGrapheme(Space) * tabSize`
            // — so a number-valued `tabSize` is interpreted as a
            // MULTIPLIER of space-width regardless of how it got there.
            // We must mirror that quirk: parse `"2em"` to `36` (px-resolved)
            // and store it as a multiplier, not as an absolute pixel
            // length.
            if let Some(n) = v.as_f64() {
                s.tab_size = Some(n as f32);
            } else if let Some(str_v) = v.as_str() {
                let trimmed = str_v.trim();
                let parsed = if let Some(stripped) = trimmed.strip_suffix("rem") {
                    stripped.trim().parse::<f32>().ok().map(|n| n * 16.0)
                } else if let Some(stripped) = trimmed.strip_suffix("em") {
                    stripped.trim().parse::<f32>().ok().map(|n| n * fs)
                } else if let Some(stripped) = trimmed.strip_suffix("px") {
                    stripped.trim().parse::<f32>().ok()
                } else {
                    trimmed.parse::<f32>().ok()
                };
                if let Some(n) = parsed {
                    s.tab_size = Some(n);
                }
            }
        }
        "wordBreak" => {
            s.word_break = match v.as_str() {
                Some("normal") => Some(WordBreak::Normal),
                Some("break-all") => Some(WordBreak::BreakAll),
                Some("keep-all") => Some(WordBreak::KeepAll),
                Some("break-word") => Some(WordBreak::BreakWord),
                _ => s.word_break,
            };
        }
        "textWrap" => {
            s.text_wrap = match v.as_str() {
                Some("wrap") => Some(TextWrap::Wrap),
                Some("nowrap") => Some(TextWrap::Nowrap),
                Some("balance") => Some(TextWrap::Balance),
                Some("pretty") => Some(TextWrap::Pretty),
                _ => s.text_wrap,
            };
        }
        "textOverflow" => {
            s.text_overflow = match v.as_str() {
                Some("clip") => Some(TextOverflow::Clip),
                Some("ellipsis") => Some(TextOverflow::Ellipsis),
                _ => s.text_overflow,
            };
        }
        "lineClamp" => {
            // Mirror JS `parseLineClamp`: `<n>` or `<n> "<ellipsis>"`
            // / `<n> '<ellipsis>'`.
            if let Some(n) = v.as_u64() {
                s.line_clamp = Some(n as u32);
                s.line_clamp_ellipsis = None;
            } else if let Some(str_v) = v.as_str() {
                let trimmed = str_v.trim();
                if let Some((n, ell)) = parse_line_clamp_with_ellipsis(trimmed) {
                    s.line_clamp = Some(n);
                    s.line_clamp_ellipsis = ell;
                } else {
                    s.line_clamp = trimmed.parse().ok();
                    s.line_clamp_ellipsis = None;
                }
            }
        }
        "WebkitLineClamp" => {
            if let Some(n) = v.as_u64() {
                s.webkit_line_clamp = Some(n as u32);
            } else if let Some(str_v) = v.as_str() {
                s.webkit_line_clamp = str_v.parse().ok();
            }
        }
        "WebkitBoxOrient" => {
            s.webkit_box_orient = v.as_str().map(|s| s.to_string());
        }
        "textDecorationLine" | "textDecoration" => {
            // `text-decoration` can be `<line> <style> <color>` (or just any single token).
            if let Some(str_v) = v.as_str() {
                for token in str_v.split_whitespace() {
                    match token {
                        "none" => s.text_decoration_line = Some(TextDecorationLine::None),
                        "underline" => s.text_decoration_line = Some(TextDecorationLine::Underline),
                        "line-through" => s.text_decoration_line = Some(TextDecorationLine::LineThrough),
                        "solid" => s.text_decoration_style = Some(TextDecorationStyle::Solid),
                        "dashed" => s.text_decoration_style = Some(TextDecorationStyle::Dashed),
                        "dotted" => s.text_decoration_style = Some(TextDecorationStyle::Dotted),
                        "double" => s.text_decoration_style = Some(TextDecorationStyle::Double),
                        other => {
                            // Treat as color if parseable
                            if parse_color(other).is_some() {
                                s.text_decoration_color = Some(other.to_string());
                            }
                        }
                    }
                }
            }
        }
        "textDecorationColor" => {
            s.text_decoration_color = v.as_str().map(|s| s.to_string());
        }
        "textDecorationStyle" => {
            s.text_decoration_style = match v.as_str() {
                Some("solid") => Some(TextDecorationStyle::Solid),
                Some("dashed") => Some(TextDecorationStyle::Dashed),
                Some("dotted") => Some(TextDecorationStyle::Dotted),
                Some("double") => Some(TextDecorationStyle::Double),
                _ => s.text_decoration_style,
            };
        }
        "textDecorationSkipInk" => {
            // CSS spec values: `auto | none` (satori only consults
            // these two). Anything else throws.
            if let Some(raw) = v.as_str() {
                if matches!(raw, "auto" | "none") {
                    s.text_decoration_skip_ink = Some(raw.to_string());
                } else {
                    record_validation_error(format!(
                        "Invalid `textDecorationSkipInk` value: \"{raw}\". Expected `auto` or `none`."
                    ));
                }
            }
        }
        "textTransform" => {
            s.text_transform = match v.as_str() {
                Some("none") => Some(TextTransform::None),
                Some("uppercase") => Some(TextTransform::Uppercase),
                Some("lowercase") => Some(TextTransform::Lowercase),
                Some("capitalize") => Some(TextTransform::Capitalize),
                _ => s.text_transform,
            };
        }
        "WebkitTextFillColor" => {
            s._webkit_text_fill_color = v.as_str().map(|s| s.to_string());
        }
        "filter" => {
            // Raw CSS `filter` value (e.g. `blur(1px)`). We pass it
            // through verbatim into SVG element style attributes.
            s.filter = v.as_str().map(|s| s.to_string()).filter(|s| s != "none");
        }
        "WebkitTextStrokeWidth" => {
            s._webkit_text_stroke_width = Some(px_number(v, fs));
        }
        "WebkitTextStrokeColor" => {
            s._webkit_text_stroke_color = v.as_str().map(|s| s.to_string());
        }
        "WebkitTextStroke" => {
            // Shorthand: `<width> <color>` — JS satori throws on the
            // single-token / extra-token / non-string cases.
            if let Some(raw) = v.as_str() {
                let mut parts = raw.split_whitespace();
                let parsed = match (parts.next(), parts.next(), parts.next()) {
                    (Some(w), Some(c), None) => Some((w.to_string(), c.to_string())),
                    _ => None,
                };
                if let Some((w, c)) = parsed {
                    s._webkit_text_stroke_width =
                        Some(px_number(&Value::String(w), fs));
                    s._webkit_text_stroke_color = Some(c);
                } else {
                    record_validation_error(format!(
                        "Invalid `WebkitTextStroke` value: \"{raw}\". Expected `<width> <color>`."
                    ));
                }
            } else {
                record_validation_error(
                    "Invalid `WebkitTextStroke` value: expected a string `<width> <color>`.",
                );
            }
        }
        "whiteSpace" => {
            s.white_space = match v.as_str() {
                Some("normal") => Some(WhiteSpace::Normal),
                Some("nowrap") => Some(WhiteSpace::NoWrap),
                Some("pre") => Some(WhiteSpace::Pre),
                Some("pre-wrap") => Some(WhiteSpace::PreWrap),
                Some("pre-line") => Some(WhiteSpace::PreLine),
                Some("break-spaces") => Some(WhiteSpace::BreakSpaces),
                _ => s.white_space,
            };
        }

        "objectFit" => {
            s.object_fit = match v.as_str() {
                Some("fill") => Some(ObjectFit::Fill),
                Some("contain") => Some(ObjectFit::Contain),
                Some("cover") => Some(ObjectFit::Cover),
                Some("scale-down") => Some(ObjectFit::ScaleDown),
                Some("none") => Some(ObjectFit::None),
                _ => s.object_fit,
            };
        }
        "objectPosition" => {
            s.object_position = v.as_str().map(|s| s.to_string());
        }

        // unknown / unimplemented prop — silently ignored for now.
        _ => {}
    }
}

fn px_number(v: &Value, fs: f32) -> f32 {
    if let Some(n) = v.as_f64() {
        return n as f32;
    }
    if let Some(s) = v.as_str() {
        let s = s.trim();
        if let Some(stripped) = s.strip_suffix("px") {
            return stripped.trim().parse().unwrap_or(0.0);
        }
        if let Some(stripped) = s.strip_suffix("em") {
            let rem = stripped.strip_suffix('r');
            let v: f32 = match rem {
                Some(num) => num.trim().parse().unwrap_or(0.0),
                None => stripped.trim().parse().unwrap_or(0.0),
            };
            return v * if rem.is_some() { 16.0 } else { fs };
        }
        return s.parse().unwrap_or(0.0);
    }
    0.0
}

/// f64 mirror of `px_number` for the `font-size` resolution chain.
/// Same algorithm, but every multiplication is in f64 so the result
/// keeps JS satori's bit-exact `0.8 * 16 * 1.5 = 19.20000000000000284217`
/// precision instead of f32-widening to `19.200000762939453`.
fn px_number_f64(v: &Value, fs: f64) -> f64 {
    if let Some(n) = v.as_f64() {
        return n;
    }
    if let Some(s) = v.as_str() {
        let s = s.trim();
        if let Some(stripped) = s.strip_suffix("px") {
            return stripped.trim().parse().unwrap_or(0.0);
        }
        if let Some(stripped) = s.strip_suffix("em") {
            let rem = stripped.strip_suffix('r');
            let v: f64 = match rem {
                Some(num) => num.trim().parse().unwrap_or(0.0),
                None => stripped.trim().parse().unwrap_or(0.0),
            };
            return v * if rem.is_some() { 16.0 } else { fs };
        }
        return s.parse().unwrap_or(0.0);
    }
    0.0
}

fn apply_box_shorthand(v: &Value, fs: f32, ctx: ExpandContext, mut set: impl FnMut(usize, Dim)) {
    // JS satori treats shorthand strings like "10px 20px"; numbers are
    // applied uniformly to all sides.
    if let Some(n) = v.as_f64() {
        let d = Dim::Px(n as f32);
        for i in 0..4 {
            set(i, d);
        }
        return;
    }
    if let Some(s_val) = v.as_str() {
        let parts: Vec<Dim> = s_val
            .split_whitespace()
            .filter_map(|p| parse_dimension(&Value::String(p.into()), fs, ctx.viewport_width, ctx.viewport_height))
            .collect();
        match parts.len() {
            1 => { for i in 0..4 { set(i, parts[0]); } }
            2 => {
                set(0, parts[0]); set(2, parts[0]);
                set(1, parts[1]); set(3, parts[1]);
            }
            3 => {
                set(0, parts[0]);
                set(1, parts[1]); set(3, parts[1]);
                set(2, parts[2]);
            }
            4 => { for (i, d) in parts.into_iter().enumerate() { set(i, d); } }
            _ => {}
        }
    }
}

fn parse_display(s: &str) -> Option<Display> {
    Some(match s {
        "flex" => Display::Flex,
        "block" => Display::Block,
        "none" => Display::None,
        "contents" => Display::Contents,
        "-webkit-box" => Display::WebkitBox,
        _ => return None,
    })
}

fn parse_position(s: &str) -> Option<Position> {
    Some(match s {
        "static" => Position::Static,
        "relative" => Position::Relative,
        "absolute" => Position::Absolute,
        _ => return None,
    })
}

fn parse_flex_direction(s: &str) -> Option<FlexDirection> {
    Some(match s {
        "row" => FlexDirection::Row,
        "column" => FlexDirection::Column,
        "row-reverse" => FlexDirection::RowReverse,
        "column-reverse" => FlexDirection::ColumnReverse,
        _ => return None,
    })
}

fn parse_justify(s: &str) -> Option<JustifyContent> {
    Some(match s {
        "flex-start" => JustifyContent::FlexStart,
        "flex-end" => JustifyContent::FlexEnd,
        "center" => JustifyContent::Center,
        "space-between" => JustifyContent::SpaceBetween,
        "space-around" => JustifyContent::SpaceAround,
        "space-evenly" => JustifyContent::SpaceEvenly,
        _ => return None,
    })
}

fn parse_align_items(s: &str) -> Option<AlignItems> {
    Some(match s {
        "auto" => AlignItems::Auto,
        "stretch" => AlignItems::Stretch,
        "flex-start" => AlignItems::FlexStart,
        "flex-end" => AlignItems::FlexEnd,
        "center" => AlignItems::Center,
        "baseline" => AlignItems::Baseline,
        _ => return None,
    })
}

fn parse_color_str(v: &Value) -> Option<String> {
    // Mirror JS satori: preserve the original CSS color token verbatim
    // (so `red` stays `red`, not `#ff0000`). For colors with alpha != 1
    // we still re-emit as `rgba(...)` to match `normalizeColor`.
    let s = v.as_str()?;
    Some(normalize_color_token(s))
}

fn apply_border_color_all(s: &mut ComputedStyle, v: &Value) {
    let c = parse_color_str(v);
    s.border_top_color = c.clone();
    s.border_right_color = c.clone();
    s.border_bottom_color = c.clone();
    s.border_left_color = c;
}

fn parse_border_style(v: &Value) -> Option<BorderStyle> {
    Some(match v.as_str()? {
        "none" => BorderStyle::None,
        "solid" => BorderStyle::Solid,
        "dashed" => BorderStyle::Dashed,
        "dotted" => BorderStyle::Dotted,
        "double" => BorderStyle::Double,
        "hidden" => BorderStyle::Hidden,
        _ => return None,
    })
}

fn apply_border_style_all(s: &mut ComputedStyle, v: &Value) {
    let st = parse_border_style(v);
    s.border_top_style = st;
    s.border_right_style = st;
    s.border_bottom_style = st;
    s.border_left_style = st;
}

/// Parse the `border` shorthand: `<width> <style> <color>` in any order.
fn apply_border_shorthand(s: &mut ComputedStyle, v: &Value, fs: f32) {
    let (w, style, color) = split_border_shorthand(v, fs);
    if let Some(w) = w {
        s.border_top_width = Some(w);
        s.border_right_width = Some(w);
        s.border_bottom_width = Some(w);
        s.border_left_width = Some(w);
    }
    if let Some(st) = style {
        s.border_top_style = Some(st);
        s.border_right_style = Some(st);
        s.border_bottom_style = Some(st);
        s.border_left_style = Some(st);
    }
    if let Some(c) = color {
        s.border_top_color = Some(c.clone());
        s.border_right_color = Some(c.clone());
        s.border_bottom_color = Some(c.clone());
        s.border_left_color = Some(c);
    }
}

fn apply_border_side_shorthand(s: &mut ComputedStyle, v: &Value, fs: f32, side: u8) {
    let (w, style, color) = split_border_shorthand(v, fs);
    match side {
        0 => {
            if let Some(w) = w { s.border_top_width = Some(w); }
            if let Some(st) = style { s.border_top_style = Some(st); }
            if let Some(c) = color { s.border_top_color = Some(c); }
        }
        1 => {
            if let Some(w) = w { s.border_right_width = Some(w); }
            if let Some(st) = style { s.border_right_style = Some(st); }
            if let Some(c) = color { s.border_right_color = Some(c); }
        }
        2 => {
            if let Some(w) = w { s.border_bottom_width = Some(w); }
            if let Some(st) = style { s.border_bottom_style = Some(st); }
            if let Some(c) = color { s.border_bottom_color = Some(c); }
        }
        3 => {
            if let Some(w) = w { s.border_left_width = Some(w); }
            if let Some(st) = style { s.border_left_style = Some(st); }
            if let Some(c) = color { s.border_left_color = Some(c); }
        }
        _ => {}
    }
}

fn split_border_shorthand(v: &Value, fs: f32) -> (Option<f32>, Option<BorderStyle>, Option<String>) {
    let Some(raw) = v.as_str() else { return (None, None, None); };
    let mut width = None;
    let mut style: Option<BorderStyle> = None;
    let mut color: Option<String> = None;
    for tok in tokenize_paren_aware(raw) {
        let tok = tok.as_str();
        // Try width: numeric value with optional px/em.
        if let Some(n) = tok.strip_suffix("px") {
            if let Ok(n) = n.trim().parse::<f32>() { width = Some(n); continue; }
        }
        if let Ok(n) = tok.parse::<f32>() { width = Some(n); continue; }
        if let Some(n) = tok.strip_suffix("em") {
            if let Ok(n) = n.parse::<f32>() { width = Some(n * fs); continue; }
        }
        match tok {
            "solid" => { style = Some(BorderStyle::Solid); continue; }
            "dashed" => { style = Some(BorderStyle::Dashed); continue; }
            "dotted" => { style = Some(BorderStyle::Dotted); continue; }
            "double" => { style = Some(BorderStyle::Double); continue; }
            "none" => { style = Some(BorderStyle::None); continue; }
            "hidden" => { style = Some(BorderStyle::Hidden); continue; }
            _ => {}
        }
        // Color — preserve the original CSS token (JS satori does the
        // same; `normalizeColor` only kicks in for alpha != 1).
        color = Some(normalize_color_token(tok));
    }
    (width, style, color)
}

/// Split a CSS value on whitespace, treating any run of characters
/// inside balanced parentheses as a single atomic token. Used by the
/// `border` shorthand so that values like `rgba(0, 0, 0, 0.5)` stay
/// intact instead of getting chopped on the internal commas.
fn tokenize_paren_aware(raw: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    for c in raw.chars() {
        if c == '(' {
            depth += 1;
            cur.push(c);
        } else if c == ')' {
            depth -= 1;
            cur.push(c);
        } else if c.is_whitespace() && depth == 0 {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        } else {
            cur.push(c);
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Expand the `border-radius` shorthand into the 4 corner properties.
///
/// `<h-radii> [ / <v-radii> ]?` where each side accepts 1–4 values in
/// the `TL [TR [BR [BL]]]` order. Per-corner, JS satori's `resolveRadius`
/// decides "single value" iff the corner string was one token and not a
/// percentage — we replicate that by splatting tokens into 4 single-token
/// corner strings, then parsing each.
fn apply_border_radius_shorthand(s: &mut ComputedStyle, v: &Value, fs: f32, ctx: ExpandContext) {
    let (h_parts, v_parts): (Vec<RadiusLen>, Option<Vec<RadiusLen>>) = if let Some(n) = v.as_f64() {
        (vec![RadiusLen::Px(n as f32)], None)
    } else if let Some(raw) = v.as_str() {
        let (h_str, v_str) = match raw.split_once('/') {
            Some((a, b)) => (a, Some(b)),
            None => (raw, None),
        };
        let vw = s._viewport_width.or(ctx.viewport_width);
        let vh = s._viewport_height.or(ctx.viewport_height);
        let h: Vec<RadiusLen> = h_str
            .split_whitespace()
            .map(|p| parse_radius_part_full(p, fs, vw, vh))
            .collect();
        let vv: Option<Vec<RadiusLen>> = v_str.map(|s| {
            s.split_whitespace()
                .map(|p| parse_radius_part_full(p, fs, vw, vh))
                .collect()
        });
        if h.is_empty() {
            return;
        }
        (h, vv)
    } else {
        return;
    };

    let h4 = expand_corner_list(&h_parts);
    let has_slash = v_parts.is_some();
    let v4 = match &v_parts {
        Some(parts) if !parts.is_empty() => expand_corner_list(parts),
        _ => h4,
    };

    // Per-corner: when there is no slash and the horizontal value isn't a
    // percentage, the corner is treated as "single" (post-resolution we
    // collapse it to a square via `makeSmaller`).
    let single_for = |h: RadiusLen| !has_slash && matches!(h, RadiusLen::Px(_));
    let to_radius = |h: RadiusLen, v: RadiusLen| RadiusValue {
        h,
        v,
        single: single_for(h),
    };

    s.border_top_left_radius = Some(to_radius(h4[0], v4[0]));
    s.border_top_right_radius = Some(to_radius(h4[1], v4[1]));
    s.border_bottom_right_radius = Some(to_radius(h4[2], v4[2]));
    s.border_bottom_left_radius = Some(to_radius(h4[3], v4[3]));
}

fn expand_corner_list(parts: &[RadiusLen]) -> [RadiusLen; 4] {
    match parts.len() {
        1 => [parts[0], parts[0], parts[0], parts[0]],
        2 => [parts[0], parts[1], parts[0], parts[1]],
        3 => [parts[0], parts[1], parts[2], parts[1]],
        _ => [parts[0], parts[1], parts[2], parts[3]],
    }
}

fn parse_radius_value(v: &Value, fs: f32, ctx: ExpandContext) -> Option<RadiusValue> {
    if let Some(n) = v.as_f64() {
        let n = RadiusLen::Px(n as f32);
        return Some(RadiusValue { h: n, v: n, single: true });
    }
    let s = v.as_str()?.trim();
    let parts: Vec<&str> = s.split_whitespace().collect();
    let vw = ctx.viewport_width;
    let vh = ctx.viewport_height;
    match parts.len() {
        1 => {
            let p = parts[0];
            // JS `resolveRadius` only marks a corner as "single" when the
            // input is a single value AND not a percentage.
            let single = !p.ends_with('%');
            let n = parse_radius_part_full(p, fs, vw, vh);
            Some(RadiusValue { h: n, v: n, single })
        }
        2 => {
            let h = parse_radius_part_full(parts[0], fs, vw, vh);
            let vv = parse_radius_part_full(parts[1], fs, vw, vh);
            Some(RadiusValue { h, v: vv, single: false })
        }
        _ => None,
    }
}

fn parse_radius_part_full(
    p: &str,
    fs: f32,
    viewport_w: Option<u32>,
    viewport_h: Option<u32>,
) -> RadiusLen {
    if let Some(n) = p.strip_suffix("px") {
        return RadiusLen::Px(n.trim().parse().unwrap_or(0.0));
    }
    if let Some(n) = p.strip_suffix("rem") {
        let v: f32 = n.trim().parse().unwrap_or(0.0);
        return RadiusLen::Px(v * 16.0);
    }
    if let Some(n) = p.strip_suffix("em") {
        let v: f32 = n.trim().parse().unwrap_or(0.0);
        return RadiusLen::Px(v * fs);
    }
    if let Some(n) = p.strip_suffix("vw") {
        if let Ok(v) = n.trim().parse::<f32>() {
            let vp = viewport_w.unwrap_or(0) as f32;
            return RadiusLen::Px((v * vp / 100.0).trunc());
        }
    }
    if let Some(n) = p.strip_suffix("vh") {
        if let Ok(v) = n.trim().parse::<f32>() {
            let vp = viewport_h.unwrap_or(0) as f32;
            return RadiusLen::Px((v * vp / 100.0).trunc());
        }
    }
    if let Some(n) = p.strip_suffix('%') {
        return RadiusLen::Percent(n.trim().parse().unwrap_or(0.0));
    }
    RadiusLen::Px(p.parse().unwrap_or(0.0))
}

/// Port of the upstream `transform: '<fn>(<args>) <fn>(...)'` parser
/// (`css-to-react-native` + the `lengthToNumber` post-pass in `expand.ts`).
///
/// We parse directly into our typed `TransformOp` enum:
/// - `translate(x[, y])` desugars into `TranslateX`+`TranslateY`.
/// - `scale(x)` stays uniform; `scale(x, y)` desugars into `ScaleX`+`ScaleY`.
/// - Angles are pre-converted from rad/turn/grad to degrees.
/// - Percentages on `translate*` are kept as percentages because they are
///   resolved relative to the element's box size at render time.
///
/// **Quirk we faithfully replicate:** `css-to-react-native` (used by
/// upstream `satori`) emits the resulting array in *reverse source order*,
/// with the components of multi-arg functions also reversed (so
/// `scale(2, 0.2)` becomes `[scaleY:0.2, scaleX:2]`, and
/// `rotate(45deg) scale(2,0.2) translate(50,50)` becomes
/// `[translateY:50, translateX:50, scaleY:0.2, scaleX:2, rotate:45]`).
/// We push in CSS source order here and reverse once at the end —
/// that yields the same final ordering, which the satori matrix-folding
/// math depends on for the exact pixel output.
fn parse_transform_value(v: &Value, fs: f32) -> Option<Vec<TransformOp>> {
    let raw = v.as_str()?.trim();
    if raw.is_empty() || raw == "none" {
        return None;
    }
    let mut ops: Vec<TransformOp> = Vec::new();
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_whitespace() || c == ',' {
            i += 1;
            continue;
        }
        let name_start = i;
        while i < bytes.len() {
            let c = bytes[i] as char;
            if c.is_ascii_alphanumeric() {
                i += 1;
            } else {
                break;
            }
        }
        let name = &raw[name_start..i];
        if name.is_empty() {
            break;
        }
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'(' {
            break;
        }
        i += 1; // consume '('
        let args_start = i;
        let mut depth = 1;
        while i < bytes.len() {
            match bytes[i] {
                b'(' => {
                    depth += 1;
                    i += 1;
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                    i += 1;
                }
                _ => i += 1,
            }
        }
        let args_str = &raw[args_start..i];
        if i < bytes.len() && bytes[i] == b')' {
            i += 1;
        }
        let args: Vec<&str> = args_str.split(',').map(str::trim).filter(|s| !s.is_empty()).collect();
        if args.is_empty() {
            continue;
        }
        match name {
            "translate" => {
                if let Some(x) = parse_transform_len(args[0], fs) {
                    ops.push(TransformOp::TranslateX(x));
                }
                if args.len() > 1 {
                    if let Some(y) = parse_transform_len(args[1], fs) {
                        ops.push(TransformOp::TranslateY(y));
                    }
                }
            }
            "translateX" => {
                if let Some(x) = parse_transform_len(args[0], fs) {
                    ops.push(TransformOp::TranslateX(x));
                }
            }
            "translateY" => {
                if let Some(y) = parse_transform_len(args[0], fs) {
                    ops.push(TransformOp::TranslateY(y));
                }
            }
            "scale" => {
                if args.len() == 1 {
                    if let Ok(x) = args[0].parse::<f32>() {
                        ops.push(TransformOp::Scale(x));
                    }
                } else {
                    if let Ok(x) = args[0].parse::<f32>() {
                        ops.push(TransformOp::ScaleX(x));
                    }
                    if let Ok(y) = args[1].parse::<f32>() {
                        ops.push(TransformOp::ScaleY(y));
                    }
                }
            }
            "scaleX" => {
                if let Ok(x) = args[0].parse::<f32>() {
                    ops.push(TransformOp::ScaleX(x));
                }
            }
            "scaleY" => {
                if let Ok(y) = args[0].parse::<f32>() {
                    ops.push(TransformOp::ScaleY(y));
                }
            }
            "rotate" => {
                if let Some(d) = parse_angle(args[0]) {
                    ops.push(TransformOp::Rotate(d));
                }
            }
            "skewX" => {
                if let Some(d) = parse_angle(args[0]) {
                    ops.push(TransformOp::SkewX(d));
                }
            }
            "skewY" => {
                if let Some(d) = parse_angle(args[0]) {
                    ops.push(TransformOp::SkewY(d));
                }
            }
            "skew" => {
                if let Some(d) = parse_angle(args[0]) {
                    ops.push(TransformOp::SkewX(d));
                }
                if args.len() > 1 {
                    if let Some(d) = parse_angle(args[1]) {
                        ops.push(TransformOp::SkewY(d));
                    }
                }
            }
            "matrix" => {
                if args.len() == 6 {
                    let mut arr = [0.0f32; 6];
                    let mut ok = true;
                    for k in 0..6 {
                        match args[k].parse::<f32>() {
                            Ok(n) => arr[k] = n,
                            Err(_) => {
                                ok = false;
                                break;
                            }
                        }
                    }
                    if ok {
                        ops.push(TransformOp::Matrix(arr));
                    }
                }
            }
            _ => {}
        }
    }
    if ops.is_empty() {
        None
    } else {
        // Replicate css-to-react-native's reversed order.
        ops.reverse();
        Some(ops)
    }
}

fn parse_transform_len(s: &str, fs: f32) -> Option<TransformLen> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(stripped) = s.strip_suffix('%') {
        return stripped.trim().parse::<f32>().ok().map(TransformLen::Percent);
    }
    if let Some(stripped) = s.strip_suffix("px") {
        return stripped.trim().parse::<f32>().ok().map(TransformLen::Px);
    }
    if let Some(stripped) = s.strip_suffix("rem") {
        return stripped.trim().parse::<f32>().ok().map(|v| TransformLen::Px(v * 16.0));
    }
    if let Some(stripped) = s.strip_suffix("em") {
        return stripped.trim().parse::<f32>().ok().map(|v| TransformLen::Px(v * fs));
    }
    s.parse::<f32>().ok().map(TransformLen::Px)
}

/// Port of `calcDegree` in `src/utils.ts`. Bare numbers default to deg.
fn parse_angle(s: &str) -> Option<f32> {
    let s = s.trim();
    if let Some(stripped) = s.strip_suffix("deg") {
        return stripped.trim().parse::<f32>().ok();
    }
    if let Some(stripped) = s.strip_suffix("rad") {
        return stripped
            .trim()
            .parse::<f32>()
            .ok()
            .map(|v| v * 180.0 / std::f32::consts::PI);
    }
    if let Some(stripped) = s.strip_suffix("turn") {
        return stripped.trim().parse::<f32>().ok().map(|v| v * 360.0);
    }
    if let Some(stripped) = s.strip_suffix("grad") {
        return stripped.trim().parse::<f32>().ok().map(|v| v * 0.9);
    }
    s.parse::<f32>().ok()
}

/// Port of `parseTransformOrigin` in `src/transform-origin.ts`.
fn parse_transform_origin_value(v: &Value, fs: f32) -> Option<TransformOrigin> {
    if let Some(n) = v.as_f64() {
        return Some(TransformOrigin {
            x_absolute: Some(n as f32),
            ..Default::default()
        });
    }
    let raw = v.as_str()?.trim();
    let mut words: Vec<&str> = raw.split_whitespace().collect();
    match words.len() {
        1 => Some(handle_origin_word(words[0], fs, true)),
        2 => {
            // `top/bottom` is unambiguously vertical, `left/right` is
            // unambiguously horizontal. Normalize so words[0] is the
            // horizontal axis. (Mirrors the JS `words.reverse()` branch.)
            if matches!(words[0], "top" | "bottom")
                || matches!(words[1], "left" | "right")
            {
                words.reverse();
            }
            let mut a = handle_origin_word(words[0], fs, true);
            let b = handle_origin_word(words[1], fs, false);
            if b.x_relative.is_some() {
                a.x_relative = b.x_relative;
            }
            if b.y_relative.is_some() {
                a.y_relative = b.y_relative;
            }
            if b.x_absolute.is_some() {
                a.x_absolute = b.x_absolute;
            }
            if b.y_absolute.is_some() {
                a.y_absolute = b.y_absolute;
            }
            Some(a)
        }
        _ => Some(TransformOrigin::default()),
    }
}

fn handle_origin_word(word: &str, fs: f32, unit_is_horizontal: bool) -> TransformOrigin {
    match word {
        "top" => TransformOrigin {
            y_relative: Some(0.0),
            ..Default::default()
        },
        "left" => TransformOrigin {
            x_relative: Some(0.0),
            ..Default::default()
        },
        "right" => TransformOrigin {
            x_relative: Some(100.0),
            ..Default::default()
        },
        "bottom" => TransformOrigin {
            y_relative: Some(100.0),
            ..Default::default()
        },
        "center" => TransformOrigin::default(),
        _ => parse_origin_length(word, fs, unit_is_horizontal),
    }
}

fn parse_origin_length(word: &str, fs: f32, unit_is_horizontal: bool) -> TransformOrigin {
    if let Some(stripped) = word.strip_suffix('%') {
        if let Ok(v) = stripped.trim().parse::<f32>() {
            return if unit_is_horizontal {
                TransformOrigin {
                    x_relative: Some(v),
                    ..Default::default()
                }
            } else {
                TransformOrigin {
                    y_relative: Some(v),
                    ..Default::default()
                }
            };
        }
    }
    if let Some(stripped) = word.strip_suffix("px") {
        if let Ok(v) = stripped.trim().parse::<f32>() {
            return if unit_is_horizontal {
                TransformOrigin {
                    x_absolute: Some(v),
                    ..Default::default()
                }
            } else {
                TransformOrigin {
                    y_absolute: Some(v),
                    ..Default::default()
                }
            };
        }
    }
    if let Some(stripped) = word.strip_suffix("rem") {
        if let Ok(v) = stripped.trim().parse::<f32>() {
            let px = v * 16.0;
            return if unit_is_horizontal {
                TransformOrigin {
                    x_absolute: Some(px),
                    ..Default::default()
                }
            } else {
                TransformOrigin {
                    y_absolute: Some(px),
                    ..Default::default()
                }
            };
        }
    }
    if let Some(stripped) = word.strip_suffix("em") {
        if let Ok(v) = stripped.trim().parse::<f32>() {
            let px = v * fs;
            return if unit_is_horizontal {
                TransformOrigin {
                    x_absolute: Some(px),
                    ..Default::default()
                }
            } else {
                TransformOrigin {
                    y_absolute: Some(px),
                    ..Default::default()
                }
            };
        }
    }
    // Bare number → treat as px (matches handleWord falling through).
    if let Ok(v) = word.parse::<f32>() {
        return if unit_is_horizontal {
            TransformOrigin {
                x_absolute: Some(v),
                ..Default::default()
            }
        } else {
            TransformOrigin {
                y_absolute: Some(v),
                ..Default::default()
            }
        };
    }
    TransformOrigin::default()
}

/// Top-level comma split that respects nested `()` — needed because
/// `background-image` layers and color stops both nest commas.
fn split_top_level_commas(s: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut buf = String::new();
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                buf.push(ch);
            }
            ')' => {
                depth -= 1;
                buf.push(ch);
            }
            ',' if depth == 0 => out.push(std::mem::take(&mut buf)),
            c => buf.push(c),
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

fn is_gradient_layer(s: &str) -> bool {
    let s = s.trim();
    s.starts_with("linear-gradient(")
        || s.starts_with("repeating-linear-gradient(")
        || s.starts_with("radial-gradient(")
        || s.starts_with("repeating-radial-gradient(")
        || s.starts_with("conic-gradient(")
        || s.starts_with("repeating-conic-gradient(")
}

fn parse_one_background_layer(s: &str) -> Option<BackgroundImage> {
    let s = s.trim();
    if s.starts_with("linear-gradient(") || s.starts_with("repeating-linear-gradient(") {
        parse_linear_gradient(s).map(BackgroundImage::Linear)
    } else if s.starts_with("radial-gradient(") || s.starts_with("repeating-radial-gradient(") {
        parse_radial_gradient(s).map(BackgroundImage::Radial)
    } else if s.starts_with("conic-gradient(") || s.starts_with("repeating-conic-gradient(") {
        parse_conic_gradient(s).map(BackgroundImage::Conic)
    } else if let Some(rest) = s.strip_prefix("url(") {
        rest.strip_suffix(')').map(|inner| BackgroundImage::Url {
            src: inner
                .trim()
                .trim_matches(|c| c == '"' || c == '\'')
                .to_string(),
            resolved: None,
        })
    } else {
        None
    }
}

/// CSS `flex` shorthand expansion (one, two, or three values).
/// Mirrors browser behavior:
///   * a single `<number>` (e.g. `flex: 1`) sets
///     `flex-grow: <n>`, `flex-shrink: 1`, `flex-basis: 0`.
///   * `flex: none` → `0 0 auto`.
///   * `flex: auto` → `1 1 auto`.
///   * Otherwise, parse up to three space-separated tokens.
fn apply_flex_shorthand(s: &mut ComputedStyle, v: &serde_json::Value, fs: f32) {
    if let Some(n) = v.as_f64() {
        s.flex_grow = Some(n as f32);
        s.flex_shrink = Some(1.0);
        s.flex_basis = Some(super::dimension::Dim::Px(0.0));
        return;
    }
    let Some(raw) = v.as_str() else { return };
    let trimmed = raw.trim();
    if trimmed.eq_ignore_ascii_case("none") {
        s.flex_grow = Some(0.0);
        s.flex_shrink = Some(0.0);
        s.flex_basis = Some(super::dimension::Dim::Auto);
        return;
    }
    if trimmed.eq_ignore_ascii_case("auto") {
        s.flex_grow = Some(1.0);
        s.flex_shrink = Some(1.0);
        s.flex_basis = Some(super::dimension::Dim::Auto);
        return;
    }
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    // Helper: is the token a unitless number?
    fn parse_unitless(t: &str) -> Option<f32> {
        if t.ends_with('%') || t.contains(|c: char| c.is_alphabetic()) {
            return None;
        }
        t.parse::<f32>().ok()
    }
    let parse_basis = |t: &str| -> Option<super::dimension::Dim> {
        parse_dimension(
            &serde_json::Value::String(t.to_string()),
            fs,
            s._viewport_width,
            s._viewport_height,
        )
    };
    match parts.len() {
        1 => {
            let t = parts[0];
            if let Some(n) = parse_unitless(t) {
                s.flex_grow = Some(n);
                s.flex_shrink = Some(1.0);
                s.flex_basis = Some(super::dimension::Dim::Px(0.0));
            } else if let Some(b) = parse_basis(t) {
                s.flex_grow = Some(1.0);
                s.flex_shrink = Some(1.0);
                s.flex_basis = Some(b);
            }
        }
        2 => {
            let (a, b) = (parts[0], parts[1]);
            if let (Some(g), Some(sh)) = (parse_unitless(a), parse_unitless(b)) {
                s.flex_grow = Some(g);
                s.flex_shrink = Some(sh);
                s.flex_basis = Some(super::dimension::Dim::Px(0.0));
            } else if let (Some(g), Some(basis)) = (parse_unitless(a), parse_basis(b)) {
                s.flex_grow = Some(g);
                s.flex_shrink = Some(1.0);
                s.flex_basis = Some(basis);
            }
        }
        _ => {
            let (a, b, c) = (parts[0], parts[1], parts[2]);
            if let (Some(g), Some(sh)) = (parse_unitless(a), parse_unitless(b)) {
                s.flex_grow = Some(g);
                s.flex_shrink = Some(sh);
                if let Some(basis) = parse_basis(c) {
                    s.flex_basis = Some(basis);
                }
            }
        }
    }
}

fn apply_background_image(s: &mut ComputedStyle, v: &serde_json::Value) {
    let Some(raw) = v.as_str() else { return };
    let raw_trimmed = raw.trim();
    // JS satori (`parser/background.ts`) accepts `none`, gradients,
    // `url(...)`, AND any parseable CSS color. A bare unrecognised
    // string like `"foo"` throws "Invalid background image".
    if raw_trimmed.is_empty() || raw_trimmed.eq_ignore_ascii_case("none") {
        return;
    }
    let pieces: Vec<String> = split_top_level_commas(raw)
        .into_iter()
        .map(|p| super::gradient::normalize_webkit_gradient(&p))
        .collect();
    let mut layers: Vec<BackgroundImage> = Vec::with_capacity(pieces.len());
    for piece in &pieces {
        if let Some(bi) = parse_one_background_layer(piece) {
            layers.push(bi);
        } else if looks_like_css_color(piece.trim()) {
            // JS satori renders a single-color `backgroundImage` as a
            // `<pattern>` filled with that colour. We don't have a
            // dedicated SVG-pattern path yet, so map the color to
            // `background-color` (no-op when bg-color is already set,
            // otherwise visually equivalent for the snapshot tests).
            if s.background_color.is_none() {
                s.background_color = Some(piece.trim().to_string());
            }
        } else {
            record_validation_error(format!("Invalid background image: \"{raw}\""));
            return;
        }
    }
    if !layers.is_empty() {
        s.background_image = Some(layers);
    }
}

/// Loose CSS-color detector mirroring the `cssColorParse(image)` branch
/// in `reference/src/builder/background-image.ts`. Matches `#hex`,
/// `rgb[a](...)` / `hsl[a](...)` / modern color functions, and a small
/// set of CSS3 named colors that show up in the satori test corpus.
fn looks_like_css_color(s: &str) -> bool {
    if s.is_empty() { return false; }
    if s.starts_with('#') { return true; }
    let lower = s.to_ascii_lowercase();
    if matches!(lower.as_str(), "transparent" | "currentcolor" | "inherit" | "initial") {
        return true;
    }
    for prefix in ["rgb(", "rgba(", "hsl(", "hsla(", "oklch(", "oklab(", "lab(", "lch(", "color(", "hwb("] {
        if lower.starts_with(prefix) { return true; }
    }
    matches!(
        lower.as_str(),
        "black" | "white" | "red" | "green" | "blue" | "yellow" | "magenta"
        | "cyan" | "gray" | "grey" | "orange" | "purple" | "pink" | "brown"
        | "lime" | "navy" | "olive" | "teal" | "silver" | "maroon"
        | "fuchsia" | "aqua"
    )
}

/// Mirror of `parseMask` (reference/src/parser/mask.ts). Splits the raw
/// `maskImage` value into layers in JS-order, drops `none`, then
/// reverses so the rendering loop emits them in JS order (last listed
/// first painted).
fn apply_mask_image(s: &mut ComputedStyle, v: &serde_json::Value) {
    let Some(raw) = v.as_str() else { return };
    let mut layers: Vec<BackgroundImage> = split_top_level_commas(raw)
        .into_iter()
        .filter(|p| !p.trim().eq_ignore_ascii_case("none") && !p.trim().is_empty())
        .filter_map(|p| parse_one_background_layer(&p))
        .collect();
    layers.reverse();
    if !layers.is_empty() {
        s.mask_image = Some(layers);
    }
}

fn apply_background_shorthand(s: &mut ComputedStyle, v: &serde_json::Value) {
    let Some(raw) = v.as_str() else { return };
    let layers_raw = split_top_level_commas(raw);
    let mut images: Vec<BackgroundImage> = Vec::new();
    let mut color_token: Option<String> = None;
    for layer in layers_raw {
        let trimmed = layer.trim();
        if is_gradient_layer(trimmed) || trimmed.starts_with("url(") {
            if let Some(img) = parse_one_background_layer(trimmed) {
                images.push(img);
            }
        } else if !trimmed.is_empty() {
            // Last non-gradient token becomes the background color — preserve
            // the original CSS string verbatim (matches JS satori: a parsed
            // color is stored as the raw token, not re-emitted as hex).
            color_token = Some(trimmed.to_string());
        }
    }
    if !images.is_empty() {
        s.background_image = Some(images);
    }
    if let Some(c_raw) = color_token {
        s.background_color = Some(normalize_color_token(&c_raw));
    }
}

/// Mirror of `normalizeColor` in `src/handler/expand.ts`. If the color has
/// alpha != 1, re-emit it as `rgba(r, g, b, alpha)` with JS-style number
/// formatting. Otherwise pass the raw token through verbatim.
pub fn normalize_color_token(value: &str) -> String {
    let trimmed = value.trim();
    // `currentColor` is resolved earlier in the pipeline; leave it as-is.
    if trimmed.eq_ignore_ascii_case("currentcolor") {
        return value.to_string();
    }
    // Mirror JS: only `type === 'rgb'` colors with alpha != 1 get
    // re-emitted as `rgba(...)`. `hsl/hsla`, `lab/lch`, named colors,
    // etc. are preserved verbatim.
    let lower = trimmed.to_ascii_lowercase();
    let looks_like_hsl = lower.starts_with("hsl(") || lower.starts_with("hsla(");
    let looks_like_rgb = lower.starts_with("rgb(") || lower.starts_with("rgba(");
    let looks_like_hex = lower.starts_with('#');
    if looks_like_hsl {
        return value.to_string();
    }
    if !(looks_like_rgb || looks_like_hex || is_named_with_alpha(&lower)) {
        return value.to_string();
    }
    if let Some(c) = parse_color(trimmed) {
        if c.a != 0xff {
            let alpha = extract_original_alpha(trimmed).unwrap_or(c.a as f64 / 255.0);
            return format!("rgba({}, {}, {}, {})", c.r, c.g, c.b, format_js_number(alpha));
        }
    }
    value.to_string()
}

/// Named CSS colors don't carry alpha in the source string except for
/// `transparent` — that one alone normalizes to `rgba(0, 0, 0, 0)`.
fn is_named_with_alpha(lower: &str) -> bool {
    lower == "transparent"
}

/// Replace any case-insensitive `currentcolor` in `value` with the
/// resolved color string (`current_color`). Mirrors JS
/// `convertCurrentColorToActualValue` in `src/handler/expand.ts`.
fn convert_current_color(value: &str, current_color: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let needle: &[u8] = b"currentcolor";
    let mut i = 0;
    while i < bytes.len() {
        let remaining = &bytes[i..];
        if remaining.len() >= needle.len() {
            let prefix = &remaining[..needle.len()];
            if prefix.eq_ignore_ascii_case(needle) {
                out.push_str(current_color);
                i += needle.len();
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Pull the alpha component out of the raw input string. For `rgba()`,
/// `hsla()`, `rgb()` (functional 4-arg form), `hsl()` (4-arg form), and
/// 4 / 8-char hex with alpha we recover the original alpha precision so
/// the re-emitted token matches JS `${number}` formatting.
fn extract_original_alpha(s: &str) -> Option<f64> {
    let lower = s.trim().to_ascii_lowercase();
    if lower.starts_with("rgba(")
        || lower.starts_with("hsla(")
        || lower.starts_with("rgb(")
        || lower.starts_with("hsl(")
    {
        let open = lower.find('(')? + 1;
        let close = lower.rfind(')')?;
        let inner = &lower[open..close];
        // Both legacy (`,`) and modern (`/`-separated) syntaxes are
        // accepted; treat whitespace, commas and slashes uniformly so a
        // value like `rgb(255 0 0 / 50%)` produces 4 numeric tokens.
        let parts: Vec<&str> = inner
            .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
            .map(str::trim)
            .filter(|p| !p.is_empty())
            .collect();
        if parts.len() < 4 {
            return None;
        }
        let alpha = parts[3];
        if let Some(p) = alpha.strip_suffix('%') {
            return p.parse::<f64>().ok().map(|v| v / 100.0);
        }
        return alpha.parse::<f64>().ok();
    }
    if let Some(hex) = s.trim().strip_prefix('#') {
        let bytes = hex.as_bytes();
        match hex.len() {
            4 => hex_digit(bytes[3]).map(|v| (v as f64 * 17.0) / 255.0),
            8 => {
                let hi = hex_digit(bytes[6])?;
                let lo = hex_digit(bytes[7])?;
                Some(((hi * 16 + lo) as f64) / 255.0)
            }
            _ => None,
        }
    } else {
        None
    }
}

fn hex_digit(b: u8) -> Option<u32> {
    match b {
        b'0'..=b'9' => Some((b - b'0') as u32),
        b'a'..=b'f' => Some((b - b'a' + 10) as u32),
        b'A'..=b'F' => Some((b - b'A' + 10) as u32),
        _ => None,
    }
}

/// JS `${number}` formatting for f64 values: integers render without
/// a trailing `.0`. JS uses the ECMA "Number to String" rules which
/// produce the shortest decimal that round-trips through f64; Rust's
/// `Display` impl is close enough for the values seen in CSS color
/// tokens.
pub fn format_js_number(n: f64) -> String {
    if n == n.trunc() && n.is_finite() && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    format!("{n}")
}

fn parse_align_self(s: &str) -> Option<AlignSelf> {
    Some(match s {
        "auto" => AlignSelf::Auto,
        "stretch" => AlignSelf::Stretch,
        "flex-start" => AlignSelf::FlexStart,
        "flex-end" => AlignSelf::FlexEnd,
        "center" => AlignSelf::Center,
        "baseline" => AlignSelf::Baseline,
        _ => return None,
    })
}

fn parse_align_content(s: &str) -> Option<AlignContent> {
    Some(match s {
        "auto" => AlignContent::Auto,
        "stretch" => AlignContent::Stretch,
        "flex-start" => AlignContent::FlexStart,
        "flex-end" => AlignContent::FlexEnd,
        "center" => AlignContent::Center,
        "space-between" => AlignContent::SpaceBetween,
        "space-around" => AlignContent::SpaceAround,
        "space-evenly" => AlignContent::SpaceEvenly,
        _ => return None,
    })
}

/// Split a CSS value list on top-level commas (commas outside `()`).
///
/// Mirrors the JS `VALUES_REG = /,(?![^\(]*\))/` used by upstream
/// `css-box-shadow` to keep `rgba(0, 0, 0, 0.5)` and friends intact.
/// Tokenize a single shadow entry on whitespace, ignoring whitespace
/// inside parentheses. Mirrors `PARTS_REG = /\s(?![^(]*\))/`.
fn tokenize_shadow_entry(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '(' => { depth += 1; cur.push(c); }
            ')' => { depth -= 1; cur.push(c); }
            c if c.is_whitespace() && depth == 0 => {
                if !cur.is_empty() {
                    out.push(cur.clone());
                    cur.clear();
                }
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() { out.push(cur); }
    out
}

/// Parse a single shadow length token (px / em / rem / bare 0 / signed
/// numbers). Returns `None` if the token doesn't look like a length —
/// i.e. probably a color or the `inset` keyword.
fn parse_shadow_length(s: &str, fs: f32) -> Option<f32> {
    let s = s.trim();
    if s.is_empty() { return None; }
    // Hex colors start with '#'; never a length.
    if s.starts_with('#') { return None; }
    if let Some(stripped) = s.strip_suffix("px") {
        return stripped.trim().parse::<f32>().ok();
    }
    if let Some(stripped) = s.strip_suffix("rem") {
        return stripped.trim().parse::<f32>().ok().map(|v| v * 16.0);
    }
    if let Some(stripped) = s.strip_suffix("em") {
        return stripped.trim().parse::<f32>().ok().map(|v| v * fs);
    }
    if let Some(stripped) = s.strip_suffix('%') {
        // Percentages aren't valid in box-shadow but treat as bare px for safety.
        return stripped.trim().parse::<f32>().ok();
    }
    // Bare number — only treat as length if the whole token parses cleanly.
    s.parse::<f32>().ok()
}

/// Parse the `boxShadow` value (one or more comma-separated entries).
fn parse_box_shadow_value(v: &Value, fs: f32) -> Option<Vec<BoxShadow>> {
    let raw = v.as_str()?.trim();
    if raw.is_empty() || raw == "none" { return None; }
    let entries = split_top_level_commas(raw);
    let mut out: Vec<BoxShadow> = Vec::new();
    for entry in entries {
        if let Some(sh) = parse_single_box_shadow(&entry, fs) {
            out.push(sh);
        }
    }
    if out.is_empty() { None } else { Some(out) }
}

fn parse_single_box_shadow(entry: &str, fs: f32) -> Option<BoxShadow> {
    let tokens = tokenize_shadow_entry(entry);
    let mut lengths: Vec<f32> = Vec::new();
    let mut color: Option<String> = None;
    let mut inset = false;
    for tok in &tokens {
        if tok == "inset" {
            inset = true;
            continue;
        }
        if let Some(v) = parse_shadow_length(tok, fs) {
            lengths.push(v);
            continue;
        }
        // Anything else is the color. Preserve the original token verbatim
        // — JS satori passes the raw `css-box-shadow` color string through
        // to `flood-color=` without normalization, so we match that to
        // avoid alpha-precision drift like `rgba(0, 0, 0, 0.5)` →
        // `rgba(0,0,0,0.501961)`.
        color = Some(tok.clone());
    }
    if lengths.len() < 2 { return None; }
    let offset_x = lengths[0];
    let offset_y = lengths[1];
    let blur = if lengths.len() > 2 { lengths[2] } else { 0.0 };
    let spread = if lengths.len() > 3 { lengths[3] } else { 0.0 };
    Some(BoxShadow {
        offset_x,
        offset_y,
        blur,
        spread,
        color: color.unwrap_or_else(|| "black".to_string()),
        inset,
    })
}

/// Parse the `textShadow` value. The grammar is the same as `box-shadow`
/// minus the optional `spread` length and `inset` keyword.
fn parse_text_shadow_value(v: &Value, fs: f32) -> Option<Vec<TextShadow>> {
    let raw = v.as_str()?.trim();
    if raw.is_empty() || raw == "none" { return None; }
    let entries = split_top_level_commas(raw);
    let mut out: Vec<TextShadow> = Vec::new();
    for entry in entries {
        let tokens = tokenize_shadow_entry(&entry);
        let mut lengths: Vec<f32> = Vec::new();
        let mut color: Option<String> = None;
        for tok in &tokens {
            if let Some(v) = parse_shadow_length(tok, fs) {
                lengths.push(v);
                continue;
            }
            color = Some(tok.clone());
        }
        if lengths.len() < 2 { continue; }
        let offset_x = lengths[0];
        let offset_y = lengths[1];
        let blur = if lengths.len() > 2 { lengths[2] } else { 0.0 };
        out.push(TextShadow {
            offset_x,
            offset_y,
            blur,
            color: color.unwrap_or_else(|| "black".to_string()),
        });
    }
    if out.is_empty() { None } else { Some(out) }
}
