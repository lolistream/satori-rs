//! Shared test harness for the satori-rs port.
//!
//! - Provides `to_image(svg, width)` mirroring `test/utils.tsx`'s `toImage`.
//! - Provides `init_fonts()` mirroring the JS helper.
//! - Provides `assert_image_snapshot(png, name)` mirroring `jest-image-snapshot`.

pub mod harness;
pub mod snapshot;
