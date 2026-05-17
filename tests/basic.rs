//! Snapshot tests sourced from `fixtures/basic__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn basic_should_combine_text_nodes_correctly() {
    run_fixture("basic__basic-should-combine-text-nodes-correctly__1.json");
}

#[test]
fn basic_should_render_basic_div_with_background_color() {
    run_fixture("basic__basic-should-render-basic-div-with-background-color__1.json");
}

#[test]
fn basic_should_render_basic_div_with_text_and_background_color() {
    run_fixture("basic__basic-should-render-basic-div-with-text-and-background-color__1.json");
}

#[test]
fn basic_should_render_basic_div_with_text() {
    run_fixture("basic__basic-should-render-basic-div-with-text__1.json");
}

#[test]
fn basic_should_render_empty_div() {
    run_fixture("basic__basic-should-render-empty-div__1.json");
}

#[test]
fn basic_should_respect_points_scale_factor() {
    run_fixture("basic__basic-should-respect-points-scale-factor__1.json");
}

#[test]
fn basic_should_support_array_in_jsx_children() {
    run_fixture("basic__basic-should-support-array-in-jsx-children__1.json");
}

#[test]
fn basic_should_support_custom_components() {
    run_fixture("basic__basic-should-support-custom-components__1.json");
}

#[test]
fn basic_should_support_custom_components_call2() {
    run_fixture("basic__basic-should-support-custom-components__2.json");
}

#[test]
fn basic_should_support_hex_colors() {
    run_fixture("basic__basic-should-support-hex-colors__1.json");
}

#[test]
fn basic_should_support_skipping_embedded_fonts() {
    run_fixture("basic__basic-should-support-skipping-embedded-fonts__1.json");
}
