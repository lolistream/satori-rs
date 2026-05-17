//! Snapshot tests sourced from `crates/satori-tests/fixtures/coverage-gradients__*.json`.

use satori_tests::harness::{run_fixture, run_assertions_fixture};

#[test]
fn gradient_coverage_conic_should_fall_back_position_resolution_when_the_keyword_is_unparseable() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-fall-back-position-resolution-when-the-keyword-is-unparseable__1.json");
}

#[test]
fn gradient_coverage_conic_should_fall_through_interpolate_color_when_a_stop_has_an_unparseable_color_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-conic-should-fall-through-interpolate-color-when-a-stop-has-an-unparseable-color__assertions.json");
}

#[test]
fn gradient_coverage_conic_should_render_repeating_conic_gradient_with_a_single_stop_interpolate_color_single_stop_path() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-render-repeating-conic-gradient-with-a-single-stop-interpolate-color-single-stop-path__1.json");
}

#[test]
fn gradient_coverage_conic_should_resolve_resolve_position_part_center_via_two_part_position() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-resolve-resolve-position-part-center-via-two-part-position__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_at_single_horizontal_keyword_left() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-at-single-horizontal-keyword-left__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_at_single_horizontal_keyword_right() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-at-single-horizontal-keyword-right__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_at_single_length_keyword() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-at-single-length-keyword__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_at_single_vertical_keyword_bottom() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-at-single-vertical-keyword-bottom__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_at_single_vertical_keyword_top() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-at-single-vertical-keyword-top__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_explicit_non_zero_first_offset_and_hint() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-explicit-non-zero-first-offset-and-hint__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_hint_at_segment_end_h_1() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-hint-at-segment-end-h-1__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_hint_at_segment_start_h_0() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-hint-at-segment-start-h-0__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_hsl_color_including_alpha() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-hsl-color-including-alpha__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_hsl_color_stops_covering_all_hue_branches() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-hsl-color-stops-covering-all-hue-branches__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_no_repeat_background_repeat_repeat_x_false_repeat_y_false() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-no-repeat-background-repeat-repeat-x-false-repeat-y-false__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_percentage_hint() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-percentage-hint__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_percentage_keyword_swap_50_left() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-percentage-keyword-swap-50-left__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_percentage_two_part_position() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-percentage-two-part-position__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_two_keyword_swap_bottom_right() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-two-keyword-swap-bottom-right__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_conic_gradient_with_two_keyword_swap_top_left() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-conic-gradient-with-two-keyword-swap-top-left__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_repeating_conic_gradient_where_last_offset_evaluates_to_0deg_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-conic-should-support-repeating-conic-gradient-where-last-offset-evaluates-to-0deg__assertions.json");
}

#[test]
fn gradient_coverage_conic_should_support_repeating_conic_gradient_with_no_explicit_last_offset() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-repeating-conic-gradient-with-no-explicit-last-offset__1.json");
}

#[test]
fn gradient_coverage_conic_should_support_repeating_conic_gradient_with_percentage_last_offset() {
    run_fixture("coverage-gradients__gradient-coverage-conic-should-support-repeating-conic-gradient-with-percentage-last-offset__1.json");
}

#[test]
fn gradient_coverage_linear_should_support_linear_gradient_with_non_axis_directional_value() {
    run_fixture("coverage-gradients__gradient-coverage-linear-should-support-linear-gradient-with-non-axis-directional-value__1.json");
}

#[test]
fn gradient_coverage_linear_should_support_repeating_linear_gradient_with_first_stop_in_calc_percentage_branch() {
    run_fixture("coverage-gradients__gradient-coverage-linear-should-support-repeating-linear-gradient-with-first-stop-in-calc-percentage-branch__1.json");
}

#[test]
fn gradient_coverage_linear_should_support_repeating_linear_gradient_with_first_stop_in_px_calc_percentage_non_branch() {
    run_fixture("coverage-gradients__gradient-coverage-linear-should-support-repeating-linear-gradient-with-first-stop-in-px-calc-percentage-non-branch__1.json");
}

#[test]
fn gradient_coverage_linear_should_support_repeating_linear_gradient_with_last_stop_missing_offset() {
    run_fixture("coverage-gradients__gradient-coverage-linear-should-support-repeating-linear-gradient-with-last-stop-missing-offset__1.json");
}

#[test]
fn gradient_coverage_linear_should_support_repeating_linear_gradient_with_no_repeat_background_repeat() {
    run_fixture("coverage-gradients__gradient-coverage-linear-should-support-repeating-linear-gradient-with-no-repeat-background-repeat__1.json");
}

#[test]
fn gradient_coverage_radial_should_fall_back_to_x_delta_y_delta_halves_for_unparseable_radial_positions() {
    run_fixture("coverage-gradients__gradient-coverage-radial-should-fall-back-to-x-delta-y-delta-halves-for-unparseable-radial-positions__1.json");
}

#[test]
fn gradient_coverage_radial_should_handle_ellipse_closest_corner_with_center_at_corner_fx_0() {
    run_fixture("coverage-gradients__gradient-coverage-radial-should-handle-ellipse-closest-corner-with-center-at-corner-fx-0__1.json");
}

#[test]
fn gradient_coverage_radial_should_support_circle_farthest_side_radial_gradient() {
    run_fixture("coverage-gradients__gradient-coverage-radial-should-support-circle-farthest-side-radial-gradient__1.json");
}

#[test]
fn gradient_coverage_radial_should_support_radial_gradient_with_at_right_keyword() {
    run_fixture("coverage-gradients__gradient-coverage-radial-should-support-radial-gradient-with-at-right-keyword__1.json");
}

#[test]
fn gradient_coverage_radial_should_support_radial_gradient_with_no_repeat_background_repeat() {
    run_fixture("coverage-gradients__gradient-coverage-radial-should-support-radial-gradient-with-no-repeat-background-repeat__1.json");
}

#[test]
fn gradient_coverage_utils_normalize_stops_should_convert_a_fully_transparent_mask_stop_into_opaque_rgba_0_0_0_1_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-utils-normalize-stops-should-convert-a-fully-transparent-mask-stop-into-opaque-rgba-0-0-0-1__assertions.json");
}

#[test]
fn gradient_coverage_utils_normalize_stops_should_distribute_multiple_gaps_of_undefined_offsets() {
    run_fixture("coverage-gradients__gradient-coverage-utils-normalize-stops-should-distribute-multiple-gaps-of-undefined-offsets__1.json");
}

#[test]
fn gradient_coverage_utils_normalize_stops_should_keep_mask_stops_with_unparseable_colors_verbatim_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-utils-normalize-stops-should-keep-mask-stops-with-unparseable-colors-verbatim__assertions.json");
}

#[test]
fn gradient_coverage_utils_normalize_stops_unit_should_fall_back_to_a_single_transparent_stop_when_color_stops_is_empty_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-utils-normalize-stops-unit-should-fall-back-to-a-single-transparent-stop-when-color-stops-is-empty__assertions.json");
}

#[test]
fn gradient_coverage_utils_normalize_stops_unit_should_rewrite_the_last_repeating_stop_when_its_offset_1_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-utils-normalize-stops-unit-should-rewrite-the-last-repeating-stop-when-its-offset-1__assertions.json");
}

#[test]
fn gradient_coverage_webkit_should_pass_through_webkit_radial_gradient_that_starts_with_color_stop_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-webkit-should-pass-through-webkit-radial-gradient-that-starts-with-color-stop__assertions.json");
}

#[test]
fn gradient_coverage_webkit_should_support_webkit_linear_gradient_with_grad_units_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-webkit-should-support-webkit-linear-gradient-with-grad-units__assertions.json");
}

#[test]
fn gradient_coverage_webkit_should_support_webkit_radial_gradient_with_position_only_no_shape_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-webkit-should-support-webkit-radial-gradient-with-position-only-no-shape__assertions.json");
}

#[test]
fn gradient_coverage_webkit_should_support_webkit_radial_gradient_with_px_position_only_assertions() {
    run_assertions_fixture("coverage-gradients__gradient-coverage-webkit-should-support-webkit-radial-gradient-with-px-position-only__assertions.json");
}
