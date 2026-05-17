//! Snapshot tests sourced from `crates/satori-tests/fixtures/transform__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn transform_behavior_with_parent_overflow_should_not_inherit_parent_clip_path() {
    run_fixture("transform__transform-behavior-with-parent-overflow-should-not-inherit-parent-clip-path__1.json");
}

#[test]
fn transform_multiple_transforms_should_support_translate_rotate_and_scale() {
    run_fixture("transform__transform-multiple-transforms-should-support-translate-rotate-and-scale__1.json");
}

#[test]
fn transform_rotate_should_rotate_shape() {
    run_fixture("transform__transform-rotate-should-rotate-shape__1.json");
}

#[test]
fn transform_rotate_should_rotate_text_with_overflow() {
    run_fixture("transform__transform-rotate-should-rotate-text-with-overflow__1.json");
}

#[test]
fn transform_scale_should_scale_shape_in_two_directions() {
    run_fixture("transform__transform-scale-should-scale-shape-in-two-directions__1.json");
}

#[test]
fn transform_scale_should_scale_shape() {
    run_fixture("transform__transform-scale-should-scale-shape__1.json");
}

#[test]
fn transform_translate_should_support() {
    run_fixture("transform__transform-translate-should-support__1.json");
}

#[test]
fn transform_translate_should_translate_shape_in_x_axis() {
    run_fixture("transform__transform-translate-should-translate-shape-in-x-axis__1.json");
}

#[test]
fn transform_translate_should_translate_shape_in_y_axis() {
    run_fixture("transform__transform-translate-should-translate-shape-in-y-axis__1.json");
}

#[test]
fn transform_translate_should_translate_shape() {
    run_fixture("transform__transform-translate-should-translate-shape__1.json");
}
