//! Build SVG path-data strings byte-identical to
//! `@shuding/opentype.js`'s `Glyph.getPath(x, y, fontSize).toPathData(1)`
//! output.
//!
//! Mirrors the two-pronged JS pipeline used by `src/font.ts:getSVG`:
//!
//! 1. `Glyph.getPath(x, y, fontSize)` transforms each font-unit command:
//!    - `x_out = x + cmd.x * scale`
//!    - `y_out = y - cmd.y * scale`   (note Y is flipped — TrueType
//!      coordinates have Y up, SVG paths have Y down)
//!   where `scale = (1 / unitsPerEm) * fontSize`.
//!
//! 2. `Path.toPathData(decimalPlaces=1)` formats each coordinate using
//!    `floatToString(v)`:
//!    - if `Math.round(v) === v` (i.e. v is an integer): emit `'' + Math.round(v)`
//!    - else: emit `v.toFixed(decimalPlaces)`
//!    and packs values with `' '` only when `v >= 0 && i > 0` — so a
//!    negative value's `-` sign acts as its own separator.
//!
//! Commands are emitted as `M`, `L`, `Q`, `C`, `Z` (uppercase) per the
//! JS code path. `ttf-parser`'s `OutlineBuilder` callbacks line up
//! exactly: `move_to`, `line_to`, `quad_to`, `curve_to`, `close`.

use ttf_parser::OutlineBuilder;

/// Mirror of `opentype.js/src/path.js:floatToString(v)`:
///
/// ```js
/// if (Math.round(v) === v) return '' + Math.round(v)
/// else return v.toFixed(decimalPlaces)
/// ```
///
/// `Math.round(v) === v` is true iff `v` is an integer (or ±0). For
/// integer values JS collapses `-0` to `"0"` via the implicit string
/// conversion, so we cast through `i64` to do the same.
pub(crate) fn js_float_to_string(v: f64, dp: u32) -> String {
    if v.is_finite() && v == v.trunc() {
        let n = v as i64;
        return n.to_string();
    }
    js_to_fixed(v, dp)
}

/// Mirror of `Number.prototype.toFixed(dp)` (ECMA-262 §21.1.3.3).
///
/// The spec picks integer `n` such that `|n/10^dp - v|` is minimal
/// using EXACT arithmetic; ties go to the larger `n` (round half
/// toward +∞). For negative values it operates on `|v|` and prepends
/// `-`, which is why `(-0.04).toFixed(1) === "-0.0"` even though `0`
/// itself is unsigned.
///
/// Rust's `{:.dp}` uses round-half-to-even (banker's rounding) which
/// disagrees with JS only in the EXACT-tie case (e.g. `0.25` → JS
/// "0.3", Rust "0.2"). We use Rust's exact rounding to get the
/// correct "round to nearest decimal" answer for the common case,
/// then specifically bump the result up by 1 ULP when:
///   - the original `v * 10^dp` is exactly a half-integer (IEEE `*.5`)
///   - and Rust's banker's rounding picked the lower (even) neighbor
///
/// For inputs in the format `v = k / 2^n + integer` (which covers
/// every path coordinate we emit — glyph-units divided by an integer
/// `unitsPerEm`, offset by an integer baseline), the IEEE
/// multiplication by 10^dp is exact, so the half-integer check is
/// reliable and the result is byte-identical to JS's `toFixed`.
pub(crate) fn js_to_fixed(v: f64, dp: u32) -> String {
    if !v.is_finite() {
        return format!("{v}");
    }
    let neg = v < 0.0;
    let abs = v.abs();

    // JS `Number.prototype.toFixed(dp)` rounds the EXACT mathematical
    // value of the f64 to the nearest decimal candidate with `dp`
    // fractional digits, breaking true ties by picking the larger
    // candidate. The f64 representation of e.g. `20.95` is actually
    // `20.94999999...`, so JS rounds it DOWN to `"20.9"` despite the
    // decimal literal looking like a tie. Rust's `format!("{:.dp}")`
    // banker-rounds, which agrees with JS on the not-tie case but
    // disagrees at exact f64 ties (e.g. `0.25` → Rust "0.2", JS
    // "0.3").
    //
    // To reproduce JS behavior exactly, format `abs` to enough decimal
    // places to expose the exact f64 value (50 digits is more than
    // sufficient; an f64 has at most ~17 significant digits), then
    // apply round-half-up at position `dp` based on the EXACT digit
    // string.
    let high = format!("{abs:.50}");
    let bytes = high.as_bytes();
    let dot = bytes.iter().position(|&b| b == b'.').unwrap_or(bytes.len());

    // Truncate / pad the fractional part to `dp` digits, then decide
    // whether to round up by inspecting the next digit and the tail.
    let int_part = &bytes[..dot];
    let frac_part = if dot + 1 < bytes.len() { &bytes[dot + 1..] } else { &[][..] };

    // Build the "decimal integer" representation of `abs * 10^dp`
    // truncated. Then add 1 if round-up.
    let mut digits: Vec<u8> = Vec::with_capacity(int_part.len() + dp as usize);
    digits.extend_from_slice(int_part);
    for i in 0..dp as usize {
        digits.push(*frac_part.get(i).unwrap_or(&b'0'));
    }

    let next_digit = frac_part.get(dp as usize).copied().unwrap_or(b'0');
    let round_up = match next_digit {
        b'0'..=b'4' => false,
        b'6'..=b'9' => true,
        // next digit is '5' or beyond; we have a true tie iff every
        // subsequent digit is '0'.
        b'5' => {
            // Round up if any subsequent digit is non-zero (more than
            // half) OR if all are zero (exact tie → JS picks larger).
            // Either way: round up.
            let _tail = &frac_part[(dp as usize + 1).min(frac_part.len())..];
            // `_tail` only matters if we wanted to handle a hypothetical
            // round-half-to-even or rejected-tie variant; JS picks the
            // larger candidate in both sub-cases, so always round up.
            true
        }
        _ => false,
    };

    if round_up {
        // In-place increment with carry.
        let mut i = digits.len();
        let mut carry = 1u8;
        while i > 0 && carry > 0 {
            i -= 1;
            let d = digits[i] + carry;
            if d > b'9' {
                digits[i] = b'0';
                carry = 1;
            } else {
                digits[i] = d;
                carry = 0;
            }
        }
        if carry > 0 {
            digits.insert(0, b'1');
        }
    }

    // Strip leading zeros from the integer part (keep at least one).
    let int_len = digits.len() - dp as usize;
    let int_str = std::str::from_utf8(&digits[..int_len]).unwrap_or("0");
    let int_trimmed = int_str.trim_start_matches('0');
    let int_final = if int_trimmed.is_empty() { "0" } else { int_trimmed };
    let frac_str = std::str::from_utf8(&digits[int_len..]).unwrap_or("");

    let fmt = if dp == 0 {
        int_final.to_string()
    } else {
        format!("{int_final}.{frac_str}")
    };

    if neg {
        format!("-{fmt}")
    } else {
        fmt
    }
}


/// `OutlineBuilder` that produces the exact same path-data string as
/// `opentype.js` for a single glyph.
///
/// `pen_x` and `baseline_y` are the glyph's origin in pixels; the
/// glyph's font-unit coordinates are scaled by `scale = font_size /
/// unitsPerEm` and Y is flipped (TrueType / OpenType use Y-up).
pub(crate) struct PathDataBuilder {
    pen_x: f64,
    baseline_y: f64,
    scale: f64,
    pub(crate) out: String,
    decimal_places: u32,
}

impl PathDataBuilder {
    pub(crate) fn new(pen_x: f64, baseline_y: f64, scale: f64, dp: u32) -> Self {
        Self {
            pen_x,
            baseline_y,
            scale,
            out: String::new(),
            decimal_places: dp,
        }
    }

    fn px(&self, x: f32) -> f64 {
        self.pen_x + (x as f64) * self.scale
    }
    fn py(&self, y: f32) -> f64 {
        self.baseline_y - (y as f64) * self.scale
    }

    /// Mirror of `Path.toPathData:packValues(...)`:
    /// joins values with `' '` only when the value is non-negative and
    /// there's a previous value (negative values rely on their `-`
    /// sign as a separator).
    fn pack(&mut self, values: &[f64]) {
        for (i, &v) in values.iter().enumerate() {
            if v >= 0.0 && i > 0 {
                self.out.push(' ');
            }
            self.out.push_str(&js_float_to_string(v, self.decimal_places));
        }
    }
}

impl OutlineBuilder for PathDataBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.out.push('M');
        let (px, py) = (self.px(x), self.py(y));
        self.pack(&[px, py]);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.out.push('L');
        let (px, py) = (self.px(x), self.py(y));
        self.pack(&[px, py]);
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.out.push('Q');
        let (px1, py1) = (self.px(x1), self.py(y1));
        let (px, py) = (self.px(x), self.py(y));
        self.pack(&[px1, py1, px, py]);
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.out.push('C');
        let (px1, py1) = (self.px(x1), self.py(y1));
        let (px2, py2) = (self.px(x2), self.py(y2));
        let (px, py) = (self.px(x), self.py(y));
        self.pack(&[px1, py1, px2, py2, px, py]);
    }
    fn close(&mut self) {
        self.out.push('Z');
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn js_to_fixed_matches_v8() {
        // Cross-checked against Node `(value).toFixed(1)`. Inputs
        // restricted to `k / 2^n` (no decimal-storage rounding), which
        // is the only domain we emit path coordinates from.
        let cases: &[(f64, &str)] = &[
            (0.25, "0.3"),    // IEEE-exact tie → JS rounds up
            (0.75, "0.8"),    // tie → up
            (1.25, "1.3"),    // tie → up
            (1.75, "1.8"),    // tie → up
            (0.5, "0.5"),     // already a one-decimal value
            (-0.5, "-0.5"),
            (-1.5, "-1.5"),
            (-0.25, "-0.3"),
            (14.84375, "14.8"),
            (10.1015625, "10.1"),    // 1293/128 — H's leftmost x
            (8.59375, "8.6"),         // 1100/128
            (-11.40625, "-11.4"),     // a glyph upper y
            (3.4375, "3.4"),
            (0.7109375, "0.7"),       // 'o' glyph first M x
            (-4.203125, "-4.2"),
        ];
        for (v, want) in cases {
            assert_eq!(js_to_fixed(*v, 1), *want, "v={v}");
        }
    }

    #[test]
    fn js_float_to_string_integer_branch() {
        assert_eq!(js_float_to_string(0.0, 1), "0");
        assert_eq!(js_float_to_string(-0.0, 1), "0");
        assert_eq!(js_float_to_string(3.0, 1), "3");
        assert_eq!(js_float_to_string(-3.0, 1), "-3");
        assert_eq!(js_float_to_string(15.0, 1), "15");
    }

    #[test]
    fn js_float_to_string_non_integer_branch() {
        assert_eq!(js_float_to_string(0.5, 1), "0.5");
        assert_eq!(js_float_to_string(-0.5, 1), "-0.5");
        assert_eq!(js_float_to_string(14.84375, 1), "14.8");
    }
}
