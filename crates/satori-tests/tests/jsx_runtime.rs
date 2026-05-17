//! Snapshot tests sourced from `crates/satori-tests/fixtures/jsx-runtime__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn minimal_jsx_runtime_should_support_async_function_components() {
    run_fixture("jsx-runtime__minimal-jsx-runtime-should-support-async-function-components__1.json");
}

#[test]
fn minimal_jsx_runtime_should_support_async_function_components_call2() {
    run_fixture("jsx-runtime__minimal-jsx-runtime-should-support-async-function-components__2.json");
}

#[test]
fn minimal_jsx_runtime_should_support_fragment_elements() {
    run_fixture("jsx-runtime__minimal-jsx-runtime-should-support-fragment-elements__1.json");
}
