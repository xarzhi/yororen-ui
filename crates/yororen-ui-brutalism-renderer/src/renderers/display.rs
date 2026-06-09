//! Brutalist display renderers: `Label`, `Heading`, `Divider`,
//! `FocusRing`, `Badge`, `Tag`, `Skeleton`, `ProgressBar`,
//! `EmptyState`.

use gpui::{FontWeight, Hsla, Pixels, SharedString, px};
use yororen_ui_core::headless::badge::BadgeVariant;
use yororen_ui_core::theme::Theme;
use yororen_ui_default_renderer::renderers::spec::Edges;

use crate::style::{
    BRUTAL_BORDER, BRUTAL_BORDER_WIDTH, BRUTAL_FONT_FAMILY, BRUTAL_RADIUS,
    brutal_border_color,
};

// =====================================================================
// Label
// =====================================================================

pub use yororen_ui_default_renderer::renderers::label::{LabelRenderState, LabelRenderer};

pub struct BrutalLabelRenderer;

impl LabelRenderer for BrutalLabelRenderer {
    fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla {
        if state.muted {
            theme
                .get_color("content.secondary")
                .unwrap_or(BRUTAL_BORDER)
        } else if state.inherit_color {
            theme
                .get_color("content.primary")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("content.primary")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    fn strong_weight(&self, _: &LabelRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.control.label.weight")
                .or_else(|| theme.get_number("tokens.typography.weight_medium"))
                .unwrap_or(700.0) as f32,
        )
    }

    fn family_mono(&self, _: &LabelRenderState, theme: &Theme) -> SharedString {
        theme
            .get_string("tokens.typography.family_mono")
            .unwrap_or(BRUTAL_FONT_FAMILY)
            .to_string()
            .into()
    }
}

// =====================================================================
// Heading
// =====================================================================

pub use yororen_ui_default_renderer::renderers::heading::{HeadingRenderState, HeadingRenderer};

pub struct BrutalHeadingRenderer;

impl HeadingRenderer for BrutalHeadingRenderer {
    fn size(&self, state: &HeadingRenderState, theme: &Theme) -> Pixels {
        let path = match state.level {
            yororen_ui_core::headless::heading::HeadingLevel::H1 => {
                "tokens.control.heading.font_size_lg"
            }
            yororen_ui_core::headless::heading::HeadingLevel::H2 => {
                "tokens.control.heading.font_size_md"
            }
            _ => "tokens.control.heading.font_size_sm",
        };
        px(theme.get_number(path).unwrap_or(24.0) as f32)
    }

    fn weight(&self, state: &HeadingRenderState, theme: &Theme) -> FontWeight {
        let default = match state.level {
            yororen_ui_core::headless::heading::HeadingLevel::H1 => 800.0,
            _ => 800.0,
        };
        FontWeight(
            theme
                .get_number("tokens.control.heading.weight")
                .or_else(|| theme.get_number("tokens.typography.weight_bold"))
                .unwrap_or(default) as f32,
        )
    }

    fn color(&self, _: &HeadingRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.primary")
            .unwrap_or(BRUTAL_BORDER)
    }
}

// =====================================================================
// Divider
// =====================================================================

pub use yororen_ui_default_renderer::renderers::divider::{DividerRenderState, DividerRenderer};

pub struct BrutalDividerRenderer;

impl DividerRenderer for BrutalDividerRenderer {
    fn color(&self, _: &DividerRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("border.divider")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn thickness(&self, _: &DividerRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.divider.thickness")
                .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32,
        )
    }
}

// =====================================================================
// FocusRing
// =====================================================================

pub use yororen_ui_default_renderer::renderers::focus_ring::{
    FocusRingRenderState, FocusRingRenderer,
};

pub struct BrutalFocusRingRenderer;

impl FocusRingRenderer for BrutalFocusRingRenderer {
    fn color(&self, _: &FocusRingRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("border.focus")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn width(&self, _: &FocusRingRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.focus_ring.width")
                .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32,
        )
    }
}

// =====================================================================
// Badge
// =====================================================================

pub use yororen_ui_default_renderer::renderers::badge::{BadgeRenderState, BadgeRenderer};

pub struct BrutalBadgeRenderer;

impl BadgeRenderer for BrutalBadgeRenderer {
    fn bg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        let key = match state.variant {
            BadgeVariant::Neutral => "neutral",
            BadgeVariant::Success => "success",
            BadgeVariant::Warning => "warning",
            BadgeVariant::Danger => "danger",
            BadgeVariant::Info => "info",
        };
        theme
            .get_color(&format!("status.{key}.bg"))
            .unwrap_or(BRUTAL_BORDER)
    }

    fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
        let key = match state.variant {
            BadgeVariant::Neutral => "neutral",
            BadgeVariant::Success => "success",
            BadgeVariant::Warning => "warning",
            BadgeVariant::Danger => "danger",
            BadgeVariant::Info => "info",
        };
        theme
            .get_color(&format!("status.{key}.fg"))
            .unwrap_or(BRUTAL_BORDER)
    }

    fn padding_x(&self, _: &BadgeRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.badge.horizontal_padding")
                .unwrap_or(8.0) as f32,
        )
    }

    fn height(&self, _: &BadgeRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.badge.min_height")
                .unwrap_or(22.0) as f32,
        )
    }

    fn font_size(&self, _: &BadgeRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.badge.font_size")
                .unwrap_or(11.0) as f32,
        )
    }

    fn font_weight(&self, _: &BadgeRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.control.badge.weight")
                .or_else(|| theme.get_number("tokens.typography.weight_bold"))
                .unwrap_or(800.0) as f32,
        )
    }

    fn border_radius(&self, _: &BadgeRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

// =====================================================================
// Tag
// =====================================================================

pub use yororen_ui_default_renderer::renderers::tag::{TagRenderState, TagRenderer};

pub struct BrutalTagRenderer;

impl TagRenderer for BrutalTagRenderer {
    fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme
                .get_color("action.primary.bg")
                .unwrap_or(BRUTAL_BORDER)
        } else if state.has_custom_tone {
            theme
                .get_color("content.on_status")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.bg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
        if state.selected {
            theme
                .get_color("action.primary.fg")
                .unwrap_or(BRUTAL_BORDER)
        } else if state.has_custom_tone {
            theme
                .get_color("content.on_status")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("action.neutral.fg")
                .unwrap_or(BRUTAL_BORDER)
        }
    }

    fn min_height(&self, _: &TagRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.tag.min_height")
                .unwrap_or(28.0) as f32,
        )
    }

    fn padding_x(&self, _: &TagRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.tag.horizontal_padding")
                .unwrap_or(12.0) as f32,
        )
    }

    fn font_size(&self, _: &TagRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.typography.font_size_xs")
                .unwrap_or(12.0) as f32,
        )
    }

    fn font_weight(&self, _: &TagRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_bold")
                .unwrap_or(700.0) as f32,
        )
    }

    fn border_radius(&self, _: &TagRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    fn close_size(&self, _: &TagRenderState, _: &Theme) -> Pixels {
        px(16.0)
    }

    fn close_hover_bg(&self, _: &TagRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
}

// =====================================================================
// Skeleton
// =====================================================================

pub use yororen_ui_default_renderer::renderers::skeleton::{
    SkeletonRenderState, SkeletonRenderer,
};

pub struct BrutalSkeletonRenderer;

impl SkeletonRenderer for BrutalSkeletonRenderer {
    fn bg(&self, _: &SkeletonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.hover")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block {
            px(
                theme
                    .get_number("tokens.control.skeleton.block_min_h")
                    .unwrap_or(48.0) as f32,
            )
        } else {
            px(
                theme
                    .get_number("tokens.control.skeleton.line_h")
                    .unwrap_or(16.0) as f32,
            )
        }
    }

    fn border_radius(&self, _: &SkeletonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

// =====================================================================
// ProgressBar
// =====================================================================

pub use yororen_ui_default_renderer::renderers::progress::{
    ProgressBarRenderState, ProgressBarRenderer,
};

pub struct BrutalProgressBarRenderer;

impl ProgressBarRenderer for BrutalProgressBarRenderer {
    fn track(&self, _: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.hover")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn fill(&self, _: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }

    fn height(&self, _: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.progress.height")
                .unwrap_or(28.0) as f32,
        )
    }

    fn border_color(&self, _: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    fn border_radius(&self, _: &ProgressBarRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

// =====================================================================
// EmptyState
// =====================================================================

pub use yororen_ui_default_renderer::renderers::empty_state::{
    EmptyStateRenderState, EmptyStateRenderer,
};

pub struct BrutalEmptyStateRenderer;

impl EmptyStateRenderer for BrutalEmptyStateRenderer {
    fn icon_color(&self, _: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.tertiary")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn title_color(&self, _: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.primary")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn body_color(&self, _: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.secondary")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn padding(&self, _: &EmptyStateRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.empty_state.padding")
            .unwrap_or(32.0) as f32;
        Edges::all(px(p))
    }
    fn icon_size(&self, _: &EmptyStateRenderState, _: &Theme) -> Pixels {
        px(48.0)
    }
    fn gap(&self, _: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.spacing.loose")
                .unwrap_or(16.0) as f32,
        )
    }
}

// End of empty-state impl.
