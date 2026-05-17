//! Snapshot tests sourced from `crates/satori-tests/fixtures/display-contents__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn display_contents_should_apply_flex_properties_to_grandchildren_through_display_contents() {
    run_fixture("display-contents__display-contents-should-apply-flex-properties-to-grandchildren-through-display-contents__1.json");
}

#[test]
fn display_contents_should_ignore_padding_and_margin_on_display_contents_elements() {
    run_fixture("display-contents__display-contents-should-ignore-padding-and-margin-on-display-contents-elements__1.json");
}

#[test]
fn display_contents_should_render_display_contents() {
    run_fixture("display-contents__display-contents-should-render-display-contents__1.json");
}

#[test]
fn display_contents_should_treat_display_contents_children_as_direct_children_of_parent() {
    run_fixture("display-contents__display-contents-should-treat-display-contents-children-as-direct-children-of-parent__1.json");
}

#[test]
fn display_contents_should_work_with_nested_display_contents() {
    run_fixture("display-contents__display-contents-should-work-with-nested-display-contents__1.json");
}

#[test]
fn display_contents_should_work_with_text_children() {
    run_fixture("display-contents__display-contents-should-work-with-text-children__1.json");
}
