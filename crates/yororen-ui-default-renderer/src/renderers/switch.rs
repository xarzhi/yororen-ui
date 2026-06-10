//! `TokenSwitchRenderer` — default `SwitchRenderer` impl.

use std::sync::Arc;

use gpui::{
    App, Div, FocusHandle, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};

use yororen_ui_core::headless::switch::SwitchProps;
use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::switch::{SwitchRenderState, SwitchRenderer};

pub struct TokenSwitchRenderer;

// Inherent helpers — *not* part of the `SwitchRenderer` trait
// surface.
impl TokenSwitchRenderer {
    pub fn track_w(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.track_w")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn track_h(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.track_h")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn knob_size(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.knob_size")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn padding(&self, _state: &SwitchRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.switch.padding")
                .unwrap_or(0.0) as f32,
        )
    }

    pub fn track_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
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
    pub fn track_border(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme.get_color("border.muted").unwrap_or_default()
        } else {
            theme.get_color("border.default").unwrap_or_default()
        }
    }
    pub fn track_hover_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or_default()
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    pub fn track_active_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or_default()
        } else {
            theme.get_color("surface.sunken").unwrap_or_default()
        }
    }
    pub fn knob_bg(&self, state: &SwitchRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("content.disabled").unwrap_or_default()
        } else if state.checked {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }
    pub fn focus_color(&self, _state: &SwitchRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    pub fn disabled_opacity(&self, _state: &SwitchRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

impl SwitchRenderer for TokenSwitchRenderer {
    fn compose(
        &self,
        props: &SwitchProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
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
        let pill_radius = px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32);
        let track_hover = self.track_hover_bg(&state, theme);
        let track_active = self.track_active_bg(&state, theme);

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(track)
            .w(w)
            .h(h)
            .rounded(pill_radius)
            .p(pad)
            .flex()
            .items_center()
            .track_focus(focus_handle);
        if props.checked {
            el = el.justify_end();
        } else {
            el = el.justify_start();
        }
        el = el.child(div().bg(knob).size(knob_size).rounded(pill_radius));
        el.hover(|s| s.bg(track_hover))
            .active(|s| s.bg(track_active))
    }
}

pub fn arc_switch<T: SwitchRenderer + 'static>(r: T) -> Arc<dyn SwitchRenderer> {
    Arc::new(r)
}

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
