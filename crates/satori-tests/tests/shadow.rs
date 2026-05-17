//! Snapshot tests sourced from `crates/satori-tests/fixtures/shadow__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn shadow_box_shadow_should_be_affected_by_container_opacity() {
    run_fixture("shadow__shadow-box-shadow-should-be-affected-by-container-opacity__1.json");
}

#[test]
fn shadow_box_shadow_should_render_box_shadow_with_offset_and_spread() {
    run_fixture("shadow__shadow-box-shadow-should-render-box-shadow-with-offset-and-spread__1.json");
}

#[test]
fn shadow_box_shadow_should_render_box_shadow_with_offset() {
    run_fixture("shadow__shadow-box-shadow-should-render-box-shadow-with-offset__1.json");
}

#[test]
fn shadow_box_shadow_should_render_multiple_box_shadows() {
    run_fixture("shadow__shadow-box-shadow-should-render-multiple-box-shadows__1.json");
}

#[test]
fn shadow_box_shadow_should_render_regular_box_shadow() {
    run_fixture("shadow__shadow-box-shadow-should-render-regular-box-shadow__1.json");
}

#[test]
fn shadow_box_shadow_should_render_white_text_with_shadow_when_background_clip_is_text() {
    run_fixture("shadow__shadow-box-shadow-should-render-white-text-with-shadow-when-background-clip-is-text__1.json");
}

#[test]
fn shadow_box_shadow_should_show_box_shadow_without_specifying_height() {
    run_fixture("shadow__shadow-box-shadow-should-show-box-shadow-without-specifying-height__1.json");
}

#[test]
fn shadow_box_shadow_should_support_box_shadow_for_transparent_elements() {
    run_fixture("shadow__shadow-box-shadow-should-support-box-shadow-for-transparent-elements__1.json");
}

#[test]
fn shadow_box_shadow_should_support_box_shadow_spread_with_transparency() {
    run_fixture("shadow__shadow-box-shadow-should-support-box-shadow-spread-with-transparency__1.json");
}

#[test]
fn shadow_box_shadow_should_support_inset_box_shadows() {
    run_fixture("shadow__shadow-box-shadow-should-support-inset-box-shadows__1.json");
}

#[test]
fn shadow_box_shadow_should_support_multiple_text_shadows() {
    run_fixture("shadow__shadow-box-shadow-should-support-multiple-text-shadows__1.json");
}

#[test]
fn shadow_box_shadow_should_support_negative_spread() {
    run_fixture("shadow__shadow-box-shadow-should-support-negative-spread__1.json");
}

#[test]
fn shadow_box_shadow_should_support_text_shadows_if_exist_unexpected_comma() {
    run_fixture("shadow__shadow-box-shadow-should-support-text-shadows-if-exist-unexpected-comma__1.json");
}

#[test]
fn shadow_box_shadow_should_support_text_shadows_with_background_clip_and_no_background() {
    run_fixture("shadow__shadow-box-shadow-should-support-text-shadows-with-background-clip-and-no-background__1.json");
}

#[test]
fn shadow_box_shadow_should_support_text_shadows_with_transparent_text_color() {
    run_fixture("shadow__shadow-box-shadow-should-support-text-shadows-with-transparent-text-color__1.json");
}

#[test]
fn shadow_box_shadow_should_work_correct_with_zero_border_radius() {
    run_fixture("shadow__shadow-box-shadow-should-work-correct-with-zero-border-radius__1.json");
}
