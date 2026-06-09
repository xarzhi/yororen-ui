//! `BadgeRenderer` — the visual side of `Badge`.

use std::any::Any;
use std::sync::Arc;

use gpui::{FontWeight, Hsla, Pixels};

pub use yororen_ui_core::renderer::badge::{BadgeRenderState, BadgeRenderer};
use yororen_ui_core::theme::Theme;

pub struct TokenBadgeRenderer;

impl BadgeRenderer for TokenBadgeRenderer {
    fn bg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        let key = match state.variant {
            yororen_ui_core::headless::badge::BadgeVariant::Neutral => "neutral",
            yororen_ui_core::headless::badge::BadgeVariant::Success => "success",
            yororen_ui_core::headless::badge::BadgeVariant::Warning => "warning",
            yororen_ui_core::headless::badge::BadgeVariant::Danger => "danger",
            yororen_ui_core::headless::badge::BadgeVariant::Info => "info",
        };
        theme
            .get_color(&format!("status.{key}.bg"))
            .unwrap_or_default()
    }

    fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        if state.has_custom_tone {
            theme.get_color("content.on_status").unwrap_or_default()
        } else {
            let key = match state.variant {
                yororen_ui_core::headless::badge::BadgeVariant::Neutral => "neutral",
                yororen_ui_core::headless::badge::BadgeVariant::Success => "success",
                yororen_ui_core::headless::badge::BadgeVariant::Warning => "warning",
                yororen_ui_core::headless::badge::BadgeVariant::Danger => "danger",
                yororen_ui_core::headless::badge::BadgeVariant::Info => "info",
            };
            theme
                .get_color(&format!("status.{key}.fg"))
                .unwrap_or_default()
        }
    }

    fn padding_x(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.spacing.inset_sm").unwrap_or(0.0) as f32)
    }

    fn height(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.control.badge.min_height")
                .unwrap_or(0.0) as f32,
        )
    }

    fn font_size(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        gpui::px(
            theme
                .get_number("tokens.typography.font_size_xs")
                .unwrap_or(0.0) as f32,
        )
    }

    fn font_weight(&self, _state: &BadgeRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_medium")
                .unwrap_or(500.0) as f32,
        )
    }

    fn border_radius(&self, _state: &BadgeRenderState, theme: &Theme) -> Pixels {
        gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32)
    }
}

pub fn arc_badge<T: BadgeRenderer + 'static>(r: T) -> Arc<dyn BadgeRenderer> {
    Arc::new(r)
}
