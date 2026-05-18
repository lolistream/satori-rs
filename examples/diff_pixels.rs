//! Decode actual + expected PNG, print which pixel positions differ.
//!
//! Helpful while iterating on byte-perfect glyph emission: even one
//! pixel difference fails the snapshot test, and seeing the (x, y)
//! coordinate usually reveals whether the diff is at a glyph edge
//! (anti-aliasing artefact) or a structural mismatch (wrong shape).

#[path = "../tests/common/mod.rs"]
mod common;
use common::harness::{data_root, load_fixture, mock_image_urls_for_test, to_image};

fn main() {
    let arg = std::env::args().nth(1).expect("usage: diff_pixels <fixture.json>");
    let mut fx = load_fixture(&arg);
    mock_image_urls_for_test(&mut fx.element);
    let snapshot = fx.snapshot.clone();
    let width = fx.width;
    let opts = fx.to_satori_options();
    let svg = satori_rs::satori_from_value(fx.element.clone(), opts).unwrap();
    let actual_png = to_image(&svg, width);

    let snapshot_path = data_root().join("snapshots").join(&snapshot);
    let expected_png = std::fs::read(&snapshot_path).expect("read snapshot");

    let actual = decode(&actual_png).unwrap();
    let expected = decode(&expected_png).unwrap();

    eprintln!("actual:   {}x{}", actual.width, actual.height);
    eprintln!("expected: {}x{}", expected.width, expected.height);

    let mut count = 0usize;
    let pixel_count = (actual.width * actual.height) as usize;
    for i in 0..pixel_count {
        let a = &actual.rgba[i * 4..i * 4 + 4];
        let e = &expected.rgba[i * 4..i * 4 + 4];
        if a != e {
            let x = (i as u32) % actual.width;
            let y = (i as u32) / actual.width;
            if count < 20 {
                eprintln!(
                    "  ({},{}): actual={:?} expected={:?}",
                    x, y, a, e
                );
            }
            count += 1;
        }
    }
    eprintln!("differing pixels: {}/{}", count, pixel_count);
}

struct Decoded {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn decode(bytes: &[u8]) -> Result<Decoded, String> {
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    buf.truncate(info.buffer_size());
    let rgba = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => buf,
        (png::ColorType::Rgb, png::BitDepth::Eight) => buf
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 0xff])
            .collect(),
        _ => return Err(format!("unsupported png ({:?}, {:?})", info.color_type, info.bit_depth)),
    };
    Ok(Decoded {
        width: info.width,
        height: info.height,
        rgba,
    })
}
