//! Snapshot tests sourced from `crates/satori-tests/fixtures/event__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn event_should_trigger_the_on_node_detected_callback_assertions() {
    run_assertions_fixture("event__event-should-trigger-the-on-node-detected-callback__assertions.json");
}
