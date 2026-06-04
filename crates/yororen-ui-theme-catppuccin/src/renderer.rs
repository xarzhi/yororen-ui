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

use gpui::{FontWeight, Hsla, Pixels, SharedString, px};

use yororen_ui_core::component::HeadingLevel;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::renderer::{
    AvatarRenderState, AvatarRenderer, BadgeRenderState, BadgeRenderer, ButtonRenderState,
    ButtonRenderer, CardRenderState, CardRenderer, CheckboxRenderState, CheckboxRenderer,
    ComboBoxRenderState, ComboBoxRenderer, DisclosureRenderState, DisclosureRenderer,
    DividerRenderState, DividerRenderer, DropdownMenuRenderState, DropdownMenuRenderer,
    EmptyStateRenderState, EmptyStateRenderer, FilePathInputRenderState, FilePathInputRenderer,
    FocusRingRenderState, FocusRingRenderer, FormRenderState, FormRenderer, HeadingRenderState,
    HeadingRenderer, IconButtonRenderState, IconButtonRenderer, KeybindingInputRenderState,
    KeybindingInputRenderer, LabelRenderState,
    LabelRenderer, ListItemRenderState, ListItemRenderer, ModalRenderState, ModalRenderer,
    NotificationRenderState, NotificationRenderer, NumberInputRenderState, NumberInputRenderer,
    PanelRenderState, PanelRenderer, PasswordInputRenderState, PasswordInputRenderer,
    PopoverRenderState, PopoverRenderer, ProgressBarRenderState, ProgressBarRenderer,
    RadioRenderState, RadioRenderer, RendererRegistry, SearchInputRenderState, SearchInputRenderer,
    SelectRenderState, SelectRenderer, SkeletonRenderState, SkeletonRenderer,
    SplitButtonRenderState, SplitButtonRenderer, SwitchRenderState, SwitchRenderer, TagRenderState,
    TagRenderer, TextAreaRenderState, TextAreaRenderer, TextInputRenderState, TextInputRenderer,
    ToastRenderState, ToastRenderer, ToggleButtonRenderState, ToggleButtonRenderer,
    TooltipRenderState, TooltipRenderer, TreeItemRenderState, TreeItemRenderer,
};
use yororen_ui_core::theme::{ActionVariantKind, Theme};

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
// Panel
// ---------------------------------------------------------------------------

/// Catppuccin panel: `surface.raised` background, `border.default`
/// border, `lg` (16-px) radius, soft shadow. Reuses the same color
/// slots as `Card` since `Panel` is the generic dialog/sheet
/// primitive that `Modal` composes.
pub struct CatppuccinPanelRenderer;

impl PanelRenderer for CatppuccinPanelRenderer {
    fn bg(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn border(&self, _state: &PanelRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn padding(&self, _state: &PanelRenderState, _theme: &Theme) -> Edges<Pixels> {
        // Panel is a generic container; the Modal that uses it
        // overrides padding to 0 because Modal lays out its own
        // title / content / actions spacing. The default 16-px
        // padding is for direct Panel usage.
        Edges::all(px(16.0))
    }
    fn border_radius(&self, _state: &PanelRenderState, _theme: &Theme) -> Pixels {
        px(16.0)
    }
    fn shadow_alpha(&self, _state: &PanelRenderState, _theme: &Theme) -> f32 {
        0.25
    }
}

// ---------------------------------------------------------------------------
// Avatar
// ---------------------------------------------------------------------------

/// Catppuccin avatar: `surface.hover` background, pill radius for
/// circular avatars, `surface.base` status-dot border for clear
/// separation from the avatar surface.
pub struct CatppuccinAvatarRenderer;

impl AvatarRenderer for CatppuccinAvatarRenderer {
    fn default_bg(&self, _state: &AvatarRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
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
// Badge
// ---------------------------------------------------------------------------

/// Catppuccin badge: pill radius, `status.info.bg` background,
/// `status.info.fg` text color.
pub struct CatppuccinBadgeRenderer;

impl BadgeRenderer for CatppuccinBadgeRenderer {
    fn bg(&self, _state: &BadgeRenderState, theme: &Theme) -> Hsla {
        theme.status.info.bg
    }
    fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            theme.content.on_status
        } else {
            theme.status.info.fg
        }
    }
    fn padding_x(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_sm
    }
    fn height(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.badge.min_height
    }
    fn font_size(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        theme.tokens.typography.font_size_xs
    }
    fn font_weight(&self, _state: &BadgeRenderState, theme: &Theme) -> FontWeight {
        theme.tokens.typography.weight_medium
    }
    fn border_radius(&self, _state: &BadgeRenderState, _theme: &Theme) -> Pixels {
        px(999.0)
    }
}

// ---------------------------------------------------------------------------
// Divider
// ---------------------------------------------------------------------------

/// Catppuccin divider: `border.divider` color, 1-px thickness.
pub struct CatppuccinDividerRenderer;

impl DividerRenderer for CatppuccinDividerRenderer {
    fn color(&self, _state: &DividerRenderState, theme: &Theme) -> Hsla {
        theme.border.divider
    }
    fn thickness(&self, _state: &DividerRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.divider.thickness
    }
}

// ---------------------------------------------------------------------------
// Heading
// ---------------------------------------------------------------------------

/// Catppuccin heading: `content.primary` color, level-driven size
/// from tokens, semibold weight.
pub struct CatppuccinHeadingRenderer;

impl HeadingRenderer for CatppuccinHeadingRenderer {
    fn size(&self, state: &HeadingRenderState, theme: &Theme) -> Pixels {
        let t = &theme.tokens.typography;
        // `HeadingLevel` is a `pub` plain fieldless enum, so we
        // can match its variants directly. (No need for
        // `discriminant` or `transmute_copy` of the discriminant —
        // both of those are unsound-ish and never necessary when
        // the enum is public.)
        match state.level {
            HeadingLevel::H1 => t.font_size_2xl,
            HeadingLevel::H2 => t.font_size_xl,
            HeadingLevel::H3 => t.font_size_lg,
        }
    }
    fn weight(&self, state: &HeadingRenderState, theme: &Theme) -> FontWeight {
        match state.level {
            HeadingLevel::H1 => theme.tokens.typography.weight_bold,
            _ => theme.tokens.typography.weight_semibold,
        }
    }
    fn color(&self, _state: &HeadingRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
}

// ---------------------------------------------------------------------------
// Icon
// ---------------------------------------------------------------------------

// The Icon component reads `theme.content.secondary` and the size
// tokens directly; the icon renderer trait was a single trait with
// no state of its own, so it added an indirection layer without
// giving the theme package anything to override.

// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// IconButton
// ---------------------------------------------------------------------------

/// Catppuccin icon button: `surface.hover` background on hover,
/// `md` radius, 0.5 disabled opacity.
pub struct CatppuccinIconButtonRenderer;

impl IconButtonRenderer for CatppuccinIconButtonRenderer {
    fn bg(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Hsla {
        // Static transparent; hover state is drawn separately.
        gpui::hsla(0.0, 0.0, 0.0, 0.0)
    }
    fn hover_bg(&self, _state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn size(&self, _state: &IconButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.icon_button.min_size
    }
    fn border_radius(&self, _state: &IconButtonRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn disabled_opacity(&self, _state: &IconButtonRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

// ---------------------------------------------------------------------------
// ToggleButton
// ---------------------------------------------------------------------------

/// Catppuccin toggle button: action-variant-aware bg, 12-px radius.
pub struct CatppuccinToggleButtonRenderer;

impl ToggleButtonRenderer for CatppuccinToggleButtonRenderer {
    fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.action_variant(state.variant).active_bg
        } else {
            theme.action_variant(state.variant).bg
        }
    }
    fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.action_variant(state.variant).disabled_fg
        } else {
            theme.action_variant(state.variant).fg
        }
    }
    fn min_height(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }
    fn border_radius(&self, _state: &ToggleButtonRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn disabled_opacity(&self, _state: &ToggleButtonRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

// ---------------------------------------------------------------------------
// ProgressBar
// ---------------------------------------------------------------------------

/// Catppuccin progress bar: `surface.hover` track, `action.primary`
/// fill, 8-px radius.
pub struct CatppuccinProgressBarRenderer;

impl ProgressBarRenderer for CatppuccinProgressBarRenderer {
    fn track(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn fill(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.bg
    }
    fn height(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.progress.bar_default_h
    }
    fn border_color(&self, _state: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.border.muted
    }
    fn border_radius(&self, _state: &ProgressBarRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS_SM)
    }
}

// ---------------------------------------------------------------------------
// Skeleton
// ---------------------------------------------------------------------------

/// Catppuccin skeleton: `surface.hover` background, `md` radius.
pub struct CatppuccinSkeletonRenderer;

impl SkeletonRenderer for CatppuccinSkeletonRenderer {
    fn bg(&self, _state: &SkeletonRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn min_height(&self, _state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.skeleton.line_h
    }
    fn border_radius(&self, _state: &SkeletonRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS_SM)
    }
}

// ---------------------------------------------------------------------------
// Tooltip
// ---------------------------------------------------------------------------

/// Catppuccin tooltip: `surface.canvas` background, `content.primary`
/// text, `xs` font size, `sm` radius.
pub struct CatppuccinTooltipRenderer;

impl TooltipRenderer for CatppuccinTooltipRenderer {
    fn bg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme.surface.canvas
    }
    fn fg(&self, _state: &TooltipRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn padding(&self, _state: &TooltipRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(theme.tokens.spacing.inset_sm, theme.tokens.spacing.inset_xs)
    }
    fn font_size(&self, _state: &TooltipRenderState, theme: &Theme) -> Pixels {
        theme.tokens.typography.font_size_xs
    }
    fn border_radius(&self, _state: &TooltipRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS_SM)
    }
}

// ---------------------------------------------------------------------------
// Notification
// ---------------------------------------------------------------------------

/// Catppuccin notification: `surface.raised` background,
/// `border.default` border, 12-px radius, soft shadow.
pub struct CatppuccinNotificationRenderer;

impl NotificationRenderer for CatppuccinNotificationRenderer {
    fn bg(&self, _state: &NotificationRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn border(&self, _state: &NotificationRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn padding(&self, _state: &NotificationRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::all(theme.tokens.spacing.inset_md)
    }
    fn border_radius(&self, _state: &NotificationRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn shadow_alpha(&self, _state: &NotificationRenderState, _theme: &Theme) -> f32 {
        0.30
    }
}

// ---------------------------------------------------------------------------
// Popover
// ---------------------------------------------------------------------------

/// Catppuccin popover: `surface.raised` background, `border.default`
/// border, 12-px radius.
pub struct CatppuccinPopoverRenderer;

impl PopoverRenderer for CatppuccinPopoverRenderer {
    fn bg(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.surface.raised
    }
    fn border(&self, _state: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn shadow_alpha(&self, _state: &PopoverRenderState, _theme: &Theme) -> f32 {
        0.30
    }
    fn border_radius(&self, _state: &PopoverRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn offset(&self, _state: &PopoverRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.popover.offset
    }
}

// ---------------------------------------------------------------------------
// DropdownMenu
// ---------------------------------------------------------------------------

/// Catppuccin dropdown menu: `surface.hover` trigger, `surface.hover`
/// trigger hover, 12-px radius.
pub struct CatppuccinDropdownMenuRenderer;

impl DropdownMenuRenderer for CatppuccinDropdownMenuRenderer {
    fn trigger_bg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn trigger_hover_bg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn trigger_fg(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn min_height(&self, _state: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }
    fn border_radius(&self, _state: &DropdownMenuRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn chevron_rotation(&self, _state: &DropdownMenuRenderState, _theme: &Theme) -> f32 {
        0.0
    }
}

// ---------------------------------------------------------------------------
// Select
// ---------------------------------------------------------------------------

/// Catppuccin select: `surface.base` background, `border.focus`
/// focus border, 12-px radius.
pub struct CatppuccinSelectRenderer;

impl SelectRenderer for CatppuccinSelectRenderer {
    fn bg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn border(&self, _state: &SelectRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &SelectRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn fg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else {
            theme.content.primary
        }
    }
    fn hint_color(&self, _state: &SelectRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
    fn min_height(&self, _state: &SelectRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn padding(&self, _state: &SelectRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn border_radius(&self, _state: &SelectRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn chevron_rotation(&self, _state: &SelectRenderState, _theme: &Theme) -> f32 {
        0.0
    }
}

// ---------------------------------------------------------------------------
// ComboBox
// ---------------------------------------------------------------------------

/// Catppuccin combo box: `surface.base` background, `border.focus`
/// focus border, 12-px radius.
pub struct CatppuccinComboBoxRenderer;

impl ComboBoxRenderer for CatppuccinComboBoxRenderer {
    fn bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn border(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else {
            theme.content.primary
        }
    }
    fn search_bg(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn min_height(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn padding(&self, _state: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn border_radius(&self, _state: &ComboBoxRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
}

// ---------------------------------------------------------------------------
// TextArea
// ---------------------------------------------------------------------------

/// Catppuccin text area: `surface.base` background, `border.focus`
/// focus border, 12-px radius.
pub struct CatppuccinTextAreaRenderer;

impl TextAreaRenderer for CatppuccinTextAreaRenderer {
    fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn border(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else {
            theme.content.primary
        }
    }
    fn min_height(&self, _state: &TextAreaRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height * 2.0
    }
    fn padding(&self, _state: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn border_radius(&self, _state: &TextAreaRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
}

// ---------------------------------------------------------------------------
// NumberInput
// ---------------------------------------------------------------------------

/// Catppuccin number input: `surface.base` background, `surface.hover`
/// stepper background, 12-px radius.
pub struct CatppuccinNumberInputRenderer;

impl NumberInputRenderer for CatppuccinNumberInputRenderer {
    fn bg(&self, state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn stepper_bg(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn stepper_fg(&self, _state: &NumberInputRenderState, theme: &Theme) -> Hsla {
        theme.content.secondary
    }
    fn min_height(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn padding(&self, _state: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn stepper_button_size(&self, _state: &NumberInputRenderState, _theme: &Theme) -> Pixels {
        px(28.0)
    }
    fn stepper_icon_size(&self, _state: &NumberInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_sm
    }
    fn border_radius(&self, _state: &NumberInputRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
}

// ---------------------------------------------------------------------------
// PasswordInput
// ---------------------------------------------------------------------------

/// Catppuccin password input: `surface.base` background,
/// `border.focus` focus border, 12-px radius.
pub struct CatppuccinPasswordInputRenderer;

impl PasswordInputRenderer for CatppuccinPasswordInputRenderer {
    fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn border(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else {
            theme.content.primary
        }
    }
    fn min_height(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn padding(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn border_radius(&self, _state: &PasswordInputRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn toggle_icon_size(&self, _state: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_sm
    }
}

// ---------------------------------------------------------------------------
// FilePathInput
// ---------------------------------------------------------------------------

/// Catppuccin file-path input: `surface.base` background,
/// `surface.hover` browse button, 12-px radius.
pub struct CatppuccinFilePathInputRenderer;

impl FilePathInputRenderer for CatppuccinFilePathInputRenderer {
    fn bg(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn button_bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn button_fg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn min_height(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn padding(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn action_gap(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_sm
    }
    fn border_radius(&self, _state: &FilePathInputRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn icon_size(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_sm
    }
}

// ---------------------------------------------------------------------------
// SearchInput
// ---------------------------------------------------------------------------

/// Catppuccin search input: `surface.base` background,
/// `content.tertiary` icon, 12-px radius.
pub struct CatppuccinSearchInputRenderer;

impl SearchInputRenderer for CatppuccinSearchInputRenderer {
    fn bg(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.surface.sunken
        } else {
            theme.surface.base
        }
    }
    fn border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn icon_color(&self, _state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
    fn fg(&self, state: &SearchInputRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.content.disabled
        } else {
            theme.content.primary
        }
    }
    fn min_height(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn padding(&self, _state: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            theme.tokens.control.input.horizontal_padding,
            theme.tokens.control.input.vertical_padding,
        )
    }
    fn border_radius(&self, _state: &SearchInputRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn input_gap(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_sm
    }
    fn icon_size(&self, _state: &SearchInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_sm
    }
}

// ---------------------------------------------------------------------------
// Disclosure
// ---------------------------------------------------------------------------

/// Catppuccin disclosure: `surface.hover` trigger background,
/// `surface.hover` trigger hover, 12-px radius.
pub struct CatppuccinDisclosureRenderer;

impl DisclosureRenderer for CatppuccinDisclosureRenderer {
    fn trigger_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn trigger_fg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn trigger_hover_bg(&self, _state: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn min_height(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }
    fn border_radius(&self, _state: &DisclosureRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn chevron_rotation(&self, state: &DisclosureRenderState, _theme: &Theme) -> f32 {
        if state.open { 90.0 } else { 0.0 }
    }
    fn body_padding(&self, _state: &DisclosureRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_md
    }
}

// ---------------------------------------------------------------------------
// KeybindingInput
// ---------------------------------------------------------------------------

/// Catppuccin keybinding input: `surface.base` background,
/// `surface.hover` kbd background, 12-px radius.
pub struct CatppuccinKeybindingInputRenderer;

impl KeybindingInputRenderer for CatppuccinKeybindingInputRenderer {
    fn bg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.border.default
    }
    fn focus_border(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn kbd_bg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn kbd_fg(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn kbd_padding(&self, _state: &KeybindingInputRenderState, _theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(px(6.0), px(2.0))
    }
    fn kbd_min_width(&self, _state: &KeybindingInputRenderState, _theme: &Theme) -> Pixels {
        px(24.0)
    }
    fn min_height(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.input.min_height
    }
    fn border_radius(&self, _state: &KeybindingInputRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn icon_size(&self, _state: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_sm
    }
}

// ---------------------------------------------------------------------------
// SplitButton
// ---------------------------------------------------------------------------

/// Catppuccin split button: `action.primary.bg` primary,
/// `action.primary.active_bg` chevron, 12-px radius.
pub struct CatppuccinSplitButtonRenderer;

impl SplitButtonRenderer for CatppuccinSplitButtonRenderer {
    fn primary_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.bg
    }
    fn primary_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.fg
    }
    fn chevron_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.active_bg
    }
    fn chevron_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.fg
    }
    fn chevron_hover_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme.action.primary.hover_bg
    }
    fn min_height(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }
    fn border_radius(&self, _state: &SplitButtonRenderState, _theme: &Theme) -> Pixels {
        px(CATPPUCCIN_RADIUS)
    }
    fn gap(&self, _state: &SplitButtonRenderState, _theme: &Theme) -> Pixels {
        px(2.0)
    }
}

// ---------------------------------------------------------------------------
// Form
// ---------------------------------------------------------------------------

/// Catppuccin form: token-driven gap, status colors for
/// error/helper.
pub struct CatppuccinFormRenderer;

impl FormRenderer for CatppuccinFormRenderer {
    fn gap(&self, _state: &FormRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_md
    }
    fn label_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.content.primary
    }
    fn error_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.status.error.bg
    }
    fn helper_color(&self, _state: &FormRenderState, theme: &Theme) -> Hsla {
        theme.content.tertiary
    }
}

// ---------------------------------------------------------------------------
// TreeItem
// ---------------------------------------------------------------------------

/// Catppuccin tree item: `surface.base` bg, `surface.hover` hover,
/// `border.focus` selected.
pub struct CatppuccinTreeItemRenderer;

impl TreeItemRenderer for CatppuccinTreeItemRenderer {
    fn bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.surface.base
    }
    fn hover_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.surface.hover
    }
    fn selected_bg(&self, _state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        theme.border.focus
    }
    fn fg(&self, state: &TreeItemRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme.content.on_primary
        } else {
            theme.content.primary
        }
    }
    fn indent(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        theme.tokens.spacing.inset_md
    }
    fn padding(&self, _state: &TreeItemRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(theme.tokens.spacing.inset_sm, theme.tokens.spacing.inset_xs)
    }
    fn min_height(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.list_item.min_height
    }
    fn chevron_size(&self, _state: &TreeItemRenderState, theme: &Theme) -> Pixels {
        theme.tokens.sizes.icon_sm
    }
}

// ---------------------------------------------------------------------------
// Label
// ---------------------------------------------------------------------------

/// Catppuccin label: `content.primary` color, semibold weight when
/// `strong` is set.
pub struct CatppuccinLabelRenderer;

impl LabelRenderer for CatppuccinLabelRenderer {
    fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla {
        if state.muted {
            theme.content.tertiary
        } else {
            theme.content.primary
        }
    }
    fn strong_weight(&self, _state: &LabelRenderState, theme: &Theme) -> FontWeight {
        theme.tokens.typography.weight_semibold
    }
    fn family_mono(&self, _state: &LabelRenderState, _theme: &Theme) -> SharedString {
        SharedString::from("monospace")
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
        // First batch of renderers: button, card, modal,
        // focus_ring, text_input, switch, checkbox, radio, toast,
        // tag, list_item, empty_state.
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
        // Second batch closes the gap to a complete skin.
        .with_avatar(Arc::new(CatppuccinAvatarRenderer))
        .with_panel(Arc::new(CatppuccinPanelRenderer))
        .with_badge(Arc::new(CatppuccinBadgeRenderer))
        .with_divider(Arc::new(CatppuccinDividerRenderer))
        .with_heading(Arc::new(CatppuccinHeadingRenderer))
        .with_icon_button(Arc::new(CatppuccinIconButtonRenderer))
        .with_toggle_button(Arc::new(CatppuccinToggleButtonRenderer))
        .with_progress_bar(Arc::new(CatppuccinProgressBarRenderer))
        .with_skeleton(Arc::new(CatppuccinSkeletonRenderer))
        .with_tooltip(Arc::new(CatppuccinTooltipRenderer))
        .with_notification(Arc::new(CatppuccinNotificationRenderer))
        .with_popover(Arc::new(CatppuccinPopoverRenderer))
        .with_dropdown_menu(Arc::new(CatppuccinDropdownMenuRenderer))
        .with_select(Arc::new(CatppuccinSelectRenderer))
        .with_combo_box(Arc::new(CatppuccinComboBoxRenderer))
        .with_text_area(Arc::new(CatppuccinTextAreaRenderer))
        .with_number_input(Arc::new(CatppuccinNumberInputRenderer))
        .with_password_input(Arc::new(CatppuccinPasswordInputRenderer))
        .with_file_path_input(Arc::new(CatppuccinFilePathInputRenderer))
        .with_search_input(Arc::new(CatppuccinSearchInputRenderer))
        .with_disclosure(Arc::new(CatppuccinDisclosureRenderer))
        .with_keybinding_input(Arc::new(CatppuccinKeybindingInputRenderer))
        .with_split_button(Arc::new(CatppuccinSplitButtonRenderer))
        .with_form(Arc::new(CatppuccinFormRenderer))
        .with_tree_item(Arc::new(CatppuccinTreeItemRenderer))
        .with_label(Arc::new(CatppuccinLabelRenderer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::palette;
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
        assert_ne!(
            light_bg, dark_bg,
            "light and dark should produce different card bg"
        );
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
        let state = SwitchRenderState {
            checked: true,
            ..Default::default()
        };
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
        let state = CheckboxRenderState {
            checked: true,
            ..Default::default()
        };
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
        let _ = reg.get_button().expect("ButtonRenderer registered").bg(&ButtonRenderState::default(), &cat_light());
        let _ = reg.get_card().expect("CardRenderer registered").bg(&CardRenderState::default(), &cat_light());
        let _ = reg
            .get_modal().expect("ModalRenderer registered")
            .panel_bg(&ModalRenderState::default(), &cat_light());
        let _ = reg
            .get_focus_ring().expect("FocusRingRenderer registered")
            .color(&FocusRingRenderState::default(), &cat_light());
        let _ = reg
            .get_text_input().expect("TextInputRenderer registered")
            .bg(&TextInputRenderState::default(), &cat_light());
        let _ = reg
            .get_switch().expect("SwitchRenderer registered")
            .track_bg(&SwitchRenderState::default(), &cat_light());
        let _ = reg
            .get_checkbox().expect("CheckboxRenderer registered")
            .box_bg(&CheckboxRenderState::default(), &cat_light());
        let _ = reg
            .get_radio().expect("RadioRenderer registered")
            .ring_bg(&RadioRenderState::default(), &cat_light());
        let _ = reg.get_toast().expect("ToastRenderer registered").bg(&ToastRenderState::default(), &cat_light());
        let _ = reg.get_tag().expect("TagRenderer registered").bg(&TagRenderState::default(), &cat_light());
        let _ = reg
            .get_list_item().expect("ListItemRenderer registered")
            .bg(&ListItemRenderState::default(), &cat_light());
        let _ = reg
            .get_empty_state().expect("EmptyStateRenderer registered")
            .icon_color(&EmptyStateRenderState::default(), &cat_light());
    }

    /// Every entry that ships a Catppuccin variant
    /// are wired into the registry and can be invoked without
    /// panicking. This is a smoke test — it doesn't verify the
    /// output, just that the trait methods are callable.
    #[test]
    fn registry_includes_new_renderers() {
        use yororen_ui_core::renderer::{
            AvatarRenderState, BadgeRenderState, ComboBoxRenderState, DisclosureRenderState,
            DividerRenderState, DropdownMenuRenderState, FilePathInputRenderState, FormRenderState,
            HeadingRenderState, IconButtonRenderState, KeybindingInputRenderState,
            LabelRenderState, NotificationRenderState, NumberInputRenderState,
            PasswordInputRenderState, PopoverRenderState, ProgressBarRenderState,
            SearchInputRenderState, SelectRenderState, SkeletonRenderState, SplitButtonRenderState,
            TextAreaRenderState, ToggleButtonRenderState, TooltipRenderState, TreeItemRenderState,
        };
        let reg = catppuccin_registry();
        let theme = cat_light();
        let _ = reg.get_avatar().expect("AvatarRenderer registered").default_bg(&AvatarRenderState::default(), &theme);
        let _ = reg.get_badge().expect("BadgeRenderer registered").bg(&BadgeRenderState::default(), &theme);
        let _ = reg.get_divider().expect("DividerRenderer registered").color(&DividerRenderState::default(), &theme);
        let _ = reg.get_heading().expect("HeadingRenderer registered").size(
            &HeadingRenderState {
                level: HeadingLevel::H1,
            },
            &theme,
        );
        let _ = reg
            .get_icon_button().expect("IconButtonRenderer registered")
            .bg(&IconButtonRenderState::default(), &theme);
        let _ = reg
            .get_toggle_button().expect("ToggleButtonRenderer registered")
            .bg(&ToggleButtonRenderState::default(), &theme);
        let _ = reg
            .get_progress_bar().expect("ProgressBarRenderer registered")
            .track(&ProgressBarRenderState::default(), &theme);
        let _ = reg.get_skeleton().expect("SkeletonRenderer registered").bg(&SkeletonRenderState::default(), &theme);
        let _ = reg.get_tooltip().expect("TooltipRenderer registered").bg(&TooltipRenderState::default(), &theme);
        let _ = reg
            .get_notification().expect("NotificationRenderer registered")
            .bg(&NotificationRenderState::default(), &theme);
        let _ = reg.get_popover().expect("PopoverRenderer registered").bg(&PopoverRenderState::default(), &theme);
        let _ = reg
            .get_dropdown_menu().expect("DropdownMenuRenderer registered")
            .trigger_bg(&DropdownMenuRenderState::default(), &theme);
        let _ = reg.get_select().expect("SelectRenderer registered").bg(&SelectRenderState::default(), &theme);
        let _ = reg.get_combo_box().expect("ComboBoxRenderer registered").bg(&ComboBoxRenderState::default(), &theme);
        let _ = reg.get_text_area().expect("TextAreaRenderer registered").bg(&TextAreaRenderState::default(), &theme);
        let _ = reg
            .get_number_input().expect("NumberInputRenderer registered")
            .bg(&NumberInputRenderState::default(), &theme);
        let _ = reg
            .get_password_input().expect("PasswordInputRenderer registered")
            .bg(&PasswordInputRenderState::default(), &theme);
        let _ = reg
            .get_file_path_input().expect("FilePathInputRenderer registered")
            .bg(&FilePathInputRenderState::default(), &theme);
        let _ = reg
            .get_search_input().expect("SearchInputRenderer registered")
            .bg(&SearchInputRenderState::default(), &theme);
        let _ = reg
            .get_disclosure().expect("DisclosureRenderer registered")
            .trigger_bg(&DisclosureRenderState::default(), &theme);
        let _ = reg
            .get_keybinding_input().expect("KeybindingInputRenderer registered")
            .bg(&KeybindingInputRenderState::default(), &theme);
        let _ = reg
            .get_split_button().expect("SplitButtonRenderer registered")
            .primary_bg(&SplitButtonRenderState::default(), &theme);
        let _ = reg.get_form().expect("FormRenderer registered").gap(&FormRenderState::default(), &theme);
        let _ = reg.get_tree_item().expect("TreeItemRenderer registered").bg(&TreeItemRenderState::default(), &theme);
        let _ = reg.get_label().expect("LabelRenderer registered").color(&LabelRenderState::default(), &theme);
    }
}

#[cfg(test)]
mod panel_tests {
    use super::*;
    use yororen_ui_core::renderer::PanelRenderState;

    fn cat_light() -> Theme {
        crate::factories::light()
    }
    fn cat_dark() -> Theme {
        crate::factories::dark()
    }

    #[test]
    fn catppuccin_panel_renderer_uses_theme_surface() {
        let r = CatppuccinPanelRenderer;
        let light = cat_light();
        let dark = cat_dark();
        let state = PanelRenderState::default();
        // Sanity: light/dark produce different bg.
        assert_ne!(r.bg(&state, &light), r.bg(&state, &dark));
        // The renderer reads theme.surface.raised (the same slot
        // Modal uses for its bg) so the visual is consistent
        // between Modal and direct Panel usage.
        assert_eq!(r.bg(&state, &dark), dark.surface.raised);
    }
}
