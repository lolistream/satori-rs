//! SVG builder modules (port of `src/builder/*`).
//!
//! Currently implemented (minimal): svg root, rect (background only).
//! TODO: border, gradient, mask, transform, text-decoration, clip-path,
//!       background-image, shadow, content-mask, full rect.

pub mod background_image;
pub mod border;
pub mod border_radius;
pub mod clip_path;
pub mod mask_image;
pub mod rect;
pub mod shadow;
pub mod svg;
pub mod text;
pub mod text_decoration;
pub mod transform;
pub mod xml;

pub use background_image::render_background_image;
pub use mask_image::build_mask_image;
pub use rect::render_rect;
pub use shadow::{box_shadow, build_drop_shadow, BoxShadowArgs};
pub use svg::render_svg;
pub use text::{render_text, render_text_path, TextArgs, TextPathArgs};
pub use text_decoration::{build_decoration, DecorationArgs, GlyphBox};
pub use transform::{matrix_to_string, render_transform, resolve_effective};
pub use xml::{build_xml, AttrValue};
