//! Snapshot tests sourced from `crates/satori-tests/fixtures/text-wrap__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn text_wrap_should_wrap_balancedly_with_text_wrap_balance() {
    run_fixture("text-wrap__text-wrap-should-wrap-balancedly-with-text-wrap-balance__1.json");
}

#[test]
fn text_wrap_should_wrap_normally_with_text_wrap_wrap() {
    run_fixture("text-wrap__text-wrap-should-wrap-normally-with-text-wrap-wrap__1.json");
}
