# satori-rs

Pure-Rust reimplementation of [Vercel Satori](https://github.com/vercel/satori).

No JavaScript runtime, no Node subprocess: SVG generation
is pure Rust and rasterization to PNG goes through the in-process Rust
`usvg`/`resvg`/`tiny-skia` stack.

## Layout

```
crates/
  satori          # public entry: satori() / satori_from_value()
  satori-css      # ComputedStyle, JS-camelCase expand, color/length/gradient/
                  # shadow/clip-path/border-radius parsers, CSS variables,
                  # 30+ CSS property handlers
  satori-font     # FontLoader, ParsedFont (ttf-parser metrics + kerning),
                  # PathDataBuilder (opentype.js-compatible glyph extraction)
  satori-text     # Line-breaking (UAX#14), text-align, white-space state
                  # machine, tab-size, word-break, text-wrap, line-clamp,
                  # text-indent, text-decoration positioning, skip-ink
  satori-jsx      # JSX-shape JSON Element/Props types
  satori-layout   # yoga-rs (pure Rust) bindings + measure-fn for text nodes
  satori-handler  # image: data:URI + asset-file loader, dimension sniffing
  satori-builder  # SVG builders: svg, rect, border, border-radius, transform,
                  # background-image (linear/radial/conic gradients + url()),
                  # shadow (box + text), text, text-decoration, clip-path,
                  # mask-image
  satori-tests    # Vendored test data + snapshot-driven test suite
xtask/            # Cargo workspace tooling (no Node code)
```

## Test data

`crates/satori-tests/` is fully self-contained:

```
crates/satori-tests/
  assets/        # font files used by the tests
  snapshots/     # PNG ground truth (one per snapshot test)
  fixtures/      # JSON-shape JSX trees + options for each test (committed)
  tests/         # the actual #[test] functions, each calling run_fixture
```
