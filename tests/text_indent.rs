//! Snapshot tests sourced from `fixtures/text-indent__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn text_indent_should_inherit_from_parent() {
    run_fixture("text-indent__text-indent-should-inherit-from-parent__1.json");
}

#[test]
fn text_indent_should_override_inherited_value() {
    run_fixture("text-indent__text-indent-should-override-inherited-value__1.json");
}

#[test]
fn text_indent_should_work_correctly_with_em_units() {
    run_fixture("text-indent__text-indent-should-work-correctly-with-em-units__1.json");
}

#[test]
fn text_indent_should_work_correctly_with_negative_indent_hanging_indent() {
    run_fixture("text-indent__text-indent-should-work-correctly-with-negative-indent-hanging-indent__1.json");
}

#[test]
fn text_indent_should_work_correctly_with_percentage_value() {
    run_fixture("text-indent__text-indent-should-work-correctly-with-percentage-value__1.json");
}

#[test]
fn text_indent_should_work_correctly_with_positive_pixel_indent() {
    run_fixture("text-indent__text-indent-should-work-correctly-with-positive-pixel-indent__1.json");
}

#[test]
fn text_indent_should_work_correctly_with_single_line_text() {
    run_fixture("text-indent__text-indent-should-work-correctly-with-single-line-text__1.json");
}

#[test]
fn text_indent_should_work_correctly_with_text_align_center() {
    run_fixture("text-indent__text-indent-should-work-correctly-with-text-align-center__1.json");
}

#[test]
fn text_indent_should_work_correctly_with_text_align_justify() {
    run_fixture("text-indent__text-indent-should-work-correctly-with-text-align-justify__1.json");
}

#[test]
fn text_indent_should_work_correctly_with_text_align_right() {
    run_fixture("text-indent__text-indent-should-work-correctly-with-text-align-right__1.json");
}

#[test]
fn text_indent_should_work_correctly_with_zero_indent() {
    run_fixture("text-indent__text-indent-should-work-correctly-with-zero-indent__1.json");
}
