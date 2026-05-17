//! Snapshot tests sourced from `fixtures/gradient__*.json`.

mod common;
use common::harness::{run_fixture, run_assertions_fixture};

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_at_top_left_corner() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-at-top-left-corner__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_checkerboard_pattern() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-checkerboard-pattern__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_on_non_square_element() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-on-non-square-element__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_smooth_rainbow() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-smooth-rainbow__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_three_way_smooth_blend() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-three-way-smooth-blend__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_via_background_shorthand() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-via-background-shorthand__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_at_position() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-at-position__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_deg_stops() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-deg-stops__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_from_angle_and_at_position() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-from-angle-and-at-position__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_from_angle() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-from-angle__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_hard_stops_pie_chart() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-hard-stops-pie-chart__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_rgba_transparency() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-rgba-transparency__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_single_color() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-single-color__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_transition_hints() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-transition-hints__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_turn_units() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-turn-units__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_conic_gradient_with_uneven_stops() {
    run_fixture("gradient__gradient-conic-gradient-should-support-conic-gradient-with-uneven-stops__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_repeating_conic_gradient_with_hard_stops() {
    run_fixture("gradient__gradient-conic-gradient-should-support-repeating-conic-gradient-with-hard-stops__1.json");
}

#[test]
fn gradient_conic_gradient_should_support_repeating_conic_gradient() {
    run_fixture("gradient__gradient-conic-gradient-should-support-repeating-conic-gradient__1.json");
}

#[test]
fn gradient_linear_gradient_should_support_linear_gradient_with_omitted_orientation() {
    run_fixture("gradient__gradient-linear-gradient-should-support-linear-gradient-with-omitted-orientation__1.json");
}

#[test]
fn gradient_linear_gradient_should_support_linear_gradient_with_transparency() {
    run_fixture("gradient__gradient-linear-gradient-should-support-linear-gradient-with-transparency__1.json");
}

#[test]
fn gradient_linear_gradient_should_support_linear_gradient() {
    run_fixture("gradient__gradient-linear-gradient-should-support-linear-gradient__1.json");
}

#[test]
fn gradient_linear_gradient_should_support_multiple_direction_keywords() {
    run_fixture("gradient__gradient-linear-gradient-should-support-multiple-direction-keywords__1.json");
}

#[test]
fn gradient_linear_gradient_should_support_other_degree_unit() {
    run_fixture("gradient__gradient-linear-gradient-should-support-other-degree-unit__1.json");
}

#[test]
fn gradient_linear_gradient_should_support_other_degree_unit_call2() {
    run_fixture("gradient__gradient-linear-gradient-should-support-other-degree-unit__2.json");
}

#[test]
fn gradient_linear_gradient_should_support_other_degree_unit_call3() {
    run_fixture("gradient__gradient-linear-gradient-should-support-other-degree-unit__3.json");
}

#[test]
fn gradient_linear_gradient_should_support_repeating_linear_gradient() {
    run_fixture("gradient__gradient-linear-gradient-should-support-repeating-linear-gradient__1.json");
}

#[test]
fn gradient_linear_gradient_should_support_using_background_instead_of_background_image() {
    run_fixture("gradient__gradient-linear-gradient-should-support-using-background-instead-of-background-image__1.json");
}

#[test]
fn gradient_radial_gradient_should_make_sense_if_x_of_y_is_zero() {
    run_fixture("gradient__gradient-radial-gradient-should-make-sense-if-x-of-y-is-zero__1.json");
}

#[test]
fn gradient_radial_gradient_should_support_default_value() {
    run_fixture("gradient__gradient-radial-gradient-should-support-default-value__1.json");
}

#[test]
fn gradient_radial_gradient_should_support_explicitly_setting_rg_size() {
    run_fixture("gradient__gradient-radial-gradient-should-support-explicitly-setting-rg-size__1.json");
}

#[test]
fn gradient_radial_gradient_should_support_explicitly_setting_rg_size_call2() {
    run_fixture("gradient__gradient-radial-gradient-should-support-explicitly-setting-rg-size__2.json");
}

#[test]
fn gradient_radial_gradient_should_support_radial_gradient_with_unspecified_ending_shape() {
    run_fixture("gradient__gradient-radial-gradient-should-support-radial-gradient-with-unspecified-ending-shape__1.json");
}

#[test]
fn gradient_radial_gradient_should_support_radial_gradient() {
    run_fixture("gradient__gradient-radial-gradient-should-support-radial-gradient__1.json");
}

#[test]
fn gradient_radial_gradient_should_support_releative_unit() {
    run_fixture("gradient__gradient-radial-gradient-should-support-releative-unit__1.json");
}

#[test]
fn gradient_radial_gradient_should_support_releative_unit_call2() {
    run_fixture("gradient__gradient-radial-gradient-should-support-releative-unit__2.json");
}

#[test]
fn gradient_radial_gradient_should_support_releative_unit_call3() {
    run_fixture("gradient__gradient-radial-gradient-should-support-releative-unit__3.json");
}

#[test]
fn gradient_radial_gradient_should_support_releative_unit_call4() {
    run_fixture("gradient__gradient-radial-gradient-should-support-releative-unit__4.json");
}

#[test]
fn gradient_radial_gradient_should_support_rg_size_with_rg_extent_keyword() {
    run_fixture("gradient__gradient-radial-gradient-should-support-rg-size-with-rg-extent-keyword__1.json");
}

#[test]
fn gradient_radial_gradient_should_support_rg_size_with_rg_extent_keyword_call2() {
    run_fixture("gradient__gradient-radial-gradient-should-support-rg-size-with-rg-extent-keyword__2.json");
}

#[test]
fn gradient_radial_gradient_should_support_rg_size_with_rg_extent_keyword_call3() {
    run_fixture("gradient__gradient-radial-gradient-should-support-rg-size-with-rg-extent-keyword__3.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_compute_correct_cycle() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-compute-correct-cycle__1.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_compute_correct_cycle_call2() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-compute-correct-cycle__2.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_support_background_size_and_background_repeat() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-support-background-size-and-background-repeat__1.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_support_degree() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-support-degree__1.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_support_degree_call2() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-support-degree__2.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_support_degree_call3() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-support-degree__3.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_support_degree_call4() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-support-degree__4.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_support_multiple_repeating_linear_gradient() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-support-multiple-repeating-linear-gradient__1.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_support_repeating_linear_gradient() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-support-repeating-linear-gradient__1.json");
}

#[test]
fn gradient_repeating_linear_gradient_should_support_repeating_linear_gradient_call2() {
    run_fixture("gradient__gradient-repeating-linear-gradient-should-support-repeating-linear-gradient__2.json");
}

#[test]
fn gradient_repeating_radial_gradient_should_support_repeating_radial_gradient() {
    run_fixture("gradient__gradient-repeating-radial-gradient-should-support-repeating-radial-gradient__1.json");
}

#[test]
fn gradient_repeating_radial_gradient_should_support_repeating_radial_gradient_call2() {
    run_fixture("gradient__gradient-repeating-radial-gradient-should-support-repeating-radial-gradient__2.json");
}

#[test]
fn gradient_repeating_radial_gradient_should_support_repeating_radial_gradient_call3() {
    run_fixture("gradient__gradient-repeating-radial-gradient-should-support-repeating-radial-gradient__3.json");
}

#[test]
fn gradient_repeating_radial_gradient_should_support_repeating_radial_gradient_call4() {
    run_fixture("gradient__gradient-repeating-radial-gradient-should-support-repeating-radial-gradient__4.json");
}

#[test]
fn gradient_repeating_radial_gradient_should_support_repeating_radial_gradient_call5() {
    run_fixture("gradient__gradient-repeating-radial-gradient-should-support-repeating-radial-gradient__5.json");
}

#[test]
fn gradient_repeating_radial_gradient_should_support_repeating_radial_gradient_call6() {
    run_fixture("gradient__gradient-repeating-radial-gradient-should-support-repeating-radial-gradient__6.json");
}

#[test]
fn gradient_should_be_able_to_render_grid_backgrounds() {
    run_fixture("gradient__gradient-should-be-able-to-render-grid-backgrounds__1.json");
}

#[test]
fn gradient_should_calculate_the_gradient_angle_and_length_correctly_with_offset() {
    run_fixture("gradient__gradient-should-calculate-the-gradient-angle-and-length-correctly-with-offset__1.json");
}

#[test]
fn gradient_should_calculate_the_gradient_angle_and_length_correctly() {
    run_fixture("gradient__gradient-should-calculate-the-gradient-angle-and-length-correctly__1.json");
}

#[test]
fn gradient_should_render_gradient_patterns_in_the_correct_object_space() {
    run_fixture("gradient__gradient-should-render-gradient-patterns-in-the-correct-object-space__1.json");
}

#[test]
fn gradient_should_resolve_gradient_layers_in_the_correct_order() {
    run_fixture("gradient__gradient-should-resolve-gradient-layers-in-the-correct-order__1.json");
}

#[test]
fn gradient_should_support_advanced_usage() {
    run_fixture("gradient__gradient-should-support-advanced-usage__1.json");
}

#[test]
fn gradient_should_support_gradient_with_color_background() {
    run_fixture("gradient__gradient-should-support-gradient-with-color-background__1.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_linear_gradient_via_background_shorthand_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-linear-gradient-via-background-shorthand__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_linear_gradient_with_45deg_angle_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-linear-gradient-with-45deg-angle__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_linear_gradient_with_angle_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-linear-gradient-with-angle__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_linear_gradient_with_bottom_direction_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-linear-gradient-with-bottom-direction__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_linear_gradient_with_combo_direction_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-linear-gradient-with-combo-direction__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_linear_gradient_with_direction_keyword_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-linear-gradient-with-direction-keyword__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_linear_gradient_with_other_angle_units_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-linear-gradient-with-other-angle-units__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_linear_gradient_with_rgba_color_stops_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-linear-gradient-with-rgba-color-stops__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_linear_gradient_without_direction_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-linear-gradient-without-direction__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_radial_gradient_with_contain_keyword_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-radial-gradient-with-contain-keyword__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_radial_gradient_with_cover_keyword_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-radial-gradient-with-cover-keyword__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_radial_gradient_with_length_size_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-radial-gradient-with-length-size__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_radial_gradient_with_position_and_shape_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-radial-gradient-with-position-and-shape__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_radial_gradient_with_px_position_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-radial-gradient-with-px-position__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_radial_gradient_with_shape_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-radial-gradient-with-shape__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_repeating_linear_gradient_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-repeating-linear-gradient__assertions.json");
}

#[test]
fn gradient_webkit_gradient_should_support_webkit_repeating_radial_gradient_assertions() {
    run_assertions_fixture("gradient__gradient-webkit-gradient-should-support-webkit-repeating-radial-gradient__assertions.json");
}
