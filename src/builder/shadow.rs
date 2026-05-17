//! Port of `src/builder/shadow.ts` — the box-shadow path.
//!
//! `box_shadow` wraps an element's shape with one or more SVG filters
//! that approximate CSS `box-shadow`. For each entry it emits:
//!
//!   * a `<mask>` (and optional negative-spread mask) cut to the
//!     element's contour, so the blur stops at the element's edges;
//!   * a `<defs>` containing a `<filter>` with `feGaussianBlur` →
//!     `feFlood` → `feComposite` that colors the alpha into the
//!     shadow color;
//!   * a `<g mask=… filter=… opacity=…>` wrapping a *shifted* copy of
//!     the shape that produces the visible shadow.
//!
//! Filter primitive order, attribute order, and JS number formatting
//! all mirror upstream byte-for-byte so resvg renders pixel-identical
//! output. The function returns `(outer_shadow, inner_shadow)` mirroring
//! the JS `[shadow, innerShadow]` shape: the caller emits `outer_shadow`
//! *before* the element shape and `inner_shadow` *after* it.
//!
//! Text-shadow (`buildDropShadow` in JS) lives in `build_drop_shadow`
//! below; the text emitter calls it when `parentStyle.textShadowOffset`
//! is set and wraps the resulting `<filter>` in a `<defs>`. The text
//! glyph `<g>` is then wrapped in `<g filter="url(#satori_s-{id})">`.

use crate::css::style::{ComputedStyle, TextShadow};

use super::xml::{build_xml, js_number_to_string, AttrValue};

pub struct BoxShadowArgs<'a> {
    pub id: &'a str,
    pub width: f32,
    pub height: f32,
    /// Element opacity to copy onto the shadow `<g>` wrapper (the JS
    /// emits `opacity={opacity}` unconditionally, including `opacity="1"`).
    pub opacity: f32,
    /// The shape string used both as the mask body and (after shifting
    /// by each shadow's offset) the shadow body. Construct it the same
    /// way upstream `rect.ts` does:
    ///
    /// ```text
    /// build_xml(type_tag, &[
    ///     ("x", left), ("y", top),
    ///     ("width", width), ("height", height),
    ///     ("fill", "#fff"), ("stroke", "#fff"), ("stroke-width", 0),
    ///     ("d", path),
    ///     ("transform", matrix),
    ///     ("clip-path", current_clip_path),
    ///     ("mask", mask_id),
    /// ], None)
    /// ```
    pub shape: &'a str,
}

/// Returns `(outer_shadow_xml, inner_shadow_xml)`. Both are empty if the
/// element has no `box-shadow`. The caller emits `outer_shadow` before
/// the element shape and `inner_shadow` after it (matching the JS rect
/// emit order: `defs + outer + shape + inner`).
pub fn box_shadow(args: &BoxShadowArgs<'_>, style: &ComputedStyle) -> Option<(String, String)> {
    let shadows = style.box_shadow.as_ref()?;
    if shadows.is_empty() {
        return None;
    }

    let mut outer = String::new();
    let mut inner = String::new();

    // JS iterates last → first so that the first listed shadow ends up
    // emitted last (i.e. rendered on top, matching CSS box-shadow stacking).
    for i in (0..shadows.len()).rev() {
        let sh = &shadows[i];

        // Mirror JS: `if (spreadRadius && inset) spreadRadius = -spreadRadius`.
        // We compute a local value rather than mutating the parsed style.
        let spread = if sh.inset && sh.spread != 0.0 {
            -sh.spread
        } else {
            sh.spread
        };

        let grow = (sh.blur * sh.blur) / 4.0 + spread;

        let offx_inset = if sh.inset { sh.offset_x } else { 0.0 };
        let offy_inset = if sh.inset { sh.offset_y } else { 0.0 };
        let left = (-grow - offx_inset).min(0.0);
        let right = (grow + args.width - offx_inset).max(args.width);
        let top = (-grow - offy_inset).min(0.0);
        let bottom = (grow + args.height - offy_inset).max(args.height);

        let sid = format!("satori_s-{}-{}", args.id, i);
        let mask_id = format!("satori_ms-{}-{}", args.id, i);

        // `shapeWithSpread = spread ? shape.replace(stroke-width="0", stroke-width="<spread*2>") : shape`
        let shape_with_spread = if spread != 0.0 {
            replace_first(
                args.shape,
                "stroke-width=\"0\"",
                &format!("stroke-width=\"{}\"", js_number_to_string(spread * 2.0)),
            )
        } else {
            args.shape.to_string()
        };

        let mut s = String::new();

        // Mask:
        //   <mask id="…" maskUnits="userSpaceOnUse">
        //     <rect x="0" y="0" width="<vw|100%>" height="<vh|100%>" fill="<#fff or #000>"/>
        //     <shape with fill flipped and stroke="#fff" stripped>
        //   </mask>
        let mask_bg_w_attr: AttrValue = match style._viewport_width {
            Some(w) => AttrValue::Int(w as i64),
            None => AttrValue::Str("100%"),
        };
        let mask_bg_h_attr: AttrValue = match style._viewport_height {
            Some(h) => AttrValue::Int(h as i64),
            None => AttrValue::Str("100%"),
        };
        let mask_bg = build_xml(
            "rect",
            &[
                ("x", AttrValue::Int(0)),
                ("y", AttrValue::Int(0)),
                ("width", mask_bg_w_attr),
                ("height", mask_bg_h_attr),
                ("fill", AttrValue::Str(if sh.inset { "#000" } else { "#fff" })),
            ],
            None,
        );
        // Shape with `fill="#fff"` → either `fill="#fff"` (inset) or
        // `fill="#000"` (outset), and `stroke="#fff"` stripped entirely.
        let mask_shape = {
            let fill_target = if sh.inset { "fill=\"#fff\"" } else { "fill=\"#000\"" };
            let with_fill = replace_first(&shape_with_spread, "fill=\"#fff\"", fill_target);
            replace_first(&with_fill, "stroke=\"#fff\"", "")
        };
        let mask_inner = format!("{mask_bg}{mask_shape}");
        s.push_str(&build_xml(
            "mask",
            &[
                ("id", AttrValue::Str(mask_id.as_str())),
                ("maskUnits", AttrValue::Str("userSpaceOnUse")),
            ],
            Some(&mask_inner),
        ));

        // Shift `d`, `x`, `y` by the shadow's offset (path move-to start
        // first, then the `x`/`y` attributes — `d` only matters for the
        // `<path>` shape but the regex no-ops on `<rect>`).
        let final_shape_base = shift_shape(&shape_with_spread, sh.offset_x, sh.offset_y);

        // Negative spread: an extra mask carved out by the shifted shape
        // restored to a stroke of `-spread*2`, wrapped around the shifted
        // shape so only the "interior of the original outline" remains.
        let final_shape = if spread < 0.0 {
            let neg_mask_id = format!("{mask_id}-neg");
            let neg_body = {
                let s1 = replace_first(&final_shape_base, "stroke=\"#fff\"", "stroke=\"#000\"");
                replace_stroke_width(
                    &s1,
                    &format!("stroke-width=\"{}\"", js_number_to_string(-spread * 2.0)),
                )
            };
            s.push_str(&build_xml(
                "mask",
                &[
                    ("id", AttrValue::Str(neg_mask_id.as_str())),
                    ("maskUnits", AttrValue::Str("userSpaceOnUse")),
                ],
                Some(&neg_body),
            ));
            build_xml(
                "g",
                &[("mask", AttrValue::Owned(format!("url(#{neg_mask_id})")))],
                Some(&final_shape_base),
            )
        } else {
            final_shape_base
        };

        // Filter:
        //   <defs>
        //     <filter id=… x=…% y=…% width=…% height=…%>
        //       <feGaussianBlur stdDeviation=<blur/2> result="b"/>
        //       <feFlood flood-color=<color> in="SourceGraphic" result="f"/>
        //       <feComposite in="f" in2="b" operator=<in|out>/>
        //     </filter>
        //   </defs>
        let filter_children = format!(
            "{}{}{}",
            build_xml(
                "feGaussianBlur",
                &[
                    ("stdDeviation", AttrValue::Number(sh.blur / 2.0)),
                    ("result", AttrValue::Str("b")),
                ],
                None,
            ),
            build_xml(
                "feFlood",
                &[
                    ("flood-color", AttrValue::Str(sh.color.as_str())),
                    ("in", AttrValue::Str("SourceGraphic")),
                    ("result", AttrValue::Str("f")),
                ],
                None,
            ),
            build_xml(
                "feComposite",
                &[
                    ("in", AttrValue::Str("f")),
                    ("in2", AttrValue::Str("b")),
                    ("operator", AttrValue::Str(if sh.inset { "out" } else { "in" })),
                ],
                None,
            ),
        );

        // Percentage math in f64 so values like (120/50)*100 = 240 stay
        // exact — f32 would round-trip to 240.00001525878906 and we'd
        // emit a different attribute string than JS satori.
        let (lw, rh) = (args.width as f64, args.height as f64);
        let filter_x = format!(
            "{}%",
            js_f64_to_string((left as f64 / lw) * 100.0)
        );
        let filter_y = format!(
            "{}%",
            js_f64_to_string((top as f64 / rh) * 100.0)
        );
        let filter_w = format!(
            "{}%",
            js_f64_to_string(((right as f64 - left as f64) / lw) * 100.0)
        );
        let filter_h = format!(
            "{}%",
            js_f64_to_string(((bottom as f64 - top as f64) / rh) * 100.0)
        );

        let filter_elem = build_xml(
            "filter",
            &[
                ("id", AttrValue::Str(sid.as_str())),
                ("x", AttrValue::Owned(filter_x)),
                ("y", AttrValue::Owned(filter_y)),
                ("width", AttrValue::Owned(filter_w)),
                ("height", AttrValue::Owned(filter_h)),
            ],
            Some(&filter_children),
        );

        s.push_str(&build_xml("defs", &[], Some(&filter_elem)));
        s.push_str(&build_xml(
            "g",
            &[
                ("mask", AttrValue::Owned(format!("url(#{mask_id})"))),
                ("filter", AttrValue::Owned(format!("url(#{sid})"))),
                ("opacity", AttrValue::Number(args.opacity)),
            ],
            Some(&final_shape),
        ));

        if sh.inset {
            inner.push_str(&s);
        } else {
            outer.push_str(&s);
        }
    }

    Some((outer, inner))
}

/// Mirror of JS satori's `buildDropShadow` (`reference/builder/shadow.ts`).
/// Takes the measured text bounding box (`width`, `height`) and the
/// parent's text-shadow list and returns the `<filter>` element that
/// the text emitter wraps in `<defs>` — *without* the `<defs>` wrapper.
///
/// `transparent_text` controls whether the `<feMerge>` includes the
/// `SourceGraphic` (false → text is fully transparent so we render
/// only the shadow, no glyph fill).
pub fn build_drop_shadow(
    id: &str,
    width: f32,
    height: f32,
    shadows: &[TextShadow],
    transparent_text: bool,
) -> String {
    if shadows.is_empty() {
        return String::new();
    }
    const SCALE: f64 = 1.1;
    let count = shadows.len();
    let multi_or_opaque = count > 1 || !transparent_text;

    let mut effects = String::new();
    let mut merge = String::new();

    // Normalize SourceAlpha to full opacity (CSS shadows render at
    // full intensity regardless of the source glyph's alpha).
    effects.push_str(&build_xml(
        "feComponentTransfer",
        &[
            ("in", AttrValue::Str("SourceAlpha")),
            ("result", AttrValue::Str("satori_sa_full")),
        ],
        Some(&build_xml(
            "feFuncA",
            &[
                ("type", AttrValue::Str("linear")),
                ("slope", AttrValue::Str("255")),
                ("intercept", AttrValue::Str("0")),
            ],
            None,
        )),
    ));

    let mut left = 0.0_f32;
    let mut right = width;
    let mut top = 0.0_f32;
    let mut bottom = height;

    for (i, sh) in shadows.iter().enumerate() {
        let grow = (sh.blur * sh.blur) / 4.0;
        left = (sh.offset_x - grow).min(left);
        right = (sh.offset_x + grow + width).max(right);
        top = (sh.offset_y - grow).min(top);
        bottom = (sh.offset_y + grow + height).max(bottom);

        let result_id = format!("satori_s-{id}-result-{i}");
        let result_blur = format!("{result_id}-blur");
        let result_offset = format!("{result_id}-offset");
        let result_color = format!("{result_id}-color");

        effects.push_str(&build_xml(
            "feGaussianBlur",
            &[
                ("in", AttrValue::Str("satori_sa_full")),
                ("stdDeviation", AttrValue::Number(sh.blur / 2.0)),
                ("result", AttrValue::Str(&result_blur)),
            ],
            None,
        ));
        effects.push_str(&build_xml(
            "feOffset",
            &[
                ("in", AttrValue::Str(&result_blur)),
                ("dx", AttrValue::Number(sh.offset_x)),
                ("dy", AttrValue::Number(sh.offset_y)),
                ("result", AttrValue::Str(&result_offset)),
            ],
            None,
        ));
        effects.push_str(&build_xml(
            "feFlood",
            &[
                ("flood-color", AttrValue::Str(sh.color.as_str())),
                ("flood-opacity", AttrValue::Int(1)),
                ("result", AttrValue::Str(&result_color)),
            ],
            None,
        ));
        let composite_result_attr = if multi_or_opaque {
            AttrValue::Str(&result_id)
        } else {
            AttrValue::Skip
        };
        effects.push_str(&build_xml(
            "feComposite",
            &[
                ("in", AttrValue::Str(&result_color)),
                ("in2", AttrValue::Str(&result_offset)),
                ("operator", AttrValue::Str("in")),
                ("result", composite_result_attr),
            ],
            None,
        ));

        if multi_or_opaque {
            // Merge in reverse order — JS satori prepends each new node.
            let merge_node = build_xml(
                "feMergeNode",
                &[("in", AttrValue::Str(&result_id))],
                None,
            );
            merge = format!("{merge_node}{merge}");
        }
    }

    if !transparent_text {
        merge.push_str(&build_xml(
            "feMergeNode",
            &[("in", AttrValue::Str("SourceGraphic"))],
            None,
        ));
    }

    let body = if merge.is_empty() {
        effects
    } else {
        format!(
            "{effects}{}",
            build_xml("feMerge", &[], Some(&merge))
        )
    };

    let lw = width as f64;
    let rh = height as f64;
    // JS uses `((left / width) * 100 * SCALE).toFixed(2) + '%'` etc.
    let fmt_pct = |n: f64| -> String {
        // toFixed(2) for pct values
        format!("{:.2}%", n)
    };
    let filter_x = fmt_pct((left as f64 / lw) * 100.0 * SCALE);
    let filter_y = fmt_pct((top as f64 / rh) * 100.0 * SCALE);
    let filter_w = fmt_pct(((right as f64 - left as f64) / lw) * 100.0 * SCALE);
    let filter_h = fmt_pct(((bottom as f64 - top as f64) / rh) * 100.0 * SCALE);

    build_xml(
        "filter",
        &[
            ("id", AttrValue::Owned(format!("satori_s-{id}"))),
            ("x", AttrValue::Owned(filter_x)),
            ("y", AttrValue::Owned(filter_y)),
            ("width", AttrValue::Owned(filter_w)),
            ("height", AttrValue::Owned(filter_h)),
        ],
        Some(&body),
    )
}

/// JS `${n}` for an f64 value — drops trailing `.0`s. Mirrors
/// `js_number_to_string` from `xml.rs` but operates in f64 so that
/// values computed from clean integers (like `(120.0/50.0)*100.0 = 240`)
/// don't pick up the f32 rounding wobble.
fn js_f64_to_string(n: f64) -> String {
    if n == n.trunc() && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

/// JS-style `s.replace(needle, replacement)` — first occurrence only.
fn replace_first(s: &str, needle: &str, replacement: &str) -> String {
    if let Some(idx) = s.find(needle) {
        let mut out = String::with_capacity(s.len() - needle.len() + replacement.len());
        out.push_str(&s[..idx]);
        out.push_str(replacement);
        out.push_str(&s[idx + needle.len()..]);
        out
    } else {
        s.to_string()
    }
}

/// JS regex `/stroke-width="[^"]+"/`: replace the entire first
/// `stroke-width="…"` attribute, regardless of its current value.
fn replace_stroke_width(s: &str, replacement: &str) -> String {
    let needle = "stroke-width=\"";
    let Some(start) = s.find(needle) else { return s.to_string(); };
    let after_open = start + needle.len();
    let Some(close_offset) = s[after_open..].find('"') else { return s.to_string(); };
    let close = after_open + close_offset;
    let mut out = String::with_capacity(s.len());
    out.push_str(&s[..start]);
    out.push_str(replacement);
    out.push_str(&s[close + 1..]);
    out
}

/// JS-equivalent shape shift:
/// ```text
/// .replace(/d="([^"]+)"/, (_, path) => 'd="' + shiftPath(path, dx, dy) + '"')
/// .replace(/x="([^"]+)"/, (_, x) => 'x="' + (parseFloat(x) + dx) + '"')
/// .replace(/y="([^"]+)"/, (_, y) => 'y="' + (parseFloat(y) + dy) + '"')
/// ```
fn shift_shape(shape: &str, dx: f32, dy: f32) -> String {
    let mut out = shift_d_attribute(shape, dx, dy);
    out = shift_numeric_attribute(&out, "x", dx);
    out = shift_numeric_attribute(&out, "y", dy);
    out
}

fn shift_numeric_attribute(s: &str, attr: &str, delta: f32) -> String {
    let needle = format!("{attr}=\"");
    let Some(start) = s.find(&needle) else { return s.to_string(); };
    let after_open = start + needle.len();
    let Some(close_offset) = s[after_open..].find('"') else { return s.to_string(); };
    let close = after_open + close_offset;
    let value_str = &s[after_open..close];
    let Ok(v) = value_str.parse::<f32>() else { return s.to_string(); };
    let new_v = v + delta;
    let mut out = String::with_capacity(s.len());
    out.push_str(&s[..after_open]);
    out.push_str(&js_number_to_string(new_v));
    out.push_str(&s[close..]);
    out
}

fn shift_d_attribute(s: &str, dx: f32, dy: f32) -> String {
    let needle = "d=\"";
    let Some(start) = s.find(needle) else { return s.to_string(); };
    let after_open = start + needle.len();
    let Some(close_offset) = s[after_open..].find('"') else { return s.to_string(); };
    let close = after_open + close_offset;
    let path = &s[after_open..close];
    let shifted = shift_path(path, dx, dy);
    let mut out = String::with_capacity(s.len() + shifted.len());
    out.push_str(&s[..after_open]);
    out.push_str(&shifted);
    out.push_str(&s[close..]);
    out
}

/// JS regex `/([MA])([0-9.-]+),([0-9.-]+)/g`, with the captured numbers
/// shifted by `(dx, dy)`. The character class `[0-9.-]+` is greedy and
/// matches digits, dots, and minus signs — we replicate the same
/// permissive behavior here. Only uppercase `M`/`A` are shifted
/// (absolute commands); the lowercase relative commands emitted by
/// `border-radius` are intentionally left alone.
fn shift_path(path: &str, dx: f32, dy: f32) -> String {
    let bytes = path.as_bytes();
    let mut out = String::with_capacity(path.len() + 16);
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        if (c == b'M' || c == b'A') && i + 1 < bytes.len() && is_num_char(bytes[i + 1]) {
            let (x_end, x_val) = read_num(bytes, i + 1);
            if x_end > i + 1 && x_end < bytes.len() && bytes[x_end] == b',' {
                let (y_end, y_val) = read_num(bytes, x_end + 1);
                if y_end > x_end + 1 {
                    out.push(c as char);
                    out.push_str(&js_number_to_string(x_val + dx));
                    out.push(',');
                    out.push_str(&js_number_to_string(y_val + dy));
                    i = y_end;
                    continue;
                }
            }
        }
        out.push(c as char);
        i += 1;
    }
    out
}

fn is_num_char(b: u8) -> bool {
    matches!(b, b'0'..=b'9' | b'-' | b'.' | b'+')
}

/// Greedy read of `[0-9.-]+`, returns `(end_index, parsed_value)`.
fn read_num(bytes: &[u8], start: usize) -> (usize, f32) {
    let mut i = start;
    while i < bytes.len() && is_num_char(bytes[i]) {
        i += 1;
    }
    if i == start {
        return (start, 0.0);
    }
    let s = std::str::from_utf8(&bytes[start..i]).unwrap_or("");
    // JS `parseFloat` parses the longest leading prefix; in our paths
    // the value is always a clean signed decimal so a direct parse is
    // sufficient.
    let v: f32 = s.parse().unwrap_or_else(|_| {
        // Fallback: scan a clean prefix `[-+]?[0-9]*\.?[0-9]*`.
        let mut j = 0usize;
        let bs = s.as_bytes();
        if !bs.is_empty() && (bs[0] == b'-' || bs[0] == b'+') {
            j += 1;
        }
        let mut saw_dot = false;
        while j < bs.len() {
            match bs[j] {
                b'0'..=b'9' => j += 1,
                b'.' if !saw_dot => {
                    saw_dot = true;
                    j += 1;
                }
                _ => break,
            }
        }
        s[..j].parse::<f32>().unwrap_or(0.0)
    });
    (i, v)
}
