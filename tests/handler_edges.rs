//! Snapshot tests sourced from `fixtures/handler-edges__*.json`.

mod common;
use common::harness::run_assertions_fixture;

#[test]
fn compute_ts_edge_branches_div_with_max_width_and_max_height_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-div-with-max-width-and-max-height__assertions.json");
}

#[test]
fn compute_ts_edge_branches_div_with_min_width_and_min_height_covers_set_min_branches_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-div-with-min-width-and-min-height-covers-set-min-branches__assertions.json");
}

#[test]
fn compute_ts_edge_branches_img_with_height_only_and_a_known_aspect_ratio_derives_width_covers_content_box_width_height_r_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-img-with-height-only-and-a-known-aspect-ratio-derives-width-covers-content-box-width-height-r__assertions.json");
}

#[test]
fn compute_ts_edge_branches_img_with_style_height_as_percentage_and_known_aspect_ratio_uses_aspect_ratio_covers_content_box_width_undefined_string_branch_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-img-with-style-height-as-percentage-and-known-aspect-ratio-uses-aspect-ratio-covers-content-box-width-undefined-string-branch__assertions.json");
}

#[test]
fn compute_ts_edge_branches_svg_with_only_height_and_no_view_box_sets_width_0_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-svg-with-only-height-and-no-view-box-sets-width-0__assertions.json");
}

#[test]
fn compute_ts_edge_branches_svg_with_only_height_string_and_view_box_derives_width_as_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-svg-with-only-height-string-and-view-box-derives-width-as__assertions.json");
}

#[test]
fn compute_ts_edge_branches_svg_with_only_width_and_no_view_box_sets_height_0_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-svg-with-only-width-and-no-view-box-sets-height-0__assertions.json");
}

#[test]
fn compute_ts_edge_branches_svg_with_only_width_string_and_view_box_derives_height_as_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-svg-with-only-width-string-and-view-box-derives-height-as__assertions.json");
}

#[test]
fn compute_ts_edge_branches_svg_with_width_and_height_both_set_and_a_view_box_covers_else_branch_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-svg-with-width-and-height-both-set-and-a-view-box-covers-else-branch__assertions.json");
}

#[test]
fn compute_ts_edge_branches_svg_with_width_auto_like_unparseable_string_falls_back_to_raw_value_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-svg-with-width-auto-like-unparseable-string-falls-back-to-raw-value__assertions.json");
}

#[test]
fn compute_ts_edge_branches_throws_when_img_size_cannot_be_determined_covers_compute_ts_size_error_assertions() {
    run_assertions_fixture("handler-edges__compute-ts-edge-branches-throws-when-img-size-cannot-be-determined-covers-compute-ts-size-error__assertions.json");
}

#[test]
fn expand_ts_edge_branches_passes_through_internal_style_properties_covers_underscore_branch_assertions() {
    run_assertions_fixture("handler-edges__expand-ts-edge-branches-passes-through-internal-style-properties-covers-underscore-branch__assertions.json");
}

#[test]
fn expand_ts_edge_branches_warns_when_z_index_is_used_covers_handle_special_case_z_index_branch_assertions() {
    run_assertions_fixture("handler-edges__expand-ts-edge-branches-warns-when-z-index-is-used-covers-handle-special-case-z-index-branch__assertions.json");
}

#[test]
fn expand_ts_edge_branches_wraps_errors_thrown_from_special_case_handlers_with_rule_context_assertions() {
    run_assertions_fixture("handler-edges__expand-ts-edge-branches-wraps-errors-thrown-from-special-case-handlers-with-rule-context__assertions.json");
}

#[test]
fn preprocess_ts_edge_branches_handles_svg_image_children_without_href_covers_image_src_falsy_branch_assertions() {
    run_assertions_fixture("handler-edges__preprocess-ts-edge-branches-handles-svg-image-children-without-href-covers-image-src-falsy-branch__assertions.json");
}

#[test]
fn preprocess_ts_edge_branches_rejects_when_an_svg_text_child_is_passed_covers_text_type_throw_assertions() {
    run_assertions_fixture("handler-edges__preprocess-ts-edge-branches-rejects-when-an-svg-text-child-is-passed-covers-text-type-throw__assertions.json");
}

#[test]
fn preprocess_ts_edge_branches_renders_svg_with_array_children_covers_array_is_array_branch_in_translate_svg_node_to_svg_string_assertions() {
    run_assertions_fixture("handler-edges__preprocess-ts-edge-branches-renders-svg-with-array-children-covers-array-is-array-branch-in-translate-svg-node-to-svg-string__assertions.json");
}
