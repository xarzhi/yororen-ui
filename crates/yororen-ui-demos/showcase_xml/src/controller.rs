//! Application controller — all business logic.
//!
//! The XML in `src/ui/showcase.xml` is purely
//! declarative: every `on_click={...}` references a
//! method on [`Controller`]. The codegen auto-wraps
//! bare path expressions into the standard 3-arg
//! event callback shape, so the XML stays free of
//! inline closures and `update(cx, …)` boilerplate.
//!
//! [`Controller`] is `Clone` because the `xml!` macro
//! captures it into multiple closures (one per event
//! handler). Each clone is cheap — the only field is
//! an `Entity<ShowcaseState>`, which itself is an
//! `Arc` underneath.

use gpui::{App, ClickEvent, Entity, Window};

use crate::state::{ConnectionStatus, ShowcaseState};

/// All user-driven actions in the showcase. Each method
/// has the standard 3-arg event signature
/// `(arg0, &mut Window, &mut App)` so the codegen's
/// auto-wrap can call it without an explicit closure
/// in the XML.
#[derive(Clone)]
pub struct Controller {
    state: Entity<ShowcaseState>,
}

impl Controller {
    pub fn new(state: Entity<ShowcaseState>) -> Self {
        Self { state }
    }

    /// The underlying state entity. Used by the view to
    /// subscribe to change notifications — every time
    /// any field of `ShowcaseState` mutates, the view
    /// re-renders.
    pub fn state(&self) -> Entity<ShowcaseState> {
        self.state.clone()
    }

    // -- counter ------------------------------------------------------------

    pub fn increment(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.counter.update(cx, |c, cx| {
                c.value += 1;
                cx.notify();
            });
            cx.notify();
        });
    }

    pub fn decrement(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.counter.update(cx, |c, cx| {
                c.value -= 1;
                cx.notify();
            });
            cx.notify();
        });
    }

    pub fn reset_counter(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.counter.update(cx, |c, cx| {
                c.value = 0;
                cx.notify();
            });
            cx.notify();
        });
    }

    // -- two-way bindings (each toggleable has its own entity) --------------

    // -- name (string binding) + Clear --------------------------------------

    /// Clear the name input. The TextInput's internal
    /// state is renderer-private, so we can't reach
    /// in to wipe its buffer. Instead we bump
    /// `name_input_key`; the view reads it and uses
    /// it to build a fresh `id` for the input, which
    /// forces the renderer to mint a new
    /// `TextInputState` (keyed by `id`). The name
    /// entity is also cleared in case other UI
    /// elements read it.
    pub fn clear_name(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.name_input_key.update(cx, |k, cx| {
                k.value += 1;
                cx.notify();
            });
            s.name.update(cx, |n, cx| {
                n.clear();
                cx.notify();
            });
            cx.notify();
        });
    }

    // -- connection (status enum) ------------------------------------------

    pub fn advance_connection(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, cx| {
            s.connection.update(cx, |c, cx| {
                *c = match *c {
                    ConnectionStatus::Disconnected => ConnectionStatus::Connecting,
                    ConnectionStatus::Connecting => ConnectionStatus::Connected,
                    ConnectionStatus::Connected => ConnectionStatus::Failed,
                    ConnectionStatus::Failed => ConnectionStatus::Disconnected,
                };
                cx.notify();
            });
            cx.notify();
        });
    }
}
