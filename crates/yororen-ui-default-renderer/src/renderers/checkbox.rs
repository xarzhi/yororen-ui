//! `CheckboxRenderer` — the visual side of `Checkbox`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::checkbox::{CheckboxRenderState, CheckboxRenderer};

pub struct TokenCheckboxRenderer;

impl CheckboxRenderer for TokenCheckboxRenderer {
    fn box_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.checkbox.box_size")
                .unwrap_or(0.0) as f32,
        )
    }
    fn check_size(&self, _state: &CheckboxRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.checkbox.check_size")
                .unwrap_or(0.0) as f32,
        )
    }
    fn box_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme.get_color("surface.sunken").unwrap_or_default()
        } else if state.checked {
            if state.has_custom_tone {
                state.custom_tone.unwrap_or_default()
            } else {
                theme.get_color("action.primary.bg").unwrap_or_default()
            }
        } else {
            theme.get_color("surface.base").unwrap_or_default()
        }
    }
    fn box_border(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
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
    fn box_hover_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or_default()
        } else {
            theme.get_color("surface.hover").unwrap_or_default()
        }
    }
    fn box_active_bg(&self, state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        if state.checked {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or_default()
        } else {
            theme.get_color("surface.sunken").unwrap_or_default()
        }
    }
    fn check_fg(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("action.primary.fg").unwrap_or_default()
    }
    fn focus_color(&self, _state: &CheckboxRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    fn disabled_opacity(&self, _state: &CheckboxRenderState, _theme: &Theme) -> f32 {
        0.5
    }
}

pub fn arc_checkbox<T: CheckboxRenderer + 'static>(r: T) -> Arc<dyn CheckboxRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultCheckbox` — `headless::CheckboxProps` sugar.
// =====================================================================

use gpui::{
    App, InteractiveElement, ParentElement, Stateful, StatefulInteractiveElement, Styled, div, px,
};
use yororen_ui_core::headless::checkbox::CheckboxProps;
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
    fn custom_tone_overrides_checked_state_color() {
        // Regression for the P2 audit finding:
        // `has_custom_tone` is a *real* field, not a flag the
        // renderer silently drops.
        let theme = fixture();
        let r = TokenCheckboxRenderer;
        let custom = rgb(0xabcdef).into();
        let state = CheckboxRenderState {
            checked: true,
            disabled: false,
            has_custom_tone: true,
            custom_tone: Some(custom),
        };
        assert_eq!(r.box_bg(&state, &theme), custom);
        assert_eq!(r.box_border(&state, &theme), custom);
        // Unchecked state: tone does not apply.
        let state_unchecked = CheckboxRenderState {
            checked: false,
            ..state
        };
        assert_ne!(r.box_bg(&state_unchecked, &theme), custom);
    }
}
