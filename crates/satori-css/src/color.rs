//! CSS color parsing. JS satori uses `parse-css-color` + `css-background-parser`.
//! We implement just enough to cover the common test cases: named colors,
//! hex (#rgb, #rgba, #rrggbb, #rrggbbaa), `rgb()`, `rgba()`, `hsl()`,
//! `hsla()`, and the keyword `transparent`/`currentColor`.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const TRANSPARENT: Rgba = Rgba { r: 0, g: 0, b: 0, a: 0 };
    pub const BLACK: Rgba = Rgba { r: 0, g: 0, b: 0, a: 0xff };
    pub const WHITE: Rgba = Rgba { r: 0xff, g: 0xff, b: 0xff, a: 0xff };

    /// CSS hex form, matching JS satori output style.
    /// Uses `rgba(r, g, b, a)` if alpha != 1, else `#rrggbb`.
    pub fn to_css(self) -> String {
        if self.a == 0xff {
            format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
        } else {
            let a = self.a as f32 / 255.0;
            format!("rgba({},{},{},{})", self.r, self.g, self.b, trim_float(a))
        }
    }
}

fn trim_float(f: f32) -> String {
    let s = format!("{:.6}", f);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s.is_empty() { "0".into() } else { s.into() }
}

/// Parse a CSS color string. Returns None for unrecognized.
pub fn parse_color(input: &str) -> Option<Rgba> {
    let s = input.trim().to_ascii_lowercase();
    if s == "transparent" {
        return Some(Rgba::TRANSPARENT);
    }
    if s == "currentcolor" {
        // Caller decides what currentColor means; treated as black here.
        return Some(Rgba::BLACK);
    }
    if let Some(stripped) = s.strip_prefix('#') {
        return parse_hex(stripped);
    }
    if s.starts_with("rgb") {
        return parse_rgb_func(&s);
    }
    if s.starts_with("hsl") {
        return parse_hsl_func(&s);
    }
    NAMED_COLORS
        .iter()
        .find_map(|(name, rgb)| if *name == s { Some(*rgb) } else { None })
}

fn parse_hex(s: &str) -> Option<Rgba> {
    let to_u8 = |c: u8| -> Option<u8> {
        match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        }
    };
    let bytes = s.as_bytes();
    match bytes.len() {
        3 => Some(Rgba {
            r: to_u8(bytes[0])? * 17,
            g: to_u8(bytes[1])? * 17,
            b: to_u8(bytes[2])? * 17,
            a: 0xff,
        }),
        4 => Some(Rgba {
            r: to_u8(bytes[0])? * 17,
            g: to_u8(bytes[1])? * 17,
            b: to_u8(bytes[2])? * 17,
            a: to_u8(bytes[3])? * 17,
        }),
        6 => Some(Rgba {
            r: to_u8(bytes[0])? * 16 + to_u8(bytes[1])?,
            g: to_u8(bytes[2])? * 16 + to_u8(bytes[3])?,
            b: to_u8(bytes[4])? * 16 + to_u8(bytes[5])?,
            a: 0xff,
        }),
        8 => Some(Rgba {
            r: to_u8(bytes[0])? * 16 + to_u8(bytes[1])?,
            g: to_u8(bytes[2])? * 16 + to_u8(bytes[3])?,
            b: to_u8(bytes[4])? * 16 + to_u8(bytes[5])?,
            a: to_u8(bytes[6])? * 16 + to_u8(bytes[7])?,
        }),
        _ => None,
    }
}

fn parse_rgb_func(s: &str) -> Option<Rgba> {
    let inner = s.strip_prefix("rgb")?.trim_start_matches('a').trim();
    let inner = inner.strip_prefix('(')?.strip_suffix(')')?;
    let parts: Vec<&str> = inner.split([',', '/', ' '])
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() < 3 { return None; }
    let r = parse_byte_channel(parts[0])?;
    let g = parse_byte_channel(parts[1])?;
    let b = parse_byte_channel(parts[2])?;
    let a = if parts.len() >= 4 { parse_alpha(parts[3])? } else { 0xff };
    Some(Rgba { r, g, b, a })
}

fn parse_byte_channel(s: &str) -> Option<u8> {
    let s = s.trim();
    if let Some(p) = s.strip_suffix('%') {
        let v: f32 = p.parse().ok()?;
        Some((v / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let v: f32 = s.parse().ok()?;
        Some(v.round().clamp(0.0, 255.0) as u8)
    }
}

fn parse_alpha(s: &str) -> Option<u8> {
    let s = s.trim();
    if let Some(p) = s.strip_suffix('%') {
        let v: f32 = p.parse().ok()?;
        Some((v / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8)
    } else {
        let v: f32 = s.parse().ok()?;
        Some((v * 255.0).round().clamp(0.0, 255.0) as u8)
    }
}

fn parse_hsl_func(s: &str) -> Option<Rgba> {
    let inner = s.strip_prefix("hsl")?.trim_start_matches('a').trim();
    let inner = inner.strip_prefix('(')?.strip_suffix(')')?;
    let parts: Vec<&str> = inner.split([',', '/', ' '])
        .filter(|s| !s.is_empty())
        .collect();
    if parts.len() < 3 { return None; }
    let h: f32 = parts[0].trim_end_matches("deg").parse().ok()?;
    let s_pct: f32 = parts[1].trim_end_matches('%').parse().ok()?;
    let l_pct: f32 = parts[2].trim_end_matches('%').parse().ok()?;
    let a = if parts.len() >= 4 { parse_alpha(parts[3])? } else { 0xff };
    let (r, g, b) = hsl_to_rgb(h, s_pct / 100.0, l_pct / 100.0);
    Some(Rgba {
        r: (r * 255.0).round().clamp(0.0, 255.0) as u8,
        g: (g * 255.0).round().clamp(0.0, 255.0) as u8,
        b: (b * 255.0).round().clamp(0.0, 255.0) as u8,
        a,
    })
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h_prime = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - (h_prime.rem_euclid(2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    (r1 + m, g1 + m, b1 + m)
}

// A subset of CSS named colors. Add more on demand.
const NAMED_COLORS: &[(&str, Rgba)] = &[
    ("black", Rgba { r: 0, g: 0, b: 0, a: 0xff }),
    ("white", Rgba { r: 0xff, g: 0xff, b: 0xff, a: 0xff }),
    ("red", Rgba { r: 0xff, g: 0, b: 0, a: 0xff }),
    ("green", Rgba { r: 0, g: 0x80, b: 0, a: 0xff }),
    ("blue", Rgba { r: 0, g: 0, b: 0xff, a: 0xff }),
    ("yellow", Rgba { r: 0xff, g: 0xff, b: 0, a: 0xff }),
    ("cyan", Rgba { r: 0, g: 0xff, b: 0xff, a: 0xff }),
    ("magenta", Rgba { r: 0xff, g: 0, b: 0xff, a: 0xff }),
    ("gray", Rgba { r: 0x80, g: 0x80, b: 0x80, a: 0xff }),
    ("grey", Rgba { r: 0x80, g: 0x80, b: 0x80, a: 0xff }),
    ("orange", Rgba { r: 0xff, g: 0xa5, b: 0, a: 0xff }),
    ("purple", Rgba { r: 0x80, g: 0, b: 0x80, a: 0xff }),
    ("pink", Rgba { r: 0xff, g: 0xc0, b: 0xcb, a: 0xff }),
    ("brown", Rgba { r: 0xa5, g: 0x2a, b: 0x2a, a: 0xff }),
    ("silver", Rgba { r: 0xc0, g: 0xc0, b: 0xc0, a: 0xff }),
    ("gold", Rgba { r: 0xff, g: 0xd7, b: 0, a: 0xff }),
    ("lime", Rgba { r: 0, g: 0xff, b: 0, a: 0xff }),
    ("navy", Rgba { r: 0, g: 0, b: 0x80, a: 0xff }),
    ("teal", Rgba { r: 0, g: 0x80, b: 0x80, a: 0xff }),
    ("maroon", Rgba { r: 0x80, g: 0, b: 0, a: 0xff }),
    ("olive", Rgba { r: 0x80, g: 0x80, b: 0, a: 0xff }),
    ("aqua", Rgba { r: 0, g: 0xff, b: 0xff, a: 0xff }),
    ("fuchsia", Rgba { r: 0xff, g: 0, b: 0xff, a: 0xff }),
];
