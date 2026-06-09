//! Brutalist overlay renderers: `Modal`, `Popover`,
//! `DropdownMenu`, `Disclosure`.

use gpui::{Hsla, Pixels, px};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::Theme;

use crate::style::{BRUTAL_BORDER, BRUTAL_RADIUS, brutal_border_color};

// =====================================================================
// Modal
// =====================================================================

pub use yororen_ui_core::renderer::modal::{ModalRenderState, ModalRenderer};

pub struct BrutalModalRenderer;

impl ModalRenderer for BrutalModalRenderer {
    fn scrim(&self, _: &ModalRenderState, _: &Theme) -> Hsla {
        // Half-transparent black scrim.
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.5,
        }
    }
    fn panel_bg(&self, _: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }
    fn panel_border(&self, _: &ModalRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn panel_padding(&self, _: &ModalRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.modal.padding")
            .unwrap_or(24.0) as f32;
        Edges::all(px(p))
    }
    fn panel_border_radius(&self, _: &ModalRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn panel_shadow_alpha(&self, _: &ModalRenderState, _: &Theme) -> f32 {
        1.0
    }
}

// =====================================================================
// Popover
// =====================================================================

pub use yororen_ui_core::renderer::popover::{PopoverRenderState, PopoverRenderer};

pub struct BrutalPopoverRenderer;

impl PopoverRenderer for BrutalPopoverRenderer {
    fn bg(&self, _: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }
    fn border(&self, _: &PopoverRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn shadow_alpha(&self, _: &PopoverRenderState, _: &Theme) -> f32 {
        1.0
    }
    fn border_radius(&self, _: &PopoverRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn offset(&self, _: &PopoverRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.popover.offset")
            .unwrap_or(8.0) as f32)
    }
}

// =====================================================================
// DropdownMenu
// =====================================================================

pub use yororen_ui_core::renderer::dropdown_menu::{DropdownMenuRenderState, DropdownMenuRenderer};

pub struct BrutalDropdownMenuRenderer;

impl DropdownMenuRenderer for BrutalDropdownMenuRenderer {
    fn trigger_bg(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn trigger_hover_bg(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn trigger_fg(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn min_height(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.dropdown_menu.min_height")
            .unwrap_or(44.0) as f32)
    }
    fn border_radius(&self, _: &DropdownMenuRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn chevron_rotation(&self, state: &DropdownMenuRenderState, _: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

// =====================================================================
// Disclosure
// =====================================================================

pub use yororen_ui_core::renderer::disclosure::{DisclosureRenderState, DisclosureRenderer};

pub struct BrutalDisclosureRenderer;

impl DisclosureRenderer for BrutalDisclosureRenderer {
    fn trigger_bg(&self, _: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn trigger_fg(&self, _: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn trigger_hover_bg(&self, _: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn min_height(&self, _: &DisclosureRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.disclosure.min_height")
            .unwrap_or(44.0) as f32)
    }
    fn border_radius(&self, _: &DisclosureRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn chevron_rotation(&self, state: &DisclosureRenderState, _: &Theme) -> f32 {
        if state.open { 90.0 } else { 0.0 }
    }
    fn body_padding(&self, _: &DisclosureRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.disclosure.padding")
            .unwrap_or(12.0) as f32)
    }
}
