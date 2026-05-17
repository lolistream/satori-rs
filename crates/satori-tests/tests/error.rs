//! Snapshot tests sourced from `crates/satori-tests/fixtures/error__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn error_should_not_allowed_to_set_negative_value_to_rg_size_assertions() {
    run_assertions_fixture("error__error-should-not-allowed-to-set-negative-value-to-rg-size__assertions.json");
}

#[test]
fn error_should_not_throw_if_display_none_on_div_that_has_children_assertions() {
    run_assertions_fixture("error__error-should-not-throw-if-display-none-on-div-that-has-children__assertions.json");
}

#[test]
fn error_should_not_throw_if_flex_missing_on_div_without_children_assertions() {
    run_assertions_fixture("error__error-should-not-throw-if-flex-missing-on-div-without-children__assertions.json");
}

#[test]
fn error_should_not_throw_if_flex_missing_on_span_that_has_children_assertions() {
    run_assertions_fixture("error__error-should-not-throw-if-flex-missing-on-span-that-has-children__assertions.json");
}

#[test]
fn error_should_throw_if_display_inline_block_on_div_that_has_children_assertions() {
    run_assertions_fixture("error__error-should-throw-if-display-inline-block-on-div-that-has-children__assertions.json");
}

#[test]
fn error_should_throw_if_flex_missing_on_div_that_has_children_assertions() {
    run_assertions_fixture("error__error-should-throw-if-flex-missing-on-div-that-has-children__assertions.json");
}

#[test]
fn error_should_throw_if_using_invalid_values_assertions() {
    run_assertions_fixture("error__error-should-throw-if-using-invalid-values__assertions.json");
}
