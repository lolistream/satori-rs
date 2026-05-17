//! Snapshot tests sourced from `crates/satori-tests/fixtures/css-variables__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn css_variables_should_handle_undefined_variables_with_fallback_chain() {
    run_fixture("css-variables__css-variables-should-handle-undefined-variables-with-fallback-chain__1.json");
}

#[test]
fn css_variables_should_support_basic_css_variable_declaration_and_usage() {
    run_fixture("css-variables__css-variables-should-support-basic-css-variable-declaration-and-usage__1.json");
}

#[test]
fn css_variables_should_support_css_variable_fallback_values() {
    run_fixture("css-variables__css-variables-should-support-css-variable-fallback-values__1.json");
}

#[test]
fn css_variables_should_support_css_variable_for_inherited_text_color() {
    run_fixture("css-variables__css-variables-should-support-css-variable-for-inherited-text-color__1.json");
}

#[test]
fn css_variables_should_support_css_variable_for_text_color() {
    run_fixture("css-variables__css-variables-should-support-css-variable-for-text-color__1.json");
}

#[test]
fn css_variables_should_support_css_variable_inheritance() {
    run_fixture("css-variables__css-variables-should-support-css-variable-inheritance__1.json");
}

#[test]
fn css_variables_should_support_css_variable_override_in_children() {
    run_fixture("css-variables__css-variables-should-support-css-variable-override-in-children__1.json");
}

#[test]
fn css_variables_should_support_css_variables_with_border_properties() {
    run_fixture("css-variables__css-variables-should-support-css-variables-with-border-properties__1.json");
}

#[test]
fn css_variables_should_support_css_variables_with_dimensions() {
    run_fixture("css-variables__css-variables-should-support-css-variables-with-dimensions__1.json");
}

#[test]
fn css_variables_should_support_css_variables_with_percentage_values() {
    run_fixture("css-variables__css-variables-should-support-css-variables-with-percentage-values__1.json");
}

#[test]
fn css_variables_should_support_multiple_css_variables_in_nested_inheritance() {
    run_fixture("css-variables__css-variables-should-support-multiple-css-variables-in-nested-inheritance__1.json");
}

#[test]
fn css_variables_should_support_nested_css_variables() {
    run_fixture("css-variables__css-variables-should-support-nested-css-variables__1.json");
}
