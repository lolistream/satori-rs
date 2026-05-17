//! Snapshot tests sourced from `crates/satori-tests/fixtures/clip-path__*.json`.

use satori_tests::harness::run_fixture;

#[test]
fn clip_path_should_make_clip_path_compatible_with_overflow() {
    run_fixture("clip-path__clip-path-should-make-clip-path-compatible-with-overflow__1.json");
}

#[test]
fn clip_path_should_render_clip_path() {
    run_fixture("clip-path__clip-path-should-render-clip-path__1.json");
}

#[test]
fn clip_path_should_render_clip_path_call2() {
    run_fixture("clip-path__clip-path-should-render-clip-path__2.json");
}

#[test]
fn clip_path_should_render_clip_path_call3() {
    run_fixture("clip-path__clip-path-should-render-clip-path__3.json");
}

#[test]
fn clip_path_should_render_clip_path_call4() {
    run_fixture("clip-path__clip-path-should-render-clip-path__4.json");
}

#[test]
fn clip_path_should_render_clip_path_call5() {
    run_fixture("clip-path__clip-path-should-render-clip-path__5.json");
}

#[test]
fn clip_path_should_render_clip_path_call6() {
    run_fixture("clip-path__clip-path-should-render-clip-path__6.json");
}

#[test]
fn clip_path_should_render_clip_path_call7() {
    run_fixture("clip-path__clip-path-should-render-clip-path__7.json");
}

#[test]
fn clip_path_should_respect_left_and_top() {
    run_fixture("clip-path__clip-path-should-respect-left-and-top__1.json");
}

#[test]
fn clip_path_should_respect_the_position_value() {
    run_fixture("clip-path__clip-path-should-respect-the-position-value__1.json");
}
