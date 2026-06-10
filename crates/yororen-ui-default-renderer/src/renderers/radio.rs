//! `RadioRenderer` — the visual side of `Radio`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::radio::{RadioRenderState, RadioRenderer};

pub struct TokenRadioRenderer;

impl RadioRenderer for TokenRadioRenderer {
    fn ring_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.radio.ring_size")
                .unwrap_or(0.0) as f32,
        )
    }
    fn dot_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.radio.dot_size")
                .unwrap_or(0.0) as f32,
        )
    }
    fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or_default()
            } else {
                theme.get_color("action.primary.bg").unwrap_or_default()
            }
        } else {
            theme.get_color("border.default").unwrap_or_default()
        }
    }
    fn ring_hover_bg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    fn ring_active_bg(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.sunken").unwrap_or_default()
    }
    fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            state.custom_tone.unwrap_or_default()
        } else {
            theme.get_color("action.primary.bg").unwrap_or_default()
        }
    }
    fn focus_color(&self, _state: &RadioRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    fn disabled_opacity(&self, _state: &RadioRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

pub fn arc_radio<T: RadioRenderer + 'static>(r: T) -> Arc<dyn RadioRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultRadio` — `headless::RadioProps` sugar.
// =====================================================================

use gpui::{
    App, InteractiveElement, ParentElement, Stateful, StatefulInteractiveElement, Styled, div,
};
use yororen_ui_core::headless::radio::RadioProps;
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::ActiveTheme;

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::rgb;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/system-light.json");
        Theme::from_json(json).expect("system-light.json is valid")
    }

    #[test]
    fn custom_tone_overrides_checked_ring_and_dot() {
        let theme = fixture();
        let r = TokenRadioRenderer;
        let custom = rgb(0x123456).into();
        let state = RadioRenderState {
            checked: true,
            disabled: false,
            has_custom_tone: true,
            custom_tone: Some(custom),
        };
        assert_eq!(r.ring_border(&state, &theme), custom);
        assert_eq!(r.dot_fg(&state, &theme), custom);
    }
}
