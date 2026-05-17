//! Snapshot tests sourced from `fixtures/coverage-mocks__*.json`.

mod common;
use common::harness::run_assertions_fixture;

#[test]
fn image_ts_180_typeof_window_undefined_false_branch_allows_relative_ur_ls_in_a_browser_like_environment_window_defined_assertions() {
    run_assertions_fixture("coverage-mocks__image-ts-180-typeof-window-undefined-false-branch-allows-relative-ur-ls-in-a-browser-like-environment-window-defined__assertions.json");
}

#[test]
fn image_ts_svg_dimension_parsing_edge_cases_parses_an_svg_with_view_box_and_height_only_image_ts_124_assertions() {
    run_assertions_fixture("coverage-mocks__image-ts-svg-dimension-parsing-edge-cases-parses-an-svg-with-view-box-and-height-only-image-ts-124__assertions.json");
}

#[test]
fn image_ts_svg_dimension_parsing_edge_cases_parses_an_svg_with_view_box_and_width_only_image_ts_122_assertions() {
    run_assertions_fixture("coverage-mocks__image-ts-svg-dimension-parsing-edge-cases-parses-an-svg-with-view-box-and-width-only-image-ts-122__assertions.json");
}

#[test]
fn layout_ts_127_props_fallback_renders_an_element_returned_from_a_function_component_with_no_props_key_assertions() {
    run_assertions_fixture("coverage-mocks__layout-ts-127-props-fallback-renders-an-element-returned-from-a-function-component-with-no-props-key__assertions.json");
}

#[test]
fn preprocess_ts_edge_branches_pre_process_node_handles_a_falsy_node_early_preprocess_ts_walk_assertions() {
    run_assertions_fixture("coverage-mocks__preprocess-ts-edge-branches-pre-process-node-handles-a-falsy-node-early-preprocess-ts-walk__assertions.json");
}

#[test]
fn preprocess_ts_edge_branches_pre_process_node_handles_a_node_with_no_props_preprocess_ts_157_assertions() {
    run_assertions_fixture("coverage-mocks__preprocess-ts-edge-branches-pre-process-node-handles-a-node-with-no-props-preprocess-ts-157__assertions.json");
}

#[test]
fn preprocess_ts_edge_branches_svg_node_to_image_handles_an_svg_node_whose_props_is_undefined_preprocess_ts_201_assertions() {
    run_assertions_fixture("coverage-mocks__preprocess-ts-edge-branches-svg-node-to-image-handles-an-svg-node-whose-props-is-undefined-preprocess-ts-201__assertions.json");
}

#[test]
fn preprocess_ts_edge_branches_translate_svg_node_to_svg_string_handles_an_element_with_no_props_preprocess_ts_116_assertions() {
    run_assertions_fixture("coverage-mocks__preprocess-ts-edge-branches-translate-svg-node-to-svg-string-handles-an-element-with-no-props-preprocess-ts-116__assertions.json");
}

#[test]
fn preprocess_ts_edge_branches_translate_svg_node_to_svg_string_resolves_image_href_via_cache_preprocess_ts_126_assertions() {
    run_assertions_fixture("coverage-mocks__preprocess-ts-edge-branches-translate-svg-node-to-svg-string-resolves-image-href-via-cache-preprocess-ts-126__assertions.json");
}

#[test]
fn transform_origin_ts_91_postcss_value_parser_throws_returns_when_value_parser_throws_assertions() {
    run_assertions_fixture("coverage-mocks__transform-origin-ts-91-postcss-value-parser-throws-returns-when-value-parser-throws__assertions.json");
}

#[test]
fn utils_ts_191_intl_segmenter_unavailable_throws_when_intl_segmenter_is_missing_assertions() {
    run_assertions_fixture("coverage-mocks__utils-ts-191-intl-segmenter-unavailable-throws-when-intl-segmenter-is-missing__assertions.json");
}

#[test]
fn variables_ts_138_141_value_parser_throws_inside_resolve_variables_falls_through_to_original_value_when_parsing_throws_assertions() {
    run_assertions_fixture("coverage-mocks__variables-ts-138-141-value-parser-throws-inside-resolve-variables-falls-through-to-original-value-when-parsing-throws__assertions.json");
}
