//! Brutalist control renderers: `Switch`, `Checkbox`, `Radio`.

use gpui::{
    App, Div, FocusHandle, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;

use crate::style::{BRUTAL_BORDER, brutal_border_color};

// =====================================================================
// Switch
// =====================================================================

pub use yororen_ui_core::renderer::switch::{SwitchRenderState, SwitchRenderer};

pub struct BrutalSwitchRenderer;

// Inherent helpers — *not* part of the `SwitchRenderer` trait surface.
impl BrutalSwitchRenderer {
    pub fn track_w(&self, _: &SwitchRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.switch.track_w")
            .unwrap_or(52.0) as f32)
    }
    pub fn track_h(&self, _: &SwitchRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.switch.track_h")
            .unwrap_or(30.0) as f32)
    }
    pub fn knob_size(&self, _: &SwitchRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.switch.knob_size")
            .unwrap_or(22.0) as f32)
    }
    pub fn padding(&self, _: &SwitchRenderState, _: &Theme) -> Pixels {
        px(0.0)
    }
    pub fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        } else if state.checked {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or(BRUTAL_BORDER)
            } else {
                theme
                    .get_color("action.primary.bg")
                    .unwrap_or(BRUTAL_BORDER)
            }
        } else {
            theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn track_border(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    pub fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn track_active_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or(BRUTAL_BORDER)
        } else if state.checked {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn focus_color(&self, _: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }
    pub fn disabled_opacity(&self, _: &SwitchRenderState, _: &Theme) -> f32 {
        0.5
    }
}

impl SwitchRenderer for BrutalSwitchRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::switch::SwitchProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = SwitchRenderState {
            checked: props.checked,
            disabled: props.disabled,
            has_custom_tone: props.has_custom_tone,
            custom_tone: props.custom_tone,
        };
        let track = self.track_bg(&state, theme);
        let knob = self.knob_bg(&state, theme);
        let w = self.track_w(&state, theme);
        let h = self.track_h(&state, theme);
        let knob_size = self.knob_size(&state, theme);
        let pad = self.padding(&state, theme);
        let track_hover = self.track_hover_bg(&state, theme);
        let track_active = self.track_active_bg(&state, theme);
        let border = brutal_border_color(theme);

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(track)
            .border_2()
            .border_color(border)
            .w(w)
            .h(h)
            .p(pad)
            .flex()
            .items_center()
            .track_focus(focus_handle);
        if props.checked {
            el = el.justify_end();
        } else {
            el = el.justify_start();
        }
        el = el.child(div().bg(knob).size(knob_size));
        el.hover(|s| s.bg(track_hover))
            .active(|s| s.bg(track_active))
    }
}

// =====================================================================
// Checkbox
// =====================================================================

pub use yororen_ui_core::renderer::checkbox::{CheckboxRenderState, CheckboxRenderer};

pub struct BrutalCheckboxRenderer;

// Inherent helpers — *not* part of the `CheckboxRenderer` trait surface.
impl BrutalCheckboxRenderer {
    pub fn box_size(&self, _: &CheckboxRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.checkbox.size")
            .unwrap_or(24.0) as f32)
    }
    pub fn check_size(&self, _: &CheckboxRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.checkbox.size")
            .unwrap_or(24.0) as f32)
    }
    pub fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        } else if state.checked {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or(BRUTAL_BORDER)
            } else {
                theme
                    .get_color("action.primary.bg")
                    .unwrap_or(BRUTAL_BORDER)
            }
        } else {
            theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked && state.has_custom_tone {
            state.custom_tone.unwrap_or(BRUTAL_BORDER)
        } else {
            brutal_border_color(theme)
        }
    }
    pub fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn box_active_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn check_fg(&self, _: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn focus_color(&self, _: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }
    pub fn disabled_opacity(&self, _: &CheckboxRenderState, _: &Theme) -> f32 {
        0.5
    }
}

impl CheckboxRenderer for BrutalCheckboxRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::checkbox::CheckboxProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = CheckboxRenderState {
            checked: props.checked,
            disabled: props.disabled,
            has_custom_tone: props.has_custom_tone,
            custom_tone: props.custom_tone,
        };
        let bg = self.box_bg(&state, theme);
        let border = self.box_border(&state, theme);
        let size = self.box_size(&state, theme);
        let check_size = self.check_size(&state, theme);
        let hover_bg = self.box_hover_bg(&state, theme);
        let active_bg = self.box_active_bg(&state, theme);

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_2()
            .border_color(border)
            .size(size)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);
        if props.checked {
            el = el.child(div().bg(border).size(check_size));
        }
        el.hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}

// =====================================================================
// Radio
// =====================================================================

pub use yororen_ui_core::renderer::radio::{RadioRenderState, RadioRenderer};

pub struct BrutalRadioRenderer;

// Inherent helpers — *not* part of the `RadioRenderer` trait surface.
impl BrutalRadioRenderer {
    pub fn ring_size(&self, _: &RadioRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.radio.size")
            .unwrap_or(24.0) as f32)
    }
    pub fn dot_size(&self, _: &RadioRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.radio.size")
            .unwrap_or(24.0) as f32)
    }
    pub fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked && state.has_custom_tone {
            state.custom_tone.unwrap_or(BRUTAL_BORDER)
        } else {
            brutal_border_color(theme)
        }
    }
    pub fn ring_hover_bg(&self, _: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn ring_active_bg(&self, _: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
    }
    pub fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            state.custom_tone.unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.primary.bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }
    pub fn focus_color(&self, _: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }
    pub fn disabled_opacity(&self, _: &RadioRenderState, _: &Theme) -> f32 {
        0.5
    }
}

impl RadioRenderer for BrutalRadioRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::radio::RadioProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = RadioRenderState {
            checked: props.checked,
            disabled: props.disabled,
            has_custom_tone: props.has_custom_tone,
            custom_tone: props.custom_tone,
        };
        let bg = self.ring_bg(&state, theme);
        let border = self.ring_border(&state, theme);
        let ring_size = self.ring_size(&state, theme);
        let dot_size = self.dot_size(&state, theme);
        let dot_fg = self.dot_fg(&state, theme);
        let hover_bg = self.ring_hover_bg(&state, theme);
        let active_bg = self.ring_active_bg(&state, theme);

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .border_2()
            .border_color(border)
            .size(ring_size)
            .rounded(px(9999.))
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);
        if props.checked {
            el = el.child(div().bg(dot_fg).size(dot_size).rounded(px(9999.)));
        }
        el.hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}
