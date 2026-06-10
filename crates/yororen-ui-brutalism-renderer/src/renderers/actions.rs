//! Brutalist action renderers: `Button`, `IconButton`,
//! `ToggleButton`, `SplitButton`.

use gpui::{
    App, Div, ElementId, FocusHandle, Hsla, InteractiveElement, ParentElement, Pixels, Stateful,
    StatefulInteractiveElement, Styled, div, px,
};
use yororen_ui_core::headless::button::ButtonProps;
use yororen_ui_core::headless::icon::IconProps;
use yororen_ui_core::headless::icon_button::IconButtonProps;
use yororen_ui_core::headless::toggle_button::ToggleButtonProps;
use yororen_ui_core::renderer::spec::{BorderSpec, Edges, ShadowSpec};
use yororen_ui_core::renderer::variant::ActionVariantKind;
use yororen_ui_core::renderer::variant::VariantState;
use yororen_ui_core::theme::ActiveTheme;
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

// Inherent helpers — *not* part of the `ButtonRenderer` trait
// surface. They exist so `compose` (below) can stay readable
// and so other code in this crate can share the palette
// lookups.
impl BrutalButtonRenderer {
    pub fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn padding(&self, _: &ButtonRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.button.horizontal_padding")
            .unwrap_or(20.0) as f32;
        let v = theme
            .get_number("tokens.control.button.vertical_padding")
            .unwrap_or(12.0) as f32;
        Edges::symmetric(px(h), px(v))
    }

    pub fn border_radius(&self, _: &ButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn border(&self, _: &ButtonRenderState, theme: &Theme) -> Option<BorderSpec> {
        let w = theme
            .get_number("tokens.control.button.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32;
        Some(BorderSpec::new(w, brutal_border_color(theme)))
    }

    pub fn shadow(&self, _: &ButtonRenderState, theme: &Theme) -> Option<ShadowSpec> {
        Some(brutal_shadow(theme))
    }

    pub fn min_height(&self, _: &ButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.button.min_height")
            .unwrap_or(44.0) as f32)
    }

    pub fn disabled_opacity(&self, state: &ButtonRenderState, _: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        BRUTAL_DISABLED_OPACITY
    }

    pub fn hover_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn active_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla {
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

impl ButtonRenderer for BrutalButtonRenderer {
    fn compose(
        &self,
        props: &ButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = ButtonRenderState {
            variant: props.variant,
            disabled: props.disabled,
            ..Default::default()
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let padding = self.padding(&state, theme);
        let radius = self.border_radius(&state, theme);
        let min_h = self.min_height(&state, theme);
        let opacity = if props.disabled {
            self.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = self.hover_bg(&state, theme);
        let active_bg = self.active_bg(&state, theme);
        let border = self.border(&state, theme);
        let shadow = self.shadow(&state, theme);
        let icon_gap = theme
            .get_number("tokens.control.button.icon_gap")
            .unwrap_or(8.0) as f32;

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .text_color(fg)
            .min_h(min_h)
            .rounded(radius)
            .px(padding.left)
            .py(padding.top)
            .gap(px(icon_gap))
            .opacity(opacity)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if let Some(b) = border {
            el = el.border(b.width).border_color(b.color);
        }
        if let Some(s) = shadow {
            el = el.shadow(vec![gpui::BoxShadow {
                color: s.color,
                offset: gpui::point(px(0.0), s.offset_y),
                blur_radius: s.blur,
                spread_radius: px(0.0),
            }]);
        }

        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(props.icon_size),
                color: Some(fg),
            }
            .render();
            el = el.child(icon_el);
        }
        if let Some(caption) = props.caption.clone() {
            el = el.child(caption);
        }

        el.hover(|s| s.bg(hover_bg)).active(|s| s.bg(active_bg))
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

// Inherent helpers — *not* part of the `IconButtonRenderer`
// trait surface.
impl BrutalIconButtonRenderer {
    pub fn bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn fg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
        let key = action_variant_key(state.variant);
        let field = if state.disabled { "disabled_fg" } else { "fg" };
        theme
            .get_color(&format!("action.{}.{}", key, field))
            .unwrap_or(BRUTAL_BORDER)
    }

    pub fn hover_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn active_bg(&self, state: &IconButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn size(&self, _: &IconButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.icon_button.size")
            .unwrap_or(44.0) as f32)
    }

    pub fn border_radius(&self, _: &IconButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn disabled_opacity(&self, state: &IconButtonRenderState, _: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        BRUTAL_DISABLED_OPACITY
    }
}

impl IconButtonRenderer for BrutalIconButtonRenderer {
    fn compose(
        &self,
        props: &IconButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div> {
        let theme = cx.theme();
        let state = IconButtonRenderState {
            variant: props.variant,
            disabled: props.disabled,
            has_custom_bg: false,
            has_custom_hover_bg: false,
            custom_style: None,
        };
        let bg = self.bg(&state, theme);
        let fg = self.fg(&state, theme);
        let radius = self.border_radius(&state, theme);
        let opacity = if props.disabled {
            self.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = self.hover_bg(&state, theme);
        let active_bg = self.active_bg(&state, theme);
        let side = self.size(&state, theme);
        let border_color = brutal_border_color(theme);
        let border_w = theme
            .get_number("tokens.control.button.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32;

        let mut el: Stateful<Div> = div()
            .id(props.id.clone())
            .bg(bg)
            .rounded(radius)
            .size(side)
            .opacity(opacity)
            .border(px(border_w))
            .border_color(border_color)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(props.icon_size),
                color: Some(fg),
            }
            .render();
            el = el.child(icon_el);
        }

        el.hover(|s| s.bg(hover_bg)).active(|s| s.bg(active_bg))
    }
}

// =====================================================================
// ToggleButton
// =====================================================================

pub use yororen_ui_core::renderer::toggle_button::{ToggleButtonRenderState, ToggleButtonRenderer};

pub struct BrutalToggleButtonRenderer;

// Inherent helpers — *not* part of the `ToggleButtonRenderer`
// trait surface.
impl BrutalToggleButtonRenderer {
    pub fn bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn fg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn hover_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn active_bg(&self, state: &ToggleButtonRenderState, theme: &Theme) -> Hsla {
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

    pub fn min_height(&self, _: &ToggleButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.toggle_button.min_height")
            .unwrap_or(44.0) as f32)
    }

    pub fn border_radius(&self, _: &ToggleButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }

    pub fn disabled_opacity(&self, state: &ToggleButtonRenderState, _: &Theme) -> f32 {
        if let Some(s) = &state.custom_style {
            return s.disabled_opacity();
        }
        BRUTAL_DISABLED_OPACITY
    }
}

impl ToggleButtonRenderer for BrutalToggleButtonRenderer {
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
        let border_color = brutal_border_color(theme);
        let border_w = theme
            .get_number("tokens.control.toggle_button.border_width")
            .unwrap_or(BRUTAL_BORDER_WIDTH as f64) as f32;

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
            .border(px(border_w))
            .border_color(border_color)
            .flex()
            .items_center()
            .justify_center()
            .track_focus(focus_handle);

        if let Some(source) = props.icon.clone() {
            let icon_id: ElementId = format!("{:?}-icon", props.id).into();
            let icon_el = IconProps {
                id: icon_id,
                source,
                size: Some(props.icon_size),
                color: Some(fg),
            }
            .render();
            el = el.child(icon_el);
        }
        if let Some(caption) = props.caption.clone() {
            el = el.child(caption);
        }

        el.hover(|s| s.bg(hover_bg)).active(|s| s.bg(active_bg))
    }
}

// =====================================================================
// SplitButton
// =====================================================================

pub use yororen_ui_core::renderer::split_button::{SplitButtonRenderState, SplitButtonRenderer};

pub struct BrutalSplitButtonRenderer;

// Inherent helpers — *not* part of the trait surface.
impl BrutalSplitButtonRenderer {
    pub fn primary_bg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn primary_fg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.primary.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn chevron_bg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn chevron_fg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn chevron_hover_bg(&self, _: &SplitButtonRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    pub fn min_height(&self, _: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.min_height")
            .unwrap_or(44.0) as f32)
    }
    pub fn border_radius(&self, _: &SplitButtonRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    pub fn gap(&self, _: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.separator_w")
            .unwrap_or(3.0) as f32)
    }
}

impl SplitButtonRenderer for BrutalSplitButtonRenderer {
    fn compose(
        &self,
        props: &yororen_ui_core::headless::split_button::SplitButtonProps,
        cx: &App,
    ) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let state = SplitButtonRenderState {
            open: false,
            disabled: props.disabled,
        };
        let pbg = self.primary_bg(&state, theme);
        let pfg = self.primary_fg(&state, theme);
        let cbg = self.chevron_bg(&state, theme);
        let cfg = self.chevron_fg(&state, theme);
        let h = self.min_height(&state, theme);
        let r = self.border_radius(&state, theme);
        let _ = props;
        gpui::div()
            .flex()
            .items_center()
            .bg(pbg)
            .text_color(pfg)
            .min_h(h)
            .rounded(r)
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .px(px(12.0))
                    .child("Run"),
            )
            .child(
                gpui::div()
                    .flex()
                    .items_center()
                    .bg(cbg)
                    .text_color(cfg)
                    .px(px(8.0))
                    .child("▼"),
            )
    }
}
