//! Port of `buildXMLString` from `src/utils.ts`.
//!
//! JS:
//! ```js
//! function buildXMLString(type, attrs, children) {
//!   let attrString = ''
//!   for (const [k, _v] of Object.entries(attrs)) {
//!     if (typeof _v !== 'undefined') attrString += ` ${k}="${_v}"`
//!   }
//!   if (children) return `<${type}${attrString}>${children}</${type}>`
//!   return `<${type}${attrString}/>`
//! }
//! ```
//!
//! Important: we use an ordered list of attributes (`Vec`) because attribute
//! order in the emitted SVG matters for byte-for-byte / pixel-for-pixel
//! reproducibility (resvg doesn't care about order, but the upstream
//! `buildXMLString` preserves Object insertion order).
//!
//! JS semantics: any undefined entry is silently dropped. We model the
//! "undefined" case as `AttrValue::Skip`.

pub enum AttrValue<'a> {
    Str(&'a str),
    Owned(String),
    Number(f32),
    NumberF64(f64),
    Int(i64),
    Skip,
}

impl<'a> From<&'a str> for AttrValue<'a> {
    fn from(s: &'a str) -> Self { AttrValue::Str(s) }
}
impl<'a> From<String> for AttrValue<'a> {
    fn from(s: String) -> Self { AttrValue::Owned(s) }
}
impl<'a> From<f32> for AttrValue<'a> {
    fn from(n: f32) -> Self { AttrValue::Number(n) }
}
impl<'a> From<i64> for AttrValue<'a> {
    fn from(n: i64) -> Self { AttrValue::Int(n) }
}
impl<'a> From<u32> for AttrValue<'a> {
    fn from(n: u32) -> Self { AttrValue::Int(n as i64) }
}
impl<'a, T> From<Option<T>> for AttrValue<'a>
where
    T: Into<AttrValue<'a>>,
{
    fn from(o: Option<T>) -> Self {
        match o { Some(v) => v.into(), None => AttrValue::Skip }
    }
}

/// JS `${value}` coerces numbers using ECMAScript's `ToString`, which:
///   - drops trailing zeros: `1.0` -> `"1"`, `1.5` -> `"1.5"`
///   - uses scientific notation only for very small/large magnitudes.
///
/// For the resolution range we care about (a few thousand px) we can just
/// pretty-print and trim.
pub fn js_number_to_string(n: f32) -> String {
    // IMPORTANT: do NOT widen via `n as f64` — that turns the f32 literal
    // `0.6` into `0.6000000238418579`. JS numbers are already f64 at the
    // source, but our ComputedStyle stores f32; we use f32's shortest
    // round-trippable formatter (Ryu) directly.
    if n.is_finite() && n == n.trunc() && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    if !n.is_finite() {
        if n.is_nan() { return "NaN".to_string(); }
        return if n > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    // Rust's default Display for f32 uses the shortest round-trippable
    // representation (same Ryu algorithm V8 uses for Number.toString),
    // which gives "0.6" for f32 0.6.
    let s = format!("{}", n);
    s
}

/// f64 variant used by paths that need to match JS satori's double-precision
/// math byte-for-byte (e.g. gradient geometry whose sqrt / tan output is
/// stringified directly into SVG attribute values).
///
/// Mirrors ECMA-262 `Number.prototype.toString` for finite values:
/// - exact-integer doubles (`1.0`) drop trailing `.0` → `"1"`
/// - very small values (`|x| < 1e-6` and nonzero) switch to scientific
///   notation (`1e-7`, `6.123233995736766e-17`) — matches the JS
///   "compact" form the SVG output relies on
/// - very large values (`|x| >= 1e21`) likewise switch, with an explicit
///   `e+` sign in the exponent (`1e+21`).
/// - everything else falls through to Rust's default `Display` for f64,
///   which uses the same Grisu/Ryu shortest round-trippable digits as
///   V8's NumberToString.
pub fn js_number_to_string_f64(n: f64) -> String {
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
    let abs = n.abs();
    if abs < 1e-6 {
        // Rust's `{:e}` already produces JS-compatible output for the
        // small case (`-1.4210854715202004e-14`, `6.123233995736766e-17`),
        // including dropping `1.0e-` → `1e-` and using a `-` (not `-+`)
        // sign on the exponent.
        return format!("{:e}", n);
    }
    if abs >= 1e21 {
        // JS emits `1e+21`, Rust's `{:e}` emits `1e21`. Splice the `+`
        // in so the sign on the exponent matches JS exactly.
        let s = format!("{:e}", n);
        if let Some(idx) = s.find('e') {
            let (mantissa, exp) = s.split_at(idx);
            // exp starts with `e`; if the next char isn't `-` or `+`,
            // it's a positive bare digit run — insert `+`.
            let rest = &exp[1..];
            if !rest.starts_with('-') && !rest.starts_with('+') {
                return format!("{mantissa}e+{rest}");
            }
        }
        return s;
    }
    format!("{n}")
}

pub fn build_xml<'a>(tag: &str, attrs: &[(&str, AttrValue<'a>)], children: Option<&str>) -> String {
    let mut out = String::with_capacity(32);
    out.push('<');
    out.push_str(tag);
    for (k, v) in attrs {
        match v {
            AttrValue::Skip => continue,
            AttrValue::Str(s) => {
                out.push(' ');
                out.push_str(k);
                out.push('=');
                out.push('"');
                out.push_str(s);
                out.push('"');
            }
            AttrValue::Owned(s) => {
                out.push(' ');
                out.push_str(k);
                out.push('=');
                out.push('"');
                out.push_str(s);
                out.push('"');
            }
            AttrValue::Number(n) => {
                out.push(' ');
                out.push_str(k);
                out.push('=');
                out.push('"');
                out.push_str(&js_number_to_string(*n));
                out.push('"');
            }
            AttrValue::NumberF64(n) => {
                out.push(' ');
                out.push_str(k);
                out.push('=');
                out.push('"');
                out.push_str(&js_number_to_string_f64(*n));
                out.push('"');
            }
            AttrValue::Int(n) => {
                out.push(' ');
                out.push_str(k);
                out.push('=');
                out.push('"');
                out.push_str(&n.to_string());
                out.push('"');
            }
        }
    }
    match children {
        Some(c) if !c.is_empty() => {
            out.push('>');
            out.push_str(c);
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
        _ => out.push_str("/>"),
    }
    out
}
