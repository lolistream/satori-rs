//! Snapshot tests sourced from `crates/satori-tests/fixtures/tailwind__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn tailwind_handler_unit_all_default_shadow_presets_are_addressable_assertions() {
    run_assertions_fixture("tailwind__tailwind-handler-unit-all-default-shadow-presets-are-addressable__assertions.json");
}

#[test]
fn tailwind_handler_unit_create_tw_with_config_containing_plugins_assertions() {
    run_assertions_fixture("tailwind__tailwind-handler-unit-create-tw-with-config-containing-plugins__assertions.json");
}

#[test]
fn tailwind_handler_unit_create_tw_with_config_without_plugins_assertions() {
    run_assertions_fixture("tailwind__tailwind-handler-unit-create-tw-with-config-without-plugins__assertions.json");
}

#[test]
fn tailwind_handler_unit_create_tw_without_config_assertions() {
    run_assertions_fixture("tailwind__tailwind-handler-unit-create-tw-without-config__assertions.json");
}

#[test]
fn tailwind_via_satori_integration_combines_shadow_preset_and_shadow_color_covers_shadow_color_box_shadow_replace_branch_assertions() {
    run_assertions_fixture("tailwind__tailwind-via-satori-integration-combines-shadow-preset-and-shadow-color-covers-shadow-color-box-shadow-replace-branch__assertions.json");
}

#[test]
fn tailwind_via_satori_integration_renders_shadow_class_through_satori_to_exercise_shadow_color_branch_assertions() {
    run_assertions_fixture("tailwind__tailwind-via-satori-integration-renders-shadow-class-through-satori-to-exercise-shadow-color-branch__assertions.json");
}

#[test]
fn tailwind_via_satori_integration_renders_with_tailwind_config_provided_assertions() {
    run_assertions_fixture("tailwind__tailwind-via-satori-integration-renders-with-tailwind-config-provided__assertions.json");
}

#[test]
fn tailwind_via_satori_integration_renders_with_tw_class_names_and_shadow_preset_covers_get_tw_styles_assertions() {
    run_assertions_fixture("tailwind__tailwind-via-satori-integration-renders-with-tw-class-names-and-shadow-preset-covers-get-tw-styles__assertions.json");
}
