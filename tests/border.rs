//! Snapshot tests sourced from `fixtures/border__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn border_border_color_should_fallback_border_color_to_the_current_color() {
    run_fixture("border__border-border-color-should-fallback-border-color-to-the-current-color__1.json");
}

#[test]
fn border_border_color_should_render_black_border_by_default() {
    run_fixture("border__border-border-color-should-render-black-border-by-default__1.json");
}

#[test]
fn border_border_color_should_support_overriding_border_color() {
    run_fixture("border__border-border-color-should-support-overriding-border-color__1.json");
}

#[test]
fn border_border_color_should_support_specifying_border_color() {
    run_fixture("border__border-border-color-should-support-specifying-border-color__1.json");
}

#[test]
fn border_border_radius_should_not_exceed_the_length_of_the_short_side() {
    run_fixture("border__border-border-radius-should-not-exceed-the-length-of-the-short-side__1.json");
}

#[test]
fn border_border_radius_should_support_percentage_border_radius() {
    run_fixture("border__border-border-radius-should-support-percentage-border-radius__1.json");
}

#[test]
fn border_border_radius_should_support_radius_for_a_certain_corner() {
    run_fixture("border__border-border-radius-should-support-radius-for-a-certain-corner__1.json");
}

#[test]
fn border_border_radius_should_support_slash_and_2_value_corner() {
    run_fixture("border__border-border-radius-should-support-slash-and-2-value-corner__1.json");
}

#[test]
fn border_border_radius_should_support_the_shorthand() {
    run_fixture("border__border-border-radius-should-support-the-shorthand__1.json");
}

#[test]
fn border_border_radius_should_support_vw_vh_em_and_rem_units() {
    run_fixture("border__border-border-radius-should-support-vw-vh-em-and-rem-units__1.json");
}

#[test]
fn border_border_should_support_the_shorthand() {
    run_fixture("border__border-border-should-support-the-shorthand__1.json");
}

#[test]
fn border_border_style_should_support_dashed_border() {
    run_fixture("border__border-border-style-should-support-dashed-border__1.json");
}

#[test]
fn border_border_width_should_render_border_inside_the_shape() {
    run_fixture("border__border-border-width-should-render-border-inside-the-shape__1.json");
}

#[test]
fn border_directional_should_support_advanced_border_with_radius() {
    run_fixture("border__border-directional-should-support-advanced-border-with-radius__1.json");
}

#[test]
fn border_directional_should_support_directional_border() {
    run_fixture("border__border-directional-should-support-directional-border__1.json");
}

#[test]
fn border_directional_should_support_non_complete_border() {
    run_fixture("border__border-directional-should-support-non-complete-border__1.json");
}
