//! Snapshot tests sourced from `crates/satori-tests/fixtures/layout__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn layout_should_stretch_items_by_default() {
    run_fixture("layout__layout-should-stretch-items-by-default__1.json");
}
