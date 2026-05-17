//! Snapshot tests sourced from `fixtures/position__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn position_absolute_should_have_correct_size_calculation_of_absolutely_positioned_elements() {
    run_fixture("position__position-absolute-should-have-correct-size-calculation-of-absolutely-positioned-elements__1.json");
}

#[test]
fn position_absolute_should_support_absolute_position() {
    run_fixture("position__position-absolute-should-support-absolute-position__1.json");
}

#[test]
fn position_relative_should_support_relative_position() {
    run_fixture("position__position-relative-should-support-relative-position__1.json");
}

#[test]
fn position_static_should_support_static_position() {
    run_fixture("position__position-static-should-support-static-position__1.json");
}
