//! CSS-shaped property representation + JS-style expansion.
//!
//! Mirrors `src/handler/expand.ts`, `presets.ts`, `inheritable.ts`,
//! `variables.ts`, and the `vendor/parse-css-dimension/` helper from
//! upstream satori.
//!
//! The JS satori takes an arbitrary JS object whose keys are camelCase CSS
//! property names. We reproduce that shape here with a `SerializedStyle`
//! map (so previously unknown keys round-trip), plus a strongly-typed
//! `ComputedStyle` for the subset of properties the renderer needs.

pub mod color;
pub mod dimension;
pub mod expand;
pub mod gradient;
pub mod style;
pub mod variables;

pub use color::{parse_color, Rgba};
pub use dimension::{parse_dimension, Dim};
pub use expand::{expand_style, ExpandContext};
pub use gradient::{
    parse_conic_gradient, parse_linear_gradient, parse_radial_gradient, ColorStop, ConicGradient,
    LinearGradient, LinearOrientation, RadialGradient, RadialPosition, RadialPropertyValue,
    StopOffset,
};
pub use style::{
    AlignContent, AlignItems, AlignSelf, BackgroundImage, BorderStyle, BoxShadow, ClipPathShape,
    ComputedStyle, Display, FlexDirection, FontStyle, JustifyContent, ObjectFit, Overflow,
    Position, RadiusLen, RadiusValue, TextAlign, TextDecorationLine, TextDecorationStyle, TextOverflow,
    TextShadow, TextTransform, TextWrap, TransformLen, TransformOp, TransformOrigin, WhiteSpace,
    WordBreak,
};
pub use variables::{extract_vars, merge_vars, substitute, Vars};
