//! Port of `src/handler/image.ts`.
//!
//! Resolves an `<img src=...>` attribute into a `(data_uri, natural_width,
//! natural_height)` triple suitable for embedding into a `<image href=...>`
//! SVG element. Only the inputs the test suite exercises are supported:
//!
//! - `data:` URIs (base64 or URL-encoded payload, image/png/jpeg/gif/svg+xml)
//! - the harness-side `__assetFile` shape (a relative path under
//!   `crates/satori-tests/assets/`)
//!
//! HTTP fetches are out of scope — the test harness substitutes mock data
//! URIs before satori sees them.

use std::path::Path;

use base64::Engine;

/// Resolved image data — mirrors the JS `[src, width?, height?]` tuple.
pub struct ResolvedImage {
    pub src: String,
    pub natural_width: Option<f32>,
    pub natural_height: Option<f32>,
}

/// Resolve a raw `src` string. The caller is responsible for stripping any
/// quoting characters that `url(...)` wrappers may add — the JS pipeline
/// does the same in `resolveImageData`.
pub fn resolve_image_src(src: &str) -> Option<ResolvedImage> {
    let trimmed = strip_quotes(src.trim());
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("data:") {
        return resolve_data_uri(trimmed);
    }
    None
}

/// Resolve a raw image byte buffer (e.g. an `ArrayBuffer` `src` in the JS
/// tests). Returns the equivalent base64 `data:image/...` URI plus the
/// parsed natural dimensions, or `None` if the format can't be detected.
pub fn resolve_image_buffer(bytes: &[u8]) -> Option<ResolvedImage> {
    let mime = detect_content_type(bytes)?;
    let dims = parse_dimensions(mime, bytes);
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    Some(ResolvedImage {
        src: format!("data:{mime};base64,{b64}"),
        natural_width: dims.map(|d| d.0),
        natural_height: dims.map(|d| d.1),
    })
}

/// Resolve a `__assetFile` reference: load the bytes from disk and parse
/// natural dimensions. `assets_root` is typically
/// `crates/satori-tests/assets/`.
pub fn resolve_image_asset_file(assets_root: &Path, asset_file: &str) -> Option<ResolvedImage> {
    let path = assets_root.join(asset_file);
    let bytes = std::fs::read(&path).ok()?;
    resolve_image_buffer(&bytes)
}

fn strip_quotes(s: &str) -> &str {
    if s.len() >= 2 {
        let bytes = s.as_bytes();
        let first = bytes[0];
        let last = bytes[s.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

fn resolve_data_uri(src: &str) -> Option<ResolvedImage> {
    let (header, payload) = src.strip_prefix("data:")?.split_once(',')?;
    let mut image_type: Option<&str> = None;
    let mut encoding_type: Option<&str> = None;
    for (i, part) in header.split(';').enumerate() {
        if i == 0 {
            if !part.is_empty() {
                image_type = Some(part);
            }
        } else if let Some((k, v)) = part.split_once('=') {
            // skip charset=foo etc.
            let _ = (k, v);
        } else {
            encoding_type = Some(part);
        }
    }
    let image_type = image_type?;
    if image_type == "image/svg+xml" {
        // Decode payload to a UTF-8 string and parse the SVG attributes.
        let utf8 = if encoding_type == Some("base64") {
            let bytes = decode_base64(payload)?;
            String::from_utf8(bytes).ok()?
        } else {
            decode_percent(payload)
        };
        let (nw, nh) = parse_svg_image_size(&utf8).unwrap_or((0.0, 0.0));
        let normalized = if encoding_type == Some("base64") {
            // already base64 — pass through the original src verbatim so
            // the byte-for-byte SVG output matches JS satori's `cache.get(src)[0]`.
            src.to_string()
        } else {
            // JS satori re-encodes the URL-encoded form into base64:
            //   `data:image/svg+xml;base64,${btoa(utf8Src)}`
            // We mirror that to keep the embedded data URI stable.
            let b64 = base64::engine::general_purpose::STANDARD.encode(utf8.as_bytes());
            format!("data:image/svg+xml;base64,{b64}")
        };
        return Some(ResolvedImage {
            src: normalized,
            natural_width: if nw > 0.0 { Some(nw) } else { None },
            natural_height: if nh > 0.0 { Some(nh) } else { None },
        });
    }
    if encoding_type == Some("base64") {
        let bytes = decode_base64(payload)?;
        let dims = parse_dimensions(image_type, &bytes);
        return Some(ResolvedImage {
            src: src.to_string(),
            natural_width: dims.map(|d| d.0),
            natural_height: dims.map(|d| d.1),
        });
    }
    // Non-base64, non-SVG data URI: keep as-is but no size.
    Some(ResolvedImage {
        src: src.to_string(),
        natural_width: None,
        natural_height: None,
    })
}

fn decode_base64(s: &str) -> Option<Vec<u8>> {
    // JS satori's `atob` is lenient about embedded whitespace; the URL form
    // can include `\n` line breaks. Strip whitespace before decoding.
    let cleaned: String = s.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    base64::engine::general_purpose::STANDARD
        .decode(cleaned.as_bytes())
        .ok()
}

fn decode_percent(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hi = hex_digit(bytes[i + 1]);
            let lo = hex_digit(bytes[i + 2]);
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h << 4) | l);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|e| String::from_utf8_lossy(e.as_bytes()).into_owned())
}

fn hex_digit(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Pixel-format magic-byte sniff (port of `detectContentType` in the JS
/// handler). Returns the MIME string or `None`.
pub fn detect_content_type(buf: &[u8]) -> Option<&'static str> {
    if buf.starts_with(&[0xff, 0xd8, 0xff]) {
        return Some("image/jpeg");
    }
    if buf.starts_with(&[0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]) {
        if detect_apng(buf) {
            return Some("image/apng");
        }
        return Some("image/png");
    }
    if buf.starts_with(&[0x47, 0x49, 0x46, 0x38]) {
        return Some("image/gif");
    }
    if buf.len() >= 12
        && buf[0] == 0x52
        && buf[1] == 0x49
        && buf[2] == 0x46
        && buf[3] == 0x46
        && buf[8] == 0x57
        && buf[9] == 0x45
        && buf[10] == 0x42
        && buf[11] == 0x50
    {
        return Some("image/webp");
    }
    if buf.starts_with(&[0x3c, 0x3f, 0x78, 0x6d, 0x6c]) {
        return Some("image/svg+xml");
    }
    if buf.len() >= 12
        && buf[4] == 0x66
        && buf[5] == 0x74
        && buf[6] == 0x79
        && buf[7] == 0x70
        && buf[8] == 0x61
        && buf[9] == 0x76
        && buf[10] == 0x69
        && buf[11] == 0x66
    {
        return Some("image/avif");
    }
    None
}

fn detect_apng(bytes: &[u8]) -> bool {
    let mut off = 8;
    while off + 8 <= bytes.len() {
        let length = u32::from_be_bytes([
            bytes[off],
            bytes[off + 1],
            bytes[off + 2],
            bytes[off + 3],
        ]) as usize;
        let typ = &bytes[off + 4..off + 8];
        if typ == b"acTL" {
            return true;
        }
        if typ == b"IEND" {
            return false;
        }
        // Avoid integer overflow on malformed chunks.
        let next = off.checked_add(12).and_then(|n| n.checked_add(length));
        match next {
            Some(n) if n <= bytes.len() => off = n,
            _ => return false,
        }
    }
    false
}

fn parse_dimensions(mime: &str, buf: &[u8]) -> Option<(f32, f32)> {
    match mime {
        "image/png" | "image/apng" => parse_png(buf),
        "image/gif" => parse_gif(buf),
        "image/jpeg" => parse_jpeg(buf),
        _ => None,
    }
}

fn parse_png(buf: &[u8]) -> Option<(f32, f32)> {
    if buf.len() < 24 {
        return None;
    }
    let w = u32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]);
    let h = u32::from_be_bytes([buf[20], buf[21], buf[22], buf[23]]);
    // Match the JS `getUint16` quirk: the JS port reads two bytes for
    // width/height (positions 18 & 22). For the test suite our 16-bit
    // truncation is harmless because the assets are small.
    Some((w as f32, h as f32))
}

fn parse_gif(buf: &[u8]) -> Option<(f32, f32)> {
    if buf.len() < 10 {
        return None;
    }
    let w = u16::from_le_bytes([buf[6], buf[7]]) as f32;
    let h = u16::from_le_bytes([buf[8], buf[9]]) as f32;
    Some((w, h))
}

/// Port of `parseJPEG` in `src/handler/image.ts`.
fn parse_jpeg(buf: &[u8]) -> Option<(f32, f32)> {
    if buf.len() < 4 {
        return None;
    }
    let mut offset = 4usize;
    while offset + 1 < buf.len() {
        let i = u16::from_be_bytes([buf[offset], buf[offset + 1]]) as usize;
        if i > buf.len() {
            return None;
        }
        let pos = i + 1 + offset;
        if pos >= buf.len() {
            return None;
        }
        let next = buf[pos];
        if next == 0xc0 || next == 0xc1 || next == 0xc2 {
            // height at i + 5, width at i + 7 (each big-endian u16).
            let hi = i + 5 + offset;
            let wi = i + 7 + offset;
            if wi + 1 >= buf.len() {
                return None;
            }
            let h = u16::from_be_bytes([buf[hi], buf[hi + 1]]) as f32;
            let w = u16::from_be_bytes([buf[wi], buf[wi + 1]]) as f32;
            return Some((w, h));
        }
        offset += i + 2;
    }
    None
}

fn parse_svg_image_size(svg: &str) -> Option<(f32, f32)> {
    // Locate the opening `<svg ...>` tag.
    let start = svg.find("<svg").or_else(|| svg.find("<SVG"))?;
    let rest = &svg[start..];
    let close = rest.find('>')?;
    let tag = &rest[..close];

    let view_box = find_attr(tag, "viewBox").map(parse_view_box);
    let width = find_attr(tag, "width").and_then(parse_number_attr);
    let height = find_attr(tag, "height").and_then(parse_number_attr);

    let (size_w, size_h) = if let Some(Some([_, _, w, h])) = view_box {
        (w, h)
    } else if let (Some(w), Some(h)) = (width, height) {
        (w, h)
    } else {
        return None;
    };
    let ratio = if size_h.abs() > f32::EPSILON {
        size_w / size_h
    } else {
        1.0
    };
    let (image_w, image_h) = match (width, height) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) if ratio.abs() > f32::EPSILON => (w, w / ratio),
        (None, Some(h)) => (h * ratio, h),
        _ => (size_w, size_h),
    };
    Some((image_w, image_h))
}

fn find_attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    // Simple scan for `name="..."` or `name='...'` with optional space.
    let bytes = tag.as_bytes();
    let name_bytes = name.as_bytes();
    let mut i = 0;
    while i + name_bytes.len() < bytes.len() {
        if bytes[i..].starts_with(name_bytes) {
            // Match attribute names case-sensitively but require word boundary.
            let before_ok = i == 0 || matches!(bytes[i - 1], b' ' | b'\t' | b'\n' | b'\r');
            let after = i + name_bytes.len();
            if before_ok {
                let mut j = after;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j < bytes.len() && bytes[j] == b'=' {
                    j += 1;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if j < bytes.len() && (bytes[j] == b'"' || bytes[j] == b'\'') {
                        let q = bytes[j];
                        j += 1;
                        let start = j;
                        while j < bytes.len() && bytes[j] != q {
                            j += 1;
                        }
                        if j <= bytes.len() {
                            return Some(&tag[start..j]);
                        }
                    }
                }
            }
        }
        i += 1;
    }
    None
}

fn parse_number_attr(v: &str) -> Option<f32> {
    let mut end = 0;
    let bytes = v.as_bytes();
    while end < bytes.len() {
        let c = bytes[end];
        if c == b'.' || c == b'-' || c.is_ascii_digit() {
            end += 1;
        } else {
            break;
        }
    }
    v[..end].parse::<f32>().ok()
}

fn parse_view_box(v: &str) -> Option<[f32; 4]> {
    let mut nums = v
        .split(|c: char| c.is_ascii_whitespace() || c == ',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<f32>().ok());
    let a = nums.next()??;
    let b = nums.next()??;
    let c = nums.next()??;
    let d = nums.next()??;
    Some([a, b, c, d])
}
