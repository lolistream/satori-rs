//! Snapshot tests sourced from `crates/satori-tests/fixtures/language__*.json`.

use satori_tests::harness::{run_fixture, run_assertions_fixture};

#[test]
fn detect_language_code_should_detect_arabic_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-arabic__assertions.json");
}

#[test]
fn detect_language_code_should_detect_bengali_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-bengali__assertions.json");
}

#[test]
fn detect_language_code_should_detect_devanagari_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-devanagari__assertions.json");
}

#[test]
fn detect_language_code_should_detect_emoji_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-emoji__assertions.json");
}

#[test]
fn detect_language_code_should_detect_hebrew_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-hebrew__assertions.json");
}

#[test]
fn detect_language_code_should_detect_japanese_hiragana_when_locale_is_zh_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-japanese-hiragana-when-locale-is-zh__assertions.json");
}

#[test]
fn detect_language_code_should_detect_japanese_hiragana_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-japanese-hiragana__assertions.json");
}

#[test]
fn detect_language_code_should_detect_japanese_kanji_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-japanese-kanji__assertions.json");
}

#[test]
fn detect_language_code_should_detect_japanese_katakana_when_locale_is_zh_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-japanese-katakana-when-locale-is-zh__assertions.json");
}

#[test]
fn detect_language_code_should_detect_japanese_katakana_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-japanese-katakana__assertions.json");
}

#[test]
fn detect_language_code_should_detect_korean_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-korean__assertions.json");
}

#[test]
fn detect_language_code_should_detect_malayalam_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-malayalam__assertions.json");
}

#[test]
fn detect_language_code_should_detect_math_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-math__assertions.json");
}

#[test]
fn detect_language_code_should_detect_simplified_chinese_when_locale_is_zh_cn_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-simplified-chinese-when-locale-is-zh-cn__assertions.json");
}

#[test]
fn detect_language_code_should_detect_symbol_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-symbol__assertions.json");
}

#[test]
fn detect_language_code_should_detect_tamil_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-tamil__assertions.json");
}

#[test]
fn detect_language_code_should_detect_telegu_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-telegu__assertions.json");
}

#[test]
fn detect_language_code_should_detect_thai_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-thai__assertions.json");
}

#[test]
fn detect_language_code_should_detect_traditional_chinese_hk_when_locale_is_zh_cn_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-traditional-chinese-hk-when-locale-is-zh-cn__assertions.json");
}

#[test]
fn detect_language_code_should_detect_traditional_chinese_tw_when_locale_is_zh_tw_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-traditional-chinese-tw-when-locale-is-zh-tw__assertions.json");
}

#[test]
fn detect_language_code_should_detect_unknown_assertions() {
    run_assertions_fixture("language__detect-language-code-should-detect-unknown__assertions.json");
}

#[test]
fn detect_language_code_should_not_crash_when_rendering_arabic_letters() {
    run_fixture("language__detect-language-code-should-not-crash-when-rendering-arabic-letters__1.json");
}
