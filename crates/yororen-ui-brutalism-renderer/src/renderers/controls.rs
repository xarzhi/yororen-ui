//! Brutalist control renderers: `Switch`, `Checkbox`, `Radio`.

use gpui::{Hsla, Pixels, px};
use yororen_ui_core::theme::Theme;

use crate::style::{BRUTAL_BORDER, brutal_border_color};

// =====================================================================
// Switch
// =====================================================================

pub use yororen_ui_core::renderer::switch::{SwitchRenderState, SwitchRenderer};

pub struct BrutalSwitchRenderer;

impl SwitchRenderer for BrutalSwitchRenderer {
    fn track_w(&self, _: &SwitchRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.switch.track_w")
            .unwrap_or(52.0) as f32)
    }

    fn track_h(&self, _: &SwitchRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.switch.track_h")
            .unwrap_or(30.0) as f32)
    }

    fn knob_size(&self, _: &SwitchRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.switch.knob_size")
            .unwrap_or(22.0) as f32)
    }

    fn padding(&self, _: &SwitchRenderState, _: &Theme) -> Pixels {
        px(0.0)
    }

    fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
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

    fn track_border(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        let _ = state;
        brutal_border_color(theme)
    }

    fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
        }
    }

    fn track_active_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        }
    }

    fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
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

    fn focus_color(&self, _: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }

    fn disabled_opacity(&self, _: &SwitchRenderState, _: &Theme) -> f32 {
        0.5
    }
}

// =====================================================================
// Checkbox
// =====================================================================

pub use yororen_ui_core::renderer::checkbox::{CheckboxRenderState, CheckboxRenderer};

pub struct BrutalCheckboxRenderer;

impl CheckboxRenderer for BrutalCheckboxRenderer {
    fn box_size(&self, _: &CheckboxRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.checkbox.size")
            .unwrap_or(24.0) as f32)
    }

    fn check_size(&self, _: &CheckboxRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.checkbox.size")
            .unwrap_or(24.0) as f32)
    }

    fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
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

    fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked && state.has_custom_tone {
            state.custom_tone.unwrap_or(BRUTAL_BORDER)
        } else {
            brutal_border_color(theme)
        }
    }

    fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
        }
    }

    fn box_active_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        }
    }

    fn check_fg(&self, _: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn focus_color(&self, _: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }

    fn disabled_opacity(&self, _: &CheckboxRenderState, _: &Theme) -> f32 {
        0.5
    }
}

// =====================================================================
// Radio
// =====================================================================

pub use yororen_ui_core::renderer::radio::{RadioRenderState, RadioRenderer};

pub struct BrutalRadioRenderer;

impl RadioRenderer for BrutalRadioRenderer {
    fn ring_size(&self, _: &RadioRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.radio.size")
            .unwrap_or(24.0) as f32)
    }

    fn dot_size(&self, _: &RadioRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.radio.size")
            .unwrap_or(24.0) as f32)
    }

    fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("surface.base").unwrap_or(BRUTAL_BORDER)
        }
    }

    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked && state.has_custom_tone {
            state.custom_tone.unwrap_or(BRUTAL_BORDER)
        } else {
            brutal_border_color(theme)
        }
    }

    fn ring_hover_bg(&self, _: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }

    fn ring_active_bg(&self, _: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.sunken").unwrap_or(BRUTAL_BORDER)
    }

    fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            state.custom_tone.unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.primary.bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    fn focus_color(&self, _: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }

    fn disabled_opacity(&self, _: &RadioRenderState, _: &Theme) -> f32 {
        0.5
    }
}

// End of radio impl.
