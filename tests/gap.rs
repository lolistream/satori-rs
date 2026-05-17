//! Snapshot tests sourced from `fixtures/gap__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn flex_gap_should_support_gap() {
    run_fixture("gap__flex-gap-should-support-gap__1.json");
}

#[test]
fn flex_gap_should_support_percentage_values_as_gap() {
    run_fixture("gap__flex-gap-should-support-percentage-values-as-gap__1.json");
}

#[test]
fn flex_gap_should_support_row_gap_and_column_gap() {
    run_fixture("gap__flex-gap-should-support-row-gap-and-column-gap__1.json");
}
