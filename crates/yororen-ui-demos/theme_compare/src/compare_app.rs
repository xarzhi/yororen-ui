//! Root component for the theme-compare demo.

use gpui::{
    Context, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

use yororen_ui::component::{button, card, label, with_theme};
use yororen_ui::theme::{ActionVariantKind, ActiveTheme};

use crate::state::ThemeCompareState;
use crate::{mini_theme, system_theme};

/// Half-window panel header.
const SIDE_HEADER_FONT_PX: f32 = 14.0;

pub struct ThemeCompareApp;

impl ThemeCompareApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ThemeCompareApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<ThemeCompareState>();
        let right_uses_mini = state.right_uses_mini(cx);
        let theme = cx.theme();
        // Resolve the right-half palette against the current OS
        // appearance so the system half follows light/dark and the
        // mini half uses the matching palette + mini registry.
        let appearance = cx.window_appearance();
        let right_theme = if right_uses_mini {
            mini_theme(appearance)
        } else {
            system_theme(appearance)
        };

        let right_title: &'static str = if right_uses_mini {
            "Right (mini)"
        } else {
            "Right (system)"
        };
        let switch_label = if right_uses_mini {
            "Switch right to: system"
        } else {
            "Switch right to: mini"
        };
        // Extract primitives the right-half closure needs *before*
        // capturing, so the closure can be `'static` (required by
        // `RenderOnce`).
        let right_bg = theme.surface.canvas;

        // Clone the entity handle so the click handler can take
        // `&mut App` for the update without conflicting with the
        // `&App` borrow of the global above.
        let right_uses_mini_entity = state.right_uses_mini.clone();

        div()
            .size_full()
            .bg(theme.surface.canvas)
            .flex()
            .flex_col()
            .gap(px(20.))
            .p(px(24.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(
                        label("Theme Compare").strong(true).text_size(px(24.)),
                    )
                    .child(label(
                        "Same UI, different skin. Left = theme-system, right = theme-mini. \
                         Click the button to swap the right half's renderer registry.",
                    )),
            )
            .child(
                div()
                    .flex()
                    .gap(px(16.))
                    .child(half_panel(
                        "Left (system)",
                        theme.surface.base,
                    ))
                    .child(with_theme(right_theme, move || {
                        half_panel(right_title, right_bg)
                    })),
            )
            .child(
                button("theme-compare:switch")
                    .variant(ActionVariantKind::Primary)
                    .child(switch_label.to_string())
                    .on_click(move |_ev, _window, cx| {
                        // Just flip the boolean. The next render
                        // picks the opposite registry via
                        // `with_theme(right_theme, ...)`. The
                        // global theme is untouched — with_theme
                        // scopes the override to the right half.
                        right_uses_mini_entity.update(cx, |v, _| {
                            *v = !*v;
                        });
                        cx.refresh_windows();
                    }),
            )
    }
}

fn half_panel(
    title: &str,
    bg: gpui::Hsla,
) -> gpui::AnyElement {
    // For the demo we render two items per half: a primary button
    // and a card. The visual difference comes from the active
    // theme — the left half uses the global theme (theme-system),
    // the right half is wrapped in `with_theme(...)` by the caller
    // so its `cx.theme()` resolves to the override.
    div()
        .flex_1()
        .min_w(px(360.))
        .p(px(16.))
        .bg(bg)
        .rounded_lg()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(
            label(title.to_string())
                .strong(true)
                .text_size(px(SIDE_HEADER_FONT_PX)),
        )
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(button("left:primary").variant(ActionVariantKind::Primary).child("Primary"))
                .child(button("left:danger").variant(ActionVariantKind::Danger).child("Danger"))
                .child(button("left:neutral").child("Neutral")),
        )
        .child(
            card("left:card").child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(label("Card title").strong(true))
                    .child(label("Card body text. Radius / padding / border come from the renderer.")),
            ),
        )
        .into_any_element()
}
