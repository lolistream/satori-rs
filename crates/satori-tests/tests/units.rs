//! Snapshot tests sourced from `crates/satori-tests/fixtures/units__*.json`.

use satori_tests::harness::{run_fixture, run_assertions_fixture};

#[test]
fn units_should_support_em() {
    run_fixture("units__units-should-support-em__1.json");
}

#[test]
fn units_should_support_px_and_numbers() {
    run_fixture("units__units-should-support-px-and-numbers__1.json");
}

#[test]
fn units_should_support_rem() {
    run_fixture("units__units-should-support-rem__1.json");
}

#[test]
fn units_should_support_rgb_syntaxs() {
    run_fixture("units__units-should-support-rgb-syntaxs__1.json");
}

#[test]
fn units_should_support_split_multiple_effect_assertions() {
    run_assertions_fixture("units__units-should-support-split-multiple-effect__assertions.json");
}

#[test]
fn units_should_support_vh_and_vw() {
    run_fixture("units__units-should-support-vh-and-vw__1.json");
}

#[test]
fn units_should_support() {
    run_fixture("units__units-should-support__1.json");
}
