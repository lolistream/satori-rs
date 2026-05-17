//! Snapshot tests sourced from `fixtures/box-sizing__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn box_sizing_should_default_to_border_box() {
    run_fixture("box-sizing__box-sizing-should-default-to-border-box__1.json");
}

#[test]
fn box_sizing_should_support_content_box() {
    run_fixture("box-sizing__box-sizing-should-support-content-box__1.json");
}
