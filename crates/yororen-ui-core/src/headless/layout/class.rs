//! `LayoutClass` — the parsed representation of a CSS-like
//! class string (`"flex gap-md p-md"`).
//!
//! Each variant corresponds to one class token. The
//! `apply` method expands the class into a chain of
//! gpui builder calls on a `Stateful<Div>`.
//!
//! Class strings are parsed at compile time by
//! `yororen-ui-xml-macro`'s `classes!` proc-macro and by
//! the XML `class` attribute handler. Unknown tokens are
//! rejected at compile time.

use gpui::{App, Div, Stateful, StatefulInteractiveElement, Styled};
use crate::theme::ActiveTheme;

use super::types::{AlignItems, Inset, JustifyContent, Length, Spacing};

/// One parsed class token. Variants mirror the Tailwind-like
/// classes described in `tmp/layout-system-plan.md` §4.1.
#[derive(Clone, Debug, PartialEq)]
pub enum LayoutClass {
    // ----- flex / layout -----
    Flex,
    FlexCol,
    FlexRow,
    FlexWrap,
    Flex1,
    // ----- align-items -----
    ItemsStart,
    ItemsEnd,
    ItemsCenter,
    ItemsBaseline,
    ItemsStretch,
    // ----- justify-content -----
    JustifyStart,
    JustifyEnd,
    JustifyCenter,
    JustifyBetween,
    JustifyAround,
    JustifyEvenly,
    // ----- gap (with value) -----
    Gap(Spacing),
    // ----- padding (with value) -----
    P(Inset),
    Px(Spacing),
    Py(Spacing),
    // ----- margin (with value) -----
    M(Inset),
    Mx(Inset),
    My(Inset),
    // ----- size -----
    WFull,
    HFull,
    SizeFull,
    // ----- position -----
    Relative,
    Absolute,
    Top0,
    Right0,
    Bottom0,
    Left0,
    Inset0,
    // ----- overflow -----
    OverflowHidden,
    OverflowScroll,
    // ----- border -----
    Border,
    Border1,
    // ----- radius -----
    Rounded,
    RoundedMd,
    RoundedLg,
    // ----- shadow -----
    ShadowMd,
    ShadowLg,
}

impl LayoutClass {
    /// Apply this class to a `Stateful<Div>`, expanding it
    /// into the appropriate gpui builder call(s).
    ///
    /// `cx` is used to resolve theme tokens (e.g. `gap-md`
    /// reads `tokens.spacing.gap_3`).
    pub fn apply(&self, cx: &App, el: Stateful<Div>) -> Stateful<Div> {
        match self {
            LayoutClass::Flex => el.flex(),
            LayoutClass::FlexCol => el.flex_col(),
            LayoutClass::FlexRow => el.flex_row(),
            LayoutClass::FlexWrap => el.flex_wrap(),
            LayoutClass::Flex1 => el.flex_1(),
            LayoutClass::ItemsStart => el.items_start(),
            LayoutClass::ItemsEnd => el.items_end(),
            LayoutClass::ItemsCenter => el.items_center(),
            LayoutClass::ItemsBaseline => el.items_baseline(),
            LayoutClass::ItemsStretch => el, // CSS default
            LayoutClass::JustifyStart => el.justify_start(),
            LayoutClass::JustifyEnd => el.justify_end(),
            LayoutClass::JustifyCenter => el.justify_center(),
            LayoutClass::JustifyBetween => el.justify_between(),
            LayoutClass::JustifyAround => el.justify_around(),
            LayoutClass::JustifyEvenly => el.justify_evenly(),
            LayoutClass::Gap(g) => el.gap(g.to_pixels(cx.theme())),
            LayoutClass::P(p) => el.p(p.to_pixels(cx.theme())),
            LayoutClass::Px(x) => el.px(x.to_pixels(cx.theme())),
            LayoutClass::Py(y) => el.py(y.to_pixels(cx.theme())),
            LayoutClass::M(m) => el.m(m.to_pixels(cx.theme())),
            LayoutClass::Mx(x) => el.mx(x.to_pixels(cx.theme())),
            LayoutClass::My(y) => el.my(y.to_pixels(cx.theme())),
            LayoutClass::WFull => el.w_full(),
            LayoutClass::HFull => el.h_full(),
            LayoutClass::SizeFull => el.size_full(),
            LayoutClass::Relative => el.relative(),
            LayoutClass::Absolute => el.absolute(),
            LayoutClass::Top0 => el.top_0(),
            LayoutClass::Right0 => el.right_0(),
            LayoutClass::Bottom0 => el.bottom_0(),
            LayoutClass::Left0 => el.left_0(),
            LayoutClass::Inset0 => el.inset_0(),
            LayoutClass::OverflowHidden => el.overflow_hidden(),
            LayoutClass::OverflowScroll => el.overflow_scroll(),
            LayoutClass::Border => el.border_1(),
            LayoutClass::Border1 => el.border_1(),
            LayoutClass::Rounded => el.rounded_md(),
            LayoutClass::RoundedMd => el.rounded_md(),
            LayoutClass::RoundedLg => el.rounded_lg(),
            LayoutClass::ShadowMd => el.shadow_md(),
            LayoutClass::ShadowLg => el.shadow_lg(),
        }
    }
}

/// Apply a list of classes to a `Stateful<Div>` in order.
/// Unknown classes are silently ignored (the parser should
/// have caught them at compile time).
pub fn apply_all(cx: &App, el: Stateful<Div>, classes: &[LayoutClass]) -> Stateful<Div> {
    let mut el = el;
    for c in classes {
        el = c.apply(cx, el);
    }
    el
}

/// Convenience: build a `Vec<LayoutClass>` from an iterator.
pub fn classes_vec<I>(iter: I) -> Vec<LayoutClass>
where
    I: IntoIterator<Item = LayoutClass>,
{
    iter.into_iter().collect()
}

// Re-export the named-variant tables so the parser in
// `yororen-ui-xml/src/class.rs` can match against them
// without duplicating the variant lists.
pub const SPACING_NAMED: &[(&str, Spacing)] = &[
    ("xs", Spacing::Xs),
    ("sm", Spacing::Sm),
    ("md", Spacing::Md),
    ("lg", Spacing::Lg),
    ("xl", Spacing::Xl),
    ("xxl", Spacing::Xxl),
];

pub const INSET_NAMED: &[(&str, Inset)] = &[
    ("xs", Inset::Xs),
    ("sm", Inset::Sm),
    ("md", Inset::Md),
    ("lg", Inset::Lg),
    ("xl", Inset::Xl),
];

#[allow(dead_code)]
pub const ALIGN_ITEMS_NAMED: &[(&str, AlignItems)] = &[
    ("start", AlignItems::Start),
    ("end", AlignItems::End),
    ("center", AlignItems::Center),
    ("baseline", AlignItems::Baseline),
    ("stretch", AlignItems::Stretch),
];

#[allow(dead_code)]
pub const JUSTIFY_CONTENT_NAMED: &[(&str, JustifyContent)] = &[
    ("start", JustifyContent::Start),
    ("end", JustifyContent::End),
    ("center", JustifyContent::Center),
    ("between", JustifyContent::Between),
    ("around", JustifyContent::Around),
    ("evenly", JustifyContent::Evenly),
];

#[allow(dead_code)]
pub const LENGTH_NAMED: &[(&str, Length)] = &[
    ("full", Length::Full),
    ("fit", Length::Fit),
    ("auto", Length::Auto),
];
