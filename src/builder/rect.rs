//! Port of `src/builder/rect.ts` — incremental slice.
//!
//! Implemented:
//!   - background-color fill
//!   - border-radius (path replaces rect)
//!   - opacity wrapping
//!   - display: none short-circuit
//!   - borders (uniform — minimal subset of border.ts)
//!   - `<img>` emission (object-fit / object-position math) for the
//!     subset of cases used by the test suite.
//!   - always-emitted `satori_om-{id}` content mask (matches JS satori
//!     `overflow()` → `content-mask` flow, regardless of whether the
//!     element actually has text/img children that reference it).
//!
//! TODO: transform, mask, background-image (gradients), shadow.

use crate::css::style::{ComputedStyle, Display, ObjectFit};

use super::background_image::render_background_image;
use super::border::{render_border, BorderArgs};
use super::border_radius::radius_path;
use super::clip_path::{build_clip_path, clip_path_url, parse_shape};
use super::mask_image::build_mask_image;
use super::shadow::{box_shadow, BoxShadowArgs};
use super::xml::{build_xml, AttrValue};

/// Build the JS-satori `satori_om-{id}` content mask: a `<mask>` element
/// whose body is a `<rect>` covering the inner content area
/// (`border + padding` excluded when `has_src=true`, only `border`
/// excluded otherwise) filled with `#fff`. Mirrors `contentMask` in
/// `src/builder/content-mask.ts` for the no-`asContentMask`-border path.
pub fn build_content_mask(
    id: &str,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    style: &ComputedStyle,
    has_src: bool,
    matrix: Option<&str>,
) -> String {
    let border_left = style.border_left_width.unwrap_or(0.0) as f64;
    let border_top = style.border_top_width.unwrap_or(0.0) as f64;
    let border_right = style.border_right_width.unwrap_or(0.0) as f64;
    let border_bottom = style.border_bottom_width.unwrap_or(0.0) as f64;
    // `borderOnly` is the inverse of `has_src` in the JS overflow() call.
    let pad_left = if has_src { dim_px(style.padding_left) as f64 } else { 0.0 };
    let pad_top = if has_src { dim_px(style.padding_top) as f64 } else { 0.0 };
    let pad_right = if has_src { dim_px(style.padding_right) as f64 } else { 0.0 };
    let pad_bottom = if has_src { dim_px(style.padding_bottom) as f64 } else { 0.0 };

    let offset_left = border_left + pad_left;
    let offset_top = border_top + pad_top;
    let offset_right = border_right + pad_right;
    let offset_bottom = border_bottom + pad_bottom;

    let content_x = left as f64 + offset_left;
    let content_y = top as f64 + offset_top;
    let content_w = (width as f64 - offset_left - offset_right).max(0.0);
    let content_h = (height as f64 - offset_top - offset_bottom).max(0.0);

    // JS content-mask.ts: when _inheritedMaskId is set, the content
    // mask's inner rect also inherits the parent's mask. This is how
    // nested overflow:hidden / clip-path containers maintain the
    // outer mask through their own masks.
    let inherited_mask_url = style
        ._inherited_mask_id
        .as_deref()
        .map(|id| format!("url(#{id})"));
    // Mirror JS content-mask.ts: add the matrix transform to the inner
    // rect when `overflow: hidden && transform`.
    let is_overflow_hidden = matches!(style.overflow, Some(crate::css::style::Overflow::Hidden));
    let mask_rect_transform = if is_overflow_hidden && style.transform.is_some() {
        matrix
    } else {
        None
    };
    let inner_rect = build_xml(
        "rect",
        &[
            ("x", AttrValue::NumberF64(content_x)),
            ("y", AttrValue::NumberF64(content_y)),
            ("width", AttrValue::NumberF64(content_w)),
            ("height", AttrValue::NumberF64(content_h)),
            ("fill", AttrValue::Str("#fff")),
            ("transform", AttrValue::from(mask_rect_transform)),
            ("mask", AttrValue::from(inherited_mask_url.as_deref())),
        ],
        None,
    );
    // JS contentMask emits the directional border paths inside the
    // mask too (with `stroke="#000"`). This carves the border area
    // out of the mask so children don't paint over the border.
    let border_mask = super::border::content_mask_border(
        left, top, width, height, style, !has_src,
    );
    let inner = format!("{inner_rect}{border_mask}");
    build_xml(
        "mask",
        &[("id", AttrValue::Owned(format!("satori_om-{id}")))],
        Some(&inner),
    )
}

pub fn render_rect(
    id: &str,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    style: &ComputedStyle,
    transform_attr: Option<&str>,
) -> String {
    if matches!(style.display, Some(Display::None)) {
        return String::new();
    }

    // Compute the rounded-rect path (empty if no radius).
    let path_d = radius_path(left, top, width, height, style);
    let has_path = !path_d.is_empty();
    let has_src = style._src.is_some();

    // Compute the explicit `clip-path` shape (if any) resolved against
    // the element's box. The `<clipPath>` element is emitted in the
    // `clip` block below; the URL is also applied as `clip-path="..."`
    // to the fills.
    let clip_path_shape = style.clip_path.as_deref().and_then(|raw| {
        parse_shape(raw, width, height, style.font_size.unwrap_or(16.0))
    });
    let own_clip_path_url = if clip_path_shape.is_some() {
        Some(clip_path_url(id))
    } else {
        None
    };
    // Combine with an inherited clip-path from a parent that has
    // `overflow: hidden` (port of `_inheritedClipPathId`). When this
    // element owns a `background-clip: text` context, that takes
    // precedence so the background paints only behind glyphs.
    let current_clip_path: Option<String> = if style._bg_clip_text_self == Some(true) {
        Some(format!("url(#satori_bct-{id})"))
    } else {
        own_clip_path_url.clone().or_else(|| {
            style
                ._inherited_clip_path_id
                .as_deref()
                .map(|id| format!("url(#{id})"))
        })
    };

    let mut fills: Vec<String> = Vec::new();
    if let Some(c) = &style.background_color {
        fills.push(c.clone());
    }

    // Background images: each layer becomes its own <pattern> in defs +
    // an additional `url(#...)` fill stacked on top of the bg color.
    // Per CSS, the first layer paints on top, so we walk the array
    // forward and unshift fills to mirror JS satori's order.
    let mut bg_defs = String::new();
    if let Some(bgs) = &style.background_image {
        let mut new_fills: Vec<String> = Vec::new();
        for (index, bg) in bgs.iter().enumerate() {
            if let Some((pattern_id, defs)) = render_background_image(
                id,
                left,
                top,
                width,
                height,
                index,
                bg,
                style.background_size.as_deref(),
                style.background_position.as_deref(),
                style.background_repeat.as_deref(),
            ) {
                new_fills.insert(0, format!("url(#{pattern_id})"));
                bg_defs.push_str(&defs);
            }
        }
        fills.extend(new_fills);
    }

    let opacity = style.opacity.unwrap_or(1.0);

    // Build the per-element mask-image (`buildMaskImage` in JS).
    let (mi_id, mi_defs) = match style.mask_image.as_deref() {
        Some(layers) if !layers.is_empty() => {
            build_mask_image(
                id,
                left,
                top,
                width,
                height,
                layers,
                style.mask_size.as_deref(),
                style.mask_position.as_deref(),
                style.mask_repeat.as_deref(),
            )
            .map(|(id, xml)| (Some(id), xml))
            .unwrap_or((None, String::new()))
        }
        _ => (None, String::new()),
    };
    // JS rect.ts mask resolution:
    //   maskId = miId ? `url(#${miId})` : style._inheritedMaskId ? `url(#${...})` : undefined
    let mask_url: Option<String> = mi_id
        .as_deref()
        .map(|id| format!("url(#{id})"))
        .or_else(|| {
            style
                ._inherited_mask_id
                .as_deref()
                .map(|id| format!("url(#{id})"))
        });

    let mut shape = String::new();
    for fill in &fills {
        // Per JS rect.ts: `clip-path` / `mask` only go on the shape when
        // the element has no `transform` (transforms inherit them
        // through a wrapper `<g>` instead).
        let fill_clip = if style.transform.is_some() {
            None
        } else {
            current_clip_path.as_deref()
        };
        let fill_mask = if style.transform.is_some() {
            None
        } else {
            mask_url.as_deref()
        };
        let filter_style = style.filter.as_deref().map(|f| format!("filter:{f}"));
        if has_path {
            shape.push_str(&build_xml(
                "path",
                &[
                    ("x", AttrValue::Number(left)),
                    ("y", AttrValue::Number(top)),
                    ("width", AttrValue::Number(width)),
                    ("height", AttrValue::Number(height)),
                    ("fill", AttrValue::Str(fill.as_str())),
                    ("d", AttrValue::Owned(path_d.clone())),
                    ("transform", AttrValue::from(transform_attr)),
                    ("clip-path", AttrValue::from(fill_clip)),
                    ("mask", AttrValue::from(fill_mask)),
                    ("style", filter_style.as_deref().map(AttrValue::Str).unwrap_or(AttrValue::Skip)),
                ],
                None,
            ));
        } else {
            shape.push_str(&build_xml(
                "rect",
                &[
                    ("x", AttrValue::Number(left)),
                    ("y", AttrValue::Number(top)),
                    ("width", AttrValue::Number(width)),
                    ("height", AttrValue::Number(height)),
                    ("fill", AttrValue::Str(fill.as_str())),
                    ("transform", AttrValue::from(transform_attr)),
                    ("clip-path", AttrValue::from(fill_clip)),
                    ("mask", AttrValue::from(fill_mask)),
                    ("style", filter_style.as_deref().map(AttrValue::Str).unwrap_or(AttrValue::Skip)),
                ],
                None,
            ));
        }
    }

    // <img>: emit an `<image>` element on top of the background fills.
    // Mirrors the JS rect.ts `if (isImage)` branch.
    //
    // JS satori threads the image's clip-path/mask through `overflow()`,
    // which emits raw `<clipPath>` and `<mask>` elements AFTER `<defs>`
    // (not inside it). We replicate that ordering so the resulting SVG
    // is byte-identical with JS satori's.
    let mut image_clip = String::new();
    if let Some(src) = style._src.as_deref() {
        let (img_xml, clip_xml) =
            render_image_layer(id, left, top, width, height, src, style, transform_attr, mi_id.as_deref());
        shape.push_str(&img_xml);
        image_clip.push_str(&clip_xml);
    }

    // Borders. The JS implementation emits the border stroke after the
    // fill, sharing the same clip path.
    let (border_defs, border_body) = render_border(
        &BorderArgs {
            id,
            left,
            top,
            width,
            height,
            matrix: transform_attr.map(|s| s.to_string()),
            current_clip_path: None,
        },
        style,
    );
    shape.push_str(&border_body);

    // Box-shadow: build the JS-equivalent `shape` string (fill="#fff",
    // stroke="#fff", stroke-width="0") that boxShadow uses both as the
    // mask source and the shifted shadow body, then call box_shadow.
    let shadow_shape = if has_path {
        build_xml(
            "path",
            &[
                ("x", AttrValue::Number(left)),
                ("y", AttrValue::Number(top)),
                ("width", AttrValue::Number(width)),
                ("height", AttrValue::Number(height)),
                ("fill", AttrValue::Str("#fff")),
                ("stroke", AttrValue::Str("#fff")),
                ("stroke-width", AttrValue::Int(0)),
                ("d", AttrValue::Owned(path_d.clone())),
                ("transform", AttrValue::from(transform_attr)),
            ],
            None,
        )
    } else {
        build_xml(
            "rect",
            &[
                ("x", AttrValue::Number(left)),
                ("y", AttrValue::Number(top)),
                ("width", AttrValue::Number(width)),
                ("height", AttrValue::Number(height)),
                ("fill", AttrValue::Str("#fff")),
                ("stroke", AttrValue::Str("#fff")),
                ("stroke-width", AttrValue::Int(0)),
                ("transform", AttrValue::from(transform_attr)),
            ],
            None,
        )
    };
    let shadow = box_shadow(
        &BoxShadowArgs {
            id,
            width,
            height,
            opacity,
            shape: &shadow_shape,
        },
        style,
    );
    let (shadow_outer, shadow_inner) = shadow.unwrap_or_default();

    // Build the always-emitted content mask (JS overflow() returns it
    // unconditionally; resvg's compositing pre-allocates a backing
    // buffer for any declared mask, which subtly changes adjacent
    // antialiasing).
    let content_mask = build_content_mask(id, left, top, width, height, style, style._src.is_some(), transform_attr);

    // Explicit `clip-path` becomes a `<clipPath>` element (matches JS
    // overflow() → buildClipPath() order: clipPath FIRST, then mask).
    // The clipPath ELEMENT itself gets `clip-path="..."` set to the
    // currentClipPath the rect computed above — for own-clip-path this
    // becomes a self-reference (matches JS).
    let explicit_clip_path = clip_path_shape
        .as_ref()
        .map(|shape| {
            build_clip_path(id, left, top, current_clip_path.as_deref(), shape)
        })
        .unwrap_or_default();

    // When `overflow: hidden` is set OR the element has an `<img>`
    // src, JS `overflow()` emits an overflowClipPath: a box (or border-
    // radius path) that constrains children/the image to the element's
    // bounds. Its id is `satori_ocp-{id}` if an explicit clip-path was
    // already set on this element, else `satori_cp-{id}` (same shape
    // id used by children).
    let is_overflow_hidden =
        matches!(style.overflow, Some(crate::css::style::Overflow::Hidden));
    // The image case already emits its own clipPath + mask via
    // render_image_layer, so skip the second emit here.
    let needs_overflow_clip = is_overflow_hidden && !has_src;
    let overflow_clip_path = if needs_overflow_clip {
        let ocp_id = if clip_path_shape.is_some() {
            format!("satori_ocp-{id}")
        } else {
            format!("satori_cp-{id}")
        };
        // JS overflow.ts: the inner rect/path's `transform` is the
        // element's matrix when `overflow: hidden && transform`.
        let overflow_transform = if is_overflow_hidden && style.transform.is_some() {
            transform_attr
        } else {
            None
        };
        let shape_xml = if has_path {
            build_xml(
                "path",
                &[
                    ("x", AttrValue::Number(left)),
                    ("y", AttrValue::Number(top)),
                    ("width", AttrValue::Number(width)),
                    ("height", AttrValue::Number(height)),
                    ("d", AttrValue::Str(path_d.as_str())),
                    ("transform", AttrValue::from(overflow_transform)),
                ],
                None,
            )
        } else {
            build_xml(
                "rect",
                &[
                    ("x", AttrValue::Number(left)),
                    ("y", AttrValue::Number(top)),
                    ("width", AttrValue::Number(width)),
                    ("height", AttrValue::Number(height)),
                    ("transform", AttrValue::from(overflow_transform)),
                ],
                None,
            )
        };
        let mut attrs: Vec<(&str, AttrValue)> = vec![("id", AttrValue::Owned(ocp_id))];
        if let Some(cp) = current_clip_path.as_deref() {
            attrs.push(("clip-path", AttrValue::Str(cp)));
        }
        build_xml("clipPath", &attrs, Some(&shape_xml))
    } else {
        String::new()
    };

    if shape.is_empty()
        && border_defs.is_empty()
        && bg_defs.is_empty()
        && image_clip.is_empty()
        && shadow_outer.is_empty()
        && shadow_inner.is_empty()
        && content_mask.is_empty()
        && explicit_clip_path.is_empty()
        && overflow_clip_path.is_empty()
    {
        return String::new();
    }

    let combined_defs = format!("{bg_defs}{mi_defs}{border_defs}");
    let defs = if combined_defs.is_empty() {
        String::new()
    } else {
        format!("<defs>{}</defs>", combined_defs)
    };

    // JS rect.ts wraps transformed shapes in a `<g>` carrying the
    // inherited `currentClipPath` / `maskId` so the clip/mask is
    // evaluated in the parent's un-transformed coordinate space.
    let needs_transform_wrapper = style.transform.is_some()
        && (current_clip_path.is_some() || mask_url.is_some());
    let transform_wrapped = if needs_transform_wrapper {
        let mut open = String::from("<g");
        if let Some(cp) = current_clip_path.as_deref() {
            open.push_str(&format!(" clip-path=\"{}\"", cp));
        }
        if let Some(m) = mask_url.as_deref() {
            open.push_str(&format!(" mask=\"{}\"", m));
        }
        open.push('>');
        format!("{open}{shape}</g>")
    } else {
        shape
    };
    let wrapped = if (opacity - 1.0).abs() > f32::EPSILON {
        format!("<g opacity=\"{}\">{}</g>", opacity, transform_wrapped)
    } else {
        transform_wrapped
    };

    // JS rect.ts order:
    //   defs + shadow[0] + imageBorderRadius[0] + clip + (opacity_open
    //   + transform_open) + shapes + (transform_close + opacity_close)
    //   + shadow[1]
    //
    // `clip` (from `overflow()`) is: explicitClipPath + overflowClipPath
    //   + contentMask — for the image case `image_clip` already covers
    //   the clipPath + content mask pair, so we skip our own
    //   `content_mask` to avoid emitting two `satori_om-{id}` masks.
    let clip = if image_clip.is_empty() {
        format!("{explicit_clip_path}{overflow_clip_path}{content_mask}")
    } else {
        format!("{explicit_clip_path}{overflow_clip_path}{image_clip}")
    };

    format!("{defs}{shadow_outer}{clip}{wrapped}{shadow_inner}")
}

/// Port of the `isImage` branch in JS satori's `src/builder/rect.ts`.
///
/// Computes the `<image>` element geometry from `object-fit` /
/// `object-position` semantics, mirroring the JS algorithm verbatim so
/// that the resulting SVG is byte-identical for the same input.
///
/// All arithmetic runs at `f64` to match the JS `Number` type — `f32`
/// rounding introduces 1–2 ULP drift that shows up as visible pixel
/// differences after rasterization.
///
/// Returns `(image_xml, defs_xml)`. `defs_xml` always contains the
/// `clipPath` + content `mask` that JS satori's `overflow()` helper
/// emits whenever `src` is truthy — even on the simple
/// no-border/no-padding case where the clipPath is functionally a
/// no-op, because resvg's compositing path differs subtly depending on
/// whether `clip-path` is set on the `<image>` element.
fn render_image_layer(
    id: &str,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    src: &str,
    style: &ComputedStyle,
    transform_attr: Option<&str>,
    mask_image_id: Option<&str>,
) -> (String, String) {
    let left = left as f64;
    let top = top as f64;
    let width = width as f64;
    let height = height as f64;

    let border_left = style.border_left_width.unwrap_or(0.0) as f64;
    let border_top = style.border_top_width.unwrap_or(0.0) as f64;
    let border_right = style.border_right_width.unwrap_or(0.0) as f64;
    let border_bottom = style.border_bottom_width.unwrap_or(0.0) as f64;
    let padding_left = dim_px(style.padding_left) as f64;
    let padding_top = dim_px(style.padding_top) as f64;
    let padding_right = dim_px(style.padding_right) as f64;
    let padding_bottom = dim_px(style.padding_bottom) as f64;

    let offset_left = border_left + padding_left;
    let offset_top = border_top + padding_top;
    let offset_right = border_right + padding_right;
    let offset_bottom = border_bottom + padding_bottom;

    let container_inner_width = width - offset_left - offset_right;
    let container_inner_height = height - offset_top - offset_bottom;

    let position = style.object_position.as_deref().unwrap_or("center");
    let (obj_pos_x, obj_pos_y) =
        parse_object_position(position, container_inner_width, container_inner_height);

    let natural_width = style
        ._natural_width
        .map(|n| n as f64)
        .unwrap_or(container_inner_width);
    let natural_height = style
        ._natural_height
        .map(|n| n as f64)
        .unwrap_or(container_inner_height);

    // Default behavior (no object-fit) and `fill` both stretch.
    let mut image_width = container_inner_width;
    let mut image_height = container_inner_height;
    let mut image_x = left + offset_left;
    let mut image_y = top + offset_top;

    match style.object_fit {
        Some(ObjectFit::Contain) | Some(ObjectFit::Cover) => {
            let scale_x = container_inner_width / natural_width;
            let scale_y = container_inner_height / natural_height;
            let scale = if matches!(style.object_fit, Some(ObjectFit::Cover)) {
                scale_x.max(scale_y)
            } else {
                scale_x.min(scale_y)
            };
            image_width = natural_width * scale;
            image_height = natural_height * scale;
            image_x = left
                + offset_left
                + obj_pos_x
                - (image_width * obj_pos_x) / container_inner_width;
            image_y = top
                + offset_top
                + obj_pos_y
                - (image_height * obj_pos_y) / container_inner_height;
        }
        Some(ObjectFit::ScaleDown) => {
            if natural_width > 0.0 && natural_height > 0.0 {
                let scale_x = container_inner_width / natural_width;
                let scale_y = container_inner_height / natural_height;
                let min_scale = scale_x.min(scale_y);
                if min_scale >= 1.0 {
                    image_width = natural_width;
                    image_height = natural_height;
                } else {
                    image_width = natural_width * min_scale;
                    image_height = natural_height * min_scale;
                }
                image_x = left
                    + offset_left
                    + obj_pos_x
                    - (image_width * obj_pos_x) / container_inner_width;
                image_y = top
                    + offset_top
                    + obj_pos_y
                    - (image_height * obj_pos_y) / container_inner_height;
            } else {
                let scale_x = container_inner_width / natural_width;
                let scale_y = container_inner_height / natural_height;
                let scale = scale_x.min(scale_y);
                image_width = natural_width * scale;
                image_height = natural_height * scale;
                image_x = left
                    + offset_left
                    + obj_pos_x
                    - (image_width * obj_pos_x) / container_inner_width;
                image_y = top
                    + offset_top
                    + obj_pos_y
                    - (image_height * obj_pos_y) / container_inner_height;
            }
        }
        _ => {}
    }

    // Content area: border + padding excluded. Mirrors
    // `src/builder/content-mask.ts`'s `contentArea`.
    let content_x = left + offset_left;
    let content_y = top + offset_top;
    let content_w = container_inner_width;
    let content_h = container_inner_height;

    let clip_id = format!("satori_cp-{id}");
    let mask_id = format!("satori_om-{id}");

    // The overflowClipPath uses the FULL element box (border-radius
    // path included when present). Matches JS `overflow.ts` line 55.
    let outer_radius_path =
        radius_path(left as f32, top as f32, width as f32, height as f32, style);
    let clip_shape = if outer_radius_path.is_empty() {
        build_xml(
            "rect",
            &[
                ("x", AttrValue::NumberF64(left)),
                ("y", AttrValue::NumberF64(top)),
                ("width", AttrValue::NumberF64(width)),
                ("height", AttrValue::NumberF64(height)),
            ],
            None,
        )
    } else {
        build_xml(
            "path",
            &[
                ("x", AttrValue::NumberF64(left)),
                ("y", AttrValue::NumberF64(top)),
                ("width", AttrValue::NumberF64(width)),
                ("height", AttrValue::NumberF64(height)),
                ("d", AttrValue::Owned(outer_radius_path.clone())),
            ],
            None,
        )
    };
    let clip_path = build_xml(
        "clipPath",
        &[("id", AttrValue::Str(clip_id.as_str()))],
        Some(&clip_shape),
    );

    let mask_rect = build_xml(
        "rect",
        &[
            ("x", AttrValue::NumberF64(content_x)),
            ("y", AttrValue::NumberF64(content_y)),
            ("width", AttrValue::NumberF64(content_w)),
            ("height", AttrValue::NumberF64(content_h)),
            ("fill", AttrValue::Str("#fff")),
        ],
        None,
    );
    // Carve the border-shaped strokes out of the mask (so the image
    // doesn't paint over the border). Matches JS `content-mask.ts` +
    // `border.ts` `asContentMask: true` walk.
    let border_carve = super::border::content_mask_border(
        left as f32,
        top as f32,
        width as f32,
        height as f32,
        style,
        false,
    );
    let mask_body = format!("{mask_rect}{border_carve}");
    let mask = build_xml(
        "mask",
        &[("id", AttrValue::Str(mask_id.as_str()))],
        Some(&mask_body),
    );

    let defs = format!("{clip_path}{mask}");

    let clip_path_url = format!("url(#{clip_id})");
    // JS rect.ts: `mask: miId ? url(#miId) : url(#satori_om-{id})`.
    let mask_url = match mask_image_id {
        Some(mi) => format!("url(#{mi})"),
        None => format!("url(#{mask_id})"),
    };

    // JS satori always emits `preserveAspectRatio="none"` for the
    // <img>-on-rect path. The SVG renderer scales the embedded raster to
    // exactly the requested width/height, leaving aspect-ratio handling
    // entirely to our pre-computed `image_width` / `image_height`.
    //
    // When the element has a transform, JS satori applies the matrix
    // directly on `<image>` and replaces the content-area clip with a
    // dedicated border-radius clipPath (because the content-area mask
    // would be evaluated in the un-transformed coordinate space).
    // Without a transform we keep the simpler clip-path + mask pair.
    let has_transform = style.transform.is_some();

    if has_transform {
        // Build the border-radius clip path for the full image box, if
        // the element has any rounded corners. (Port of
        // `getBorderRadiusClipPath` in `src/builder/border-radius.ts`.)
        let outer_path = radius_path(left as f32, top as f32, width as f32, height as f32, style);
        let brc_id = format!("satori_brc-{id}");
        let inner = if outer_path.is_empty() {
            build_xml(
                "rect",
                &[
                    ("x", AttrValue::NumberF64(left)),
                    ("y", AttrValue::NumberF64(top)),
                    ("width", AttrValue::NumberF64(width)),
                    ("height", AttrValue::NumberF64(height)),
                ],
                None,
            )
        } else {
            // JS satori emits a `<path>` element with stale `x`/`y`/
            // `width`/`height` attrs alongside the `d` attr — odd but
            // required for byte-identical output.
            build_xml(
                "path",
                &[
                    ("x", AttrValue::NumberF64(left)),
                    ("y", AttrValue::NumberF64(top)),
                    ("width", AttrValue::NumberF64(width)),
                    ("height", AttrValue::NumberF64(height)),
                    ("d", AttrValue::Owned(outer_path.clone())),
                ],
                None,
            )
        };
        let radius_clip = build_xml(
            "clipPath",
            &[("id", AttrValue::Str(brc_id.as_str()))],
            Some(&inner),
        );
        // Only emit a `clip-path` attribute when we actually have a
        // border-radius — otherwise JS satori omits it (the transform
        // doesn't need clipping on its own).
        let has_radius = !outer_path.is_empty();
        let clip_attr = if has_radius {
            Some(format!("url(#{brc_id})"))
        } else {
            None
        };
        let mut attrs: Vec<(&str, AttrValue)> = vec![
            ("x", AttrValue::NumberF64(image_x)),
            ("y", AttrValue::NumberF64(image_y)),
            ("width", AttrValue::NumberF64(image_width)),
            ("height", AttrValue::NumberF64(image_height)),
            ("href", AttrValue::Str(src)),
            ("preserveAspectRatio", AttrValue::Str("none")),
            ("transform", AttrValue::from(transform_attr)),
        ];
        if let Some(attr) = clip_attr.as_deref() {
            attrs.push(("clip-path", AttrValue::Str(attr)));
        }
        if let Some(s) = image_filter_style(style) {
            attrs.push(("style", AttrValue::Owned(s)));
        }
        let image = build_xml("image", &attrs, None);
        let radius_defs = if has_radius {
            format!("<defs>{radius_clip}</defs>")
        } else {
            String::new()
        };
        return (image, radius_defs);
    }

    let mut attrs: Vec<(&str, AttrValue)> = vec![
        ("x", AttrValue::NumberF64(image_x)),
        ("y", AttrValue::NumberF64(image_y)),
        ("width", AttrValue::NumberF64(image_width)),
        ("height", AttrValue::NumberF64(image_height)),
        ("href", AttrValue::Str(src)),
        ("preserveAspectRatio", AttrValue::Str("none")),
        ("clip-path", AttrValue::Owned(clip_path_url)),
        ("mask", AttrValue::Owned(mask_url)),
    ];
    if let Some(s) = image_filter_style(style) {
        attrs.push(("style", AttrValue::Owned(s)));
    }
    let image = build_xml("image", &attrs, None);
    (image, defs)
}

fn dim_px(d: Option<crate::css::dimension::Dim>) -> f32 {
    match d {
        Some(crate::css::dimension::Dim::Px(n)) => n,
        _ => 0.0,
    }
}

/// Port of `parseObjectPosition` in `src/builder/rect.ts`.
///
/// Returns the (x, y) offset in pixels relative to the container's
/// inner content area, used by `cover` / `contain` / `scale-down` to
/// reposition the image within the container. f64 to match JS Number.
fn parse_object_position(position: &str, container_w: f64, container_h: f64) -> (f64, f64) {
    let lower = position.to_ascii_lowercase();
    let parts: Vec<&str> = lower.split_whitespace().collect();

    let keyword_to_percent = |kw: &str| -> Option<&'static str> {
        match kw {
            "left" | "top" => Some("0%"),
            "center" => Some("50%"),
            "right" | "bottom" => Some("100%"),
            _ => None,
        }
    };

    let (x_value, y_value): (String, String) = if parts.len() == 1 {
        let p = parts[0];
        match p {
            "left" | "center" | "right" => (
                keyword_to_percent(p).unwrap().to_string(),
                "50%".to_string(),
            ),
            "top" | "bottom" => ("50%".to_string(), keyword_to_percent(p).unwrap().to_string()),
            _ => (p.to_string(), "50%".to_string()),
        }
    } else {
        let first = parts[0];
        let second = parts[1];
        if first == "top" || first == "bottom" {
            let y_val = keyword_to_percent(first).unwrap();
            if second == "left" || second == "right" || second == "center" {
                let x_val = keyword_to_percent(second).unwrap();
                (x_val.to_string(), y_val.to_string())
            } else {
                ("50%".to_string(), y_val.to_string())
            }
        } else {
            let x_val = keyword_to_percent(first)
                .map(str::to_string)
                .unwrap_or_else(|| first.to_string());
            let y_val = keyword_to_percent(second)
                .map(str::to_string)
                .unwrap_or_else(|| second.to_string());
            (x_val, y_val)
        }
    };

    fn parse_value(s: &str, container: f64) -> f64 {
        let s = s.trim();
        if let Some(p) = s.strip_suffix('%') {
            return p.parse::<f64>().unwrap_or(0.0) * container / 100.0;
        }
        if let Some(p) = s.strip_suffix("px") {
            return p.parse::<f64>().unwrap_or(0.0);
        }
        if let Some(p) = s.strip_suffix("rem") {
            return p.parse::<f64>().unwrap_or(0.0) * 16.0;
        }
        if let Some(p) = s.strip_suffix("em") {
            return p.parse::<f64>().unwrap_or(0.0) * 16.0;
        }
        s.parse::<f64>().unwrap_or(0.0)
    }

    (
        parse_value(&x_value, container_w),
        parse_value(&y_value, container_h),
    )
}

fn image_filter_style(style: &ComputedStyle) -> Option<String> {
    let mut parts: Vec<String> = Vec::new();
    if let Some(f) = style.filter.as_deref() {
        parts.push(f.to_string());
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("filter:{}", parts.join(" ")))
    }
}


