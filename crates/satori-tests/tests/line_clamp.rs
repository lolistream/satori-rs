//! Snapshot tests sourced from `crates/satori-tests/fixtures/line-clamp__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn line_clamp_should_not_work_when_display_is_not_set_to_block() {
    run_fixture("line-clamp__line-clamp-should-not-work-when-display-is-not-set-to-block__1.json");
}

#[test]
fn line_clamp_should_replace_custom_block_ellipsis_with_default_ellipsis_when_too_long() {
    run_fixture("line-clamp__line-clamp-should-replace-custom-block-ellipsis-with-default-ellipsis-when-too-long__1.json");
}

#[test]
fn line_clamp_should_work_correctly_when_text_align_center() {
    run_fixture("line-clamp__line-clamp-should-work-correctly-when-text-align-center__1.json");
}

#[test]
fn line_clamp_should_work_correctly() {
    run_fixture("line-clamp__line-clamp-should-work-correctly__1.json");
}
