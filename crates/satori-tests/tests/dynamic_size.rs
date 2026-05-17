//! Snapshot tests sourced from `crates/satori-tests/fixtures/dynamic-size__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn dynamic_size_should_render_image_with_dynamic_height() {
    run_fixture("dynamic-size__dynamic-size-should-render-image-with-dynamic-height__1.json");
}

#[test]
fn dynamic_size_should_render_image_with_dynamic_width() {
    run_fixture("dynamic-size__dynamic-size-should-render-image-with-dynamic-width__1.json");
}
