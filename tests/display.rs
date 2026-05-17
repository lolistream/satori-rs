//! Snapshot tests sourced from `fixtures/display__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn display_should_support_display_contents() {
    run_fixture("display__display-should-support-display-contents__1.json");
}
