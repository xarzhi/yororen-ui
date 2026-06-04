//! yororen-ui Headless / Hooks Demo
//!
//! A minimal app demonstrating the advanced APIs in action:
//! - 8 headless \`use_xxx()\` hooks
//! - 6 composite \`XxxRoot\` APIs
//!
//! ## Run this demo
//! ```bash
//! cd crates/yororen-ui-demos/headless_demo && cargo run
//! ```

use std::sync::Arc;

use gpui::{
    App, AppContext, Application, ClickEvent, Context, Entity, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, WindowOptions,
    div, prelude::FluentBuilder, px, size,
};

use yororen_ui::assets::UiAsset;
use yororen_ui::component::{
    ComboBoxOption, SelectOption, combo_box, init as init_component, label as label_comp, modal,
    panel, popover, select, tooltip,
};
use yororen_ui::component::tooltip::TooltipPlacement;
use yororen_ui::component::button as button_comp;
use yororen_ui::hooks::{
    self, ButtonProps, CheckboxProps, IconButtonProps, RadioProps, SwitchProps, TextInputProps,
    ToggleButtonProps,
};
use yororen_ui::locale_en;
use yororen_ui::theme::ActiveTheme;

use yororen_ui_theme_system as theme_system;

pub struct HeadlessDemoApp {
    pub counter: Entity<i32>,
    pub switch_checked: bool,
    pub checkbox_checked: bool,
    pub radio_choice: &'static str,
    pub toggle_selected: bool,
    pub text_value: SharedString,
    pub icon_btn_clicks: u32,
    pub popover_open: bool,
    pub modal_open: bool,
    // The text input's `FocusHandle` is a stateful identity — it
    // must be created *once* in `new()` and reused across renders.
    // `cx.focus_handle()` returns a fresh handle (new `FocusId`)
    // on every call; if we created one inside `render()`, the
    // handle would change between frames, so a click that set
    // focus would be invalidated by the next `refresh()`. Same
    // reason the real `text_input` component stores the handle
    // inside its `Entity<TextInputState>`.
    pub text_focus_handle: gpui::FocusHandle,
}

impl HeadlessDemoApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let counter = cx.new(|_| 0i32);
        Self {
            counter,
            switch_checked: true,
            checkbox_checked: false,
            radio_choice: "alpha",
            toggle_selected: false,
            text_value: SharedString::new("hello"),
            icon_btn_clicks: 0,
            popover_open: false,
            modal_open: false,
            text_focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for HeadlessDemoApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            checked: self.switch_checked,
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_toggle: Some(Arc::new(move |next, _ev, _w, cx| {
                me_for_switch.update(cx, |s, cx| {
                    s.switch_checked = next;
                    cx.notify();
                });
            })),
        };

        let me2 = me.clone();
        let checkbox = CheckboxProps {
            id: "demo-checkbox".into(),
            checked: self.checkbox_checked,
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_toggle: Some(Arc::new(move |next, _ev, _w, cx| {
                me2.update(cx, |s, cx| {
                    s.checkbox_checked = next;
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

        // `use_text_input` is a *headless* hook — it only tracks
        // focus via `FocusHandle` and exposes an `on_change`
        // signal. Per the hook's own docstring (hooks.rs:626-637),
        // "the caller is responsible for actually rendering the
        // `text()` content; this method only wires the interaction
        // model." So to make typing work we hand-roll the key
        // dispatch on top of the hook. The hook gives us:
        //   - `focus_handle` so the div participates in focus
        //   - `on_change` to demonstrate the signal contract
        // We add:
        //   - `on_key_down` on the styled div for backspace +
        //     printable chars
        //   - a label child rendering the current value
        // (No IME / clipboard / selection — this is a demo of the
        // hook, not a production text input. For those, use the
        // real `text_input()` component.)
        let me_for_text = me.clone();
        let text_input_hook = TextInputProps {
            id: "demo-text".into(),
            value: self.text_value.to_string(),
            placeholder: "Type something".into(),
            disabled: false,
            // Reuse the *stable* focus handle from the app
            // state. `cx.focus_handle()` would mint a new id
            // every render, breaking focus across refreshes.
            focus_handle: self.text_focus_handle.clone(),
            on_change: Some(Arc::new(move |v, _w, cx| {
                me_for_text.update(cx, |s, cx| {
                    s.text_value = SharedString::from(v);
                    cx.notify();
                });
            })),
            on_submit: None,
            max_length: Some(64),
        };
        // Cloned (Arc-bumped) so the key-down closure can call
        // the same `on_change` handler the hook exposes.
        let text_input_hook_for_keys = text_input_hook.clone();

        let me_for_icon = me.clone();
        let icon_btn = IconButtonProps {
            id: "demo-icon-btn".into(),
            disabled: false,
            focus_handle: cx.focus_handle(),
            on_click: Some(Arc::new(move |_ev, _w, cx| {
                me_for_icon.update(cx, |s, cx| {
                    s.icon_btn_clicks += 1;
                    // Clear the text input. The framework's
                    // TextInputState syncs from `.content(...)`
                    // via the `last_prop_content` guard
                    // (text_input.rs:914), so on the next render
                    // the internal state is overwritten with "".
                    s.text_value = SharedString::new("");
                    cx.notify();
                });
            })),
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
        // Visual focus feedback (border thickens and changes to
        // `theme.border.focus`). The hook's `track_focus` only
        // makes the element eligible for focus styling — it does
        // not pick a style for us, so we apply the conditional
        // border ourselves. `compute_input_style` is the
        // canonical source for the focus color in the framework.
        let is_text_focused = self.text_focus_handle.is_focused(window);

        // Build the styled `use_text_input` element here (after
        // `theme` is borrowed) so we can use theme colors for
        // bg / border. The hook is *headless*; we hand-roll
        // backspace + printable-char handling on `on_key_down`
        // since the hook explicitly delegates keystroke
        // dispatch to the caller (see hooks.rs:626-637).
        let me_for_keys = me.clone();
        // The hook's `apply()` only sets `track_focus(...)` plus
        // an empty `on_mouse_down` — it does NOT focus the handle
        // on click, so the input would never receive focus. We
        // clone the handle and chain a real `on_mouse_down` that
        // explicitly calls `.focus(window)` (this overrides the
        // hook's empty handler via the fluent builder).
        let text_input_focus = text_input_hook.focus_handle.clone();
        let text_input_el = text_input_hook
            .apply(
                div()
                    .px(px(8.))
                    .py(px(4.))
                    .rounded(px(6.))
                    .bg(theme.surface.base)
                    .border_1()
                    .border_color(theme.border.default)
                    .min_h(px(20.))
                    .flex()
                    .items_center(),
            )
            .on_mouse_down(
                gpui::MouseButton::Left,
                move |_ev, window, _cx| {
                    text_input_focus.focus(window);
                },
            )
            .on_key_down(move |event, window, cx| {
                let ks = &event.keystroke;
                // Only react to keys while the input holds focus.
                if !text_input_hook_for_keys.is_focused(window) {
                    return;
                }
                let max = text_input_hook_for_keys.max_length.unwrap_or(64);
                let on_change = text_input_hook_for_keys.on_change.clone();

                if ks.key == "backspace" {
                    me_for_keys.update(cx, |s, cx| {
                        if !s.text_value.is_empty() {
                            let s_ref: &str = s.text_value.as_ref();
                            let mut end = s_ref.len();
                            while end > 0 && !s_ref.is_char_boundary(end) {
                                end -= 1;
                            }
                            if end > 0 {
                                s.text_value = SharedString::from(s_ref[..end].to_string());
                            }
                            cx.notify();
                        }
                    });
                    if let Some(handler) = on_change.as_ref() {
                        let cur = me_for_keys.read(cx).text_value.to_string();
                        handler(cur, window, cx);
                    }
                } else if let Some(ch) = ks.key.chars().next() {
                    // Treat single-char keys as printable inserts;
                    // skip control chars and lone modifier keys.
                    if ch.is_control() {
                        return;
                    }
                    me_for_keys.update(cx, |s, cx| {
                        if s.text_value.len() < max {
                            let mut new_val = s.text_value.to_string();
                            new_val.push(ch);
                            s.text_value = SharedString::from(new_val);
                            cx.notify();
                        }
                    });
                    if let Some(handler) = on_change.as_ref() {
                        let cur = me_for_keys.read(cx).text_value.to_string();
                        handler(cur, window, cx);
                    }
                }
            })
            .when(is_text_focused, |s| {
                // Match the real `text_input` component:
                // thicken the border from 1 to 2 and switch to
                // the focus color when the handle is focused.
                s.border_2().border_color(theme.border.focus)
            })
            .child(label_comp(self.text_value.clone()));

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
                label_comp("yororen-ui — Headless & Composite").strong(true),
            )
            .child(label_comp("Headless hooks + composite APIs").muted(true));

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
            text_input_el,
            icon_btn,
            self.icon_btn_clicks,
            theme,
        );

        let composite_col = render_composite_col(
            &popover_btn,
            &modal_btn,
            close_modal.clone(),
            self.popover_open,
            self.modal_open,
            self.radio_choice,
            theme,
        );

        let info_col = render_info_col(theme);

        // The modal is mounted only when needed; the builder
        // doesn't have an `open` setter, so the call site gates
        // it. The builder's `on_close` is 2-arg (no reason).
        let modal_root = {
            let close = close_modal.clone();
            modal()
                .id("demo-modal")
                .title("Confirm")
                .content(div().p(px(8.)).child(label_comp("Modal body.")))
                .actions(label_comp("actions"))
                .closable(true)
                .on_close(move |w, cx| (close)(w, cx))
        };

        let modal_scrim = theme
            .renderers
            .modal
            .scrim(&yororen_ui::renderer::ModalRenderState::default(), theme);
        let close_modal_for_scrim = close_modal.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.surface.base)
            .text_color(theme.content.primary)
            .child(top_bar)
            .child(
                div()
                    .id("headless-demo-scroll")
                    .flex_1()
                    .overflow_scroll()
                    .flex()
                    .gap(px(16.))
                    .p(px(16.))
                    .child(hooks_col)
                    .child(composite_col)
                    .child(info_col),
            )
            .when(self.modal_open, |d| {
                d.child(
                    div()
                        .id("headless-demo-modal-scrim")
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(modal_scrim)
                        .on_mouse_down(gpui::MouseButton::Left, move |_, w, cx| {
                            (close_modal_for_scrim)(w, cx);
                        })
                        .child(modal_root),
                )
            })
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
    text_input: impl IntoElement,
    icon_btn: IconButtonProps,
    icon_btn_clicks: u32,
    theme: &yororen_ui::theme::Theme,
) -> impl IntoElement {
    panel("hooks-col")
        .p(px(16.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(label_comp("1. Hooks (8 use_xxx)").strong(true))
                .child(label_comp("use_button: counter").muted(true))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(8.))
                        .child(dec.clone().apply(div().px(px(10.)).py(px(4.)).rounded(px(6.)).bg(theme.surface.hover)).child(label_comp("−").strong(true)))
                        .child(label_comp(format!("Count: {count}")).strong(true))
                        .child(inc.clone().apply(div().px(px(10.)).py(px(4.)).rounded(px(6.)).bg(theme.surface.hover)).child(label_comp("+").strong(true)))
                        .child(reset.clone().apply(div().px(px(10.)).py(px(4.)).rounded(px(6.))).child(label_comp("reset").muted(true))),
                )
                .child(label_comp("use_switch").muted(true))
                .child(div().flex().items_center().gap(px(8.)).child(switch.clone().apply(div().w(px(36.)).h(px(20.)).rounded(px(999.)).bg(if switch.checked { theme.action.primary.bg } else { theme.surface.sunken }))))
                .child(label_comp("use_checkbox").muted(true))
                .child(div().flex().items_center().gap(px(8.)).child(checkbox.clone().apply(div().w(px(18.)).h(px(18.)).rounded(px(4.)).bg(if checkbox.checked { theme.action.primary.bg } else { theme.surface.base }))))
                .child(label_comp("use_radio").muted(true))
                .child(
                    div()
                        .flex()
                        .gap(px(12.))
                        .child(radio_a.clone().apply(div().flex().items_center().gap(px(4.)).child(div().w(px(16.)).h(px(16.)).rounded(px(999.)).bg(if radio_a.checked { theme.action.primary.bg } else { theme.surface.base })).child(label_comp("Alpha"))))
                        .child(radio_b.clone().apply(div().flex().items_center().gap(px(4.)).child(div().w(px(16.)).h(px(16.)).rounded(px(999.)).bg(if radio_b.checked { theme.action.primary.bg } else { theme.surface.base })).child(label_comp("Beta")))),
                )
                .child(label_comp("use_toggle_button").muted(true))
                .child(toggle.clone().apply(div().px(px(10.)).py(px(4.)).rounded(px(6.)).bg(if toggle.selected { theme.action.primary.bg } else { theme.surface.hover })).child(label_comp(toggle.label.clone())))
                .child(label_comp("use_text_input").muted(true))
                .child(text_input)
                .child(label_comp("use_icon_button").muted(true))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        // `flex()` on the apply target is what
                        // makes `items_center` / `justify_center`
                        // take effect (a non-flex div ignores
                        // those). The hook's `apply()` only adds
                        // focus + click; layout is up to us.
                        .child(icon_btn.clone().apply(div().flex().w(px(32.)).h(px(32.)).rounded(px(999.)).bg(theme.surface.hover).items_center().justify_center()).child(label_comp("✕").strong(true)))
                        .child(label_comp(format!("clicks: {icon_btn_clicks}")).muted(true)),
                ),
        )
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
    let _ = (popover_btn, close_modal); // popover_btn is still passed for symmetry; close_modal is wired in the parent.

    // Popovers, dropdowns, selects, combo boxes, and tooltips all
    // use the component builders directly.

    let popover_root = popover("demo-popover")
        .open(popover_open)
        .trigger(
            popover_btn
                .clone()
                .apply(
                    div()
                        .flex()
                        .items_center()
                        .justify_center()
                        .px(px(10.))
                        .py(px(4.))
                        .rounded(px(6.))
                        .bg(theme.surface.hover),
                )
                .child(label_comp("Open popover").strong(true)),
        )
        .content(div().p(px(12.)).child(label_comp("Popover body")));

    let dropdown_root = yororen_ui::component::dropdown_menu("demo-dropdown")
        .label("Choose")
        .items([
            yororen_ui::component::DropdownItem::Item(
                yororen_ui::component::DropdownMenuItem::new("a", "Apple"),
            ),
            yororen_ui::component::DropdownItem::Item(
                yororen_ui::component::DropdownMenuItem::new("b", "Banana"),
            ),
            yororen_ui::component::DropdownItem::Item(
                yororen_ui::component::DropdownMenuItem::new("c", "Cherry"),
            ),
        ]);

    let select_root = select("demo-select")
        .options([
            SelectOption::new().value("red").label("Red"),
            SelectOption::new().value("green").label("Green"),
            SelectOption::new().value("blue").label("Blue"),
        ])
        .placeholder("Pick a color");

    let combo_root = combo_box("demo-combo")
        .options([
            ComboBoxOption::new("ny", "New York"),
            ComboBoxOption::new("sf", "San Francisco"),
            ComboBoxOption::new("tk", "Tokyo"),
        ])
        .placeholder("Pick a city");

    let tooltip_root = tooltip("This is a TooltipRoot popup")
        .placement(TooltipPlacement::Bottom);

    panel("composite-col")
        .p(px(16.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(label_comp("2. Composites (6 XxxRoot)").strong(true))
                .child(label_comp("PopoverRoot").muted(true))
                .child(popover_root)
                .child(label_comp("ModalRoot").muted(true))
                .child(
                    modal_btn
                        .clone()
                        .apply(
                            div()
                                .px(px(10.))
                                .py(px(4.))
                                .rounded(px(6.))
                                .bg(theme.surface.hover)
                                .items_center()
                                .justify_center(),
                        )
                        .child(label_comp("Open modal").strong(true)),
                )
                .child(label_comp(if modal_open { "(modal overlay is open)" } else { "(modal closed)" }).muted(true))
                .child(label_comp("DropdownMenuRoot").muted(true))
                .child(dropdown_root)
                .child(label_comp("SelectRoot").muted(true))
                .child(select_root)
                .child(label_comp("ComboBoxRoot").muted(true))
                .child(combo_root)
                .child(label_comp("TooltipRoot (hover the button)").muted(true))
                .child(tooltip_root)
                .child(label_comp(format!("radio_choice = {radio_choice}")).muted(true)),
        )
}

fn render_info_col(theme: &yororen_ui::theme::Theme) -> impl IntoElement {
    panel("info-col")
        .p(px(16.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(label_comp("3. About this demo").strong(true))
                .child(label_comp("8 use_xxx hooks").strong(true))
                .child(label_comp("6 XxxRoot composite APIs").strong(true))
                .child(label_comp("Switch themes via the meta-crate's feature flags:").muted(true))
                .child(label_comp("  yororen-ui = { version = \"0.3\", features = [\"catppuccin\"] }").muted(true))
                .child(label_comp("  yororen-ui = { version = \"0.3\", features = [\"material\"] }").muted(true))
                .child(label_comp("See HOOK_GUIDE.md for the full API walkthrough.").muted(true))
                .child(label_comp("Active theme:").muted(true))
                .child(label_comp(format!("surface.base: {:?}", theme.surface.base)))
                .child(label_comp(format!("action.primary: {:?}", theme.action.primary.bg))),
        )
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
