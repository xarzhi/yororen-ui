//! yororen-ui Counter Component
//!
//! Demonstrates the v0.3 split using the **default-render
//! sugar** path: each `headless::button` is built with
//! `DefaultButton::default_render(cx)` which returns a
//! pre-styled `Stateful<Div>`. The renderer paints the
//! container (background, padding, radius, min-height, etc.)
//! from the registered `ButtonRenderer`; the caller chains
//! `.child("...")` on the result to add the button's text
//! content.
//!
//! For full visual control (custom `div()` composition), the
//! v0.3 API is:
//!
//! ```ignore
//! button("id", cx).on_click(...).apply(div().bg(red).child("Save"))
//! ```
//!
//! — the headless path lets the caller own the entire div,
//! including content.
//!
//! ## `&mut App` from `&mut Context<T>`
//!
//! Headless factories like `button(id, cx)` need `&mut App`
//! to mint a fresh `FocusHandle`. Inside a `Render::render`
//! closure the caller only has `&mut Context<Self>`. The
//! conversion is sound: `Context<T>: DerefMut<Target = App>`,
//! so `&mut **cx` is a `&mut App`. We use it inline at each
//! call site (as a temporary borrow) so it does not collide
//! with later uses of `cx` (e.g. `default_render(cx)`).

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div};
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;
use yororen_ui::renderer::DefaultButton;
use yororen_ui::renderer::DefaultLabel;

use crate::state::CounterState;

pub struct CounterApp;

impl Render for CounterApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<CounterState>();
        let count = state.counter.read(cx).value;
        // Three separate clones of the same `Entity<CounterValue>`
        // handle — each `move` closure owns its own. The names are
        // a hint at which button uses which, not a sign of three
        // different entities.
        let inc_entity = state.counter.clone();
        let dec_entity = state.counter.clone();
        let reset_entity = state.counter.clone();

        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_3()
            .p_4()
            .child(
                label("subtitle", "Counter Demo", &mut **cx)
                    .strong(true)
                    .default_render(cx)
                    .text_size(gpui::px(28.))
                    .into_any_element(),
            )
            .child(
                label("count", count.to_string(), &mut **cx)
                    .default_render(cx)
                    .text_size(gpui::px(20.))
                    .into_any_element(),
            )
            .child(
                div()
                    .flex()
                    .gap_2()
                    .child(
                        button("decrease", &mut **cx)
                            .on_click(move |_, _, cx| {
                                dec_entity.update(cx, |c, cx| {
                                    c.value -= 1;
                                    cx.notify();
                                });
                            })
                            .default_render(cx)
                            .child("-"),
                    )
                    .child(
                        button("reset", &mut **cx)
                            .on_click(move |_, _, cx| {
                                reset_entity.update(cx, |c, cx| {
                                    c.value = 0;
                                    cx.notify();
                                });
                            })
                            .default_render(cx)
                            .child("Reset"),
                    )
                    .child(
                        button("increase", &mut **cx)
                            .on_click(move |_, _, cx| {
                                inc_entity.update(cx, |c, cx| {
                                    c.value += 1;
                                    cx.notify();
                                });
                            })
                            .default_render(cx)
                            .child("+"),
                    ),
            )
    }
}
