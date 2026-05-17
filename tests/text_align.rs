//! Snapshot tests sourced from `fixtures/text-align__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn text_align_should_work_correctly_when_text_align_center() {
    run_fixture("text-align__text-align-should-work-correctly-when-text-align-center__1.json");
}

#[test]
fn text_align_should_work_correctly_when_text_align_end() {
    run_fixture("text-align__text-align-should-work-correctly-when-text-align-end__1.json");
}

#[test]
fn text_align_should_work_correctly_when_text_align_justify() {
    run_fixture("text-align__text-align-should-work-correctly-when-text-align-justify__1.json");
}

#[test]
fn text_align_should_work_correctly_when_text_align_left() {
    run_fixture("text-align__text-align-should-work-correctly-when-text-align-left__1.json");
}

#[test]
fn text_align_should_work_correctly_when_text_align_right() {
    run_fixture("text-align__text-align-should-work-correctly-when-text-align-right__1.json");
}
