//! Snapshot tests sourced from `fixtures/layout__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn layout_should_stretch_items_by_default() {
    run_fixture("layout__layout-should-stretch-items-by-default__1.json");
}
