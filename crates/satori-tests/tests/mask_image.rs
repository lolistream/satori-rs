//! Snapshot tests sourced from `crates/satori-tests/fixtures/mask-image__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn mask_should_support_mask_image_on_img() {
    run_fixture("mask-image__mask-should-support-mask-image-on-img__1.json");
}

#[test]
fn mask_should_support_mask_image_on_positioned_elements() {
    run_fixture("mask-image__mask-should-support-mask-image-on-positioned-elements__1.json");
}

#[test]
fn mask_should_support_mask_image_on_text() {
    run_fixture("mask-image__mask-should-support-mask-image-on-text__1.json");
}

#[test]
fn mask_should_support_mask_image() {
    run_fixture("mask-image__mask-should-support-mask-image__1.json");
}

#[test]
fn mask_should_support_mask_image_call2() {
    run_fixture("mask-image__mask-should-support-mask-image__2.json");
}

#[test]
fn mask_should_support_mask_image_call3() {
    run_fixture("mask-image__mask-should-support-mask-image__3.json");
}

#[test]
fn mask_should_support_mask_position() {
    run_fixture("mask-image__mask-should-support-mask-position__1.json");
}

#[test]
fn mask_should_support_mask_repeat() {
    run_fixture("mask-image__mask-should-support-mask-repeat__1.json");
}

#[test]
fn mask_should_support_mask_size() {
    run_fixture("mask-image__mask-should-support-mask-size__1.json");
}

#[test]
fn mask_should_support_multiple_mask_image() {
    run_fixture("mask-image__mask-should-support-multiple-mask-image__1.json");
}
