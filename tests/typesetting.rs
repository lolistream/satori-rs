//! Snapshot tests sourced from `fixtures/typesetting__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn typesetting_should_wrap_normally_for_special_characters() {
    run_fixture("typesetting__typesetting-should-wrap-normally-for-special-characters__1.json");
}

#[test]
fn typesetting_should_wrap_normally() {
    run_fixture("typesetting__typesetting-should-wrap-normally__1.json");
}
