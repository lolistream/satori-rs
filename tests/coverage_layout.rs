//! Snapshot tests sourced from `fixtures/coverage-layout__*.json`.

mod common;
use common::harness::run_assertions_fixture;

#[test]
fn coverage_filler_layout_should_accept_a_function_component_that_returns_null_assertions() {
    run_assertions_fixture("coverage-layout__coverage-filler-layout-should-accept-a-function-component-that-returns-null__assertions.json");
}

#[test]
fn coverage_filler_layout_should_accept_a_null_returning_component_combined_with_load_additional_asset_assertions() {
    run_assertions_fixture("coverage-layout__coverage-filler-layout-should-accept-a-null-returning-component-combined-with-load-additional-asset__assertions.json");
}

#[test]
fn coverage_filler_layout_should_throw_when_a_class_component_is_rendered_assertions() {
    run_assertions_fixture("coverage-layout__coverage-filler-layout-should-throw-when-a-class-component-is-rendered__assertions.json");
}

#[test]
fn coverage_filler_layout_should_throw_when_dangerously_set_inner_html_is_used_assertions() {
    run_assertions_fixture("coverage-layout__coverage-filler-layout-should-throw-when-dangerously-set-inner-html-is-used__assertions.json");
}
