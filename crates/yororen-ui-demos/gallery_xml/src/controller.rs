//! Controller — every business-logic method the XML
//! references. The XML itself stays purely declarative:
//! every `on_click={controller.foo}` resolves to a method
//! here via the macro's auto-wrap.
//!
//! `Controller` is `Clone` because the macro pre-clones the
//! receiver into a hygienic local per event handler.

#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap};
use std::ops::Range;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, Entity, IntoElement, ParentElement, SharedString,
    Styled, Window, div, hsla, px,
};

use yororen_ui::headless::dropdown_menu::{DropdownItem, DropdownMenuItem};
use yororen_ui::headless::table::TableColumn;
use yororen_ui::headless::tree::TreeData;
use yororen_ui::headless::tree_item::TreeNodeId;
use yororen_ui::notification::center::{Notification, NotificationCenter, ToastKind};

use crate::state::{DarkMode, GalleryState, LocaleChoice, RendererKind};
use yororen_ui::i18n::Translate;

#[derive(Clone)]
pub struct Controller {
    state: Entity<GalleryState>,
}

// Thin 4-arg event adapters for the toolbar toggle buttons so the
// XML can use bare method references instead of inline closures.
macro_rules! toggle_selectors {
    ($($name:ident => $method:ident($value:expr)),* $(,)?) => {
        $(
            pub fn $name(
                &self,
                _checked: bool,
                _ev: Option<&ClickEvent>,
                _w: &mut Window,
                cx: &mut App,
            ) {
                self.$method($value, _w, cx);
            }
        )*
    };
}

// Thin 4-arg event adapters so the XML can use bare method
// references (`on_toggle={controller.select_radio_0}`) instead
// of inline Rust closures.
macro_rules! radio_selectors {
    ($($name:ident => $value:expr),* $(,)?) => {
        $(
            pub fn $name(
                &self,
                _checked: bool,
                _ev: Option<&ClickEvent>,
                _w: &mut Window,
                cx: &mut App,
            ) {
                self.set_radio($value, _w, cx);
            }
        )*
    };
}

impl Controller {
    pub fn new(state: Entity<GalleryState>, cx: &mut App) -> Self {
        Self::wire_composite_state(&state, cx);
        Self { state }
    }

    pub fn state(&self) -> Entity<GalleryState> {
        self.state.clone()
    }

    // -------- Toolbar read-only accessors used by view.rs --------

    pub fn current_renderer(&self, cx: &App) -> RendererKind {
        self.state.read(cx).current_renderer
    }

    pub fn dark_mode(&self, cx: &App) -> DarkMode {
        self.state.read(cx).dark_mode
    }

    pub fn modal_state(&self, cx: &App) -> Entity<yororen_ui::headless::modal::ModalState> {
        self.state.read(cx).modal_state.clone()
    }

    // -------- Read-only helpers used directly by XML --------

    pub fn toast_count(&self, cx: &App) -> usize {
        self.state.read(cx).toast_count.value
    }

    pub fn tag_close_count(&self, cx: &App) -> usize {
        self.state.read(cx).tag_closable_count.value
    }

    pub fn tag_selected(&self, cx: &App) -> bool {
        self.state.read(cx).tag_selected
    }

    pub fn progress(&self, cx: &App) -> f32 {
        self.state.read(cx).progress_value
    }

    pub fn toggle_btn_selected(&self, cx: &App) -> bool {
        self.state.read(cx).toggle_btn_selected
    }

    pub fn is_modal_open(&self, cx: &App) -> bool {
        self.state.read(cx).modal_state.read(cx).open
    }

    pub fn popover_visible(&self, cx: &App) -> bool {
        self.state.read(cx).popover_state.read(cx).is_visible()
    }

    pub fn dropdown_visible(&self, cx: &App) -> bool {
        self.state.read(cx).dropdown_state.read(cx).is_visible()
    }

    pub fn disclosure_open(&self, cx: &App) -> bool {
        self.state.read(cx).disclosure_open
    }

    pub fn dropdown_demo_value(&self, cx: &App) -> String {
        self.state.read(cx).dropdown_demo_value.clone()
    }

    pub fn menu_demo_value(&self, cx: &App) -> String {
        self.state.read(cx).menu_demo_value.clone()
    }

    pub fn listbox_demo_value(&self, cx: &App) -> String {
        self.state.read(cx).listbox_demo_value.clone()
    }

    pub fn select_demo_value(&self, cx: &App) -> String {
        self.state.read(cx).select_demo_value.clone()
    }

    pub fn combo_demo_value(&self, cx: &App) -> String {
        self.state.read(cx).combo_demo_value.clone()
    }

    pub fn form_email_error(&self, cx: &App) -> Option<String> {
        self.state.read(cx).form_email_error.clone()
    }

    pub fn form_email_value(&self, cx: &App) -> String {
        self.state.read(cx).form_email_value.read(cx).clone()
    }

    pub fn form_email_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).form_email_value.clone()
    }

    pub fn form_submit_count(&self, cx: &App) -> usize {
        self.state.read(cx).form_submit_count
    }

    pub fn selected_table_row(&self, cx: &App) -> Option<usize> {
        self.state.read(cx).selected_table_row
    }

    pub fn selected_list_item(&self, cx: &App) -> Option<usize> {
        self.state.read(cx).selected_list_item
    }

    pub fn vl_status(&self, cx: &App) -> (String, usize, usize) {
        let s = self.state.read(cx);
        let visible = format!("{:?}", s.vl_visible_range);
        (visible, s.vl_item_count, s.vl_load_count)
    }

    pub fn tree_expanded(&self, cx: &App) -> BTreeSet<TreeNodeId> {
        self.state.read(cx).tree_expanded.clone()
    }

    pub fn tree_selected(&self, cx: &App) -> Option<TreeNodeId> {
        self.state.read(cx).tree_selected.clone()
    }

    pub fn radio_value(&self, cx: &App) -> usize {
        self.state.read(cx).radio_value
    }

    pub fn checkbox_value(&self, cx: &App) -> bool {
        *self.state.read(cx).checkbox_value.read(cx)
    }

    pub fn checkbox_value_entity(&self, cx: &App) -> Entity<bool> {
        self.state.read(cx).checkbox_value.clone()
    }

    pub fn switch_value(&self, cx: &App) -> bool {
        *self.state.read(cx).switch_value.read(cx)
    }

    pub fn switch_value_entity(&self, cx: &App) -> Entity<bool> {
        self.state.read(cx).switch_value.clone()
    }

    pub fn slider_value(&self, cx: &App) -> f32 {
        *self.state.read(cx).slider_value.read(cx)
    }

    pub fn slider_value_entity(&self, cx: &App) -> Entity<f32> {
        self.state.read(cx).slider_value.clone()
    }

    pub fn text_value(&self, cx: &App) -> String {
        self.state.read(cx).text_value.read(cx).clone()
    }

    pub fn text_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).text_value.clone()
    }

    pub fn search_value(&self, cx: &App) -> String {
        self.state.read(cx).search_value.read(cx).clone()
    }

    pub fn search_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).search_value.clone()
    }

    pub fn number_value(&self, cx: &App) -> f64 {
        *self.state.read(cx).number_value.read(cx)
    }

    pub fn number_value_entity(&self, cx: &App) -> Entity<f64> {
        self.state.read(cx).number_value.clone()
    }

    pub fn keybinding_value(&self, cx: &App) -> String {
        self.state.read(cx).keybinding_value.clone()
    }

    pub fn keybinding_mode(
        &self,
        cx: &App,
    ) -> yororen_ui::headless::keybinding_input::KeybindingInputMode {
        self.state.read(cx).keybinding_mode
    }

    pub fn text_area_value(&self, cx: &App) -> String {
        self.state.read(cx).text_area_value.read(cx).clone()
    }

    pub fn text_area_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).text_area_value.clone()
    }

    pub fn password_value(&self, cx: &App) -> String {
        self.state.read(cx).password_value.read(cx).clone()
    }

    pub fn password_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).password_value.clone()
    }

    pub fn file_path_value(&self, cx: &App) -> String {
        self.state.read(cx).file_path_value.read(cx).clone()
    }

    pub fn file_path_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).file_path_value.clone()
    }

    pub fn current_locale(&self, cx: &App) -> LocaleChoice {
        self.state.read(cx).current_locale
    }

    /// Reusable demo "cell" wrapper: a small muted label above the
    /// component inside a 1px-bordered rounded box. Mirrors
    /// `gallery_demo::sections::cell`.
    fn cell(&self, label: impl Into<String>, element: AnyElement, cx: &mut App) -> AnyElement {
        let label_el = yororen_ui::headless::label::label("cmp-name", label, cx)
            .muted(true)
            .render(cx)
            .text_size(px(11.));
        div()
            .relative()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(2.))
            .p(px(8.))
            .rounded(px(6.))
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.5, 0.15))
            .child(label_el)
            .child(element)
            .into_any_element()
    }

    // -------- Composite state accessors --------

    pub fn select_state(&self, cx: &App) -> Entity<yororen_ui::headless::select::SelectState> {
        self.state.read(cx).select_state.clone()
    }

    pub fn combo_state(&self, cx: &App) -> Entity<yororen_ui::headless::combo_box::ComboBoxState> {
        self.state.read(cx).combo_state.clone()
    }

    pub fn listbox_state(&self, cx: &App) -> Entity<yororen_ui::headless::listbox::ListboxState> {
        self.state.read(cx).listbox_state.clone()
    }

    pub fn menu_state(&self, cx: &App) -> Entity<yororen_ui::headless::menu::MenuState> {
        self.state.read(cx).menu_state.clone()
    }

    pub fn dropdown_state(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::dropdown_menu::DropdownMenuState> {
        self.state.read(cx).dropdown_state.clone()
    }

    pub fn split_button_state(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::dropdown_menu::DropdownMenuState> {
        self.state.read(cx).split_dropdown_state.clone()
    }

    pub fn popover_state(&self, cx: &App) -> Entity<yororen_ui::headless::popover::PopoverState> {
        self.state.read(cx).popover_state.clone()
    }

    pub fn tooltip_state(&self, cx: &App) -> Entity<yororen_ui::headless::tooltip::TooltipState> {
        self.state.read(cx).tooltip_state.clone()
    }

    pub fn list_controller(
        &self,
        cx: &App,
    ) -> yororen_ui::headless::virtual_list::VirtualListController {
        self.state.read(cx).list_controller.clone()
    }

    pub fn uniform_list_controller(
        &self,
        cx: &App,
    ) -> yororen_ui::headless::virtual_list::UniformVirtualListController {
        self.state.read(cx).uniform_list_controller.clone()
    }

    // -------- Toolbar actions --------

    pub fn set_renderer(&self, kind: RendererKind, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| s.current_renderer = kind);
    }

    pub fn set_dark_mode(&self, dark: DarkMode, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| s.dark_mode = dark);
    }

    pub fn set_locale(&self, choice: LocaleChoice, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| s.current_locale = choice);
        crate::i18n::install_for_locale(cx, choice);
    }

    toggle_selectors! {
        select_default_renderer => set_renderer(RendererKind::Default),
        select_brutalism_renderer => set_renderer(RendererKind::Brutalism),
        select_light_theme => set_dark_mode(DarkMode::Light),
        select_dark_theme => set_dark_mode(DarkMode::Dark),
        select_english_locale => set_locale(LocaleChoice::En),
        select_chinese_locale => set_locale(LocaleChoice::ZhCn),
        select_arabic_locale => set_locale(LocaleChoice::Ar),
    }

    pub fn show_toast(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        let id = self.state.update(cx, |s, _cx| {
            s.toast_count.value += 1;
            s.toast_count.value
        });
        let id_str = id.to_string();
        let center = cx.global::<NotificationCenter>().clone();
        center.notify(
            Notification::new(cx.t_with_args("demo.toast_message", &[&id_str]))
                .title(cx.t("demo.toast_title"))
                .kind(ToastKind::Info),
            cx,
        );
    }

    pub fn show_notification(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        let id = self.state.update(cx, |s, _cx| {
            s.toast_count.value += 1;
            s.toast_count.value
        });
        let id_str = id.to_string();
        let center = cx.global::<NotificationCenter>().clone();
        center.notify(
            Notification::new(cx.t_with_args("demo.notification_message", &[&id_str]))
                .title(cx.t("demo.notification_title"))
                .kind(ToastKind::Success)
                .sticky(true),
            cx,
        );
    }

    // -------- Actions section --------

    pub fn split_button_items(&self, _cx: &App) -> Vec<DropdownItem> {
        vec![
            DropdownItem::Item(DropdownMenuItem::new(
                "save",
                _cx.t("demo.actions.save"),
            )),
            DropdownItem::Item(DropdownMenuItem::new(
                "save_as",
                _cx.t("demo.actions.save_as"),
            )),
            DropdownItem::Item(DropdownMenuItem::new(
                "save_all",
                _cx.t("demo.actions.save_all"),
            )),
        ]
    }

    pub fn split_button_primary(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.toast_count.value += 1;
        });
    }

    pub fn split_button_select(&self, _id: gpui::SharedString, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.toast_count.value += 1;
        });
    }

    pub fn press_toggle(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.toggle_btn_selected = !s.toggle_btn_selected;
        });
    }

    pub fn noop_click(&self, _ev: &ClickEvent, _w: &mut Window, _cx: &mut App) {}

    pub fn press_toggle_from_bool(&self, value: bool, _cx: &mut App) {
        self.state.update(_cx, |s, _cx2| {
            s.toggle_btn_selected = value;
        });
    }

    /// Adapter for `<ToggleButton on_toggle={controller.press_toggle_from_event}>`.
    /// The XML macro auto-wraps this 4-arg event signature into the
    /// 2-arg business-logic method.
    pub fn press_toggle_from_event(
        &self,
        selected: bool,
        _ev: Option<&ClickEvent>,
        _w: &mut Window,
        cx: &mut App,
    ) {
        self.press_toggle_from_bool(selected, cx);
    }

    pub fn bump_progress(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.progress_value = (s.progress_value + 0.1).min(1.0);
        });
    }

    pub fn reset_progress(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.progress_value = 0.0;
        });
    }

    pub fn close_tag(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.tag_closable_count.value += 1;
        });
    }

    pub fn toggle_tag_selected(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.tag_selected = !s.tag_selected;
        });
    }

    // -------- Controls section --------

    pub fn set_radio(&self, value: usize, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.radio_value = value;
        });
    }

    radio_selectors! {
        select_radio_0 => 0,
        select_radio_1 => 1,
        select_radio_2 => 2,
    }

    pub fn set_checkbox(
        &self,
        value: bool,
        _ev: Option<&ClickEvent>,
        _w: &mut Window,
        cx: &mut App,
    ) {
        let entity = self.state.read(cx).checkbox_value.clone();
        entity.update(cx, |s, _cx| *s = value);
    }

    pub fn set_switch(&self, value: bool, _ev: Option<&ClickEvent>, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).switch_value.clone();
        entity.update(cx, |s, _cx| *s = value);
    }

    pub fn set_slider(&self, value: f32, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).slider_value.clone();
        entity.update(cx, |s, _cx| *s = value);
    }

    // -------- Inputs section --------

    pub fn set_text_value(&self, value: &str, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).text_value.clone();
        entity.update(cx, |s, _cx| *s = value.to_string());
    }

    pub fn set_password_value(&self, value: &str, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).password_value.clone();
        entity.update(cx, |s, _cx| *s = value.to_string());
    }

    pub fn set_search_value(&self, value: &str, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).search_value.clone();
        entity.update(cx, |s, _cx| *s = value.to_string());
    }

    pub fn clear_search(&self, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).search_value.clone();
        entity.update(cx, |s, _cx| s.clear());
    }

    pub fn set_number_value(&self, value: f64, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).number_value.clone();
        entity.update(cx, |s, _cx| *s = value);
    }

    pub fn set_text_area_value(&self, value: &str, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).text_area_value.clone();
        entity.update(cx, |s, _cx| *s = value.to_string());
    }

    pub fn set_keybinding_value(&self, value: &str, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.keybinding_value = value.to_string();
        });
    }

    pub fn set_file_path_value(&self, value: &str, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).file_path_value.clone();
        entity.update(cx, |s, _cx| *s = value.to_string());
    }

    pub fn browse_file_path(&self, _value: &str, _w: &mut Window, cx: &mut App) {
        let entity = self.state.read(cx).file_path_value.clone();
        entity.update(cx, |s, _cx| *s = "/tmp/example.txt".to_string());
    }

    pub fn start_keybinding_capture(&self, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.keybinding_mode =
                yororen_ui::headless::keybinding_input::KeybindingInputMode::Capturing;
        });
    }

    pub fn cancel_keybinding_capture(&self, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.keybinding_mode = yororen_ui::headless::keybinding_input::KeybindingInputMode::Idle;
        });
    }

    // -------- Overlays section --------

    pub fn open_modal(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, ctx| {
            s.modal_state.update(ctx, |st, _| st.open());
        });
    }

    pub fn close_modal(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, ctx| {
            s.modal_state.update(ctx, |st, _| st.close());
        });
    }

    pub fn toggle_popover(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, ctx| {
            s.popover_state.update(ctx, |st, _| st.toggle());
        });
    }

    pub fn toggle_dropdown(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, ctx| {
            s.dropdown_state.update(ctx, |st, _| st.toggle());
        });
    }

    pub fn toggle_disclosure(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.disclosure_open = !s.disclosure_open;
        });
    }

    pub fn disclosure_body(&self, cx: &mut App) -> AnyElement {
        if !self.disclosure_open(cx) {
            return div().into_any_element();
        }
        div()
            .flex()
            .flex_col()
            .pl(px(16.0))
            .child(
                yororen_ui::headless::label::label(
                    "ov-disc-body",
                    cx.t("demo.disclosure.body"),
                    cx,
                )
                .render(cx),
            )
            .into_any_element()
    }

    // -------- Lists section --------

    pub fn submit_form(&self, vals: HashMap<SharedString, String>, _w: &mut Window, cx: &mut App) {
        let must_contain = cx.t("demo.form.must_contain_at");
        self.state.update(cx, |s, _cx| {
            s.form_submit_count += 1;
            if let Some(e) = vals.get("email") {
                s.form_email_error = if e.contains('@') {
                    None
                } else {
                    Some(must_contain.to_string())
                };
            }
        });
    }

    pub fn set_table_row(&self, value: usize, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.selected_table_row = Some(value);
        });
    }

    pub fn table_row_handler(
        &self,
    ) -> impl Fn(usize, &mut gpui::Window, &mut gpui::App) + Clone + 'static {
        let state = self.state.clone();
        move |value, _w, cx| {
            state.update(cx, |s, _cx| {
                s.selected_table_row = Some(value);
            });
        }
    }

    pub fn toggle_tree_node(
        &self,
        id: TreeNodeId,
        _ev: &ClickEvent,
        _w: &mut Window,
        cx: &mut App,
    ) {
        self.state.update(cx, |s, _cx| {
            if !s.tree_expanded.remove(&id) {
                s.tree_expanded.insert(id);
            }
        });
    }

    pub fn select_tree_node(
        &self,
        id: TreeNodeId,
        _ev: &ClickEvent,
        _w: &mut Window,
        cx: &mut App,
    ) {
        self.state.update(cx, |s, _cx| {
            s.tree_selected = Some(id);
        });
    }

    pub fn select_list_item(&self, value: usize, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| {
            s.selected_list_item = Some(value);
        });
    }

    pub fn select_list_item_handler(
        &self,
        index: usize,
    ) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, _cx| {
                s.selected_list_item = Some(index);
            });
        }
    }

    pub fn virtual_list_scroll_top(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state
            .update(cx, |s, _| s.list_controller.scroll_to_top());
    }

    pub fn virtual_list_scroll_bottom(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state
            .update(cx, |s, _| s.list_controller.scroll_to_bottom());
    }

    pub fn uniform_list_scroll_top(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state
            .update(cx, |s, _| s.uniform_list_controller.scroll_to_top());
    }

    pub fn uniform_list_scroll_bottom(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.state
            .update(cx, |s, _| s.uniform_list_controller.scroll_to_bottom());
    }

    pub fn on_visible_range_change(
        &self,
        range: Range<usize>,
        total: usize,
        _w: &mut Window,
        cx: &mut App,
    ) {
        self.state.update(cx, |s, _cx| {
            s.vl_visible_range = Some(range);
            if total + 50 >= s.vl_item_count && s.vl_item_count < 5_000 {
                s.vl_item_count += 100;
                s.vl_load_count += 1;
            }
        });
    }

    pub fn virtual_list_row(&self, ix: usize, _w: &mut Window, cx: &mut App) -> gpui::AnyElement {
        let selected = self.state.read(cx).selected_list_item == Some(ix);
        let label = format!("Item {}", ix);
        yororen_ui::headless::list_item::list_item(format!("vl-row-{ix}"), label, cx)
            .selected(selected)
            .on_click({
                let state = self.state.clone();
                let ix = ix;
                move |_ev, _w, cx| {
                    state.update(cx, |s, _cx| {
                        s.selected_list_item = Some(ix);
                    });
                }
            })
            .render(cx)
            .into_any_element()
    }

    pub fn uniform_list_row(&self, ix: usize, _w: &mut Window, cx: &mut App) -> gpui::AnyElement {
        let label = format!("Uniform {}", ix);
        yororen_ui::headless::list_item::list_item(format!("uvl-row-{ix}"), label, cx)
            .render(cx)
            .into_any_element()
    }

    // -------- Complex layout helpers (used by lists.xml) --------

    pub fn sync_virtual_list(&self, cx: &mut App) {
        self.state.update(cx, |s, _| {
            let current = s.list_controller.state().item_count();
            if current < s.vl_item_count {
                s.list_controller.append(s.vl_item_count - current);
            } else if current > s.vl_item_count {
                s.list_controller.reset(s.vl_item_count);
            }
        });
    }

    pub fn listbox_status_text(&self, cx: &App) -> String {
        let value = self.listbox_demo_value(cx);
        let status = if value.is_empty() {
            "—".to_string()
        } else {
            value
        };
        cx.t("demo.lists.listbox_selected")
            .replacen("{}", &status, 1)
    }

    pub fn email_input_element(&self, cx: &mut App, window: &mut Window) -> AnyElement {
        let entity = self.state.read(cx).form_email_value.clone();
        let value = entity.read(cx).clone();
        yororen_ui::headless::text_input::text_input("lists-ff-email-input")
            .placeholder(cx.t("demo.form.email_placeholder"))
            .value(value)
            .on_change(move |new: &str, _w, cx| {
                entity.update(cx, |s, _cx| *s = new.to_string());
            })
            .render(cx, window)
            .into_any_element()
    }

    pub fn form_element(&self, cx: &mut App, window: &mut Window) -> AnyElement {
        let email_input = self.email_input_element(cx, window);
        let form_field =
            yororen_ui::headless::form_field::form_field("lists-ff-email", "email", cx)
                .label(cx.t("demo.form.email_label"))
                .required(true)
                .input(email_input)
                .render(cx);

        let form_props = yororen_ui::headless::form::form("lists-form", cx)
            .value(
                "email",
                self.state.read(cx).form_email_value.read(cx).clone(),
            )
            .on_submit({
                let controller = self.clone();
                move |vals, _w, cx| controller.submit_form(vals, _w, cx)
            })
            .submit(cx.t("demo.form.email_label"));

        let submit_btn = form_props.submit_button(cx).expect("submit_label set");
        let submit_count = self.state.read(cx).form_submit_count;
        let email_error = self.state.read(cx).form_email_error.clone();

        let el = form_props
            .render(cx)
            .w(px(300.))
            .child(form_field)
            .child(submit_btn)
            .child(
                yororen_ui::headless::label::label(
                    "form-submit-count",
                    format!(
                        "{} {} | {} {:?}",
                        cx.t("demo.form.submitted"),
                        submit_count,
                        cx.t("demo.form.last_error"),
                        email_error
                    ),
                    cx,
                )
                .muted(true)
                .render(cx),
            )
            .into_any_element();
        self.cell(cx.t("demo.form.cell"), el, cx)
    }

    pub fn tree_element(&self, cx: &mut App, window: &mut Window) -> AnyElement {
        let state = self.state.clone();
        let data = self.tree_data(cx);
        let expanded = self.tree_expanded(cx);
        let selected = self.tree_selected(cx);
        let visible = data.flatten(&expanded);

        let mut tree = yororen_ui::headless::tree::tree("lists-tree", cx)
            .data(data.clone())
            .render(cx)
            .w(px(240.));

        for (id, depth) in visible {
            let has_children = !data.children_of(&id).is_empty();
            let label = data.label_of(&id).unwrap_or("").to_string();
            let is_expanded = expanded.contains(&id);
            let is_selected = selected.as_ref() == Some(&id);
            let row_id: ElementId = format!("lists-tree-row-{}", id.0).into();

            let state_toggle = state.clone();
            let state_select = state.clone();
            let state_double = state.clone();
            let toggle_id = id.clone();
            let select_id = id.clone();
            let double_id = id.clone();

            tree = tree.child(
                yororen_ui::headless::tree_item::tree_item(row_id, id, label, cx)
                    .depth(depth)
                    .has_children(has_children)
                    .expanded(is_expanded)
                    .selected(is_selected)
                    .on_toggle(move |_ev, _w, cx| {
                        let tid = toggle_id.clone();
                        state_toggle.update(cx, |s, _cx| {
                            if !s.tree_expanded.remove(&tid) {
                                s.tree_expanded.insert(tid);
                            }
                        });
                    })
                    .on_click(move |_ev, _w, cx| {
                        state_select.update(cx, |s, _cx| {
                            s.tree_selected = Some(select_id.clone());
                        });
                    })
                    .on_double_click(move |_ev, _w, cx| {
                        let did = double_id.clone();
                        state_double.update(cx, |s, _cx| {
                            if !s.tree_expanded.remove(&did) {
                                s.tree_expanded.insert(did);
                            }
                        });
                    })
                    .render(cx, window),
            );
        }

        self.cell(
            cx.t("demo.lists.cell_tree"),
            tree.into_any_element(),
            cx,
        )
    }

    pub fn virtual_list_element(&self, cx: &mut App, _window: &mut Window) -> AnyElement {
        self.sync_virtual_list(cx);
        let state_for_row = self.state.clone();
        let state_for_range = self.state.clone();
        let vl_item_template = cx.t("demo.lists.vl_item");
        let vl = yororen_ui::headless::virtual_list::virtual_list(
            "lists-vl",
            &self.list_controller(cx),
            cx,
        )
        .row(move |ix, _window, cx| {
            let app_entity = state_for_row.clone();
            let selected = app_entity.read(cx).selected_list_item == Some(ix);
            let row_id: ElementId = format!("vl-row-{ix}").into();
            let row_label = vl_item_template.replacen("{}", &ix.to_string(), 1);
            yororen_ui::headless::list_item::list_item(row_id, row_label, cx)
                .selected(selected)
                .on_click({
                    let app_entity = app_entity.clone();
                    move |_ev, _w, cx| {
                        app_entity.update(cx, |s, _cx| {
                            s.selected_list_item = Some(ix);
                        });
                    }
                })
                .render(cx)
                .into_any_element()
        })
        .on_visible_range_change({
            move |range, total, _window, cx| {
                state_for_range.update(cx, |s, _cx| {
                    s.vl_visible_range = Some(range.clone());
                    if range.end + 50 >= total && s.vl_item_count < 5_000 {
                        s.vl_item_count += 100;
                        s.vl_load_count += 1;
                    }
                });
            }
        })
        .render(cx)
        .w(px(240.))
        .h(px(180.));

        let top_state = self.state.clone();
        let bottom_state = self.state.clone();
        let top_btn = yororen_ui::headless::button::button("vl-top", cx)
            .on_click(move |_, _, cx| {
                top_state.update(cx, |s, _| s.list_controller.scroll_to_top());
            })
            .render(cx)
            .child(cx.t("demo.common.top"));
        let bottom_btn = yororen_ui::headless::button::button("vl-bottom", cx)
            .on_click(move |_, _, cx| {
                bottom_state.update(cx, |s, _| s.list_controller.scroll_to_bottom());
            })
            .render(cx)
            .child(cx.t("demo.common.bottom"));
        let controls = div()
            .flex()
            .flex_row()
            .gap(px(6.))
            .child(top_btn)
            .child(bottom_btn);

        let visible = format!("{:?}", self.state.read(cx).vl_visible_range);
        let item_count = self.state.read(cx).vl_item_count;
        let load_count = self.state.read(cx).vl_load_count;
        let status = cx
            .t("demo.lists.vl_status")
            .replacen("{:?}", &visible, 1)
            .replacen("{}", &item_count.to_string(), 1)
            .replacen("{}", &load_count.to_string(), 1);
        let status_label = yororen_ui::headless::label::label("vl-status", status, cx)
            .muted(true)
            .render(cx);

        let el = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(vl)
            .child(controls)
            .child(status_label)
            .into_any_element();
        self.cell(cx.t("demo.lists.cell_vl"), el, cx)
    }

    pub fn uniform_list_element(&self, cx: &mut App, _window: &mut Window) -> AnyElement {
        let _state = self.state.clone();
        let uvl_item_template = cx.t("demo.lists.uvl_item");
        let uvl = yororen_ui::headless::virtual_list::uniform_virtual_list(
            "lists-uvl",
            1_000,
            &self.uniform_list_controller(cx),
            cx,
        )
        .row(move |ix, _w, cx| {
            let row_id: ElementId = format!("uvl-row-{ix}").into();
            let row_label = uvl_item_template.replacen("{}", &ix.to_string(), 1);
            yororen_ui::headless::list_item::list_item(row_id, row_label, cx)
                .render(cx)
                .into_any_element()
        })
        .render(cx)
        .w(px(240.))
        .h(px(180.));

        let top_state = self.state.clone();
        let bottom_state = self.state.clone();
        let top_btn = yororen_ui::headless::button::button("uvl-top", cx)
            .on_click(move |_, _, cx| {
                top_state.update(cx, |s, _| s.uniform_list_controller.scroll_to_top());
            })
            .render(cx)
            .child(cx.t("demo.common.top"));
        let bottom_btn = yororen_ui::headless::button::button("uvl-bottom", cx)
            .on_click(move |_, _, cx| {
                bottom_state.update(cx, |s, _| s.uniform_list_controller.scroll_to_bottom());
            })
            .render(cx)
            .child(cx.t("demo.common.bottom"));
        let controls = div()
            .flex()
            .flex_row()
            .gap(px(6.))
            .child(top_btn)
            .child(bottom_btn);

        let el = div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(uvl)
            .child(controls)
            .into_any_element();
        self.cell(cx.t("demo.lists.cell_uvl"), el, cx)
    }

    pub fn spacer_element(&self, cx: &mut App) -> AnyElement {
        let el = yororen_ui::headless::spacer::spacer("lists-spacer", cx)
            .render(cx)
            .h(px(16.))
            .w_full()
            .into_any_element();
        self.cell(cx.t("demo.lists.cell_spacer"), el, cx)
    }

    // -------- Data helpers used by XML --------

    pub fn split_items(&self, _cx: &App) -> Vec<DropdownItem> {
        vec![
            DropdownItem::Item(DropdownMenuItem::new("save", "Save")),
            DropdownItem::Item(DropdownMenuItem::new("save_as", "Save as…")),
            DropdownItem::Item(DropdownMenuItem::new("save_all", "Save all")),
        ]
    }

    pub fn table_columns(&self, cx: &App) -> Vec<TableColumn> {
        vec![
            TableColumn::new("name", cx.t("demo.lists.table_col_name")).width(120.),
            TableColumn::new("age", cx.t("demo.lists.table_col_age")).width(60.),
            TableColumn::new("city", cx.t("demo.lists.table_col_city")).width(120.),
        ]
    }

    pub fn table_rows(&self, cx: &App) -> Vec<Vec<SharedString>> {
        vec![
            vec![
                cx.t("demo.lists.table_row_alice").into(),
                cx.t("demo.lists.table_row_age_30").into(),
                cx.t("demo.lists.table_row_beijing").into(),
            ],
            vec![
                cx.t("demo.lists.table_row_bob").into(),
                cx.t("demo.lists.table_row_age_25").into(),
                cx.t("demo.lists.table_row_shanghai").into(),
            ],
            vec![
                cx.t("demo.lists.table_row_carol").into(),
                cx.t("demo.lists.table_row_age_28").into(),
                cx.t("demo.lists.table_row_shenzhen").into(),
            ],
        ]
    }

    pub fn tree_data(&self, cx: &App) -> TreeData {
        let mut data = TreeData::new();
        let root = yororen_ui::headless::tree::node_id("root");
        let child1 = yororen_ui::headless::tree::node_id("child1");
        let child2 = yororen_ui::headless::tree::node_id("child2");
        let leaf1 = yororen_ui::headless::tree::node_id("leaf1");
        let leaf2 = yororen_ui::headless::tree::node_id("leaf2");
        data.add(None, root.clone(), cx.t("demo.lists.tree_root"));
        data.add(
            Some(root.clone()),
            child1.clone(),
            cx.t("demo.lists.tree_child1"),
        );
        data.add(
            Some(root.clone()),
            child2.clone(),
            cx.t("demo.lists.tree_child2"),
        );
        data.add(
            Some(child1.clone()),
            leaf1.clone(),
            cx.t("demo.lists.tree_leaf1"),
        );
        data.add(
            Some(child1.clone()),
            leaf2.clone(),
            cx.t("demo.lists.tree_leaf2"),
        );
        data
    }

    // -------- Internal wiring --------

    fn wire_composite_state(state: &Entity<GalleryState>, cx: &mut App) {
        state.update(cx, |s, cx| {
            let state_for_select = state.clone();
            s.select_state.update(cx, |st, _cx| {
                st.set_on_change(move |value, _w, cx| {
                    let v = value.to_string();
                    state_for_select.update(cx, |s, _cx| s.select_demo_value = v);
                });
            });

            let state_for_combo = state.clone();
            s.combo_state.update(cx, |st, _cx| {
                st.set_on_change(move |value, _w, cx| {
                    let v = value.to_string();
                    state_for_combo.update(cx, |s, _cx| s.combo_demo_value = v);
                });
            });

            let state_for_listbox = state.clone();
            s.listbox_state.update(cx, |st, _cx| {
                st.set_on_change(move |value, _w, cx| {
                    let v = value.to_string();
                    state_for_listbox.update(cx, |s, _cx| s.listbox_demo_value = v);
                });
            });

            let state_for_menu = state.clone();
            let popover_state = s.popover_state.clone();
            s.menu_state.update(cx, |st, _cx| {
                st.set_on_select(move |id, _w, cx| {
                    let id_s = id.to_string();
                    state_for_menu.update(cx, |s, _cx| {
                        s.menu_demo_value = id_s.clone();
                        s.dropdown_demo_value = id_s;
                    });
                    popover_state.update(cx, |s, _cx| s.close());
                });
            });
        });
    }
}
