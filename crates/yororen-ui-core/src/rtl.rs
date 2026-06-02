//! RTL (right-to-left) layout helpers.
//!
//! GPUI itself doesn't provide a global layout direction flag for style resolution,
//! so this module provides small helpers to flip common "start/end" concepts.

use gpui::{AbsoluteLength, DefiniteLength, Length, Pixels, relative};

use crate::i18n::TextDirection;

/// Convert a logical *start* alignment into a concrete GPUI `TextAlign`.
pub fn text_align_start(direction: TextDirection) -> gpui::TextAlign {
    match direction {
        TextDirection::Ltr => gpui::TextAlign::Left,
        TextDirection::Rtl => gpui::TextAlign::Right,
    }
}

/// Convert a logical *end* alignment into a concrete GPUI `TextAlign`.
pub fn text_align_end(direction: TextDirection) -> gpui::TextAlign {
    match direction {
        TextDirection::Ltr => gpui::TextAlign::Right,
        TextDirection::Rtl => gpui::TextAlign::Left,
    }
}

/// Map a logical arrow direction for UI affordances.
///
/// In RTL, "back" should generally point right and "forward" should point left.
pub fn flip_left_right<T>(direction: TextDirection, left: T, right: T) -> T {
    match direction {
        TextDirection::Ltr => left,
        TextDirection::Rtl => right,
    }
}

/// Return `row` or `row-reverse` based on layout direction.
pub fn is_row_reverse(direction: TextDirection) -> bool {
    direction.is_rtl()
}

/// Return the appropriate [`FlexDirection`] for a horizontal flex container.
///
/// Use this for any row that contains direction-sensitive children
/// (e.g. icon + label, leading + trailing).
pub fn flex_row(direction: TextDirection) -> gpui::FlexDirection {
    if direction.is_rtl() {
        gpui::FlexDirection::RowReverse
    } else {
        gpui::FlexDirection::Row
    }
}

/// Logical *start* justification for flex containers.
///
/// Returns [`JustifyContent::FlexStart`], which respects `RowReverse`.
pub fn justify_start() -> gpui::JustifyContent {
    gpui::JustifyContent::FlexStart
}

/// Logical *end* justification for flex containers.
///
/// Returns [`JustifyContent::FlexEnd`], which respects `RowReverse`.
pub fn justify_end() -> gpui::JustifyContent {
    gpui::JustifyContent::FlexEnd
}

// ---------------------------------------------------------------------------
// Padding helpers
// ---------------------------------------------------------------------------

/// Set padding on the logical *start* side.
pub fn padding_start(
    style: &mut gpui::StyleRefinement,
    direction: TextDirection,
    value: impl Into<DefiniteLength>,
) {
    let v = Some(value.into());
    if direction.is_rtl() {
        style.padding.right = v;
    } else {
        style.padding.left = v;
    }
}

/// Set padding on the logical *end* side.
pub fn padding_end(
    style: &mut gpui::StyleRefinement,
    direction: TextDirection,
    value: impl Into<DefiniteLength>,
) {
    let v = Some(value.into());
    if direction.is_rtl() {
        style.padding.left = v;
    } else {
        style.padding.right = v;
    }
}

// ---------------------------------------------------------------------------
// Margin helpers
// ---------------------------------------------------------------------------

/// Set margin on the logical *start* side.
pub fn margin_start(
    style: &mut gpui::StyleRefinement,
    direction: TextDirection,
    value: impl Into<Length>,
) {
    let v = Some(value.into());
    if direction.is_rtl() {
        style.margin.right = v;
    } else {
        style.margin.left = v;
    }
}

/// Set margin on the logical *end* side.
pub fn margin_end(
    style: &mut gpui::StyleRefinement,
    direction: TextDirection,
    value: impl Into<Length>,
) {
    let v = Some(value.into());
    if direction.is_rtl() {
        style.margin.left = v;
    } else {
        style.margin.right = v;
    }
}

// ---------------------------------------------------------------------------
// Inset helpers (absolute positioning)
// ---------------------------------------------------------------------------

/// Place an absolutely positioned element at the logical start.
pub fn place_start(style: &mut gpui::StyleRefinement, direction: TextDirection, value: Pixels) {
    if direction.is_rtl() {
        style.inset.right = Some(Length::from(value));
    } else {
        style.inset.left = Some(Length::from(value));
    }
}

/// Place an absolutely positioned element at the logical start with 0px.
pub fn place_start_0(style: &mut gpui::StyleRefinement, direction: TextDirection) {
    if direction.is_rtl() {
        style.inset.right = Some(relative(0.).into());
    } else {
        style.inset.left = Some(relative(0.).into());
    }
}

/// Place an absolutely positioned element at the logical end.
pub fn place_end(style: &mut gpui::StyleRefinement, direction: TextDirection, value: Pixels) {
    if direction.is_rtl() {
        style.inset.left = Some(Length::from(value));
    } else {
        style.inset.right = Some(Length::from(value));
    }
}

/// Place an absolutely positioned element at the logical end with 0px.
pub fn place_end_0(style: &mut gpui::StyleRefinement, direction: TextDirection) {
    if direction.is_rtl() {
        style.inset.left = Some(relative(0.).into());
    } else {
        style.inset.right = Some(relative(0.).into());
    }
}

// ---------------------------------------------------------------------------
// Corner radius helpers
// ---------------------------------------------------------------------------

/// Set corner radius on the logical *start* side (top-start and bottom-start).
pub fn corner_radius_start(
    style: &mut gpui::StyleRefinement,
    direction: TextDirection,
    value: impl Into<AbsoluteLength>,
) {
    let v = Some(value.into());
    if direction.is_rtl() {
        style.corner_radii.top_right = v;
        style.corner_radii.bottom_right = v;
    } else {
        style.corner_radii.top_left = v;
        style.corner_radii.bottom_left = v;
    }
}

/// Set corner radius on the logical *end* side (top-end and bottom-end).
pub fn corner_radius_end(
    style: &mut gpui::StyleRefinement,
    direction: TextDirection,
    value: impl Into<AbsoluteLength>,
) {
    let v = Some(value.into());
    if direction.is_rtl() {
        style.corner_radii.top_left = v;
        style.corner_radii.bottom_left = v;
    } else {
        style.corner_radii.top_right = v;
        style.corner_radii.bottom_right = v;
    }
}

/// Remove corner radius on the logical *start* side.
pub fn corner_radius_start_none(style: &mut gpui::StyleRefinement, direction: TextDirection) {
    if direction.is_rtl() {
        style.corner_radii.top_right = Some(gpui::px(0.).into());
        style.corner_radii.bottom_right = Some(gpui::px(0.).into());
    } else {
        style.corner_radii.top_left = Some(gpui::px(0.).into());
        style.corner_radii.bottom_left = Some(gpui::px(0.).into());
    }
}

/// Remove corner radius on the logical *end* side.
pub fn corner_radius_end_none(style: &mut gpui::StyleRefinement, direction: TextDirection) {
    if direction.is_rtl() {
        style.corner_radii.top_left = Some(gpui::px(0.).into());
        style.corner_radii.bottom_left = Some(gpui::px(0.).into());
    } else {
        style.corner_radii.top_right = Some(gpui::px(0.).into());
        style.corner_radii.bottom_right = Some(gpui::px(0.).into());
    }
}

#[cfg(test)]
mod tests {
    use gpui::px;

    use crate::i18n::TextDirection;
    use crate::rtl;

    #[test]
    fn text_align_start_returns_left_for_ltr() {
        assert_eq!(
            rtl::text_align_start(TextDirection::Ltr),
            gpui::TextAlign::Left
        );
    }

    #[test]
    fn text_align_start_returns_right_for_rtl() {
        assert_eq!(
            rtl::text_align_start(TextDirection::Rtl),
            gpui::TextAlign::Right
        );
    }

    #[test]
    fn text_align_end_returns_right_for_ltr() {
        assert_eq!(
            rtl::text_align_end(TextDirection::Ltr),
            gpui::TextAlign::Right
        );
    }

    #[test]
    fn text_align_end_returns_left_for_rtl() {
        assert_eq!(
            rtl::text_align_end(TextDirection::Rtl),
            gpui::TextAlign::Left
        );
    }

    #[test]
    fn flex_row_returns_row_for_ltr() {
        assert_eq!(rtl::flex_row(TextDirection::Ltr), gpui::FlexDirection::Row);
    }

    #[test]
    fn flex_row_returns_row_reverse_for_rtl() {
        assert_eq!(
            rtl::flex_row(TextDirection::Rtl),
            gpui::FlexDirection::RowReverse
        );
    }

    #[test]
    fn flip_left_right_returns_correct_value() {
        assert_eq!(
            rtl::flip_left_right(TextDirection::Ltr, "left", "right"),
            "left"
        );
        assert_eq!(
            rtl::flip_left_right(TextDirection::Rtl, "left", "right"),
            "right"
        );
    }

    #[test]
    fn is_row_reverse_matches_direction() {
        assert!(!rtl::is_row_reverse(TextDirection::Ltr));
        assert!(rtl::is_row_reverse(TextDirection::Rtl));
    }

    #[test]
    fn padding_start_sets_left_for_ltr() {
        let mut style = gpui::StyleRefinement::default();
        rtl::padding_start(&mut style, TextDirection::Ltr, px(8.));
        assert!(style.padding.left.is_some());
        assert!(style.padding.right.is_none());
    }

    #[test]
    fn padding_start_sets_right_for_rtl() {
        let mut style = gpui::StyleRefinement::default();
        rtl::padding_start(&mut style, TextDirection::Rtl, px(8.));
        assert!(style.padding.right.is_some());
        assert!(style.padding.left.is_none());
    }

    #[test]
    fn padding_end_sets_right_for_ltr() {
        let mut style = gpui::StyleRefinement::default();
        rtl::padding_end(&mut style, TextDirection::Ltr, px(8.));
        assert!(style.padding.right.is_some());
        assert!(style.padding.left.is_none());
    }

    #[test]
    fn padding_end_sets_left_for_rtl() {
        let mut style = gpui::StyleRefinement::default();
        rtl::padding_end(&mut style, TextDirection::Rtl, px(8.));
        assert!(style.padding.left.is_some());
        assert!(style.padding.right.is_none());
    }

    #[test]
    fn margin_start_sets_left_for_ltr() {
        let mut style = gpui::StyleRefinement::default();
        rtl::margin_start(&mut style, TextDirection::Ltr, px(8.));
        assert!(style.margin.left.is_some());
        assert!(style.margin.right.is_none());
    }

    #[test]
    fn margin_start_sets_right_for_rtl() {
        let mut style = gpui::StyleRefinement::default();
        rtl::margin_start(&mut style, TextDirection::Rtl, px(8.));
        assert!(style.margin.right.is_some());
        assert!(style.margin.left.is_none());
    }

    #[test]
    fn inset_start_sets_left_for_ltr() {
        let mut style = gpui::StyleRefinement::default();
        rtl::place_start(&mut style, TextDirection::Ltr, px(4.));
        assert!(style.inset.left.is_some());
        assert!(style.inset.right.is_none());
    }

    #[test]
    fn inset_start_sets_right_for_rtl() {
        let mut style = gpui::StyleRefinement::default();
        rtl::place_start(&mut style, TextDirection::Rtl, px(4.));
        assert!(style.inset.right.is_some());
        assert!(style.inset.left.is_none());
    }

    #[test]
    fn inset_end_sets_right_for_ltr() {
        let mut style = gpui::StyleRefinement::default();
        rtl::place_end(&mut style, TextDirection::Ltr, px(4.));
        assert!(style.inset.right.is_some());
        assert!(style.inset.left.is_none());
    }

    #[test]
    fn inset_end_sets_left_for_rtl() {
        let mut style = gpui::StyleRefinement::default();
        rtl::place_end(&mut style, TextDirection::Rtl, px(4.));
        assert!(style.inset.left.is_some());
        assert!(style.inset.right.is_none());
    }

    #[test]
    fn corner_radius_start_sets_top_left_for_ltr() {
        let mut style = gpui::StyleRefinement::default();
        rtl::corner_radius_start(&mut style, TextDirection::Ltr, px(4.));
        assert!(style.corner_radii.top_left.is_some());
        assert!(style.corner_radii.top_right.is_none());
    }

    #[test]
    fn corner_radius_start_sets_top_right_for_rtl() {
        let mut style = gpui::StyleRefinement::default();
        rtl::corner_radius_start(&mut style, TextDirection::Rtl, px(4.));
        assert!(style.corner_radii.top_right.is_some());
        assert!(style.corner_radii.top_left.is_none());
    }

    #[test]
    fn corner_radius_end_none_sets_top_right_to_zero_for_ltr() {
        let mut style = gpui::StyleRefinement::default();
        rtl::corner_radius_end_none(&mut style, TextDirection::Ltr);
        let zero = Some(gpui::px(0.).into());
        assert_eq!(style.corner_radii.top_right, zero);
        assert_eq!(style.corner_radii.bottom_right, zero);
    }

    #[test]
    fn corner_radius_end_none_sets_top_left_to_zero_for_rtl() {
        let mut style = gpui::StyleRefinement::default();
        rtl::corner_radius_end_none(&mut style, TextDirection::Rtl);
        let zero = Some(gpui::px(0.).into());
        assert_eq!(style.corner_radii.top_left, zero);
        assert_eq!(style.corner_radii.bottom_left, zero);
    }

    #[test]
    fn justify_start_returns_flex_start() {
        assert_eq!(rtl::justify_start(), gpui::JustifyContent::FlexStart);
    }

    #[test]
    fn justify_end_returns_flex_end() {
        assert_eq!(rtl::justify_end(), gpui::JustifyContent::FlexEnd);
    }
}
