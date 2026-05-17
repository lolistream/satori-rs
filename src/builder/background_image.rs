//! Port of `src/builder/background-image.ts` plus `src/builder/gradient/*`
//! for linear + radial CSS gradients.
//!
//! The JS dispatcher (`backgroundImage(...)`) returns a `[patternId, defs]`
//! pair per layer. The renderer wraps each gradient inside an SVG
//! `<pattern>` so that `background-size` / `background-position` /
//! `background-repeat` can be honored uniformly via SVG pattern tiling.
//!
//! We port the math from `linear.ts` and `radial.ts` verbatim. To keep
//! pixel-identical output with the JS reference we:
//!   - mirror attribute insertion order
//!   - emit numeric attributes via `js_number_to_string_f64` so trailing
//!     zeros drop the same way (`1.0` → `1`)
//!   - compute geometry in `f64` because JS satori's gradient math runs
//!     on doubles and the rasterized output is sensitive to mantissa
//!     differences (a single ULP in a `<circle r="...">` value shifts a
//!     handful of pixels)
//!   - use the same `satori_pattern_<id>` / `satori_bi<id>` /
//!     `satori_radial_<id>` / `satori_mask_<id>` naming
//!   - preserve the `<mask>` + extra fill `<rect>` quirk in radial output.

use crate::css::color::parse_color;
use crate::css::gradient::{
    ColorStop, ConicGradient, LinearGradient, LinearOrientation, RadialGradient,
    RadialPropertyValue,
};
use crate::css::style::{BackgroundImage, ResolvedUrlImage};

use super::xml::{build_xml, js_number_to_string_f64, AttrValue};

/// Render a single `background-image` layer. Returns
/// `(pattern_id, defs_xml)`. The caller emits one `<rect>` (or `<path>`)
/// per layer with `fill="url(#<pattern_id>)"` and accumulates the defs
/// into a single `<defs>...</defs>` block.
///
/// Returns `None` for unsupported layer kinds (e.g. URL images, which
/// are not implemented in this slice).
pub fn render_background_image(
    id: &str,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    index: usize,
    bg: &BackgroundImage,
    background_size: Option<&str>,
    background_position: Option<&str>,
    background_repeat: Option<&str>,
) -> Option<(String, String)> {
    render_background_image_from(
        id,
        left,
        top,
        width,
        height,
        index,
        bg,
        background_size,
        background_position,
        background_repeat,
        From::Background,
    )
}

/// `from` parameter — controls the gradient color-stop remap. When
/// `Mask`, every stop becomes `rgba(0,0,0,1)` (alpha=0 origin) or
/// `rgba(255,255,255,alpha)` (alpha>0 origin) so the resulting
/// gradient acts as a luminance/alpha mask.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum From {
    Background,
    Mask,
}

pub fn render_background_image_from(
    id: &str,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    index: usize,
    bg: &BackgroundImage,
    background_size: Option<&str>,
    background_position: Option<&str>,
    background_repeat: Option<&str>,
    from: From,
) -> Option<(String, String)> {
    render_background_image_sub(
        &format!("{id}_{index}"),
        left,
        top,
        width,
        height,
        bg,
        background_size,
        background_position,
        background_repeat,
        from,
    )
}

/// Like `render_background_image_from`, but the caller supplies the
/// fully-resolved `sub_id` directly (no `_{index}` suffix appended).
/// Used by `mask-image` so the pattern id matches JS satori's
/// `buildBackgroundImage({id: '${miId}-${i}'})` shape exactly.
pub fn render_background_image_sub(
    sub_id: &str,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    bg: &BackgroundImage,
    background_size: Option<&str>,
    background_position: Option<&str>,
    background_repeat: Option<&str>,
    from: From,
) -> Option<(String, String)> {
    let sub_id = sub_id.to_string();

    let repeat = background_repeat.unwrap_or("repeat");
    let repeat_x = repeat == "repeat-x" || repeat == "repeat";
    let repeat_y = repeat == "repeat-y" || repeat == "repeat";

    let width_f = width as f64;
    let height_f = height as f64;

    // Gradients evaluate `parseLengthPairs` against keyword/length values
    // *without* the natural-image dimensions. URLs need the natural dims
    // for `cover`/`contain`/`auto`, so we route them through a separate
    // helper.
    let is_url = matches!(bg, BackgroundImage::Url { .. });
    if is_url {
        if let BackgroundImage::Url {
            src: _,
            resolved: Some(resolved),
        } = bg
        {
            return Some(build_url_pattern(
                &sub_id,
                left as f64,
                top as f64,
                width_f,
                height_f,
                repeat_x,
                repeat_y,
                background_size,
                background_position,
                resolved,
            ));
        }
        return None;
    }

    let dimensions =
        parse_length_pairs(background_size, width_f, height_f, [width_f, height_f]);
    let offsets = parse_length_pairs(background_position, width_f, height_f, [0.0, 0.0]);

    let _ = (left, top);

    match bg {
        BackgroundImage::Linear(g) => Some(build_linear_gradient(
            &sub_id, width_f, height_f, repeat_x, repeat_y, g, dimensions, offsets, from,
        )),
        BackgroundImage::Radial(g) => Some(build_radial_gradient(
            &sub_id, width_f, height_f, repeat_x, repeat_y, g, dimensions, offsets, from,
        )),
        BackgroundImage::Conic(g) => Some(build_conic_gradient(
            &sub_id, width_f, height_f, repeat_x, repeat_y, g, dimensions, offsets, from,
        )),
        BackgroundImage::Url { .. } => unreachable!(
            "`BackgroundImage::Url` is dispatched to `build_url_pattern` via \
             the `is_url` early-return above; reaching this arm means the \
             dispatch table lost a variant - fix the routing rather than \
             swallowing it here (audit #5)"
        ),
    }
}

/// Port of the `image.startsWith('url(')` branch in
/// `src/builder/background-image.ts`. Emits a `<pattern>` (with the
/// `userSpaceOnUse` units that JS satori uses) wrapping a single
/// `<image>` reference. Repeating in either axis switches between
/// `100%` and the resolved tile size.
fn build_url_pattern(
    id: &str,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    repeat_x: bool,
    repeat_y: bool,
    background_size: Option<&str>,
    background_position: Option<&str>,
    resolved: &ResolvedUrlImage,
) -> (String, String) {
    let image_width = resolved.natural_width.map(|n| n as f64).unwrap_or(0.0);
    let image_height = resolved.natural_height.map(|n| n as f64).unwrap_or(0.0);

    let is_keyword_size = matches!(
        background_size.map(str::trim),
        Some("cover") | Some("contain") | Some("auto"),
    ) || background_size
        .map(|s| s.split_whitespace().any(|w| w == "auto"))
        .unwrap_or(false);

    let (resolved_width, resolved_height) = if is_keyword_size {
        let (w, h) = calculate_keyword_size(
            background_size.unwrap_or("auto").trim(),
            width,
            height,
            image_width,
            image_height,
        );
        (w, h)
    } else {
        let dims = parse_length_pairs(background_size, width, height, [0.0, 0.0]);
        // JS background-image.ts (`from === 'background'`):
        //   resolvedWidth  = dims[0] || imageWidth
        //   resolvedHeight = dims[1] || imageHeight
        let w = if dims[0] != 0.0 { dims[0] } else { image_width };
        let h = if dims[1] != 0.0 { dims[1] } else { image_height };
        (w, h)
    };

    let offsets = parse_length_pairs(background_position, width, height, [0.0, 0.0]);

    let pattern_id = format!("satori_bi{id}");

    let inner_image = build_xml(
        "image",
        &[
            ("x", AttrValue::NumberF64(0.0)),
            ("y", AttrValue::NumberF64(0.0)),
            ("width", AttrValue::NumberF64(resolved_width)),
            ("height", AttrValue::NumberF64(resolved_height)),
            ("preserveAspectRatio", AttrValue::Str("none")),
            ("href", AttrValue::Str(resolved.src.as_str())),
        ],
        None,
    );

    // `width` / `height` on the pattern: `repeat` axis uses the tile
    // size, otherwise `100%` (which makes it span the whole rect even
    // if the tile is smaller). JS emits both dims even when the
    // resolved size is 0 — the `<image>` is then degenerate but the
    // resulting empty fill matches.
    let pattern_w = if repeat_x {
        AttrValue::NumberF64(resolved_width)
    } else {
        AttrValue::Str("100%")
    };
    let pattern_h = if repeat_y {
        AttrValue::NumberF64(resolved_height)
    } else {
        AttrValue::Str("100%")
    };

    let pattern = build_xml(
        "pattern",
        &[
            ("id", AttrValue::Str(pattern_id.as_str())),
            ("patternContentUnits", AttrValue::Str("userSpaceOnUse")),
            ("patternUnits", AttrValue::Str("userSpaceOnUse")),
            ("x", AttrValue::NumberF64(offsets[0] + left)),
            ("y", AttrValue::NumberF64(offsets[1] + top)),
            ("width", pattern_w),
            ("height", pattern_h),
        ],
        Some(&inner_image),
    );

    (pattern_id, pattern)
}

/// Port of `calculateKeywordSize` in `src/builder/background-image.ts`.
///
/// `cover` / `contain` scale the natural image size to fit/cover the
/// container while preserving aspect ratio. `auto` (with up to two
/// tokens) keeps the natural size for `auto` axes and resolves
/// length/percentage values for the explicit axis. When the natural
/// size is unknown (zero), we fall back to the container dims.
fn calculate_keyword_size(
    keyword: &str,
    container_width: f64,
    container_height: f64,
    image_width: f64,
    image_height: f64,
) -> (f64, f64) {
    if image_width == 0.0 || image_height == 0.0 {
        return (container_width, container_height);
    }
    if keyword == "cover" {
        let scale_x = container_width / image_width;
        let scale_y = container_height / image_height;
        let scale = scale_x.max(scale_y);
        return (image_width * scale, image_height * scale);
    }
    if keyword == "contain" {
        let scale_x = container_width / image_width;
        let scale_y = container_height / image_height;
        let scale = scale_x.min(scale_y);
        return (image_width * scale, image_height * scale);
    }
    if keyword == "auto" || keyword.contains("auto") {
        let parts: Vec<&str> = keyword.split_whitespace().collect();
        let width_part = parts.first().copied().unwrap_or("auto");
        let height_part = parts.get(1).copied().or_else(|| parts.first().copied()).unwrap_or("auto");

        let mut final_width = image_width;
        let mut final_height = image_height;
        if width_part == "auto" && height_part != "auto" {
            let parsed_h = to_absolute_value(height_part, container_height);
            final_height = parsed_h;
            final_width = (image_width / image_height) * parsed_h;
        } else if height_part == "auto" && width_part != "auto" {
            let parsed_w = to_absolute_value(width_part, container_width);
            final_width = parsed_w;
            final_height = (image_height / image_width) * parsed_w;
        }
        return (final_width, final_height);
    }
    (container_width, container_height)
}

fn to_absolute_value(v: &str, base: f64) -> f64 {
    let v = v.trim();
    if let Some(p) = v.strip_suffix('%') {
        if let Ok(n) = p.parse::<f64>() {
            return base * n / 100.0;
        }
    }
    if let Some(p) = v.strip_suffix("px") {
        if let Ok(n) = p.parse::<f64>() {
            return n;
        }
    }
    v.parse::<f64>().unwrap_or(0.0)
}

/// Port of `parseLengthPairs` in `background-image.ts`. Supports the
/// common test forms: `"<n>px <n>px"`, `"<n>% <n>%"`, or omitted.
fn parse_length_pairs(
    raw: Option<&str>,
    x: f64,
    y: f64,
    defaults: [f64; 2],
) -> [f64; 2] {
    let Some(s) = raw else { return defaults };
    let s = s.trim();
    if s.is_empty() {
        return defaults;
    }
    let parts: Vec<&str> = s.split_whitespace().collect();
    let raw_x: Option<&str> = parts.first().copied();
    let raw_y: Option<&str> = parts.get(1).copied().or(raw_x);
    let to_abs = |v: &str, base: f64| -> f64 {
        if let Some(p) = v.strip_suffix('%') {
            if let Ok(n) = p.parse::<f64>() {
                return base * n / 100.0;
            }
        }
        if let Some(p) = v.strip_suffix("px") {
            if let Ok(n) = p.parse::<f64>() {
                return n;
            }
        }
        v.parse::<f64>().unwrap_or(0.0)
    };
    [
        raw_x.map(|v| to_abs(v, x)).unwrap_or(defaults[0]),
        raw_y.map(|v| to_abs(v, y)).unwrap_or(defaults[1]),
    ]
}

// ----- linear gradient -----------------------------------------------

#[derive(Clone, Copy)]
struct LinearPoints {
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
}

fn build_linear_gradient(
    id: &str,
    width: f64,
    height: f64,
    repeat_x: bool,
    repeat_y: bool,
    g: &LinearGradient,
    dimensions: [f64; 2],
    offsets: [f64; 2],
    from: From,
) -> (String, String) {
    let [image_width, image_height] = dimensions;
    let repeating = g.repeating;

    let (points, length) = match &g.orientation {
        LinearOrientation::Directional(dir) => {
            let p = resolve_xy_from_direction(dir);
            let len = (((p.x2 - p.x1) * image_width).powi(2)
                + ((p.y2 - p.y1) * image_height).powi(2))
            .sqrt();
            (p, len)
        }
        LinearOrientation::Angular { value, unit } => {
            let deg = calc_degree(value, unit);
            let rad = deg / 180.0 * std::f64::consts::PI;
            let (p, len) = calc_normal_point(rad, image_width, image_height);
            (p, len)
        }
    };

    let xys = if repeating {
        calc_percentage(&g.stops, length, &points)
    } else {
        points
    };

    let total = if repeating {
        resolve_repeating_cycle(&g.stops, length)
    } else {
        length
    };
    let stops = normalize_stops(total, &g.stops, repeating, from);

    let gradient_id = format!("satori_bi{id}");
    let pattern_id = format!("satori_pattern_{id}");

    // Build <stop> children first.
    let mut stops_xml = String::new();
    for stop in &stops {
        let off = stop.offset.unwrap_or(0.0);
        let offset_str = format!("{}%", js_number_to_string_f64(off * 100.0));
        stops_xml.push_str(&build_xml(
            "stop",
            &[
                ("offset", AttrValue::Owned(offset_str)),
                ("stop-color", AttrValue::Str(stop.color.as_str())),
            ],
            None,
        ));
    }

    let spread = if repeating { "repeat" } else { "pad" };
    let gradient_el = build_xml(
        "linearGradient",
        &[
            ("id", AttrValue::Str(gradient_id.as_str())),
            ("x1", AttrValue::NumberF64(xys.x1)),
            ("y1", AttrValue::NumberF64(xys.y1)),
            ("x2", AttrValue::NumberF64(xys.x2)),
            ("y2", AttrValue::NumberF64(xys.y2)),
            ("spreadMethod", AttrValue::Str(spread)),
        ],
        Some(&stops_xml),
    );

    let inner_rect = build_xml(
        "rect",
        &[
            ("x", AttrValue::NumberF64(0.0)),
            ("y", AttrValue::NumberF64(0.0)),
            ("width", AttrValue::NumberF64(image_width)),
            ("height", AttrValue::NumberF64(image_height)),
            ("fill", AttrValue::Owned(format!("url(#{gradient_id})"))),
        ],
        None,
    );
    let children = format!("{gradient_el}{inner_rect}");

    let pattern_w_value = pattern_dim(repeat_x, image_width, width);
    let pattern_h_value = pattern_dim(repeat_y, image_height, height);

    let pattern = build_xml(
        "pattern",
        &[
            ("id", AttrValue::Str(pattern_id.as_str())),
            ("x", AttrValue::NumberF64(offsets[0] / width)),
            ("y", AttrValue::NumberF64(offsets[1] / height)),
            ("width", AttrValue::Owned(pattern_w_value)),
            ("height", AttrValue::Owned(pattern_h_value)),
            ("patternUnits", AttrValue::Str("objectBoundingBox")),
        ],
        Some(&children),
    );

    (pattern_id, pattern)
}

fn pattern_dim(repeat: bool, image: f64, container: f64) -> String {
    if repeat {
        js_number_to_string_f64(image / container)
    } else {
        "1".to_string()
    }
}

fn resolve_xy_from_direction(dir: &str) -> LinearPoints {
    let mut x1 = 0.0;
    let mut y1 = 0.0;
    let mut x2 = 0.0;
    let mut y2 = 0.0;
    let d = dir.to_ascii_lowercase();
    if d.contains("top") {
        y1 = 1.0;
    } else if d.contains("bottom") {
        y2 = 1.0;
    }
    if d.contains("left") {
        x1 = 1.0;
    } else if d.contains("right") {
        x2 = 1.0;
    }
    if x1 == 0.0 && x2 == 0.0 && y1 == 0.0 && y2 == 0.0 {
        y1 = 1.0;
    }
    LinearPoints { x1, y1, x2, y2 }
}

/// Port of `calcNormalPoint` in `linear.ts`. Returns normalized points
/// (in 0..1 of the image box) along with the gradient length in pixels.
fn calc_normal_point(v: f64, w: f64, h: f64) -> (LinearPoints, f64) {
    let r = (h / w).powi(2);
    let two_pi = std::f64::consts::PI * 2.0;
    let v = ((v % two_pi) + two_pi) % two_pi;

    fn dfs(angle: f64, w: f64, h: f64, r: f64) -> (f64, f64, f64, f64, f64) {
        if angle == 0.0 {
            return (0.0, h, 0.0, 0.0, h);
        }
        if (angle - std::f64::consts::FRAC_PI_2).abs() < f64::EPSILON {
            return (0.0, 0.0, w, 0.0, w);
        }
        if angle > 0.0 && angle < std::f64::consts::FRAC_PI_2 {
            let tan = angle.tan();
            let x1_ = ((r * w) / 2.0 / tan - h / 2.0) / (tan + r / tan);
            let y1_ = tan * x1_ + h;
            let x2_ = (w / 2.0 - x1_).abs() + w / 2.0;
            let y2_ = h / 2.0 - (y1_ - h / 2.0).abs();
            // Length recomputed via midpoint reflection — mirrors JS exactly.
            let a = (w / 2.0 / tan - h / 2.0) / (tan + 1.0 / tan);
            let b = tan * a + h;
            let length = 2.0 * ((w / 2.0 - a).powi(2) + (h / 2.0 - b).powi(2)).sqrt();
            return (x1_, y1_, x2_, y2_, length);
        }
        if angle > std::f64::consts::FRAC_PI_2 && angle < std::f64::consts::PI {
            let tan = angle.tan();
            let x1_ = (h / 2.0 + (r * w) / 2.0 / tan) / (tan + r / tan);
            let y1_ = tan * x1_;
            let x2_ = (w / 2.0 - x1_).abs() + w / 2.0;
            let y2_ = h / 2.0 + (y1_ - h / 2.0).abs();
            let a = (w / 2.0 / tan + h / 2.0) / (tan + 1.0 / tan);
            let b = tan * a;
            let length = 2.0 * ((w / 2.0 - a).powi(2) + (h / 2.0 - b).powi(2)).sqrt();
            return (x1_, y1_, x2_, y2_, length);
        }
        if angle >= std::f64::consts::PI {
            let (x1_, y1_, x2_, y2_, length) = dfs(angle - std::f64::consts::PI, w, h, r);
            return (x2_, y2_, x1_, y1_, length);
        }
        (0.0, 0.0, 0.0, 0.0, 0.0)
    }

    let (x1, y1, x2, y2, length) = dfs(v, w, h, r);
    (
        LinearPoints {
            x1: x1 / w,
            y1: y1 / h,
            x2: x2 / w,
            y2: y2 / h,
        },
        length,
    )
}

fn resolve_repeating_cycle(stops: &[ColorStop], length: f64) -> f64 {
    let Some(last) = stops.last() else { return length };
    let Some(offset) = last.offset.as_ref() else { return length };
    if offset.unit == "%" {
        if let Ok(n) = offset.value.parse::<f64>() {
            return n / 100.0 * length;
        }
    }
    offset.value.parse::<f64>().unwrap_or(length)
}

fn calc_percentage(stops: &[ColorStop], length: f64, points: &LinearPoints) -> LinearPoints {
    let p1 = stop_relative(stops.first(), length, 0.0);
    let p2 = stop_relative(stops.last(), length, 1.0);
    let sx = (points.x2 - points.x1) * p1 + points.x1;
    let sy = (points.y2 - points.y1) * p1 + points.y1;
    let ex = (points.x2 - points.x1) * p2 + points.x1;
    let ey = (points.y2 - points.y1) * p2 + points.y1;
    LinearPoints {
        x1: sx,
        y1: sy,
        x2: ex,
        y2: ey,
    }
}

fn stop_relative(stop: Option<&ColorStop>, length: f64, default: f64) -> f64 {
    let Some(s) = stop else { return default };
    let Some(o) = s.offset.as_ref() else { return default };
    if o.unit == "%" {
        return o.value.parse::<f64>().unwrap_or(0.0) / 100.0;
    }
    length_to_number_px(&o.value, &o.unit, length) / length
}

// ----- radial gradient -----------------------------------------------

#[derive(Default, Clone, Copy)]
struct RadialSpread {
    r: Option<f64>,
    rx: Option<f64>,
    ry: Option<f64>,
    ratio: Option<f64>,
}

fn build_radial_gradient(
    id: &str,
    width: f64,
    height: f64,
    repeat_x: bool,
    repeat_y: bool,
    g: &RadialGradient,
    dimensions: [f64; 2],
    offsets: [f64; 2],
    from: From,
) -> (String, String) {
    let [x_delta, y_delta] = dimensions;

    let (cx, cy) = calc_radial_position(&g.position.x, &g.position.y, x_delta, y_delta);

    let color_stop_total = if !g.repeating {
        width
    } else if let Some(last) = g.stops.last() {
        if let Some(off) = &last.offset {
            if off.unit == "%" {
                width
            } else {
                length_to_number_px(&off.value, &off.unit, width)
            }
        } else {
            width
        }
    } else {
        width
    };

    let stops = normalize_stops(color_stop_total, &g.stops, g.repeating, from);

    let gradient_id = format!("satori_radial_{id}");
    let pattern_id = format!("satori_pattern_{id}");
    let mask_id = format!("satori_mask_{id}");

    let spread = calc_radius(&g.shape, &g.size, cx, cy, x_delta, y_delta, g.repeating);
    let props = calc_radial_gradient_props(
        &g.shape,
        &g.stops,
        x_delta,
        y_delta,
        g.repeating,
        &spread,
    );

    let mut stops_xml = String::new();
    for stop in &stops {
        let off = stop.offset.unwrap_or(0.0);
        stops_xml.push_str(&build_xml(
            "stop",
            &[
                ("offset", AttrValue::Owned(js_number_to_string_f64(off))),
                ("stop-color", AttrValue::Str(stop.color.as_str())),
            ],
            None,
        ));
    }

    let mut gradient_attrs: Vec<(&str, AttrValue)> = vec![
        ("id", AttrValue::Str(gradient_id.as_str())),
        ("spreadMethod", AttrValue::Str(if g.repeating { "repeat" } else { "pad" })),
    ];
    if let Some(cx_attr) = &props.cx {
        gradient_attrs.push(("cx", AttrValue::Owned(cx_attr.clone())));
    }
    if let Some(cy_attr) = &props.cy {
        gradient_attrs.push(("cy", AttrValue::Owned(cy_attr.clone())));
    }
    if let Some(r_attr) = &props.r {
        gradient_attrs.push(("r", AttrValue::Owned(r_attr.clone())));
    }
    let gradient_el = build_xml("radialGradient", &gradient_attrs, Some(&stops_xml));

    let mask_inner = build_xml(
        "rect",
        &[
            ("x", AttrValue::NumberF64(0.0)),
            ("y", AttrValue::NumberF64(0.0)),
            ("width", AttrValue::NumberF64(x_delta)),
            ("height", AttrValue::NumberF64(y_delta)),
            ("fill", AttrValue::Str("#fff")),
        ],
        None,
    );
    let mask_el = build_xml(
        "mask",
        &[("id", AttrValue::Str(mask_id.as_str()))],
        Some(&mask_inner),
    );

    let fallback_color = stops
        .last()
        .map(|s| s.color.clone())
        .unwrap_or_else(|| "transparent".to_string());
    let fallback_rect = build_xml(
        "rect",
        &[
            ("x", AttrValue::NumberF64(0.0)),
            ("y", AttrValue::NumberF64(0.0)),
            ("width", AttrValue::NumberF64(x_delta)),
            ("height", AttrValue::NumberF64(y_delta)),
            ("fill", AttrValue::Owned(fallback_color)),
        ],
        None,
    );

    let mut shape_attrs: Vec<(&str, AttrValue)> = vec![
        ("cx", AttrValue::NumberF64(cx)),
        ("cy", AttrValue::NumberF64(cy)),
        ("width", AttrValue::NumberF64(x_delta)),
        ("height", AttrValue::NumberF64(y_delta)),
    ];
    if g.shape == "circle" {
        if let Some(r) = spread.r {
            shape_attrs.push(("r", AttrValue::NumberF64(r)));
        }
    } else {
        if let Some(rx) = spread.rx {
            shape_attrs.push(("rx", AttrValue::NumberF64(rx)));
        }
        if let Some(ry) = spread.ry {
            shape_attrs.push(("ry", AttrValue::NumberF64(ry)));
        }
    }
    shape_attrs.push(("fill", AttrValue::Owned(format!("url(#{gradient_id})"))));
    shape_attrs.push(("mask", AttrValue::Owned(format!("url(#{mask_id})"))));
    let shape_el = build_xml(&g.shape, &shape_attrs, None);

    let children = format!("{gradient_el}{mask_el}{fallback_rect}{shape_el}");

    let pattern = build_xml(
        "pattern",
        &[
            ("id", AttrValue::Str(pattern_id.as_str())),
            ("x", AttrValue::NumberF64(offsets[0] / width)),
            ("y", AttrValue::NumberF64(offsets[1] / height)),
            ("width", AttrValue::Owned(pattern_dim(repeat_x, x_delta, width))),
            ("height", AttrValue::Owned(pattern_dim(repeat_y, y_delta, height))),
            ("patternUnits", AttrValue::Str("objectBoundingBox")),
        ],
        Some(&children),
    );

    (pattern_id, pattern)
}

#[derive(Default, Clone)]
struct RadialGradientProps {
    cx: Option<String>,
    cy: Option<String>,
    r: Option<String>,
}

fn calc_radial_gradient_props(
    shape: &str,
    color_stops: &[ColorStop],
    x_delta: f64,
    y_delta: f64,
    repeating: bool,
    spread: &RadialSpread,
) -> RadialGradientProps {
    if !repeating {
        return RadialGradientProps::default();
    }
    let last = match color_stops.last() {
        Some(s) => s,
        None => return RadialGradientProps::default(),
    };
    let Some(off) = last.offset.as_ref() else {
        return RadialGradientProps::default();
    };
    let radius = if shape == "circle" {
        spread.r.unwrap_or(0.0) * 2.0
    } else {
        spread.rx.unwrap_or(0.0) * 2.0
    };
    let r_str = if off.unit == "%" {
        let val: f64 = off.value.parse().unwrap_or(0.0);
        let ratio = spread.ratio.unwrap_or(1.0);
        format!(
            "{}%",
            js_number_to_string_f64(val * (y_delta / x_delta).min(1.0) / ratio)
        )
    } else {
        let n = length_to_number_px(&off.value, &off.unit, x_delta);
        js_number_to_string_f64(n / radius)
    };
    RadialGradientProps {
        cx: Some("50%".to_string()),
        cy: Some("50%".to_string()),
        r: Some(r_str),
    }
}

fn calc_radial_position(
    px: &RadialPropertyValue,
    py: &RadialPropertyValue,
    x_delta: f64,
    y_delta: f64,
) -> (f64, f64) {
    let mut x = x_delta / 2.0;
    let mut y = y_delta / 2.0;
    match px {
        RadialPropertyValue::Keyword(k) => {
            if let Some((nx, ny)) = pos_keyword(k, x_delta, y_delta, true) {
                x = nx;
                if let Some(v) = ny {
                    y = v;
                }
            }
        }
        RadialPropertyValue::Length(o) => {
            x = length_to_number_px(&o.value, &o.unit, x_delta);
        }
    }
    match py {
        RadialPropertyValue::Keyword(k) => {
            if let Some((ny, nx)) = pos_keyword(k, x_delta, y_delta, false) {
                y = ny;
                if let Some(v) = nx {
                    x = v;
                }
            }
        }
        RadialPropertyValue::Length(o) => {
            y = length_to_number_px(&o.value, &o.unit, y_delta);
        }
    }
    (x, y)
}

fn pos_keyword(
    key: &str,
    x_delta: f64,
    y_delta: f64,
    is_x: bool,
) -> Option<(f64, Option<f64>)> {
    match key {
        "center" => Some(if is_x {
            (x_delta / 2.0, None)
        } else {
            (y_delta / 2.0, None)
        }),
        "left" => Some((0.0, None)),
        "right" => Some((x_delta, None)),
        "top" => Some((0.0, None)),
        "bottom" => Some((y_delta, None)),
        _ => None,
    }
}

fn calc_radius(
    shape: &str,
    ending_shape: &[RadialPropertyValue],
    cx: f64,
    cy: f64,
    x_delta: f64,
    y_delta: f64,
    repeating: bool,
) -> RadialSpread {
    let mut spread = RadialSpread::default();
    let fx: f64;
    let fy: f64;

    if is_size_all_length(ending_shape) {
        if shape == "circle" {
            let s = match &ending_shape[0] {
                RadialPropertyValue::Length(o) => length_to_number_px(&o.value, &o.unit, x_delta),
                _ => 0.0,
            };
            spread.r = Some(s);
        } else {
            let s0 = match &ending_shape[0] {
                RadialPropertyValue::Length(o) => length_to_number_px(&o.value, &o.unit, x_delta),
                _ => 0.0,
            };
            let s1 = match ending_shape.get(1) {
                Some(RadialPropertyValue::Length(o)) => length_to_number_px(&o.value, &o.unit, y_delta),
                _ => 0.0,
            };
            spread.rx = Some(s0);
            spread.ry = Some(s1);
        }
        patch_spread(&mut spread, x_delta, y_delta, cx, cy, repeating, shape);
        return spread;
    }

    let keyword = match ending_shape.first() {
        Some(RadialPropertyValue::Keyword(k)) => k.as_str(),
        _ => "farthest-corner",
    };

    match keyword {
        "farthest-corner" => {
            fx = (x_delta - cx).abs().max(cx.abs());
            fy = (y_delta - cy).abs().max(cy.abs());
        }
        "closest-corner" => {
            fx = (x_delta - cx).abs().min(cx.abs());
            fy = (y_delta - cy).abs().min(cy.abs());
        }
        "farthest-side" => {
            if shape == "circle" {
                spread.r = Some(
                    (x_delta - cx)
                        .abs()
                        .max(cx.abs())
                        .max((y_delta - cy).abs())
                        .max(cy.abs()),
                );
            } else {
                spread.rx = Some((x_delta - cx).abs().max(cx.abs()));
                spread.ry = Some((y_delta - cy).abs().max(cy.abs()));
            }
            patch_spread(&mut spread, x_delta, y_delta, cx, cy, repeating, shape);
            return spread;
        }
        "closest-side" => {
            if shape == "circle" {
                spread.r = Some(
                    (x_delta - cx)
                        .abs()
                        .min(cx.abs())
                        .min((y_delta - cy).abs())
                        .min(cy.abs()),
                );
            } else {
                spread.rx = Some((x_delta - cx).abs().min(cx.abs()));
                spread.ry = Some((y_delta - cy).abs().min(cy.abs()));
            }
            patch_spread(&mut spread, x_delta, y_delta, cx, cy, repeating, shape);
            return spread;
        }
        _ => {
            fx = (x_delta - cx).abs().max(cx.abs());
            fy = (y_delta - cy).abs().max(cy.abs());
        }
    }

    if shape == "circle" {
        spread.r = Some((fx * fx + fy * fy).sqrt());
    } else {
        let (rx, ry) = f2r(fx, fy);
        spread.rx = Some(rx);
        spread.ry = Some(ry);
    }
    patch_spread(&mut spread, x_delta, y_delta, cx, cy, repeating, shape);
    spread
}

fn patch_spread(
    spread: &mut RadialSpread,
    x_delta: f64,
    y_delta: f64,
    cx: f64,
    cy: f64,
    repeating: bool,
    shape: &str,
) {
    if !repeating {
        return;
    }
    if shape == "ellipse" {
        let mfx = (x_delta - cx).abs().max(cx.abs());
        let mfy = (y_delta - cy).abs().max(cy.abs());
        let (mrx, mry) = f2r(mfx, mfy);
        let rx = spread.rx.unwrap_or(0.0);
        let ry = spread.ry.unwrap_or(0.0);
        let ratio = (mrx / rx).max(mry / ry);
        spread.ratio = Some(ratio);
        if ratio > 1.0 {
            spread.rx = Some(rx * ratio);
            spread.ry = Some(ry * ratio);
        }
    } else {
        let mfx = (x_delta - cx).abs().max(cx.abs());
        let mfy = (y_delta - cy).abs().max(cy.abs());
        let mr = (mfx * mfx + mfy * mfy).sqrt();
        let r = spread.r.unwrap_or(0.0);
        let ratio = mr / r;
        spread.ratio = Some(ratio);
        if ratio > 1.0 {
            spread.r = Some(mr);
        }
    }
}

fn f2r(fx: f64, fy: f64) -> (f64, f64) {
    let ratio = if fy != 0.0 { fx / fy } else { 1.0 };
    if fx == 0.0 {
        return (0.0, 0.0);
    }
    let ry = (fx * fx + fy * fy * ratio * ratio).sqrt() / ratio;
    let rx = ry * ratio;
    (rx, ry)
}

fn is_size_all_length(v: &[RadialPropertyValue]) -> bool {
    // An empty list shouldn't be treated as "all length" — let the
    // keyword branch decide (it defaults to `farthest-corner`).
    !v.is_empty() && !v.iter().any(|s| matches!(s, RadialPropertyValue::Keyword(_)))
}

// ----- conic gradient ------------------------------------------------

/// Number of angular samples used to discretize the conic sweep into SVG
/// path slices. Must match the JS satori constant (`SEGMENT_COUNT`)
/// because the per-slice angles fall on the same boundaries and the
/// produced `<path>` coordinates have to be byte-identical.
const CONIC_SEGMENT_COUNT: usize = 360;

fn build_conic_gradient(
    id: &str,
    width: f64,
    height: f64,
    repeat_x: bool,
    repeat_y: bool,
    g: &ConicGradient,
    dimensions: [f64; 2],
    offsets: [f64; 2],
    from: From,
) -> (String, String) {
    let [x_delta, y_delta] = dimensions;

    // `from <angle>` — defaults to 0deg per the JS shim.
    let start_angle = parse_degree_string(&g.angle);

    let (cx, cy) = resolve_conic_position(&g.position, x_delta, y_delta);

    let total_length = calc_conic_total_length(&g.stops, g.repeating);

    let stops = normalize_stops(total_length, &g.stops, g.repeating, from);

    // Hint resolution: hints attach to a stop and live BETWEEN it and the
    // next one. JS satori indexes them against the *post-normalize* stop
    // list, which prepends a synthetic 0% stop unless the very first stop
    // already starts at exactly 0. We replicate the same `idxOff` shift
    // so the hint indexes line up.
    let first_has_explicit_offset = match g.stops.first() {
        Some(s) => s.offset.as_ref().map(|o| o.value != "0").unwrap_or(false),
        None => false,
    };
    let idx_off = if first_has_explicit_offset { 1 } else { 0 };

    let mut hints: Vec<Option<f64>> = vec![None; stops.len()];
    for (pi, parsed_stop) in g.stops.iter().enumerate() {
        if let Some(hint) = parsed_stop.hint.as_ref() {
            let hint_deg = if hint.unit == "%" {
                hint.value.parse::<f64>().unwrap_or(0.0) / 100.0 * total_length
            } else {
                let d = calc_degree(&hint.value, &hint.unit);
                if d == 0.0 { 0.0 } else { d }
            };
            let ni = pi + idx_off;
            if ni < stops.len().saturating_sub(1) {
                hints[ni] = Some(hint_deg / total_length);
            }
        }
    }
    let has_hints = hints.iter().any(|h| h.is_some());

    // Radius = distance from center to the farthest of the 4 corners.
    let radius = ((cx * cx + cy * cy).sqrt())
        .max(((x_delta - cx).powi(2) + cy * cy).sqrt())
        .max(((x_delta - cx).powi(2) + (y_delta - cy).powi(2)).sqrt())
        .max((cx * cx + (y_delta - cy).powi(2)).sqrt());

    let pattern_id = format!("satori_conic_pattern_{id}");
    let clip_id = format!("satori_conic_clip_{id}");

    // Pre-parse stop colors once. Conic interpolation rounds R/G/B per
    // sample and keeps alpha as a float, then formats with JS-style
    // `rgb(...)` / `rgba(...)` so identical adjacent samples collapse into
    // a single path slice.
    let parsed_colors: Vec<Option<(u8, u8, u8, f64)>> =
        stops.iter().map(|s| parse_color_with_alpha(&s.color)).collect();

    // 360 samples → at most 360 path slices, but adjacent identical
    // colors get merged into one.
    let cycle_deg = if g.repeating { total_length } else { 360.0 };

    let mut slices = String::new();
    let mut prev_color: Option<String> = None;
    let mut merge_start: usize = 0;

    let flush_slice = |slices: &mut String, start_idx: usize, end_idx: usize, color: &str| {
        let a1 = start_angle + (start_idx as f64 / CONIC_SEGMENT_COUNT as f64) * 360.0;
        let a2 = start_angle + (end_idx as f64 / CONIC_SEGMENT_COUNT as f64) * 360.0;

        if end_idx - start_idx >= CONIC_SEGMENT_COUNT {
            slices.push_str(&build_xml(
                "circle",
                &[
                    ("cx", AttrValue::NumberF64(cx)),
                    ("cy", AttrValue::NumberF64(cy)),
                    ("r", AttrValue::NumberF64(radius)),
                    ("fill", AttrValue::Str(color)),
                ],
                None,
            ));
            return;
        }

        let r1 = (a1 - 90.0) * std::f64::consts::PI / 180.0;
        let r2 = (a2 - 90.0) * std::f64::consts::PI / 180.0;
        // Use the `libm` crate's `sin`/`cos` (a pure-Rust port of msun
        // libm) instead of the platform-native `r1.sin()` / `r1.cos()`.
        // V8 / Node also derives its `Math.sin`/`Math.cos` from fdlibm-
        // style routines, so this gives identical mantissas down to the
        // last bit. The platform libm on macOS differs by 1 ULP on some
        // inputs, which would slightly shift gradient slice endpoints in
        // the emitted SVG and break the pixel-perfect snapshot match.
        let x1 = cx + radius * libm::cos(r1);
        let y1 = cy + radius * libm::sin(r1);
        let x2 = cx + radius * libm::cos(r2);
        let y2 = cy + radius * libm::sin(r2);
        let large_arc = if a2 - a1 > 180.0 { 1 } else { 0 };

        // Path-data must match JS char-for-char: it stringifies the
        // numbers via the implicit `${n}` (ECMA NumberToString), which is
        // what `js_number_to_string_f64` reproduces. The literal punctuation
        // (`M`, `,`, `L`, `A`, `Z`, `0` for x-axis-rotation, `1` for sweep)
        // is reproduced verbatim.
        let d = format!(
            "M{cx_s},{cy_s}L{x1_s},{y1_s}A{r_s},{r_s},0,{la},1,{x2_s},{y2_s}Z",
            cx_s = js_number_to_string_f64(cx),
            cy_s = js_number_to_string_f64(cy),
            x1_s = js_number_to_string_f64(x1),
            y1_s = js_number_to_string_f64(y1),
            x2_s = js_number_to_string_f64(x2),
            y2_s = js_number_to_string_f64(y2),
            r_s = js_number_to_string_f64(radius),
            la = large_arc,
        );
        slices.push_str(&build_xml(
            "path",
            &[
                ("d", AttrValue::Owned(d)),
                ("fill", AttrValue::Str(color)),
            ],
            None,
        ));
    };

    for i in 0..CONIC_SEGMENT_COUNT {
        let angle_deg = (i as f64 / CONIC_SEGMENT_COUNT as f64) * 360.0;
        let t = if cycle_deg > 0.0 {
            (angle_deg % cycle_deg) / cycle_deg
        } else {
            0.0
        };
        let color = interpolate_conic_color(t, &stops, &parsed_colors, if has_hints { Some(&hints) } else { None });

        if Some(&color) != prev_color.as_ref() {
            if let Some(prev) = &prev_color {
                flush_slice(&mut slices, merge_start, i, prev);
            }
            merge_start = i;
            prev_color = Some(color);
        }
    }
    if let Some(prev) = &prev_color {
        flush_slice(&mut slices, merge_start, CONIC_SEGMENT_COUNT, prev);
    }

    // <clipPath><rect/></clipPath> + <g clip-path=...>slices</g>
    let clip_rect = build_xml(
        "rect",
        &[
            ("x", AttrValue::NumberF64(0.0)),
            ("y", AttrValue::NumberF64(0.0)),
            ("width", AttrValue::NumberF64(x_delta)),
            ("height", AttrValue::NumberF64(y_delta)),
        ],
        None,
    );
    let clip_el = build_xml(
        "clipPath",
        &[("id", AttrValue::Str(clip_id.as_str()))],
        Some(&clip_rect),
    );
    let g_el = build_xml(
        "g",
        &[("clip-path", AttrValue::Owned(format!("url(#{clip_id})")))],
        Some(&slices),
    );
    let children = format!("{clip_el}{g_el}");

    let pattern_w = if repeat_x {
        js_number_to_string_f64(x_delta / width)
    } else {
        "1".to_string()
    };
    let pattern_h = if repeat_y {
        js_number_to_string_f64(y_delta / height)
    } else {
        "1".to_string()
    };
    let pattern = build_xml(
        "pattern",
        &[
            ("id", AttrValue::Str(pattern_id.as_str())),
            ("x", AttrValue::NumberF64(offsets[0] / width)),
            ("y", AttrValue::NumberF64(offsets[1] / height)),
            ("width", AttrValue::Owned(pattern_w)),
            ("height", AttrValue::Owned(pattern_h)),
            ("patternUnits", AttrValue::Str("objectBoundingBox")),
        ],
        Some(&children),
    );

    (pattern_id, pattern)
}

/// Port of `resolvePosition` in `gradient/conic.ts`.
fn resolve_conic_position(position: &str, x_delta: f64, y_delta: f64) -> (f64, f64) {
    let trimmed = position.trim();
    if trimmed.is_empty() || trimmed == "center" {
        return (x_delta / 2.0, y_delta / 2.0);
    }
    let parts: Vec<&str> = trimmed.split_ascii_whitespace().collect();
    if parts.len() == 1 {
        let p = parts[0];
        if p == "top" || p == "bottom" {
            return (x_delta / 2.0, resolve_conic_position_part(p, y_delta));
        }
        return (resolve_conic_position_part(p, x_delta), y_delta / 2.0);
    }

    // CSS positions accept either axis first. The JS swap rule: if part[0]
    // is a y-keyword OR part[1] is an x-keyword, swap so xVal holds the
    // horizontal component.
    let (x_val, y_val) = {
        let yk = |t: &str| t == "top" || t == "bottom";
        let xk = |t: &str| t == "left" || t == "right";
        if (yk(parts[0]) && !yk(parts[1])) || (xk(parts[1]) && !xk(parts[0])) {
            (parts[1], parts[0])
        } else {
            (parts[0], parts[1])
        }
    };
    (
        resolve_conic_position_part(x_val, x_delta),
        resolve_conic_position_part(y_val, y_delta),
    )
}

fn resolve_conic_position_part(v: &str, dim: f64) -> f64 {
    match v {
        "left" | "top" => 0.0,
        "center" => dim / 2.0,
        "right" | "bottom" => dim,
        _ => {
            // Length / percentage / fallback to center.
            if let Some(p) = v.strip_suffix('%') {
                if let Ok(n) = p.parse::<f64>() {
                    return dim * n / 100.0;
                }
            }
            if let Some(p) = v.strip_suffix("px") {
                if let Ok(n) = p.parse::<f64>() {
                    return n;
                }
            }
            if let Ok(n) = v.parse::<f64>() {
                return n;
            }
            dim / 2.0
        }
    }
}

fn calc_conic_total_length(stops: &[ColorStop], repeating: bool) -> f64 {
    if !repeating {
        return 360.0;
    }
    let Some(last) = stops.last() else { return 360.0 };
    let Some(off) = last.offset.as_ref() else { return 360.0 };
    if off.unit == "%" {
        return 360.0;
    }
    let deg = calc_degree(&off.value, &off.unit);
    if deg == 0.0 { 360.0 } else { deg }
}

/// Parse the leading `from <angle>` value of a conic gradient (stored as
/// a raw string like `"45deg"` / `"0.25turn"` / `""`). Returns degrees.
fn parse_degree_string(s: &str) -> f64 {
    let s = s.trim();
    if s.is_empty() { return 0.0; }
    // Split into numeric prefix + unit using the same rule as the parser.
    let bytes = s.as_bytes();
    let mut i = 0;
    if i < bytes.len() && (bytes[i] == b'-' || bytes[i] == b'+') { i += 1; }
    while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
    if i < bytes.len() && bytes[i] == b'.' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_digit() { i += 1; }
    }
    let value = &s[..i];
    let unit = &s[i..];
    calc_degree(value, unit)
}

/// Port of `parseToRGBA` in `conic.ts`. Returns (r:u8, g:u8, b:u8, a:f64)
/// where `a` is preserved as a float in 0..1 so the per-sample alpha
/// interpolation matches JS bit-for-bit.
fn parse_color_with_alpha(color: &str) -> Option<(u8, u8, u8, f64)> {
    let s = color.trim();

    // Try `rgb(r, g, b)` / `rgba(r, g, b, a)` first because we need to
    // preserve the source alpha as a float (the existing `parse_color`
    // quantizes to u8).
    let lower = s.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("rgba") {
        if let Some(out) = parse_rgb_func_with_alpha(rest, true) { return Some(out); }
    }
    if let Some(rest) = lower.strip_prefix("rgb") {
        if let Some(out) = parse_rgb_func_with_alpha(rest, false) { return Some(out); }
    }
    if let Some(rest) = lower.strip_prefix("hsla") {
        if let Some(out) = parse_hsl_func_with_alpha(rest, true) { return Some(out); }
    }
    if let Some(rest) = lower.strip_prefix("hsl") {
        if let Some(out) = parse_hsl_func_with_alpha(rest, false) { return Some(out); }
    }

    // Everything else (named, hex) lands here with discrete alpha. The
    // `parse_color` helper returns alpha as u8; convert to float using
    // the same `parsed.alpha` convention that `parse-css-color` uses
    // (1 for fully opaque, fractional otherwise).
    let rgba = parse_color(s)?;
    let alpha = if rgba.a == 0xff {
        1.0
    } else {
        rgba.a as f64 / 255.0
    };
    Some((rgba.r, rgba.g, rgba.b, alpha))
}

fn parse_rgb_func_with_alpha(rest: &str, _has_alpha_in_name: bool) -> Option<(u8, u8, u8, f64)> {
    let rest = rest.trim_start();
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?;
    let parts: Vec<&str> = inner
        .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() < 3 { return None; }
    let r = parse_channel_byte(parts[0])?;
    let g = parse_channel_byte(parts[1])?;
    let b = parse_channel_byte(parts[2])?;
    let a = if parts.len() >= 4 {
        parse_alpha_float(parts[3])?
    } else {
        1.0
    };
    Some((r, g, b, a))
}

fn parse_hsl_func_with_alpha(rest: &str, _has_alpha_in_name: bool) -> Option<(u8, u8, u8, f64)> {
    let rest = rest.trim_start();
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?;
    let parts: Vec<&str> = inner
        .split(|c: char| c == ',' || c == '/' || c.is_whitespace())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() < 3 { return None; }
    let h: f64 = parts[0].trim_end_matches("deg").parse().ok()?;
    let s_pct: f64 = parts[1].trim_end_matches('%').parse().ok()?;
    let l_pct: f64 = parts[2].trim_end_matches('%').parse().ok()?;
    let a = if parts.len() >= 4 { parse_alpha_float(parts[3])? } else { 1.0 };
    let (r, g, b) = hsl_to_rgb_js(h, s_pct, l_pct);
    Some((r, g, b, a))
}

/// JS `hslToRgb` in `conic.ts` (component math performed in float, then
/// `Math.round` per channel).
fn hsl_to_rgb_js(h: f64, s_pct: f64, l_pct: f64) -> (u8, u8, u8) {
    let s = s_pct / 100.0;
    let l = l_pct / 100.0;
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = l - c / 2.0;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    (
        js_round_to_byte((r1 + m) * 255.0),
        js_round_to_byte((g1 + m) * 255.0),
        js_round_to_byte((b1 + m) * 255.0),
    )
}

fn parse_channel_byte(s: &str) -> Option<u8> {
    let s = s.trim();
    if let Some(p) = s.strip_suffix('%') {
        let v: f64 = p.parse().ok()?;
        Some((v / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let v: f64 = s.parse().ok()?;
        Some(v.round().clamp(0.0, 255.0) as u8)
    }
}

fn parse_alpha_float(s: &str) -> Option<f64> {
    let s = s.trim();
    if let Some(p) = s.strip_suffix('%') {
        let v: f64 = p.parse().ok()?;
        Some((v / 100.0).clamp(0.0, 1.0))
    } else {
        let v: f64 = s.parse().ok()?;
        Some(v.clamp(0.0, 1.0))
    }
}

/// JS `Math.round`: ties round toward +∞. For positive inputs (which is
/// the only range we feed in here) Rust's `f64::round()` rounds away from
/// zero — same direction for positives. We still bracket to u8 explicitly.
fn js_round_to_byte(v: f64) -> u8 {
    if v.is_nan() {
        return 0;
    }
    v.round().clamp(0.0, 255.0) as u8
}

/// Port of `interpolateColor` in `conic.ts`. `stops` is the normalized
/// stop list (offsets resolved into 0..1); `parsed_colors` is `stops` but
/// pre-parsed to RGBA tuples; `hints` is the matching list of optional
/// transition hints, sized to `stops.len()`.
fn interpolate_conic_color(
    t: f64,
    stops: &[ResolvedStop],
    parsed_colors: &[Option<(u8, u8, u8, f64)>],
    hints: Option<&[Option<f64>]>,
) -> String {
    if stops.is_empty() {
        return "transparent".to_string();
    }
    if stops.len() == 1 {
        return match parsed_colors[0] {
            Some(c) => format_rgba_js(c),
            None => stops[0].color.clone(),
        };
    }

    // Locate the segment [i, i+1] containing t. The JS code clamps to
    // the endpoints if t falls outside the range.
    let mut i: usize = 0;
    let first_off = stops[0].offset.unwrap_or(0.0);
    let last_off = stops[stops.len() - 1].offset.unwrap_or(1.0);
    if t <= first_off {
        i = 0;
    } else if t >= last_off {
        i = stops.len() - 2;
    } else {
        while i < stops.len() - 1 {
            let next_off = stops[i + 1].offset.unwrap_or(1.0);
            if next_off <= t { i += 1; } else { break; }
        }
    }
    if i >= stops.len() - 1 {
        i = stops.len() - 2;
    }

    let s1 = &stops[i];
    let s2 = &stops[i + 1];
    let c1 = match parsed_colors[i] { Some(c) => c, None => return s1.color.clone() };
    let c2 = match parsed_colors[i + 1] { Some(c) => c, None => return s1.color.clone() };

    let o1 = s1.offset.unwrap_or(0.0);
    let o2 = s2.offset.unwrap_or(1.0);
    if o1 == o2 {
        return format_rgba_js(c2);
    }

    let mut local_t = ((t - o1) / (o2 - o1)).clamp(0.0, 1.0);

    if let Some(hs) = hints {
        if let Some(hint_val) = hs.get(i).copied().flatten() {
            let range = o2 - o1;
            let h = (hint_val - o1) / range;
            if h <= 0.0 {
                local_t = if local_t > 0.0 { 1.0 } else { 0.0 };
            } else if h >= 1.0 {
                local_t = if local_t >= 1.0 { 1.0 } else { 0.0 };
            } else {
                let p = (0.5_f64).ln() / h.ln();
                local_t = local_t.powf(p);
            }
        }
    }

    let r = js_round_to_byte(c1.0 as f64 + (c2.0 as f64 - c1.0 as f64) * local_t);
    let g = js_round_to_byte(c1.1 as f64 + (c2.1 as f64 - c1.1 as f64) * local_t);
    let b = js_round_to_byte(c1.2 as f64 + (c2.2 as f64 - c1.2 as f64) * local_t);
    let a = c1.3 + (c2.3 - c1.3) * local_t;
    format_rgba_js((r, g, b, a))
}

/// JS `formatRGBA`: `rgb(r,g,b)` when alpha is exactly 1, else
/// `rgba(r,g,b,a)` with `a` formatted via JS NumberToString.
fn format_rgba_js(c: (u8, u8, u8, f64)) -> String {
    if c.3 == 1.0 {
        format!("rgb({},{},{})", c.0, c.1, c.2)
    } else {
        format!("rgba({},{},{},{})", c.0, c.1, c.2, js_number_to_string_f64(c.3))
    }
}

// ----- stop normalization (port of `gradient/utils.ts`) ---------------

#[derive(Clone)]
struct ResolvedStop {
    color: String,
    offset: Option<f64>,
}

fn normalize_stops(
    total_length: f64,
    color_stops: &[ColorStop],
    repeating: bool,
    from: From,
) -> Vec<ResolvedStop> {
    let mut stops: Vec<ResolvedStop> = Vec::new();
    let last = color_stops.last();
    let total_percentage = match last {
        Some(s) if repeating => match s.offset.as_ref() {
            Some(o) if o.unit == "%" => o.value.parse::<f64>().unwrap_or(100.0),
            _ => 100.0,
        },
        _ => 100.0,
    };

    for stop in color_stops {
        let color = stop.color.clone();
        if stops.is_empty() {
            stops.push(ResolvedStop {
                offset: Some(0.0),
                color: color.clone(),
            });
            if stop.offset.is_none() {
                continue;
            }
            if let Some(o) = stop.offset.as_ref() {
                if o.value == "0" {
                    continue;
                }
            }
        }
        let offset: Option<f64> = match stop.offset.as_ref() {
            None => None,
            Some(o) if o.unit == "%" => Some(o.value.parse::<f64>().unwrap_or(0.0) / total_percentage),
            Some(o) => {
                let n = length_to_number_px(&o.value, &o.unit, total_length);
                Some(n / total_length)
            }
        };
        stops.push(ResolvedStop { offset, color });
    }
    if stops.is_empty() {
        stops.push(ResolvedStop {
            offset: Some(0.0),
            color: "transparent".to_string(),
        });
    }

    let last_idx = stops.len() - 1;
    let last_offset = stops[last_idx].offset;
    if last_offset != Some(1.0) {
        match last_offset {
            None => stops[last_idx].offset = Some(1.0),
            Some(_) if repeating => {
                let c = stops[last_idx].color.clone();
                stops[last_idx] = ResolvedStop {
                    offset: Some(1.0),
                    color: c,
                };
            }
            Some(_) => {
                let c = stops[last_idx].color.clone();
                stops.push(ResolvedStop {
                    offset: Some(1.0),
                    color: c,
                });
            }
        }
    }

    let mut previous_stop = 0usize;
    let mut next_stop = 1usize;
    let n = stops.len();
    for i in 0..n {
        if stops[i].offset.is_none() {
            if next_stop < i {
                next_stop = i;
            }
            while next_stop < n && stops[next_stop].offset.is_none() {
                next_stop += 1;
            }
            if next_stop < n {
                let prev = stops[previous_stop].offset.unwrap_or(0.0);
                let nxt = stops[next_stop].offset.unwrap_or(1.0);
                stops[i].offset = Some(
                    (nxt - prev) / ((next_stop - previous_stop) as f64) * ((i - previous_stop) as f64)
                        + prev,
                );
            }
        } else {
            previous_stop = i;
        }
    }

    // `from === 'mask'` (JS gradient/utils.ts) — every stop becomes
    // `rgba(0,0,0,1)` (alpha=0 origin) or `rgba(255,255,255,alpha)`
    // (alpha>0 origin) so a luminance/alpha mask falls out.
    if matches!(from, From::Mask) {
        for s in &mut stops {
            let (_, _, _, alpha) = parse_color_with_alpha(&s.color).unwrap_or((0, 0, 0, 1.0));
            if alpha == 0.0 {
                s.color = "rgba(0, 0, 0, 1)".to_string();
            } else {
                s.color = format!(
                    "rgba(255, 255, 255, {})",
                    js_number_to_string_f64(alpha)
                );
            }
        }
    }

    stops
}

fn length_to_number_px(value: &str, unit: &str, base_length: f64) -> f64 {
    let n: f64 = value.parse().unwrap_or(0.0);
    match unit {
        "px" | "" => n,
        "%" => base_length * n / 100.0,
        "em" => n * 16.0,
        "rem" => n * 16.0,
        // JS `lengthToNumber` routes angle units through `calcDegree`,
        // returning a value in degrees. Conic-gradient stop normalization
        // relies on this so e.g. `0.5turn` -> 180 (degrees), not 0.5.
        "deg" => n,
        "rad" => n * 180.0 / std::f64::consts::PI,
        "turn" => n * 360.0,
        "grad" => n * 0.9,
        _ => n,
    }
}

fn calc_degree(value: &str, unit: &str) -> f64 {
    let n: f64 = value.parse().unwrap_or(0.0);
    match unit {
        "deg" | "" => n,
        "rad" => n * 180.0 / std::f64::consts::PI,
        "turn" => n * 360.0,
        "grad" => n * 0.9,
        _ => n,
    }
}
