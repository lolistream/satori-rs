//! Snapshot tests sourced from `crates/satori-tests/fixtures/image-units__*.json`.

use satori_tests::harness::run_assertions_fixture;

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_detects_apng_and_returns_data_uri_with_size_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-detects-apng-and-returns-data-uri-with-size__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_detects_gif_and_returns_data_uri_with_size_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-detects-gif-and-returns-data-uri-with-size__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_detects_jpeg_and_returns_data_uri_with_size_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-detects-jpeg-and-returns-data-uri-with-size__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_detects_png_and_returns_data_uri_with_size_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-detects-png-and-returns-data-uri-with-size__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_hits_the_svg_signature_branch_in_detect_content_type_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-hits-the-svg-signature-branch-in-detect-content-type__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_throws_for_avif_buffer_not_in_allowed_image_types_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-throws-for-avif-buffer-not-in-allowed-image-types__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_throws_for_unknown_bytes_no_signature_match_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-throws-for-unknown-bytes-no-signature-match__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_throws_for_webp_buffer_not_in_allowed_image_types_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-throws-for-webp-buffer-not-in-allowed-image-types__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_throws_invalid_jpeg_when_a_chunk_length_exceeds_the_buffer_parse_jpeg_mid_loop_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-throws-invalid-jpeg-when-a-chunk-length-exceeds-the-buffer-parse-jpeg-mid-loop__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_throws_invalid_jpeg_when_no_sof_marker_is_found_parse_jpeg_terminal_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-throws-invalid-jpeg-when-no-sof-marker-is-found-parse-jpeg-terminal__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_throws_when_source_is_empty_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-throws-when-source-is-empty__assertions.json");
}

#[test]
fn image_ts_resolve_image_data_with_array_buffer_covers_detect_content_type_branches_throws_when_svg_data_uri_lacks_view_box_and_width_height_assertions() {
    run_assertions_fixture("image-units__image-ts-resolve-image-data-with-array-buffer-covers-detect-content-type-branches-throws-when-svg-data-uri-lacks-view-box-and-width-height__assertions.json");
}
