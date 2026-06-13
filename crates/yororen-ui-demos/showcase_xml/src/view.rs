//! The `ShowcaseApp` view. The UI itself is described
//! in `src/ui/showcase.xml`; this file just owns the
//! state, wires it into the XML, and exposes
//! `Render::render`.

use gpui::{AppContext, Context, IntoElement, Render, Window};

use crate::state::ShowcaseState;
use yororen_ui::xml_file;

pub struct ShowcaseApp;

impl Render for ShowcaseApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<ShowcaseState>();
        let counter = state.counter.read(cx).value;
        // `state.flag.read(cx)` returns `&bool`; the XML
        // uses `flag` in conditions where it expects a
        // value (not a reference), so deref.
        let flag = *state.flag.read(cx);
        let name = state.name.read(cx).clone();
        let todos = state.todos.clone();
        let connection = *state.connection.read(cx);
        let inc = state.counter.clone();
        let dec = state.counter.clone();
        let reset = state.counter.clone();
        let flag_entity = state.flag.clone();
        let name_entity = state.name.clone();
        let connect_entity = state.connection.clone();

        xml_file!(cx = &mut **cx, "ui/showcase.xml")
    }
}
