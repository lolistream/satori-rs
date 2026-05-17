//! Port of `src/handler/preprocess.ts::SVGNodeToImage` plus
//! `translateSVGNodeToSVGString`, and the `type === 'svg'` branch in
//! `src/handler/compute.ts`.
//!
//! When the input JSX tree contains an inline `<svg>...</svg>` element,
//! JS satori serializes the entire subtree back into an SVG document,
//! URL-encodes special characters, and prefixes
//! `data:image/svg+xml;utf8,`. The result is treated as if the user had
//! written `<img src="data:image/svg+xml;utf8,...">` — i.e. it goes
//! through the same `<image>` rendering path. To match the JS output
//! byte-for-byte we reproduce the (slightly buggy) JS code:
//!
//! - Attribute insertion order: original `restProps` first (in source
//!   order), then the post-destructure assignments `xmlns`, `width`,
//!   `height`, `viewBox`. Because the JS conditionals evaluate
//!   `width = (width || (ratio && height)) ? height/ratio : null` we
//!   sometimes inject `width="null"`, `width="NaN"`, or
//!   `width="Infinity"` literally. We faithfully reproduce these to
//!   keep snapshots stable.
//! - `currentColor` propagation: each child sees the inherited color
//!   substituted into any case-insensitive `currentcolor` attribute
//!   value. The current-color value is read from the SVG node's own
//!   `style.color` when present, otherwise inherits from the parent
//!   element.
//! - URL encoding: a small fixed alphabet of characters
//!   (`/[\r\n%#()<>?[\\\]^`{|}"']/`) is encoded via
//!   `encodeURIComponent`; everything else (notably spaces) passes
//!   through untouched.

use serde_json::Value;

/// Port of the `ATTRIBUTE_MAPPING` table in `src/handler/preprocess.ts`.
fn map_attr_name(name: &str) -> String {
    match name {
        "accentHeight" => "accent-height".into(),
        "alignmentBaseline" => "alignment-baseline".into(),
        "arabicForm" => "arabic-form".into(),
        "baselineShift" => "baseline-shift".into(),
        "capHeight" => "cap-height".into(),
        "clipPath" => "clip-path".into(),
        "clipRule" => "clip-rule".into(),
        "colorInterpolation" => "color-interpolation".into(),
        "colorInterpolationFilters" => "color-interpolation-filters".into(),
        "colorProfile" => "color-profile".into(),
        "colorRendering" => "color-rendering".into(),
        "dominantBaseline" => "dominant-baseline".into(),
        "enableBackground" => "enable-background".into(),
        "fillOpacity" => "fill-opacity".into(),
        "fillRule" => "fill-rule".into(),
        "floodColor" => "flood-color".into(),
        "floodOpacity" => "flood-opacity".into(),
        "fontFamily" => "font-family".into(),
        "fontSize" => "font-size".into(),
        "fontSizeAdjust" => "font-size-adjust".into(),
        "fontStretch" => "font-stretch".into(),
        "fontStyle" => "font-style".into(),
        "fontVariant" => "font-variant".into(),
        "fontWeight" => "font-weight".into(),
        "glyphName" => "glyph-name".into(),
        "glyphOrientationHorizontal" => "glyph-orientation-horizontal".into(),
        "glyphOrientationVertical" => "glyph-orientation-vertical".into(),
        "horizAdvX" => "horiz-adv-x".into(),
        "horizOriginX" => "horiz-origin-x".into(),
        "href" => "href".into(),
        "imageRendering" => "image-rendering".into(),
        "letterSpacing" => "letter-spacing".into(),
        "lightingColor" => "lighting-color".into(),
        "markerEnd" => "marker-end".into(),
        "markerMid" => "marker-mid".into(),
        "markerStart" => "marker-start".into(),
        "overlinePosition" => "overline-position".into(),
        "overlineThickness" => "overline-thickness".into(),
        "paintOrder" => "paint-order".into(),
        "panose1" => "panose-1".into(),
        "pointerEvents" => "pointer-events".into(),
        "renderingIntent" => "rendering-intent".into(),
        "shapeRendering" => "shape-rendering".into(),
        "stopColor" => "stop-color".into(),
        "stopOpacity" => "stop-opacity".into(),
        "strikethroughPosition" => "strikethrough-position".into(),
        "strikethroughThickness" => "strikethrough-thickness".into(),
        "strokeDasharray" => "stroke-dasharray".into(),
        "strokeDashoffset" => "stroke-dashoffset".into(),
        "strokeLinecap" => "stroke-linecap".into(),
        "strokeLinejoin" => "stroke-linejoin".into(),
        "strokeMiterlimit" => "stroke-miterlimit".into(),
        "strokeOpacity" => "stroke-opacity".into(),
        "strokeWidth" => "stroke-width".into(),
        "textAnchor" => "text-anchor".into(),
        "textDecoration" => "text-decoration".into(),
        "textRendering" => "text-rendering".into(),
        "underlinePosition" => "underline-position".into(),
        "underlineThickness" => "underline-thickness".into(),
        "unicodeBidi" => "unicode-bidi".into(),
        "unicodeRange" => "unicode-range".into(),
        "unitsPerEm" => "units-per-em".into(),
        "vAlphabetic" => "v-alphabetic".into(),
        "vHanging" => "v-hanging".into(),
        "vIdeographic" => "v-ideographic".into(),
        "vMathematical" => "v-mathematical".into(),
        "vectorEffect" => "vector-effect".into(),
        "vertAdvY" => "vert-adv-y".into(),
        "vertOriginX" => "vert-origin-x".into(),
        "vertOriginY" => "vert-origin-y".into(),
        "wordSpacing" => "word-spacing".into(),
        "writingMode" => "writing-mode".into(),
        "xHeight" => "x-height".into(),
        "xlinkActuate" => "xlink:actuate".into(),
        "xlinkArcrole" => "xlink:arcrole".into(),
        "xlinkHref" => "xlink:href".into(),
        "xlinkRole" => "xlink:role".into(),
        "xlinkShow" => "xlink:show".into(),
        "xlinkTitle" => "xlink:title".into(),
        "xlinkType" => "xlink:type".into(),
        "xmlBase" => "xml:base".into(),
        "xmlLang" => "xml:lang".into(),
        "xmlSpace" => "xml:space".into(),
        "xmlnsXlink" => "xmlns:xlink".into(),
        _ => name.to_string(),
    }
}

/// Port of `midline` in `src/utils.ts`: camelCase → kebab-case.
fn midline(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for ch in s.chars() {
        if ch.is_ascii_uppercase() {
            out.push('-');
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// Encode the SVGSymbols character class via `encodeURIComponent`.
/// Mirrors the JS regex `/[\r\n%#()<>?[\\\]^`{|}"']/g` exactly — note
/// that spaces are NOT encoded.
fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 2);
    for ch in s.chars() {
        let needs_encoding = matches!(
            ch,
            '\r' | '\n'
                | '%'
                | '#'
                | '('
                | ')'
                | '<'
                | '>'
                | '?'
                | '['
                | ']'
                | '\\'
                | '^'
                | '`'
                | '{'
                | '|'
                | '}'
                | '"'
                | '\''
        );
        if needs_encoding {
            // encodeURIComponent emits %XX with uppercase hex digits.
            let mut buf = [0u8; 4];
            for &byte in ch.encode_utf8(&mut buf).as_bytes() {
                out.push_str(&format!("%{:02X}", byte));
            }
        } else {
            out.push(ch);
        }
    }
    out
}

/// JS template-literal coercion of a value: `${v}` for null/NaN/Infinity/
/// numbers/strings/booleans. The only quirks vs `Display::to_string` are
/// the special floats and the integer-valued doubles dropping `.0`.
fn js_stringify(v: &SvgVal) -> String {
    match v {
        SvgVal::Undefined => "undefined".to_string(),
        SvgVal::Null => "null".to_string(),
        SvgVal::NaN => "NaN".to_string(),
        SvgVal::Infinity => "Infinity".to_string(),
        SvgVal::NegInfinity => "-Infinity".to_string(),
        SvgVal::Bool(b) => b.to_string(),
        SvgVal::Number(n) => js_number_to_string(*n),
        SvgVal::Str(s) => s.clone(),
    }
}

/// Mirrors ECMA-262 `Number.prototype.toString` for the finite range we
/// hit here.
fn js_number_to_string(n: f64) -> String {
    if !n.is_finite() {
        return if n.is_nan() {
            "NaN".to_string()
        } else if n > 0.0 {
            "Infinity".to_string()
        } else {
            "-Infinity".to_string()
        };
    }
    if n == 0.0 {
        return "0".to_string();
    }
    if n == n.trunc() && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    format!("{n}")
}

/// JS-flavored value: distinguishes the quirky null/NaN/Infinity cases
/// that the JS code can produce when conditionals fall through, plus
/// `Undefined` (= JS `undefined`, distinct from `null` in arithmetic
/// because `Number(undefined) === NaN` while `Number(null) === 0`).
#[derive(Debug, Clone)]
enum SvgVal {
    Undefined,
    Null,
    NaN,
    Infinity,
    NegInfinity,
    Bool(bool),
    Number(f64),
    Str(String),
}

impl SvgVal {
    fn from_value(v: &Value) -> Self {
        match v {
            Value::Null => SvgVal::Null,
            Value::Bool(b) => SvgVal::Bool(*b),
            Value::Number(n) => SvgVal::Number(n.as_f64().unwrap_or(f64::NAN)),
            Value::String(s) => SvgVal::Str(s.clone()),
            _ => SvgVal::Str(v.to_string()),
        }
    }

    fn is_truthy(&self) -> bool {
        match self {
            SvgVal::Undefined | SvgVal::Null | SvgVal::NaN => false,
            SvgVal::Bool(false) => false,
            SvgVal::Number(n) if *n == 0.0 => false,
            SvgVal::Str(s) if s.is_empty() => false,
            _ => true,
        }
    }

    fn as_string_lower(&self) -> Option<String> {
        match self {
            SvgVal::Str(s) => Some(s.to_ascii_lowercase()),
            _ => None,
        }
    }

    fn as_f64(&self) -> f64 {
        match self {
            SvgVal::Number(n) => *n,
            SvgVal::NaN | SvgVal::Undefined => f64::NAN,
            SvgVal::Infinity => f64::INFINITY,
            SvgVal::NegInfinity => f64::NEG_INFINITY,
            SvgVal::Str(s) => s.parse::<f64>().unwrap_or(f64::NAN),
            SvgVal::Bool(true) => 1.0,
            SvgVal::Bool(false) => 0.0,
            SvgVal::Null => 0.0,
        }
    }
}

/// Result of parsing the JSX `viewBox` (or `viewbox`) prop into four
/// numbers. Returns `None` if absent or malformed (mirrors JS
/// `parseViewBox`).
fn parse_view_box(v: Option<&Value>) -> Option<[f64; 4]> {
    let s = v.and_then(|x| x.as_str())?;
    let nums: Vec<f64> = s
        .split(|c: char| c == ',' || c.is_ascii_whitespace())
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.parse::<f64>().ok())
        .collect();
    if nums.len() != 4 {
        return None;
    }
    Some([nums[0], nums[1], nums[2], nums[3]])
}

/// Minimal port of `lengthToNumber(value, fontSize, baseLength,
/// inheritedStyle, percentageAware)` from `src/utils.ts` — only the
/// length forms used by the test fixtures (`"40"`, `"40px"`, `"2em"`).
/// Returns `None` for unsupported / unparseable inputs.
fn length_to_number(value: &Value, font_size: f64) -> Option<f64> {
    if let Some(n) = value.as_f64() {
        return Some(n);
    }
    let s = value.as_str()?;
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(p) = s.strip_suffix("rem") {
        return p.parse::<f64>().ok().map(|n| n * 16.0);
    }
    if let Some(p) = s.strip_suffix("em") {
        return p.parse::<f64>().ok().map(|n| n * font_size);
    }
    if let Some(p) = s.strip_suffix("px") {
        return p.parse::<f64>().ok();
    }
    if s.ends_with('%') {
        // Percentages aren't resolved by lengthToNumber in the SVG path.
        return None;
    }
    s.parse::<f64>().ok()
}

/// Resolve the `(width, height)` that should land on `style.width` and
/// `style.height` for the JSX `<svg>` node, matching the
/// `type === 'svg'` branch in `src/handler/compute.ts`.
pub fn compute_svg_size(
    props: &serde_json::Map<String, Value>,
    font_size: f64,
) -> (Option<f64>, Option<f64>) {
    let view_box = props.get("viewBox").or_else(|| props.get("viewbox"));
    let view_box_size = parse_view_box(view_box);
    let ratio: Option<f64> = match view_box_size {
        Some([_, _, w, h]) if w != 0.0 => Some(h / w),
        _ => None,
    };

    let raw_width = props.get("width");
    let raw_height = props.get("height");

    let width_defined = raw_width.is_some_and(|v| !v.is_null());
    let height_defined = raw_height.is_some_and(|v| !v.is_null());

    let mut width_val: Option<f64> = None;
    let mut height_val: Option<f64> = None;

    if !width_defined && height_defined {
        match ratio {
            None => {
                width_val = Some(0.0);
                height_val = length_to_number(raw_height.unwrap(), font_size);
            }
            Some(r) => {
                let h_str = raw_height.unwrap();
                if let Some(s) = h_str.as_str() {
                    if s.ends_with('%') {
                        if let Some(parsed) = parse_int_prefix(s) {
                            width_val = Some(parsed / r);
                        }
                    } else if let Some(h_num) = length_to_number(h_str, font_size) {
                        height_val = Some(h_num);
                        width_val = Some(h_num / r);
                    }
                } else if let Some(h_num) = h_str.as_f64() {
                    height_val = Some(h_num);
                    width_val = Some(h_num / r);
                }
            }
        }
    } else if !height_defined && width_defined {
        match ratio {
            None => {
                width_val = Some(0.0);
                height_val = None;
            }
            Some(r) => {
                let w_str = raw_width.unwrap();
                if let Some(s) = w_str.as_str() {
                    if s.ends_with('%') {
                        if let Some(parsed) = parse_int_prefix(s) {
                            height_val = Some(parsed * r);
                        }
                    } else if let Some(w_num) = length_to_number(w_str, font_size) {
                        width_val = Some(w_num);
                        height_val = Some(w_num * r);
                    }
                } else if let Some(w_num) = w_str.as_f64() {
                    width_val = Some(w_num);
                    height_val = Some(w_num * r);
                }
            }
        }
    } else {
        if let Some(w) = raw_width {
            width_val = length_to_number(w, font_size).or_else(|| w.as_f64());
        }
        if let Some(h) = raw_height {
            height_val = length_to_number(h, font_size).or_else(|| h.as_f64());
        }
        // `width ||= viewBoxSize?.[2]; height ||= viewBoxSize?.[3]`
        if width_val.unwrap_or(0.0) == 0.0 {
            if let Some([_, _, w, _]) = view_box_size {
                width_val = Some(w);
            }
        }
        if height_val.unwrap_or(0.0) == 0.0 {
            if let Some([_, _, _, h]) = view_box_size {
                height_val = Some(h);
            }
        }
    }

    (width_val, height_val)
}

fn parse_int_prefix(s: &str) -> Option<f64> {
    let s = s.trim_start();
    let mut end = 0;
    let bytes = s.as_bytes();
    if matches!(bytes.first(), Some(&b'-') | Some(&b'+')) {
        end = 1;
    }
    while end < bytes.len() && bytes[end].is_ascii_digit() {
        end += 1;
    }
    if end == 0 {
        return None;
    }
    s[..end].parse::<f64>().ok()
}

thread_local! {
    static SVG_VALIDATION_ERROR: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

fn record_svg_validation_error(msg: impl Into<String>) {
    SVG_VALIDATION_ERROR.with(|cell| {
        let mut slot = cell.borrow_mut();
        if slot.is_none() {
            *slot = Some(msg.into());
        }
    });
}

/// Take the first validation error recorded during the most recent
/// `build_data_uri` call, clearing the slot. Callers in
/// `satori::build_node` invoke this immediately after every
/// SVG-handling call to convert the recorded error into
/// `SatoriError::Parse`.
pub fn take_svg_validation_error() -> Option<String> {
    SVG_VALIDATION_ERROR.with(|c| c.borrow_mut().take())
}

/// Build the `data:image/svg+xml;utf8,...` URI for a JSX `<svg>`
/// element. Errors recorded during the recursive walk (e.g.
/// unsupported `<text>` children) can be retrieved with
/// `take_svg_validation_error`.
pub fn build_data_uri(svg_props: &serde_json::Map<String, Value>, inherited_color: &str) -> String {
    let view_box_value: Option<&Value> = svg_props
        .get("viewBox")
        .or_else(|| svg_props.get("viewbox"));
    let view_box_str: Option<String> = view_box_value
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let raw_width = svg_props.get("width");
    let raw_height = svg_props.get("height");
    let style_obj = svg_props.get("style").and_then(|v| v.as_object());

    let current_color = style_obj
        .and_then(|s| s.get("color"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| inherited_color.to_string());

    let view_box_size = parse_view_box(view_box_value);
    let ratio: Option<f64> = view_box_size.and_then(|[_, _, w, h]| {
        if w != 0.0 {
            Some(h / w)
        } else {
            None
        }
    });

    // Mirror JS:
    //   width  = width  || (ratio && height) ? height / ratio : null
    //   height = height || (ratio && width)  ? width  * ratio : null
    // A missing prop is JS `undefined` (NaN in arithmetic), not `null`
    // (which is 0 in arithmetic). Track them separately.
    let width_in: SvgVal = raw_width.map(SvgVal::from_value).unwrap_or(SvgVal::Undefined);
    let height_in: SvgVal = raw_height.map(SvgVal::from_value).unwrap_or(SvgVal::Undefined);

    let width_post = svg_dim_first_pass(&width_in, &height_in, ratio);
    let height_post = svg_dim_second_pass(&height_in, &width_post, ratio);

    let mut rest_keys: Vec<String> = Vec::new();
    for key in svg_props.keys() {
        match key.as_str() {
            "viewBox" | "viewbox" | "width" | "height" | "className" | "style" | "children" => {}
            _ => rest_keys.push(key.clone()),
        }
    }
    for key in ["xmlns", "width", "height", "viewBox"] {
        if !rest_keys.iter().any(|k| k == key) {
            rest_keys.push(key.to_string());
        }
    }

    let mut attrs = String::new();
    for key in &rest_keys {
        let raw = match key.as_str() {
            "xmlns" => SvgVal::Str("http://www.w3.org/2000/svg".to_string()),
            "width" => width_post.clone(),
            "height" => height_post.clone(),
            "viewBox" => match &view_box_str {
                Some(s) => SvgVal::Str(s.clone()),
                None => continue,
            },
            _ => match svg_props.get(key) {
                Some(v) => SvgVal::from_value(v),
                None => continue,
            },
        };
        let final_value = if let Some(lower) = raw.as_string_lower() {
            if lower == "currentcolor" {
                SvgVal::Str(current_color.clone())
            } else {
                raw
            }
        } else {
            raw
        };
        let mapped = map_attr_name(key);
        attrs.push(' ');
        attrs.push_str(&mapped);
        attrs.push_str("=\"");
        attrs.push_str(&js_stringify(&final_value));
        attrs.push('"');
    }

    let children_xml = match svg_props.get("children") {
        Some(child) => translate_node(child, &current_color),
        None => String::new(),
    };

    // JS: `<svg ${attrs}>${children}</svg>`. There's a literal space
    // between `<svg` and the attrs, then attrs already start with a
    // space, so the rendered string starts with two spaces (matches the
    // JS reference output).
    let inner = format!("<svg {attrs}>{children_xml}</svg>");
    let encoded = url_encode(&inner);
    format!("data:image/svg+xml;utf8,{encoded}")
}

/// `width = (width || (ratio && height)) ? height/ratio : null`.
///
/// `ratio = None` represents JS `null` (`Number(null) === 0`), so we
/// coerce it to `0.0` in the arithmetic branch to reproduce
/// `30 / null === Infinity`. This is distinct from the missing-prop /
/// `undefined` path, which feeds `NaN` directly via `SvgVal::Undefined`.
fn svg_dim_first_pass(width: &SvgVal, height: &SvgVal, ratio: Option<f64>) -> SvgVal {
    let ratio_v = ratio.map(SvgVal::Number).unwrap_or(SvgVal::Null);
    let r_and_h = if ratio_v.is_truthy() {
        height.clone()
    } else {
        ratio_v
    };
    let condition = if width.is_truthy() {
        width.clone()
    } else {
        r_and_h
    };
    if condition.is_truthy() {
        let h_num = height.as_f64();
        let r = ratio.unwrap_or(0.0);
        finite_to_svgval(h_num / r)
    } else {
        SvgVal::Null
    }
}

/// `height = (height || (ratio && width)) ? width*ratio : null`.
fn svg_dim_second_pass(height: &SvgVal, width: &SvgVal, ratio: Option<f64>) -> SvgVal {
    let ratio_v = ratio.map(SvgVal::Number).unwrap_or(SvgVal::Null);
    let r_and_w = if ratio_v.is_truthy() {
        width.clone()
    } else {
        ratio_v
    };
    let condition = if height.is_truthy() {
        height.clone()
    } else {
        r_and_w
    };
    if condition.is_truthy() {
        let w_num = width.as_f64();
        let r = ratio.unwrap_or(0.0);
        finite_to_svgval(w_num * r)
    } else {
        SvgVal::Null
    }
}

fn finite_to_svgval(n: f64) -> SvgVal {
    if n.is_nan() {
        SvgVal::NaN
    } else if n == f64::INFINITY {
        SvgVal::Infinity
    } else if n == f64::NEG_INFINITY {
        SvgVal::NegInfinity
    } else {
        SvgVal::Number(n)
    }
}

/// Recursively serialize a JSX subtree as an SVG fragment, mirroring
/// `translateSVGNodeToSVGString`. `inherited_color` is the
/// post-`style?.color` value used to substitute `currentColor`.
fn translate_node(node: &Value, inherited_color: &str) -> String {
    match node {
        Value::Null => String::new(),
        Value::Bool(_) => String::new(),
        Value::Array(items) => items
            .iter()
            .map(|i| translate_node(i, inherited_color))
            .collect(),
        Value::String(s) => s.clone(),
        Value::Number(n) => js_number_to_string(n.as_f64().unwrap_or(0.0)),
        Value::Object(map) => {
            let type_name = map.get("type").and_then(|v| v.as_str()).unwrap_or("");
            // `<text>` is explicitly rejected by JS satori
            // (`translateSVGNodeToSVGString` throws). Record the
            // error in the thread-local stash so build_node can
            // surface it.
            if type_name == "text" {
                record_svg_validation_error(
                    "<text> nodes are not currently supported inside <svg>",
                );
                return String::new();
            }
            let props = map.get("props").and_then(|v| v.as_object());
            let Some(props) = props else {
                return format!("<{type_name}></{type_name}>");
            };

            let style_obj = props.get("style").and_then(|v| v.as_object());
            let current_color = style_obj
                .and_then(|s| s.get("color"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| inherited_color.to_string());

            let mut attrs = String::new();
            for (key, value) in props {
                if key == "children" || key == "style" {
                    continue;
                }
                let raw = SvgVal::from_value(value);
                let final_value = if let Some(lower) = raw.as_string_lower() {
                    if lower == "currentcolor" {
                        SvgVal::Str(current_color.clone())
                    } else {
                        raw
                    }
                } else {
                    raw
                };
                let mapped = map_attr_name(key);
                attrs.push(' ');
                attrs.push_str(&mapped);
                attrs.push_str("=\"");
                attrs.push_str(&js_stringify(&final_value));
                attrs.push('"');
            }

            let mut style_attr = String::new();
            if let Some(s) = style_obj {
                if !s.is_empty() {
                    style_attr.push_str(" style=\"");
                    let mut first = true;
                    for (k, v) in s {
                        if !first {
                            style_attr.push(';');
                        }
                        first = false;
                        style_attr.push_str(&midline(k));
                        style_attr.push(':');
                        style_attr.push_str(&js_stringify(&SvgVal::from_value(v)));
                    }
                    style_attr.push('"');
                }
            }

            let children = match props.get("children") {
                Some(c) => translate_node(c, &current_color),
                None => String::new(),
            };

            format!("<{type_name}{attrs}{style_attr}>{children}</{type_name}>")
        }
    }
}
