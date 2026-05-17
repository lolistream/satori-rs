//! Port of `src/builder/transform.ts` + the matrix algebra in
//! `src/utils.ts::multiply`.
//!
//! Each `transform: ...` op on a node is resolved into a 2D affine
//! `[a, b, c, d, e, f]` matrix. The element's own ops are applied around
//! its transform-origin (move-to-origin → apply ops → move-back), and
//! finally pre-multiplied by the parent's already-resolved effective
//! matrix. The result is formatted to an SVG `matrix(...)` attribute
//! that's byte-for-byte identical to the JS satori output.
//!
//! See the JS reference:
//! ```js
//! result = multiply(
//!   [1, 0, 0, 1, x, y],
//!   multiply(matrix, [1, 0, 0, 1, -x, -y])
//! )
//! if (matrix.__parent) result = multiply(matrix.__parent, result)
//! return `matrix(${result.map(v => v.toFixed(2)).join(',')})`
//! ```

use crate::css::style::{TransformLen, TransformOp, TransformOrigin};

const BASE_MATRIX: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

/// 2D affine matrix multiplication. Mirrors `multiply` in `src/utils.ts`.
///
/// Matrices are stored in CSS column-major form `[a, b, c, d, e, f]`,
/// representing the 3×3:
/// ```text
///   a c e
///   b d f
///   0 0 1
/// ```
pub fn multiply(m1: &[f32; 6], m2: &[f32; 6]) -> [f32; 6] {
    [
        m1[0] * m2[0] + m1[2] * m2[1],
        m1[1] * m2[0] + m1[3] * m2[1],
        m1[0] * m2[2] + m1[2] * m2[3],
        m1[1] * m2[2] + m1[3] * m2[3],
        m1[0] * m2[4] + m1[2] * m2[5] + m1[4],
        m1[1] * m2[4] + m1[3] * m2[5] + m1[5],
    ]
}

fn op_to_matrix(op: &TransformOp, width: f32, height: f32) -> [f32; 6] {
    let mut m = BASE_MATRIX;
    match op {
        TransformOp::TranslateX(v) => {
            m[4] = match v {
                TransformLen::Px(p) => *p,
                TransformLen::Percent(p) => p / 100.0 * width,
            };
        }
        TransformOp::TranslateY(v) => {
            m[5] = match v {
                TransformLen::Px(p) => *p,
                TransformLen::Percent(p) => p / 100.0 * height,
            };
        }
        TransformOp::Scale(s) => {
            m[0] = *s;
            m[3] = *s;
        }
        TransformOp::ScaleX(s) => m[0] = *s,
        TransformOp::ScaleY(s) => m[3] = *s,
        TransformOp::Rotate(deg) => {
            let rad = deg * std::f32::consts::PI / 180.0;
            let c = rad.cos();
            let s = rad.sin();
            m[0] = c;
            m[1] = s;
            m[2] = -s;
            m[3] = c;
        }
        TransformOp::SkewX(deg) => {
            m[2] = (deg * std::f32::consts::PI / 180.0).tan();
        }
        TransformOp::SkewY(deg) => {
            m[1] = (deg * std::f32::consts::PI / 180.0).tan();
        }
        TransformOp::Matrix(arr) => return *arr,
    }
    m
}

/// Reduce a chain of `TransformOp`s to a single 2D matrix.
///
/// Mirrors the inner loop in `resolveTransforms`:
/// each new op is `multiply(transformMatrix, matrix)` — i.e. the new op
/// is *pre-multiplied* onto the running matrix.
pub fn resolve_ops(ops: &[TransformOp], width: f32, height: f32) -> [f32; 6] {
    let mut matrix = BASE_MATRIX;
    for op in ops {
        let tm = op_to_matrix(op, width, height);
        matrix = multiply(&tm, &matrix);
    }
    matrix
}

/// Compute the px offset of the `transform-origin` relative to the
/// element's top-left corner.
///
/// JS satori uses absolute-overrides-relative: any explicit length wins
/// over a default 50% center.
pub fn origin_offset(origin: Option<&TransformOrigin>, width: f32, height: f32) -> (f32, f32) {
    let xo = match origin {
        Some(o) => o
            .x_absolute
            .unwrap_or_else(|| o.x_relative.unwrap_or(50.0) * width / 100.0),
        None => 50.0 * width / 100.0,
    };
    let yo = match origin {
        Some(o) => o
            .y_absolute
            .unwrap_or_else(|| o.y_relative.unwrap_or(50.0) * height / 100.0),
        None => 50.0 * height / 100.0,
    };
    (xo, yo)
}

/// JS `Number.prototype.toFixed(2)`: always 2 decimal places. We also
/// fold `-0.00` → `0.00` because Rust's `Display` prints negative zeros
/// (e.g. `(-0f32).to_string()`) whereas `toFixed` does not.
pub fn to_fixed_2(n: f32) -> String {
    let s = format!("{:.2}", n);
    if s == "-0.00" { "0.00".to_string() } else { s }
}

pub fn matrix_to_string(m: &[f32; 6]) -> String {
    format!(
        "matrix({},{},{},{},{},{})",
        to_fixed_2(m[0]),
        to_fixed_2(m[1]),
        to_fixed_2(m[2]),
        to_fixed_2(m[3]),
        to_fixed_2(m[4]),
        to_fixed_2(m[5]),
    )
}

/// Compute the effective matrix for an element that has its own
/// `transform:` ops (the "not inheriting" branch in transform.ts).
///
/// `parent` is the parent element's already-resolved effective matrix
/// (`matrix.__parent` in JS, propagated by the caller via the layout
/// tree's parent index).
pub fn resolve_effective(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    ops: &[TransformOp],
    origin: Option<&TransformOrigin>,
    parent: Option<&[f32; 6]>,
) -> [f32; 6] {
    let local = resolve_ops(ops, width, height);
    let (xo, yo) = origin_offset(origin, width, height);
    let x = left + xo;
    let y = top + yo;
    let shifted = multiply(
        &[1.0, 0.0, 0.0, 1.0, x, y],
        &multiply(&local, &[1.0, 0.0, 0.0, 1.0, -x, -y]),
    );
    match parent {
        Some(p) => multiply(p, &shifted),
        None => shifted,
    }
}

/// Public façade used by `rect.rs` callers that want the JS shape
/// directly (`matrix(...)` string or empty string for "no transform").
///
/// When `ops` is empty and `parent` is `None`, returns an empty
/// `String`; downstream code must not emit `transform=""`.
pub fn render_transform(
    left: f32,
    top: f32,
    width: f32,
    height: f32,
    ops: &[TransformOp],
    origin: Option<&TransformOrigin>,
    parent: Option<&[f32; 6]>,
) -> String {
    if ops.is_empty() && parent.is_none() {
        return String::new();
    }
    if ops.is_empty() {
        // Inheriting branch: format parent's matrix directly.
        return matrix_to_string(parent.unwrap());
    }
    let m = resolve_effective(left, top, width, height, ops, origin, parent);
    matrix_to_string(&m)
}
