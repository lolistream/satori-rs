//! Snapshot tests sourced from `crates/satori-tests/fixtures/emoji__*.json`.

use satori_tests::harness::{run_fixture, run_assertions_fixture};

#[test]
fn emojis_should_detect_emojis_correctly_assertions() {
    run_assertions_fixture("emoji__emojis-should-detect-emojis-correctly__assertions.json");
}

#[test]
fn emojis_should_render_emojis_correctly_with_alphabetic_emoji() {
    run_fixture("emoji__emojis-should-render-emojis-correctly-with-alphabetic-emoji__1.json");
}

#[test]
fn emojis_should_render_emojis_correctly_with_word_break_break_all() {
    run_fixture("emoji__emojis-should-render-emojis-correctly-with-word-break-break-all__1.json");
}

#[test]
fn emojis_should_render_emojis_correctly() {
    run_fixture("emoji__emojis-should-render-emojis-correctly__1.json");
}
