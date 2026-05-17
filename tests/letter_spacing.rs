//! Snapshot tests sourced from `fixtures/letter-spacing__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn letter_spacing_should_render_letter_spacing_on_single_character() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-on-single-character__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_background_clip_text() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-background-clip-text__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_color() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-color__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_different_font_sizes() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-different-font-sizes__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_font_weight_bold() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-font-weight-bold__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_mixed_case_text() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-mixed-case-text__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_multiple_lines() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-multiple-lines__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_numbers() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-numbers__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_opacity() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-opacity__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_text_align_center() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-text-align-center__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_text_align_left() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-text-align-left__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_text_align_right() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-text-align-right__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_text_decoration_line_through() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-text-decoration-line-through__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_text_decoration_underline() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-text-decoration-underline__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_text_shadow() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-text-shadow__1.json");
}

#[test]
fn letter_spacing_should_render_letter_spacing_with_wrapped_text() {
    run_fixture("letter-spacing__letter-spacing-should-render-letter-spacing-with-wrapped-text__1.json");
}

#[test]
fn letter_spacing_should_render_text_with_large_letter_spacing() {
    run_fixture("letter-spacing__letter-spacing-should-render-text-with-large-letter-spacing__1.json");
}

#[test]
fn letter_spacing_should_render_text_with_negative_letter_spacing() {
    run_fixture("letter-spacing__letter-spacing-should-render-text-with-negative-letter-spacing__1.json");
}

#[test]
fn letter_spacing_should_render_text_with_positive_letter_spacing() {
    run_fixture("letter-spacing__letter-spacing-should-render-text-with-positive-letter-spacing__1.json");
}

#[test]
fn letter_spacing_should_render_text_with_very_small_letter_spacing() {
    run_fixture("letter-spacing__letter-spacing-should-render-text-with-very-small-letter-spacing__1.json");
}

#[test]
fn letter_spacing_should_render_text_with_zero_letter_spacing() {
    run_fixture("letter-spacing__letter-spacing-should-render-text-with-zero-letter-spacing__1.json");
}
