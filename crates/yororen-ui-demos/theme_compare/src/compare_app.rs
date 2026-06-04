//! Root component for the theme-compare demo.
//!
//! `with_theme` was removed because it was panic-unsafe. The
//! "theme compare" demo now exercises a single global theme at a
//! time and lets the user flip between two theme packages to see the
//! same UI render with two different skins.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};

use yororen_ui::component::{button, card, label};
use yororen_ui::theme::{ActionVariantKind, ActiveTheme, GlobalTheme};

use crate::state::ThemeCompareState;
use crate::{mini_theme, system_theme};

pub struct ThemeCompareApp;

impl ThemeCompareApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ThemeCompareApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<ThemeCompareState>();
        let uses_mini = *state.uses_mini.read(cx);
        let theme = cx.theme();

        let title: &'static str = if uses_mini {
            "Active: theme-mini"
        } else {
            "Active: theme-system"
        };
        let switch_label = if uses_mini {
            "Switch to: system"
        } else {
            "Switch to: mini"
        };

        let uses_mini_entity = state.uses_mini.clone();

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
                    .child(label("Theme Compare").strong(true).text_size(px(24.)))
                    .child(label(
                        "Same UI, different skin. The active theme is global; click the button \
                         to swap the renderer registry. The previous per-element `with_theme` \
                         was removed because it was panic-unsafe.",
                    )),
            )
            .child(panel(title, theme.surface.base))
            .child(
                button("theme-compare:switch")
                    .variant(ActionVariantKind::Primary)
                    .child(switch_label.to_string())
                    .on_click(move |_ev, _window, cx| {
                        let appearance = cx.window_appearance();
                        let current_mini = *uses_mini_entity.read(cx);
                        let theme = if current_mini {
                            system_theme(appearance)
                        } else {
                            mini_theme(appearance)
                        };
                        cx.set_global(GlobalTheme::new(theme));
                        uses_mini_entity.update(cx, |v, _| {
                            *v = !*v;
                        });
                        cx.refresh_windows();
                    }),
            )
    }
}

fn panel(title: &str, bg: gpui::Hsla) -> gpui::AnyElement {
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
                .text_size(px(14.0)),
        )
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(
                    button("compare:primary")
                        .variant(ActionVariantKind::Primary)
                        .child("Primary"),
                )
                .child(
                    button("compare:danger")
                        .variant(ActionVariantKind::Danger)
                        .child("Danger"),
                )
                .child(button("compare:neutral").child("Neutral")),
        )
        .child(
            card("compare:card").child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(label("Card title").strong(true))
                    .child(label(
                        "Card body text. Radius / padding / border come from the renderer.",
                    )),
            ),
        )
        .into_any_element()
}
