//! Pre-flight sanity check that the in-process Rust rasterizer
//! (`usvg` -> `resvg::render` -> `tiny_skia` PNG encode) is
//! reachable and matches the vendored snapshots for a few
//! hand-written SVGs. If this fails, every snapshot-driven
//! downstream test will too.
//! If this file fails, every snapshot-based downstream test will
//! also fail (the failure is usually `npm install` not having been
//! run inside `xtask/` yet, or `node` missing from `PATH`).

mod common;
use common::harness::{assert_image_snapshot, to_image};

#[test]
fn rust_resvg_matches_empty_div_snapshot() {
    // src/builder/svg.ts wraps "" content as `<svg .../>` (empty content path).
    let svg = r#"<svg width="100" height="100" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"/>"#;
    let png = to_image(svg, 100);
    assert_image_snapshot(
        &png,
        "basic-test-tsx-test-basic-test-tsx-basic-should-render-empty-div-1-snap.png",
    );
}

#[test]
fn rust_resvg_matches_red_bg_snapshot() {
    let svg = r#"<svg width="100" height="100" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><rect x="0" y="0" width="100" height="100" fill="red"/></svg>"#;
    let png = to_image(svg, 100);
    assert_image_snapshot(
        &png,
        "basic-test-tsx-test-basic-test-tsx-basic-should-render-basic-div-with-background-color-1-snap.png",
    );
}

#[test]
fn rust_resvg_matches_hex_bg_snapshot() {
    let svg = r##"<svg width="100" height="100" viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg"><rect x="0" y="0" width="100" height="100" fill="#ff0"/></svg>"##;
    let png = to_image(svg, 100);
    assert_image_snapshot(
        &png,
        "basic-test-tsx-test-basic-test-tsx-basic-should-support-hex-colors-1-snap.png",
    );
}
