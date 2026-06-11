//! `ToggleButtonRenderer` — visual side of `ToggleButton`.

use std::sync::Arc;

use gpui::{App, Div, ElementId, FocusHandle, Hsla, InteractiveElement, ParentElement, Pixels,
           Stateful, StatefulInteractiveElement, Styled, div, px};

use yororen_ui_core::headless::icon::IconProps;
use yororen_ui_core::headless::toggle_button::ToggleButtonProps;
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;

use yororen_ui_core::renderer::variant::VariantState;

pub use yororen_ui_core::renderer::toggle_button::{ToggleButtonRenderState, ToggleButtonRenderer};

pub struct TokenToggleButtonRenderer;

// Inherent helpers — *not* part of the `ToggleButtonRenderer`
// trait surface. They exist so `compose` can stay readable and
// so unit tests can assert on individual palette paths.
impl TokenToggleButtonRenderer {
    pub fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            // ToggleButton has a binary visual state (selected vs. not);
            // the registered custom variant controls the unselected
            // look. When selected we keep mapping to theme.primary so
            // existing toggle semantics are preserved.
            if state.selected {
                return theme.get_color("action.primary.bg").unwrap_or_default();
            }
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.disabled {
            theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or_default()
        } else if state.selected {
            theme.get_color("action.primary.bg").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.bg").unwrap_or_default()
        }
    }
    pub fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            if state.selected {
                return theme.get_color("action.primary.fg").unwrap_or_default();
            }
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.selected {
            theme.get_color("action.primary.fg").unwrap_or_default()
        } else {
            theme.get_color("action.neutral.fg").unwrap_or_default()
        }
    }
    pub fn hover_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or_default();
        }
        if state.selected {
            return theme
                .get_color("action.primary.hover_bg")
                .unwrap_or_default();
        }
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    pub fn active_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or_default();
        }
        if state.selected {
            return theme
                .get_color("action.primary.active_bg")
                .unwrap_or_default();
        }
        theme
            .get_color("action.neutral.active_bg")
            .unwrap_or_default()
    }
    pub fn min_height(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.toggle_button.min_height")
                .unwrap_or(0.0) as f32,
        )
    }
    pub fn border_radius(&self, _state: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    pub fn disabled_opacity(&self, state: &ToggleButtonRenderState, _theme: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        1.0
    }
}

impl ToggleButtonRenderer for TokenToggleButtonRenderer {
    fn compose(
        &self,
        props: &ToggleButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = ToggleButtonRenderState {
            variant: props.variant,
            selected: props.selected,
            disabled: props.disabled,
            custom_style: None,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let min_h = self.min_height(&state, theme);
        let radius = self.border_radius(&state, theme);
        let opacity = if props.disabled {
            self.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = self.hover_bg(&state, theme);
        let active_bg = self.active_bg(&state, theme);
        let icon_gap = theme
            .get_number("tokens.control.toggle_button.icon_gap")
            .unwrap_or(8.0) as f32;

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .text_color(fg)
            .min_h(min_h)
            .rounded(radius)
            .px(px(12.))
            .py(px(6.))
            .gap(px(icon_gap))
            .opacity(opacity)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_size = props.icon_size;
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(icon_size),
                color: Some(fg),
            }
            .render(cx);
            el = el.child(icon_el);
        }
        if let Some(caption) = props.caption.clone() {
            el = el.child(caption);
        }

        el.hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}

pub fn arc_toggle_button<T: ToggleButtonRenderer + 'static>(r: T) -> Arc<dyn ToggleButtonRenderer> {
    Arc::new(r)
}
