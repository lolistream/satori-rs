//! Snapshot tests sourced from `fixtures/coverage-builders__*.json`.

mod common;
use common::harness::{run_fixture, run_assertions_fixture};

#[test]
fn coverage_filler_builders_border_radius_should_resolve_size_correctly_when_one_side_is_small_and_other_exceeds_the_limit() {
    run_fixture("coverage-builders__coverage-filler-builders-border-radius-should-resolve-size-correctly-when-one-side-is-small-and-other-exceeds-the-limit__1.json");
}

#[test]
fn coverage_filler_builders_clip_path_should_return_empty_string_for_unrecognized_clip_path_value_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-clip-path-should-return-empty-string-for-unrecognized-clip-path-value__assertions.json");
}

#[test]
fn coverage_filler_builders_mask_image_should_render_no_mask_when_mask_image_is_none_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-mask-image-should-render-no-mask-when-mask-image-is-none__assertions.json");
}

#[test]
fn coverage_filler_builders_text_debug_rect_should_render_a_debug_bounding_rect_around_text_when_debug_embed_font_false_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-debug-rect-should-render-a-debug-bounding-rect-around-text-when-debug-embed-font-false__assertions.json");
}

#[test]
fn coverage_filler_builders_text_debug_rect_should_render_a_debug_rect_with_clip_path_when_embed_font_false_and_parent_has_overflow_hidden_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-debug-rect-should-render-a-debug-rect-with-clip-path-when-embed-font-false-and-parent-has-overflow-hidden__assertions.json");
}

#[test]
fn coverage_filler_builders_text_decoration_extra_branches_should_render_background_clip_text_inside_an_overflow_hidden_parent_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-decoration-extra-branches-should-render-background-clip-text-inside-an-overflow-hidden-parent__assertions.json");
}

#[test]
fn coverage_filler_builders_text_decoration_extra_branches_should_render_bordered_element_inside_overflow_hidden_parent_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-decoration-extra-branches-should-render-bordered-element-inside-overflow-hidden-parent__assertions.json");
}

#[test]
fn coverage_filler_builders_text_decoration_extra_branches_should_render_text_decoration_double_without_explicit_color_inherits_color_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-decoration-extra-branches-should-render-text-decoration-double-without-explicit-color-inherits-color__assertions.json");
}

#[test]
fn coverage_filler_builders_text_decoration_extra_branches_should_render_text_decoration_inside_overflow_hidden_parent_clip_path_wraps_the_lines_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-decoration-extra-branches-should-render-text-decoration-inside-overflow-hidden-parent-clip-path-wraps-the-lines__assertions.json");
}

#[test]
fn coverage_filler_builders_text_decoration_should_not_generate_decoration_markup_when_text_decoration_line_is_none_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-decoration-should-not-generate-decoration-markup-when-text-decoration-line-is-none__assertions.json");
}

#[test]
fn coverage_filler_builders_text_image_grapheme_branches_should_render_image_grapheme_inside_overflow_hidden_parent_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-image-grapheme-branches-should-render-image-grapheme-inside-overflow-hidden-parent__assertions.json");
}

#[test]
fn coverage_filler_builders_text_image_grapheme_branches_should_render_image_grapheme_with_css_filter_style_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-image-grapheme-branches-should-render-image-grapheme-with-css-filter-style__assertions.json");
}

#[test]
fn coverage_filler_builders_text_image_grapheme_branches_should_render_image_grapheme_with_opacity_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-image-grapheme-branches-should-render-image-grapheme-with-opacity__assertions.json");
}

#[test]
fn coverage_filler_builders_text_image_grapheme_branches_should_render_image_grapheme_with_text_shadow_drop_shadow_filter_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-image-grapheme-branches-should-render-image-grapheme-with-text-shadow-drop-shadow-filter__assertions.json");
}

#[test]
fn coverage_filler_builders_text_non_embedded_font_branches_should_render_gradient_text_via_background_clip_text_with_embed_font_false_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-non-embedded-font-branches-should-render-gradient-text-via-background-clip-text-with-embed-font-false__assertions.json");
}

#[test]
fn coverage_filler_builders_text_non_embedded_font_branches_should_render_text_inside_overflow_hidden_parent_with_embed_font_false_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-non-embedded-font-branches-should-render-text-inside-overflow-hidden-parent-with-embed-font-false__assertions.json");
}

#[test]
fn coverage_filler_builders_text_non_embedded_font_branches_should_render_text_with_css_filter_and_embed_font_false_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-non-embedded-font-branches-should-render-text-with-css-filter-and-embed-font-false__assertions.json");
}

#[test]
fn coverage_filler_builders_text_non_embedded_font_branches_should_render_text_with_opacity_1_and_embed_font_false_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-non-embedded-font-branches-should-render-text-with-opacity-1-and-embed-font-false__assertions.json");
}

#[test]
fn coverage_filler_builders_text_non_embedded_font_branches_should_render_text_with_webkit_text_stroke_and_embed_font_false_assertions() {
    run_assertions_fixture("coverage-builders__coverage-filler-builders-text-non-embedded-font-branches-should-render-text-with-webkit-text-stroke-and-embed-font-false__assertions.json");
}
