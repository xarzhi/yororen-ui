//! `SwitchRenderer` — the visual side of `Switch`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::switch::{SwitchRenderState, SwitchRenderer};

pub struct TokenSwitchRenderer;

impl SwitchRenderer for TokenSwitchRenderer {
    fn track_w(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.track_w")
                .unwrap_or(0.0) as f32,
        )
    }
    fn track_h(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.track_h")
                .unwrap_or(0.0) as f32,
        )
    }
    fn knob_size(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.knob_size")
                .unwrap_or(0.0) as f32,
        )
    }
    fn padding(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.padding")
                .unwrap_or(0.0) as f32,
        )
    }

    fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else if state.checked {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or_default()
            } else {
                theme.get_color("action.primary.bg").unwrap_or_default()
            }
        } else {
            theme.get_color("surface.hover").unwrap_or_default()
        }
    }
    fn track_border(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            theme.get_color("border.default").unwrap_or_default()
        }
    }
    fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or_default()
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    fn track_active_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or_default()
        } else {
            theme.get_color("surface.sunken").unwrap_or_default()
        }
    }
    fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.checked {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    fn focus_color(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    fn disabled_opacity(&self, _state: &SwitchRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

pub fn arc_switch<T: SwitchRenderer + 'static>(r: T) -> Arc<dyn SwitchRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultSwitch` — `headless::SwitchProps` sugar.
// =====================================================================

use gpui::{
    App, InteractiveElement, ParentElement, Stateful, StatefulInteractiveElement, Styled, div,
};
use yororen_ui_core::headless::switch::SwitchProps;
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::ActiveTheme;

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/system-light.json");
        Theme::from_json(json).expect("system-light.json is valid")
    }

    #[test]
    fn track_w_h_knob_size_padding_read_switch_tokens() {
        let theme = fixture();
        let r = TokenSwitchRenderer;
        let state = SwitchRenderState::default();
        assert_eq!(
            r.track_w(&state, &theme),
            gpui::px(
                theme
                    .get_number("tokens.control.switch.track_w")
                    .unwrap_or(0.0) as f32
            ),
        );
        assert_eq!(
            r.track_h(&state, &theme),
            gpui::px(
                theme
                    .get_number("tokens.control.switch.track_h")
                    .unwrap_or(0.0) as f32
            ),
        );
        assert_eq!(
            r.knob_size(&state, &theme),
            gpui::px(
                theme
                    .get_number("tokens.control.switch.knob_size")
                    .unwrap_or(0.0) as f32
            ),
        );
    }

    #[test]
    fn track_bg_uses_action_primary_when_checked() {
        let theme = fixture();
        let r = TokenSwitchRenderer;
        let state = SwitchRenderState {
            checked: true,
            ..Default::default()
        };
        assert_eq!(
            r.track_bg(&state, &theme),
            theme.get_color("action.primary.bg").unwrap(),
        );
    }

    #[test]
    fn track_bg_uses_surface_hover_when_unchecked() {
        let theme = fixture();
        let r = TokenSwitchRenderer;
        let state = SwitchRenderState {
            checked: false,
            ..Default::default()
        };
        assert_eq!(
            r.track_bg(&state, &theme),
            theme.get_color("surface.hover").unwrap(),
        );
    }

    #[test]
    fn disabled_state_doesnt_panic() {
        let theme = fixture();
        let r = TokenSwitchRenderer;
        let state = SwitchRenderState {
            disabled: true,
            ..Default::default()
        };
        // All methods should not panic on a disabled state.
        let _ = r.track_bg(&state, &theme);
        let _ = r.knob_bg(&state, &theme);
        assert_eq!(r.disabled_opacity(&state, &theme), 0.5);
    }

    #[test]
    fn custom_tone_overrides_checked_track_color() {
        let theme = fixture();
        let r = TokenSwitchRenderer;
        let custom = gpui::rgb(0xdeadbe).into();
        let state = SwitchRenderState {
            checked: true,
            disabled: false,
            has_custom_tone: true,
            custom_tone: Some(custom),
        };
        assert_eq!(r.track_bg(&state, &theme), custom);
    }
}
