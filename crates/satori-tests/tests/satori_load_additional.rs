//! Snapshot tests sourced from `crates/satori-tests/fixtures/satori-load-additional__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn satori_load_additional_asset_variants_accepts_a_null_undefined_asset_covers_the_falsy_branch_assertions() {
    run_assertions_fixture("satori-load-additional__satori-load-additional-asset-variants-accepts-a-null-undefined-asset-covers-the-falsy-branch__assertions.json");
}

#[test]
fn satori_load_additional_asset_variants_accepts_a_single_font_options_object_as_the_asset_covers_fonts_push_asset_assertions() {
    run_assertions_fixture("satori-load-additional__satori-load-additional-asset-variants-accepts-a-single-font-options-object-as-the-asset-covers-fonts-push-asset__assertions.json");
}

#[test]
fn satori_load_additional_asset_variants_accepts_a_string_asset_image_assertions() {
    run_assertions_fixture("satori-load-additional__satori-load-additional-asset-variants-accepts-a-string-asset-image__assertions.json");
}

#[test]
fn satori_load_additional_asset_variants_accepts_an_array_of_font_options_as_the_asset_covers_fonts_push_asset_assertions() {
    run_assertions_fixture("satori-load-additional__satori-load-additional-asset-variants-accepts-an-array-of-font-options-as-the-asset-covers-fonts-push-asset__assertions.json");
}

#[test]
fn satori_load_additional_asset_variants_renders_with_point_scale_factor_option_assertions() {
    run_assertions_fixture("satori-load-additional__satori-load-additional-asset-variants-renders-with-point-scale-factor-option__assertions.json");
}

#[test]
fn satori_load_additional_asset_variants_renders_without_crashing_when_lang_is_set_covers_locale_get_lang_from_font_name_indirectly_assertions() {
    run_assertions_fixture("satori-load-additional__satori-load-additional-asset-variants-renders-without-crashing-when-lang-is-set-covers-locale-get-lang-from-font-name-indirectly__assertions.json");
}
