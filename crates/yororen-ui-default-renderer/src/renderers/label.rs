//! `LabelRenderer` — the visual side of `Label`.

use std::any::Any;
use std::sync::Arc;

use gpui::{FontWeight, Hsla, SharedString};

use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct LabelRenderState {
    pub muted: bool,
    pub strong: bool,
    pub mono: bool,
    pub inherit_color: bool,
    /// Single-line text gets an ellipsis when it overflows.
    pub ellipsis: bool,
    /// Allow text to wrap at the parent's width boundary.
    pub wrap: bool,
    /// Cap the visible line count to `Some(n)`. `None` means
    /// unlimited (the default).
    pub max_lines: Option<usize>,
}

pub trait LabelRenderer: Any + Send + Sync {
    fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla;
    fn strong_weight(&self, state: &LabelRenderState, theme: &Theme) -> FontWeight;
    fn family_mono(&self, state: &LabelRenderState, theme: &Theme) -> SharedString;
}

pub struct TokenLabelRenderer;

impl LabelRenderer for TokenLabelRenderer {
    fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla {
        if state.inherit_color {
            // Inherit means "use whatever parent div set".
            // For simplicity, return the base content color.
            theme.get_color("content.primary").unwrap_or_default()
        } else if state.muted {
            theme.get_color("content.secondary").unwrap_or_default()
        } else {
            theme.get_color("content.primary").unwrap_or_default()
        }
    }

    fn strong_weight(&self, _state: &LabelRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_semibold")
                .unwrap_or(600.0) as f32,
        )
    }

    fn family_mono(&self, _state: &LabelRenderState, theme: &Theme) -> SharedString {
        theme
            .get_string("tokens.typography.family_mono")
            .unwrap_or("ui-monospace")
            .to_string()
            .into()
    }
}

pub fn arc_label<T: LabelRenderer + 'static>(r: T) -> Arc<dyn LabelRenderer> {
    Arc::new(r)
}

// =====================================================================
// `DefaultLabel` — `headless::LabelProps` sugar.
// =====================================================================

use gpui::{App, ParentElement, Stateful, Styled, div};
use yororen_ui_core::headless::label::LabelProps;
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::ActiveTheme;

pub trait DefaultLabel: Sized {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div>;
}

impl DefaultLabel for LabelProps {
    fn default_render(self, cx: &App) -> Stateful<gpui::Div> {
        let theme = cx.theme();
        let r: &Arc<dyn LabelRenderer> = cx
            .renderer_arc::<markers::Label, dyn LabelRenderer>()
            .expect("LabelRenderer registered");
        let state = LabelRenderState {
            muted: self.muted,
            strong: self.strong,
            mono: self.mono,
            inherit_color: self.inherit_color,
            ellipsis: self.ellipsis,
            wrap: self.wrap,
            max_lines: self.max_lines,
        };
        let color = r.color(&state, theme);
        let weight = r.strong_weight(&state, theme);
        let family = r.family_mono(&state, theme);
        let mut el = div();
        if !self.inherit_color {
            el = el.text_color(color);
        }
        if self.strong {
            el = el.font_weight(weight);
        }
        if self.mono {
            el = el.font_family(family);
        }
        if self.ellipsis {
            el = el.overflow_hidden().text_ellipsis().whitespace_nowrap();
        }
        if self.wrap {
            el = el.whitespace_normal();
        }
        if let Some(n) = self.max_lines {
            // gpui exposes `line_clamp` on `Styled`; it both
            // truncates and disables wrapping. Pair with
            // `overflow_hidden` for safety.
            el = el.line_clamp(n).overflow_hidden();
        }
        el = el.child(self.text.clone());
        self.apply(el)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Theme {
        let json = include_str!("../../themes/system-light.json");
        Theme::from_json(json).expect("system-light.json is valid")
    }

    #[test]
    fn color_picks_primary_when_not_muted() {
        let theme = fixture();
        let r = TokenLabelRenderer;
        let state = LabelRenderState::default();
        assert_eq!(
            r.color(&state, &theme),
            theme.get_color("content.primary").unwrap(),
        );
    }

    #[test]
    fn color_picks_secondary_when_muted() {
        let theme = fixture();
        let r = TokenLabelRenderer;
        let state = LabelRenderState {
            muted: true,
            ..Default::default()
        };
        assert_eq!(
            r.color(&state, &theme),
            theme.get_color("content.secondary").unwrap(),
        );
    }

    #[test]
    fn color_returns_primary_when_inherit() {
        let theme = fixture();
        let r = TokenLabelRenderer;
        let state = LabelRenderState {
            inherit_color: true,
            ..Default::default()
        };
        assert_eq!(
            r.color(&state, &theme),
            theme.get_color("content.primary").unwrap(),
        );
    }

    #[test]
    fn strong_weight_reads_weight_semibold() {
        let theme = fixture();
        let r = TokenLabelRenderer;
        let state = LabelRenderState::default();
        let expected = theme
            .get_number("tokens.typography.weight_semibold")
            .unwrap_or(600.0) as f32;
        assert_eq!(r.strong_weight(&state, &theme), FontWeight(expected));
    }

    #[test]
    fn family_mono_reads_family_mono() {
        let theme = fixture();
        let r = TokenLabelRenderer;
        let state = LabelRenderState::default();
        assert_eq!(
            r.family_mono(&state, &theme).to_string(),
            theme
                .get_string("tokens.typography.family_mono")
                .unwrap_or("ui-monospace"),
        );
    }
}
