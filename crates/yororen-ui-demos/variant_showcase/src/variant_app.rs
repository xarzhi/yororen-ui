//! yororen-ui Variant Showcase Demo
//!
//! Three side-by-side buttons, one per `ActionVariantKind`.
//! The renderer reads `action.<variant>.<field>` paths from
//! the theme JSON, so the only thing the variant changes
//! here is the `ButtonRenderState.variant` field.

use gpui::div as gpui_div;
use gpui::{
    Context, InteractiveElement, IntoElement, ParentElement, Render, StatefulInteractiveElement,
    Styled, Window, div, px,
};
use std::sync::Arc;
use yororen_ui::ActionVariantKind;
use yororen_ui::ActiveTheme;
use yororen_ui::RendererContext;
use yororen_ui::Theme;
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;
use yororen_ui::markers::Button as ButtonMarker;
use yororen_ui::renderer::{ButtonRenderState, ButtonRenderer, DefaultButton, DefaultLabel};

pub struct VariantApp;

impl VariantApp {
    pub fn new() -> Self {
        Self
    }
}

impl Render for VariantApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Pull the registered ButtonRenderer + theme once and
        // resolve every per-variant color / size upfront. Both
        // `r` and `theme` are immutable borrows of `cx`; we
        // scope them to this block so they're gone before we
        // call the headless factories (which need `&mut App`).
        let (
            primary_bg,
            primary_fg,
            primary_pad,
            primary_radius,
            primary_min_h,
            primary_hover_bg,
            primary_active_bg,
        ) = {
            let r: &Arc<dyn ButtonRenderer> = cx
                .renderer_arc::<ButtonMarker, dyn ButtonRenderer>()
                .expect("ButtonRenderer registered");
            let theme: &Theme = cx.theme();
            let state = ButtonRenderState {
                variant: ActionVariantKind::Primary,
                ..Default::default()
            };
            (
                r.bg(&state, theme),
                r.fg(&state, theme),
                r.padding(&state, theme),
                r.border_radius(&state, theme),
                r.min_height(&state, theme),
                r.hover_bg(&state, theme),
                r.active_bg(&state, theme),
            )
        };

        let (
            danger_bg,
            danger_fg,
            danger_pad,
            danger_radius,
            danger_min_h,
            danger_hover_bg,
            danger_active_bg,
        ) = {
            let r: &Arc<dyn ButtonRenderer> = cx
                .renderer_arc::<ButtonMarker, dyn ButtonRenderer>()
                .expect("ButtonRenderer registered");
            let theme: &Theme = cx.theme();
            let state = ButtonRenderState {
                variant: ActionVariantKind::Danger,
                ..Default::default()
            };
            (
                r.bg(&state, theme),
                r.fg(&state, theme),
                r.padding(&state, theme),
                r.border_radius(&state, theme),
                r.min_height(&state, theme),
                r.hover_bg(&state, theme),
                r.active_bg(&state, theme),
            )
        };

        let primary = button("primary-btn", cx)
            .on_click(|_, _, _| {})
            .apply(
                gpui_div()
                    .bg(primary_bg)
                    .text_color(primary_fg)
                    .px(primary_pad.left)
                    .py(primary_pad.top)
                    .rounded(primary_radius)
                    .min_h(primary_min_h)
                    .cursor(gpui::CursorStyle::PointingHand)
                    .child("Primary"),
            )
            .hover(|s| s.bg(primary_hover_bg))
            .active(|s| s.bg(primary_active_bg));

        let danger = button("danger-btn", cx)
            .on_click(|_, _, _| {})
            .apply(
                gpui_div()
                    .bg(danger_bg)
                    .text_color(danger_fg)
                    .px(danger_pad.left)
                    .py(danger_pad.top)
                    .rounded(danger_radius)
                    .min_h(danger_min_h)
                    .cursor(gpui::CursorStyle::PointingHand)
                    .child("Danger"),
            )
            .hover(|s| s.bg(danger_hover_bg))
            .active(|s| s.bg(danger_active_bg));

        let neutral = button("neutral-btn", cx).default_render(cx);

        div()
            .size_full()
            .p(px(24.))
            .flex()
            .flex_col()
            .gap_3()
            .child(label("title", "Variant showcase", cx).default_render(cx))
            .child(
                label(
                    "blurb",
                    "Same headless::button, different ButtonRenderState.variant → different action.<key>.* paths from the JSON.",
                    cx,
                )
                .default_render(cx),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(label("n", "Neutral (default_render):", cx).default_render(cx))
                    .child(neutral),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(label("p", "Primary (hand-rolled apply):", cx).default_render(cx))
                    .child(primary),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(label("d", "Danger (hand-rolled apply):", cx).default_render(cx))
                    .child(danger),
            )
    }
}
