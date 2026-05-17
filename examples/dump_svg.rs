//! Run one fixture and print the produced SVG to stdout.
//!
//! Usage: `cargo run --example dump_svg -- <fixture.json>`

#[path = "../tests/common/mod.rs"]
mod common;
use common::harness::{load_fixture, mock_image_urls_for_test, to_image};

fn main() {
    let arg = std::env::args().nth(1).expect("usage: dump_svg <fixture.json>");
    let mut fx = load_fixture(&arg);
    mock_image_urls_for_test(&mut fx.element);
    let opts = fx.to_satori_options();
    let svg = satori::satori_from_value(fx.element.clone(), opts).unwrap();
    println!("SVG:\n{}", svg);
    let png = to_image(&svg, fx.width);
    eprintln!("PNG: {} bytes (snapshot: {})", png.len(), fx.snapshot);
}
