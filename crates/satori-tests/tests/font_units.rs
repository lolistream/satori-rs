//! Snapshot tests sourced from `crates/satori-tests/fixtures/font-units__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn font_loader_unit_tests_baseline_height_work_for_normal_line_height_and_a_numeric_line_height_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-baseline-height-work-for-normal-line-height-and-a-numeric-line-height__assertions.json");
}

#[test]
fn font_loader_unit_tests_classifies_lang_tagged_fonts_as_non_specified_when_locale_differs_covers_non_specified_lang_fonts_push_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-classifies-lang-tagged-fonts-as-non-specified-when-locale-differs-covers-non-specified-lang-fonts-push__assertions.json");
}

#[test]
fn font_loader_unit_tests_compare_font_covers_all_weight_comparison_sub_branches_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-compare-font-covers-all-weight-comparison-sub-branches__assertions.json");
}

#[test]
fn font_loader_unit_tests_compare_font_covers_style_and_weight_permutations_covers_283_289_and_weight_400_branches_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-compare-font-covers-style-and-weight-permutations-covers-283-289-and-weight-400-branches__assertions.json");
}

#[test]
fn font_loader_unit_tests_compares_multiple_fonts_under_the_same_name_to_pick_the_best_weight_match_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-compares-multiple-fonts-under-the-same-name-to-pick-the-best-weight-match__assertions.json");
}

#[test]
fn font_loader_unit_tests_falls_back_to_unknown_lang_suffix_when_no_lang_specified_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-falls-back-to-unknown-lang-suffix-when-no-lang-specified__assertions.json");
}

#[test]
fn font_loader_unit_tests_finds_font_with_the_exact_stored_key_covers_get_normal_truthy_return_branch_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-finds-font-with-the-exact-stored-key-covers-get-normal-truthy-return-branch__assertions.json");
}

#[test]
fn font_loader_unit_tests_get_svg_returns_a_non_empty_path_when_font_size_0_covers_final_return_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-get-svg-returns-a-non-empty-path-when-font-size-0-covers-final-return__assertions.json");
}

#[test]
fn font_loader_unit_tests_get_svg_returns_empty_path_boxes_when_font_size_is_0_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-get-svg-returns-empty-path-boxes-when-font-size-is-0__assertions.json");
}

#[test]
fn font_loader_unit_tests_has_returns_true_for_newline_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-has-returns-true-for-newline__assertions.json");
}

#[test]
fn font_loader_unit_tests_measure_returns_a_positive_width_for_known_text_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-measure-returns-a-positive-width-for-known-text__assertions.json");
}

#[test]
fn font_loader_unit_tests_rejects_fonts_with_an_invalid_lang_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-rejects-fonts-with-an-invalid-lang__assertions.json");
}

#[test]
fn font_loader_unit_tests_selects_lang_specific_fonts_via_get_lang_from_font_name_when_locale_is_given_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-selects-lang-specific-fonts-via-get-lang-from-font-name-when-locale-is-given__assertions.json");
}

#[test]
fn font_loader_unit_tests_throws_when_no_fonts_are_loaded_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-throws-when-no-fonts-are-loaded__assertions.json");
}

#[test]
fn font_loader_unit_tests_uses_additional_fonts_path_when_locale_set_but_font_has_unknown_lang_assertions() {
    run_assertions_fixture("font-units__font-loader-unit-tests-uses-additional-fonts-path-when-locale-set-but-font-has-unknown-lang__assertions.json");
}
