//! Snapshot tests sourced from `fixtures/svg__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn svg_should_parse_view_box_correctly() {
    run_fixture("svg__svg-should-parse-view-box-correctly__1.json");
}

#[test]
fn svg_should_render_svg_attributes_correctly() {
    run_fixture("svg__svg-should-render-svg-attributes-correctly__1.json");
}

#[test]
fn svg_should_render_svg_nodes() {
    run_fixture("svg__svg-should-render-svg-nodes__1.json");
}

#[test]
fn svg_should_render_svg_prefer_size_props_rather_than_view_box() {
    run_fixture("svg__svg-should-render-svg-prefer-size-props-rather-than-view-box__1.json");
}

#[test]
fn svg_should_render_svg_size_correctly() {
    run_fixture("svg__svg-should-render-svg-size-correctly__1.json");
}

#[test]
fn svg_should_render_svg_without_view_box() {
    run_fixture("svg__svg-should-render-svg-without-view-box__1.json");
}

#[test]
fn svg_should_respect_style_on_svg_node() {
    run_fixture("svg__svg-should-respect-style-on-svg-node__1.json");
}

#[test]
fn svg_should_support_current_color_for_svg_fill() {
    run_fixture("svg__svg-should-support-current-color-for-svg-fill__1.json");
}

#[test]
fn svg_should_support_current_color_for_svg_stroke() {
    run_fixture("svg__svg-should-support-current-color-for-svg-stroke__1.json");
}

#[test]
fn svg_should_support_current_color_when_color_is_set_on_parent_element() {
    run_fixture("svg__svg-should-support-current-color-when-color-is-set-on-parent-element__1.json");
}

#[test]
fn svg_should_support_current_color_when_used_on_svg_nodes() {
    run_fixture("svg__svg-should-support-current-color-when-used-on-svg-nodes__1.json");
}

#[test]
fn svg_should_support_em_in_svg_size() {
    run_fixture("svg__svg-should-support-em-in-svg-size__1.json");
}
