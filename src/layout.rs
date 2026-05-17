//! Yoga-backed layout (port of the layout-tree parts of `src/layout.ts`).
//!
//! Builds a yoga node tree from a JSX-shape element tree + per-node
//! `ComputedStyle`, runs `calculate_layout`, and returns a flat list of
//! laid-out nodes (with absolute screen coords) for the renderer to walk.
//!
//! The yoga backend is the pure-Rust port published as `yoga-rs` on
//! crates.io (depended on as `yoga = { package = "yoga-rs", ... }`). Its
//! API differs from the historical `yoga = "0.5"` C-binding crate in
//! three meaningful ways for our purposes:
//! - Nodes are `NodeRef`s (a `Rc<RefCell<NodeData>>`) rather than owned
//!   `Node` values; setters take `&self` and there's no `&mut`.
//! - Setters are named `set_style_<prop>` and split into one entry
//!   point per `Unit` (`set_style_width(v)` / `set_style_width_percent(v)`
//!   / `set_style_width_auto()`) instead of a single
//!   `set_width(StyleUnit::Point(v))`.
//! - Measure functions are arbitrary closures (`Fn(&NodeRef, f32,
//!   MeasureMode, f32, MeasureMode) -> Size + 'static`) — no
//!   `set_context`/raw pointer/`extern "C"` ABI needed.

use std::sync::Arc;

use crate::css::{
    dimension::Dim,
    style::{
        AlignContent, AlignItems, AlignSelf, BoxSizing as CssBoxSizing, ComputedStyle, Display,
        FlexDirection, JustifyContent,
    },
};
use crate::font::ParsedFont;
use crate::text::{layout_text, TextStyle};
use yoga::{
    Align as YAlign, BoxSizing as YBoxSizing, Direction, Edge, FlexDirection as YFlexDirection,
    Gutter, Justify as YJustify, NodeRef, Overflow, PositionType, Wrap,
};

/// One laid-out element. The renderer walks a flat list in render order.
#[derive(Clone)]
pub struct LaidOut {
    pub id: String,
    pub tag: String,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub style: ComputedStyle,
    /// Index of the parent in the output list (or None for the root frame).
    pub parent: Option<usize>,
    /// Optional text content (already-text child string of this element).
    pub text: Option<String>,
    /// Resolved font list (only meaningful for `tag == "_text"`).
    pub fonts: Vec<Arc<ParsedFont>>,
}

impl std::fmt::Debug for LaidOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LaidOut")
            .field("id", &self.id)
            .field("tag", &self.tag)
            .field("left", &self.left)
            .field("top", &self.top)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("parent", &self.parent)
            .field("text", &self.text)
            .finish_non_exhaustive()
    }
}

pub struct LayoutTree {
    pub nodes: Vec<LaidOut>,
    pub root_width: f32,
    pub root_height: f32,
}

pub struct Built {
    pub yoga: NodeRef,
    pub children: Vec<Built>,
    pub id: String,
    pub tag: String,
    pub style: ComputedStyle,
    pub text: Option<String>,
    /// Resolved font list for this text node (only populated when
    /// `tag == "_text"`). Stored alongside `Built` so the orchestrator
    /// can re-layout text after yoga's `calculate_layout` runs.
    pub fonts: Vec<Arc<ParsedFont>>,
}

/// Yoga measure-func payload for a `_text` synthetic node. The
/// closure registered by `install_text_measure` captures one of
/// these and re-runs `layout_text` against it for each measurement
/// request yoga makes.
pub struct TextMeasureCtx {
    pub text: String,
    pub style: TextStyle,
    pub fonts: Vec<Arc<ParsedFont>>,
}

/// Build a yoga subtree for an element + its already-expanded style.
pub fn make_yoga(style: &ComputedStyle) -> NodeRef {
    let n = NodeRef::new();
    apply_style(&n, style);
    n
}

/// Attach a `TextMeasureCtx` to the yoga node and wire a closure
/// measure callback that reproduces JS yoga's "every mode constrains"
/// semantics. Used by the orchestrator when building a `_text`
/// synthetic node.
pub fn install_text_measure(node: &NodeRef, ctx: TextMeasureCtx) {
    let ctx = std::rc::Rc::new(ctx);
    node.set_measure_fn(move |_n, width, width_mode, _h, _hm| {
        // Mirror JS yoga-asmjs behavior: every mode constrains the
        // measurement (Undefined skips constraint entirely).
        let max_w = match width_mode {
            yoga::MeasureMode::Undefined => None,
            _ => Some(width),
        };
        let layout = layout_text(&ctx.text, &ctx.style, &ctx.fonts, max_w);
        // `Math.ceil` mirrors the JS satori width quirk where yoga
        // sometimes returns a different `getComputedWidth` for
        // non-integer widths.
        yoga::Size {
            width: layout.width.ceil(),
            height: layout.height,
        }
    });
}

/// Apply a length/percent/auto dimension to a generic setter trio
/// (`set_px(v)`, `set_percent(v)`, `set_auto()`).
fn apply_dim(
    d: Dim,
    set_px: impl FnOnce(f32),
    set_percent: impl FnOnce(f32),
    set_auto: impl FnOnce(),
) {
    match d {
        Dim::Px(v) => set_px(v),
        Dim::Percent(v) => set_percent(v),
        Dim::Auto => set_auto(),
    }
}

pub fn apply_style(n: &NodeRef, s: &ComputedStyle) {
    if let Some(d) = s.width {
        apply_dim(
            d,
            |v| n.set_style_width(v),
            |v| n.set_style_width_percent(v),
            || n.set_style_width_auto(),
        );
    }
    if let Some(d) = s.height {
        apply_dim(
            d,
            |v| n.set_style_height(v),
            |v| n.set_style_height_percent(v),
            || n.set_style_height_auto(),
        );
    }
    if let Some(d) = s.min_width {
        apply_dim(
            d,
            |v| n.set_style_min_width(v),
            |v| n.set_style_min_width_percent(v),
            || { /* min-width: auto is yoga's default */ },
        );
    }
    if let Some(d) = s.min_height {
        apply_dim(
            d,
            |v| n.set_style_min_height(v),
            |v| n.set_style_min_height_percent(v),
            || {},
        );
    }
    if let Some(d) = s.max_width {
        apply_dim(
            d,
            |v| n.set_style_max_width(v),
            |v| n.set_style_max_width_percent(v),
            || {},
        );
    }
    if let Some(d) = s.max_height {
        apply_dim(
            d,
            |v| n.set_style_max_height(v),
            |v| n.set_style_max_height_percent(v),
            || {},
        );
    }

    apply_edge_dim(s.margin_top, Edge::Top, n, EdgeKind::Margin);
    apply_edge_dim(s.margin_right, Edge::Right, n, EdgeKind::Margin);
    apply_edge_dim(s.margin_bottom, Edge::Bottom, n, EdgeKind::Margin);
    apply_edge_dim(s.margin_left, Edge::Left, n, EdgeKind::Margin);

    apply_edge_dim(s.padding_top, Edge::Top, n, EdgeKind::Padding);
    apply_edge_dim(s.padding_right, Edge::Right, n, EdgeKind::Padding);
    apply_edge_dim(s.padding_bottom, Edge::Bottom, n, EdgeKind::Padding);
    apply_edge_dim(s.padding_left, Edge::Left, n, EdgeKind::Padding);

    if let Some(v) = s.border_top_width { n.set_style_border(Edge::Top, v); }
    if let Some(v) = s.border_right_width { n.set_style_border(Edge::Right, v); }
    if let Some(v) = s.border_bottom_width { n.set_style_border(Edge::Bottom, v); }
    if let Some(v) = s.border_left_width { n.set_style_border(Edge::Left, v); }

    // JS satori always calls `node.setFlexDirection` with a `row`
    // default when style.flexDirection isn't specified (compute.ts:251).
    // Yoga's intrinsic default is Column, so we have to set Row
    // explicitly even when the user didn't set anything.
    n.set_style_flex_direction(match s.flex_direction {
        Some(FlexDirection::Row) | None => YFlexDirection::Row,
        Some(FlexDirection::Column) => YFlexDirection::Column,
        Some(FlexDirection::RowReverse) => YFlexDirection::RowReverse,
        Some(FlexDirection::ColumnReverse) => YFlexDirection::ColumnReverse,
    });
    if let Some(g) = s.flex_grow { n.set_style_flex_grow(g); }
    if let Some(g) = s.flex_shrink { n.set_style_flex_shrink(g); }
    if let Some(d) = s.flex_basis {
        apply_dim(
            d,
            |v| n.set_style_flex_basis(v),
            |v| n.set_style_flex_basis_percent(v),
            || n.set_style_flex_basis_auto(),
        );
    }
    if let Some(w) = s.flex_wrap {
        n.set_style_flex_wrap(if w { Wrap::Wrap } else { Wrap::NoWrap });
    }
    if let Some(g) = s.gap {
        apply_dim(
            g,
            |v| n.set_style_gap(Gutter::All, v),
            |v| n.set_style_gap_percent(Gutter::All, v),
            || {},
        );
    }
    if let Some(g) = s.row_gap {
        apply_dim(
            g,
            |v| n.set_style_gap(Gutter::Row, v),
            |v| n.set_style_gap_percent(Gutter::Row, v),
            || {},
        );
    }
    if let Some(g) = s.column_gap {
        apply_dim(
            g,
            |v| n.set_style_gap(Gutter::Column, v),
            |v| n.set_style_gap_percent(Gutter::Column, v),
            || {},
        );
    }
    if let Some(j) = s.justify_content {
        n.set_style_justify_content(match j {
            JustifyContent::FlexStart => YJustify::FlexStart,
            JustifyContent::FlexEnd => YJustify::FlexEnd,
            JustifyContent::Center => YJustify::Center,
            JustifyContent::SpaceBetween => YJustify::SpaceBetween,
            JustifyContent::SpaceAround => YJustify::SpaceAround,
            JustifyContent::SpaceEvenly => YJustify::SpaceEvenly,
        });
    }
    if let Some(a) = s.align_items {
        n.set_style_align_items(match a {
            AlignItems::Auto => YAlign::Auto,
            AlignItems::Stretch => YAlign::Stretch,
            AlignItems::FlexStart => YAlign::FlexStart,
            AlignItems::FlexEnd => YAlign::FlexEnd,
            AlignItems::Center => YAlign::Center,
            AlignItems::Baseline => YAlign::Baseline,
        });
    }
    if let Some(a) = s.align_self {
        n.set_style_align_self(match a {
            AlignSelf::Auto => YAlign::Auto,
            AlignSelf::Stretch => YAlign::Stretch,
            AlignSelf::FlexStart => YAlign::FlexStart,
            AlignSelf::FlexEnd => YAlign::FlexEnd,
            AlignSelf::Center => YAlign::Center,
            AlignSelf::Baseline => YAlign::Baseline,
        });
    }
    if let Some(a) = s.align_content {
        n.set_style_align_content(match a {
            AlignContent::Auto => YAlign::Auto,
            AlignContent::Stretch => YAlign::Stretch,
            AlignContent::FlexStart => YAlign::FlexStart,
            AlignContent::FlexEnd => YAlign::FlexEnd,
            AlignContent::Center => YAlign::Center,
            AlignContent::SpaceBetween => YAlign::SpaceBetween,
            AlignContent::SpaceAround => YAlign::SpaceAround,
            AlignContent::SpaceEvenly => YAlign::SpaceEvenly,
        });
    }
    // `display: none` removes the node from layout. `display: contents`
    // collapses the node so its children participate in the parent's
    // layout (yoga handles both natively as DISPLAY_NONE / DISPLAY_CONTENTS).
    match s.display {
        Some(Display::None) => n.set_style_display(yoga::Display::None),
        Some(Display::Contents) => n.set_style_display(yoga::Display::Contents),
        _ => {}
    }

    // Position handling (absolute / relative). `static` (the default)
    // ignores left/top/right/bottom (per CSS spec).
    let is_static = matches!(s.position, None | Some(crate::css::Position::Static));
    if matches!(s.position, Some(crate::css::Position::Absolute)) {
        n.set_style_position_type(PositionType::Absolute);
    }
    if !is_static {
        apply_edge_dim(s.top, Edge::Top, n, EdgeKind::Position);
        apply_edge_dim(s.right, Edge::Right, n, EdgeKind::Position);
        apply_edge_dim(s.bottom, Edge::Bottom, n, EdgeKind::Position);
        apply_edge_dim(s.left, Edge::Left, n, EdgeKind::Position);
    }

    // `box-sizing`: yoga-rs's intrinsic default matches JS satori's
    // `border-box` default, so we only need to call the setter when
    // the style explicitly opts in to either variant. (CSS's UA
    // default is `content-box`, but JS satori overrides to
    // `border-box` regardless of the spec — see
    // `compute.ts` in upstream.)
    if let Some(bs) = s.box_sizing {
        n.set_style_box_sizing(match bs {
            CssBoxSizing::BorderBox => YBoxSizing::BorderBox,
            CssBoxSizing::ContentBox => YBoxSizing::ContentBox,
        });
    }

    // Aspect-ratio hint (used by `<img>` to keep the box ratio when only
    // one of width/height is constrained — port of
    // `node.setAspectRatio(1 / r)` in JS satori's `compute.ts`).
    if let Some(ar) = s._aspect_ratio {
        n.set_style_aspect_ratio(ar);
    }
}

#[derive(Copy, Clone)]
enum EdgeKind {
    Margin,
    Padding,
    Position,
}

/// Dispatch an `Option<Dim>` to one of the per-`EdgeKind` setter
/// trios. Centralised so adding new edge-shaped props (e.g. `inset-*`)
/// doesn't fan out into another N-way switch.
fn apply_edge_dim(d: Option<Dim>, edge: Edge, n: &NodeRef, kind: EdgeKind) {
    let Some(d) = d else { return };
    match (kind, d) {
        (EdgeKind::Margin, Dim::Px(v)) => n.set_style_margin(edge, v),
        (EdgeKind::Margin, Dim::Percent(v)) => n.set_style_margin_percent(edge, v),
        (EdgeKind::Margin, Dim::Auto) => n.set_style_margin_auto(edge),
        (EdgeKind::Padding, Dim::Px(v)) => n.set_style_padding(edge, v),
        (EdgeKind::Padding, Dim::Percent(v)) => n.set_style_padding_percent(edge, v),
        // `padding: auto` is not a CSS value; if upstream gave us one,
        // ignore it (yoga's default is 0 which matches CSS).
        (EdgeKind::Padding, Dim::Auto) => {}
        (EdgeKind::Position, Dim::Px(v)) => n.set_style_position(edge, v),
        (EdgeKind::Position, Dim::Percent(v)) => n.set_style_position_percent(edge, v),
        (EdgeKind::Position, Dim::Auto) => n.set_style_position_auto(edge),
    }
}

/// Run yoga layout on a constructed tree and return a flat laid-out list
/// in pre-order (parent before children).
///
/// The `width`/`height` are the outer SVG viewport.
pub fn lay_out(tree: Built, width: Option<u32>, height: Option<u32>) -> LayoutTree {
    let root_w = width.map(|w| w as f32).unwrap_or(f32::NAN);
    let root_h = height.map(|h| h as f32).unwrap_or(f32::NAN);

    // Wrap in a synthetic root that mirrors `getRootNode` in `src/satori.ts`.
    let root = NodeRef::new();
    if let Some(w) = width { root.set_style_width(w as f32); }
    if let Some(h) = height { root.set_style_height(h as f32); }
    root.set_style_flex_direction(YFlexDirection::Row);
    root.set_style_flex_wrap(Wrap::Wrap);
    root.set_style_align_content(YAlign::Auto);
    root.set_style_align_items(YAlign::FlexStart);
    root.set_style_justify_content(YJustify::FlexStart);
    root.set_style_overflow(Overflow::Hidden);

    insert_into(&root, &tree.yoga, &tree.children);

    root.calculate_layout(root_w, root_h, Direction::LTR);

    let computed_root_w = root.layout_width();
    let computed_root_h = root.layout_height();

    // Walk the tree and collect laid-out nodes in pre-order.
    let mut nodes: Vec<LaidOut> = Vec::new();
    visit(&tree, None, 0.0, 0.0, &mut nodes);

    LayoutTree {
        nodes,
        root_width: if computed_root_w.is_finite() {
            computed_root_w
        } else {
            width.unwrap_or(0) as f32
        },
        root_height: if computed_root_h.is_finite() {
            computed_root_h
        } else {
            height.unwrap_or(0) as f32
        },
    }
}

fn insert_into(parent: &NodeRef, this: &NodeRef, children: &[Built]) {
    let count = parent.child_count();
    parent.insert_child(this, count);
    for c in children {
        insert_into(this, &c.yoga, &c.children);
    }
}

fn visit(b: &Built, parent_idx: Option<usize>, parent_left: f32, parent_top: f32, out: &mut Vec<LaidOut>) {
    let left = parent_left + b.yoga.layout_left();
    let top = parent_top + b.yoga.layout_top();
    let width = b.yoga.layout_width();
    let height = b.yoga.layout_height();

    out.push(LaidOut {
        id: b.id.clone(),
        tag: b.tag.clone(),
        left,
        top,
        width,
        height,
        style: b.style.clone(),
        parent: parent_idx,
        text: b.text.clone(),
        fonts: b.fonts.clone(),
    });
    let this_idx = out.len() - 1;
    for c in &b.children {
        visit(c, Some(this_idx), left, top, out);
    }
}
