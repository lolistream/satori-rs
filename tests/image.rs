//! Snapshot tests sourced from `fixtures/image__*.json`.

mod common;
use common::harness::{run_fixture, run_assertions_fixture};

#[test]
fn background_image_url_should_correctly_position_the_background_pattern() {
    run_fixture("image__background-image-url-should-correctly-position-the-background-pattern__1.json");
}

#[test]
fn background_image_url_should_handle_charset_utf_8_with_comma_in_data() {
    run_fixture("image__background-image-url-should-handle-charset-utf-8-with-comma-in-data__1.json");
}

#[test]
fn background_image_url_should_handle_charset_utf_8_with_in_base64_assertions() {
    run_assertions_fixture("image__background-image-url-should-handle-charset-utf-8-with-in-base64__assertions.json");
}

#[test]
fn background_image_url_should_handle_charset_utf_8() {
    run_fixture("image__background-image-url-should-handle-charset-utf-8__1.json");
}

#[test]
fn background_image_url_should_resolve_data_uris_with_size_for_supported_image_formats() {
    run_fixture("image__background-image-url-should-resolve-data-uris-with-size-for-supported-image-formats__1.json");
}

#[test]
fn background_image_url_should_resolve_data_uris_with_size_for_supported_image_formats_call2() {
    run_fixture("image__background-image-url-should-resolve-data-uris-with-size-for-supported-image-formats__2.json");
}

#[test]
fn background_image_url_should_resolve_data_uris_with_size_for_supported_image_formats_call3() {
    run_fixture("image__background-image-url-should-resolve-data-uris-with-size-for-supported-image-formats__3.json");
}

#[test]
fn background_image_url_should_resolve_data_uris_with_size_for_supported_image_formats_call4() {
    run_fixture("image__background-image-url-should-resolve-data-uris-with-size-for-supported-image-formats__4.json");
}

#[test]
fn background_image_url_should_resolve_data_uris_with_size_for_supported_image_formats_call5() {
    run_fixture("image__background-image-url-should-resolve-data-uris-with-size-for-supported-image-formats__5.json");
}

#[test]
fn background_image_url_should_resolve_image_data() {
    run_fixture("image__background-image-url-should-resolve-image-data__1.json");
}

#[test]
fn background_image_url_should_support_background_size_auto() {
    run_fixture("image__background-image-url-should-support-background-size-auto__1.json");
}

#[test]
fn background_image_url_should_support_background_size_contain() {
    run_fixture("image__background-image-url-should-support-background-size-contain__1.json");
}

#[test]
fn background_image_url_should_support_background_size_cover_with_non_square_container() {
    run_fixture("image__background-image-url-should-support-background-size-cover-with-non-square-container__1.json");
}

#[test]
fn background_image_url_should_support_background_size_cover() {
    run_fixture("image__background-image-url-should-support-background-size-cover__1.json");
}

#[test]
fn background_image_url_should_support_double_quotes_inside_url() {
    run_fixture("image__background-image-url-should-support-double-quotes-inside-url__1.json");
}

#[test]
fn background_image_url_should_support_single_quotes_inside_url() {
    run_fixture("image__background-image-url-should-support-single-quotes-inside-url__1.json");
}

#[test]
fn background_image_url_should_support_stretched_background_size() {
    run_fixture("image__background-image-url-should-support-stretched-background-size__1.json");
}

#[test]
fn background_image_url_should_support_svg_data_uris_with_various_quotes_inside_url_assertions() {
    run_assertions_fixture("image__background-image-url-should-support-svg-data-uris-with-various-quotes-inside-url__assertions.json");
}

#[test]
fn image_should_clip_content_in_the_border_and_padding_areas() {
    run_fixture("image__image-should-clip-content-in-the-border-and-padding-areas__1.json");
}

#[test]
fn image_should_clip_content_in_the_border_area() {
    run_fixture("image__image-should-clip-content-in-the-border-area__1.json");
}

#[test]
fn image_should_deduplicate_image_data_requests() {
    run_fixture("image__image-should-deduplicate-image-data-requests__1.json");
}

#[test]
fn image_should_have_a_separate_border_radius_clip_path_when_transform_is_used() {
    run_fixture("image__image-should-have-a-separate-border-radius-clip-path-when-transform-is-used__1.json");
}

#[test]
fn image_should_not_throw_when_image_is_not_valid() {
    run_fixture("image__image-should-not-throw-when-image-is-not-valid__1.json");
}

#[test]
fn image_should_render_svg_with_image_using_xlink_href() {
    run_fixture("image__image-should-render-svg-with-image-using-xlink-href__1.json");
}

#[test]
fn image_should_render_svg_with_image() {
    run_fixture("image__image-should-render-svg-with-image__1.json");
}

#[test]
fn image_should_resolve_image_data() {
    run_fixture("image__image-should-resolve-image-data__1.json");
}

#[test]
fn image_should_resolve_non_square_image_size_correctly() {
    run_fixture("image__image-should-resolve-non-square-image-size-correctly__1.json");
}

#[test]
fn image_should_resolve_the_image_size_and_scale_automatically() {
    run_fixture("image__image-should-resolve-the-image-size-and-scale-automatically__1.json");
}

#[test]
fn image_should_scale_image_to_fit_max_width_and_max_height_but_maintain_the_aspect_ratio() {
    run_fixture("image__image-should-scale-image-to-fit-max-width-and-max-height-but-maintain-the-aspect-ratio__1.json");
}

#[test]
fn image_should_scale_image_to_fit_max_width_and_max_height_but_maintain_the_aspect_ratio_call2() {
    run_fixture("image__image-should-scale-image-to-fit-max-width-and-max-height-but-maintain-the-aspect-ratio__2.json");
}

#[test]
fn image_should_support_array_buffer_as_src() {
    run_fixture("image__image-should-support-array-buffer-as-src__1.json");
}

#[test]
fn image_should_support_opacity() {
    run_fixture("image__image-should-support-opacity__1.json");
}

#[test]
fn image_should_support_styles() {
    run_fixture("image__image-should-support-styles__1.json");
}

#[test]
fn image_should_support_svg_images_and_percentage_with_correct_aspect_ratio() {
    run_fixture("image__image-should-support-svg-images-and-percentage-with-correct-aspect-ratio__1.json");
}

#[test]
fn image_should_support_transparent_image_with_background() {
    run_fixture("image__image-should-support-transparent-image-with-background__1.json");
}

#[test]
fn image_should_throw_error_when_relative_path_is_used_assertions() {
    run_assertions_fixture("image__image-should-throw-error-when-relative-path-is-used__assertions.json");
}

#[test]
fn object_fit_and_object_position_object_fit_fill_should_stretch_image_to_fill_container_aspect_ratio_not_preserved() {
    run_fixture("image__object-fit-and-object-position-object-fit-fill-should-stretch-image-to-fill-container-aspect-ratio-not-preserved__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_fill_should_stretch_with_fill_on_non_square_container() {
    run_fixture("image__object-fit-and-object-position-object-fit-fill-should-stretch-with-fill-on-non-square-container__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_not_scale_up_when_image_is_smaller_than_container() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-not-scale-up-when-image-is-smaller-than-container__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_respect_object_position_bottom_right_with_scale_down() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-respect-object-position-bottom-right-with-scale-down__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_respect_object_position_with_scale_down() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-respect-object-position-with-scale-down__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_scale_down_when_image_is_larger_than_container() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-scale-down-when-image-is-larger-than-container__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_support_0_0_for_object_position_top_left() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-support-0-0-for-object-position-top-left__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_support_100_100_for_object_position_bottom_right() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-support-100-100-for-object-position-bottom-right__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_support_mixed_keyword_and_percentage_for_object_position() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-support-mixed-keyword-and-percentage-for-object-position__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_support_object_position_with_contain_and_percentages() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-support-object-position-with-contain-and-percentages__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_support_object_position_with_scale_down_and_percentages() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-support-object-position-with-scale-down-and-percentages__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_support_percentage_values_for_object_position() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-support-percentage-values-for-object-position__1.json");
}

#[test]
fn object_fit_and_object_position_object_fit_scale_down_should_support_pixel_values_for_object_position() {
    run_fixture("image__object-fit-and-object-position-object-fit-scale-down-should-support-pixel-values-for-object-position__1.json");
}

#[test]
fn object_fit_and_object_position_should_default_to_center_center_with_contain() {
    run_fixture("image__object-fit-and-object-position-should-default-to-center-center-with-contain__1.json");
}

#[test]
fn object_fit_and_object_position_should_default_to_center_center_with_cover() {
    run_fixture("image__object-fit-and-object-position-should-default-to-center-center-with-cover__1.json");
}

#[test]
fn object_fit_and_object_position_should_position_to_bottom_left_with_contain() {
    run_fixture("image__object-fit-and-object-position-should-position-to-bottom-left-with-contain__1.json");
}

#[test]
fn object_fit_and_object_position_should_position_to_bottom_right_with_cover() {
    run_fixture("image__object-fit-and-object-position-should-position-to-bottom-right-with-cover__1.json");
}

#[test]
fn object_fit_and_object_position_should_position_to_bottom_with_cover() {
    run_fixture("image__object-fit-and-object-position-should-position-to-bottom-with-cover__1.json");
}

#[test]
fn object_fit_and_object_position_should_position_to_left_with_cover() {
    run_fixture("image__object-fit-and-object-position-should-position-to-left-with-cover__1.json");
}

#[test]
fn object_fit_and_object_position_should_position_to_right_with_cover() {
    run_fixture("image__object-fit-and-object-position-should-position-to-right-with-cover__1.json");
}

#[test]
fn object_fit_and_object_position_should_position_to_top_left_with_cover() {
    run_fixture("image__object-fit-and-object-position-should-position-to-top-left-with-cover__1.json");
}

#[test]
fn object_fit_and_object_position_should_position_to_top_with_contain() {
    run_fixture("image__object-fit-and-object-position-should-position-to-top-with-contain__1.json");
}

#[test]
fn object_fit_and_object_position_should_position_to_top_with_cover() {
    run_fixture("image__object-fit-and-object-position-should-position-to-top-with-cover__1.json");
}
