//! Port of `src/vendor/parse-css-dimension/` and `lengthToNumber` from
//! `src/utils.ts`.

/// A CSS dimension value as it can appear inside `width`, `height`,
/// `padding`, `margin`, etc.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dim {
    /// `length` in px (already converted).
    Px(f32),
    /// `percentage` (e.g. `"100%"`)
    Percent(f32),
    /// `auto`
    Auto,
}

/// Parse a JS-style length input.
///
/// JS satori accepts: numbers (treated as px), `"<n>px"`, `"<n>%"`,
/// `"auto"`, `"<n>em"`, `"<n>rem"`, `"<n>vw"`, `"<n>vh"`.
///
/// `base_font_size` is the inherited `font-size` in px (for em).
/// `viewport_w`/`viewport_h` are the viewport extents (for vw/vh).
pub fn parse_dimension(
    raw: &serde_json::Value,
    base_font_size: f32,
    viewport_w: Option<u32>,
    viewport_h: Option<u32>,
) -> Option<Dim> {
    if let Some(n) = raw.as_f64() {
        return Some(Dim::Px(n as f32));
    }
    let s = raw.as_str()?.trim();
    if s == "auto" {
        return Some(Dim::Auto);
    }
    if let Some(stripped) = s.strip_suffix('%') {
        return stripped.parse::<f32>().ok().map(Dim::Percent);
    }
    if let Some(stripped) = s.strip_suffix("px") {
        return stripped.trim().parse::<f32>().ok().map(Dim::Px);
    }
    if let Some(stripped) = s.strip_suffix("em") {
        let rem = stripped.strip_suffix('r');
        let v: f32 = match rem {
            Some(num) => num.trim().parse().ok()?,
            None => stripped.trim().parse().ok()?,
        };
        return Some(Dim::Px(v * if rem.is_some() { 16.0 } else { base_font_size }));
    }
    if let Some(stripped) = s.strip_suffix("vw") {
        let v: f32 = stripped.trim().parse().ok()?;
        let vp = viewport_w.unwrap_or(0) as f32;
        return Some(Dim::Px((v * vp / 100.0).trunc()));
    }
    if let Some(stripped) = s.strip_suffix("vh") {
        let v: f32 = stripped.trim().parse().ok()?;
        let vp = viewport_h.unwrap_or(0) as f32;
        return Some(Dim::Px((v * vp / 100.0).trunc()));
    }
    // bare number-as-string: "100"
    s.parse::<f32>().ok().map(Dim::Px)
}
