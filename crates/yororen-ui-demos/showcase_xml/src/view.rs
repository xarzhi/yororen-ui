//! The `ShowcaseApp` view. The UI itself is described
//! in `src/ui/showcase.xml`; this file just spreads the
//! state into the macro's scope and exposes
//! `Render::render`.
//!
//! All event handlers live in [`crate::controller`];
//! the XML references them by method name.

use gpui::{AppContext, Context, IntoElement, Render, Window};

use crate::controller::Controller;
use crate::state::StateRef;
use yororen_ui::xml_file;

pub struct ShowcaseApp {
    controller: Controller,
}

impl ShowcaseApp {
    pub fn new(cx: &mut Context<Self>, controller: Controller) -> Self {
        // Notify the view whenever the state changes —
        // the XML reads values directly, so we need a
        // full re-render on update.
        cx.observe(&controller.state(), |_, _, cx| cx.notify())
            .detach();
        Self { controller }
    }
}

impl Render for ShowcaseApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Read the state once and project the values /
        // entity handles the XML needs into scope. The
        // XML is purely declarative — every identifier
        // it references is bound here.
        let state = cx.global::<StateRef>().state.read(cx);
        let counter = state.counter.read(cx).value;
        let flag = *state.flag.read(cx);
        let _name = state.name.read(cx).clone();
        let todos = state.todos.clone();
        let connection = *state.connection.read(cx);
        // Entity handles for `@bind` and the controller
        // for `on_click={...}` references.
        let flag_entity = state.flag.clone();
        let name_entity = state.name.clone();
        let controller = self.controller.clone();

        xml_file!(cx = &mut **cx, "ui/showcase.xml")
    }
}