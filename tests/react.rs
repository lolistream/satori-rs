//! Snapshot tests sourced from `fixtures/react__*.json`.

mod common;
use common::harness::run_fixture;

#[test]
fn react_ap_is_should_support_forward_ref_wrapped_components() {
    run_fixture("react__react-ap-is-should-support-forward-ref-wrapped-components__1.json");
}
