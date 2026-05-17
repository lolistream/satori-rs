//! Port of `src/builder/svg.ts`.
//!
//! JS:
//! ```ts
//! return buildXMLString('svg', {
//!   width, height,
//!   viewBox: `0 0 ${width} ${height}`,
//!   xmlns: 'http://www.w3.org/2000/svg',
//! }, content)
//! ```
//!
//! Importantly: JS coerces `width`/`height` numbers via `${n}`, so a width
//! of `100` is emitted as `"100"`, not `"100.0"`. The `xmlns` attribute is
//! emitted last and the order is fixed.

use crate::xml::{build_xml, js_number_to_string, AttrValue};

pub fn render_svg(width: f32, height: f32, content: &str) -> String {
    let w = js_number_to_string(width);
    let h = js_number_to_string(height);
    let viewbox = format!("0 0 {} {}", w, h);
    let attrs: Vec<(&str, AttrValue)> = vec![
        ("width", AttrValue::Owned(w)),
        ("height", AttrValue::Owned(h)),
        ("viewBox", AttrValue::Owned(viewbox)),
        ("xmlns", AttrValue::Str("http://www.w3.org/2000/svg")),
    ];
    build_xml("svg", &attrs, if content.is_empty() { None } else { Some(content) })
}
