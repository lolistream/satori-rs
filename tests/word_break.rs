//! Snapshot tests sourced from `fixtures/word-break__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn word_break_break_all_should_always_break_words_eagerly() {
    run_fixture("word-break__word-break-break-all-should-always-break-words-eagerly__1.json");
}

#[test]
fn word_break_break_word_should_break_words_if_cannot_fit_into_one_line() {
    run_fixture("word-break__word-break-break-word-should-break-words-if-cannot-fit-into-one-line__1.json");
}

#[test]
fn word_break_break_word_should_not_break_cjk_with_word_break_keep_all() {
    run_fixture("word-break__word-break-break-word-should-not-break-cjk-with-word-break-keep-all__1.json");
}

#[test]
fn word_break_break_word_should_try_to_wrap_words_if_possible() {
    run_fixture("word-break__word-break-break-word-should-try-to-wrap-words-if-possible__1.json");
}

#[test]
fn word_break_break_word_should_wrap_first_and_then_break_long_words() {
    run_fixture("word-break__word-break-break-word-should-wrap-first-and-then-break-long-words__1.json");
}

#[test]
fn word_break_normal_should_not_break_long_word() {
    run_fixture("word-break__word-break-normal-should-not-break-long-word__1.json");
}

#[test]
fn word_break_normal_should_not_break_word_if_possible_to_wrap() {
    run_fixture("word-break__word-break-normal-should-not-break-word-if-possible-to-wrap__1.json");
}

#[test]
fn word_break_should_support_non_breaking_space() {
    run_fixture("word-break__word-break-should-support-non-breaking-space__1.json");
}
