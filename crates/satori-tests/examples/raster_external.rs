//! Rasterize an arbitrary SVG file with our exact harness pipeline.
//!
//! Used to verify whether the residual N-pixel diff on byte-identical
//! SVG output is coming from our path emitter (in which case the
//! diff would disappear when we feed JS satori's SVG straight in) or
//! from the rasterizer version skew (in which case it would persist).

use satori_tests::harness::{assert_image_snapshot, to_image};

fn main() {
    let svg_path = std::env::args().nth(1).expect("usage: raster_external <svg> <width> [<snapshot>]");
    let width: u32 = std::env::args()
        .nth(2)
        .expect("width")
        .parse()
        .expect("parse width");
    let svg = std::fs::read_to_string(&svg_path).expect("read svg");
    let png = to_image(&svg, width);
    if let Some(snapshot) = std::env::args().nth(3) {
        // Use the regular harness assert (panics on mismatch).
        assert_image_snapshot(&png, &snapshot);
        eprintln!("snapshot matched: {snapshot}");
    } else {
        std::io::Write::write_all(&mut std::io::stdout(), &png).unwrap();
    }
}
