//! Brutalist action renderers: `Button`, `IconButton`,
//! `ToggleButton`, `SplitButton`.

use gpui::{Hsla, Pixels, px};
use yororen_ui_core::renderer::spec::{BorderSpec, Edges, ShadowSpec};
use yororen_ui_core::renderer::variant::ActionVariantKind;
use yororen_ui_core::renderer::variant::VariantState;
use yororen_ui_core::theme::Theme;

use crate::style::{
    BRUTAL_BORDER, BRUTAL_BORDER_WIDTH, BRUTAL_DISABLED_OPACITY, BRUTAL_RADIUS,
    brutal_border_color, brutal_shadow,
};

// =====================================================================
// Button
// =====================================================================

pub use yororen_ui_core::renderer::button::{ButtonRenderState, ButtonRenderer};

pub struct BrutalButtonRenderer;

impl ButtonRenderer for BrutalButtonRenderer {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled { "disabled_bg" } else { "bg" };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or(BRUTAL_BORDER)
    }

    fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled { "disabled_fg" } else { "fg" };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or(BRUTAL_BORDER)
    }

    fn padding(&self, _: &ButtonRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.button.horizontal_padding")
            .unwrap_or(20.0) as f32;
        let v = theme
            .get_number("tokens.control.button.vertical_padding")
            .unwrap_or(12.0) as f32;
        Edges::symmetric(px(h), px(v))
    }

    fn border_radius(&self, _: &ButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    fn border(&self, _: &ButtonRenderState, theme: &Theme) -> Option<BorderSpec> {
        let w = theme
            .get_number("tokens.control.button.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32;
        Some(BorderSpec::new(w, brutal_border_color(theme)))
    }

    fn shadow(&self, _: &ButtonRenderState, theme: &Theme) -> Option<ShadowSpec> {
        Some(brutal_shadow(theme))
    }

    fn min_height(&self, _: &ButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.button.min_height")
            .unwrap_or(44.0) as f32)
    }

    fn disabled_opacity(&self, state: &ButtonRenderState, _: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        BRUTAL_DISABLED_OPACITY
    }

    fn hover_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "hover_bg"
        };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or(BRUTAL_BORDER)
    }

    fn active_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let field = if state.disabled {
            "disabled_bg"
        } else {
            "active_bg"
        };
        theme
            .get_color(&format!("action.{}.{}", state.variant.as_str(), field))
            .unwrap_or(BRUTAL_BORDER)
    }
}

fn action_variant_key(variant: ActionVariantKind) -> &'static str {
    match variant {
        ActionVariantKind::Neutral => "neutral",
        ActionVariantKind::Primary => "primary",
        ActionVariantKind::Danger => "danger",
    }
}

// =====================================================================
// IconButton
// =====================================================================

pub use yororen_ui_core::renderer::icon_button::{IconButtonRenderState, IconButtonRenderer};

pub struct BrutalIconButtonRenderer;

impl IconButtonRenderer for BrutalIconButtonRenderer {
    fn bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let key = action_variant_key(state.variant);
        let field = if state.disabled { "disabled_bg" } else { "bg" };
        theme
            .get_color(&format!("action.{}.{}", key, field))
            .unwrap_or(BRUTAL_BORDER)
    }

    fn hover_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let key = action_variant_key(state.variant);
        theme
            .get_color(&format!("action.{}.hover_bg", key))
            .unwrap_or(BRUTAL_BORDER)
    }

    fn active_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        let key = action_variant_key(state.variant);
        theme
            .get_color(&format!("action.{}.active_bg", key))
            .unwrap_or(BRUTAL_BORDER)
    }

    fn size(&self, _: &IconButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.icon_button.size")
            .unwrap_or(44.0) as f32)
    }

    fn border_radius(&self, _: &IconButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    fn disabled_opacity(&self, state: &IconButtonRenderState, _: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        BRUTAL_DISABLED_OPACITY
    }
}

// =====================================================================
// ToggleButton
// =====================================================================

pub use yororen_ui_core::renderer::toggle_button::{ToggleButtonRenderState, ToggleButtonRenderer};

pub struct BrutalToggleButtonRenderer;

impl ToggleButtonRenderer for BrutalToggleButtonRenderer {
    fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            if state.selected {
                return theme
                    .get_color("action.primary.bg")
                    .unwrap_or(BRUTAL_BORDER);
            }
            return s.bg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.disabled {
            theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else if state.selected {
            theme
                .get_color("action.primary.bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if let Some(s) = &state.custom_style {
            if state.selected {
                return theme
                    .get_color("action.primary.fg")
                    .unwrap_or(BRUTAL_BORDER);
            }
            return s.fg(&VariantState {
                disabled: state.disabled,
            });
        }
        if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.fg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    fn hover_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or(BRUTAL_BORDER);
        }
        if state.selected {
            theme
                .get_color("action.primary.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.hover_bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    fn active_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            return theme
                .get_color("action.neutral.disabled_bg")
                .unwrap_or(BRUTAL_BORDER);
        }
        if state.selected {
            theme
                .get_color("action.primary.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.active_bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    fn min_height(&self, _: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.toggle_button.min_height")
            .unwrap_or(44.0) as f32)
    }

    fn border_radius(&self, _: &ToggleButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    fn disabled_opacity(&self, state: &ToggleButtonRenderState, _: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        BRUTAL_DISABLED_OPACITY
    }
}

// =====================================================================
// SplitButton
// =====================================================================

pub use yororen_ui_core::renderer::split_button::{SplitButtonRenderState, SplitButtonRenderer};

pub struct BrutalSplitButtonRenderer;

impl SplitButtonRenderer for BrutalSplitButtonRenderer {
    fn primary_bg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn primary_fg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn chevron_bg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn chevron_fg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn chevron_hover_bg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn min_height(&self, _: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.min_height")
            .unwrap_or(44.0) as f32)
    }
    fn border_radius(&self, _: &SplitButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn gap(&self, _: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.separator_w")
            .unwrap_or(3.0) as f32)
    }
}
