//! Gallery demo state — pure data, each mutable field
//! wrapped in its own `Entity<T>` so the XML layer's
//! `@bind` and brace expressions can read and write it
//! without races.
//!
//! The shape mirrors `gallery_demo::state::GalleryApp`,
//! but the XML view reads it through a global `StateRef`.

#![allow(dead_code)]

use std::collections::BTreeSet;

use gpui::{App, AppContext, Entity, Global};

use yororen_ui::headless::combo_box::ComboBoxState;
use yororen_ui::headless::dropdown_menu::DropdownMenuState;
use yororen_ui::headless::keybinding_input::KeybindingInputMode;
use yororen_ui::headless::listbox::{ListboxOption, ListboxState};
use yororen_ui::headless::menu::MenuState;
use yororen_ui::headless::modal::ModalState;
use yororen_ui::headless::popover::PopoverState;
use yororen_ui::headless::select::SelectState;
use yororen_ui::headless::tooltip::TooltipState;
use yororen_ui::headless::tree_item::TreeNodeId;
use yororen_ui::headless::virtual_list::{UniformVirtualListController, VirtualListController};

/// The 3 locales the toolbar can switch between.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LocaleChoice {
    #[default]
    En,
    ZhCn,
    Ar,
}

impl LocaleChoice {
    /// BCP-47 tag used by the `yororen_ui_locale_*` installers.
    pub fn tag(self) -> &'static str {
        match self {
            LocaleChoice::En => "en",
            LocaleChoice::ZhCn => "zh-CN",
            LocaleChoice::Ar => "ar",
        }
    }
}

pub use crate::theme_switcher::{DarkMode, RendererKind};

/// A simple counter used by the toolbar toast counter and the
/// closable-tag demo.
#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    pub value: usize,
}

/// A todo row — each row owns its own `done` entity so two
/// checkboxes in different rows never sync with each other.
#[derive(Debug, Clone)]
pub struct TodoItem {
    pub label: String,
    pub done: Entity<bool>,
}

/// The application state. Plain-value fields are read directly by
/// the XML; `Entity<T>` fields are used for `@bind` and for
/// composite state handles.
pub struct GalleryState {
    // -------- Toolbar state --------
    pub current_renderer: RendererKind,
    pub dark_mode: DarkMode,
    pub current_locale: LocaleChoice,
    pub toast_count: Counter,

    // -------- Composite `Entity<XxxState>` --------
    pub modal_state: Entity<ModalState>,
    pub popover_state: Entity<PopoverState>,
    pub tooltip_state: Entity<TooltipState>,
    pub select_state: Entity<SelectState>,
    pub combo_state: Entity<ComboBoxState>,
    pub dropdown_state: Entity<DropdownMenuState>,
    pub split_dropdown_state: Entity<DropdownMenuState>,
    pub menu_state: Entity<MenuState>,
    pub listbox_state: Entity<ListboxState>,

    // -------- Input values (bound via `@bind`) --------
    pub text_value: Entity<String>,
    pub password_value: Entity<String>,
    pub number_value: Entity<f64>,
    pub search_value: Entity<String>,
    pub file_path_value: Entity<String>,
    pub keybinding_value: String,
    pub keybinding_mode: KeybindingInputMode,
    pub text_area_value: Entity<String>,

    // -------- Composite on_change values --------
    pub select_demo_value: String,
    pub combo_demo_value: String,
    pub dropdown_demo_value: String,
    pub menu_demo_value: String,
    pub listbox_demo_value: String,

    // -------- Controls --------
    pub checkbox_value: Entity<bool>,
    pub switch_value: Entity<bool>,
    pub radio_value: usize, // 0/1/2
    pub slider_value: Entity<f32>,

    // -------- Display --------
    pub progress_value: f32,
    pub progress_indeterminate: bool,
    pub toggle_btn_selected: bool,
    pub tag_selected: bool,
    pub tag_closable_count: Counter,

    // -------- Overlays --------
    pub disclosure_open: bool,

    // -------- Lists --------
    pub selected_list_item: Option<usize>,
    pub selected_table_row: Option<usize>,
    pub form_submit_count: usize,
    pub form_email_value: Entity<String>,
    pub form_email_error: Option<String>,
    pub tree_expanded: BTreeSet<TreeNodeId>,
    pub tree_selected: Option<TreeNodeId>,
    pub list_controller: Entity<VirtualListController>,
    pub vl_visible_range: Option<std::ops::Range<usize>>,
    pub vl_item_count: usize,
    pub vl_load_count: usize,
    pub uniform_list_controller: Entity<UniformVirtualListController>,
}

impl GalleryState {
    pub fn new_data(cx: &mut App) -> Self {
        // Mint all composite `Entity<XxxState>`s here. Each
        // `&mut **cx` is a temporary borrow that drops before the
        // next call, so successive `XxxState::new(&mut **cx)` calls
        // do not alias.
        let modal_state = ModalState::new(cx);
        let popover_state = PopoverState::new(cx);
        let tooltip_state = TooltipState::new(cx);
        let select_state = SelectState::new(cx);
        let combo_state = ComboBoxState::new(cx);
        let dropdown_state = DropdownMenuState::new(cx);
        let split_dropdown_state = DropdownMenuState::new(cx);
        let menu_state = MenuState::new(cx);
        let listbox_state = ListboxState::new(cx);

        select_state.update(cx, |s, _cx| {
            s.set_options(vec![
                yororen_ui::headless::select::SelectOption::new("apple", "Apple"),
                yororen_ui::headless::select::SelectOption::new("banana", "Banana"),
                yororen_ui::headless::select::SelectOption::new("cherry", "Cherry"),
                yororen_ui::headless::select::SelectOption::new("durian", "Durian"),
            ]);
        });
        combo_state.update(cx, |s, _cx| {
            s.set_options(vec![
                yororen_ui::headless::combo_box::ComboBoxOption::new("rust", "Rust"),
                yororen_ui::headless::combo_box::ComboBoxOption::new("go", "Go"),
                yororen_ui::headless::combo_box::ComboBoxOption::new("python", "Python"),
                yororen_ui::headless::combo_box::ComboBoxOption::new("zig", "Zig"),
            ]);
        });
        dropdown_state.update(cx, |s, _cx| {
            use yororen_ui::headless::dropdown_menu::{DropdownItem, DropdownMenuItem};
            s.set_items(vec![
                DropdownItem::Item(DropdownMenuItem::new("cut", "Cut")),
                DropdownItem::Item(DropdownMenuItem::new("copy", "Copy")),
                DropdownItem::Item(DropdownMenuItem::new("paste", "Paste")),
                DropdownItem::Separator,
                DropdownItem::Item(DropdownMenuItem::new("select_all", "Select all")),
            ]);
        });
        menu_state.update(cx, |s, _cx| {
            use yororen_ui::headless::dropdown_menu::{DropdownItem, DropdownMenuItem};
            s.set_items(vec![
                DropdownItem::Item(DropdownMenuItem::new("profile", "Profile")),
                DropdownItem::Item(DropdownMenuItem::new("settings", "Settings")),
                DropdownItem::Separator,
                DropdownItem::Item(DropdownMenuItem::new("logout", "Log out")),
            ]);
        });
        listbox_state.update(cx, |s, _cx| {
            s.set_options(vec![
                ListboxOption::new("apple", "Apple"),
                ListboxOption::new("banana", "Banana"),
                ListboxOption::new("cherry", "Cherry").disabled(true),
                ListboxOption::new("durian", "Durian"),
                ListboxOption::new("elderberry", "Elderberry"),
            ]);
        });

        Self {
            current_renderer: RendererKind::default(),
            dark_mode: DarkMode::default(),
            current_locale: LocaleChoice::default(),
            toast_count: Counter { value: 0 },

            modal_state,
            popover_state,
            tooltip_state,
            select_state,
            combo_state,
            dropdown_state,
            split_dropdown_state,
            menu_state,
            listbox_state,

            text_value: cx.new(|_| String::new()),
            password_value: cx.new(|_| String::new()),
            number_value: cx.new(|_| 42.0),
            search_value: cx.new(|_| String::new()),
            file_path_value: cx.new(|_| String::new()),
            keybinding_value: String::new(),
            keybinding_mode: KeybindingInputMode::Idle,
            text_area_value: cx.new(|_| String::new()),

            select_demo_value: String::new(),
            combo_demo_value: String::new(),
            dropdown_demo_value: String::new(),
            menu_demo_value: String::new(),
            listbox_demo_value: String::new(),

            checkbox_value: cx.new(|_| false),
            switch_value: cx.new(|_| false),
            radio_value: 0,
            slider_value: cx.new(|_| 40.0),

            progress_value: 0.65,
            progress_indeterminate: false,
            toggle_btn_selected: false,
            tag_selected: true,
            tag_closable_count: Counter { value: 0 },

            disclosure_open: false,

            selected_list_item: Some(0),
            selected_table_row: Some(1),
            form_submit_count: 0,
            form_email_value: cx.new(|_| String::new()),
            form_email_error: None,
            tree_expanded: BTreeSet::new(),
            tree_selected: None,
            list_controller: cx.new(|_| {
                VirtualListController::new(100, gpui::ListAlignment::Top, gpui::px(20.))
            }),
            vl_visible_range: None,
            vl_item_count: 100,
            vl_load_count: 0,
            uniform_list_controller: cx.new(|_| UniformVirtualListController::new()),
        }
    }
}

/// Global handle to the state entity. Stored once at
/// startup; the view reads it via `cx.global::<StateRef>()`.
#[derive(Clone)]
pub struct StateRef {
    pub state: Entity<GalleryState>,
}

impl Global for StateRef {}
