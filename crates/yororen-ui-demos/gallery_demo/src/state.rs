//! State owned by [`GalleryApp`](crate::gallery_app::GalleryApp).
//!
//! Holds plain-value fields for the simple (props-only / input
//! on_change) components, and `Entity<XxxState>` for the 8 composite
//! stateful components (modal / popover / tooltip / select /
//! combo_box / dropdown_menu / menu). The composite `Entity`s are
//! minted in [`GalleryApp::new`](crate::gallery_app::GalleryApp::new)
//! using `&mut **cx` to recover a `&mut gpui::App` from the
//! `&mut Context<Self>` (per the v0.3 `DerefMut<Target = App>`
//! pattern — see `memory.md` "Context<T> → App").

use std::collections::BTreeSet;

use gpui::Entity;

use yororen_ui::headless::combo_box::ComboBoxState;
use yororen_ui::headless::dropdown_menu::DropdownMenuState;
use yororen_ui::headless::keybinding_input::KeybindingInputMode;
use yororen_ui::headless::menu::MenuState;
use yororen_ui::headless::modal::ModalState;
use yororen_ui::headless::popover::PopoverState;
use yororen_ui::headless::select::SelectState;
use yororen_ui::headless::tooltip::TooltipState;
use yororen_ui::headless::tree_item::TreeNodeId;

use crate::theme_switcher::{DarkMode, RendererKind};

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

// The `progress_indeterminate` / `form_password_value` /
// `tree_selected` fields are reserved for future section
// expansion (each is a logical "next step" of an existing
// section's existing field); suppress the dead-code warning
// so the demo compiles clean.
#[allow(dead_code)]
pub struct GalleryApp {
    // -------- Toolbar state --------
    pub current_renderer: RendererKind,
    pub dark_mode: DarkMode,
    pub current_locale: LocaleChoice,
    pub toast_count: usize,

    // -------- Composite `Entity<XxxState>` --------
    pub modal_state: Entity<ModalState>,
    pub popover_state: Entity<PopoverState>,
    pub tooltip_state: Entity<TooltipState>,
    pub select_state: Entity<SelectState>,
    pub combo_state: Entity<ComboBoxState>,
    pub dropdown_state: Entity<DropdownMenuState>,
    pub split_dropdown_state: Entity<DropdownMenuState>,
    pub menu_state: Entity<MenuState>,

    // -------- Input values (mirrored via on_change) --------
    pub text_value: String,
    pub password_value: String,
    pub number_value: f64,
    pub search_value: String,
    pub file_path_value: String,
    pub keybinding_value: String,
    pub keybinding_mode: KeybindingInputMode,
    pub text_area_value: String,

    // -------- Composite on_change values --------
    pub select_demo_value: String,
    pub combo_demo_value: String,
    pub dropdown_demo_value: String,
    pub menu_demo_value: String,

    // -------- Controls --------
    pub checkbox_value: bool,
    pub switch_value: bool,
    pub radio_value: usize, // 0/1/2
    pub slider_value: f32,

    // -------- Display --------
    pub progress_value: f32,
    pub progress_indeterminate: bool,
    pub toggle_btn_selected: bool,
    pub tag_selected: bool,
    pub tag_closable_count: usize,

    // -------- Overlays --------
    pub disclosure_open: bool,

    // -------- Lists --------
    pub selected_list_item: Option<usize>,
    pub selected_table_row: Option<usize>,
    pub form_submit_count: usize,
    pub form_email_value: String,
    pub form_password_value: String,
    pub form_email_error: Option<String>,
    pub tree_expanded: BTreeSet<TreeNodeId>,
    pub tree_selected: Option<TreeNodeId>,
}

impl GalleryApp {
    // The `&mut **cx` is the documented v0.3 pattern to recover a
    // `&mut gpui::App` from a `&mut Context<Self>` (see `memory.md`).
    #[allow(clippy::explicit_auto_deref)]
    pub fn new(cx: &mut gpui::Context<Self>) -> Self {
        // Mint all 8 composite `Entity<XxxState>`s here. Each
        // `&mut **cx` is a temporary borrow that drops before the
        // next call, so successive `XxxState::new(&mut **cx)` calls
        // do not alias. The result is an owned `Entity<XxxState>`
        // that the `Render` closure can clone into callbacks.
        let modal_state = ModalState::new(&mut **cx);
        let popover_state = PopoverState::new(&mut **cx);
        let tooltip_state = TooltipState::new(&mut **cx);
        let select_state = SelectState::new(&mut **cx);
        let combo_state = ComboBoxState::new(&mut **cx);
        let dropdown_state = DropdownMenuState::new(&mut **cx);
        let split_dropdown_state = DropdownMenuState::new(&mut **cx);
        let menu_state = MenuState::new(&mut **cx);

        // Seed the select / combo / dropdown / menu options so
        // the renderer's first paint shows the full menu.
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
        // `split_dropdown_state` is a `DropdownMenuState` re-used
        // only for its `open` flag — the split_button demo cell
        // passes its own items / on_select via builder methods,
        // so we don't seed `items` here.
        menu_state.update(cx, |s, _cx| {
            use yororen_ui::headless::dropdown_menu::{DropdownItem, DropdownMenuItem};
            s.set_items(vec![
                DropdownItem::Item(DropdownMenuItem::new("profile", "Profile")),
                DropdownItem::Item(DropdownMenuItem::new("settings", "Settings")),
                DropdownItem::Separator,
                DropdownItem::Item(DropdownMenuItem::new("logout", "Log out")),
            ]);
        });

        Self {
            // Toolbar
            current_renderer: RendererKind::default(),
            dark_mode: DarkMode::default(),
            current_locale: LocaleChoice::default(),
            toast_count: 0,

            // Composite
            modal_state,
            popover_state,
            tooltip_state,
            select_state,
            combo_state,
            dropdown_state,
            split_dropdown_state,
            menu_state,

            // Inputs
            text_value: String::new(),
            password_value: String::new(),
            number_value: 42.0,
            search_value: String::new(),
            file_path_value: String::new(),
            keybinding_value: String::new(),
            keybinding_mode: KeybindingInputMode::Idle,
            text_area_value: String::new(),

            // Composite on_change mirrors
            select_demo_value: String::new(),
            combo_demo_value: String::new(),
            dropdown_demo_value: String::new(),
            menu_demo_value: String::new(),

            // Controls
            checkbox_value: false,
            switch_value: false,
            radio_value: 0,
            slider_value: 40.0,

            // Display
            progress_value: 0.65,
            progress_indeterminate: false,
            toggle_btn_selected: false,
            tag_selected: true,
            tag_closable_count: 0,

            // Overlays
            disclosure_open: false,

            // Lists
            selected_list_item: Some(0),
            selected_table_row: Some(1),
            form_submit_count: 0,
            form_email_value: String::new(),
            form_password_value: String::new(),
            form_email_error: None,
            tree_expanded: BTreeSet::new(),
            tree_selected: None,
        }
    }
}
