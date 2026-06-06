//! `RadioRenderer` — the visual side of `Radio`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct RadioRenderState {
    pub checked: bool,
    pub disabled: bool,
    /// `true` if the caller supplied `.custom_tone(...)`.
    pub has_custom_tone: bool,
    /// Caller-supplied override for the checked-state dot /
    /// ring color. When `None`, the renderer falls back to
    /// `action.primary.bg`.
    pub custom_tone: Option<Hsla>,
}

pub trait RadioRenderer: Any + Send + Sync {
    fn ring_size(&self, state: &RadioRenderState, theme: &Theme) -> Pixels;
    fn dot_size(&self, state: &RadioRenderState, theme: &Theme) -> Pixels;
    fn ring_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_border(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_hover_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn ring_active_bg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn dot_fg(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn focus_color(&self, state: &RadioRenderState, theme: &Theme) -> Hsla;
    fn disabled_opacity(&self, state: &RadioRenderState, theme: &Theme) -> f32;
}

pub struct TokenRadioRenderer;

impl RadioRenderer for TokenRadioRenderer {
    fn ring_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.radio.ring_size").unwrap_or(0.0) as f32)
    }
    fn dot_size(&self, _state: &RadioRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.radio.dot_size").unwrap_or(0.0) as f32)
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

use gpui::{div, App, InteractiveElement, ParentElement, Stateful, StatefulInteractiveElement, Styled};
use yororen_ui_core::headless::radio::RadioProps;
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultRadio: Sized {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div>;
}

impl DefaultRadio for RadioProps {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn RadioRenderer> = cx
            .renderer_arc::<markers::Radio, dyn RadioRenderer>()
            .expect("RadioRenderer registered");
        let state = RadioRenderState {
            checked: self.checked,
            disabled: self.disabled,
            has_custom_tone: self.has_custom_tone,
            custom_tone: self.custom_tone,
        };
        let bg = r.ring_bg(&state, theme);
        let border = r.ring_border(&state, theme);
        let ring_size = r.ring_size(&state, theme);
        let dot_size = r.dot_size(&state, theme);
        let dot_fg = r.dot_fg(&state, theme);
        let pill_radius = gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32);
        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border)
            .size(ring_size)
            .rounded(pill_radius)
            .flex()
            .items_center()
            .justify_center();
        if self.checked {
            el = el.child(div().bg(dot_fg).size(dot_size).rounded(pill_radius));
        }
        let hover_bg = r.ring_hover_bg(&state, theme);
        let active_bg = r.ring_active_bg(&state, theme);
        self.raw_hover(false)
            .apply(el)
            .hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}

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
