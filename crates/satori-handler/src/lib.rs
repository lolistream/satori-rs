//! Style handlers (port of `src/handler/*`).
//! TODO: compute, expand, inheritable, preprocess, presets, tailwind, variables.

pub mod image;

pub use image::{
    detect_content_type, resolve_image_asset_file, resolve_image_buffer, resolve_image_src,
    ResolvedImage,
};
