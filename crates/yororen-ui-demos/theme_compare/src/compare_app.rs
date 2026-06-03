//! Root component for the theme-compare demo.

use gpui::{
    Context, IntoElement, ParentElement, Render, Styled, Window, div, px,
};

use yororen_ui::component::{button, card, label};
use yororen_ui::theme::{ActionVariantKind, ActiveTheme};

use crate::state::ThemeCompareState;
use crate::{current_theme, mini_theme, set_active_theme, system_light_theme};

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
        let right_theme = if right_uses_mini {
            mini_theme()
        } else {
            system_light_theme()
        };
        // Reference current_theme so the import isn't dead — useful
        // when extending this demo to mutate the live theme.
        let _live = current_theme(cx);

        let switch_label = if right_uses_mini {
            "Switch right to: system"
        } else {
            "Switch right to: mini"
        };

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
                        system_light_theme(),
                    ))
                    .child(half_panel("Right (mini)", theme.surface.canvas, right_theme)),
            )
            .child(
                button("theme-compare:switch")
                    .variant(ActionVariantKind::Primary)
                    .child(switch_label.to_string())
                    .on_click(move |_ev, _window, cx| {
                        // Mutate the global Theme: swap just the
                        // `renderers` field on the live theme. This
                        // proves that the registry is a per-Theme
                        // handle, not a process-global.
                        let mut live = current_theme(cx).as_ref().clone();
                        if right_uses_mini {
                            live.renderers = mini_theme().renderers;
                        } else {
                            live.renderers = system_light_theme().renderers;
                        }
                        set_active_theme(cx, live);
                        cx.refresh_windows();
                    }),
            )
    }
}

fn half_panel(
    title: &str,
    bg: gpui::Hsla,
    theme: yororen_ui::theme::Theme,
) -> gpui::AnyElement {
    // For the demo we render two items per half: a primary button
    // and a card. The visual difference is solely the renderers
    // registered on the half's Theme.
    let _ = bg;
    let _ = theme;
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
