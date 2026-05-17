//! Snapshot tests sourced from `crates/satori-tests/fixtures/opacity__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn opacity_should_apply_near_full_opacity() {
    run_fixture("opacity__opacity-should-apply-near-full-opacity__1.json");
}

#[test]
fn opacity_should_apply_opacity_to_elements_with_border_radius() {
    run_fixture("opacity__opacity-should-apply-opacity-to-elements-with-border-radius__1.json");
}

#[test]
fn opacity_should_apply_opacity_to_elements_with_border() {
    run_fixture("opacity__opacity-should-apply-opacity-to-elements-with-border__1.json");
}

#[test]
fn opacity_should_apply_opacity_to_elements_with_box_shadow() {
    run_fixture("opacity__opacity-should-apply-opacity-to-elements-with-box-shadow__1.json");
}

#[test]
fn opacity_should_apply_opacity_to_flex_container() {
    run_fixture("opacity__opacity-should-apply-opacity-to-flex-container__1.json");
}

#[test]
fn opacity_should_apply_opacity_to_multiple_siblings() {
    run_fixture("opacity__opacity-should-apply-opacity-to-multiple-siblings__1.json");
}

#[test]
fn opacity_should_apply_opacity_to_overlapping_elements() {
    run_fixture("opacity__opacity-should-apply-opacity-to-overlapping-elements__1.json");
}

#[test]
fn opacity_should_apply_opacity_to_positioned_elements() {
    run_fixture("opacity__opacity-should-apply-opacity-to-positioned-elements__1.json");
}

#[test]
fn opacity_should_apply_opacity_to_text_elements() {
    run_fixture("opacity__opacity-should-apply-opacity-to-text-elements__1.json");
}

#[test]
fn opacity_should_apply_opacity_to_text_with_text_shadow() {
    run_fixture("opacity__opacity-should-apply-opacity-to-text-with-text-shadow__1.json");
}

#[test]
fn opacity_should_apply_opacity_with_transform() {
    run_fixture("opacity__opacity-should-apply-opacity-with-transform__1.json");
}

#[test]
fn opacity_should_apply_very_low_opacity() {
    run_fixture("opacity__opacity-should-apply-very-low-opacity__1.json");
}

#[test]
fn opacity_should_cascade_opacity_through_nested_elements() {
    run_fixture("opacity__opacity-should-cascade-opacity-through-nested-elements__1.json");
}

#[test]
fn opacity_should_combine_multiple_opacity_values_in_nested_elements() {
    run_fixture("opacity__opacity-should-combine-multiple-opacity-values-in-nested-elements__1.json");
}

#[test]
fn opacity_should_combine_opacity_with_background_clip_text() {
    run_fixture("opacity__opacity-should-combine-opacity-with-background-clip-text__1.json");
}

#[test]
fn opacity_should_combine_opacity_with_linear_gradients() {
    run_fixture("opacity__opacity-should-combine-opacity-with-linear-gradients__1.json");
}

#[test]
fn opacity_should_combine_opacity_with_radial_gradients() {
    run_fixture("opacity__opacity-should-combine-opacity-with-radial-gradients__1.json");
}

#[test]
fn opacity_should_handle_opacity_0_with_nested_content() {
    run_fixture("opacity__opacity-should-handle-opacity-0-with-nested-content__1.json");
}

#[test]
fn opacity_should_render_element_with_opacity_0_5() {
    run_fixture("opacity__opacity-should-render-element-with-opacity-0-5__1.json");
}

#[test]
fn opacity_should_render_element_with_opacity_0() {
    run_fixture("opacity__opacity-should-render-element-with-opacity-0__1.json");
}

#[test]
fn opacity_should_render_element_with_opacity_1() {
    run_fixture("opacity__opacity-should-render-element-with-opacity-1__1.json");
}
