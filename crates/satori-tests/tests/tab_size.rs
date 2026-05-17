//! Snapshot tests sourced from `crates/satori-tests/fixtures/tab-size__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn tab_size_tab_renders_as_space_when_white_space_is_not_pre_or_pre_wrap() {
    run_fixture("tab-size__tab-size-tab-renders-as-space-when-white-space-is-not-pre-or-pre-wrap__1.json");
}

#[test]
fn tab_size_tabs_render_correctly_when_tab_size_is_a_number() {
    run_fixture("tab-size__tab-size-tabs-render-correctly-when-tab-size-is-a-number__1.json");
}

#[test]
fn tab_size_tabs_render_correctly_when_tab_size_is_a_string() {
    run_fixture("tab-size__tab-size-tabs-render-correctly-when-tab-size-is-a-string__1.json");
}

#[test]
fn tab_size_tabs_render_correctly_with_default_tab_size_of_8_when_white_space_is_pre_wrap() {
    run_fixture("tab-size__tab-size-tabs-render-correctly-with-default-tab-size-of-8-when-white-space-is-pre-wrap__1.json");
}

#[test]
fn tab_size_tabs_render_correctly_with_default_tab_size_of_8_when_white_space_is_pre() {
    run_fixture("tab-size__tab-size-tabs-render-correctly-with-default-tab-size-of-8-when-white-space-is-pre__1.json");
}
