//! Snapshot tests sourced from `crates/satori-tests/fixtures/coverage-text__*.json`.

use satori_tests::harness::{run_fixture, run_assertions_fixture};

#[test]
fn text_coverage_index_should_accept_text_indent_expressed_as_0_assertions() {
    run_assertions_fixture("coverage-text__text-coverage-index-should-accept-text-indent-expressed-as-0__assertions.json");
}

#[test]
fn text_coverage_index_should_collect_decoration_boxes_for_non_embedded_underline_text_with_descenders_assertions() {
    run_assertions_fixture("coverage-text__text-coverage-index-should-collect-decoration-boxes-for-non-embedded-underline-text-with-descenders__assertions.json");
}

#[test]
fn text_coverage_index_should_not_divide_by_zero_when_justifying_a_line_with_a_single_segment_assertions() {
    run_assertions_fixture("coverage-text__text-coverage-index-should-not-divide-by-zero-when-justifying-a-line-with-a-single-segment__assertions.json");
}

#[test]
fn text_coverage_index_should_preserve_consecutive_newlines_under_white_space_pre() {
    run_fixture("coverage-text__text-coverage-index-should-preserve-consecutive-newlines-under-white-space-pre__1.json");
}

#[test]
fn text_coverage_index_should_reflow_short_last_lines_via_text_wrap_pretty() {
    run_fixture("coverage-text__text-coverage-index-should-reflow-short-last-lines-via-text-wrap-pretty__1.json");
}

#[test]
fn text_coverage_index_should_render_debug_overlays_inside_a_clipped_transformed_parent_assertions() {
    run_assertions_fixture("coverage-text__text-coverage-index-should-render-debug-overlays-inside-a-clipped-transformed-parent__assertions.json");
}

#[test]
fn text_coverage_index_should_render_debug_overlays_with_transform() {
    run_fixture("coverage-text__text-coverage-index-should-render-debug-overlays-with-transform__1.json");
}

#[test]
fn text_coverage_index_should_render_decoration_only_output_when_text_is_fully_transparent_with_shadow_assertions() {
    run_assertions_fixture("coverage-text__text-coverage-index-should-render-decoration-only-output-when-text-is-fully-transparent-with-shadow__assertions.json");
}

#[test]
fn text_coverage_index_should_render_no_descender_text_with_underline_decoration_glyphs_fallback_assertions() {
    run_assertions_fixture("coverage-text__text-coverage-index-should-render-no-descender-text-with-underline-decoration-glyphs-fallback__assertions.json");
}

#[test]
fn text_coverage_index_should_render_text_with_css_filter_applied_via_parent_style_assertions() {
    run_assertions_fixture("coverage-text__text-coverage-index-should-render-text-with-css-filter-applied-via-parent-style__assertions.json");
}

#[test]
fn text_coverage_index_should_render_text_with_debug_overlays() {
    run_fixture("coverage-text__text-coverage-index-should-render-text-with-debug-overlays__1.json");
}

#[test]
fn text_coverage_index_should_respect_explicit_string_tab_size() {
    run_fixture("coverage-text__text-coverage-index-should-respect-explicit-string-tab-size__1.json");
}

#[test]
fn text_coverage_index_should_treat_tab_size_0_specially_in_tab_handling() {
    run_fixture("coverage-text__text-coverage-index-should-treat-tab-size-0-specially-in-tab-handling__1.json");
}

#[test]
fn text_coverage_processor_should_capitalize_text_via_text_transform() {
    run_fixture("coverage-text__text-coverage-processor-should-capitalize-text-via-text-transform__1.json");
}

#[test]
fn text_coverage_processor_should_clamp_lines_with_display_webkit_box_and_webkit_line_clamp() {
    run_fixture("coverage-text__text-coverage-processor-should-clamp-lines-with-display-webkit-box-and-webkit-line-clamp__1.json");
}

#[test]
fn text_coverage_processor_should_fall_back_when_line_clamp_string_does_not_match_either_regex_assertions() {
    run_assertions_fixture("coverage-text__text-coverage-processor-should-fall-back-when-line-clamp-string-does-not-match-either-regex__assertions.json");
}

#[test]
fn text_coverage_processor_should_lowercase_text_via_text_transform() {
    run_fixture("coverage-text__text-coverage-processor-should-lowercase-text-via-text-transform__1.json");
}

#[test]
fn text_coverage_processor_should_uppercase_text_via_text_transform() {
    run_fixture("coverage-text__text-coverage-processor-should-uppercase-text-via-text-transform__1.json");
}
