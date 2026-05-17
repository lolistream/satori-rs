//! Snapshot tests sourced from `crates/satori-tests/fixtures/coverage-parser__*.json`.

use satori_tests::harness::{run_fixture, run_assertions_fixture};

#[test]
fn parser_coverage_mask_should_resolve_mask_via_webkit_mask_image_when_mask_image_is_unset_assertions() {
    run_assertions_fixture("coverage-parser__parser-coverage-mask-should-resolve-mask-via-webkit-mask-image-when-mask-image-is-unset__assertions.json");
}

#[test]
fn parser_coverage_shape_should_fall_back_to_50_radii_when_ellipse_omits_explicit_radii() {
    run_fixture("coverage-parser__parser-coverage-shape-should-fall-back-to-50-radii-when-ellipse-omits-explicit-radii__1.json");
}

#[test]
fn parser_coverage_shape_should_handle_inset_with_zero_offsets_and_no_round_corners() {
    run_fixture("coverage-parser__parser-coverage-shape-should-handle-inset-with-zero-offsets-and-no-round-corners__1.json");
}

#[test]
fn parser_coverage_shape_should_ignore_unknown_clip_path_values_without_throwing_assertions() {
    run_assertions_fixture("coverage-parser__parser-coverage-shape-should-ignore-unknown-clip-path-values-without-throwing__assertions.json");
}

#[test]
fn parser_coverage_shape_should_resolve_circle_clip_path_keyword_positions_top_right_center() {
    run_fixture("coverage-parser__parser-coverage-shape-should-resolve-circle-clip-path-keyword-positions-top-right-center__1.json");
}

#[test]
fn parser_coverage_shape_should_resolve_circle_clip_path_keyword_positions_top_right_center_call2() {
    run_fixture("coverage-parser__parser-coverage-shape-should-resolve-circle-clip-path-keyword-positions-top-right-center__2.json");
}

#[test]
fn parser_coverage_shape_should_resolve_circle_clip_path_keyword_positions_top_right_center_call3() {
    run_fixture("coverage-parser__parser-coverage-shape-should-resolve-circle-clip-path-keyword-positions-top-right-center__3.json");
}

#[test]
fn parser_coverage_shape_should_resolve_circle_clip_path_keyword_positions_top_right_center_call4() {
    run_fixture("coverage-parser__parser-coverage-shape-should-resolve-circle-clip-path-keyword-positions-top-right-center__4.json");
}

#[test]
fn parser_coverage_shape_should_support_polygon_and_path_with_explicit_nonzero_fill_rule() {
    run_fixture("coverage-parser__parser-coverage-shape-should-support-polygon-and-path-with-explicit-nonzero-fill-rule__1.json");
}

#[test]
fn parser_coverage_shape_should_support_polygon_and_path_with_explicit_nonzero_fill_rule_call2() {
    run_fixture("coverage-parser__parser-coverage-shape-should-support-polygon-and-path-with-explicit-nonzero-fill-rule__2.json");
}
