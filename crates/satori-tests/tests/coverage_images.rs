//! Snapshot tests sourced from `crates/satori-tests/fixtures/coverage-images__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn coverage_filler_background_image_should_accept_a_css_named_color_as_background_image_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-background-image-should-accept-a-css-named-color-as-background-image__assertions.json");
}

#[test]
fn coverage_filler_background_image_should_fall_back_to_container_size_when_image_has_no_intrinsic_dimensions_with_keyword_size_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-background-image-should-fall-back-to-container-size-when-image-has-no-intrinsic-dimensions-with-keyword-size__assertions.json");
}

#[test]
fn coverage_filler_background_image_should_keep_a_linear_gradient_with_keyword_background_size_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-background-image-should-keep-a-linear-gradient-with-keyword-background-size__assertions.json");
}

#[test]
fn coverage_filler_background_image_should_render_url_image_with_background_repeat_no_repeat_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-background-image-should-render-url-image-with-background-repeat-no-repeat__assertions.json");
}

#[test]
fn coverage_filler_background_image_should_support_background_size_auto_auto_with_a_url_image_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-background-image-should-support-background-size-auto-auto-with-a-url-image__assertions.json");
}

#[test]
fn coverage_filler_background_image_should_support_background_size_auto_ypct_with_a_url_image_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-background-image-should-support-background-size-auto-ypct-with-a-url-image__assertions.json");
}

#[test]
fn coverage_filler_background_image_should_support_background_size_xpct_auto_with_a_url_image_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-background-image-should-support-background-size-xpct-auto-with-a-url-image__assertions.json");
}

#[test]
fn coverage_filler_background_image_should_throw_on_invalid_background_image_value_not_gradient_url_color_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-background-image-should-throw-on-invalid-background-image-value-not-gradient-url-color__assertions.json");
}

#[test]
fn coverage_filler_background_image_should_tolerate_invalid_value_in_background_size_pair_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-background-image-should-tolerate-invalid-value-in-background-size-pair__assertions.json");
}

#[test]
fn coverage_filler_mask_image_image_edge_cases_should_fall_back_to_mask_size_when_mask_image_fails_to_load_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-mask-image-image-edge-cases-should-fall-back-to-mask-size-when-mask-image-fails-to-load__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_accept_object_position_bottom_xpct_uses_keyword_to_percent_for_bottom_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-accept-object-position-bottom-xpct-uses-keyword-to-percent-for-bottom__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_accept_object_position_top_ypct_vertical_kw_first_non_kw_second_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-accept-object-position-top-ypct-vertical-kw-first-non-kw-second__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_accept_object_position_with_a_single_non_keyword_value_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-accept-object-position-with-a-single-non-keyword-value__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_fall_back_to_contain_style_scaling_for_object_fit_scale_down_with_zero_natural_dimensions_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-fall-back-to-contain-style-scaling-for-object-fit-scale-down-with-zero-natural-dimensions__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_render_a_transformed_image_whose_own_clip_path_is_set_no_inherited_mask_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-render-a-transformed-image-whose-own-clip-path-is-set-no-inherited-mask__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_render_a_transformed_image_with_a_border_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-render-a-transformed-image-with-a-border__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_render_an_image_with_a_directional_border_and_explicit_padding_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-render-an-image-with-a-directional-border-and-explicit-padding__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_render_an_image_with_a_directional_border_covers_content_mask_border_branch_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-render-an-image-with-a-directional-border-covers-content-mask-border-branch__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_render_an_image_with_css_filter_style_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-render-an-image-with-css-filter-style__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_render_the_debug_bounding_rect_around_an_image_when_debug_is_enabled_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-render-the-debug-bounding-rect-around-an-image-when-debug-is-enabled__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_render_the_debug_bounding_rect_with_clip_path_when_image_is_inside_overflow_hidden_parent_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-render-the-debug-bounding-rect-with-clip-path-when-image-is-inside-overflow-hidden-parent__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_still_build_an_image_border_radius_clip_path_for_transformed_image_without_border_radius_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-still-build-an-image-border-radius-clip-path-for-transformed-image-without-border-radius__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_swallow_unparseable_object_position_values_in_the_catch_branch_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-swallow-unparseable-object-position-values-in-the-catch-branch__assertions.json");
}

#[test]
fn coverage_filler_rect_img_edge_cases_should_treat_a_non_length_object_position_value_as_0_assertions() {
    run_assertions_fixture("coverage-images__coverage-filler-rect-img-edge-cases-should-treat-a-non-length-object-position-value-as-0__assertions.json");
}
