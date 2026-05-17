//! Snapshot tests sourced from `fixtures/padding__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn padding_should_render_asymmetric_padding() {
    run_fixture("padding__padding-should-render-asymmetric-padding__1.json");
}

#[test]
fn padding_should_render_different_padding_on_each_side() {
    run_fixture("padding__padding-should-render-different-padding-on-each-side__1.json");
}

#[test]
fn padding_should_render_element_with_individual_padding_properties() {
    run_fixture("padding__padding-should-render-element-with-individual-padding-properties__1.json");
}

#[test]
fn padding_should_render_element_with_padding_shorthand_1_value() {
    run_fixture("padding__padding-should-render-element-with-padding-shorthand-1-value__1.json");
}

#[test]
fn padding_should_render_element_with_padding_shorthand_2_values() {
    run_fixture("padding__padding-should-render-element-with-padding-shorthand-2-values__1.json");
}

#[test]
fn padding_should_render_element_with_padding_shorthand_3_values() {
    run_fixture("padding__padding-should-render-element-with-padding-shorthand-3-values__1.json");
}

#[test]
fn padding_should_render_element_with_padding_shorthand_4_values() {
    run_fixture("padding__padding-should-render-element-with-padding-shorthand-4-values__1.json");
}

#[test]
fn padding_should_render_large_padding_values() {
    run_fixture("padding__padding-should-render-large-padding-values__1.json");
}

#[test]
fn padding_should_render_padding_with_border_radius() {
    run_fixture("padding__padding-should-render-padding-with-border-radius__1.json");
}

#[test]
fn padding_should_render_padding_with_border() {
    run_fixture("padding__padding-should-render-padding-with-border__1.json");
}

#[test]
fn padding_should_render_padding_with_box_shadow() {
    run_fixture("padding__padding-should-render-padding-with-box-shadow__1.json");
}

#[test]
fn padding_should_render_padding_with_flexbox_column_container() {
    run_fixture("padding__padding-should-render-padding-with-flexbox-column-container__1.json");
}

#[test]
fn padding_should_render_padding_with_flexbox_row_container() {
    run_fixture("padding__padding-should-render-padding-with-flexbox-row-container__1.json");
}

#[test]
fn padding_should_render_padding_with_gradient_background() {
    run_fixture("padding__padding-should-render-padding-with-gradient-background__1.json");
}

#[test]
fn padding_should_render_padding_with_multiple_text_lines() {
    run_fixture("padding__padding-should-render-padding-with-multiple-text-lines__1.json");
}

#[test]
fn padding_should_render_padding_with_nested_elements() {
    run_fixture("padding__padding-should-render-padding-with-nested-elements__1.json");
}

#[test]
fn padding_should_render_padding_with_opacity() {
    run_fixture("padding__padding-should-render-padding-with-opacity__1.json");
}

#[test]
fn padding_should_render_padding_with_text_content() {
    run_fixture("padding__padding-should-render-padding-with-text-content__1.json");
}

#[test]
fn padding_should_render_padding_with_transform() {
    run_fixture("padding__padding-should-render-padding-with-transform__1.json");
}

#[test]
fn padding_should_render_zero_padding() {
    run_fixture("padding__padding-should-render-zero-padding__1.json");
}
