//! Snapshot tests sourced from `fixtures/satori-init-error__*.json`.

mod common;
use common::harness::run_assertions_fixture;

#[test]
fn satori_initialization_throws_when_get_yoga_returns_no_yoga_instance_assertions() {
    run_assertions_fixture("satori-init-error__satori-initialization-throws-when-get-yoga-returns-no-yoga-instance__assertions.json");
}
