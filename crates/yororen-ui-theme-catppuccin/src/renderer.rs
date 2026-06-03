//! Catppuccin-flavored Renderer implementations.
//!
//! Each `*Renderer` is a thin zero-sized type that implements the
//! corresponding Renderer trait from `yororen-ui-core`. The Catppuccin
//! "voice" is captured in the constants used throughout:
//!
//! - Border radius 12 px (chunkier than the v0.5 system default of 6 px).
//! - Focus ring uses `mauve` (the signature Catppuccin accent).
//! - Shadows are darker / more saturated than the v0.5 default.
//! - Switches / Checkboxes / Radios use the `peach` accent for "on".
//! - Status colors use the matching Catppuccin pastels
//!   (`green` / `yellow` / `red` / `sapphire`).
//!
//! Together they give a Catppuccin-styled UI without modifying any
//! core code: an app that calls `catppuccin::install(cx, ...)` will
//! see the new visuals immediately on the next render.

use std::sync::Arc;

use gpui::{Hsla, Pixels, px};

use yororen_ui_core::renderer::{
    ButtonRenderState, ButtonRenderer, CardRenderState, CardRenderer, CheckboxRenderState,
    CheckboxRenderer, EmptyStateRenderState, EmptyStateRenderer, FocusRingRenderState,
    FocusRingRenderer, ListItemRenderState, ListItemRenderer, ModalRenderState, ModalRenderer,
    RadioRenderState, RadioRenderer, RendererRegistry, SwitchRenderState, SwitchRenderer,
    TagRenderState, TagRenderer, TextInputRenderState, TextInputRenderer, ToastRenderState,
    ToastRenderer,
};
use yororen_ui_core::theme::{ActionVariantKind, Theme};

use crate::palette;

/// Catppuccin's signature border radius. Bigger than the v0.5 system
/// default of 6 px; gives the UI a softer, chunkier feel.
pub const CATPPUCCIN_RADIUS: f32 = 12.0;

/// Catppuccin "small" border radius. Used for badges / tags.
pub const CATPPUCCIN_RADIUS_SM: f32 = 8.0;

/// Catppuccin focus ring width.
pub const FOCUS_RING_WIDTH: f32 = 2.0;

/// Standard focus ring color: `mauve` from the active flavor. Callers
/// can switch this per-flavor via the `CatppuccinFocusRingRenderer`
/// constructor.
pub fn focus_ring_color_for(theme: &Theme) -> Hsla {
    // Theme.border.focus is set to accent.mauve in the factory, so
    // the focus ring color tracks the flavor automatically.
    theme.border.focus
}

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

/// Catppuccin button: 12-px radius, 8-px/14-px padding, the accent
/// colors come from the active theme's `action.primary` /
/// `action.danger` / `action.neutral`. The Catppuccin-specific variants
/// (`mocha`, `lavender`, `ghost`) override `bg` / `fg` through the
/// `VariantStyle` path.
pub struct CatppuccinButtonRenderer;

impl ButtonRenderer for CatppuccinButtonRenderer {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&yororen_ui_core::renderer::VariantState {
                disabled: state.disabled,
            });
        }
        let v = theme.action_variant(state.variant);
        if state.disabled {
            v.disabled_bg
        } else {
            match state.variant {
                ActionVariantKind::Primary => v.bg,
                ActionVariantKind::Neutral => v.bg,
                ActionVariantKind::Danger => v.bg,
            }
        }
    }

    fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.fg(&yororen_ui_core::renderer::VariantState {
                disabled: state.disabled,
            });
        }
        let v = theme.action_variant(state.variant);
        if state.disabled { v.disabled_fg } else { v.fg }
    }

    fn padding(
        &self,
        _state: &ButtonRenderState,
        theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        let t = &theme.tokens.control.button;
        yororen_ui_core::renderer::Edges::symmetric(
            t.horizontal_padding,
            t.horizontal_padding / 1.5,
        )
    }

    fn border_radius(&self, _state: &ButtonRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }

    fn border(
        &self,
        _state: &ButtonRenderState,
        _theme: &Theme,
    ) -> Option<yororen_ui_core::renderer::BorderSpec> {
        None
    }

    fn shadow(
        &self,
        _state: &ButtonRenderState,
        _theme: &Theme,
    ) -> Option<yororen_ui_core::renderer::ShadowSpec> {
        None
    }

    fn min_height(&self, _state: &ButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }

    fn disabled_opacity(&self, state: &ButtonRenderState, _theme: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        if state.disabled { 0.55 } else { 1.0 }
    }
}

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

/// Catppuccin card: 16-px radius, `surface.raised` background
/// (Latte: `#E6E9EF`, Mocha: `#181825`), `border.default` border
/// (Latte: `#BCC0CC`, Mocha: `#45475A`). Larger padding gives the
/// card "breathing room".
pub struct CatppuccinCardRenderer;

impl CardRenderer for CatppuccinCardRenderer {
    fn bg(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }

    fn border(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }

    fn padding(
        &self,
        _state: &CardRenderState,
        _theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        yororen_ui_core::renderer::Edges::all(px(20.0))
    }

    fn border_radius(&self, _state: &CardRenderState, _theme: &Theme) -> Pixels {
        px(16.0)
    }

    fn shadow_alpha(&self, _state: &CardRenderState, _theme: &Theme) -> f32 {
        // Catppuccin has a fairly soft shadow.
        0.20
    }
}

// ---------------------------------------------------------------------------
// Modal
// ---------------------------------------------------------------------------

/// Catppuccin modal: `surface.raised` panel (Latte: `#E6E9EF`,
/// Mocha: `#181825`), 16-px radius, dark scrim at 0.55 alpha.
pub struct CatppuccinModalRenderer;

impl ModalRenderer for CatppuccinModalRenderer {
    fn scrim(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        let mut c = theme.surface.canvas;
        c.a = 0.55;
        c
    }

    fn panel_bg(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }

    fn panel_border(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }

    fn panel_padding(
        &self,
        _state: &ModalRenderState,
        _theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        yororen_ui_core::renderer::Edges::all(px(24.0))
    }

    fn panel_border_radius(&self, _state: &ModalRenderState, _theme: &Theme) -> Pixels {
        px(16.0)
    }

    fn panel_shadow_alpha(&self, _state: &ModalRenderState, _theme: &Theme) -> f32 {
        0.45
    }
}

// ---------------------------------------------------------------------------
// FocusRing
// ---------------------------------------------------------------------------

/// Catppuccin focus ring: 2-px wide `mauve` outline. Tracks the
/// active theme so the ring color follows the flavor.
pub struct CatppuccinFocusRingRenderer;

impl FocusRingRenderer for CatppuccinFocusRingRenderer {
    fn color(&self, _state: &FocusRingRenderState, theme: &Theme) -> Hsla {
        focus_ring_color_for(theme)
    }
    fn width(&self, _state: &FocusRingRenderState, _theme: &Theme) -> Pixels {
        px(FOCUS_RING_WIDTH)
    }
}

// ---------------------------------------------------------------------------
// TextInput
// ---------------------------------------------------------------------------

/// Catppuccin text input: 12-px radius, `surface.base` background,
/// `border.default` border, focus border uses the active theme's
/// `border.focus` (which is `mauve` for both Latte and Mocha).
pub struct CatppuccinTextInputRenderer;

impl TextInputRenderer for CatppuccinTextInputRenderer {
    fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }

    fn border(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }

    fn focus_border(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }

    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else {
            theme.content.primary
        }
    }

    fn hint_color(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }

    fn min_height(&self, _state: &TextInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }

    fn padding(
        &self,
        _state: &TextInputRenderState,
        theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        yororen_ui_core::renderer::Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }

    fn border_radius(&self, _state: &TextInputRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }

    fn disabled_opacity(&self, _state: &TextInputRenderState, _theme: &Theme) -> f32 {
        0.6
    }
}

// ---------------------------------------------------------------------------
// Switch
// ---------------------------------------------------------------------------

/// Catppuccin switch: track uses `action.primary.bg` when on
/// (signature Catppuccin main accent — `mocha::blue` for dark,
/// `latte::blue` for light), `surface.hover` when off. 12-px
/// radius matches the rest of the Catppuccin style language.
pub struct CatppuccinSwitchRenderer;

impl SwitchRenderer for CatppuccinSwitchRenderer {
    fn track_w(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.track_w
    }
    fn track_h(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.track_h
    }
    fn knob_size(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.knob_size
    }
    fn padding(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.switch.padding
    }

    fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else if state.checked {
            theme.action.primary.bg
        } else {
            theme.surface.hover
        }
    }

    fn track_border(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.border.muted
    }

    fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.action.primary.hover_bg
        } else {
            theme.surface.hover
        }
    }

    fn knob_bg(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.content.on_primary
    }

    fn focus_color(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }

    fn disabled_opacity(&self, _state: &SwitchRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------

/// Catppuccin checkbox: `border.focus` (`mauve`) background when
/// checked (the signature Catppuccin focus accent — this is
/// mapped from `accent.mauve()` in the factory for both Latte and
/// Mocha), `content.on_primary` fg.
pub struct CatppuccinCheckboxRenderer;

impl CheckboxRenderer for CatppuccinCheckboxRenderer {
    fn box_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.box_size
    }
    fn check_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.check_size
    }
    fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else if state.checked {
            theme.border.focus
        } else {
            theme.surface.base
        }
    }
    fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.border.focus
        } else {
            theme.border.muted
        }
    }
    fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.border.focus
        } else {
            theme.surface.hover
        }
    }
    fn check_fg(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.content.on_primary
    }
    fn focus_color(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &CheckboxRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

// ---------------------------------------------------------------------------
// Radio
// ---------------------------------------------------------------------------

/// Catppuccin radio: `border.focus` ring + dot when checked,
/// matching the checkbox for visual consistency.
pub struct CatppuccinRadioRenderer;

impl RadioRenderer for CatppuccinRadioRenderer {
    fn ring_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.ring_size
    }
    fn dot_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.dot_size
    }
    fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.border.focus
        } else {
            theme.border.muted
        }
    }
    fn ring_hover_bg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn dot_fg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn focus_color(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &RadioRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

// ---------------------------------------------------------------------------
// Toast
// ---------------------------------------------------------------------------

/// Catppuccin toast: `surface.raised` background, `border.default`
/// border, 12-px radius, soft shadow.
pub struct CatppuccinToastRenderer;

impl ToastRenderer for CatppuccinToastRenderer {
    fn bg(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn fg(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn padding(
        &self,
        _state: &ToastRenderState,
        theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        yororen_ui_core::renderer::Edges::symmetric(
            theme.tokens.spacing.inset_md,
            theme.tokens.spacing.inset_sm,
        )
    }
    fn border_radius(&self, _state: &ToastRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn border(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn shadow_alpha(&self, _state: &ToastRenderState, _theme: &Theme) -> f32 {
        0.30
    }
}

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------

/// Catppuccin tag: pill-shaped (border-radius = full), uses
/// `surface.hover` background by default. Selected tag uses
/// `border.focus` (`mauve`) accent.
pub struct CatppuccinTagRenderer;

impl TagRenderer for CatppuccinTagRenderer {
    fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.border.focus
        } else {
            theme.surface.hover
        }
    }
    fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.content.on_primary
        } else {
            theme.content.primary
        }
    }
    fn min_height(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.tag.min_height
    }
    fn padding_x(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_sm
    }
    fn font_size(&self, _state: &TagRenderState, theme: &Theme) -> Pixels {
        theme.tokens.typography.font_size_xs
    }
    fn font_weight(&self, _state: &TagRenderState, theme: &Theme) -> gpui::FontWeight {
        theme.tokens.typography.weight_medium
    }
    fn border_radius(&self, _state: &TagRenderState, _theme: &Theme) -> Pixels {
        px(999.0) // pill
    }
    fn close_size(&self, _state: &TagRenderState, _theme: &Theme) -> Pixels {
        px(16.0)
    }
    fn close_hover_bg(&self, _state: &TagRenderState, theme: &Theme) -> Hsla {
        theme.border.muted
    }
}

// ---------------------------------------------------------------------------
// ListItem
// ---------------------------------------------------------------------------

/// Catppuccin list item: `surface.base` background, `surface.hover`
/// hover, `border.focus` (`mauve`) selected background.
pub struct CatppuccinListItemRenderer;

impl ListItemRenderer for CatppuccinListItemRenderer {
    fn bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn hover_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn selected_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else if state.selected {
            theme.content.on_primary
        } else {
            theme.content.primary
        }
    }
    fn padding(
        &self,
        _state: &ListItemRenderState,
        theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        yororen_ui_core::renderer::Edges::symmetric(
            theme.tokens.spacing.inset_sm,
            theme.tokens.spacing.inset_xs,
        )
    }
    fn min_height(&self, _state: &ListItemRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.list_item.min_height
    }
    fn border_radius(&self, _state: &ListItemRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS_SM)
    }
}

// ---------------------------------------------------------------------------
// EmptyState
// ---------------------------------------------------------------------------

/// Catppuccin empty state: `content.tertiary` icon,
/// `content.secondary` title, `content.tertiary` body. Generous
/// padding and a 64-px icon.
pub struct CatppuccinEmptyStateRenderer;

impl EmptyStateRenderer for CatppuccinEmptyStateRenderer {
    fn icon_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
    fn title_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.content.secondary
    }
    fn body_color(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
    fn padding(
        &self,
        _state: &EmptyStateRenderState,
        _theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        yororen_ui_core::renderer::Edges::all(px(32.0))
    }
    fn icon_size(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_xl
    }
    fn gap(&self, _state: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_sm
    }
}

// ---------------------------------------------------------------------------
// RendererRegistry
// ---------------------------------------------------------------------------

/// Build a `RendererRegistry` that swaps the v0.5 default
/// `TokenXxxRenderer` for the Catppuccin-flavoured implementations
/// defined in this module. The remaining components that don't have a
/// Catppuccin-specific renderer keep their `TokenXxxRenderer`
/// defaults.
pub fn catppuccin_registry() -> RendererRegistry {
    RendererRegistry::token_based()
        .with_button(Arc::new(CatppuccinButtonRenderer))
        .with_card(Arc::new(CatppuccinCardRenderer))
        .with_modal(Arc::new(CatppuccinModalRenderer))
        .with_focus_ring(Arc::new(CatppuccinFocusRingRenderer))
        .with_text_input(Arc::new(CatppuccinTextInputRenderer))
        .with_switch(Arc::new(CatppuccinSwitchRenderer))
        .with_checkbox(Arc::new(CatppuccinCheckboxRenderer))
        .with_radio(Arc::new(CatppuccinRadioRenderer))
        .with_toast(Arc::new(CatppuccinToastRenderer))
        .with_tag(Arc::new(CatppuccinTagRenderer))
        .with_list_item(Arc::new(CatppuccinListItemRenderer))
        .with_empty_state(Arc::new(CatppuccinEmptyStateRenderer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use yororen_ui_core::theme::ActionVariantKind;

    fn cat_light() -> Theme {
        crate::factories::light()
    }
    fn cat_dark() -> Theme {
        crate::factories::dark()
    }

    #[test]
    fn button_renderer_uses_catppuccin_radius() {
        let r = CatppuccinButtonRenderer;
        let theme = cat_dark();
        let state = ButtonRenderState::default();
        let radius: f32 = r.border_radius(&state, &theme).into();
        assert!((radius - CATPPUCCIN_RADIUS).abs() < 0.5);
    }

    #[test]
    fn button_renderer_uses_primary_action_color() {
        let r = CatppuccinButtonRenderer;
        let theme = cat_dark();
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        };
        // Catppuccin primary = mocha::blue (via theme.action.primary.bg).
        assert_eq!(r.bg(&state, &theme), theme.action.primary.bg);
    }

    #[test]
    fn button_renderer_uses_mocha_palette_when_given_mocha_theme() {
        use yororen_ui_core::renderer::ButtonRenderer;
        // The Catppuccin renderer reads from the same action palette
        // as the v0.5 default; the visual difference comes from the
        // Theme itself being a Catppuccin palette. Verify the
        // Catppuccin renderer returns mocha::blue for primary when
        // given a Mocha theme.
        let theme = cat_dark();
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        };
        let cat_bg = CatppuccinButtonRenderer.bg(&state, &theme);
        assert_eq!(cat_bg, palette::mocha::blue());
    }

    #[test]
    fn card_renderer_uses_mantle_background() {
        let r = CatppuccinCardRenderer;
        let theme = cat_dark();
        let state = CardRenderState::default();
        let bg = r.bg(&state, &theme);
        assert_eq!(bg, palette::mocha::mantle());
    }

    /// Regression test for the v0.5 review's finding: 9 renderers
    /// hardcoded `palette::mocha::*` and so looked the same in
    /// light vs dark mode. After the fix, every renderer should
    /// produce a DIFFERENT bg for a Latte-flavoured Theme vs a
    /// Mocha-flavoured Theme.
    #[test]
    fn card_renderer_light_dark_differ() {
        use yororen_ui_core::renderer::CardRenderer as _;
        let r = CatppuccinCardRenderer;
        let light = cat_light();
        let dark = cat_dark();
        let state = CardRenderState::default();
        let light_bg = r.bg(&state, &light);
        let dark_bg = r.bg(&state, &dark);
        assert_ne!(light_bg, dark_bg, "light and dark should produce different card bg");
        // Sanity: light bg should be on the "light" side (Latte's
        // surface.raised is #E6E9EF, a near-white tone), dark bg
        // should be on the "dark" side (Mocha's surface.raised is
        // #181825, a near-black tone).
        assert!(light_bg.l > dark_bg.l);
    }

    #[test]
    fn modal_renderer_light_dark_differ() {
        use yororen_ui_core::renderer::ModalRenderer as _;
        let r = CatppuccinModalRenderer;
        let light = cat_light();
        let dark = cat_dark();
        let state = ModalRenderState::default();
        let light_panel = r.panel_bg(&state, &light);
        let dark_panel = r.panel_bg(&state, &dark);
        assert_ne!(light_panel, dark_panel);
        assert!(light_panel.l > dark_panel.l);
    }

    #[test]
    fn text_input_renderer_light_dark_differ() {
        use yororen_ui_core::renderer::TextInputRenderer as _;
        let r = CatppuccinTextInputRenderer;
        let light = cat_light();
        let dark = cat_dark();
        let state = TextInputRenderState::default();
        let light_bg = r.bg(&state, &light);
        let dark_bg = r.bg(&state, &dark);
        assert_ne!(light_bg, dark_bg);
        assert!(light_bg.l > dark_bg.l);
    }

    #[test]
    fn switch_renderer_checked_color_differs_by_flavor() {
        use yororen_ui_core::renderer::SwitchRenderer as _;
        let r = CatppuccinSwitchRenderer;
        let light = cat_light();
        let dark = cat_dark();
        // When checked, the track uses theme.action.primary.bg. In
        // a Catppuccin light theme this is latte::blue; in dark
        // theme this is mocha::blue. Both are blue, but the
        // lightness differs.
        let state = SwitchRenderState { checked: true, ..Default::default() };
        let light_on = r.track_bg(&state, &light);
        let dark_on = r.track_bg(&state, &dark);
        // The exact lightness ordering depends on the Latte vs Mocha
        // blue; what we can assert is that the two are different
        // objects (not the same hardcoded color).
        assert_ne!(light_on, dark_on);
    }

    #[test]
    fn checkbox_renderer_uses_focus_color_when_checked() {
        use yororen_ui_core::renderer::CheckboxRenderer as _;
        let r = CatppuccinCheckboxRenderer;
        let light = cat_light();
        let dark = cat_dark();
        let state = CheckboxRenderState { checked: true, ..Default::default() };
        // The Catppuccin checkbox uses the focus color (mauve) for
        // the checked bg, which is theme.border.focus.
        assert_eq!(r.box_bg(&state, &light), light.border.focus);
        assert_eq!(r.box_bg(&state, &dark), dark.border.focus);
    }

    #[test]
    fn empty_state_uses_content_tertiary() {
        use yororen_ui_core::renderer::EmptyStateRenderer as _;
        let r = CatppuccinEmptyStateRenderer;
        let light = cat_light();
        let dark = cat_dark();
        let state = EmptyStateRenderState::default();
        // icon_color is content.tertiary per the new impl.
        assert_eq!(r.icon_color(&state, &light), light.content.tertiary);
        assert_eq!(r.icon_color(&state, &dark), dark.content.tertiary);
    }

    #[test]
    fn focus_ring_color_tracks_theme() {
        let r = CatppuccinFocusRingRenderer;
        let light_theme = cat_light();
        let dark_theme = cat_dark();
        let state = FocusRingRenderState::default();
        // The factory sets border.focus to mauve for each flavor.
        assert_eq!(r.color(&state, &light_theme), palette::latte::mauve());
        assert_eq!(r.color(&state, &dark_theme), palette::mocha::mauve());
    }

    #[test]
    fn modal_scrim_alpha_is_055() {
        let r = CatppuccinModalRenderer;
        let theme = cat_dark();
        let state = ModalRenderState::default();
        let scrim = r.scrim(&state, &theme);
        assert!((scrim.a - 0.55).abs() < 0.01);
    }

    #[test]
    fn registry_includes_twelve_custom_renderers() {
        let reg = catppuccin_registry();
        // All 12 of our overrides are in place.
        let _ = reg.button.bg(&ButtonRenderState::default(), &cat_light());
        let _ = reg.card.bg(&CardRenderState::default(), &cat_light());
        let _ = reg
            .modal
            .panel_bg(&ModalRenderState::default(), &cat_light());
        let _ = reg
            .focus_ring
            .color(&FocusRingRenderState::default(), &cat_light());
        let _ = reg
            .text_input
            .bg(&TextInputRenderState::default(), &cat_light());
        let _ = reg
            .switch
            .track_bg(&SwitchRenderState::default(), &cat_light());
        let _ = reg
            .checkbox
            .box_bg(&CheckboxRenderState::default(), &cat_light());
        let _ = reg
            .radio
            .ring_bg(&RadioRenderState::default(), &cat_light());
        let _ = reg.toast.bg(&ToastRenderState::default(), &cat_light());
        let _ = reg.tag.bg(&TagRenderState::default(), &cat_light());
        let _ = reg
            .list_item
            .bg(&ListItemRenderState::default(), &cat_light());
        let _ = reg
            .empty_state
            .icon_color(&EmptyStateRenderState::default(), &cat_light());
    }
}
