//! The `GalleryApp` view. The actual UI lives in
//! `src/ui/gallery.xml`; this file just spreads the state
//! and controller into the macro's scope.

use gpui::{Context, IntoElement, Render, Window};

use crate::controller::Controller;
use crate::state::{StateRef, TodoItem};
use yororen_ui::xml_file;

pub struct GalleryApp {
    controller: Controller,
}

impl GalleryApp {
    pub fn new(cx: &mut Context<Self>, controller: Controller) -> Self {
        // Notify the view whenever the state changes —
        // the XML reads values directly, so we need a
        // full re-render on update.
        cx.observe(&controller.state(), |_, _, cx| cx.notify())
            .detach();
        Self { controller }
    }
}

impl Render for GalleryApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Read the state once and project the values /
        // entity handles the XML needs into scope. The
        // XML is purely declarative — every identifier
        // it references is bound here.
        let state = cx.global::<StateRef>().state.read(cx);

        // ---- Toolbar live values ----
        let toast_count = state.toast_count.read(cx).value;

        // ---- Section live values ----
        let toggle_btn = *state.toggle_btn_selected.read(cx);
        let last_action = state.last_action_label.read(cx).clone();
        let progress = *state.progress_value.read(cx);
        let tag_close_count = state.tag_closable_count.read(cx).value;

        // ---- Inputs ----
        let text_value = state.text_value.read(cx).clone();
        let _search_value = state.search_value.read(cx).clone();
        let number_value = *state.number_value.read(cx);

        // ---- Controls ----
        let checkbox = *state.checkbox_value.read(cx);
        let switch = *state.switch_value.read(cx);
        let radio = *state.radio_value.read(cx);
        let slider = *state.slider_value.read(cx);

        // ---- Lists ----
        let todos: Vec<TodoItem> = state.todos.clone();

        // ---- Routing ----
        let section = *state.section.read(cx);

        // ---- Drop controller and entity handles the XML uses ----
        let controller = self.controller.clone();
        let _toast_count_entity = state.toast_count.clone();
        let toggle_btn_entity = state.toggle_btn_selected.clone();
        let _progress_entity = state.progress_value.clone();
        let text_entity = state.text_value.clone();
        let search_entity = state.search_value.clone();
        let number_entity = state.number_value.clone();
        let checkbox_entity = state.checkbox_value.clone();
        let switch_entity = state.switch_value.clone();
        let _radio_entity = state.radio_value.clone();
        let slider_entity = state.slider_value.clone();
        let _section_entity = state.section.clone();

        xml_file!(cx = &mut **cx, window = window, "ui/gallery.xml")
    }
}
