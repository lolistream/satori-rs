//! Port of `src/builder/clip-path.ts` + `src/parser/shape.ts`.
//!
//! Parses CSS `clip-path: circle(...) | ellipse(...) | inset(...) |
//! polygon(...) | path(...)` against the element's box and emits the
//! resulting `<clipPath>` element.
//!
//! Geometry math is f64 to match JS arithmetic (we've seen single-ULP
//! differences in `circle r` shift several pixels after rasterization).

use super::xml::{build_xml, js_number_to_string_f64, AttrValue};

/// `circle(r at x y)`, `ellipse(rx ry at x y)`, etc. — geometry already
/// resolved against the box's (width, height, fontSize). `cx`/`cy` are
/// `Option` because JS satori's `lengthToNumber` returns `undefined`
/// for unrecognized tokens (e.g. `at top` leaves `cx` unset), and
/// `buildXMLString` then omits the attribute entirely.
#[derive(Debug, Clone)]
pub enum Shape {
    Circle { r: f64, cx: Option<f64>, cy: Option<f64> },
    Ellipse { rx: f64, ry: f64, cx: Option<f64>, cy: Option<f64> },
    Inset { x: f64, y: f64, width: f64, height: f64, path: Option<String> },
    Polygon { fill_rule: String, points: String },
    Path { fill_rule: String, d: String },
}

/// Parse the raw `clip-path` CSS string into a Shape. Returns None for
/// unsupported / malformed values.
pub fn parse_shape(raw: &str, width: f32, height: f32, font_size: f32) -> Option<Shape> {
    let raw = raw.trim();
    let width = width as f64;
    let height = height as f64;
    let fs = font_size as f64;

    if let Some(inner) = raw.strip_prefix("circle(").and_then(|s| s.strip_suffix(')')) {
        let inner = inner.trim();
        // Split into "<radius> at <position>".
        let (radius_part, pos_part) = match inner.split_once(" at ") {
            Some((r, p)) => (r.trim(), p.trim()),
            None => (inner, ""),
        };
        // Radius default reference: sqrt(w^2 + h^2)/sqrt(2) per spec.
        let radius_ref = (width.powi(2) + height.powi(2)).sqrt() / (2.0_f64).sqrt();
        let r = length_to_number(radius_part, fs, radius_ref).unwrap_or(0.0);
        let (cx, cy) = resolve_position_opt(pos_part, width, height, fs);
        return Some(Shape::Circle { r, cx, cy });
    }

    if let Some(inner) = raw.strip_prefix("ellipse(").and_then(|s| s.strip_suffix(')')) {
        let inner = inner.trim();
        let (radius_part, pos_part) = match inner.split_once(" at ") {
            Some((r, p)) => (r.trim(), p.trim()),
            None => (inner, ""),
        };
        let parts: Vec<&str> = radius_part.split_whitespace().collect();
        let rx_raw = parts.first().copied().unwrap_or("50%");
        let ry_raw = parts.get(1).copied().unwrap_or("50%");
        let rx = length_to_number(rx_raw, fs, width).unwrap_or(width / 2.0);
        let ry = length_to_number(ry_raw, fs, height).unwrap_or(height / 2.0);
        let (cx, cy) = resolve_position_opt(pos_part, width, height, fs);
        return Some(Shape::Ellipse { rx, ry, cx, cy });
    }

    if let Some(inner) = raw.strip_prefix("inset(").and_then(|s| s.strip_suffix(')')) {
        // `inset(<offsets> [round <radius>])` — when `round` is present,
        // compute a border-radius path and emit a `<path>`. Otherwise
        // emit a plain `<rect>`.
        let (offsets_str, radius_str) = match inner.split_once("round") {
            Some((o, r)) => (o.trim(), r.trim()),
            None => (inner.trim(), ""),
        };
        let parts: Vec<&str> = offsets_str.split_whitespace().collect();
        // Margin-shorthand expansion: 1, 2, 3 or 4 values.
        let (t, r, b, l) = match parts.len() {
            1 => {
                let v = length_to_number(parts[0], fs, height).unwrap_or(0.0);
                (v, length_to_number(parts[0], fs, width).unwrap_or(0.0), v, length_to_number(parts[0], fs, width).unwrap_or(0.0))
            }
            2 => {
                let v = length_to_number(parts[0], fs, height).unwrap_or(0.0);
                let h = length_to_number(parts[1], fs, width).unwrap_or(0.0);
                (v, h, v, h)
            }
            3 => {
                let t = length_to_number(parts[0], fs, height).unwrap_or(0.0);
                let h = length_to_number(parts[1], fs, width).unwrap_or(0.0);
                let b = length_to_number(parts[2], fs, height).unwrap_or(0.0);
                (t, h, b, h)
            }
            _ => {
                let t = length_to_number(parts.first().copied().unwrap_or("0"), fs, height).unwrap_or(0.0);
                let r = length_to_number(parts.get(1).copied().unwrap_or("0"), fs, width).unwrap_or(0.0);
                let b = length_to_number(parts.get(2).copied().unwrap_or("0"), fs, height).unwrap_or(0.0);
                let l = length_to_number(parts.get(3).copied().unwrap_or("0"), fs, width).unwrap_or(0.0);
                (t, r, b, l)
            }
        };
        let x = l;
        let y = t;
        let w = (width - l - r).max(0.0);
        let h = (height - t - b).max(0.0);
        if !radius_str.is_empty() && radius_str != "0" {
            let radius_parts: Vec<&str> = radius_str.split_whitespace().collect();
            let resolve_axis =
                |i: usize| -> (Option<f64>, Option<f64>) {
                    // CSS border-radius shorthand: 1/2/3/4 values.
                    let tl_h: Option<f64>;
                    let tr_h: Option<f64>;
                    let br_h: Option<f64>;
                    let bl_h: Option<f64>;
                    match radius_parts.len() {
                        0 => return (None, None),
                        1 => {
                            let v = length_to_number(radius_parts[0], fs, if i == 0 { height } else { width });
                            tl_h = v; tr_h = v; br_h = v; bl_h = v;
                        }
                        2 => {
                            let a = length_to_number(radius_parts[0], fs, if i == 0 { height } else { width });
                            let b = length_to_number(radius_parts[1], fs, if i == 0 { height } else { width });
                            tl_h = a; tr_h = b; br_h = a; bl_h = b;
                        }
                        3 => {
                            let a = length_to_number(radius_parts[0], fs, if i == 0 { height } else { width });
                            let b = length_to_number(radius_parts[1], fs, if i == 0 { height } else { width });
                            let c = length_to_number(radius_parts[2], fs, if i == 0 { height } else { width });
                            tl_h = a; tr_h = b; br_h = c; bl_h = b;
                        }
                        _ => {
                            let basis_v = if i == 0 { height } else { width };
                            let basis_h = if i == 0 { height } else { width };
                            let _ = basis_v;
                            tl_h = length_to_number(radius_parts[0], fs, basis_h);
                            tr_h = length_to_number(radius_parts[1], fs, basis_h);
                            br_h = length_to_number(radius_parts[2], fs, basis_h);
                            bl_h = length_to_number(radius_parts.get(3).copied().unwrap_or("0"), fs, basis_h);
                        }
                    }
                    let _ = (tr_h, br_h, bl_h);
                    (tl_h, None)
                };
            let _ = resolve_axis;
            // Build a synthetic ComputedStyle with the resolved radii so
            // we can reuse `radius_path`. JS only honours the horizontal
            // (h) component when given without a `/` separator — the spec
            // allows `<radius> / <radius>` for distinct h/v values, but
            // satori treats the round clause as a single-value shorthand.
            use crate::css::style::{ComputedStyle, RadiusLen, RadiusValue};
            fn make_radius(v: f64) -> RadiusValue {
                RadiusValue {
                    h: RadiusLen::Px(v as f32),
                    v: RadiusLen::Px(v as f32),
                    single: true,
                }
            }
            // Parse 1/2/3/4-value radius shorthand against the inset box
            // (`w` × `h`).
            let (tl, tr, br, bl) = match radius_parts.len() {
                1 => {
                    let v = length_to_number(radius_parts[0], fs, w.min(h)).unwrap_or(0.0);
                    (v, v, v, v)
                }
                2 => {
                    let a = length_to_number(radius_parts[0], fs, w.min(h)).unwrap_or(0.0);
                    let b = length_to_number(radius_parts[1], fs, w.min(h)).unwrap_or(0.0);
                    (a, b, a, b)
                }
                3 => {
                    let a = length_to_number(radius_parts[0], fs, w.min(h)).unwrap_or(0.0);
                    let b = length_to_number(radius_parts[1], fs, w.min(h)).unwrap_or(0.0);
                    let c = length_to_number(radius_parts[2], fs, w.min(h)).unwrap_or(0.0);
                    (a, b, c, b)
                }
                _ => {
                    let a = length_to_number(radius_parts[0], fs, w.min(h)).unwrap_or(0.0);
                    let b = length_to_number(radius_parts[1], fs, w.min(h)).unwrap_or(0.0);
                    let c = length_to_number(radius_parts[2], fs, w.min(h)).unwrap_or(0.0);
                    let d = length_to_number(
                        radius_parts.get(3).copied().unwrap_or("0"),
                        fs,
                        w.min(h),
                    )
                    .unwrap_or(0.0);
                    (a, b, c, d)
                }
            };
            let synthetic = ComputedStyle {
                border_top_left_radius: Some(make_radius(tl)),
                border_top_right_radius: Some(make_radius(tr)),
                border_bottom_right_radius: Some(make_radius(br)),
                border_bottom_left_radius: Some(make_radius(bl)),
                ..Default::default()
            };
            let d = super::border_radius::radius_path(x as f32, y as f32, w as f32, h as f32, &synthetic);
            if !d.is_empty() {
                return Some(Shape::Inset { x, y, width: w, height: h, path: Some(d) });
            }
        }
        return Some(Shape::Inset { x, y, width: w, height: h, path: None });
    }

    if let Some(inner) = raw.strip_prefix("polygon(").and_then(|s| s.strip_suffix(')')) {
        let (fill_rule, rest) = resolve_fill_rule(inner);
        let points_str = rest
            .split(',')
            .map(|pair| {
                let coords: Vec<&str> = pair.split_whitespace().collect();
                let x = length_to_number(coords.first().copied().unwrap_or("0"), fs, width).unwrap_or(0.0);
                let y = length_to_number(coords.get(1).copied().unwrap_or("0"), fs, height).unwrap_or(0.0);
                format!("{} {}", js_number_to_string_f64(x), js_number_to_string_f64(y))
            })
            .collect::<Vec<_>>()
            .join(", ");
        return Some(Shape::Polygon { fill_rule, points: points_str });
    }

    if let Some(inner) = raw.strip_prefix("path(").and_then(|s| s.strip_suffix(')')) {
        let (fill_rule, d) = resolve_fill_rule(inner);
        return Some(Shape::Path { fill_rule, d });
    }
    None
}

/// Build the `<clipPath id="satori_cp-{id}">…</clipPath>` element for
/// the given shape. The clip path is wrapped in a translate so its
/// inner geometry is element-local (matching JS satori).
pub fn build_clip_path(id: &str, left: f32, top: f32, current_clip_path: Option<&str>, shape: &Shape) -> String {
    let cp_id = format!("satori_cp-{id}");
    let body = match shape {
        Shape::Circle { r, cx, cy } => build_xml(
            "circle",
            &[
                ("r", AttrValue::NumberF64(*r)),
                ("cx", cx.map(AttrValue::NumberF64).unwrap_or(AttrValue::Skip)),
                ("cy", cy.map(AttrValue::NumberF64).unwrap_or(AttrValue::Skip)),
            ],
            None,
        ),
        Shape::Ellipse { rx, ry, cx, cy } => build_xml(
            "ellipse",
            &[
                ("rx", AttrValue::NumberF64(*rx)),
                ("ry", AttrValue::NumberF64(*ry)),
                ("cx", cx.map(AttrValue::NumberF64).unwrap_or(AttrValue::Skip)),
                ("cy", cy.map(AttrValue::NumberF64).unwrap_or(AttrValue::Skip)),
            ],
            None,
        ),
        Shape::Inset { x, y, width, height, path } => {
            if let Some(d) = path {
                build_xml("path", &[("d", AttrValue::Str(d.as_str()))], None)
            } else {
                build_xml(
                    "rect",
                    &[
                        ("x", AttrValue::NumberF64(*x)),
                        ("y", AttrValue::NumberF64(*y)),
                        ("width", AttrValue::NumberF64(*width)),
                        ("height", AttrValue::NumberF64(*height)),
                    ],
                    None,
                )
            }
        }
        Shape::Polygon { fill_rule, points } => build_xml(
            "polygon",
            &[
                ("fill-rule", AttrValue::Str(fill_rule.as_str())),
                ("points", AttrValue::Str(points.as_str())),
            ],
            None,
        ),
        Shape::Path { fill_rule, d } => build_xml(
            "path",
            &[
                ("fill-rule", AttrValue::Str(fill_rule.as_str())),
                ("d", AttrValue::Str(d.as_str())),
            ],
            None,
        ),
    };
    let mut attrs: Vec<(&str, AttrValue)> = vec![("id", AttrValue::Str(cp_id.as_str()))];
    if let Some(cp) = current_clip_path {
        attrs.push(("clip-path", AttrValue::Str(cp)));
    }
    let transform = format!(
        "translate({}, {})",
        js_number_to_string_f64(left as f64),
        js_number_to_string_f64(top as f64),
    );
    attrs.push(("transform", AttrValue::Owned(transform)));
    build_xml("clipPath", &attrs, Some(&body))
}

/// Public helper for callers that only need the id.
pub fn clip_path_id(id: &str) -> String {
    format!("satori_cp-{id}")
}
pub fn clip_path_url(id: &str) -> String {
    format!("url(#satori_cp-{id})")
}

fn resolve_fill_rule(inner: &str) -> (String, String) {
    let stripped = inner.replace(['\'', '"'], "");
    let trimmed = stripped.trim();
    if let Some(rest) = trimmed.strip_prefix("nonzero") {
        let d = rest.trim_start_matches(',').trim().to_string();
        return ("nonzero".to_string(), d);
    }
    if let Some(rest) = trimmed.strip_prefix("evenodd") {
        let d = rest.trim_start_matches(',').trim().to_string();
        return ("evenodd".to_string(), d);
    }
    ("nonzero".to_string(), trimmed.to_string())
}

/// Mirror of JS `resolvePosition` + `lengthToNumber`. Keywords win
/// over the initial slot value (`pos[0]` / `pos[1]` default to `"50%"`),
/// and any final value that's still a non-numeric string returns `None`.
fn resolve_position_opt(pos: &str, w: f64, h: f64, fs: f64) -> (Option<f64>, Option<f64>) {
    use TokenValue::{Keyword, Number};
    let parts: Vec<&str> = pos.split_whitespace().collect();
    enum TokenValue {
        Keyword,
        Number(f64),
    }
    fn parse_token(s: &str, base: f64, fs: f64) -> Option<f64> {
        length_to_number(s, fs, base)
    }
    let initial = |slot: Option<&&str>, base: f64| -> TokenValue {
        match slot {
            Some(t) => match parse_token(t, base, fs) {
                Some(n) => Number(n),
                None => Keyword,
            },
            None => Number(base / 2.0), // `'50%'` default
        }
    };
    let mut x = initial(parts.first(), w);
    let mut y = initial(parts.get(1), h);
    for &t in &parts {
        match t {
            "left" => x = Number(0.0),
            "right" => x = Number(w),
            "top" => y = Number(0.0),
            "bottom" => y = Number(h),
            "center" => {
                x = Number(w / 2.0);
                y = Number(h / 2.0);
            }
            _ => {}
        }
    }
    let cx = match x {
        Number(n) => Some(n),
        Keyword => None,
    };
    let cy = match y {
        Number(n) => Some(n),
        Keyword => None,
    };
    (cx, cy)
}

/// Mirror of `lengthToNumber` in `src/utils.ts` for the subset of units
/// the shape parser uses (px / em / rem / %, plus bare numbers).
pub fn length_to_number(s: &str, font_size: f64, percent_base: f64) -> Option<f64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    if let Some(p) = s.strip_suffix('%') {
        return p.parse::<f64>().ok().map(|v| v * percent_base / 100.0);
    }
    if let Some(p) = s.strip_suffix("px") {
        return p.trim().parse::<f64>().ok();
    }
    if let Some(p) = s.strip_suffix("rem") {
        return p.trim().parse::<f64>().ok().map(|v| v * 16.0);
    }
    if let Some(p) = s.strip_suffix("em") {
        return p.trim().parse::<f64>().ok().map(|v| v * font_size);
    }
    s.parse::<f64>().ok()
}
