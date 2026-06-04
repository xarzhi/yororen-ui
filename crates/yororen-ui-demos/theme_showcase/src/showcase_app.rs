//! Root component for the theme-showcase demo.
//!
//! `with_theme` was removed (panic-unsafe). The demo now
//! cycles a single global theme through 4 configurations
//! (system / catppuccin / material / catppuccin renderers on system
//! palette) to show that palette and renderer are independent swap
//! points.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};

use yororen_ui::component::{button, card, label};
use yororen_ui::renderer::{ButtonVariant, VariantKey};
use yororen_ui::theme::{ActionVariantKind, ActiveTheme, GlobalTheme};

use crate::state::{RightThemeKind, ThemeShowcaseState};
use crate::{catppuccin_renderer_only, catppuccin_theme, material_theme, system_theme};

pub struct ThemeShowcaseApp;

impl ThemeShowcaseApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ThemeShowcaseApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<ThemeShowcaseState>();
        let kind = *state.kind.read(cx);
        let theme = cx.theme();

        let title: &'static str = match kind {
            RightThemeKind::System => "Active: theme-system",
            RightThemeKind::Catppuccin => "Active: theme-catppuccin",
            RightThemeKind::Material => "Active: theme-material",
            RightThemeKind::CatppuccinRenderersOnSystemPalette => {
                "Active: catppuccin renderers on system palette"
            }
        };
        let switch_label: &'static str = match kind {
            RightThemeKind::System => "Switch: catppuccin",
            RightThemeKind::Catppuccin => "Switch: material",
            RightThemeKind::Material => "Switch: catppuccin renderers on system",
            RightThemeKind::CatppuccinRenderersOnSystemPalette => "Switch: system",
        };
        let kind_entity = state.kind.clone();

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
                    .child(label("Theme Showcase").strong(true).text_size(px(24.)))
                    .child(label(
                        "Same UI, different skin. The active theme is global; click the button \
                         to cycle palette / renderer combos. The previous per-element \
                         `with_theme` was removed because it was panic-unsafe.",
                    )),
            )
            .child(panel(title, theme.surface.base))
            .child(
                button("theme-showcase:switch")
                    .variant(ActionVariantKind::Primary)
                    .child(switch_label.to_string())
                    .on_click(move |_ev, _window, cx| {
                        let appearance = cx.window_appearance();
                        let current = *kind_entity.read(cx);
                        let next = match current {
                            RightThemeKind::System => RightThemeKind::Catppuccin,
                            RightThemeKind::Catppuccin => RightThemeKind::Material,
                            RightThemeKind::Material => {
                                RightThemeKind::CatppuccinRenderersOnSystemPalette
                            }
                            RightThemeKind::CatppuccinRenderersOnSystemPalette => {
                                RightThemeKind::System
                            }
                        };
                        let theme = match next {
                            RightThemeKind::System => system_theme(appearance),
                            RightThemeKind::Catppuccin => catppuccin_theme(appearance),
                            RightThemeKind::Material => material_theme(appearance),
                            RightThemeKind::CatppuccinRenderersOnSystemPalette => {
                                catppuccin_renderer_only(appearance)
                            }
                        };
                        cx.set_global(GlobalTheme::new(theme));
                        kind_entity.update(cx, |k, _| *k = next);
                        cx.refresh_windows();
                    }),
            )
    }
}

fn panel(title: &str, bg: gpui::Hsla) -> gpui::AnyElement {
    div()
        .flex_1()
        .min_w(px(380.0))
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
                    button("showcase:primary")
                        .variant(ActionVariantKind::Primary)
                        .child("Primary"),
                )
                .child(
                    button("showcase:danger")
                        .variant(ActionVariantKind::Danger)
                        .child("Danger"),
                )
                .child(button("showcase:neutral").child("Neutral")),
        )
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(
                    button("showcase:mocha")
                        .variant(ButtonVariant::Custom(VariantKey::borrowed("mocha")))
                        .child("mocha"),
                )
                .child(
                    button("showcase:lavender")
                        .variant(ButtonVariant::Custom(VariantKey::borrowed("lavender")))
                        .child("lavender"),
                )
                .child(
                    button("showcase:ghost")
                        .variant(ButtonVariant::Custom(VariantKey::borrowed("ghost")))
                        .child("ghost"),
                ),
        )
        .child(
            card("showcase:card").child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(label("Card title").strong(true))
                    .child(label(
                        "Card body. Radius / padding / border come from the active renderer.",
                    )),
            ),
        )
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_cycle_covers_four_states() {
        let mut k = RightThemeKind::default();
        for _ in 0..4 {
            k = match k {
                RightThemeKind::System => RightThemeKind::Catppuccin,
                RightThemeKind::Catppuccin => RightThemeKind::Material,
                RightThemeKind::Material => {
                    RightThemeKind::CatppuccinRenderersOnSystemPalette
                }
                RightThemeKind::CatppuccinRenderersOnSystemPalette => RightThemeKind::System,
            };
        }
        assert_eq!(k, RightThemeKind::default());
    }

    #[test]
    fn catppuccin_button_renderer_distinct_from_default() {
        use yororen_ui::renderer::ButtonRenderState;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        };
        let cat_theme = catppuccin_theme(gpui::WindowAppearance::Dark);
        let sys_theme = system_theme(gpui::WindowAppearance::Dark);
        let cat_bg = cat_theme.renderers.get_button().expect("ButtonRenderer registered").bg(&state, &cat_theme);
        let sys_bg = cat_theme.renderers.get_button().expect("ButtonRenderer registered").bg(&state, &sys_theme);
        assert_ne!(cat_bg, sys_bg);
    }

    #[test]
    fn material_button_uses_pill_radius() {
        use yororen_ui::renderer::ButtonRenderState;
        let state = ButtonRenderState {
            variant: ActionVariantKind::Primary,
            ..Default::default()
        };
        let mat_theme = material_theme(gpui::WindowAppearance::Dark);
        let radius = mat_theme.renderers.get_button().expect("ButtonRenderer registered").border_radius(&state, &mat_theme);
        let radius_px = radius.to_f64();
        assert!(
            radius_px > 100.0,
            "Material button should be pill-shaped, got {}",
            radius_px
        );
    }
}
