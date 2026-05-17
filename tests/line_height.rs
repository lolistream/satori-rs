//! Snapshot tests sourced from `fixtures/line-height__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn line_height_should_work_correctly() {
    run_fixture("line-height__line-height-should-work-correctly__1.json");
}

#[test]
fn line_height_should_work_correctly_call2() {
    run_fixture("line-height__line-height-should-work-correctly__2.json");
}
