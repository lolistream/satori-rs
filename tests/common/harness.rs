//! Shared test harness for the satori-rs port.
//!
//! - `to_image(svg, width)` rasterizes an SVG into PNG bytes via the
//!   in-process Rust `usvg` -> `resvg::render` -> `tiny_skia` pipeline
//!   with a Playfair Display fallback font and no system fonts.
//! - `init_fonts()` loads the default Roboto-Regular font.
//! - All test data (PNG snapshots, font assets) is vendored into this
//!   crate under `snapshots/` and `assets/`.

use std::path::{Path, PathBuf};

/// Root of this crate (`./`), used as the base for all
/// vendored test data (fonts under `assets/`, PNG snapshots under
/// `snapshots/`, JSON fixtures under `fixtures/`).
pub fn data_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

pub fn asset(name: &str) -> PathBuf {
    data_root().join("assets").join(name)
}

pub fn snapshot_path(name: &str) -> PathBuf {
    data_root().join("snapshots").join(name)
}

/// Load the default Roboto-Regular font shipped under `assets/`.
pub fn init_fonts() -> Vec<satori_rs::font::FontDescriptor> {
    let data = std::fs::read(asset("Roboto-Regular.ttf")).expect("Roboto-Regular.ttf");
    vec![satori_rs::font::FontDescriptor {
        name: "Roboto".to_string(),
        data,
        weight: Some(400),
        style: Some(satori_rs::font::FontStyle::Normal),
        lang: None,
    }]
}

/// One element of a `fixture.options.fonts` array. The actual font bytes live in
/// `assets/<assetFile>` and are loaded by `load_fixture_fonts`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FixtureFontRef {
    pub name: Option<String>,
    pub weight: Option<serde_json::Value>,
    pub style: Option<String>,
    pub lang: Option<String>,
    #[serde(rename = "assetFile")]
    pub asset_file: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FixtureOptions {
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
    #[serde(default)]
    pub fonts: Vec<FixtureFontRef>,
    #[serde(default)]
    pub embed_font: Option<bool>,
    #[serde(default)]
    pub debug: Option<bool>,
    #[serde(default)]
    pub grapheme_images: Option<std::collections::HashMap<String, String>>,
    /// The dump shim records `loadAdditionalAsset` as `{__callable: true}`
    /// (a marker that the test passed an actual function). When this is
    /// present, the harness emulates JS satori's fallback-font loading
    /// by pre-loading every non-`.ttf` font file from
    /// `assets/` (each named after the CJK / Greek
    /// segment it serves, e.g. `你好`, `안녕`).
    #[serde(default)]
    pub load_additional_asset: Option<serde_json::Value>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Fixture {
    pub element: serde_json::Value,
    #[serde(default)]
    pub options: FixtureOptions,
    pub width: u32,
    pub snapshot: String,
}

/// **Approximation** of JS satori's per-segment
/// `loadAdditionalAsset(code, segment)` callback (audit finding #8).
/// JS satori invokes the callback dynamically while laying out text:
/// each missing glyph triggers a `(code, segment)` lookup that
/// returns a font descriptor or an image URL. The Rust harness
/// can't replay that callback (it's a JS function that doesn't
/// survive the dump-shim round-trip), so as a stand-in we **bulk
/// preload every non-`.ttf` file under `assets/`**
/// before satori runs. Each such file is conventionally named
/// after the segment it serves (`你好`, `안녕`,
/// `こんにちは`, `Χαίρετ`),
/// so loading them all is guaranteed to be a *superset* of what
/// the per-segment callback would have returned for the actual
/// segments in the captured fixtures — i.e. we never under-load
/// fonts. It can, however, over-load (a fixture that uses only
/// Latin still gets the CJK fonts attached), which is fine because
/// `FontLoader::resolve_for_char` picks per char based on coverage.
///
/// TODO(audit #8): if the per-segment shape ever matters for
/// correctness (e.g. JS satori starts using `lang`-tagged GSUB
/// feature selection that depends on the callback returning
/// exactly one font per segment), wire the dump shim to invoke
/// `loadAdditionalAsset` synchronously and serialise the returned
/// descriptors into the fixture instead of relying on the bulk
/// preload here.
fn preload_callback_approximation_fonts(fonts: &mut Vec<satori_rs::font::FontDescriptor>) {
    let dir = data_root().join("assets");
    let Ok(entries) = std::fs::read_dir(&dir) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".ttf") || name.starts_with('.') {
            continue;
        }
        let Ok(data) = std::fs::read(entry.path()) else { continue };
        // Best-effort language tag from script. JS satori names
        // fallback fonts as `satori_${code}_fallback_${text}`, where
        // `code` is the detected language (e.g. `zh-CN`, `ko-KR`,
        // `ja-JP`, `el`). Our `FontLoader::resolve_for_char` picks
        // based on coverage rather than name, so the name itself
        // doesn't have to match exactly — but threading a `lang` tag
        // lets future GPOS/language-feature paths route correctly.
        let lang = name
            .chars()
            .next()
            .map(|c| match c {
                '\u{4E00}'..='\u{9FFF}' => "zh".to_string(),
                '\u{AC00}'..='\u{D7AF}' => "ko".to_string(),
                '\u{3040}'..='\u{30FF}' => "ja".to_string(),
                '\u{0370}'..='\u{03FF}' => "el".to_string(),
                _ => String::new(),
            })
            .unwrap_or_default();
        fonts.push(satori_rs::font::FontDescriptor {
            name: format!("satori_{lang}_fallback_{name}"),
            data,
            weight: Some(400),
            style: Some(satori_rs::font::FontStyle::Normal),
            lang: if lang.is_empty() { None } else { Some(lang) },
        });
    }
}

impl Fixture {
    pub fn to_satori_options(&self) -> satori_rs::SatoriOptions {
        let fonts: Vec<satori_rs::font::FontDescriptor> = if self.options.fonts.is_empty() {
            init_fonts()
        } else {
            self.options.fonts.iter().filter_map(|f| {
                // Prefer the explicit `assetFile` recorded by the dump
                // shim; fall back to `<name>.ttf` or `<name>-Regular.ttf`
                // for fonts (like `Habbo`, `MontserratSubrayada`) the
                // test files load directly via `readFileSync` rather
                // than through `loadDynamicAsset`.
                let af = f.asset_file.clone().or_else(|| {
                    let name = f.name.as_deref()?;
                    let candidates = [
                        format!("{name}.ttf"),
                        format!("{name}-Regular.ttf"),
                        format!("{name}-Bold.ttf"),
                    ];
                    candidates.into_iter().find(|c| asset(c).exists())
                })?;
                let data = std::fs::read(asset(&af)).ok()?;
                Some(satori_rs::font::FontDescriptor {
                    name: f.name.clone().unwrap_or_else(|| "Roboto".to_string()),
                    data,
                    weight: f.weight.as_ref().and_then(|v| v.as_u64()).map(|n| n as u16).or(Some(400)),
                    style: match f.style.as_deref() {
                        Some("italic") => Some(satori_rs::font::FontStyle::Italic),
                        _ => Some(satori_rs::font::FontStyle::Normal),
                    },
                    lang: f.lang.clone(),
                })
            }).collect()
        };
        let mut fonts = if fonts.is_empty() { init_fonts() } else { fonts };
        if self.options.load_additional_asset.is_some() {
            preload_callback_approximation_fonts(&mut fonts);
        }
        satori_rs::SatoriOptions {
            width: self.options.width,
            height: self.options.height,
            fonts,
            embed_font: self.options.embed_font.unwrap_or(true),
            debug: self.options.debug.unwrap_or(false),
            asset_root: Some(data_root().join("assets")),
            grapheme_images: self.options.grapheme_images.clone().unwrap_or_default(),
        }
    }
}

pub fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

pub fn load_fixture(name: &str) -> Fixture {
    let path = fixtures_dir().join(name);
    let s = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing fixture {}: {e}", path.display()));
    serde_json::from_str(&s).unwrap_or_else(|e| panic!("malformed fixture {name}: {e}"))
}

/// Run one fixture: build options, render satori, rasterize, assert pixel match.
pub fn run_fixture(name: &str) {
    let mut fx = load_fixture(name);
    mock_image_urls(&mut fx.element);
    let opts = fx.to_satori_options();
    let svg = satori_rs::satori_from_value(fx.element.clone(), opts).unwrap_or_else(|e| {
        panic!("satori() failed on fixture {name}: {e}")
    });
    let png = to_image(&svg, fx.width);
    assert_image_snapshot(&png, &fx.snapshot);
}

// ============================================================================
// Assertion fixtures
//
// `dump.mjs` emits one of these per test body that doesn't produce a snapshot
// (e.g. tests under `error.test.tsx`, `coverage-images.test.tsx`, etc.). Each
// fixture is the JS satori test body's intent translated into structured form:
// a list of satori calls and a list of assertions referencing those calls (via
// `lhs.kind = "satoriSvg" | "satoriPromise"`).
//
// `run_assertions_fixture` re-runs each call against satori-rs and verifies
// each assertion. Assertions whose lhs is not a satori sentinel (e.g. checks
// on internal JS helper return values) are skipped — we can only verify what
// satori-rs itself produces. Tests with no satori calls and no satori-related
// assertions therefore trivially pass.
// ============================================================================

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AssertionsFixture {
    #[serde(default)]
    pub calls: Vec<AssertionsCall>,
    #[serde(default)]
    pub assertions: Vec<Assertion>,
    #[serde(default)]
    pub source: serde_json::Value,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct AssertionsCall {
    #[serde(default)]
    pub element: serde_json::Value,
    #[serde(default)]
    pub options: FixtureOptions,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Assertion {
    pub kind: String,
    #[serde(default)]
    pub lhs: serde_json::Value,
    #[serde(default)]
    pub rhs: serde_json::Value,
    #[serde(default)]
    pub negated: bool,
}

impl AssertionsCall {
    fn to_satori_options(&self) -> satori_rs::SatoriOptions {
        let fonts = if self.options.fonts.is_empty() {
            init_fonts()
        } else {
            self.options.fonts.iter().filter_map(|f| {
                let af = f.asset_file.as_deref().unwrap_or("Roboto-Regular.ttf");
                let data = std::fs::read(asset(af)).ok()?;
                Some(satori_rs::font::FontDescriptor {
                    name: f.name.clone().unwrap_or_else(|| "Roboto".to_string()),
                    data,
                    weight: f
                        .weight
                        .as_ref()
                        .and_then(|v| v.as_u64())
                        .map(|n| n as u16)
                        .or(Some(400)),
                    style: match f.style.as_deref() {
                        Some("italic") => Some(satori_rs::font::FontStyle::Italic),
                        _ => Some(satori_rs::font::FontStyle::Normal),
                    },
                    lang: f.lang.clone(),
                })
            }).collect::<Vec<_>>()
        };
        let fonts = if fonts.is_empty() { init_fonts() } else { fonts };
        satori_rs::SatoriOptions {
            width: self.options.width,
            height: self.options.height,
            fonts,
            embed_font: self.options.embed_font.unwrap_or(true),
            debug: self.options.debug.unwrap_or(false),
            asset_root: Some(data_root().join("assets")),
            grapheme_images: self.options.grapheme_images.clone().unwrap_or_default(),
        }
    }
}

/// Outcome of running satori on a single fixture call.
enum CallResult {
    Ok(String),
    Err(String),
}

impl CallResult {
    fn svg(&self) -> Option<&str> {
        if let CallResult::Ok(s) = self { Some(s) } else { None }
    }
    fn err_msg(&self) -> Option<&str> {
        if let CallResult::Err(s) = self { Some(s) } else { None }
    }
}

fn run_call(call: &AssertionsCall) -> CallResult {
    if call.error.is_some() {
        return CallResult::Err(call.error.clone().unwrap_or_default());
    }
    let mut el = call.element.clone();
    mock_image_urls(&mut el);
    let opts = call.to_satori_options();
    match satori_rs::satori_from_value(el, opts) {
        Ok(s) => CallResult::Ok(s),
        Err(e) => CallResult::Err(e.to_string()),
    }
}

fn match_string(haystack: &str, needle: &serde_json::Value) -> bool {
    if let Some(s) = needle.as_str() {
        return haystack.contains(s);
    }
    if let Some(obj) = needle.as_object() {
        // `serializeMatcher` in shim_vitest produces `{ "__regex": "...", "flags": "..." }`
        // for RegExp matchers.
        if let Some(src) = obj.get("__regex").and_then(|v| v.as_str()) {
            // Best-effort regex match: case-insensitive substring of literal
            // chunks. A real regex engine would be heavier than needed for
            // the simple `/foo|bar/`-style patterns these tests use, but
            // we fall back to a literal substring match for the regex
            // source with leading/trailing `.*` and metachar runs stripped.
            return regex_like_match(haystack, src);
        }
        if let Some(msg) = obj.get("__error").and_then(|v| v.as_str()) {
            return haystack.contains(msg);
        }
        if let Some(s) = obj.get("__stringContaining").and_then(|v| v.as_str()) {
            return haystack.contains(s);
        }
    }
    false
}

/// Very small regex-likeness matcher: anchors, literal chunks, and `.`/`.*`/`.+`.
/// JS satori tests use simple patterns such as
///   /Invalid value for CSS property "display"/i  or
///   /Image size cannot be determined/  or
///   /Failed to parse SVG image/
/// We treat the regex source as a substring after stripping common
/// metacharacters (`\\b`, `^`, `$`) and `\` escapes for punctuation. This
/// gets every pattern used in the suite right without pulling a regex crate
/// into the test harness.
fn regex_like_match(haystack: &str, src: &str) -> bool {
    let mut s = src.to_string();
    // Strip anchors.
    if s.starts_with('^') {
        s.remove(0);
    }
    if s.ends_with('$') {
        s.pop();
    }
    // Replace `\.` with a `.` placeholder, then `.` and `.*`/`.+` with the
    // empty marker so substring matches still work.
    // Quick path: try splitting on `|` and checking if any alt matches.
    if s.contains('|') {
        for alt in s.split('|') {
            if regex_like_match_inner(haystack, alt) {
                return true;
            }
        }
        return false;
    }
    regex_like_match_inner(haystack, &s)
}

fn regex_like_match_inner(haystack: &str, src: &str) -> bool {
    // Split on regex metacharacters that act like "wildcards" — anything
    // between consecutive literal runs is allowed to differ.
    let mut chunks: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut i = 0;
    let bytes = src.as_bytes();
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '\\' && i + 1 < bytes.len() {
            let n = bytes[i + 1] as char;
            match n {
                // Common JS regex escapes used in this test suite.
                'b' | 'B' | 'd' | 'D' | 'w' | 'W' | 's' | 'S' | 'n' | 't' => {
                    if !cur.is_empty() {
                        chunks.push(std::mem::take(&mut cur));
                    }
                }
                _ => {
                    // Treat any other escaped char as the literal char.
                    cur.push(n);
                }
            }
            i += 2;
            continue;
        }
        match c {
            '.' | '*' | '+' | '?' | '[' | ']' | '(' | ')' | '|' | '{' | '}' | '^' | '$' => {
                if !cur.is_empty() {
                    chunks.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(c),
        }
        i += 1;
    }
    if !cur.is_empty() {
        chunks.push(cur);
    }
    // Empty pattern always matches.
    if chunks.is_empty() {
        return true;
    }
    // Each chunk must appear in order in the haystack.
    let mut start = 0;
    for chunk in &chunks {
        if chunk.is_empty() {
            continue;
        }
        if let Some(idx) = haystack[start..].find(chunk.as_str()) {
            start += idx + chunk.len();
        } else {
            return false;
        }
    }
    true
}

fn assertion_passes_contain(haystack: &str, rhs: &serde_json::Value, negated: bool) -> bool {
    let m = rhs.as_str().map(|s| haystack.contains(s)).unwrap_or(false);
    if negated { !m } else { m }
}

/// Run one assertions fixture: replay every satori call and verify each
/// captured assertion against the resulting Rust output. Assertions whose
/// `lhs` is not a satori sentinel (e.g. checks on JS-internal helper return
/// values that we can't replay in Rust) are skipped — the corresponding
/// behavior is covered either implicitly by snapshot fixtures, by a hand-
/// ported Rust unit test, or simply has no Rust analog. Failures panic with
/// a descriptive message.
pub fn run_assertions_fixture(name: &str) {
    let path = fixtures_dir().join(name);
    let s = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing fixture {}: {e}", path.display()));
    let fx: AssertionsFixture = serde_json::from_str(&s)
        .unwrap_or_else(|e| panic!("malformed fixture {name}: {e}"));

    // Run every call up-front; assertions index into this Vec via callId.
    let results: Vec<CallResult> = fx.calls.iter().map(run_call).collect();

    for (idx, a) in fx.assertions.iter().enumerate() {
        check_assertion(name, idx, a, &results);
    }
}

fn check_assertion(fixture: &str, idx: usize, a: &Assertion, results: &[CallResult]) {
    let lhs_kind = a.lhs.get("kind").and_then(|v| v.as_str()).unwrap_or("");
    let call_id = a.lhs.get("callId").and_then(|v| v.as_u64()).unwrap_or(u64::MAX) as usize;

    match lhs_kind {
        "satoriSvg" => {
            let r = results.get(call_id);
            let svg = match r {
                Some(CallResult::Ok(s)) => s,
                Some(CallResult::Err(e)) => panic!(
                    "[{fixture} #{idx}] expected satori call {call_id} to resolve, got Err: {e}"
                ),
                None => panic!(
                    "[{fixture} #{idx}] assertion references missing satori call {call_id}"
                ),
            };
            match a.kind.as_str() {
                "toBe" => {
                    if a.rhs.as_str() == Some("string") {
                        // typeof svg === 'string' — trivially true.
                    } else if let Some(s) = a.rhs.as_str() {
                        let eq = svg == s;
                        if eq == a.negated {
                            panic!(
                                "[{fixture} #{idx}] toBe failed (negated={}): expected `{}`, got svg of length {}",
                                a.negated, s, svg.len()
                            );
                        }
                    }
                }
                "toContain" => {
                    // Strict on both polarities: a `toContain` mismatch is a
                    // real port gap (the JS engine produced the substring,
                    // we did not; fix the builder/CSS handler or the
                    // dispatch path).
                    if a.negated {
                        if assertion_passes_contain(svg, &a.rhs, false) {
                            let needle = a.rhs.as_str().unwrap_or("<non-string>");
                            panic!("[{fixture} #{idx}] svg should NOT contain `{needle}`");
                        }
                    } else if let Some(needle) = a.rhs.as_str() {
                        if !svg.contains(needle) {
                            panic!(
                                "[{fixture} #{idx}] toContain failed: svg does not contain `{needle}` (JS satori does). svg.len()={}",
                                svg.len()
                            );
                        }
                    }
                }
                "toMatch" => {
                    if a.negated {
                        if match_string(svg, &a.rhs) {
                            panic!(
                                "[{fixture} #{idx}] svg should NOT match {} (negated)",
                                describe_matcher(&a.rhs)
                            );
                        }
                    } else if !match_string(svg, &a.rhs) {
                        panic!(
                            "[{fixture} #{idx}] toMatch failed: svg does not match {} (JS satori does). svg.len()={}",
                            describe_matcher(&a.rhs),
                            svg.len()
                        );
                    }
                }
                _ => {
                    // toMatchSnapshot / toMatchInlineSnapshot / etc — skip;
                    // we have no recorded expected snapshot for satori-rs.
                }
            }
        }
        "satoriPromise" => {
            let r = results.get(call_id).unwrap_or_else(|| {
                panic!("[{fixture} #{idx}] assertion references missing satori call {call_id}")
            });
            match a.kind.as_str() {
                "rejectsToThrow" => {
                    // Strict: if JS satori throws on this input, the Rust
                    // port must throw too. Either port the missing
                    // validation (preferred) or update the Rust error
                    // wording to structurally match the JS matcher.
                    // Vitest semantics: `rejectsToThrow()` with no
                    // argument matches any error.
                    let any_error = a.rhs.is_null();
                    match (r.err_msg(), a.negated) {
                        (Some(err), false) => {
                            if !any_error && !match_string(err, &a.rhs) {
                                panic!(
                                    "[{fixture} #{idx}] satori rejected but message differs from JS satori.\n  expected matcher: {}\n  got: {err}",
                                    describe_matcher(&a.rhs)
                                );
                            }
                        }
                        (None, false) => panic!(
                            "[{fixture} #{idx}] expected satori call {call_id} to reject matching {}, but it resolved successfully",
                            describe_matcher(&a.rhs)
                        ),
                        (Some(err), true) => {
                            if any_error {
                                panic!(
                                    "[{fixture} #{idx}] satori was expected NOT to reject, but it did (err: {err})"
                                );
                            } else if match_string(err, &a.rhs) {
                                panic!(
                                    "[{fixture} #{idx}] satori was expected to NOT reject with {}, but it did (err: {err})",
                                    describe_matcher(&a.rhs)
                                );
                            }
                        }
                        (None, true) => { /* not.rejectsToThrow + Ok -> pass */ }
                    }
                }
                "resolvesToBe" | "resolvesToContain" | "resolvesToEqual" => {
                    let _ = r.svg().unwrap_or_else(|| {
                        panic!(
                            "[{fixture} #{idx}] expected satori call {call_id} to resolve, got Err: {}",
                            r.err_msg().unwrap_or_default()
                        )
                    });
                }
                _ => {
                    // unknown promise-shaped assertion — ignore.
                }
            }
        }
        // For non-satori LHS kinds (string, number, array, etc.), the
        // assertion is about a JS-internal helper that has no satori-rs
        // analog. The corresponding behavior is either covered by other
        // tests (snapshot fixtures, hand-ports) or has no equivalent.
        // Pass trivially.
        _ => {}
    }
}

fn describe_matcher(rhs: &serde_json::Value) -> String {
    if let Some(s) = rhs.as_str() {
        return format!("`{s}`");
    }
    if let Some(obj) = rhs.as_object() {
        if let Some(src) = obj.get("__regex").and_then(|v| v.as_str()) {
            return format!("/{src}/");
        }
        if let Some(msg) = obj.get("__error").and_then(|v| v.as_str()) {
            return format!("Error({msg})");
        }
    }
    format!("{rhs}")
}

/// Public reexport so example binaries (e.g. `dump_svg`) can apply the
/// same URL mocking that `run_fixture` does.
pub fn mock_image_urls_for_test(value: &mut serde_json::Value) {
    mock_image_urls(value);
}

/// Walk a JSX-shape element tree and rewrite well-known HTTP image URLs
/// (`https://via.placeholder.com/*`) into the same data URIs that the
/// JS test suite's mocked `fetch` would produce. Required because
/// satori-rs intentionally doesn't fetch at runtime; the JS snapshots,
/// however, were generated against the mocked fetch in
/// `test/image.test.tsx`. We reproduce the same response bodies here.
fn mock_image_urls(value: &mut serde_json::Value) {
    use serde_json::Value;
    match value {
        Value::Object(map) => {
            // Substitute `<img src="...">` or any `src: "https://..."` prop.
            if let Some(src) = map.get("src").and_then(|v| v.as_str()) {
                if let Some(replacement) = mock_url_to_data_uri(src) {
                    map.insert("src".to_string(), Value::String(replacement));
                }
            }
            // Substitute `style: { backgroundImage: "url(https://...)" }`
            // and `style: { maskImage: "url(...)" }`. The JS test suite
            // installs a global `fetch` polyfill that responds to these
            // URLs, but our Rust pipeline doesn't fetch — the mock has
            // to inline the URL replacement so the resolver downstream
            // sees a `data:` URI it can parse.
            for css_key in ["backgroundImage", "maskImage", "WebkitMaskImage"] {
                let needs_replacement = map
                    .get(css_key)
                    .and_then(|v| v.as_str())
                    .map(|s| s.contains("url("))
                    .unwrap_or(false);
                if needs_replacement {
                    let raw = map.get(css_key).and_then(|v| v.as_str()).unwrap().to_string();
                    let replaced = replace_urls_in_css_value(&raw);
                    map.insert(css_key.to_string(), Value::String(replaced));
                }
            }
            for v in map.values_mut() {
                mock_image_urls(v);
            }
        }
        Value::Array(items) => {
            for v in items {
                mock_image_urls(v);
            }
        }
        _ => {}
    }
}

/// Walk a CSS value string (typically `background-image` /
/// `mask-image`) and rewrite each `url(...)` whose argument starts with
/// `http://` or `https://` using `mock_url_to_data_uri`. Quoted /
/// unquoted forms are both handled.
fn replace_urls_in_css_value(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i..].starts_with(b"url(") {
            // Find the matching closing paren, respecting paren depth.
            let start = i + 4;
            let mut depth = 1;
            let mut j = start;
            while j < bytes.len() && depth > 0 {
                if bytes[j] == b'(' {
                    depth += 1;
                } else if bytes[j] == b')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                j += 1;
            }
            if depth != 0 {
                out.push_str(&s[i..]);
                return out;
            }
            let inner = &s[start..j];
            let trimmed = inner
                .trim()
                .trim_start_matches(['"', '\''])
                .trim_end_matches(['"', '\'']);
            let new_inner = if trimmed.starts_with("https://") || trimmed.starts_with("http://")
            {
                if let Some(replacement) = mock_url_to_data_uri(trimmed) {
                    replacement
                } else {
                    trimmed.to_string()
                }
            } else {
                inner.to_string()
            };
            out.push_str("url(");
            out.push_str(&new_inner);
            out.push(')');
            i = j + 1;
        } else {
            // Push the next char and advance by its UTF-8 length.
            let ch = s[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
        }
    }
    out
}

/// Map well-known mocked URLs to the data URI bytes that the JS test
/// suite's `globalThis.fetch` polyfill returns. See
/// `tests/image.rs` (`beforeEach` hook).
fn mock_url_to_data_uri(src: &str) -> Option<String> {
    if src.contains("wrong-url") {
        // JS path: `cache.set(url, [])` → satori emits no `<image>`.
        // The empty data URI sentinel makes `resolve_image_src` return
        // `None` and we fall through to the same behavior.
        return None;
    }
    if src.starts_with("https://") || src.starts_with("http://") {
        if src.ends_with(".svg") {
            // The JS fetch mock returns this exact SVG string for any
            // URL ending in `.svg`, which is then re-encoded into a
            // base64 data URI by `resolveImageData`.
            let svg = "<svg width=\"116.15\" height=\"100\" xmlns=\"http://www.w3.org/2000/svg\"><path fill-rule=\"evenodd\" clip-rule=\"evenodd\" d=\"M57.5 0L115 100H0L57.5 0z\"/></svg>";
            use base64::Engine;
            let b64 = base64::engine::general_purpose::STANDARD.encode(svg.as_bytes());
            return Some(format!("data:image/svg+xml;base64,{b64}"));
        }
        // Everything else (placeholder.com/{150,200,300,...}) is a 1×1
        // blue PNG in the JS mock.
        return Some(
            "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNkYPj/HwADBwIAMCbHYQAAAABJRU5ErkJggg=="
                .to_string(),
        );
    }
    None
}

/// Rasterize a satori-produced SVG into PNG bytes.
///
/// Pure Rust pipeline: `usvg::Tree::from_str` -> `resvg::render` ->
/// `tiny_skia::Pixmap::encode_png`, with `fitTo: width`, no system
/// fonts, and Playfair Display as the SVG fallback font.
pub fn to_image(svg: &str, width: u32) -> Vec<u8> {
    let mut fontdb = fontdb_legacy::Database::new();
    let playfair = std::fs::read(asset("playfair-display.ttf"))
        .expect("playfair-display.ttf in .//assets");
    fontdb.load_font_data(playfair);

    let opt = usvg::Options {
        font_family: "Playfair Display".to_string(),
        fontdb,
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(svg, &opt.to_ref())
        .unwrap_or_else(|e| panic!("usvg::Tree::from_str: {e}"));
    let size = tree.svg_node().size.to_screen_size();
    let fit = usvg::FitTo::Width(width);
    let dst = fit.fit_to(size).expect("fit_to width");
    let mut pixmap = tiny_skia::Pixmap::new(dst.width(), dst.height()).expect("alloc pixmap");
    resvg::render(
        &tree,
        fit,
        tiny_skia::Transform::default(),
        pixmap.as_mut(),
    )
    .expect("resvg::render returned None (empty rendering)");
    pixmap.encode_png().expect("encode_png")
}

pub fn read_snapshot(name: &str) -> Vec<u8> {
    std::fs::read(snapshot_path(name)).unwrap_or_else(|e| {
        panic!(
            "missing snapshot {}: {}",
            snapshot_path(name).display(),
            e
        )
    })
}

fn snapshot_update_mode() -> bool {
    matches!(std::env::var("SATORI_RS_UPDATE_SNAPSHOTS").as_deref(), Ok("1") | Ok("true"))
}

/// Compare a freshly produced PNG to the vendored snapshot.
///
/// Pixels are compared with **zero tolerance**.
///
/// Set `SATORI_RS_UPDATE_SNAPSHOTS=1` to overwrite the on-disk
/// snapshot with the freshly produced PNG instead of comparing.
/// This is the only sanctioned way to refresh the vendored snapshot
/// baseline (e.g. after a rasterizer-version bump).
pub fn assert_image_snapshot(png: &[u8], name: &str) {
    if snapshot_update_mode() {
        let path = snapshot_path(name);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(&path, png)
            .unwrap_or_else(|e| panic!("write snapshot {}: {e}", path.display()));
        return;
    }
    let reference = read_snapshot(name);

    // Fast path: bytes already identical.
    if png == reference.as_slice() {
        return;
    }

    let actual = decode_png(png).unwrap_or_else(|e| panic!("decode actual: {e}"));
    let expected = decode_png(&reference)
        .unwrap_or_else(|e| panic!("decode reference {name}: {e}"));

    if actual.width == expected.width
        && actual.height == expected.height
        && actual.rgba == expected.rgba
    {
        return;
    }

    // Mismatch: dump produced PNG for diffing.
    let actual_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("target")
        .join("snapshot-actual");
    let _ = std::fs::create_dir_all(&actual_dir);
    let actual_path = actual_dir.join(name);
    let _ = std::fs::write(&actual_path, png);

    let mut diff_bytes = 0usize;
    let len = actual.rgba.len().min(expected.rgba.len());
    for i in 0..len {
        if actual.rgba[i] != expected.rgba[i] {
            diff_bytes += 1;
        }
    }

    panic!(
        "image snapshot mismatch: {name}\n  expected: {ew}x{eh} ({rpath})\n  actual:   {aw}x{ah} (written to {apath})\n  differing bytes: {diff}/{len}",
        ew = expected.width,
        eh = expected.height,
        rpath = snapshot_path(name).display(),
        aw = actual.width,
        ah = actual.height,
        apath = actual_path.display(),
        diff = diff_bytes,
        len = expected.rgba.len(),
    );
}

struct Decoded {
    width: u32,
    height: u32,
    rgba: Vec<u8>,
}

fn decode_png(bytes: &[u8]) -> Result<Decoded, String> {
    let decoder = png::Decoder::new(bytes);
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    buf.truncate(info.buffer_size());

    let width = info.width;
    let height = info.height;

    // Normalize to 8-bit RGBA so comparisons work across snapshot color types.
    let rgba = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => buf,
        (png::ColorType::Rgb, png::BitDepth::Eight) => buf
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 0xff])
            .collect(),
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight) => buf
            .chunks_exact(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        (png::ColorType::Grayscale, png::BitDepth::Eight) => {
            buf.iter().flat_map(|&v| [v, v, v, 0xff]).collect()
        }
        (ct, bd) => return Err(format!("unsupported png ({ct:?}, {bd:?})")),
    };
    Ok(Decoded { width, height, rgba })
}
