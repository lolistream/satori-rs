//! Snapshot tests sourced from `crates/satori-tests/fixtures/line-height__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn line_height_should_work_correctly() {
    run_fixture("line-height__line-height-should-work-correctly__1.json");
}

#[test]
fn line_height_should_work_correctly_call2() {
    run_fixture("line-height__line-height-should-work-correctly__2.json");
}
