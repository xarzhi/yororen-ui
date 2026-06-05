//! `DividerRenderer` — the visual side of `Divider`.

use std::any::Any;
use std::sync::Arc;

use gpui::{Hsla, Pixels};

use yororen_ui_core::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct DividerRenderState {
    pub vertical: bool,
}

pub trait DividerRenderer: Any + Send + Sync {
    fn color(&self, state: &DividerRenderState, theme: &Theme) -> Hsla;
    fn thickness(&self, state: &DividerRenderState, theme: &Theme) -> Pixels;
}

pub struct TokenDividerRenderer;

impl DividerRenderer for TokenDividerRenderer {
    fn color(&self, _state: &DividerRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.divider").unwrap_or_default()
    }

    fn thickness(&self, _state: &DividerRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.control.divider.thickness").unwrap_or(0.0) as f32)
    }
}

pub fn arc_divider<T: DividerRenderer + 'static>(r: T) -> Arc<dyn DividerRenderer> {
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
    fn color_reads_border_divider_path() {
        let theme = fixture();
        let r = TokenDividerRenderer;
        let state = DividerRenderState::default();
        // The renderer's `color` should equal
        // `theme.get_color("border.divider")` — same path.
        assert_eq!(
            r.color(&state, &theme),
            theme.get_color("border.divider").unwrap_or_default(),
        );
    }

    #[test]
    fn thickness_reads_control_divider_thickness_path() {
        let theme = fixture();
        let r = TokenDividerRenderer;
        let state = DividerRenderState::default();
        let expected = theme
            .get_number("tokens.control.divider.thickness")
            .unwrap_or(0.0) as f32;
        assert_eq!(r.thickness(&state, &theme), gpui::px(expected));
    }

    #[test]
    fn missing_paths_yield_zero_color() {
        // Theme with only one path — everything else returns None.
        let theme = Theme::from_value(serde_json::json!({}));
        let r = TokenDividerRenderer;
        let state = DividerRenderState::default();
        // Both should fall back to defaults; the call must
        // not panic.
        let _ = r.color(&state, &theme);
        let _ = r.thickness(&state, &theme);
    }
}
