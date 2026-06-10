//! Brutalist display renderers: `Label`, `Heading`, `Divider`,
//! `FocusRing`, `Badge`, `Tag`, `Skeleton`, `ProgressBar`,
//! `EmptyState`.

use gpui::{App, BoxShadow, Div, FontWeight, Hsla, InteractiveElement, ParentElement, Pixels, SharedString, Stateful, Styled, hsla, point, px};
use yororen_ui_core::headless::badge::BadgeVariant;
use yororen_ui_core::headless::label::LabelProps;
use yororen_ui_core::renderer::spec::Edges;
use yororen_ui_core::theme::ActiveTheme;
use yororen_ui_core::theme::Theme;

use crate::style::{
    BRUTAL_BORDER, BRUTAL_BORDER_WIDTH, BRUTAL_FONT_FAMILY, BRUTAL_RADIUS, brutal_border_color,
};

// =====================================================================
// Label
// =====================================================================

pub use yororen_ui_core::renderer::label::{LabelRenderState, LabelRenderer};

pub struct BrutalLabelRenderer;

// Inherent helpers — *not* part of the `LabelRenderer` trait
// surface.
impl BrutalLabelRenderer {
    pub fn color(&self, state: &LabelRenderState, theme: &Theme) -> Hsla {
        if state.muted {
            theme
                .get_color("content.secondary")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
        }
    }

    pub fn strong_weight(&self, _: &LabelRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.control.label.weight")
                .or_else(|| theme.get_number("tokens.typography.weight_medium"))
                .unwrap_or(700.0) as f32,
        )
    }

    pub fn family_mono(&self, _: &LabelRenderState, theme: &Theme) -> SharedString {
        theme
            .get_string("tokens.typography.family_mono")
            .unwrap_or(BRUTAL_FONT_FAMILY)
            .to_string()
            .into()
    }
}

impl LabelRenderer for BrutalLabelRenderer {
    fn compose(&self, props: &LabelProps, cx: &App) -> Div {
        let theme = cx.theme();
        let state = LabelRenderState {
            muted: props.muted,
            strong: props.strong,
            mono: props.mono,
            inherit_color: props.inherit_color,
            ellipsis: props.ellipsis,
            wrap: props.wrap,
            max_lines: props.max_lines,
        };
        let color = self.color(&state, theme);
        let weight = self.strong_weight(&state, theme);
        let family = self.family_mono(&state, theme);
        let mut el: Div = gpui::div();
        if !props.inherit_color {
            el = el.text_color(color);
        }
        if props.strong {
            el = el.font_weight(weight);
        }
        if props.mono {
            el = el.font_family(family);
        }
        if props.ellipsis {
            el = el.overflow_hidden().text_ellipsis().whitespace_nowrap();
        }
        if props.wrap {
            el = el.whitespace_normal();
        }
        if let Some(n) = props.max_lines {
            el = el.line_clamp(n).overflow_hidden();
        }
        el.child(props.text.clone())
    }
}

// =====================================================================
// Heading
// =====================================================================

pub use yororen_ui_core::renderer::heading::{HeadingRenderState, HeadingRenderer};

pub struct BrutalHeadingRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalHeadingRenderer {
    pub fn size(&self, state: &HeadingRenderState, theme: &Theme) -> Pixels {
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

    pub fn weight(&self, state: &HeadingRenderState, theme: &Theme) -> FontWeight {
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

    pub fn color(&self, _: &HeadingRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
}

impl HeadingRenderer for BrutalHeadingRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::heading::HeadingProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = HeadingRenderState {
            level: props.level,
        };
        let size = self.size(&state, theme);
        let weight = self.weight(&state, theme);
        let color = self.color(&state, theme);
        gpui::div()
            .text_color(color)
            .text_size(size)
            .font_weight(weight)
            .child(props.text.clone())
    }
}

// =====================================================================
// Divider
// =====================================================================

pub use yororen_ui_core::renderer::divider::{DividerRenderState, DividerRenderer};

pub struct BrutalDividerRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalDividerRenderer {
    pub fn color(&self, _: &DividerRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.divider").unwrap_or(BRUTAL_BORDER)
    }
    pub fn thickness(&self, _: &DividerRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.divider.thickness")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32)
    }
}

impl DividerRenderer for BrutalDividerRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::divider::DividerProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = DividerRenderState {
            horizontal: props.horizontal,
        };
        let color = self.color(&state, theme);
        let thickness = self.thickness(&state, theme);
        let mut el = gpui::div().bg(color);
        if props.horizontal {
            el = el.w_full().h(thickness);
        } else {
            el = el.h_full().w(thickness);
        }
        el
    }
}

// =====================================================================
// FocusRing
// =====================================================================

pub use yororen_ui_core::renderer::focus_ring::{FocusRingRenderState, FocusRingRenderer};

pub struct BrutalFocusRingRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalFocusRingRenderer {
    pub fn color(&self, _: &FocusRingRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or(BRUTAL_BORDER)
    }

    pub fn width(&self, _: &FocusRingRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.focus_ring.width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32)
    }
}

impl FocusRingRenderer for BrutalFocusRingRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::focus_ring::FocusRingProps,
        cx: &App,
    ) -> Stateful<Div> {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = FocusRingRenderState {
            has_custom_color: props.has_custom_color,
        };
        let color = self.color(&state, theme);
        let width = self.width(&state, theme);
        gpui::div()
            .id(props.id.clone())
            .track_focus(&props.focus_handle)
            .shadow(vec![BoxShadow {
                color,
                offset: point(px(0.), px(0.)),
                blur_radius: px(0.),
                spread_radius: width,
            }])
    }
}

// =====================================================================
// Badge
// =====================================================================

pub use yororen_ui_core::renderer::badge::{BadgeRenderState, BadgeRenderer};

pub struct BrutalBadgeRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalBadgeRenderer {
    pub fn bg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
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

    pub fn fg(&self, state: &BadgeRenderState, theme: &Theme) -> Hsla {
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

    pub fn padding_x(&self, _: &BadgeRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.badge.horizontal_padding")
            .unwrap_or(8.0) as f32)
    }

    pub fn height(&self, _: &BadgeRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.badge.min_height")
            .unwrap_or(22.0) as f32)
    }

    pub fn font_size(&self, _: &BadgeRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.badge.font_size")
            .unwrap_or(11.0) as f32)
    }

    pub fn font_weight(&self, _: &BadgeRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.control.badge.weight")
                .or_else(|| theme.get_number("tokens.typography.weight_bold"))
                .unwrap_or(800.0) as f32,
        )
    }

    pub fn border_radius(&self, _: &BadgeRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl BadgeRenderer for BrutalBadgeRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::badge::BadgeProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = BadgeRenderState {
            variant: props.variant,
            has_custom_tone: false,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let px_v = self.padding_x(&state, theme);
        let h = self.height(&state, theme);
        let fs = self.font_size(&state, theme);
        let fw = self.font_weight(&state, theme);
        let r = self.border_radius(&state, theme);
        gpui::div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .px(px_v)
            .h(h)
            .text_size(fs)
            .font_weight(fw)
            .rounded(r)
            .child(props.text.clone())
    }
}

// =====================================================================
// Tag
// =====================================================================

pub use yororen_ui_core::renderer::tag::{TagRenderState, TagRenderer};

pub struct BrutalTagRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalTagRenderer {
    pub fn bg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
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

    pub fn fg(&self, state: &TagRenderState, theme: &Theme) -> Hsla {
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

    pub fn min_height(&self, _: &TagRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tag.min_height")
            .unwrap_or(28.0) as f32)
    }

    pub fn padding_x(&self, _: &TagRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.tag.horizontal_padding")
            .unwrap_or(12.0) as f32)
    }

    pub fn font_size(&self, _: &TagRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.typography.font_size_xs")
            .unwrap_or(12.0) as f32)
    }

    pub fn font_weight(&self, _: &TagRenderState, theme: &Theme) -> FontWeight {
        FontWeight(
            theme
                .get_number("tokens.typography.weight_bold")
                .unwrap_or(700.0) as f32,
        )
    }

    pub fn border_radius(&self, _: &TagRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn close_size(&self, _: &TagRenderState, _: &Theme) -> Pixels {
        px(16.0)
    }

    pub fn close_hover_bg(&self, _: &TagRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
}

impl TagRenderer for BrutalTagRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::tag::TagProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = TagRenderState {
            selected: props.selected,
            has_custom_tone: false,
            closable: props.closable,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let h = self.min_height(&state, theme);
        let p = self.padding_x(&state, theme);
        let fs = self.font_size(&state, theme);
        let fw = self.font_weight(&state, theme);
        let r = self.border_radius(&state, theme);
        let mut el = gpui::div()
            .flex()
            .items_center()
            .bg(bg)
            .text_color(fg)
            .min_h(h)
            .px(p)
            .text_size(fs)
            .font_weight(fw)
            .rounded(r)
            .gap(p / 2.)
            .child(props.label.clone());
        if props.closable {
            let close_size = self.close_size(&state, theme);
            el = el.child(
                gpui::div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(close_size)
                    .rounded(close_size / 2.)
                    .child("×"),
            );
        }
        el
    }
}

// =====================================================================
// Skeleton
// =====================================================================

pub use yororen_ui_core::renderer::skeleton::{SkeletonRenderState, SkeletonRenderer};

pub struct BrutalSkeletonRenderer;

impl BrutalSkeletonRenderer {
    pub fn bg(&self, _: &SkeletonRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }
    pub fn min_height(&self, state: &SkeletonRenderState, theme: &Theme) -> Pixels {
        if state.block {
            px(theme
                .get_number("tokens.control.skeleton.block_min_h")
                .unwrap_or(48.0) as f32)
        } else {
            px(theme
                .get_number("tokens.control.skeleton.line_h")
                .unwrap_or(16.0) as f32)
        }
    }
    pub fn border_radius(&self, _: &SkeletonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl SkeletonRenderer for BrutalSkeletonRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::skeleton::SkeletonProps,
        cx: &App,
    ) -> Div {
        let theme = cx.theme();
        let state = SkeletonRenderState {
            block: props.block,
            block_sharp: props.block_sharp,
            rounded: props.rounded,
        };
        let bg = self.bg(&state, theme);
        let min_h = self.min_height(&state, theme);
        let radius = self.border_radius(&state, theme);
        gpui::div().bg(bg).min_h(min_h).rounded(radius)
    }
}

// =====================================================================
// ProgressBar
// =====================================================================

pub use yororen_ui_core::renderer::progress::{ProgressBarRenderState, ProgressBarRenderer};

pub struct BrutalProgressBarRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalProgressBarRenderer {
    pub fn track(&self, _: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or(BRUTAL_BORDER)
    }

    pub fn fill(&self, _: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn height(&self, _: &ProgressBarRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.progress.height")
            .unwrap_or(28.0) as f32)
    }

    pub fn border_color(&self, _: &ProgressBarRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }

    pub fn border_radius(&self, _: &ProgressBarRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

impl ProgressBarRenderer for BrutalProgressBarRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::progress::ProgressBarProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = ProgressBarRenderState {
            indeterminate: props.indeterminate,
            has_custom_height: props.has_custom_height,
        };
        let track = self.track(&state, theme);
        let fill = self.fill(&state, theme);
        let h = self.height(&state, theme);
        let bc = self.border_color(&state, theme);
        let r = self.border_radius(&state, theme);
        let ratio = if props.indeterminate || props.max <= 0.0 {
            0.0
        } else {
            (props.value / props.max).clamp(0.0, 1.0)
        };
        gpui::div()
            .flex()
            .flex_col()
            .w_full()
            .h(h)
            .bg(track)
            .rounded(r)
            .border_2()
            .border_color(bc)
            .child(
                gpui::div()
                    .h_full()
                    .w(gpui::relative(ratio))
                    .bg(fill)
                    .rounded(r),
            )
    }
}

// =====================================================================
// EmptyState
// =====================================================================

pub use yororen_ui_core::renderer::empty_state::{EmptyStateRenderState, EmptyStateRenderer};

pub struct BrutalEmptyStateRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalEmptyStateRenderer {
    pub fn icon_color(&self, _: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.tertiary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn title_color(&self, _: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme.get_color("content.primary").unwrap_or(BRUTAL_BORDER)
    }
    pub fn body_color(&self, _: &EmptyStateRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.secondary")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn padding(&self, _: &EmptyStateRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.empty_state.padding")
            .unwrap_or(32.0) as f32;
        Edges::all(px(p))
    }
    pub fn icon_size(&self, _: &EmptyStateRenderState, _: &Theme) -> Pixels {
        px(48.0)
    }
    pub fn gap(&self, _: &EmptyStateRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.spacing.loose").unwrap_or(16.0) as f32)
    }
}

impl EmptyStateRenderer for BrutalEmptyStateRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::empty_state::EmptyStateProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = EmptyStateRenderState {};
        let ic = self.icon_color(&state, theme);
        let tc = self.title_color(&state, theme);
        let bc = self.body_color(&state, theme);
        let pad = self.padding(&state, theme);
        let is = self.icon_size(&state, theme);
        let g = self.gap(&state, theme);
        let mut el = gpui::div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .p(pad.top)
            .gap(g);
        if let Some(icon) = &props.icon {
            el = el.child(gpui::div().text_color(ic).size(is).child(icon.clone()));
        }
        if let Some(title) = &props.title {
            el = el.child(gpui::div().text_color(tc).child(title.clone()));
        }
        if let Some(desc) = &props.description {
            el = el.child(gpui::div().text_color(bc).child(desc.clone()));
        }
        el
    }
}

// End of empty-state impl.
