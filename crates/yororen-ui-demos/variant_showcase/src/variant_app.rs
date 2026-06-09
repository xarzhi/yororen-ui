//! yororen-ui Variant Showcase Demo
//!
//! Three side-by-side buttons showing that swapping
//! `ActionVariantKind` (Neutral / Primary / Danger) on the
//! *same* `headless::button` factory re-routes the
//! `ButtonRenderer` through different `action.<key>.*` token
//! paths in the theme JSON.
//!
//! A fourth button demonstrates the `apply` escape hatch:
//! when you need a shape that `default_render` doesn't
//! provide (e.g. a fixed-size pill), you can read the same
//! renderer tokens by hand and compose your own `div`,
//! then wire it back to the headless button via `apply(...)`.

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
        // Read Primary tokens once for the override example
        // below. Scoped to a block so the immutable borrow of
        // `cx` is released before we call the headless factories
        // (which need `&mut App`).
        let (primary_bg, primary_fg, primary_hover_bg, primary_active_bg) = {
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
                r.hover_bg(&state, theme),
                r.active_bg(&state, theme),
            )
        };

        // === Three "default_render" buttons: same factory,
        // different variant. Only `ButtonRenderState.variant`
        // changes, which re-routes the renderer to a different
        // `action.<key>.*` token path. ===
        let neutral = button("neutral-btn", cx)
            .variant(ActionVariantKind::Neutral)
            .default_render(cx)
            .child("Neutral");

        let primary = button("primary-btn", cx)
            .variant(ActionVariantKind::Primary)
            .default_render(cx)
            .child("Primary");

        let danger = button("danger-btn", cx)
            .variant(ActionVariantKind::Danger)
            .default_render(cx)
            .child("Danger");

        // === Escape hatch: same Primary tokens, but a shape
        // that `default_render` doesn't expose — fixed 220×56
        // pill. We pull the theme colors from the renderer by
        // hand, build our own `div`, and wire a11y/click back
        // through `apply`. The hover/active overrides re-use
        // the renderer's own hover/active tokens, so the
        // visual feedback stays theme-driven. ===
        let pill = button("pill-btn", cx)
            .on_click(|_, _, _| {})
            .apply(
                div()
                    .bg(primary_bg)
                    .text_color(primary_fg)
                    .w(px(220.))
                    .h(px(56.))
                    .rounded(px(28.))
                    .cursor(gpui::CursorStyle::PointingHand)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("Pill - custom shape"),
            )
            .hover(|s| s.bg(primary_hover_bg))
            .active(|s| s.bg(primary_active_bg));

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
                    .child(label("p", "Primary (default_render):", cx).default_render(cx))
                    .child(primary),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(label("d", "Danger (default_render):", cx).default_render(cx))
                    .child(danger),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        label(
                            "o",
                            "Override (apply + custom shape):",
                            cx,
                        )
                        .default_render(cx),
                    )
                    .child(pill),
            )
    }
}
