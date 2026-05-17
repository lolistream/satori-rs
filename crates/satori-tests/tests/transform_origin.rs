//! Snapshot tests sourced from `crates/satori-tests/fixtures/transform-origin__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn parse_transform_origin_unit_handles_single_em_value_horizontal_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-single-em-value-horizontal__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_single_keyword_bottom_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-single-keyword-bottom__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_single_keyword_center_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-single-keyword-center__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_single_keyword_left_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-single-keyword-left__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_single_keyword_right_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-single-keyword-right__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_single_keyword_top_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-single-keyword-top__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_single_percentage_value_horizontal_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-single-percentage-value-horizontal__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_single_px_value_horizontal_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-single-px-value-horizontal__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_single_rem_value_horizontal_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-single-rem-value-horizontal__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_two_values_bottom_then_right_swaps_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-two-values-bottom-then-right-swaps__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_two_values_center_left_swaps_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-two-values-center-left-swaps__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_two_values_center_right_swaps_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-two-values-center-right-swaps__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_two_values_left_then_top_no_swap_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-two-values-left-then-top-no-swap__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_two_values_pixel_x_and_pixel_y_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-two-values-pixel-x-and-pixel-y__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_two_values_top_then_left_swaps_to_horizontal_first_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-two-values-top-then-left-swaps-to-horizontal-first__assertions.json");
}

#[test]
fn parse_transform_origin_unit_handles_two_values_x_and_y_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-handles-two-values-x-and-y__assertions.json");
}

#[test]
fn parse_transform_origin_unit_returns_empty_for_unrecognized_single_unit_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-returns-empty-for-unrecognized-single-unit__assertions.json");
}

#[test]
fn parse_transform_origin_unit_returns_empty_when_css_dimension_throws_invalid_word_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-returns-empty-when-css-dimension-throws-invalid-word__assertions.json");
}

#[test]
fn parse_transform_origin_unit_returns_empty_when_more_than_two_words_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-returns-empty-when-more-than-two-words__assertions.json");
}

#[test]
fn parse_transform_origin_unit_returns_empty_when_value_parser_yields_no_word_nodes_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-returns-empty-when-value-parser-yields-no-word-nodes__assertions.json");
}

#[test]
fn parse_transform_origin_unit_returns_x_absolute_when_value_is_a_number_assertions() {
    run_assertions_fixture("transform-origin__parse-transform-origin-unit-returns-x-absolute-when-value-is-a-number__assertions.json");
}

#[test]
fn transform_origin_integration_via_satori_passes_transform_origin_through_expand_without_crashing_assertions() {
    run_assertions_fixture("transform-origin__transform-origin-integration-via-satori-passes-transform-origin-through-expand-without-crashing__assertions.json");
}
