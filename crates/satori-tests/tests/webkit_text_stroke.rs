//! Snapshot tests sourced from `crates/satori-tests/fixtures/webkit-text-stroke__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn webkit_text_stroke_should_work_basic_text_stroke() {
    run_fixture("webkit-text-stroke__webkit-text-stroke-should-work-basic-text-stroke__1.json");
}

#[test]
fn webkit_text_stroke_should_work_nested_and_complex_text_stroke() {
    run_fixture("webkit-text-stroke__webkit-text-stroke-should-work-nested-and-complex-text-stroke__1.json");
}

#[test]
fn webkit_text_stroke_should_work_nested_text_stroke() {
    run_fixture("webkit-text-stroke__webkit-text-stroke-should-work-nested-text-stroke__1.json");
}
