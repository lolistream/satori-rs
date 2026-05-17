//! Snapshot tests sourced from `fixtures/white-space__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn white_space_normal_should_have_line_break_before_fast() {
    run_fixture("white-space__white-space-normal-should-have-line-break-before-fast__1.json");
}

#[test]
fn white_space_normal_should_not_render_extra_line_breaks_with_white_space_normal() {
    run_fixture("white-space__white-space-normal-should-not-render-extra-line-breaks-with-white-space-normal__1.json");
}

#[test]
fn white_space_normal_should_not_render_extra_spaces_with_white_space_normal() {
    run_fixture("white-space__white-space-normal-should-not-render-extra-spaces-with-white-space-normal__1.json");
}

#[test]
fn white_space_normal_should_wrap_automatically_with_white_space_normal() {
    run_fixture("white-space__white-space-normal-should-wrap-automatically-with-white-space-normal__1.json");
}

#[test]
fn white_space_pre_should_always_preserve_extra_line_breaks_with_white_space_pre() {
    run_fixture("white-space__white-space-pre-should-always-preserve-extra-line-breaks-with-white-space-pre__1.json");
}

#[test]
fn white_space_pre_should_always_preserve_extra_spaces_with_white_space_pre() {
    run_fixture("white-space__white-space-pre-should-always-preserve-extra-spaces-with-white-space-pre__1.json");
}

#[test]
fn white_space_pre_should_not_wrap_with_white_space_pre() {
    run_fixture("white-space__white-space-pre-should-not-wrap-with-white-space-pre__1.json");
}

#[test]
fn white_space_pre_should_render_line_breaks_correctly_without_separators() {
    run_fixture("white-space__white-space-pre-should-render-line-breaks-correctly-without-separators__1.json");
}

#[test]
fn white_space_with_n_in_content_should_render_consecutive_line_breaks_with_pre() {
    run_fixture("white-space__white-space-with-n-in-content-should-render-consecutive-line-breaks-with-pre__1.json");
}

#[test]
fn white_space_with_n_in_content_should_render_n_as_a_line_break_with_pre() {
    run_fixture("white-space__white-space-with-n-in-content-should-render-n-as-a-line-break-with-pre__1.json");
}

#[test]
fn white_space_with_n_in_content_should_render_n_as_a_whitespace() {
    run_fixture("white-space__white-space-with-n-in-content-should-render-n-as-a-whitespace__1.json");
}

#[test]
fn white_space_with_white_space_nowrap_should_not_wrap_with_white_space_nowrap_and_swallow_extra_spaces() {
    run_fixture("white-space__white-space-with-white-space-nowrap-should-not-wrap-with-white-space-nowrap-and-swallow-extra-spaces__1.json");
}

#[test]
fn white_space_with_white_space_pre_line_should_always_collapse_spaces_and_preserve_line_breaks_with_white_space_pre_line() {
    run_fixture("white-space__white-space-with-white-space-pre-line-should-always-collapse-spaces-and-preserve-line-breaks-with-white-space-pre-line__1.json");
}

#[test]
fn white_space_with_white_space_pre_wrap_should_always_preserve_extra_line_breaks_with_white_space_pre_wrap() {
    run_fixture("white-space__white-space-with-white-space-pre-wrap-should-always-preserve-extra-line-breaks-with-white-space-pre-wrap__1.json");
}

#[test]
fn white_space_with_white_space_pre_wrap_should_always_preserve_extra_spaces_with_white_space_pre_wrap() {
    run_fixture("white-space__white-space-with-white-space-pre-wrap-should-always-preserve-extra-spaces-with-white-space-pre-wrap__1.json");
}

#[test]
fn white_space_with_white_space_pre_wrap_should_automatically_wrap_with_white_space_pre_wrap() {
    run_fixture("white-space__white-space-with-white-space-pre-wrap-should-automatically-wrap-with-white-space-pre-wrap__1.json");
}
