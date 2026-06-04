//! Root component for the theme-showcase demo. Renders a 2-pane
//! window: left half is the system theme, right half is one of
//! three configurations (system / catppuccin / catppuccin
//! renderers on system palette). A switcher button cycles through
//! the three.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};

use yororen_ui::component::{button, card, label, with_theme};
use yororen_ui::renderer::{ButtonVariant, VariantKey};
use yororen_ui::theme::{ActionVariantKind, ActiveTheme};

use crate::state::{RightThemeKind, ThemeShowcaseState};
use crate::{catppuccin_renderer_only, catppuccin_theme, material_theme, system_theme};

const HALF_HEADER_PX: f32 = 14.0;

pub struct ThemeShowcaseApp;

impl ThemeShowcaseApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for ThemeShowcaseApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<ThemeShowcaseState>();
        let right_kind = state.right_kind(cx);
        let theme = cx.theme();

        let appearance = cx.window_appearance();
        let right_theme = match right_kind {
            RightThemeKind::System => system_theme(appearance),
            RightThemeKind::Catppuccin => catppuccin_theme(appearance),
            RightThemeKind::Material => material_theme(appearance),
            RightThemeKind::CatppuccinRenderersOnSystemPalette => {
                catppuccin_renderer_only(appearance)
            }
        };

        let right_title: &'static str = match right_kind {
            RightThemeKind::System => "Right (system)",
            RightThemeKind::Catppuccin => "Right (catppuccin)",
            RightThemeKind::Material => "Right (material 3)",
            RightThemeKind::CatppuccinRenderersOnSystemPalette => {
                "Right (catppuccin renderers, system palette)"
            }
        };
        let switch_label: &'static str = match right_kind {
            RightThemeKind::System => "Switch right: catppuccin",
            RightThemeKind::Catppuccin => "Switch right: material",
            RightThemeKind::Material => "Switch right: catppuccin renderers on system",
            RightThemeKind::CatppuccinRenderersOnSystemPalette => "Switch right: system",
        };
        let right_bg = theme.surface.canvas;
        let right_kind_entity = state.right_kind.clone();

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
                        label("Theme Showcase").strong(true).text_size(px(24.)),
                    )
                    .child(label(
                        "Same UI, different skins. Left = theme-system. Right cycles through 3 \
                         configurations to show that palette and renderer are independent swap points.",
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
                button("theme-showcase:switch")
                    .variant(ActionVariantKind::Primary)
                    .child(switch_label.to_string())
                    .on_click(move |_ev, _window, cx| {
                        right_kind_entity.update(cx, |k, _| {
                            *k = match *k {
                                RightThemeKind::System => RightThemeKind::Catppuccin,
                                RightThemeKind::Catppuccin => RightThemeKind::Material,
                                RightThemeKind::Material => {
                                    RightThemeKind::CatppuccinRenderersOnSystemPalette
                                }
                                RightThemeKind::CatppuccinRenderersOnSystemPalette => {
                                    RightThemeKind::System
                                }
                            };
                        });
                        cx.refresh_windows();
                    }),
            )
    }
}

/// Half-window panel: renders the same set of components on both
/// sides so a side-by-side diff is unmistakable.
fn half_panel(title: &str, bg: gpui::Hsla) -> gpui::AnyElement {
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
                .text_size(px(HALF_HEADER_PX)),
        )
        // Row 1: 3 builtin variants.
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(
                    button("primary")
                        .variant(ActionVariantKind::Primary)
                        .child("Primary"),
                )
                .child(
                    button("danger")
                        .variant(ActionVariantKind::Danger)
                        .child("Danger"),
                )
                .child(button("neutral").child("Neutral")),
        )
        // Row 2: 3 Catppuccin custom variants.
        .child(
            div()
                .flex()
                .gap(px(8.))
                .child(
                    button("mocha")
                        .variant(ButtonVariant::Custom(VariantKey::borrowed("mocha")))
                        .child("mocha"),
                )
                .child(
                    button("lavender")
                        .variant(ButtonVariant::Custom(VariantKey::borrowed("lavender")))
                        .child("lavender"),
                )
                .child(
                    button("ghost")
                        .variant(ButtonVariant::Custom(VariantKey::borrowed("ghost")))
                        .child("ghost"),
                ),
        )
        // Row 3: a card with body text.
        .child(
            card("card").child(
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
    fn right_kind_cycle_covers_four_states() {
        let mut k = RightThemeKind::default();
        // 4 states → 4 cycles to land back on default.
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
        let cat_bg = cat_theme.renderers.button.bg(&state, &cat_theme);
        let sys_bg = cat_theme.renderers.button.bg(&state, &sys_theme);
        // Both renderers are CatppuccinButtonRenderer, so the
        // difference comes from the palette. They should differ.
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
        let radius = mat_theme.renderers.button.border_radius(&state, &mat_theme);
        // Material 3 buttons are pill-shaped (~999 px).
        let radius_px = radius.to_f64();
        assert!(
            radius_px > 100.0,
            "Material button should be pill-shaped, got {}",
            radius_px
        );
    }
}
