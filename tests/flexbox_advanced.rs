//! Snapshot tests sourced from `fixtures/flexbox-advanced__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn flexbox_advanced_align_content_should_render_align_content_center() {
    run_fixture("flexbox-advanced__flexbox-advanced-align-content-should-render-align-content-center__1.json");
}

#[test]
fn flexbox_advanced_align_content_should_render_align_content_flex_start() {
    run_fixture("flexbox-advanced__flexbox-advanced-align-content-should-render-align-content-flex-start__1.json");
}

#[test]
fn flexbox_advanced_align_content_should_render_align_content_space_between() {
    run_fixture("flexbox-advanced__flexbox-advanced-align-content-should-render-align-content-space-between__1.json");
}

#[test]
fn flexbox_advanced_align_self_should_render_align_self_center() {
    run_fixture("flexbox-advanced__flexbox-advanced-align-self-should-render-align-self-center__1.json");
}

#[test]
fn flexbox_advanced_align_self_should_render_align_self_flex_end() {
    run_fixture("flexbox-advanced__flexbox-advanced-align-self-should-render-align-self-flex-end__1.json");
}

#[test]
fn flexbox_advanced_align_self_should_render_align_self_flex_start() {
    run_fixture("flexbox-advanced__flexbox-advanced-align-self-should-render-align-self-flex-start__1.json");
}

#[test]
fn flexbox_advanced_align_self_should_render_align_self_stretch() {
    run_fixture("flexbox-advanced__flexbox-advanced-align-self-should-render-align-self-stretch__1.json");
}

#[test]
fn flexbox_advanced_complex_layouts_should_combine_flex_grow_and_flex_shrink() {
    run_fixture("flexbox-advanced__flexbox-advanced-complex-layouts-should-combine-flex-grow-and-flex-shrink__1.json");
}

#[test]
fn flexbox_advanced_complex_layouts_should_render_column_gap_with_percentage_values() {
    run_fixture("flexbox-advanced__flexbox-advanced-complex-layouts-should-render-column-gap-with-percentage-values__1.json");
}

#[test]
fn flexbox_advanced_complex_layouts_should_render_complex_flex_layout_with_multiple_properties() {
    run_fixture("flexbox-advanced__flexbox-advanced-complex-layouts-should-render-complex-flex-layout-with-multiple-properties__1.json");
}

#[test]
fn flexbox_advanced_complex_layouts_should_render_flex_with_gap_and_wrapping() {
    run_fixture("flexbox-advanced__flexbox-advanced-complex-layouts-should-render-flex-with-gap-and-wrapping__1.json");
}

#[test]
fn flexbox_advanced_complex_layouts_should_render_gap_with_percentage_values() {
    run_fixture("flexbox-advanced__flexbox-advanced-complex-layouts-should-render-gap-with-percentage-values__1.json");
}

#[test]
fn flexbox_advanced_complex_layouts_should_render_nested_flex_containers() {
    run_fixture("flexbox-advanced__flexbox-advanced-complex-layouts-should-render-nested-flex-containers__1.json");
}

#[test]
fn flexbox_advanced_complex_layouts_should_render_row_gap_with_percentage_values() {
    run_fixture("flexbox-advanced__flexbox-advanced-complex-layouts-should-render-row-gap-with-percentage-values__1.json");
}

#[test]
fn flexbox_advanced_flex_basis_should_render_elements_with_flex_basis() {
    run_fixture("flexbox-advanced__flexbox-advanced-flex-basis-should-render-elements-with-flex-basis__1.json");
}

#[test]
fn flexbox_advanced_flex_basis_should_render_flex_basis_with_flex_grow() {
    run_fixture("flexbox-advanced__flexbox-advanced-flex-basis-should-render-flex-basis-with-flex-grow__1.json");
}

#[test]
fn flexbox_advanced_flex_grow_should_render_elements_with_flex_grow() {
    run_fixture("flexbox-advanced__flexbox-advanced-flex-grow-should-render-elements-with-flex-grow__1.json");
}

#[test]
fn flexbox_advanced_flex_grow_should_render_with_different_flex_grow_ratios() {
    run_fixture("flexbox-advanced__flexbox-advanced-flex-grow-should-render-with-different-flex-grow-ratios__1.json");
}

#[test]
fn flexbox_advanced_flex_shorthand_should_render_with_different_flex_values() {
    run_fixture("flexbox-advanced__flexbox-advanced-flex-shorthand-should-render-with-different-flex-values__1.json");
}

#[test]
fn flexbox_advanced_flex_shorthand_should_render_with_flex_1() {
    run_fixture("flexbox-advanced__flexbox-advanced-flex-shorthand-should-render-with-flex-1__1.json");
}

#[test]
fn flexbox_advanced_flex_shrink_should_render_elements_with_flex_shrink() {
    run_fixture("flexbox-advanced__flexbox-advanced-flex-shrink-should-render-elements-with-flex-shrink__1.json");
}

#[test]
fn flexbox_advanced_flex_shrink_should_render_with_different_flex_shrink_values() {
    run_fixture("flexbox-advanced__flexbox-advanced-flex-shrink-should-render-with-different-flex-shrink-values__1.json");
}
