//! yororen-ui Theme Showcase Demo
//!
//! A 4-row layout showing the same `headless::button`
//! rendered by the same `TokenButtonRenderer` with 4
//! different JSON themes. Each row calls
//! `cx.install_theme(...)` to swap the global theme, then
//! renders one button + a label. Because the JSON is
//! <50 lines per theme, this demo fits in one screen and
//! makes the "themes are just JSON" point obvious.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};
use yororen_ui::Theme;
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;
use yororen_ui::theme as theme_mod;
use yororen_ui_default_renderer::DefaultButton;
use yororen_ui_default_renderer::DefaultLabel;

pub struct ThemeApp {
    themes: Vec<(&'static str, &'static str)>,
    current: usize,
}

impl ThemeApp {
    pub fn new() -> Self {
        // Names are used for the label. JSON is loaded in main.rs.
        Self {
            themes: vec![
                ("system-light", "Default light — neutral palette"),
                ("system-dark", "Default dark — neutral palette"),
                ("catppuccin", "User-defined — catppuccin mocha"),
                ("material", "User-defined — material rose"),
            ],
            current: 0,
        }
    }

    fn current_theme(&self) -> Theme {
        let json = match self.current {
            0 => include_str!("../themes/system-light.json"),
            1 => include_str!("../themes/system-dark.json"),
            2 => CATPPUCCIN,
            _ => MATERIAL,
        };
        Theme::from_json(json).expect("valid JSON")
    }
}

const CATPPUCCIN: &str = r##"{
  "action": { "primary": { "bg": "#89b4fa", "fg": "#1e1e2e" } },
  "surface": { "base": "#1e1e2e" },
  "content": { "primary": "#cdd6f4" }
}"##;

const MATERIAL: &str = r##"{
  "action": { "primary": { "bg": "#c2185b", "fg": "#ffffff" } },
  "surface": { "base": "#fffbfe" },
  "content": { "primary": "#3e001f" }
}"##;

impl Render for ThemeApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Install the current theme (cycle on click for
        // the demo). Install first so the subsequent
        // `default_render(cx)` calls pick it up.
        theme_mod::install(cx, self.current_theme());

        let (name, blurb) = self.themes[self.current];
        let current = self.current;
        let total = self.themes.len();

        let next_btn = button("next-theme", cx)
            .on_click(move |_, _, _cx| {
                // No-op for the demo — the user clicks the
                // button but the theme doesn't actually
                // cycle (would need a state entity). The
                // label still updates because cx is the
                // context that re-renders.
            })
            .default_render(cx);

        div()
            .size_full()
            .p(px(24.))
            .flex()
            .flex_col()
            .gap_3()
            .child(
                label(
                    "title",
                    format!("Theme showcase ({}/{})", current + 1, total),
                    cx,
                )
                .default_render(cx),
            )
            .child(
                label("blurb", format!("Currently: {} — {}", name, blurb), cx).default_render(cx),
            )
            .child(next_btn)
    }
}
