//! Layout type system — `Spacing`, `Inset`, `Length`, `AlignItems`,
//! `JustifyContent`.
//!
//! These enums are the shared vocabulary between the imperative
//! layout API and the future XML class parser. Each named variant
//! resolves to a theme token path; `Px` / `Rem` variants resolve
//! without theme access.
//!
//! ## Token paths
//!
//! | Enum           | Variant | Token path              | Fallback px |
//! |----------------|---------|-------------------------|-------------|
//! | `Spacing`      | Xs      | `tokens.spacing.gap_1`  | 4           |
//! | `Spacing`      | Sm      | `tokens.spacing.gap_2`  | 8           |
//! | `Spacing`      | Md      | `tokens.spacing.gap_3`  | 12          |
//! | `Spacing`      | Lg      | `tokens.spacing.gap_4`  | 16          |
//! | `Spacing`      | Xl      | `tokens.spacing.gap_5`  | 20          |
//! | `Spacing`      | Xxl     | `tokens.spacing.gap_6`  | 24          |
//! | `Inset`        | Xs      | `tokens.spacing.inset_xs`| 4           |
//! | `Inset`        | Sm      | `tokens.spacing.inset_sm`| 8           |
//! | `Inset`        | Md      | `tokens.spacing.inset_md`| 12          |
//! | `Inset`        | Lg      | `tokens.spacing.inset_lg`| 16          |
//! | `Inset`        | Xl      | `tokens.spacing.inset_xl`| 24          |
//!
//! `Spacing::Rem(v)` resolves to `v * 16.0` px — gpui has no rem
//! concept; the 16 px base is a documented convention.
//!
//! `Length::Pct(v)` resolves to `gpui::relative(v / 100.0)` —
//! a 0-100 percentage scaled to 0.0-1.0.

use gpui::{Pixels, Styled, px, relative};

use crate::theme::Theme;

/// Flex / grid gap spacing. Maps to `tokens.spacing.gap_N` tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Spacing {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    Xxl,
    /// Raw pixel value — no theme lookup.
    Px(f32),
    /// Rem-like value — multiplied by 16.0 to get pixels.
    Rem(f32),
}

impl Spacing {
    /// Resolve to pixels using the given theme's token paths.
    /// Falls back to documented defaults if a path is absent.
    pub fn to_pixels(&self, theme: &Theme) -> Pixels {
        match self {
            Spacing::Xs => px(theme.get_number("tokens.spacing.gap_1").unwrap_or(4.0) as f32),
            Spacing::Sm => px(theme.get_number("tokens.spacing.gap_2").unwrap_or(8.0) as f32),
            Spacing::Md => px(theme.get_number("tokens.spacing.gap_3").unwrap_or(12.0) as f32),
            Spacing::Lg => px(theme.get_number("tokens.spacing.gap_4").unwrap_or(16.0) as f32),
            Spacing::Xl => px(theme.get_number("tokens.spacing.gap_5").unwrap_or(20.0) as f32),
            Spacing::Xxl => px(theme.get_number("tokens.spacing.gap_6").unwrap_or(24.0) as f32),
            Spacing::Px(v) => px(*v),
            Spacing::Rem(v) => px(*v * 16.0),
        }
    }
}

/// Convenience alias — `Gap` is `Spacing` used in the gap context.
pub type Gap = Spacing;

impl From<f32> for Spacing {
    fn from(v: f32) -> Self {
        Spacing::Px(v)
    }
}

impl From<f64> for Spacing {
    fn from(v: f64) -> Self {
        Spacing::Px(v as f32)
    }
}

/// Padding / margin inset. Maps to `tokens.spacing.inset_X` tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Inset {
    Xs,
    Sm,
    Md,
    Lg,
    Xl,
    /// Raw pixel value — no theme lookup.
    Px(f32),
}

impl Inset {
    /// Resolve to pixels using the given theme's token paths.
    /// Falls back to documented defaults if a path is absent.
    pub fn to_pixels(&self, theme: &Theme) -> Pixels {
        match self {
            Inset::Xs => px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(4.0) as f32),
            Inset::Sm => px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(8.0) as f32),
            Inset::Md => px(theme.get_number("tokens.spacing.inset_md").unwrap_or(12.0) as f32),
            Inset::Lg => px(theme.get_number("tokens.spacing.inset_lg").unwrap_or(16.0) as f32),
            Inset::Xl => px(theme.get_number("tokens.spacing.inset_xl").unwrap_or(24.0) as f32),
            Inset::Px(v) => px(*v),
        }
    }
}

impl From<f32> for Inset {
    fn from(v: f32) -> Self {
        Inset::Px(v)
    }
}

impl From<f64> for Inset {
    fn from(v: f64) -> Self {
        Inset::Px(v as f32)
    }
}

/// Length for width / height. Does NOT need theme access.
///
/// | Variant  | Maps to                              |
/// |----------|--------------------------------------|
/// | `Full`   | `.w_full()` / `.h_full()`            |
/// | `Fit`    | no-op (content determines size)      |
/// | `Auto`   | no-op                                |
/// | `Px(v)`  | `.w(px(v))`                          |
/// | `Rem(v)` | `.w(px(v * 16.0))`                   |
/// | `Pct(v)` | `.w(relative(v / 100.0))`            |
///
/// `Fit` and `Auto` are both no-ops because gpui has no
/// `fit-content` concept — the element's size is determined by
/// its content when no explicit size is set.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Length {
    Full,
    Fit,
    Auto,
    Px(f32),
    Rem(f32),
    /// Percentage (0-100). `Pct(50.0)` → 50% of the parent.
    Pct(f32),
}

/// Apply a `Length` as width to any `Styled` element.
pub fn apply_width<S: Styled>(el: S, w: Length) -> S {
    match w {
        Length::Full => el.w_full(),
        Length::Fit => el,
        Length::Auto => el,
        Length::Px(v) => el.w(px(v)),
        Length::Rem(v) => el.w(px(v * 16.0)),
        Length::Pct(v) => el.w(relative(v / 100.0)),
    }
}

/// Apply a `Length` as height to any `Styled` element.
pub fn apply_height<S: Styled>(el: S, h: Length) -> S {
    match h {
        Length::Full => el.h_full(),
        Length::Fit => el,
        Length::Auto => el,
        Length::Px(v) => el.h(px(v)),
        Length::Rem(v) => el.h(px(v * 16.0)),
        Length::Pct(v) => el.h(relative(v / 100.0)),
    }
}

/// Flexbox `align-items` axis.
///
/// `Stretch` is the CSS default — gpui-ce 0.3.3 has no
/// `items_stretch()` builder method, so this variant is a
/// no-op (leaving `align_items` unset yields the default
/// stretch behaviour).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Baseline,
    Stretch,
}

/// Flexbox `justify-content` axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    Between,
    Around,
    Evenly,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn theme_with_spacing() -> Theme {
        Theme::from_value(json!({
            "tokens": {
                "spacing": {
                    "gap_1": 4, "gap_2": 8, "gap_3": 12,
                    "gap_4": 16, "gap_5": 20, "gap_6": 24,
                    "inset_xs": 4, "inset_sm": 8, "inset_md": 12,
                    "inset_lg": 16, "inset_xl": 24
                }
            }
        }))
    }

    // ----- Spacing -----

    #[test]
    fn spacing_named_variants_resolve_from_theme() {
        let t = theme_with_spacing();
        assert_eq!(Spacing::Xs.to_pixels(&t), px(4.0));
        assert_eq!(Spacing::Sm.to_pixels(&t), px(8.0));
        assert_eq!(Spacing::Md.to_pixels(&t), px(12.0));
        assert_eq!(Spacing::Lg.to_pixels(&t), px(16.0));
        assert_eq!(Spacing::Xl.to_pixels(&t), px(20.0));
        assert_eq!(Spacing::Xxl.to_pixels(&t), px(24.0));
    }

    #[test]
    fn spacing_falls_back_when_theme_missing_tokens() {
        let t = Theme::new();
        assert_eq!(Spacing::Xs.to_pixels(&t), px(4.0));
        assert_eq!(Spacing::Sm.to_pixels(&t), px(8.0));
        assert_eq!(Spacing::Md.to_pixels(&t), px(12.0));
        assert_eq!(Spacing::Lg.to_pixels(&t), px(16.0));
        assert_eq!(Spacing::Xl.to_pixels(&t), px(20.0));
        assert_eq!(Spacing::Xxl.to_pixels(&t), px(24.0));
    }

    #[test]
    fn spacing_px_variant_is_raw() {
        let t = Theme::new();
        assert_eq!(Spacing::Px(10.0).to_pixels(&t), px(10.0));
        assert_eq!(Spacing::Px(0.0).to_pixels(&t), px(0.0));
        assert_eq!(Spacing::Px(-5.0).to_pixels(&t), px(-5.0));
    }

    #[test]
    fn spacing_rem_multiplies_by_16() {
        let t = Theme::new();
        assert_eq!(Spacing::Rem(1.0).to_pixels(&t), px(16.0));
        assert_eq!(Spacing::Rem(0.5).to_pixels(&t), px(8.0));
        assert_eq!(Spacing::Rem(2.0).to_pixels(&t), px(32.0));
    }

    #[test]
    fn spacing_from_f32() {
        let s: Spacing = 8.0_f32.into();
        assert_eq!(s, Spacing::Px(8.0));
    }

    #[test]
    fn spacing_from_f64() {
        let s: Spacing = 12.0_f64.into();
        assert_eq!(s, Spacing::Px(12.0));
    }

    #[test]
    fn gap_is_spacing_alias() {
        let g: Gap = Spacing::Md;
        assert_eq!(g, Spacing::Md);
    }

    // ----- Inset -----

    #[test]
    fn inset_named_variants_resolve_from_theme() {
        let t = theme_with_spacing();
        assert_eq!(Inset::Xs.to_pixels(&t), px(4.0));
        assert_eq!(Inset::Sm.to_pixels(&t), px(8.0));
        assert_eq!(Inset::Md.to_pixels(&t), px(12.0));
        assert_eq!(Inset::Lg.to_pixels(&t), px(16.0));
        assert_eq!(Inset::Xl.to_pixels(&t), px(24.0));
    }

    #[test]
    fn inset_falls_back_when_theme_missing() {
        let t = Theme::new();
        assert_eq!(Inset::Xs.to_pixels(&t), px(4.0));
        assert_eq!(Inset::Sm.to_pixels(&t), px(8.0));
        assert_eq!(Inset::Md.to_pixels(&t), px(12.0));
        assert_eq!(Inset::Lg.to_pixels(&t), px(16.0));
        assert_eq!(Inset::Xl.to_pixels(&t), px(24.0));
    }

    #[test]
    fn inset_px_variant_is_raw() {
        let t = Theme::new();
        assert_eq!(Inset::Px(15.0).to_pixels(&t), px(15.0));
    }

    #[test]
    fn inset_from_f32() {
        let i: Inset = 16.0_f32.into();
        assert_eq!(i, Inset::Px(16.0));
    }

    #[test]
    fn inset_from_f64() {
        let i: Inset = 24.0_f64.into();
        assert_eq!(i, Inset::Px(24.0));
    }
}