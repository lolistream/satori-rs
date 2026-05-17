//! Snapshot tests sourced from `crates/satori-tests/fixtures/handler-units__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn handler_variables_ts_extract_custom_properties_handles_entirely_no_variable_styles_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-extract-custom-properties-handles-entirely-no-variable-styles__assertions.json");
}

#[test]
fn handler_variables_ts_extract_custom_properties_returns_variables_and_rest_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-extract-custom-properties-returns-variables-and-rest__assertions.json");
}

#[test]
fn handler_variables_ts_merge_variables_overrides_inherited_with_current_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-merge-variables-overrides-inherited-with-current__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_style_variables_resolves_all_properties_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-style-variables-resolves-all-properties__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_falls_back_when_undefined_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-falls-back-when-undefined__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_handles_circular_reference_with_fallback_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-handles-circular-reference-with-fallback__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_handles_circular_reference_with_no_fallback_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-handles-circular-reference-with-no-fallback__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_handles_empty_var_no_nodes_inside_and_returns_string_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-handles-empty-var-no-nodes-inside-and-returns-string__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_handles_nested_var_in_declaration_value_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-handles-nested-var-in-declaration-value__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_handles_var_that_begins_with_non_word_returns_invalid_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-handles-var-that-begins-with-non-word-returns-invalid__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_resolves_basic_var_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-resolves-basic-var__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_returns_initial_when_var_undefined_and_no_fallback_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-returns-initial-when-var-undefined-and-no-fallback__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_returns_numbers_unchanged_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-returns-numbers-unchanged__assertions.json");
}

#[test]
fn handler_variables_ts_resolve_variables_returns_strings_without_var_unchanged_assertions() {
    run_assertions_fixture("handler-units__handler-variables-ts-resolve-variables-returns-strings-without-var-unchanged__assertions.json");
}

#[test]
fn language_ts_normalize_locale_returns_matching_locale_by_prefix_when_input_is_set_assertions() {
    run_assertions_fixture("handler-units__language-ts-normalize-locale-returns-matching-locale-by-prefix-when-input-is-set__assertions.json");
}

#[test]
fn language_ts_normalize_locale_returns_matching_locale_ignoring_case_assertions() {
    run_assertions_fixture("handler-units__language-ts-normalize-locale-returns-matching-locale-ignoring-case__assertions.json");
}

#[test]
fn language_ts_normalize_locale_returns_undefined_when_empty_string_is_provided_assertions() {
    run_assertions_fixture("handler-units__language-ts-normalize-locale-returns-undefined-when-empty-string-is-provided__assertions.json");
}

#[test]
fn language_ts_normalize_locale_returns_undefined_when_locale_is_not_provided_assertions() {
    run_assertions_fixture("handler-units__language-ts-normalize-locale-returns-undefined-when-locale-is-not-provided__assertions.json");
}

#[test]
fn language_ts_normalize_locale_returns_undefined_when_no_match_assertions() {
    run_assertions_fixture("handler-units__language-ts-normalize-locale-returns-undefined-when-no-match__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_auto_percentage_length_returns_auto_verbatim_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-auto-percentage-length-returns-auto-verbatim__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_auto_percentage_length_returns_formatted_percentage_strings_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-auto-percentage-length-returns-formatted-percentage-strings__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_auto_percentage_length_returns_numbers_as_is_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-auto-percentage-length-returns-numbers-as-is__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_auto_percentage_length_returns_undefined_and_warns_on_totally_invalid_value_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-auto-percentage-length-returns-undefined-and-warns-on-totally-invalid-value__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_auto_percentage_length_returns_undefined_on_invalid_percentage_and_warns_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-auto-percentage-length-returns-undefined-on-invalid-percentage-and-warns__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_auto_percentage_length_warns_without_property_name_when_omitted_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-auto-percentage-length-warns-without-property-name-when-omitted__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_percentage_length_returns_formatted_percentage_strings_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-percentage-length-returns-formatted-percentage-strings__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_percentage_length_returns_numbers_as_is_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-percentage-length-returns-numbers-as-is__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_percentage_length_returns_undefined_and_warns_on_entirely_invalid_value_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-percentage-length-returns-undefined-and-warns-on-entirely-invalid-value__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_percentage_length_returns_undefined_on_invalid_percentage_and_warns_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-percentage-length-returns-undefined-on-invalid-percentage-and-warns__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_as_point_percentage_length_warns_without_property_name_when_omitted_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-as-point-percentage-length-warns-without-property-name-when-omitted__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_midline_converts_camel_case_to_hyphenated_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-midline-converts-camel-case-to-hyphenated__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_midline_handles_multiple_capitals_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-midline-handles-multiple-capitals__assertions.json");
}

#[test]
fn utils_ts_pure_helpers_midline_handles_strings_without_capitals_assertions() {
    run_assertions_fixture("handler-units__utils-ts-pure-helpers-midline-handles-strings-without-capitals__assertions.json");
}
