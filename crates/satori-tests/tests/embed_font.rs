//! Snapshot tests sourced from `crates/satori-tests/fixtures/embed-font__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn embed_font_false_should_have_consistent_x_positions_for_multi_line_text_assertions() {
    run_assertions_fixture("embed-font__embed-font-false-should-have-consistent-x-positions-for-multi-line-text__assertions.json");
}
