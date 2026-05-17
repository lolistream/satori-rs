//! Snapshot tests sourced from `fixtures/dynamic-size__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn dynamic_size_should_render_image_with_dynamic_height() {
    run_fixture("dynamic-size__dynamic-size-should-render-image-with-dynamic-height__1.json");
}

#[test]
fn dynamic_size_should_render_image_with_dynamic_width() {
    run_fixture("dynamic-size__dynamic-size-should-render-image-with-dynamic-width__1.json");
}
