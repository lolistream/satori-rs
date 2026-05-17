//! Snapshot tests sourced from `crates/satori-tests/fixtures/display__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn display_should_support_display_contents() {
    run_fixture("display__display-should-support-display-contents__1.json");
}
