//! Port of the upstream `css-gradient-parser` (only the subset used by
//! `src/builder/background-image.ts` for linear + radial gradients).
//!
//! The JS package exposes `parseLinearGradient(input)` and
//! `parseRadialGradient(input)` returning typed AST nodes. We mirror the
//! AST shape closely so the renderer can mechanically port the JS code.
//!
//! Supported syntax:
//! - `linear-gradient(<orientation>?, <stops>)` where orientation is one
//!   of: directional keyword set (`to right`, `to bottom right`, …),
//!   `<angle><unit>` (deg/rad/turn/grad) or omitted (defaults to `to bottom`).
//! - `repeating-linear-gradient(...)` — same payload, `repeating: true`.
//! - `radial-gradient([<shape>] [<size>] [at <position>], <stops>)`.
//! - `repeating-radial-gradient(...)`.
//!
//! Color stops parse `<color>[ <length>]` where `<length>` is `<n>%`,
//! `<n>px`, `<n>em`, `<n>rem` (other units pass through textually so
//! `lengthToNumber` can resolve them).

#[derive(Debug, Clone, PartialEq)]
pub struct ColorStop {
    pub color: String,
    pub offset: Option<StopOffset>,
    /// Conic-gradient transition hint between this stop and the next, e.g.
    /// `conic-gradient(red, 30%, blue)` — the `30%` becomes the hint on
    /// the `red` stop. `None` for linear/radial gradients.
    pub hint: Option<StopOffset>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StopOffset {
    /// Numeric string (kept as a JS-style string so callers can round-trip).
    pub value: String,
    /// Unit string (e.g. `"%"`, `"px"`, `"em"`). Empty for unitless.
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LinearOrientation {
    /// Direction keywords joined by spaces, e.g. `"top"`, `"right"`,
    /// `"bottom right"`, `"top left"`. Order is normalized as in CSS source.
    Directional(String),
    Angular {
        value: String,
        unit: String, // "deg", "rad", "turn", "grad", or "" (bare number → deg)
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearGradient {
    pub orientation: LinearOrientation,
    pub stops: Vec<ColorStop>,
    pub repeating: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RadialPropertyValue {
    Keyword(String),
    Length(StopOffset),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialPosition {
    pub x: RadialPropertyValue,
    pub y: RadialPropertyValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RadialGradient {
    pub shape: String, // "circle" | "ellipse"
    pub size: Vec<RadialPropertyValue>,
    pub position: RadialPosition,
    pub stops: Vec<ColorStop>,
    pub repeating: bool,
}

/// Parsed `conic-gradient(...)` or `repeating-conic-gradient(...)`.
///
/// The grammar is:
///
/// ```text
/// conic-gradient(
///   [ [ from <angle> ]? [ at <position> ]? ]?,
///   <color-stop-list>
/// )
/// ```
///
/// `angle` defaults to `"0deg"`; `position` defaults to `"center"`. Both
/// are kept as raw strings to match the JS `css-gradient-parser` AST so the
/// downstream renderer can mechanically port `buildConicGradient`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConicGradient {
    pub angle: String,
    pub position: String,
    pub stops: Vec<ColorStop>,
    pub repeating: bool,
}

const RADIAL_EXTENT_KEYWORDS: &[&str] = &[
    "closest-corner",
    "closest-side",
    "farthest-corner",
    "farthest-side",
];

/// Parse `linear-gradient(...)` or `repeating-linear-gradient(...)`.
pub fn parse_linear_gradient(input: &str) -> Option<LinearGradient> {
    let s = input.trim();
    let (body, repeating) = if let Some(b) = strip_func_call(s, "repeating-linear-gradient") {
        (b, true)
    } else {
        (strip_func_call(s, "linear-gradient")?, false)
    };

    let parts = split_top_level(body, ',');
    if parts.is_empty() {
        return None;
    }

    // First component may be the orientation if it doesn't look like a color
    // stop (i.e. doesn't start with a known color token).
    let first = parts[0].trim();
    let (orientation, stop_parts) = if is_orientation_token(first) {
        let o = parse_linear_orientation(first)?;
        (o, &parts[1..])
    } else {
        // Default direction per CSS spec: "to bottom".
        (LinearOrientation::Directional("bottom".to_string()), &parts[..])
    };

    let stops = parse_stops(stop_parts);
    Some(LinearGradient {
        orientation,
        stops,
        repeating,
    })
}

/// Parse `radial-gradient(...)` or `repeating-radial-gradient(...)`.
pub fn parse_radial_gradient(input: &str) -> Option<RadialGradient> {
    let s = input.trim();
    let (body, repeating) = if let Some(b) = strip_func_call(s, "repeating-radial-gradient") {
        (b, true)
    } else {
        (strip_func_call(s, "radial-gradient")?, false)
    };

    let parts = split_top_level(body, ',');
    if parts.is_empty() {
        return None;
    }

    let first = parts[0].trim();
    let (shape, size, position, stop_offset) =
        if let Some(prelude) = parse_radial_prelude(first) {
            (prelude.0, prelude.1, prelude.2, 1)
        } else {
            (
                "ellipse".to_string(),
                Vec::new(),
                RadialPosition {
                    x: RadialPropertyValue::Keyword("center".to_string()),
                    y: RadialPropertyValue::Keyword("center".to_string()),
                },
                0,
            )
        };

    let stops = parse_stops(&parts[stop_offset..]);
    Some(RadialGradient {
        shape,
        size,
        position,
        stops,
        repeating,
    })
}

fn strip_func_call<'a>(s: &'a str, name: &str) -> Option<&'a str> {
    let s = s.trim();
    let rest = s.strip_prefix(name)?;
    let rest = rest.trim_start().strip_prefix('(')?;
    let rest = rest.trim_end().strip_suffix(')')?;
    Some(rest)
}

/// Split a comma-separated list, respecting nested `()`.
fn split_top_level(s: &str, sep: char) -> Vec<String> {
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
            c if c == sep && depth == 0 => {
                out.push(std::mem::take(&mut buf));
            }
            c => buf.push(c),
        }
    }
    if !buf.is_empty() {
        out.push(buf);
    }
    out
}

fn is_orientation_token(s: &str) -> bool {
    let lower = s.to_ascii_lowercase();
    if lower.starts_with("to ") {
        return true;
    }
    if let Some(_unit_split) = split_number_unit(&lower) {
        return true;
    }
    false
}

fn parse_linear_orientation(s: &str) -> Option<LinearOrientation> {
    let trimmed = s.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("to ") {
        let keywords = rest.split_whitespace().collect::<Vec<_>>().join(" ");
        return Some(LinearOrientation::Directional(keywords));
    }
    let (value, unit) = split_number_unit(trimmed)?;
    Some(LinearOrientation::Angular {
        value: value.to_string(),
        unit: unit.to_string(),
    })
}

/// Try to split a token like `"45deg"` / `"0.5turn"` / `"-1.2rad"` /
/// `"90"` into (numeric_part, unit). Returns None if the leading
/// characters don't form a valid number.
fn split_number_unit(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') {
        i += 1;
    }
    let digits_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    if i == digits_start {
        return None;
    }
    Some((&s[..i], &s[i..]))
}

/// Parse the prelude of a radial gradient — the part before the stops
/// list. Returns (shape, size, position) on success or None if the
/// first token already looks like a color stop.
fn parse_radial_prelude(
    s: &str,
) -> Option<(String, Vec<RadialPropertyValue>, RadialPosition)> {
    let lower = s.to_ascii_lowercase();
    let mut tokens: Vec<&str> = lower.split_whitespace().collect();
    if tokens.is_empty() {
        return None;
    }

    // Does it look like a radial prelude (vs. a color stop)?
    let has_at = tokens.contains(&"at");
    let has_shape_or_ext = tokens.iter().any(|t| {
        *t == "circle" || *t == "ellipse" || RADIAL_EXTENT_KEYWORDS.contains(t)
    });
    let first_looks_numeric = split_number_unit(tokens[0]).is_some();
    if !(has_at || has_shape_or_ext || first_looks_numeric) {
        return None;
    }

    // Split on "at" → [size/shape_tokens, position_tokens]
    let mut at_idx: Option<usize> = None;
    for (i, t) in tokens.iter().enumerate() {
        if *t == "at" {
            at_idx = Some(i);
            break;
        }
    }
    let (size_tokens, position_tokens): (Vec<&str>, Vec<&str>) = match at_idx {
        Some(i) => {
            let pos = tokens.split_off(i + 1);
            tokens.pop(); // remove "at"
            (tokens, pos)
        }
        None => (tokens, Vec::new()),
    };

    let mut shape = String::new();
    let mut size: Vec<RadialPropertyValue> = Vec::new();
    for tok in &size_tokens {
        if *tok == "circle" || *tok == "ellipse" {
            shape = (*tok).to_string();
        } else if RADIAL_EXTENT_KEYWORDS.contains(tok) {
            size.push(RadialPropertyValue::Keyword((*tok).to_string()));
        } else if let Some((value, unit)) = split_number_unit(tok) {
            // CSS-Images-3 §3.4.3: `<rg-size>` lengths must be
            // non-negative. Mirror JS satori, which records a
            // validation error pointing at the spec.
            if value.starts_with('-') {
                crate::expand::record_validation_error(
                    "disallow setting negative values to the size of the shape. \
                     Check https://w3c.github.io/csswg-drafts/css-images/#valdef-rg-size-length-0",
                );
            }
            size.push(RadialPropertyValue::Length(StopOffset {
                value: value.to_string(),
                unit: unit.to_string(),
            }));
        }
    }
    if shape.is_empty() {
        // Default: ellipse if 2 lengths, circle if 1 length, else ellipse.
        shape = match size.iter().filter(|v| matches!(v, RadialPropertyValue::Length(_))).count() {
            1 => "circle".to_string(),
            _ => "ellipse".to_string(),
        };
    }
    if size.is_empty() {
        size.push(RadialPropertyValue::Keyword("farthest-corner".to_string()));
    }

    let position = parse_radial_position(&position_tokens);
    Some((shape, size, position))
}

fn parse_radial_position(tokens: &[&str]) -> RadialPosition {
    let center = RadialPropertyValue::Keyword("center".to_string());
    match tokens.len() {
        0 => RadialPosition { x: center.clone(), y: center },
        1 => {
            let v = radial_value_from_token(tokens[0]);
            // Single keyword: `left|right` → x, `top|bottom` → y, else x with y=center.
            match &v {
                RadialPropertyValue::Keyword(k) if k == "top" || k == "bottom" => {
                    RadialPosition { x: center, y: v }
                }
                _ => RadialPosition { x: v, y: center },
            }
        }
        _ => {
            let a = radial_value_from_token(tokens[0]);
            let b = radial_value_from_token(tokens[1]);
            // If first is top/bottom or second is left/right, swap.
            let swap = matches!(&a, RadialPropertyValue::Keyword(k) if k == "top" || k == "bottom")
                || matches!(&b, RadialPropertyValue::Keyword(k) if k == "left" || k == "right");
            if swap {
                RadialPosition { x: b, y: a }
            } else {
                RadialPosition { x: a, y: b }
            }
        }
    }
}

fn radial_value_from_token(t: &str) -> RadialPropertyValue {
    match t {
        "center" | "left" | "right" | "top" | "bottom" => {
            RadialPropertyValue::Keyword(t.to_string())
        }
        _ => {
            if let Some((value, unit)) = split_number_unit(t) {
                RadialPropertyValue::Length(StopOffset {
                    value: value.to_string(),
                    unit: unit.to_string(),
                })
            } else {
                RadialPropertyValue::Keyword(t.to_string())
            }
        }
    }
}

/// Parse `<color> [<length>]` stops. Color tokens may themselves contain
/// commas inside `rgb(...)`/`rgba(...)`/`hsl(...)` — `split_top_level`
/// already respected the parens, so each entry here is a single stop.
fn parse_stops(parts: &[String]) -> Vec<ColorStop> {
    let mut stops = Vec::with_capacity(parts.len());
    for raw in parts {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        // Find the split point between color and optional offset. The
        // color may contain spaces inside `rgb(...)` / `hsl(...)`, so
        // walk from the right and look for the first top-level space
        // outside any parens, then check whether the trailing token is
        // a length/percentage.
        let bytes = trimmed.as_bytes();
        let mut depth = 0i32;
        let mut last_space: Option<usize> = None;
        for (i, ch) in trimmed.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                ' ' if depth == 0 => last_space = Some(i),
                _ => {}
            }
        }
        let (color_str, offset) = if let Some(sp) = last_space {
            let candidate = trimmed[sp + 1..].trim();
            if let Some(off) = parse_stop_offset(candidate) {
                (trimmed[..sp].trim().to_string(), Some(off))
            } else {
                (trimmed.to_string(), None)
            }
        } else {
            (trimmed.to_string(), None)
        };
        let _ = bytes;
        stops.push(ColorStop {
            color: color_str,
            offset,
            hint: None,
        });
    }
    stops
}

fn parse_stop_offset(s: &str) -> Option<StopOffset> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    let (value, unit) = split_number_unit(trimmed)?;
    // Require either a unit (px / % / em / rem) or a bare number that
    // we still treat as unitless (length 0 case).
    Some(StopOffset {
        value: value.to_string(),
        unit: unit.to_string(),
    })
}

// ----- conic gradient -------------------------------------------------

/// Set of tokens that introduce a keyword-prefixed prelude segment in a
/// `conic-gradient(...)`. `from <angle>` and `at <position>` set fields
/// on the result; `in <color-space>` is parsed for completeness but
/// ignored by the renderer (matches JS satori).
const CONIC_PRELUDE_KEYWORDS: &[&str] = &["from", "at", "in"];

/// Parse `conic-gradient(...)` or `repeating-conic-gradient(...)`.
///
/// Mirrors the JS `parseConicGradient` from `css-gradient-parser` after
/// running it through `expandTwoPositionStops` (i.e. `red 0% 25%` becomes
/// `red 0%, red 25%`).
pub fn parse_conic_gradient(input: &str) -> Option<ConicGradient> {
    let s = input.trim();
    let (raw_body, repeating) = if let Some(b) = strip_func_call(s, "repeating-conic-gradient") {
        (b, true)
    } else {
        (strip_func_call(s, "conic-gradient")?, false)
    };
    let body = expand_two_position_stops(raw_body);
    let parts = split_top_level(&body, ',');
    if parts.is_empty() {
        return None;
    }

    let mut angle = String::from("0deg");
    let mut position = String::from("center");
    let mut drop_first = false;

    // First entry MAY contain `from <angle>` / `at <position>` / `in <cs>`
    // segments separated by whitespace. JS walks the tokens and groups
    // consecutive non-keyword tokens under the most recently seen keyword.
    let first = parts[0].trim();
    let tokens: Vec<&str> = first.split_ascii_whitespace().collect();
    let mut cur_kw: Option<&str> = None;
    let mut cur_start: usize = 0;
    let mut saw_kw = false;
    for (i, tok) in tokens.iter().enumerate() {
        if CONIC_PRELUDE_KEYWORDS.contains(tok) {
            if let Some(prev) = cur_kw {
                apply_conic_keyword(prev, &tokens[cur_start..i], &mut angle, &mut position);
            }
            cur_kw = Some(tok);
            cur_start = i + 1;
            saw_kw = true;
        }
    }
    if let Some(prev) = cur_kw {
        apply_conic_keyword(prev, &tokens[cur_start..], &mut angle, &mut position);
    }
    if saw_kw {
        drop_first = true;
    }

    let stop_parts: &[String] = if drop_first { &parts[1..] } else { &parts[..] };
    let stops = parse_conic_stops(stop_parts);

    Some(ConicGradient { angle, position, stops, repeating })
}

fn apply_conic_keyword(kw: &str, vals: &[&str], angle: &mut String, position: &mut String) {
    match kw {
        "from" => *angle = vals.join(" "),
        "at" => *position = vals.join(" "),
        "in" => { /* color space — ignored by the renderer */ }
        _ => {}
    }
}

/// Port of `expandTwoPositionStops` in `gradient/conic.ts`:
/// turn `red 0% 25%` (a "two-position color stop" CSS shorthand) into
/// `red 0%, red 25%`. Operates on the **inside** of the gradient call.
fn expand_two_position_stops(body: &str) -> String {
    let segments = split_top_level(body, ',');
    let mut out: Vec<String> = Vec::with_capacity(segments.len());
    for seg in &segments {
        let trimmed = seg.trim();
        let toks: Vec<&str> = trimmed.split_ascii_whitespace().collect();
        // Don't expand prelude segments (from/at/in).
        if toks.iter().any(|t| CONIC_PRELUDE_KEYWORDS.contains(t)) {
            out.push(seg.clone());
            continue;
        }
        if toks.len() >= 3
            && is_dimension_value(toks[toks.len() - 1])
            && is_dimension_value(toks[toks.len() - 2])
        {
            let color = toks[..toks.len() - 2].join(" ");
            out.push(format!("{} {}", color, toks[toks.len() - 2]));
            out.push(format!("{} {}", color, toks[toks.len() - 1]));
        } else {
            out.push(seg.clone());
        }
    }
    out.join(", ")
}

/// Conic-flavored stop parser. Mirrors the JS `g(e)`:
/// walks the comma-separated entries; if entry `e[t+1]` looks like a
/// bare dimension value, it's the transition hint on `e[t]` and we
/// advance by 2 instead of 1.
fn parse_conic_stops(parts: &[String]) -> Vec<ColorStop> {
    let mut stops: Vec<ColorStop> = Vec::with_capacity(parts.len());
    let mut t = 0usize;
    while t < parts.len() {
        let entry = parts[t].trim();
        // Split on the *first* whitespace into [color, offset] — color may
        // contain spaces inside `rgb(...)`, but `split_top_level` already
        // respected the parens at the comma level. The `c(...)` JS helper
        // walks token-by-token; we approximate by scanning for the first
        // top-level whitespace.
        let (color, offset_str) = split_first_whitespace_top_level(entry);
        let offset = parse_conic_dimension(offset_str);
        // Look ahead: is the next part a single dimension value (a hint)?
        let mut hint: Option<StopOffset> = None;
        if t + 1 < parts.len() {
            let next = parts[t + 1].trim();
            if is_dimension_value(next) {
                hint = parse_conic_dimension(Some(next));
                t += 1;
            }
        }
        stops.push(ColorStop {
            color: color.to_string(),
            offset,
            hint,
        });
        t += 1;
    }
    stops
}

/// Returns true if `s` matches the JS regex
/// `^-?\d+\.?\d*(%|vw|vh|px|em|rem|deg|rad|grad|turn|ch|vmin|vmax)?$`.
fn is_dimension_value(s: &str) -> bool {
    let b = s.as_bytes();
    if b.is_empty() {
        return false;
    }
    let mut i = 0;
    if b[i] == b'-' {
        i += 1;
        if i >= b.len() {
            return false;
        }
    }
    let digits_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let saw_int_digits = i > digits_start;
    if i < b.len() && b[i] == b'.' {
        i += 1;
        let frac_start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        // JS regex `\d+\.?\d*` allows trailing dot with no fraction, but
        // the leading `\d+` is required. We require at least the int part.
        let _ = frac_start;
    }
    if !saw_int_digits {
        return false;
    }
    if i == b.len() {
        return true;
    }
    let unit = &s[i..];
    matches!(
        unit,
        "%" | "vw"
            | "vh"
            | "px"
            | "em"
            | "rem"
            | "deg"
            | "rad"
            | "grad"
            | "turn"
            | "ch"
            | "vmin"
            | "vmax"
    )
}

/// Parse a single conic dimension token. Note that, matching the JS
/// `l(e)` helper, missing units default to `"px"` (not the empty string
/// that linear/radial use). This makes the `unit === '%'` and downstream
/// `lengthToNumber("0px")` paths line up byte-for-byte.
fn parse_conic_dimension(s: Option<&str>) -> Option<StopOffset> {
    let s = s?.trim();
    if s.is_empty() {
        return None;
    }
    let (value, unit) = split_number_unit(s)?;
    let unit = if unit.is_empty() { "px" } else { unit };
    Some(StopOffset {
        value: value.to_string(),
        unit: unit.to_string(),
    })
}

/// Like `str::splitn(2, char::is_whitespace)`, but respects nested `()`
/// so `rgba(1, 2, 3, 0.5)` stays in the color half even if it has
/// internal spaces.
fn split_first_whitespace_top_level(s: &str) -> (&str, Option<&str>) {
    let mut depth = 0i32;
    for (i, ch) in s.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            c if c.is_ascii_whitespace() && depth == 0 => {
                let color = s[..i].trim_end();
                let rest = s[i + 1..].trim_start();
                return (color, if rest.is_empty() { None } else { Some(rest) });
            }
            _ => {}
        }
    }
    (s, None)
}

/// Rewrite `-webkit-(repeating-)?(linear|radial)-gradient(...)` into the
/// standards-track form. Port of
/// `reference/src/builder/gradient/webkit.ts::normalizeWebkitGradient`.
pub fn normalize_webkit_gradient(image: &str) -> String {
    let s = image.trim_start();
    let (repeating, rest) = if let Some(r) = s.strip_prefix("-webkit-repeating-") {
        ("repeating-", r)
    } else if let Some(r) = s.strip_prefix("-webkit-") {
        ("", r)
    } else {
        return image.to_string();
    };
    let (ty, body) = if let Some(b) = rest.strip_prefix("linear-gradient(") {
        ("linear", b)
    } else if let Some(b) = rest.strip_prefix("radial-gradient(") {
        ("radial", b)
    } else {
        return image.to_string();
    };
    let Some(content) = body.strip_suffix(')') else {
        return image.to_string();
    };
    let converted = if ty == "linear" {
        convert_linear_args(content)
    } else {
        convert_radial_args(content)
    };
    format!("{repeating}{ty}-gradient({converted})")
}

fn split_top_level_commas_local(s: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' => depth += 1,
            b')' => depth -= 1,
            b',' if depth == 0 => {
                out.push(s[start..i].trim().to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(s[start..].trim().to_string());
    out
}

fn direction_flip(w: &str) -> Option<&'static str> {
    match w {
        "left" => Some("right"),
        "right" => Some("left"),
        "top" => Some("bottom"),
        "bottom" => Some("top"),
        _ => None,
    }
}

fn is_direction_keywords(arg: &str) -> bool {
    let tokens: Vec<&str> = arg.split_whitespace().collect();
    !tokens.is_empty() && tokens.iter().all(|t| direction_flip(t).is_some())
}

fn parse_angle(arg: &str) -> Option<(f64, &'static str)> {
    let lower = arg.trim();
    for u in ["deg", "rad", "grad", "turn"] {
        if let Some(num) = lower.strip_suffix(u) {
            return num.parse::<f64>().ok().map(|v| (v, u));
        }
    }
    None
}

fn to_degrees(value: f64, unit: &str) -> f64 {
    match unit {
        "deg" => value,
        "rad" => value * 180.0 / std::f64::consts::PI,
        "turn" => value * 360.0,
        "grad" => value * 0.9,
        _ => value,
    }
}

fn convert_linear_args(content: &str) -> String {
    let mut parts = split_top_level_commas_local(content);
    if parts.is_empty() { return content.to_string(); }
    let first = parts[0].trim().to_string();
    if is_direction_keywords(&first) {
        let flipped: Vec<&str> = first.split_whitespace().filter_map(direction_flip).collect();
        parts[0] = format!("to {}", flipped.join(" "));
        return parts.join(", ");
    }
    if let Some((value, unit)) = parse_angle(&first) {
        let deg = to_degrees(value, unit);
        parts[0] = format!("{}deg", 90.0 - deg);
        return parts.join(", ");
    }
    content.to_string()
}

const POSITION_KEYWORDS: &[&str] = &["center", "left", "right", "top", "bottom"];
const SHAPE_SIZE_KEYWORDS: &[&str] = &[
    "circle", "ellipse", "closest-side", "closest-corner",
    "farthest-side", "farthest-corner", "contain", "cover",
];

fn is_length_token(t: &str) -> bool {
    let bytes = t.as_bytes();
    let mut i = 0;
    if bytes.first() == Some(&b'-') { i += 1; }
    let digits_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
    if i == digits_start { return false; }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        let frac_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
        if i == frac_start { return false; }
    }
    let unit = &t[i..];
    matches!(unit, "%" | "px" | "em" | "rem" | "vw" | "vh")
}

fn is_position(arg: &str) -> bool {
    let tokens: Vec<&str> = arg.split_whitespace().collect();
    !tokens.is_empty() && tokens.iter().all(|t| POSITION_KEYWORDS.contains(t) || is_length_token(t))
}

fn is_shape_or_size(arg: &str) -> bool {
    let tokens: Vec<&str> = arg.split_whitespace().collect();
    if tokens.iter().any(|t| SHAPE_SIZE_KEYWORDS.contains(t)) { return true; }
    !tokens.is_empty() && tokens.len() <= 2 && tokens.iter().all(|t| is_length_token(t))
}

fn replace_webkit_size_aliases(shape_size: &str) -> String {
    shape_size.split_whitespace().map(|t| match t {
        "contain" => "closest-side",
        "cover" => "farthest-corner",
        other => other,
    }).collect::<Vec<&str>>().join(" ")
}

fn convert_radial_args(content: &str) -> String {
    let parts = split_top_level_commas_local(content);
    if parts.is_empty() { return content.to_string(); }
    let first = parts[0].trim();
    if is_position(first) {
        if parts.len() > 1 && is_shape_or_size(parts[1].trim()) {
            let position = first;
            let shape_size = replace_webkit_size_aliases(parts[1].trim());
            let rest: Vec<String> = parts.iter().skip(2).cloned().collect();
            return format!("{shape_size} at {position}, {}", rest.join(", "));
        }
        let rest: Vec<String> = parts.iter().skip(1).cloned().collect();
        return format!("at {first}, {}", rest.join(", "));
    }
    if is_shape_or_size(first) {
        let mut new_parts = parts.clone();
        new_parts[0] = replace_webkit_size_aliases(first);
        return new_parts.join(", ");
    }
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_default_orientation() {
        let g = parse_linear_gradient("linear-gradient(red, blue)").unwrap();
        assert_eq!(
            g.orientation,
            LinearOrientation::Directional("bottom".to_string())
        );
        assert_eq!(g.stops.len(), 2);
        assert!(!g.repeating);
    }

    #[test]
    fn linear_to_right_top() {
        let g = parse_linear_gradient("linear-gradient(to right top, red, blue)").unwrap();
        assert_eq!(
            g.orientation,
            LinearOrientation::Directional("right top".to_string())
        );
    }

    #[test]
    fn linear_angular() {
        let g = parse_linear_gradient("linear-gradient(45deg, red, blue)").unwrap();
        assert_eq!(
            g.orientation,
            LinearOrientation::Angular {
                value: "45".to_string(),
                unit: "deg".to_string()
            }
        );
    }

    #[test]
    fn linear_repeating_with_pct() {
        let g = parse_linear_gradient("repeating-linear-gradient(30deg, red, blue 30%)").unwrap();
        assert!(g.repeating);
        assert_eq!(g.stops[1].offset.as_ref().unwrap().unit, "%");
    }

    #[test]
    fn linear_rgba_color_stops() {
        let g = parse_linear_gradient(
            "linear-gradient(45deg, rgba(255, 0, 0, 0), blue)",
        )
        .unwrap();
        assert_eq!(g.stops.len(), 2);
        assert_eq!(g.stops[0].color, "rgba(255, 0, 0, 0)");
    }

    #[test]
    fn radial_default() {
        let g = parse_radial_gradient("radial-gradient(blue, red)").unwrap();
        assert_eq!(g.shape, "ellipse");
        assert!(matches!(
            g.position.x,
            RadialPropertyValue::Keyword(ref k) if k == "center"
        ));
        assert_eq!(g.stops.len(), 2);
    }

    #[test]
    fn radial_circle_at_pixels() {
        let g = parse_radial_gradient("radial-gradient(circle at 25px 25px, blue, red)").unwrap();
        assert_eq!(g.shape, "circle");
        match &g.position.x {
            RadialPropertyValue::Length(o) => {
                assert_eq!(o.value, "25");
                assert_eq!(o.unit, "px");
            }
            _ => panic!("expected length"),
        }
    }

    #[test]
    fn radial_at_only() {
        let g =
            parse_radial_gradient("radial-gradient(at 3% 42%, rgb(228, 105, 236) 0px, transparent 50%)")
                .unwrap();
        assert_eq!(g.shape, "ellipse");
        match &g.position.x {
            RadialPropertyValue::Length(o) => assert_eq!(o.unit, "%"),
            _ => panic!(),
        }
    }

    #[test]
    fn conic_default_prelude() {
        let g = parse_conic_gradient("conic-gradient(red, blue)").unwrap();
        assert!(!g.repeating);
        assert_eq!(g.angle, "0deg");
        assert_eq!(g.position, "center");
        assert_eq!(g.stops.len(), 2);
        assert_eq!(g.stops[0].color, "red");
        assert_eq!(g.stops[1].color, "blue");
    }

    #[test]
    fn conic_expand_two_position_stops() {
        let g =
            parse_conic_gradient("conic-gradient(red 0% 25%, blue 25% 75%, green 75% 100%)").unwrap();
        let cs: Vec<(&str, &str)> = g
            .stops
            .iter()
            .map(|s| {
                let o = s.offset.as_ref().unwrap();
                (s.color.as_str(), o.value.as_str())
            })
            .collect();
        assert_eq!(
            cs,
            vec![
                ("red", "0"),
                ("red", "25"),
                ("blue", "25"),
                ("blue", "75"),
                ("green", "75"),
                ("green", "100"),
            ]
        );
    }

    #[test]
    fn conic_from_and_at() {
        let g = parse_conic_gradient(
            "conic-gradient(from 45deg at 75% 75%, red 0% 33%, green 33% 66%, blue 66% 100%)",
        )
        .unwrap();
        assert_eq!(g.angle, "45deg");
        assert_eq!(g.position, "75% 75%");
        assert_eq!(g.stops.len(), 6);
    }

    #[test]
    fn conic_repeating() {
        let g = parse_conic_gradient("repeating-conic-gradient(red 0deg, blue 30deg)").unwrap();
        assert!(g.repeating);
        assert_eq!(g.stops.len(), 2);
        let off = g.stops[1].offset.as_ref().unwrap();
        assert_eq!(off.value, "30");
        assert_eq!(off.unit, "deg");
    }

    #[test]
    fn conic_with_hints() {
        let g = parse_conic_gradient(
            "conic-gradient(from 0.25turn at 50% 30%, #f69d3c, 10deg, #3f87a6, 350deg, #ebf8e1)",
        )
        .unwrap();
        assert_eq!(g.angle, "0.25turn");
        assert_eq!(g.position, "50% 30%");
        assert_eq!(g.stops.len(), 3);
        assert_eq!(g.stops[0].color, "#f69d3c");
        assert_eq!(g.stops[0].hint.as_ref().unwrap().value, "10");
        assert_eq!(g.stops[0].hint.as_ref().unwrap().unit, "deg");
        assert_eq!(g.stops[1].color, "#3f87a6");
        assert_eq!(g.stops[1].hint.as_ref().unwrap().value, "350");
        assert_eq!(g.stops[2].color, "#ebf8e1");
        assert!(g.stops[2].hint.is_none());
    }

    #[test]
    fn conic_rgba_color_stops() {
        let g = parse_conic_gradient(
            "conic-gradient(rgba(255,0,0,0.8) 0% 50%, rgba(0,0,255,0.3) 50% 100%)",
        )
        .unwrap();
        assert_eq!(g.stops.len(), 4);
        assert_eq!(g.stops[0].color, "rgba(255,0,0,0.8)");
        assert_eq!(g.stops[0].offset.as_ref().unwrap().value, "0");
        assert_eq!(g.stops[2].color, "rgba(0,0,255,0.3)");
    }
}
