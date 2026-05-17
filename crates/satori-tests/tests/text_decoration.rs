//! Snapshot tests sourced from `crates/satori-tests/fixtures/text-decoration__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn text_decoration_should_render_continuous_line_when_text_decoration_skip_ink_none() {
    run_fixture("text-decoration__text-decoration-should-render-continuous-line-when-text-decoration-skip-ink-none__1.json");
}

#[test]
fn text_decoration_should_skip_ink_by_default_when_text_decoration_line_underline() {
    run_fixture("text-decoration__text-decoration-should-skip-ink-by-default-when-text-decoration-line-underline__1.json");
}

#[test]
fn text_decoration_should_skip_ink_correctly_with_complex_descenders() {
    run_fixture("text-decoration__text-decoration-should-skip-ink-correctly-with-complex-descenders__1.json");
}

#[test]
fn text_decoration_should_work_correctly_when_text_decoration_line_line_through_and_text_align_right() {
    run_fixture("text-decoration__text-decoration-should-work-correctly-when-text-decoration-line-line-through-and-text-align-right__1.json");
}

#[test]
fn text_decoration_should_work_correctly_when_text_decoration_line_underline_and_text_align_right() {
    run_fixture("text-decoration__text-decoration-should-work-correctly-when-text-decoration-line-underline-and-text-align-right__1.json");
}

#[test]
fn text_decoration_should_work_correctly_when_text_decoration_style_dashed() {
    run_fixture("text-decoration__text-decoration-should-work-correctly-when-text-decoration-style-dashed__1.json");
}

#[test]
fn text_decoration_should_work_correctly_when_text_decoration_style_dotted() {
    run_fixture("text-decoration__text-decoration-should-work-correctly-when-text-decoration-style-dotted__1.json");
}

#[test]
fn text_decoration_should_work_correctly_when_text_decoration_style_double() {
    run_fixture("text-decoration__text-decoration-should-work-correctly-when-text-decoration-style-double__1.json");
}

#[test]
fn text_decoration_should_work_correctly_with_text_decoration_and_transform() {
    run_fixture("text-decoration__text-decoration-should-work-correctly-with-text-decoration-and-transform__1.json");
}
