//! Snapshot tests sourced from `crates/satori-tests/fixtures/font__*.json`.

use satori_tests::harness::{run_fixture, run_assertions_fixture};

#[test]
fn font_font_size_should_allow_font_size_to_be_0() {
    run_fixture("font__font-font-size-should-allow-font-size-to-be-0__1.json");
}

#[test]
fn font_should_error_when_no_font_is_specified_assertions() {
    run_assertions_fixture("font__font-should-error-when-no-font-is-specified__assertions.json");
}

#[test]
fn font_should_handle_escape_html_when_embed_font_is_false() {
    run_fixture("font__font-should-handle-escape-html-when-embed-font-is-false__1.json");
}

#[test]
fn font_should_handle_font_family_fallback() {
    run_fixture("font__font-should-handle-font-family-fallback__1.json");
}

#[test]
fn font_should_handle_font_size_correctly_for_element_like_heading() {
    run_fixture("font__font-should-handle-font-size-correctly-for-element-like-heading__1.json");
}

#[test]
fn font_should_handle_font_size_correctly_for_element_like_heading_call2() {
    run_fixture("font__font-should-handle-font-size-correctly-for-element-like-heading__2.json");
}

#[test]
fn font_should_handle_font_size_correctly_for_element_like_heading_call3() {
    run_fixture("font__font-should-handle-font-size-correctly-for-element-like-heading__3.json");
}

#[test]
fn font_should_not_error_when_no_font_is_specified_and_no_text_rendered() {
    run_fixture("font__font-should-not-error-when-no-font-is-specified-and-no-text-rendered__1.json");
}

#[test]
fn font_should_use_correct_fonts() {
    run_fixture("font__font-should-use-correct-fonts__1.json");
}
