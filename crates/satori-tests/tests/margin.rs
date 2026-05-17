//! Snapshot tests sourced from `crates/satori-tests/fixtures/margin__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn margin_should_render_asymmetric_margins() {
    run_fixture("margin__margin-should-render-asymmetric-margins__1.json");
}

#[test]
fn margin_should_render_element_with_individual_margin_properties() {
    run_fixture("margin__margin-should-render-element-with-individual-margin-properties__1.json");
}

#[test]
fn margin_should_render_element_with_margin_shorthand_1_value() {
    run_fixture("margin__margin-should-render-element-with-margin-shorthand-1-value__1.json");
}

#[test]
fn margin_should_render_element_with_margin_shorthand_2_values() {
    run_fixture("margin__margin-should-render-element-with-margin-shorthand-2-values__1.json");
}

#[test]
fn margin_should_render_element_with_margin_shorthand_3_values() {
    run_fixture("margin__margin-should-render-element-with-margin-shorthand-3-values__1.json");
}

#[test]
fn margin_should_render_element_with_margin_shorthand_4_values() {
    run_fixture("margin__margin-should-render-element-with-margin-shorthand-4-values__1.json");
}

#[test]
fn margin_should_render_element_with_negative_margin_left() {
    run_fixture("margin__margin-should-render-element-with-negative-margin-left__1.json");
}

#[test]
fn margin_should_render_element_with_negative_margin() {
    run_fixture("margin__margin-should-render-element-with-negative-margin__1.json");
}

#[test]
fn margin_should_render_large_margin_values() {
    run_fixture("margin__margin-should-render-large-margin-values__1.json");
}

#[test]
fn margin_should_render_margin_auto_horizontally() {
    run_fixture("margin__margin-should-render-margin-auto-horizontally__1.json");
}

#[test]
fn margin_should_render_margin_collapsing_with_siblings() {
    run_fixture("margin__margin-should-render-margin-collapsing-with-siblings__1.json");
}

#[test]
fn margin_should_render_margin_left_auto() {
    run_fixture("margin__margin-should-render-margin-left-auto__1.json");
}

#[test]
fn margin_should_render_margin_right_auto() {
    run_fixture("margin__margin-should-render-margin-right-auto__1.json");
}

#[test]
fn margin_should_render_margin_with_different_units() {
    run_fixture("margin__margin-should-render-margin-with-different-units__1.json");
}

#[test]
fn margin_should_render_margin_with_flexbox_column_container() {
    run_fixture("margin__margin-should-render-margin-with-flexbox-column-container__1.json");
}

#[test]
fn margin_should_render_margin_with_flexbox_row_container() {
    run_fixture("margin__margin-should-render-margin-with-flexbox-row-container__1.json");
}

#[test]
fn margin_should_render_margin_with_nested_elements() {
    run_fixture("margin__margin-should-render-margin-with-nested-elements__1.json");
}

#[test]
fn margin_should_render_margin_with_positioned_elements() {
    run_fixture("margin__margin-should-render-margin-with-positioned-elements__1.json");
}

#[test]
fn margin_should_render_margin_with_text_content() {
    run_fixture("margin__margin-should-render-margin-with-text-content__1.json");
}

#[test]
fn margin_should_render_zero_margin() {
    run_fixture("margin__margin-should-render-zero-margin__1.json");
}
