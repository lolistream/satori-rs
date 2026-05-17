//! Snapshot tests sourced from `fixtures/background-clip__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn background_clip_should_preserve_color() {
    run_fixture("background-clip__background-clip-should-preserve-color__1.json");
}

#[test]
fn background_clip_should_render_background_clip_text_compatible_with_mask() {
    run_fixture("background-clip__background-clip-should-render-background-clip-text-compatible-with-mask__1.json");
}

#[test]
fn background_clip_should_render_background_clip_text_compatible_with_transform() {
    run_fixture("background-clip__background-clip-should-render-background-clip-text-compatible-with-transform__1.json");
}

#[test]
fn background_clip_should_render_background_clip_text() {
    run_fixture("background-clip__background-clip-should-render-background-clip-text__1.json");
}

#[test]
fn background_clip_should_render_webkit_text_fill_color_as_white_over_gradient() {
    run_fixture("background-clip__background-clip-should-render-webkit-text-fill-color-as-white-over-gradient__1.json");
}

#[test]
fn background_clip_should_render_webkit_text_fill_color_transparent_as_gradient_text() {
    run_fixture("background-clip__background-clip-should-render-webkit-text-fill-color-transparent-as-gradient-text__1.json");
}
