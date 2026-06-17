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
    AnyElement, App, ClickEvent, Entity, IntoElement, ParentElement, SharedString, Styled, Window,
    div, hsla, px,
};

use yororen_ui::headless::dropdown_menu::{DropdownItem, DropdownMenuItem};
use yororen_ui::headless::keybinding_input::KeybindingInputMode;
use yororen_ui::headless::table::TableColumn;
use yororen_ui::headless::tree::TreeData;
use yororen_ui::headless::tree_item::TreeNodeId;
use yororen_ui::notification::center::{Notification, NotificationCenter, ToastKind};

use crate::state::{DarkMode, GalleryState, LocaleChoice, RendererKind};
use yororen_ui::i18n::Translate;
use yororen_ui::t_named;

#[derive(Clone)]
pub struct Controller {
    state: Entity<GalleryState>,
}

/// A single-read snapshot of all scalar demo fields.
///
/// Use `controller.snapshot(cx)` in XML brace expressions to
/// read any plain value without paying for a separate
/// `state.read(cx)` borrow guard per field. `Entity<T>` fields
/// that participate in `@bind` are intentionally **not** here —
/// they keep their own `*_entity(cx)` accessors.
pub struct GallerySnapshot {
    // Toolbar
    pub current_renderer: RendererKind,
    pub dark_mode: DarkMode,
    pub current_locale: LocaleChoice,
    pub toast_count: usize,

    // Display
    pub tag_selected: bool,
    pub tag_close_count: usize,
    pub progress_value: f32,
    pub toggle_btn_selected: bool,

    // Overlays
    pub disclosure_open: bool,
    pub popover_visible: bool,
    pub dropdown_visible: bool,
    pub is_modal_open: bool,
    pub dropdown_demo_value: String,
    pub menu_demo_value: String,

    // Lists
    pub selected_list_item: Option<usize>,
    pub selected_table_row: Option<usize>,
    pub form_submit_count: usize,
    pub form_email_value: String,
    pub form_email_error: Option<String>,

    // Controls
    pub checkbox_value: bool,
    pub switch_value: bool,
    pub radio_value: usize,
    pub slider_value: f32,

    // Inputs
    pub text_value: String,
    pub password_value: String,
    pub number_value: f64,
    pub search_value: String,
    pub file_path_value: String,
    pub keybinding_value: String,
    pub keybinding_mode: KeybindingInputMode,
    pub text_area_value: String,
    pub select_demo_value: String,
    pub combo_demo_value: String,
    pub listbox_demo_value: String,
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

    /// Read all scalar fields once and return a snapshot.
    /// XML brace expressions should use this instead of individual
    /// getters to avoid repeated `state.read(cx)` borrow guards.
    pub fn snapshot(&self, cx: &App) -> GallerySnapshot {
        let s = self.state.read(cx);
        GallerySnapshot {
            current_renderer: s.current_renderer,
            dark_mode: s.dark_mode,
            current_locale: s.current_locale,
            toast_count: s.toast_count.value,

            tag_selected: s.tag_selected,
            tag_close_count: s.tag_closable_count.value,
            progress_value: s.progress_value,
            toggle_btn_selected: s.toggle_btn_selected,

            disclosure_open: s.disclosure_open,
            popover_visible: s.popover_state.read(cx).is_visible(),
            dropdown_visible: s.dropdown_state.read(cx).is_visible(),
            is_modal_open: s.modal_state.read(cx).open,
            dropdown_demo_value: s.dropdown_demo_value.clone(),
            menu_demo_value: s.menu_demo_value.clone(),

            selected_list_item: s.selected_list_item,
            selected_table_row: s.selected_table_row,
            form_submit_count: s.form_submit_count,
            form_email_value: s.form_email_value.read(cx).clone(),
            form_email_error: s.form_email_error.clone(),

            checkbox_value: *s.checkbox_value.read(cx),
            switch_value: *s.switch_value.read(cx),
            radio_value: s.radio_value,
            slider_value: *s.slider_value.read(cx),

            text_value: s.text_value.read(cx).clone(),
            password_value: s.password_value.read(cx).clone(),
            number_value: *s.number_value.read(cx),
            search_value: s.search_value.read(cx).clone(),
            file_path_value: s.file_path_value.read(cx).clone(),
            keybinding_value: s.keybinding_value.clone(),
            keybinding_mode: s.keybinding_mode,
            text_area_value: s.text_area_value.read(cx).clone(),
            select_demo_value: s.select_demo_value.clone(),
            combo_demo_value: s.combo_demo_value.clone(),
            listbox_demo_value: s.listbox_demo_value.clone(),
        }
    }

    /// Build the footer form-summary text in a single state read.
    pub fn footer_form_text(&self, cx: &App) -> String {
        let s = self.snapshot(cx);
        t_named!(cx, "demo.footer.form_summary",
            count => s.form_submit_count,
            email => s.form_email_value,
            error => format!("{:?}", s.form_email_error))
        .to_string()
    }

    /// Build the footer controls-summary text in a single state read.
    pub fn footer_controls_text(&self, cx: &App) -> String {
        let s = self.snapshot(cx);
        t_named!(cx, "demo.footer.controls_summary",
            checkbox => s.checkbox_value,
            switch => s.switch_value,
            radio => s.radio_value,
            slider => format!("{:.1}", s.slider_value))
        .to_string()
    }

    /// Build the footer toast-summary text in a single state read.
    pub fn footer_toast_text(&self, cx: &App) -> String {
        let s = self.snapshot(cx);
        t_named!(cx, "demo.footer.toast_summary",
            count => s.toast_count,
            locale => s.current_locale.tag())
        .to_string()
    }

    // -------- Entity accessors used by `@bind` and view.rs --------

    pub fn modal_state(&self, cx: &App) -> Entity<yororen_ui::headless::modal::ModalState> {
        self.state.read(cx).modal_state.clone()
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

    pub fn list_controller_entity(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::virtual_list::VirtualListController> {
        self.state.read(cx).list_controller.clone()
    }

    pub fn uniform_list_controller_entity(
        &self,
        cx: &App,
    ) -> Entity<yororen_ui::headless::virtual_list::UniformVirtualListController> {
        self.state.read(cx).uniform_list_controller.clone()
    }

    pub fn vl_item_count(&self, cx: &App) -> usize {
        self.state.read(cx).vl_item_count
    }

    // -------- Entity accessors for `@bind` --------

    pub fn form_email_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).form_email_value.clone()
    }

    pub fn checkbox_value_entity(&self, cx: &App) -> Entity<bool> {
        self.state.read(cx).checkbox_value.clone()
    }

    pub fn switch_value_entity(&self, cx: &App) -> Entity<bool> {
        self.state.read(cx).switch_value.clone()
    }

    pub fn slider_value_entity(&self, cx: &App) -> Entity<f32> {
        self.state.read(cx).slider_value.clone()
    }

    pub fn text_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).text_value.clone()
    }

    pub fn password_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).password_value.clone()
    }

    pub fn number_value_entity(&self, cx: &App) -> Entity<f64> {
        self.state.read(cx).number_value.clone()
    }

    pub fn search_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).search_value.clone()
    }

    pub fn file_path_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).file_path_value.clone()
    }

    pub fn text_area_value_entity(&self, cx: &App) -> Entity<String> {
        self.state.read(cx).text_area_value.clone()
    }

    // -------- Collection / complex accessors --------

    pub fn list_items(&self, cx: &App) -> Vec<gpui::SharedString> {
        self.state.read(cx).list_items.clone()
    }

    pub fn tree_expanded(&self, cx: &App) -> BTreeSet<TreeNodeId> {
        self.state.read(cx).tree_expanded.clone()
    }

    pub fn tree_selected(&self, cx: &App) -> Option<TreeNodeId> {
        self.state.read(cx).tree_selected.clone()
    }

    // -------- Toolbar actions --------

    pub fn set_renderer(&self, kind: RendererKind, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| s.current_renderer = kind);
    }

    pub fn set_dark_mode(&self, dark: DarkMode, _w: &mut Window, cx: &mut App) {
        self.state.update(cx, |s, _cx| s.dark_mode = dark);
    }

    pub fn set_locale(&self, choice: LocaleChoice, _w: &mut Window, cx: &mut App) {
        crate::i18n::install_for_locale(cx, choice);
        self.state.update(cx, |s, cx| {
            s.current_locale = choice;
            // Re-resolve translated labels so the tree stays in
            // sync with the new locale.
            s.tree_data = GalleryState::build_tree_data(cx);
        });
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
            DropdownItem::Item(DropdownMenuItem::new("save", _cx.t("demo.actions.save"))),
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

    pub fn browse_file_path(&self, _value: &str, _w: &mut Window, _cx: &mut App) {
        // The renderer already wrote the picked path into the
        // input's bound state and fired `on_change`; this hook is
        // only for extra work (logging, validation, etc.).
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

    // -------- Lists section --------

    pub fn submit_form(&self, _vals: HashMap<SharedString, String>, _w: &mut Window, cx: &mut App) {
        let email = self.snapshot(cx).form_email_value;
        let must_contain = cx.t("demo.form.must_contain_at");
        self.state.update(cx, |s, _cx| {
            s.form_submit_count += 1;
            s.form_email_error = if email.contains('@') {
                None
            } else {
                Some(must_contain.to_string())
            };
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
        self.list_controller_entity(cx)
            .update(cx, |c, _| c.scroll_to_top());
    }

    pub fn virtual_list_scroll_bottom(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.list_controller_entity(cx)
            .update(cx, |c, _| c.scroll_to_bottom());
    }

    pub fn uniform_list_scroll_top(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.uniform_list_controller_entity(cx)
            .update(cx, |c, _| c.scroll_to_top());
    }

    pub fn uniform_list_scroll_bottom(&self, _ev: &ClickEvent, _w: &mut Window, cx: &mut App) {
        self.uniform_list_controller_entity(cx)
            .update(cx, |c, _| c.scroll_to_bottom());
    }

    pub fn on_visible_range_change(
        &self,
        range: Range<usize>,
        total: usize,
        _w: &mut Window,
        cx: &mut App,
    ) {
        self.state.update(cx, |s, _cx| {
            s.vl_visible_range = Some(range.clone());
            // Only load the next batch when the visible end is
            // within 50 items of the total (matches gallery_demo).
            if range.end + 50 >= total && s.vl_item_count < 5_000 {
                s.vl_item_count += 100;
                s.vl_load_count += 1;
            }
        });
    }

    // -------- Complex layout helpers (used by lists.xml) --------

    pub fn sync_virtual_list(&self, cx: &mut App) {
        let target = self.state.read(cx).vl_item_count;
        self.list_controller_entity(cx).update(cx, |c, _| {
            let current = c.state().item_count();
            if current < target {
                c.append(target - current);
            } else if current > target {
                c.reset(target);
            }
        });
    }

    pub fn listbox_status_text(&self, cx: &App) -> String {
        let value = self.snapshot(cx).listbox_demo_value;
        let status = if value.is_empty() {
            "—".to_string()
        } else {
            value
        };
        cx.t("demo.lists.listbox_selected")
            .replacen("{}", &status, 1)
    }

    pub fn form_email_error_text(&self, cx: &App) -> String {
        self.snapshot(cx).form_email_error.unwrap_or_default()
    }

    pub fn form_status_text(&self, cx: &App) -> String {
        let s = self.snapshot(cx);
        format!(
            "{} {} | {} {:?}",
            cx.t("demo.form.submitted"),
            s.form_submit_count,
            cx.t("demo.form.last_error"),
            s.form_email_error
        )
    }

    pub fn tree_label(&self, id: &TreeNodeId, cx: &App) -> String {
        let state = self.state.read(cx);
        state.tree_data.label_of(id).unwrap_or("").to_string()
    }

    pub fn tree_has_children(&self, id: &TreeNodeId, cx: &App) -> bool {
        let state = self.state.read(cx);
        !state.tree_data.children_of(id).is_empty()
    }

    pub fn toggle_tree_node_for(
        &self,
        id: TreeNodeId,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, _cx| {
                if !s.tree_expanded.remove(&id) {
                    s.tree_expanded.insert(id.clone());
                }
            });
        }
    }

    pub fn select_tree_node_for(
        &self,
        id: TreeNodeId,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, _cx| {
                s.tree_selected = Some(id.clone());
            });
        }
    }

    pub fn select_virtual_list_item(
        &self,
        index: usize,
    ) -> impl Fn(&ClickEvent, &mut Window, &mut App) + Clone + 'static {
        let state = self.state.clone();
        move |_ev, _w, cx| {
            state.update(cx, |s, _cx| {
                s.selected_list_item = Some(index);
            });
        }
    }

    pub fn vl_row_label(&self, index: usize, cx: &App) -> String {
        cx.t("demo.lists.vl_item")
            .replacen("{}", &index.to_string(), 1)
    }

    pub fn uvl_row_label(&self, index: usize, cx: &App) -> String {
        cx.t("demo.lists.uvl_item")
            .replacen("{}", &index.to_string(), 1)
    }

    pub fn vl_status_text(&self, cx: &App) -> String {
        let s = self.state.read(cx);
        let visible = format!("{:?}", s.vl_visible_range);
        cx.t("demo.lists.vl_status")
            .replacen("{:?}", &visible, 1)
            .replacen("{}", &s.vl_item_count.to_string(), 1)
            .replacen("{}", &s.vl_load_count.to_string(), 1)
    }

    pub fn virtual_list_row(
        &self,
        cx: &App,
    ) -> impl FnMut(usize, &mut Window, &mut App) -> gpui::AnyElement + 'static {
        let state = self.state.clone();
        let item_template = cx.t("demo.lists.vl_item");
        move |ix, _w, cx| {
            let selected = state.read(cx).selected_list_item == Some(ix);
            let row_id: gpui::ElementId = format!("vl-row-{ix}").into();
            let row_label = item_template.replacen("{}", &ix.to_string(), 1);
            let state_for_click = state.clone();
            yororen_ui::headless::list_item::list_item(row_id, row_label, cx)
                .selected(selected)
                .on_click(move |_ev, _w, cx| {
                    state_for_click.update(cx, |s, _cx| {
                        s.selected_list_item = Some(ix);
                    });
                })
                .render(cx)
                .into_any_element()
        }
    }

    pub fn uniform_list_row(
        &self,
        cx: &App,
    ) -> impl FnMut(usize, &mut Window, &mut App) -> gpui::AnyElement + 'static {
        let item_template = cx.t("demo.lists.uvl_item");
        move |ix, _w, cx| {
            let row_id: gpui::ElementId = format!("uvl-row-{ix}").into();
            let row_label = item_template.replacen("{}", &ix.to_string(), 1);
            yororen_ui::headless::list_item::list_item(row_id, row_label, cx)
                .render(cx)
                .into_any_element()
        }
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
                cx.t("demo.lists.table_row_alice"),
                cx.t("demo.lists.table_row_age_30"),
                cx.t("demo.lists.table_row_beijing"),
            ],
            vec![
                cx.t("demo.lists.table_row_bob"),
                cx.t("demo.lists.table_row_age_25"),
                cx.t("demo.lists.table_row_shanghai"),
            ],
            vec![
                cx.t("demo.lists.table_row_carol"),
                cx.t("demo.lists.table_row_age_28"),
                cx.t("demo.lists.table_row_shenzhen"),
            ],
        ]
    }

    pub fn tree_data(&self, cx: &App) -> TreeData {
        // Tree data is built once in `GalleryState::new_data` and
        // refreshed in `set_locale`. Cloning the cached value is
        // cheaper than rebuilding the maps and resolving translations
        // on every frame.
        self.state.read(cx).tree_data.clone()
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
