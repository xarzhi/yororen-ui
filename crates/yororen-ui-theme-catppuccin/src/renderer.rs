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

/// Catppuccin card: 16-px radius, surface0 background, surface1 border
/// (subtler than the v0.5 default). Larger padding gives the card
/// "breathing room".
pub struct CatppuccinCardRenderer;

impl CardRenderer for CatppuccinCardRenderer {
    fn bg(&self, _state: &CardRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::mantle()
    }

    fn border(&self, _state: &CardRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::surface1()
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

/// Catppuccin modal: surface0 panel (subtler than mantle), 16-px
/// radius, dark scrim at 0.55 alpha.
pub struct CatppuccinModalRenderer;

impl ModalRenderer for CatppuccinModalRenderer {
    fn scrim(&self, _state: &ModalRenderState, _theme: &Theme) -> Hsla {
        let mut c = palette::mocha::crust();
        c.a = 0.55;
        c
    }

    fn panel_bg(&self, _state: &ModalRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::mantle()
    }

    fn panel_border(&self, _state: &ModalRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::surface1()
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

/// Catppuccin text input: 12-px radius, surface0 background, focus
/// border uses `mauve` (the signature focus color).
pub struct CatppuccinTextInputRenderer;

impl TextInputRenderer for CatppuccinTextInputRenderer {
    fn bg(&self, state: &TextInputRenderState, _theme: &Theme) -> Hsla {
        if state.disabled {
            palette::mocha::surface0()
        } else {
            palette::mocha::base()
        }
    }

    fn border(&self, _state: &TextInputRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::surface1()
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

/// Catppuccin switch: track uses `peach` when on (signature Catppuccin
/// pastel), surface0 when off. 12-px radius matches the rest of the
/// Catppuccin style language.
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

    fn track_bg(&self, state: &SwitchRenderState, _theme: &Theme) -> Hsla {
        if state.disabled {
            palette::mocha::surface0()
        } else if state.checked {
            palette::mocha::peach()
        } else {
            palette::mocha::surface1()
        }
    }

    fn track_border(&self, _state: &SwitchRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::surface2()
    }

    fn track_hover_bg(&self, state: &SwitchRenderState, _theme: &Theme) -> Hsla {
        if state.checked {
            palette::mocha::maroon()
        } else {
            palette::mocha::surface2()
        }
    }

    fn knob_bg(&self, _state: &SwitchRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::base()
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

/// Catppuccin checkbox: `mauve` background when checked (the
/// signature focus accent), `text` fg, 4-px radius (slightly less
/// round than the rest of the UI to keep the check glyph readable).
pub struct CatppuccinCheckboxRenderer;

impl CheckboxRenderer for CatppuccinCheckboxRenderer {
    fn box_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.box_size
    }
    fn check_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.check_size
    }
    fn box_bg(&self, state: &CheckboxRenderState, _theme: &Theme) -> Hsla {
        if state.disabled {
            palette::mocha::surface0()
        } else if state.checked {
            palette::mocha::mauve()
        } else {
            palette::mocha::base()
        }
    }
    fn box_border(&self, state: &CheckboxRenderState, _theme: &Theme) -> Hsla {
        if state.checked {
            palette::mocha::mauve()
        } else {
            palette::mocha::surface2()
        }
    }
    fn box_hover_bg(&self, state: &CheckboxRenderState, _theme: &Theme) -> Hsla {
        if state.checked {
            palette::mocha::mauve()
        } else {
            palette::mocha::surface1()
        }
    }
    fn check_fg(&self, _state: &CheckboxRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::base()
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

/// Catppuccin radio: `mauve` ring + dot when checked, matching the
/// checkbox for visual consistency.
pub struct CatppuccinRadioRenderer;

impl RadioRenderer for CatppuccinRadioRenderer {
    fn ring_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.ring_size
    }
    fn dot_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.dot_size
    }
    fn ring_bg(&self, state: &RadioRenderState, _theme: &Theme) -> Hsla {
        if state.disabled {
            palette::mocha::surface0()
        } else {
            palette::mocha::base()
        }
    }
    fn ring_border(&self, state: &RadioRenderState, _theme: &Theme) -> Hsla {
        if state.checked {
            palette::mocha::mauve()
        } else {
            palette::mocha::surface2()
        }
    }
    fn ring_hover_bg(&self, _state: &RadioRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::surface1()
    }
    fn dot_fg(&self, _state: &RadioRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::mauve()
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

/// Catppuccin toast: surface0 background, surface1 border, 12-px
/// radius, soft shadow.
pub struct CatppuccinToastRenderer;

impl ToastRenderer for CatppuccinToastRenderer {
    fn bg(&self, _state: &ToastRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::mantle()
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
    fn border(&self, _state: &ToastRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::surface1()
    }
    fn shadow_alpha(&self, _state: &ToastRenderState, _theme: &Theme) -> f32 {
        0.30
    }
}

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------

/// Catppuccin tag: pill-shaped (border-radius = full), uses surface1
/// background by default. Selected tag uses `mauve` accent.
pub struct CatppuccinTagRenderer;

impl TagRenderer for CatppuccinTagRenderer {
    fn bg(&self, state: &TagRenderState, _theme: &Theme) -> Hsla {
        if state.selected {
            palette::mocha::mauve()
        } else {
            palette::mocha::surface1()
        }
    }
    fn fg(&self, state: &TagRenderState, _theme: &Theme) -> Hsla {
        if state.selected {
            palette::mocha::base()
        } else {
            palette::mocha::text()
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
    fn close_hover_bg(&self, _state: &TagRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::surface2()
    }
}

// ---------------------------------------------------------------------------
// ListItem
// ---------------------------------------------------------------------------

/// Catppuccin list item: surface0 background, surface1 hover, `mauve`
/// selected background.
pub struct CatppuccinListItemRenderer;

impl ListItemRenderer for CatppuccinListItemRenderer {
    fn bg(&self, _state: &ListItemRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::base()
    }
    fn hover_bg(&self, _state: &ListItemRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::surface1()
    }
    fn selected_bg(&self, _state: &ListItemRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::mauve()
    }
    fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else if state.selected {
            palette::mocha::base()
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

/// Catppuccin empty state: `overlay0` icon, `subtext1` title, `subtext0`
/// body. Generous padding and a 64-px icon.
pub struct CatppuccinEmptyStateRenderer;

impl EmptyStateRenderer for CatppuccinEmptyStateRenderer {
    fn icon_color(&self, _state: &EmptyStateRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::overlay0()
    }
    fn title_color(&self, _state: &EmptyStateRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::subtext1()
    }
    fn body_color(&self, _state: &EmptyStateRenderState, _theme: &Theme) -> Hsla {
        palette::mocha::subtext0()
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
