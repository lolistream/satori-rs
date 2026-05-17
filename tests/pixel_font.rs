//! Snapshot tests sourced from `fixtures/pixel-font__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn pixel_font_alignment_should_align_pixel_and_hinted_fonts_with_pixel_boundaries() {
    run_fixture("pixel-font__pixel-font-alignment-should-align-pixel-and-hinted-fonts-with-pixel-boundaries__1.json");
}
