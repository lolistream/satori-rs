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
/// Implementation operates in the integer domain rather than via
/// `format!("{:.50}")` (which falls into the slow bignum `dragon`
/// formatter and dominates render time on `embedFont:true`):
///
/// 1. Compute `p = |v| * 10^dp` in IEEE.
/// 2. Round to the nearest integer with `JS V8 toFixed` semantics
///    (ties-to-larger). For non-tie inputs `(p.floor() if frac < 0.5
///    else p.floor() + 1)` is byte-identical to V8.
/// 3. At the IEEE tie boundary (`frac == 0.5`), the f64 product may
///    have crossed `int.5` due to rounding while the true
///    mathematical product is on the other side; resolve this with a
///    single FMA — `abs.mul_add(scale, -p)` gives the exact rounding
///    error, whose sign tells us whether to round up or down. JS
///    looks at the EXACT mathematical value, so this matches V8
///    byte-for-byte. (Without this correction inputs like `v = 0.15`
///    — f64 ≈ 0.14999… — would round UP to `"0.2"` instead of the
///    correct `"0.1"`.)
///
/// The fast path requires `|v * 10^dp| < 2^53`; every path coordinate
/// we emit (glyph-units divided by `unitsPerEm`, scaled by
/// `fontSize ≤ ~10^3` and emitted with `dp=1`) fits comfortably.
pub(crate) fn js_to_fixed(v: f64, dp: u32) -> String {
    if !v.is_finite() {
        return format!("{v}");
    }
    let neg = v < 0.0;
    let abs = v.abs();

    // 10^dp; exact f64 for `dp` up to 22.
    let scale = 10f64.powi(dp as i32);
    debug_assert!(
        abs * scale < (1u64 << 53) as f64,
        "js_to_fixed: |v * 10^dp| must fit in 2^53 for exact IEEE mul"
    );
    let p = abs * scale;
    let int_part = p.floor();
    let mut rounded = int_part as u64;
    let frac = p - int_part;
    if frac > 0.5 {
        rounded += 1;
    } else if frac == 0.5 {
        // IEEE put us exactly on the tie. The true mathematical
        // product `abs * scale` is `p + err` where err is the
        // round-once error of the multiplication. JS V8 picks the
        // larger candidate iff true >= int+0.5 (i.e. err >= 0).
        let err = abs.mul_add(scale, -p);
        if err >= 0.0 {
            rounded += 1;
        }
    }

    // Emit `rounded`'s decimal digits into a stack buffer (max u64 =
    // 20 digits). Walk the buffer back-to-front so we can copy a
    // contiguous tail without reversing.
    let mut digits = [0u8; 20];
    let mut i = digits.len();
    let mut n = rounded;
    loop {
        i -= 1;
        digits[i] = b'0' + (n % 10) as u8;
        n /= 10;
        if n == 0 {
            break;
        }
    }
    let digits = &digits[i..];
    let dp_us = dp as usize;

    let mut out = String::with_capacity(digits.len() + dp_us + 3);
    if neg {
        out.push('-');
    }
    if dp_us == 0 {
        out.push_str(std::str::from_utf8(digits).expect("ascii digits"));
    } else if digits.len() <= dp_us {
        // Magnitude < 1.0: emit "0." + leading zeros + digits.
        out.push_str("0.");
        for _ in 0..(dp_us - digits.len()) {
            out.push('0');
        }
        out.push_str(std::str::from_utf8(digits).expect("ascii digits"));
    } else {
        let split = digits.len() - dp_us;
        out.push_str(std::str::from_utf8(&digits[..split]).expect("ascii digits"));
        out.push('.');
        out.push_str(std::str::from_utf8(&digits[split..]).expect("ascii digits"));
    }
    out
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
