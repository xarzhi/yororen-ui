//! yororen-ui Headless / Hooks Demo
//!
//! A minimal app demonstrating the v0.6 advanced APIs in action:
//! - 8 headless \`use_xxx()\` hooks
//! - 6 composite \`XxxRoot\` APIs
//!
//! ## Run this demo
//! ```bash
//! cd crates/yororen-ui-demos/headless_demo && cargo run
//! ```

use std::sync::Arc;

use gpui::{
    App, AppContext, Application, ClickEvent, Context, Entity, IntoElement, ParentElement, Render,
    SharedString, Styled, Window, WindowOptions, div, prelude::FluentBuilder, px, size,
};

use yororen_ui::assets::UiAsset;
use yororen_ui::component::{init as init_component, label as label_comp, panel};
use yororen_ui::composite::{
    ComboBoxRoot, DropdownMenuRoot, ModalRoot, PopoverRoot, SelectRoot, TooltipRoot,
};
use yororen_ui::component::tooltip::TooltipPlacement;
use yororen_ui::component::button as button_comp;
use yororen_ui::hooks::{
    self, ButtonProps, CheckboxProps, IconButtonProps, LabelProps, RadioProps, SwitchProps,
    TextInputProps, ToggleButtonProps,
};
use yororen_ui::locale_en;
use yororen_ui::theme::ActiveTheme;

use yororen_ui_theme_system as theme_system;

pub struct HeadlessDemoApp {
    pub counter: Entity<i32>,
    pub checked: bool,
    pub radio_choice: &'static str,
    pub toggle_selected: bool,
    pub text_value: SharedString,
    pub popover_open: bool,
    pub modal_open: bool,
}

impl HeadlessDemoApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let counter = cx.new(|_| 0i32);
        Self {
            counter,
            checked: true,
            radio_choice: "alpha",
            toggle_selected: false,
            text_value: SharedString::new("hello"),
            popover_open: false,
            modal_open: false,
        }
    }
}

impl Render for HeadlessDemoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let count = *self.counter.read(cx);
        // We need a few entities + focus handles before we can
        // borrow `cx` immutably via `cx.theme()`.
        let me = cx.entity().clone();

        // ---- Hooks ----
        let inc = hooks::use_button("inc", cx).on_click({
            let counter = self.counter.clone();
            move |_ev, _w, cx| {
                counter.update(cx, |c, cx| {
                    *c += 1;
                    cx.notify();
                });
            }
        });
        let dec = hooks::use_button("dec", cx).on_click({
            let counter = self.counter.clone();
            move |_ev, _w, cx| {
                counter.update(cx, |c, cx| {
                    *c -= 1;
                    cx.notify();
                });
            }
        });
        let reset = hooks::use_button("reset", cx).on_click({
            let counter = self.counter.clone();
            move |_ev, _w, cx| {
                counter.update(cx, |c, cx| {
                    *c = 0;
                    cx.notify();
                });
            }
        });
        let count_label = hooks::use_label(format!("Count: {count}"), cx).strong(true);
        let _ = count_label;

        let me_for_switch = me.clone();
        let switch = SwitchProps {
            id: "demo-switch".into(),
            checked: self.checked,
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_toggle: Some(Arc::new(move |next, _ev, _w, cx| {
                me_for_switch.update(cx, |s, cx| {
                    s.checked = next;
                    cx.notify();
                });
            })),
        };

        let me2 = me.clone();
        let checkbox = CheckboxProps {
            id: "demo-checkbox".into(),
            checked: self.checked,
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_toggle: Some(Arc::new(move |next, _ev, _w, cx| {
                me2.update(cx, |s, cx| {
                    s.checked = next;
                    cx.notify();
                });
            })),
        };

        let me3 = me.clone();
        let radio_a = RadioProps {
            id: "demo-radio-a".into(),
            checked: self.radio_choice == "alpha",
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_toggle: Some(Arc::new(move |_n, _ev, _w, cx| {
                me3.update(cx, |s, cx| {
                    s.radio_choice = "alpha";
                    cx.notify();
                });
            })),
        };
        let me4 = me.clone();
        let radio_b = RadioProps {
            id: "demo-radio-b".into(),
            checked: self.radio_choice == "beta",
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_toggle: Some(Arc::new(move |_n, _ev, _w, cx| {
                me4.update(cx, |s, cx| {
                    s.radio_choice = "beta";
                    cx.notify();
                });
            })),
        };

        let me5 = me.clone();
        let toggle = ToggleButtonProps {
            id: "demo-toggle".into(),
            label: "Bold".into(),
            selected: self.toggle_selected,
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_toggle: Some(Arc::new(move |_n, _ev, _w, cx| {
                me5.update(cx, |s, cx| {
                    s.toggle_selected = !s.toggle_selected;
                    cx.notify();
                });
            })),
        };

        let text_input = TextInputProps {
            id: "demo-text".into(),
            value: self.text_value.to_string(),
            placeholder: "Type something".into(),
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_change: None,
            on_submit: None,
            max_length: Some(64),
        };

        let icon_btn = IconButtonProps {
            id: "demo-icon-btn".into(),
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_click: Some(Arc::new(|_ev, _w, _cx| {})),
        };

        let me6 = me.clone();
        let popover_btn = hooks::use_button("popover-btn", cx).on_click(move |_ev, _w, cx| {
            me6.update(cx, |s, cx| {
                s.popover_open = !s.popover_open;
                cx.notify();
            });
        });
        let me7 = me.clone();
        let modal_btn = hooks::use_button("modal-btn", cx).on_click(move |_ev, _w, cx| {
            me7.update(cx, |s, cx| {
                s.modal_open = !s.modal_open;
                cx.notify();
            });
        });
        let close_modal: Arc<dyn Fn(&mut Window, &mut App) + Send + Sync> = {
            let me = me.clone();
            Arc::new(move |_w: &mut Window, cx: &mut App| {
                me.update(cx, |s, cx| {
                    s.modal_open = false;
                    cx.notify();
                });
            })
        };

        // Now we can borrow `cx.theme()` immutably.
        let theme = cx.theme();

        // ---- Layout: 3 columns ----
        let top_bar = div()
            .flex()
            .items_center()
            .justify_between()
            .p(px(16.))
            .border_b_1()
            .border_color(theme.border.default)
            .bg(theme.surface.canvas)
            .child(
                label_comp("yororen-ui v0.6 — Headless & Composite").strong(true),
            )
            .child(label_comp("Phase I.1 + I.2 demo").muted(true));

        let hooks_col = render_hooks_col(
            &inc,
            &dec,
            &reset,
            count,
            switch,
            checkbox,
            radio_a,
            radio_b,
            toggle,
            text_input,
            icon_btn,
            theme,
        );

        let composite_col = render_composite_col(
            &popover_btn,
            &modal_btn,
            close_modal,
            self.popover_open,
            self.modal_open,
            self.radio_choice,
            theme,
        );

        let info_col = render_info_col(theme);

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.surface.base)
            .text_color(theme.content.primary)
            .child(top_bar)
            .child(
                div()
                    .flex_1()
                    .flex()
                    .gap(px(16.))
                    .p(px(16.))
                    .child(hooks_col)
                    .child(composite_col)
                    .child(info_col),
            )
    }
}

fn render_hooks_col(
    inc: &ButtonProps,
    dec: &ButtonProps,
    reset: &ButtonProps,
    count: i32,
    switch: SwitchProps,
    checkbox: CheckboxProps,
    radio_a: RadioProps,
    radio_b: RadioProps,
    toggle: ToggleButtonProps,
    text_input: TextInputProps,
    icon_btn: IconButtonProps,
    theme: &yororen_ui::theme::Theme,
) -> impl IntoElement {
    panel("hooks-col")
        .p(px(16.))
        .child(label_comp("1. Hooks (8 use_xxx)").strong(true))
        .child(div().h(px(8.)))
        .child(label_comp("use_button: counter").muted(true))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(dec.clone().apply(div().px(px(10.)).py(px(4.)).rounded(px(6.)).bg(theme.surface.hover)).child(label_comp("−").strong(true)))
                .child(label_comp(format!("Count: {count}")).strong(true))
                .child(inc.clone().apply(div().px(px(10.)).py(px(4.)).rounded(px(6.)).bg(theme.surface.hover)).child(label_comp("+").strong(true)))
                .child(reset.clone().apply(div().px(px(10.)).py(px(4.)).rounded(px(6.))).child(label_comp("reset").muted(true))),
        )
        .child(div().h(px(8.)))
        .child(label_comp("use_switch").muted(true))
        .child(div().flex().items_center().gap(px(8.)).child(switch.clone().apply(div().w(px(36.)).h(px(20.)).rounded(px(999.)).bg(if switch.checked { theme.action.primary.bg } else { theme.surface.sunken }))))
        .child(div().h(px(8.)))
        .child(label_comp("use_checkbox").muted(true))
        .child(div().flex().items_center().gap(px(8.)).child(checkbox.clone().apply(div().w(px(18.)).h(px(18.)).rounded(px(4.)).bg(if checkbox.checked { theme.action.primary.bg } else { theme.surface.base }))))
        .child(div().h(px(8.)))
        .child(label_comp("use_radio").muted(true))
        .child(
            div()
                .flex()
                .gap(px(12.))
                .child(radio_a.clone().apply(div().flex().items_center().gap(px(4.)).child(div().w(px(16.)).h(px(16.)).rounded(px(999.)).bg(if radio_a.checked { theme.action.primary.bg } else { theme.surface.base })).child(label_comp("Alpha"))))
                .child(radio_b.clone().apply(div().flex().items_center().gap(px(4.)).child(div().w(px(16.)).h(px(16.)).rounded(px(999.)).bg(if radio_b.checked { theme.action.primary.bg } else { theme.surface.base })).child(label_comp("Beta")))),
        )
        .child(div().h(px(8.)))
        .child(label_comp("use_toggle_button").muted(true))
        .child(toggle.clone().apply(div().px(px(10.)).py(px(4.)).rounded(px(6.)).bg(if toggle.selected { theme.action.primary.bg } else { theme.surface.hover })).child(label_comp(toggle.label.clone())))
        .child(div().h(px(8.)))
        .child(label_comp("use_text_input").muted(true))
        .child(
            text_input
                .clone()
                .apply(div().px(px(8.)).py(px(4.)).rounded(px(6.)).bg(theme.surface.base).border_1().border_color(theme.border.default))
                .child(label_comp(text_input.value.clone())),
        )
        .child(div().h(px(8.)))
        .child(label_comp("use_icon_button").muted(true))
        .child(icon_btn.clone().apply(div().w(px(32.)).h(px(32.)).rounded(px(999.)).bg(theme.surface.hover).items_center().justify_center()).child(label_comp("✕").strong(true)))
}

fn render_composite_col(
    popover_btn: &ButtonProps,
    modal_btn: &ButtonProps,
    close_modal: Arc<dyn Fn(&mut Window, &mut App) + Send + Sync>,
    popover_open: bool,
    modal_open: bool,
    radio_choice: &str,
    theme: &yororen_ui::theme::Theme,
) -> impl IntoElement {
    let close_modal = close_modal.clone();
    let _ = (popover_btn, modal_btn); // already wired through Root APIs below

    let popover_root = PopoverRoot::new("demo-popover")
        .open(popover_open)
        .trigger(label_comp("Open popover"))
        .content(div().p(px(12.)).child(label_comp("Popover body")));

    let modal_root = ModalRoot::new("demo-modal")
        .open(modal_open)
        .title("Confirm")
        .content(div().p(px(8.)).child(label_comp("Modal body.")))
        .actions(label_comp("actions"))
        .on_close(move |w, cx| (close_modal)(w, cx));

    let dropdown_root = DropdownMenuRoot::new("demo-dropdown")
        .label("Choose")
        .item("a", "Apple")
        .item("b", "Banana")
        .item("c", "Cherry");

    let select_root = SelectRoot::new("demo-select")
        .option("red", "Red")
        .option("green", "Green")
        .option("blue", "Blue");

    let combo_root = ComboBoxRoot::new("demo-combo")
        .option("ny", "New York")
        .option("sf", "San Francisco")
        .option("tk", "Tokyo");

    // TooltipRoot — a real hover-popup built on top of the
    // underlying `Tooltip` builder. Hover the button to see the
    // tooltip. `dismiss_on_escape = true` honours Esc.
    let tooltip_root = TooltipRoot::new("demo-tooltip")
        .trigger(button_comp("tip-btn").child("Hover me"))
        .text("This is a TooltipRoot popup (Phase I.2 P0 real impl)")
        .placement(TooltipPlacement::Bottom)
        .dismiss_on_escape(true);

    let _ = close_modal;

    panel("composite-col")
        .p(px(16.))
        .child(label_comp("2. Composites (6 XxxRoot)").strong(true))
        .child(div().h(px(8.)))
        .child(label_comp("PopoverRoot").muted(true))
        .child(popover_root)
        .child(div().h(px(8.)))
        .child(label_comp("ModalRoot").muted(true))
        .child(modal_root)
        .child(label_comp(if modal_open { "(open)" } else { "(closed)" }).muted(true))
        .child(div().h(px(8.)))
        .child(label_comp("DropdownMenuRoot").muted(true))
        .child(dropdown_root)
        .child(div().h(px(8.)))
        .child(label_comp("SelectRoot").muted(true))
        .child(select_root)
        .child(div().h(px(8.)))
        .child(label_comp("ComboBoxRoot").muted(true))
        .child(combo_root)
        .child(div().h(px(8.)))
        .child(label_comp("TooltipRoot (hover the button)").muted(true))
        .child(tooltip_root)
        .child(div().h(px(8.)))
        .child(label_comp(format!("radio_choice = {radio_choice}")).muted(true))
        .child(div().h(px(8.)))
        .child(label_comp("(no-op modal_btn placeholder)").muted(true))
}

fn render_info_col(theme: &yororen_ui::theme::Theme) -> impl IntoElement {
    panel("info-col")
        .p(px(16.))
        .child(label_comp("3. About this demo").strong(true))
        .child(div().h(px(8.)))
        .child(label_comp("Phase I.1 — 8 use_xxx hooks").strong(true))
        .child(label_comp("Phase I.2 — 6 XxxRoot composite APIs").strong(true))
        .child(div().h(px(8.)))
        .child(label_comp("Switch themes via the meta-crate's feature flags:").muted(true))
        .child(label_comp("  yororen-ui = { version = \"0.3\", features = [\"catppuccin\"] }").muted(true))
        .child(label_comp("  yororen-ui = { version = \"0.3\", features = [\"material\"] }").muted(true))
        .child(div().h(px(8.)))
        .child(label_comp("See HOOK_GUIDE.md for the full API walkthrough.").muted(true))
        .child(div().h(px(8.)))
        .child(label_comp("Active theme:").muted(true))
        .child(label_comp(format!("surface.base: {:?}", theme.surface.base)))
        .child(label_comp(format!("action.primary: {:?}", theme.action.primary.bg)))
}

fn main() {
    let app = Application::new().with_assets(UiAsset);
    app.run(|cx: &mut App| {
        init_component(cx);
        theme_system::install(cx, cx.window_appearance());
        locale_en::install(cx);

        let options = WindowOptions {
            window_bounds: Some(gpui::WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(1100.0), px(700.0)),
                cx,
            ))),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| cx.new(HeadlessDemoApp::new));
    });
}

// Suppress unused warnings for the demo-only type re-exports.
#[allow(dead_code)]
fn _suppress(_: ClickEvent) {}
