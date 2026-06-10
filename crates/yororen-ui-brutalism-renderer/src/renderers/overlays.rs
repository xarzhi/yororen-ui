//! Brutalist overlay renderers: `Modal`, `Popover`,
//! `DropdownMenu`, `Disclosure`.

use gpui::{App, Div, Hsla, ParentElement, Pixels, Styled, div, px};
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::style::{BRUTAL_BORDER, BRUTAL_RADIUS, brutal_border_color};

// =====================================================================
// Modal
// =====================================================================

pub use yororen_ui_core::renderer::modal::{ModalRenderState, ModalRenderer};

pub struct BrutalModalRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalModalRenderer {
    pub fn scrim(&self, _: &ModalRenderState, _: &Theme) -> Hsla {
        Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.5,
        }
    }
    pub fn panel_bg(&self, _: &ModalRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }
    pub fn panel_border(&self, _: &ModalRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn panel_padding(&self, _: &ModalRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.modal.padding")
            .unwrap_or(24.0) as f32;
        Edges::all(px(p))
    }
    pub fn panel_border_radius(&self, _: &ModalRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn panel_shadow_alpha(&self, _: &ModalRenderState, _: &Theme) -> f32 {
        1.0
    }
}

impl ModalRenderer for BrutalModalRenderer {
    fn compose(
        &self,
        _props: &yororen_ui_core::headless::modal::ModalProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ModalRenderState {};
        let panel_bg = self.panel_bg(&state, theme);
        let panel_border = self.panel_border(&state, theme);
        let pad = self.panel_padding(&state, theme);
        let r = self.panel_border_radius(&state, theme);
        gpui::div()
            .flex()
            .items_center()
            .justify_center()
            .size_full()
            .bg(self.scrim(&state, theme))
            .child(
                gpui::div()
                    .bg(panel_bg)
                    .border_color(panel_border)
                    .p(pad.top)
                    .rounded(r),
            )
    }
}

// =====================================================================
// Popover
// =====================================================================

pub use yororen_ui_core::renderer::popover::{PopoverRenderState, PopoverRenderer};

pub struct BrutalPopoverRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalPopoverRenderer {
    pub fn bg(&self, _: &PopoverRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.raised").unwrap_or(BRUTAL_BORDER)
    }
    pub fn border(&self, _: &PopoverRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn shadow_alpha(&self, _: &PopoverRenderState, _: &Theme) -> f32 {
        1.0
    }
    pub fn border_radius(&self, _: &PopoverRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn offset(&self, _: &PopoverRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.popover.offset")
            .unwrap_or(8.0) as f32)
    }
}

impl PopoverRenderer for BrutalPopoverRenderer {
    fn compose(
        &self,
        _props: &yororen_ui_core::headless::popover::PopoverProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = PopoverRenderState {};
        let bg = self.bg(&state, theme);
        let border = self.border(&state, theme);
        let r = self.border_radius(&state, theme);
        gpui::div().bg(bg).border_color(border).rounded(r)
    }
}

// =====================================================================
// DropdownMenu
// =====================================================================

pub use yororen_ui_core::renderer::dropdown_menu::{DropdownMenuRenderState, DropdownMenuRenderer};

pub struct BrutalDropdownMenuRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalDropdownMenuRenderer {
    pub fn trigger_bg(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn trigger_hover_bg(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn trigger_fg(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn min_height(&self, _: &DropdownMenuRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.dropdown_menu.min_height")
            .unwrap_or(44.0) as f32)
    }
    pub fn border_radius(&self, _: &DropdownMenuRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn chevron_rotation(&self, state: &DropdownMenuRenderState, _: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

impl DropdownMenuRenderer for BrutalDropdownMenuRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::dropdown_menu::DropdownMenuProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DropdownMenuRenderState {
            open: props.state.read(cx).is_open(),
        };
        let bg = self.trigger_bg(&state, theme);
        let fg = self.trigger_fg(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        gpui::div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .min_h(h)
            .rounded(r)
            .child("▼")
    }
}

// =====================================================================
// Disclosure
// =====================================================================

pub use yororen_ui_core::renderer::disclosure::{DisclosureRenderState, DisclosureRenderer};

pub struct BrutalDisclosureRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalDisclosureRenderer {
    pub fn trigger_bg(&self, _: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn trigger_fg(&self, _: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn trigger_hover_bg(&self, _: &DisclosureRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn min_height(&self, _: &DisclosureRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.disclosure.min_height")
            .unwrap_or(44.0) as f32)
    }
    pub fn border_radius(&self, _: &DisclosureRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn chevron_rotation(&self, state: &DisclosureRenderState, _: &Theme) -> f32 {
        if state.open { 90.0 } else { 0.0 }
    }
    pub fn body_padding(&self, _: &DisclosureRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.disclosure.padding")
            .unwrap_or(12.0) as f32)
    }
}

impl DisclosureRenderer for BrutalDisclosureRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::disclosure::DisclosureProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DisclosureRenderState { open: props.open };
        let bg = self.trigger_bg(&state, theme);
        let fg = self.trigger_fg(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        let chev_str = if props.open { "▼" } else { "▶" };
        div()
            .flex()
            .items_center()
            .gap(px(8.0))
            .bg(bg)
            .text_color(fg)
            .min_h(h)
            .rounded(r)
            .px(px(12.0))
            .child(chev_str)
            .child(props.title.clone())
    }
}
