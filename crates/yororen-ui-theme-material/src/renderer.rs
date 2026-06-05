//! Material Design 3 (M3) Renderer implementations.
//!
//! Each `*Renderer` is a zero-sized type that implements the
//! matching trait from `yororen-ui-core`. The M3 visual language is
//! captured in the constants used throughout:
//!
//! - Pill (full) radius for buttons, switches, FABs.
//! - `lg` (12 px) radius for cards, list items, sheets.
//! - `md` (8 px) radius for chips, snackbars.
//! - `sm` (4 px) radius for small inline surfaces (badge, divider
//!   dots).
//! - State layers: 8% primary on hover, 10% on press, 8% on focus.
//! - Focus ring uses M3 `outline` color at 1.5 px width.
//! - Shadows tinted with `primary` color (M3 elevation system).
//! - Switch uses `primary` accent when on.
//! - Checkbox / Radio use the same `primary` accent when checked.

use std::sync::Arc;

use gpui::{FontWeight, Hsla, Pixels, SharedString, px};

use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::renderer::{
    AvatarRenderState, AvatarRenderer, BadgeRenderState, BadgeRenderer, ButtonRenderState,
    ButtonRenderer, CardRenderState, CardRenderer, CheckboxRenderState, CheckboxRenderer,
    DividerRenderState, DividerRenderer, FocusRingRenderState, FocusRingRenderer,
    HeadingRenderState, HeadingRenderer, IconButtonRenderState, IconButtonRenderer,
    LabelRenderState, LabelRenderer, ListItemRenderState, ListItemRenderer, ModalRenderState,
    ModalRenderer, PanelRenderState, PanelRenderer, PopoverRenderState, PopoverRenderer,
    RadioRenderState, RadioRenderer, RendererRegistry, SwitchRenderState, SwitchRenderer,
    TagRenderState, TagRenderer, TextInputRenderState, TextInputRenderer, ToastRenderState,
    ToastRenderer, TooltipRenderState, TooltipRenderer,
};
use yororen_ui_core::theme::Theme;

use crate::palette::{self, radii, state_layer};

/// M3 focus ring width (1.5 px).
pub const FOCUS_RING_WIDTH: f32 = 1.5;

// ---------------------------------------------------------------------------
// Button
// ---------------------------------------------------------------------------

/// M3 button: pill (full) radius, 40-px height, 24-px horizontal
/// padding. M3 has 5 button styles (filled, tonal, outlined, text,
/// elevated); the renderer below ships the "filled" (primary) and
/// the "danger" styles through the variant mechanism. M3's
/// "outlined" / "text" / "elevated" / "tonal" styles are not
/// covered here — a custom variant registered through the
/// `VariantRegistry` is the recommended way to ship them.
pub struct MaterialButtonRenderer;

impl ButtonRenderer for MaterialButtonRenderer {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&yororen_ui_core::renderer::VariantState {
                disabled: state.disabled,
            });
        }
        let v = theme.action_variant(state.variant);
        if state.disabled { v.disabled_bg } else { v.bg }
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
            t.horizontal_padding / 2.4,
        )
    }

    fn border_radius(&self, _state: &ButtonRenderState, _theme: &Theme) -> Pixels {
        px(radii::PILL)
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
        if state.disabled {
            1.0 - state_layer::DISABLED_CONTAINER
        } else {
            1.0
        }
    }
}

// ---------------------------------------------------------------------------
// IconButton
// ---------------------------------------------------------------------------

/// M3 icon button: pill-radius, 40-px target, transparent base
/// with 8% primary state layer on hover.
pub struct MaterialIconButtonRenderer;

impl IconButtonRenderer for MaterialIconButtonRenderer {
    fn bg(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Hsla {
        gpui::Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.0,
        }
    }
    fn hover_bg(&self, _state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        palette::apply_state_layer(theme.surface.base, theme.border.focus, state_layer::HOVER)
    }
    fn size(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Pixels {
        px(40.0)
    }
    fn border_radius(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Pixels {
        px(radii::PILL)
    }
    fn disabled_opacity(&self, _state: &IconButtonRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

// ---------------------------------------------------------------------------
// Card
// ---------------------------------------------------------------------------

/// M3 card: 12-px radius, 1-px outline (M3 "outlined" card) on
/// `surface` background, 16-px padding.
pub struct MaterialCardRenderer;

impl CardRenderer for MaterialCardRenderer {
    fn bg(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn border(&self, _state: &CardRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn padding(
        &self,
        _state: &CardRenderState,
        _theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        yororen_ui_core::renderer::Edges::all(px(16.0))
    }
    fn border_radius(&self, _state: &CardRenderState, _theme: &Theme) -> Pixels {
        px(radii::LG)
    }
    fn shadow_alpha(&self, _state: &CardRenderState, _theme: &Theme) -> f32 {
        0.05
    }
}

// ---------------------------------------------------------------------------
// Modal
// ---------------------------------------------------------------------------

/// M3 dialog: 28-px radius (M3 dialog corner), surface.background
/// background, 24-px padding, dark scrim at 0.32 alpha (M3 spec).
pub struct MaterialModalRenderer;

impl ModalRenderer for MaterialModalRenderer {
    fn scrim(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        let mut c = theme.surface.canvas;
        c.a = 0.32;
        c
    }
    fn panel_bg(&self, _state: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn panel_border(&self, _state: &ModalRenderState, _theme: &Theme) -> Hsla {
        gpui::Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.0,
        }
    }
    fn panel_padding(
        &self,
        _state: &ModalRenderState,
        _theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        yororen_ui_core::renderer::Edges::all(px(24.0))
    }
    fn panel_border_radius(&self, _state: &ModalRenderState, _theme: &Theme) -> Pixels {
        px(28.0)
    }
    fn panel_shadow_alpha(&self, _state: &ModalRenderState, _theme: &Theme) -> f32 {
        0.11
    }
}

// ---------------------------------------------------------------------------
// Panel
// ---------------------------------------------------------------------------

/// M3 panel (generic container): 12-px radius, no border, raised
/// surface.
pub struct MaterialPanelRenderer;

impl PanelRenderer for MaterialPanelRenderer {
    fn bg(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn border(&self, _state: &PanelRenderState, _theme: &Theme) -> Hsla {
        gpui::Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.0,
        }
    }
    fn padding(&self, _state: &PanelRenderState, _theme: &Theme) -> Edges<Pixels> {
        Edges::all(px(16.0))
    }
    fn border_radius(&self, _state: &PanelRenderState, _theme: &Theme) -> Pixels {
        px(radii::LG)
    }
    fn shadow_alpha(&self, _state: &PanelRenderState, _theme: &Theme) -> f32 {
        0.05
    }
}

// ---------------------------------------------------------------------------
// FocusRing
// ---------------------------------------------------------------------------

/// M3 focus ring: 1.5-px wide, `primary` color.
pub struct MaterialFocusRingRenderer;

impl FocusRingRenderer for MaterialFocusRingRenderer {
    fn color(&self, _state: &FocusRingRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn width(&self, _state: &FocusRingRenderState, _theme: &Theme) -> Pixels {
        px(FOCUS_RING_WIDTH)
    }
}

// ---------------------------------------------------------------------------
// TextInput
// ---------------------------------------------------------------------------

/// M3 text field: filled style — 4-px top corner radius (we use
/// 4 for all sides to keep the API simple), `surface.sunken`
/// background, `primary` underline when focused.
pub struct MaterialTextInputRenderer;

impl TextInputRenderer for MaterialTextInputRenderer {
    fn bg(&self, _state: &TextInputRenderState, theme: &Theme) -> Hsla {
        theme.surface.sunken
    }
    fn border(&self, _state: &TextInputRenderState, _theme: &Theme) -> Hsla {
        gpui::Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.0,
        }
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
        theme.content.secondary
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
        px(radii::SM)
    }
    fn disabled_opacity(&self, _state: &TextInputRenderState, _theme: &Theme) -> f32 {
        1.0 - state_layer::DISABLED_CONTAINER
    }
}

// ---------------------------------------------------------------------------
// Switch
// ---------------------------------------------------------------------------

/// M3 switch: pill track, `surface.sunken` when off, `primary`
/// when on.
pub struct MaterialSwitchRenderer;

impl SwitchRenderer for MaterialSwitchRenderer {
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
            theme.border.focus
        } else {
            theme.surface.sunken
        }
    }
    fn track_border(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            palette::apply_state_layer(
                theme.border.focus,
                theme.content.on_primary,
                state_layer::HOVER,
            )
        } else {
            palette::apply_state_layer(theme.surface.sunken, theme.border.focus, state_layer::HOVER)
        }
    }
    fn knob_bg(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.content.on_primary
    }
    fn focus_color(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &SwitchRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

// ---------------------------------------------------------------------------
// Checkbox
// ---------------------------------------------------------------------------

/// M3 checkbox: `primary` fill when checked, outline when
/// unchecked.
pub struct MaterialCheckboxRenderer;

impl CheckboxRenderer for MaterialCheckboxRenderer {
    fn box_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.box_size
    }
    fn check_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.checkbox.check_size
    }
    fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return theme.surface.sunken;
        }
        if state.checked {
            theme.border.focus
        } else {
            theme.surface.base
        }
    }
    fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.border.focus
        } else {
            theme.border.default
        }
    }
    fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            palette::apply_state_layer(
                theme.border.focus,
                theme.content.on_primary,
                state_layer::HOVER,
            )
        } else {
            palette::apply_state_layer(theme.surface.base, theme.border.focus, state_layer::HOVER)
        }
    }
    fn check_fg(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.content.on_primary
    }
    fn focus_color(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &CheckboxRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

// ---------------------------------------------------------------------------
// Radio
// ---------------------------------------------------------------------------

/// M3 radio: outline when unchecked, `primary` dot when checked.
pub struct MaterialRadioRenderer;

impl RadioRenderer for MaterialRadioRenderer {
    fn ring_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.ring_size
    }
    fn dot_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.radio.dot_size
    }
    fn ring_bg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.border.focus
        } else {
            theme.border.default
        }
    }
    fn ring_hover_bg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        palette::apply_state_layer(theme.surface.base, theme.border.focus, state_layer::HOVER)
    }
    fn dot_fg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn focus_color(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn disabled_opacity(&self, _state: &RadioRenderState, _theme: &Theme) -> f32 {
        1.0
    }
}

// ---------------------------------------------------------------------------
// Toast
// ---------------------------------------------------------------------------

/// M3 snackbar: dark inverse surface, light text, 4-px radius.
pub struct MaterialToastRenderer;

impl ToastRenderer for MaterialToastRenderer {
    fn bg(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        let mut c = theme.surface.canvas;
        if theme.surface.canvas.l > 0.5 {
            c.l = 0.20;
        }
        c
    }
    fn fg(&self, _state: &ToastRenderState, theme: &Theme) -> Hsla {
        let mut c = theme.surface.canvas;
        let _ = c.l;
        c.l = 0.95;
        c
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
        px(radii::SM)
    }
    fn border(&self, _state: &ToastRenderState, _theme: &Theme) -> Hsla {
        gpui::Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.0,
        }
    }
    fn shadow_alpha(&self, _state: &ToastRenderState, _theme: &Theme) -> f32 {
        0.10
    }
}

// ---------------------------------------------------------------------------
// Tag
// ---------------------------------------------------------------------------

/// M3 assist / filter chip: small (4-px) radius, 8-px horizontal
/// padding. Selected chip uses a tinted `primary`.
pub struct MaterialTagRenderer;

impl TagRenderer for MaterialTagRenderer {
    fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            palette::apply_state_layer(theme.surface.base, theme.border.focus, state_layer::PRESSED)
        } else {
            theme.surface.base
        }
    }
    fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.content.primary
        } else {
            theme.content.secondary
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
    fn font_weight(&self, _state: &TagRenderState, _theme: &Theme) -> FontWeight {
        FontWeight::MEDIUM
    }
    fn border_radius(&self, _state: &TagRenderState, _theme: &Theme) -> Pixels {
        px(radii::SM)
    }
    fn close_size(&self, _state: &TagRenderState, _theme: &Theme) -> Pixels {
        px(16.0)
    }
    fn close_hover_bg(&self, _state: &TagRenderState, theme: &Theme) -> Hsla {
        palette::apply_state_layer(theme.surface.base, theme.border.focus, state_layer::HOVER)
    }
}

// ---------------------------------------------------------------------------
// Badge
// ---------------------------------------------------------------------------

/// M3 badge: pill radius, 16-px height, error color background.
pub struct MaterialBadgeRenderer;

impl BadgeRenderer for MaterialBadgeRenderer {
    fn bg(&self, _state: &BadgeRenderState, theme: &Theme) -> Hsla {
        theme.status.error.bg
    }
    fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            theme.content.on_status
        } else {
            theme.status.error.fg
        }
    }
    fn padding_x(&self, _state: &BadgeRenderState, _theme: &Theme) -> Pixels {
        px(6.0)
    }
    fn height(&self, _state: &BadgeRenderState, _theme: &Theme) -> Pixels {
        px(16.0)
    }
    fn font_size(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        theme.tokens.typography.font_size_xs
    }
    fn font_weight(&self, _state: &BadgeRenderState, theme: &Theme) -> FontWeight {
        theme.tokens.typography.weight_medium
    }
    fn border_radius(&self, _state: &BadgeRenderState, _theme: &Theme) -> Pixels {
        px(radii::PILL)
    }
}

// ---------------------------------------------------------------------------
// ListItem
// ---------------------------------------------------------------------------

/// M3 list item: 48-px height, 0-px radius (full-bleed), state
/// layer on hover/select.
pub struct MaterialListItemRenderer;

impl ListItemRenderer for MaterialListItemRenderer {
    fn bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn hover_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        palette::apply_state_layer(theme.surface.base, theme.border.focus, state_layer::HOVER)
    }
    fn selected_bg(&self, _state: &ListItemRenderState, theme: &Theme) -> Hsla {
        palette::apply_state_layer(theme.surface.base, theme.border.focus, state_layer::PRESSED)
    }
    fn fg(&self, state: &ListItemRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
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
            theme.tokens.spacing.inset_md,
            theme.tokens.spacing.inset_xs,
        )
    }
    fn min_height(&self, _state: &ListItemRenderState, _theme: &Theme) -> Pixels {
        px(48.0)
    }
    fn border_radius(&self, _state: &ListItemRenderState, _theme: &Theme) -> Pixels {
        px(0.0)
    }
}

// ---------------------------------------------------------------------------
// Divider
// ---------------------------------------------------------------------------

/// M3 divider: 1-px `outline-variant` line, full bleed.
pub struct MaterialDividerRenderer;

impl DividerRenderer for MaterialDividerRenderer {
    fn color(&self, _state: &DividerRenderState, theme: &Theme) -> Hsla {
        theme.border.divider
    }
    fn thickness(&self, _state: &DividerRenderState, _theme: &Theme) -> Pixels {
        px(1.0)
    }
}

// ---------------------------------------------------------------------------
// Tooltip
// ---------------------------------------------------------------------------

/// M3 tooltip: dark inverse surface, light text, 4-px radius.
pub struct MaterialTooltipRenderer;

impl TooltipRenderer for MaterialTooltipRenderer {
    fn bg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        let mut c = theme.surface.canvas;
        c.l = 0.15;
        c
    }
    fn fg(&self, _state: &TooltipRenderState, _theme: &Theme) -> Hsla {
        gpui::Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.95,
            a: 1.0,
        }
    }
    fn padding(
        &self,
        _state: &TooltipRenderState,
        _theme: &Theme,
    ) -> yororen_ui_core::renderer::Edges<Pixels> {
        yororen_ui_core::renderer::Edges::all(px(8.0))
    }
    fn font_size(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        theme.tokens.typography.font_size_sm
    }
    fn border_radius(&self, _state: &TooltipRenderState, _theme: &Theme) -> Pixels {
        px(radii::SM)
    }
}

// ---------------------------------------------------------------------------
// Popover
// ---------------------------------------------------------------------------

/// M3 menu: 12-px radius, 8-px padding, 2-px elevation shadow.
pub struct MaterialPopoverRenderer;

impl PopoverRenderer for MaterialPopoverRenderer {
    fn bg(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn border(&self, _state: &PopoverRenderState, _theme: &Theme) -> Hsla {
        gpui::Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.0,
        }
    }
    fn shadow_alpha(&self, _state: &PopoverRenderState, _theme: &Theme) -> f32 {
        0.08
    }
    fn border_radius(&self, _state: &PopoverRenderState, _theme: &Theme) -> Pixels {
        px(radii::LG)
    }
    fn offset(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.popover.offset
    }
}

// ---------------------------------------------------------------------------
// Heading
// ---------------------------------------------------------------------------

/// M3 typography scale (simplified). Always uses the same size /
/// weight to keep the API simple; the heading level is read from
/// the state for future differentiation.
pub struct MaterialHeadingRenderer;

impl HeadingRenderer for MaterialHeadingRenderer {
    fn size(&self, _state: &HeadingRenderState, theme: &Theme) -> Pixels {
        let _ = theme;
        px(24.0)
    }
    fn weight(&self, _state: &HeadingRenderState, _theme: &Theme) -> FontWeight {
        FontWeight::MEDIUM
    }
    fn color(&self, _state: &HeadingRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
}

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

/// M3 body / label text. Strong labels get a heavier weight;
/// muted labels get a lighter color. Mono labels return the
/// monospace font family from the theme.
pub struct MaterialLabelRenderer;

impl LabelRenderer for MaterialLabelRenderer {
    fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla {
        if state.inherit_color {
            return theme.content.primary;
        }
        if state.muted {
            theme.content.secondary
        } else {
            theme.content.primary
        }
    }
    fn strong_weight(&self, _state: &LabelRenderState, theme: &Theme) -> FontWeight {
        theme.tokens.typography.weight_semibold
    }
    fn family_mono(&self, _state: &LabelRenderState, theme: &Theme) -> SharedString {
        theme.tokens.typography.family_mono.clone()
    }
}

// ---------------------------------------------------------------------------
// Avatar
// ---------------------------------------------------------------------------

/// M3 avatar: tinted primary background, pill radius for
/// circular avatars.
pub struct MaterialAvatarRenderer;

impl AvatarRenderer for MaterialAvatarRenderer {
    fn default_bg(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        palette::apply_state_layer(theme.surface.base, theme.border.focus, 0.20)
    }
    fn border_radius(&self, state: &AvatarRenderState, theme: &Theme) -> Pixels {
        if state.is_circle {
            theme.tokens.radii.pill
        } else {
            theme.tokens.radii.lg
        }
    }
    fn status_dot_size(&self, _state: &AvatarRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_sm
    }
    fn status_inset(&self, _state: &AvatarRenderState, _theme: &Theme) -> Pixels {
        px(2.0)
    }
    fn status_border_w(&self, _state: &AvatarRenderState, _theme: &Theme) -> Pixels {
        px(1.5)
    }
    fn status_border_color(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

/// Build a `RendererRegistry` with the M3 renderers installed for
/// the 20 components we cover (button / icon_button / label /
/// heading / divider / focus_ring / badge / tag / list_item /
/// switch / checkbox / radio / text_input / modal / popover /
/// toast / tooltip / panel / card / avatar). Other components
/// (combo_box, progress, etc.) fall back to the `TokenXxxRenderer`
/// default.
pub fn material_registry() -> RendererRegistry {
    RendererRegistry::token_based()
        .with_button(Arc::new(MaterialButtonRenderer))
        .with_icon_button(Arc::new(MaterialIconButtonRenderer))
        .with_label(Arc::new(MaterialLabelRenderer))
        .with_heading(Arc::new(MaterialHeadingRenderer))
        .with_divider(Arc::new(MaterialDividerRenderer))
        .with_focus_ring(Arc::new(MaterialFocusRingRenderer))
        .with_badge(Arc::new(MaterialBadgeRenderer))
        .with_tag(Arc::new(MaterialTagRenderer))
        .with_switch(Arc::new(MaterialSwitchRenderer))
        .with_checkbox(Arc::new(MaterialCheckboxRenderer))
        .with_radio(Arc::new(MaterialRadioRenderer))
        .with_text_input(Arc::new(MaterialTextInputRenderer))
        .with_modal(Arc::new(MaterialModalRenderer))
        .with_popover(Arc::new(MaterialPopoverRenderer))
        .with_toast(Arc::new(MaterialToastRenderer))
        .with_tooltip(Arc::new(MaterialTooltipRenderer))
        .with_panel(Arc::new(MaterialPanelRenderer))
        .with_card(Arc::new(MaterialCardRenderer))
        .with_list_item(Arc::new(MaterialListItemRenderer))
        .with_avatar(Arc::new(MaterialAvatarRenderer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use yororen_ui_core::theme::ActionVariantKind;

    fn test_state() -> ButtonRenderState {
        ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        }
    }

    #[test]
    fn material_button_uses_pill_radius() {
        let theme = yororen_ui_theme_system::light();
        let r = MaterialButtonRenderer;
        let state = test_state();
        let radius = r.border_radius(&state, &theme);
        assert!(
            radius.to_f64() > 100.0,
            "M3 button should be pill, got {}",
            radius.to_f64()
        );
    }

    #[test]
    fn material_modal_has_28px_corner() {
        let theme = yororen_ui_theme_system::light();
        let r = MaterialModalRenderer;
        let state = ModalRenderState::default();
        let radius = r.panel_border_radius(&state, &theme);
        assert!(
            (radius.to_f64() - 28.0).abs() < 0.5,
            "M3 dialog corner should be 28, got {}",
            radius.to_f64()
        );
    }

    #[test]
    fn material_switch_uses_primary_when_on() {
        // The Material switch uses `border.focus` (M3 primary) for
        // the on track, regardless of the system theme. The system
        // theme's `border.focus` is the same slot the M3 factory
        // populates with `primary`, so we can compare against it.
        let theme = yororen_ui_theme_system::light();
        let r = MaterialSwitchRenderer;
        let state_on = SwitchRenderState {
            checked: true,
            ..Default::default()
        };
        let bg_on = r.track_bg(&state_on, &theme);
        let focus = theme.border.focus;
        assert_eq!(bg_on, focus);
    }

    #[test]
    fn material_text_input_has_4px_radius() {
        let theme = yororen_ui_theme_system::light();
        let r = MaterialTextInputRenderer;
        let state = TextInputRenderState::default();
        let radius = r.border_radius(&state, &theme);
        assert!(
            (radius.to_f64() - 4.0).abs() < 0.5,
            "M3 text field top corner should be 4, got {}",
            radius.to_f64()
        );
    }

    #[test]
    fn material_renderer_registry_built() {
        let reg = material_registry();
        let theme = yororen_ui_theme_system::light();
        let state = test_state();
        let r1 = reg
            .get_button()
            .expect("ButtonRenderer registered")
            .border_radius(&state, &theme);
        let r2 = MaterialButtonRenderer.border_radius(&state, &theme);
        assert_eq!(r1, r2);
    }
}
