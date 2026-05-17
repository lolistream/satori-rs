//! Snapshot tests sourced from `fixtures/overflow__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn overflow_should_not_show_overflowed_text() {
    run_fixture("overflow__overflow-should-not-show-overflowed-text__1.json");
}

#[test]
fn overflow_should_not_work_when_overflow_is_not_hidden_and_overflow_property_should_not_be_inherited() {
    run_fixture("overflow__overflow-should-not-work-when-overflow-is-not-hidden-and-overflow-property-should-not-be-inherited__1.json");
}

#[test]
fn overflow_should_work_with_ellipsis_nowrap() {
    run_fixture("overflow__overflow-should-work-with-ellipsis-nowrap__1.json");
}

#[test]
fn overflow_should_work_with_nested_border_border_radius_padding() {
    run_fixture("overflow__overflow-should-work-with-nested-border-border-radius-padding__1.json");
}
