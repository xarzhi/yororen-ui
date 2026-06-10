//! `LabelRenderer` — the visual side of `Label`.

use std::any::Any;
use std::sync::Arc;

use gpui::{FontWeight, Hsla, SharedString};

use yororen_ui_core::theme::Theme;

pub use yororen_ui_core::renderer::label::{LabelRenderState, LabelRenderer};

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
