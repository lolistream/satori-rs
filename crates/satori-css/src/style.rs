//! Typed slice of CSS that the satori renderer needs.
//!
//! This is the merge of:
//!   * `src/handler/expand.ts`'s `SerializedStyle` (the "after expansion"
//!     object that the layout pass consumes), and
//!   * the `inheritedStyle` / `parentStyle` shapes that are passed around
//!     in `src/layout.ts`.

use crate::dimension::Dim;
use crate::gradient::{ConicGradient, LinearGradient, RadialGradient};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Display {
    Flex,
    Block,
    None,
    Contents,
    /// `-webkit-box`. Yoga lays this out as flex, but JS satori keys
    /// `text-overflow: ellipsis` + `WebkitLineClamp` off this exact
    /// `display` value, so we keep it distinct.
    WebkitBox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Auto,
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignSelf {
    Auto,
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    Baseline,
}

/// `align-content` controls the cross-axis spacing of multiple flex lines
/// when `flex-wrap: wrap` produces them. Mirrors yoga's `YGAlign*` set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignContent {
    Auto,
    Stretch,
    FlexStart,
    FlexEnd,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle { Normal, Italic }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position { Static, Relative, Absolute }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign { Start, End, Left, Right, Center, Justify }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhiteSpace { Normal, NoWrap, Pre, PreWrap, PreLine, BreakSpaces }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordBreak { Normal, BreakAll, KeepAll, BreakWord }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextWrap { Wrap, Nowrap, Balance, Pretty }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOverflow { Clip, Ellipsis }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow { Visible, Hidden, Scroll, Auto }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecorationLine { None, Underline, LineThrough }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDecorationStyle { Solid, Dashed, Dotted, Double }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextTransform { None, Uppercase, Lowercase, Capitalize }
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle { None, Solid, Dashed, Dotted, Double, Hidden }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectFit { Fill, Contain, Cover, ScaleDown, None }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxSizing {
    BorderBox,
    ContentBox,
}

/// A single radius axis component: either an absolute px value or a
/// percentage to be resolved against the box at render time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RadiusLen {
    Px(f32),
    /// Percentage (0–100). Horizontal corners resolve against `width`;
    /// vertical corners resolve against `height`.
    Percent(f32),
}

impl RadiusLen {
    pub fn resolve(self, basis: f32) -> f32 {
        match self {
            RadiusLen::Px(v) => v,
            // Use `(p * basis) / 100` ordering to match JS — f32
            // `p/100*basis` introduces 1-ULP drift for many common
            // values (e.g. 30% of 100 = 30.000002 instead of 30).
            RadiusLen::Percent(p) => (p * basis) / 100.0,
        }
    }
}

/// Border radius value: either a single uniform value or per-axis
/// (`<horizontal> <vertical>`). Percentages are stored verbatim so the
/// renderer can resolve them against the box dimensions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadiusValue {
    pub h: RadiusLen,
    pub v: RadiusLen,
    /// Was the source a single value? If so, after axis resolution we take
    /// the min of h/v and apply it to both. Per JS `resolveRadius`, a
    /// percentage value is *not* treated as a single-corner shrink even
    /// when only one component is given.
    pub single: bool,
}

/// `translate(...)` length: percentages are resolved against the
/// element's box width/height at render time, so we keep both forms.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformLen {
    Px(f32),
    Percent(f32),
}

/// Single CSS `transform: ...` function. The JS upstream stores these as
/// `[{ translateX: 10 }, { rotate: 45 }, ...]`; we use a typed enum.
///
/// Notes:
/// - `Scale(f)` is uniform (`scale(1.5)`). `scale(2, 3)` is expanded into
///   `[ScaleX(2), ScaleY(3)]` to match how `css-to-react-native` desugars.
/// - Angles are stored in degrees (already converted from rad/turn/grad).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransformOp {
    TranslateX(TransformLen),
    TranslateY(TransformLen),
    Scale(f32),
    ScaleX(f32),
    ScaleY(f32),
    Rotate(f32),
    SkewX(f32),
    SkewY(f32),
    Matrix([f32; 6]),
}

/// Parsed `transform-origin` value (port of `ParsedTransformOrigin`).
/// Any field can be `None`; missing axes default to `50%` (center) at
/// render time.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TransformOrigin {
    pub x_relative: Option<f32>,
    pub y_relative: Option<f32>,
    pub x_absolute: Option<f32>,
    pub y_absolute: Option<f32>,
}

/// A single parsed entry in the `background-image` layer list.
///
/// Mirrors the JS shape: an array of `linear-gradient(...)`,
/// `radial-gradient(...)`, or `url(...)` images that the renderer
/// composites bottom-up (first item paints on top).
#[derive(Debug, Clone, PartialEq)]
pub enum BackgroundImage {
    Linear(LinearGradient),
    Radial(RadialGradient),
    Conic(ConicGradient),
    /// `url(...)` reference. `src` is the raw value extracted from
    /// `url(...)` (with quotes stripped). `resolved` is populated by
    /// the layout pipeline once the image bytes have been fetched / the
    /// `__assetFile` shape has been resolved on disk; `None` until then.
    Url {
        src: String,
        resolved: Option<ResolvedUrlImage>,
    },
}

/// Resolved natural dimensions + final `data:` URI for a `url(...)`
/// background image. The `<image href=...>` we emit references `src`
/// directly. Natural dimensions are needed because `background-size:
/// cover/contain/auto` resolves against them.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedUrlImage {
    pub src: String,
    pub natural_width: Option<f32>,
    pub natural_height: Option<f32>,
}

/// Parsed `clip-path` value. Port of `src/parser/shape.ts`. The values
/// are already resolved against the element's box (so `circle(50%)` on
/// a 100x100 element is stored as `ClipPathShape::Circle { r: 50.0, cx: 50.0, cy: 50.0 }`).
#[derive(Debug, Clone, PartialEq)]
pub enum ClipPathShape {
    Circle { r: f32, cx: f32, cy: f32 },
    Ellipse { rx: f32, ry: f32, cx: f32, cy: f32 },
    Inset { x: f32, y: f32, width: f32, height: f32, path: Option<String> },
    Polygon { fill_rule: String, points: String },
    Path { fill_rule: String, d: String },
}

/// One CSS `box-shadow` entry. Mirrors the shape `css-box-shadow`
/// returns: four offset/blur/spread lengths in px plus a color string
/// and an `inset` flag.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: String,
    pub inset: bool,
}

/// One CSS `text-shadow` entry. Mirrors React Native's `textShadow*`
/// triple (offset / radius / color). No `spread` or `inset`.
#[derive(Debug, Clone, PartialEq)]
pub struct TextShadow {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub color: String,
}

#[derive(Debug, Clone, Default)]
pub struct ComputedStyle {
    // Box / layout
    pub display: Option<Display>,
    pub position: Option<Position>,
    pub width: Option<Dim>,
    pub height: Option<Dim>,
    pub min_width: Option<Dim>,
    pub min_height: Option<Dim>,
    pub max_width: Option<Dim>,
    pub max_height: Option<Dim>,
    pub top: Option<Dim>,
    pub right: Option<Dim>,
    pub bottom: Option<Dim>,
    pub left: Option<Dim>,

    pub margin_top: Option<Dim>,
    pub margin_right: Option<Dim>,
    pub margin_bottom: Option<Dim>,
    pub margin_left: Option<Dim>,

    pub padding_top: Option<Dim>,
    pub padding_right: Option<Dim>,
    pub padding_bottom: Option<Dim>,
    pub padding_left: Option<Dim>,

    pub border_top_width: Option<f32>,
    pub border_right_width: Option<f32>,
    pub border_bottom_width: Option<f32>,
    pub border_left_width: Option<f32>,

    pub border_top_color: Option<String>,
    pub border_right_color: Option<String>,
    pub border_bottom_color: Option<String>,
    pub border_left_color: Option<String>,

    pub border_top_style: Option<BorderStyle>,
    pub border_right_style: Option<BorderStyle>,
    pub border_bottom_style: Option<BorderStyle>,
    pub border_left_style: Option<BorderStyle>,

    pub border_top_left_radius: Option<RadiusValue>,
    pub border_top_right_radius: Option<RadiusValue>,
    pub border_bottom_left_radius: Option<RadiusValue>,
    pub border_bottom_right_radius: Option<RadiusValue>,

    pub flex_direction: Option<FlexDirection>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<Dim>,
    pub flex_wrap: Option<bool>, // simple: wrap or no-wrap
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub align_self: Option<AlignSelf>,
    pub align_content: Option<AlignContent>,
    pub gap: Option<Dim>,
    pub row_gap: Option<Dim>,
    pub column_gap: Option<Dim>,

    // Visual
    pub background_color: Option<String>, // serialized CSS color string
    pub background_image: Option<Vec<BackgroundImage>>,
    pub background_size: Option<String>,
    pub background_position: Option<String>,
    pub background_repeat: Option<String>,
    /// `background-clip` / `-webkit-background-clip` raw value
    /// (`"text"`, `"border-box"`, `"padding-box"`, etc.). Currently
    /// only `"text"` is special-cased downstream.
    pub background_clip: Option<String>,
    /// Mask layers — same layer shape as `background_image`, but the
    /// gradient color stops are remapped to `rgba(255,255,255,alpha)` /
    /// `rgba(0,0,0,1)` so the resulting alpha mask reflects the
    /// gradient's source alpha. Sourced from either `maskImage` or
    /// `WebkitMaskImage`.
    pub mask_image: Option<Vec<BackgroundImage>>,
    pub mask_size: Option<String>,
    pub mask_position: Option<String>,
    pub mask_repeat: Option<String>,
    pub color: Option<String>,
    pub opacity: Option<f32>,

    // Text
    pub font_size: Option<f32>,
    /// Precise f64 value of `font-size` propagated through the em/rem
    /// resolution chain. JS satori computes `0.8em * 1.5em` in f64 and
    /// passes the exact f64 (e.g. `19.20000000000000284217`) to
    /// `opentype.js.getPath`. Storing only as `f32` loses ~13 bits of
    /// mantissa and yields a different `scale`, which can flip a
    /// glyph coordinate from e.g. `53.3` to `53.4` after `toFixed(1)`.
    /// Renderer code that needs JS parity (`run_path_d`,
    /// `measure_advance`) should prefer this when `Some`.
    pub _font_size_f64: Option<f64>,
    pub font_family: Option<String>,
    pub font_weight: Option<u16>,
    pub font_style: Option<FontStyle>,
    /// Numeric `line-height` value as a multiplier of `font-size`. `None`
    /// represents the CSS `normal` keyword — the renderer computes the
    /// actual height from the font's `ascender - descender + line_gap`
    /// instead, matching JS satori's `'normal' === lineHeight` branch.
    pub line_height: Option<f32>,
    pub text_align: Option<TextAlign>,
    pub white_space: Option<WhiteSpace>,
    pub letter_spacing: Option<f32>,
    pub text_indent: Option<Dim>,
    pub tab_size: Option<f32>,
    pub word_break: Option<WordBreak>,
    pub text_wrap: Option<TextWrap>,
    pub text_overflow: Option<TextOverflow>,
    pub overflow: Option<Overflow>,
    pub line_clamp: Option<u32>,
    /// Optional custom block-ellipsis from `line-clamp: <n> "<ellipsis>"`
    /// (or single-quoted form). When `None`, the renderer falls back to
    /// the default `…` (U+2026).
    pub line_clamp_ellipsis: Option<String>,
    pub webkit_line_clamp: Option<u32>,
    pub webkit_box_orient: Option<String>,
    pub text_decoration_line: Option<TextDecorationLine>,
    pub text_decoration_color: Option<String>,
    pub text_decoration_style: Option<TextDecorationStyle>,
    pub text_decoration_skip_ink: Option<String>,
    pub text_transform: Option<TextTransform>,
    pub _webkit_text_fill_color: Option<String>,
    pub _webkit_text_stroke_width: Option<f32>,
    pub _webkit_text_stroke_color: Option<String>,

    // Transform
    pub transform: Option<Vec<TransformOp>>,
    pub transform_origin: Option<TransformOrigin>,

    // Effects
    pub box_shadow: Option<Vec<BoxShadow>>,
    pub text_shadow: Option<Vec<TextShadow>>,
    /// Raw CSS `filter` value (e.g. `blur(1px)`, `grayscale(50%)`).
    /// Passed through as-is into the SVG `<image>`/`<text>` element's
    /// inline `style="filter:..."` attribute when present.
    pub filter: Option<String>,

    /// Raw `clip-path` CSS value (`circle(...)`, `ellipse(...)`, etc.).
    /// Parsed against the element's box at render time when the box's
    /// width/height are known. `none` short-circuits to `None`.
    pub clip_path: Option<String>,
    /// Inherited clip-path id from a parent that owns the explicit
    /// clip-path or has `overflow: hidden`. Children write the same
    /// `clip-path="url(#...)"` attribute on their own rects/text.
    /// Port of `_inheritedClipPathId` in `layout.ts`.
    pub _inherited_clip_path_id: Option<String>,
    /// Inherited content-mask id from `overflow: hidden` parent.
    /// Port of `_inheritedMaskId` in `layout.ts`.
    pub _inherited_mask_id: Option<String>,
    /// Set on the element that owns `background-clip: text`. The
    /// renderer emits `<clipPath id="satori_bct-{id}"><path d="..."/></clipPath>`
    /// before this element's rect, where the path data is the
    /// concatenated glyph paths of descendant text nodes.
    pub _bg_clip_text_path_d: Option<String>,
    /// Whether this element owns a `background-clip: text` context.
    /// Set during expand so the layout/rendering pipeline can mark
    /// descendants as contributing to it (`_inherited_bg_clip_text_target`).
    pub _bg_clip_text_self: Option<bool>,
    /// Set on descendants of a `background-clip: text` element so the
    /// text renderer knows to append its merged glyph path into the
    /// closest ancestor's `_bg_clip_text_path_d` collector.
    /// The string is the target element's id (so we can resolve which
    /// ancestor receives the path).
    pub _inherited_bg_clip_text_target: Option<String>,
    /// Whether the ancestor that set `_inherited_bg_clip_text_target`
    /// also has a `background-image` layer. JS satori uses this to
    /// decide whether to render the text natively (no background) or
    /// suppress it (background paint mode).
    pub _inherited_bg_clip_text_has_background: Option<bool>,

    // Layout
    pub box_sizing: Option<BoxSizing>,

    // Image (`<img>`)
    pub object_fit: Option<ObjectFit>,
    pub object_position: Option<String>,
    /// Pre-resolved image data — usually a `data:image/...` URI.
    pub _src: Option<String>,
    /// Natural pixel dimensions parsed from the image bytes.
    pub _natural_width: Option<f32>,
    pub _natural_height: Option<f32>,
    /// Yoga aspect ratio hint set by `<img>` compute (mirrors
    /// `node.setAspectRatio(1/r)` in JS satori).
    pub _aspect_ratio: Option<f32>,

    // Internal viewport hints (for vw/vh)
    pub _viewport_width: Option<u32>,
    pub _viewport_height: Option<u32>,
}

impl ComputedStyle {
    pub fn inheritable_root() -> Self {
        Self {
            font_size: Some(16.0),
            _font_size_f64: Some(16.0),
            font_weight: Some(400),
            font_family: Some("serif".into()),
            font_style: Some(FontStyle::Normal),
            color: Some("black".into()),
            opacity: Some(1.0),
            line_height: None,
            white_space: Some(WhiteSpace::Normal),
            ..Default::default()
        }
    }
}
