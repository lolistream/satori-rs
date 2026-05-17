//! Snapshot tests sourced from `crates/satori-tests/fixtures/color-models__*.json`.

use satori_tests::harness::{run_fixture, run_assertions_fixture};

#[test]
fn color_models_background_color_and_color_should_support_currentcolor_when_background() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-currentcolor-when-background__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_currentcolor_when_border() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-currentcolor-when-border__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_currentcolor_when_inherit() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-currentcolor-when-inherit__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_hexadecimal_with_transparency() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-hexadecimal-with-transparency__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_hexadecimal() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-hexadecimal__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_hsl() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-hsl__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_hsla() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-hsla__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_inherit_color() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-inherit-color__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_predefined_color_names() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-predefined-color-names__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_rgb() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-rgb__1.json");
}

#[test]
fn color_models_background_color_and_color_should_support_rgba() {
    run_fixture("color-models__color-models-background-color-and-color-should-support-rgba__1.json");
}

#[test]
fn color_models_should_support_css4_syntax_color_in_hsl_if_inherited_assertions() {
    run_assertions_fixture("color-models__color-models-should-support-css4-syntax-color-in-hsl-if-inherited__assertions.json");
}

#[test]
fn color_models_should_support_css4_syntax_color_in_hsl_assertions() {
    run_assertions_fixture("color-models__color-models-should-support-css4-syntax-color-in-hsl__assertions.json");
}
