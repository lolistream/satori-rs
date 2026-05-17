//! Port of `reference/src/builder/mask-image.ts`.
//!
//! Produces `(mask_id, mask_xml)` for an element's `mask-image` style.
//! The mask reuses the existing `background_image` builder with
//! `from: From::Mask` so gradient color stops map to grayscale alpha.

use crate::css::style::{BackgroundImage, ResolvedUrlImage};

use super::background_image::{render_background_image_sub, From};
use super::xml::{build_xml, AttrValue};

/// Build the `<mask id="satori_mi-{id}">…</mask>` element for an
/// element's `mask-image` layer list. Returns `(mask_id, xml)`. The
/// caller emits `xml` inside `<defs>` and applies
/// `mask="url(#satori_mi-{id})"` on the rect/image.
pub fn build_mask_image(
    id: &str,
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    layers: &[BackgroundImage],
    mask_size: Option<&str>,
    mask_position: Option<&str>,
    mask_repeat: Option<&str>,
) -> Option<(String, String)> {
    if layers.is_empty() {
        return None;
    }
    // Skip URL layers that haven't been resolved yet (they would emit
    // a degenerate empty pattern). JS satori always has a `resolved`
    // payload by the time the builder runs because `preProcessNode`
    // resolves before rendering. We mirror that here.
    let mi_id = format!("satori_mi-{id}");

    let mut body = String::new();
    for (i, m) in layers.iter().enumerate() {
        // Skip URL layers without a resolved data URI (fetch failure).
        if let BackgroundImage::Url { resolved: None, .. } = m {
            continue;
        }
        let inner_id = format!("{mi_id}-{i}");
        let Some((pattern_id, def)) = render_background_image_sub(
            &inner_id,
            left,
            top,
            width,
            height,
            m,
            mask_size,
            mask_position,
            mask_repeat,
            From::Mask,
        ) else {
            continue;
        };
        body.push_str(&def);
        body.push_str(&build_xml(
            "rect",
            &[
                ("x", AttrValue::Number(left)),
                ("y", AttrValue::Number(top)),
                ("width", AttrValue::Number(width)),
                ("height", AttrValue::Number(height)),
                ("fill", AttrValue::Owned(format!("url(#{pattern_id})"))),
            ],
            None,
        ));
    }
    if body.is_empty() {
        return None;
    }
    let mask_el = build_xml(
        "mask",
        &[("id", AttrValue::Str(mi_id.as_str()))],
        Some(&body),
    );
    Some((mi_id, mask_el))
}

#[doc(hidden)]
pub fn __resolved_url_marker(_: &ResolvedUrlImage) {}
