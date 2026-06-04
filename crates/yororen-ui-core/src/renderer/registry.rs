//! `RendererRegistry` — the collection of component renderers wired into
//! a `Theme`. Phase B ships the button entry; Phase C adds the remaining
//! 30+ components one trait per file.

use std::sync::Arc;

use super::avatar::{AvatarRenderer, TokenAvatarRenderer};
use super::badge::{BadgeRenderer, TokenBadgeRenderer};
use super::button::{ButtonRenderer, TokenButtonRenderer};
use super::card::{CardRenderer, TokenCardRenderer};
use super::checkbox::{CheckboxRenderer, TokenCheckboxRenderer};
use super::combo_box::{ComboBoxRenderer, TokenComboBoxRenderer};
use super::disclosure::{DisclosureRenderer, TokenDisclosureRenderer};
use super::divider::{DividerRenderer, TokenDividerRenderer};
use super::dropdown_menu::{DropdownMenuRenderer, TokenDropdownMenuRenderer};
use super::empty_state::{EmptyStateRenderer, TokenEmptyStateRenderer};
use super::file_path_input::{FilePathInputRenderer, TokenFilePathInputRenderer};
use super::focus_ring::{FocusRingRenderer, TokenFocusRingRenderer};
use super::form::{FormRenderer, TokenFormRenderer};
use super::heading::{HeadingRenderer, TokenHeadingRenderer};
use super::icon_button::{IconButtonRenderer, TokenIconButtonRenderer};
use super::keybinding_input::{KeybindingInputRenderer, TokenKeybindingInputRenderer};
use super::label::{LabelRenderer, TokenLabelRenderer};
use super::list_item::{ListItemRenderer, TokenListItemRenderer};
use super::modal::{ModalRenderer, TokenModalRenderer};
use super::notification::{NotificationRenderer, TokenNotificationRenderer};
use super::number_input::{NumberInputRenderer, TokenNumberInputRenderer};
use super::panel::{PanelRenderer, TokenPanelRenderer};
use super::password_input::{PasswordInputRenderer, TokenPasswordInputRenderer};
use super::popover::{PopoverRenderer, TokenPopoverRenderer};
use super::progress::{ProgressBarRenderer, TokenProgressBarRenderer};
use super::radio::{RadioRenderer, TokenRadioRenderer};
use super::search_input::{SearchInputRenderer, TokenSearchInputRenderer};
use super::select::{SelectRenderer, TokenSelectRenderer};
use super::skeleton::{SkeletonRenderer, TokenSkeletonRenderer};
use super::split_button::{SplitButtonRenderer, TokenSplitButtonRenderer};
use super::switch::{SwitchRenderer, TokenSwitchRenderer};
use super::tag::{TagRenderer, TokenTagRenderer};
use super::text_area::{TextAreaRenderer, TokenTextAreaRenderer};
use super::text_input::{TextInputRenderer, TokenTextInputRenderer};
use super::toast::{ToastRenderer, TokenToastRenderer};
use super::toggle_button::{ToggleButtonRenderer, TokenToggleButtonRenderer};
use super::tooltip::{TokenTooltipRenderer, TooltipRenderer};
use super::tree_item::{TokenTreeItemRenderer, TreeItemRenderer};

#[derive(Clone)]
pub struct RendererRegistry {
    pub button: Arc<dyn ButtonRenderer>,
    pub icon_button: Arc<dyn IconButtonRenderer>,
    pub toggle_button: Arc<dyn ToggleButtonRenderer>,
    pub label: Arc<dyn LabelRenderer>,
    pub heading: Arc<dyn HeadingRenderer>,
    pub divider: Arc<dyn DividerRenderer>,
    pub focus_ring: Arc<dyn FocusRingRenderer>,
    pub badge: Arc<dyn BadgeRenderer>,
    pub tag: Arc<dyn TagRenderer>,
    pub progress_bar: Arc<dyn ProgressBarRenderer>,
    pub skeleton: Arc<dyn SkeletonRenderer>,
    pub tooltip: Arc<dyn TooltipRenderer>,
    pub avatar: Arc<dyn AvatarRenderer>,
    pub switch: Arc<dyn SwitchRenderer>,
    pub checkbox: Arc<dyn CheckboxRenderer>,
    pub radio: Arc<dyn RadioRenderer>,
    pub text_input: Arc<dyn TextInputRenderer>,
    pub text_area: Arc<dyn TextAreaRenderer>,
    pub password_input: Arc<dyn PasswordInputRenderer>,
    pub number_input: Arc<dyn NumberInputRenderer>,
    pub file_path_input: Arc<dyn FilePathInputRenderer>,
    pub search_input: Arc<dyn SearchInputRenderer>,
    pub select: Arc<dyn SelectRenderer>,
    pub combo_box: Arc<dyn ComboBoxRenderer>,
    pub modal: Arc<dyn ModalRenderer>,
    pub popover: Arc<dyn PopoverRenderer>,
    pub dropdown_menu: Arc<dyn DropdownMenuRenderer>,
    pub disclosure: Arc<dyn DisclosureRenderer>,
    pub toast: Arc<dyn ToastRenderer>,
    pub notification: Arc<dyn NotificationRenderer>,
    pub panel: Arc<dyn PanelRenderer>,
    pub card: Arc<dyn CardRenderer>,
    pub form: Arc<dyn FormRenderer>,
    pub list_item: Arc<dyn ListItemRenderer>,
    pub tree_item: Arc<dyn TreeItemRenderer>,
    pub keybinding_input: Arc<dyn KeybindingInputRenderer>,
    pub split_button: Arc<dyn SplitButtonRenderer>,
    pub empty_state: Arc<dyn EmptyStateRenderer>,
}

impl std::fmt::Debug for RendererRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererRegistry").finish_non_exhaustive()
    }
}

impl Default for RendererRegistry {
    fn default() -> Self {
        Self::token_based()
    }
}

impl RendererRegistry {
    /// All renderers set to the default `TokenXxxRenderer` implementations.
    /// This is the v0.3 / v0.4 visual baseline.
    pub fn token_based() -> Self {
        Self {
            button: Arc::new(TokenButtonRenderer),
            icon_button: Arc::new(TokenIconButtonRenderer),
            toggle_button: Arc::new(TokenToggleButtonRenderer),
            label: Arc::new(TokenLabelRenderer),
            heading: Arc::new(TokenHeadingRenderer),
            divider: Arc::new(TokenDividerRenderer),
            focus_ring: Arc::new(TokenFocusRingRenderer),
            badge: Arc::new(TokenBadgeRenderer),
            tag: Arc::new(TokenTagRenderer),
            progress_bar: Arc::new(TokenProgressBarRenderer),
            skeleton: Arc::new(TokenSkeletonRenderer),
            tooltip: Arc::new(TokenTooltipRenderer),
            avatar: Arc::new(TokenAvatarRenderer),
            switch: Arc::new(TokenSwitchRenderer),
            checkbox: Arc::new(TokenCheckboxRenderer),
            radio: Arc::new(TokenRadioRenderer),
            text_input: Arc::new(TokenTextInputRenderer),
            text_area: Arc::new(TokenTextAreaRenderer),
            password_input: Arc::new(TokenPasswordInputRenderer),
            number_input: Arc::new(TokenNumberInputRenderer),
            file_path_input: Arc::new(TokenFilePathInputRenderer),
            search_input: Arc::new(TokenSearchInputRenderer),
            select: Arc::new(TokenSelectRenderer),
            combo_box: Arc::new(TokenComboBoxRenderer),
            modal: Arc::new(TokenModalRenderer),
            popover: Arc::new(TokenPopoverRenderer),
            dropdown_menu: Arc::new(TokenDropdownMenuRenderer),
            disclosure: Arc::new(TokenDisclosureRenderer),
            toast: Arc::new(TokenToastRenderer),
            notification: Arc::new(TokenNotificationRenderer),
            panel: Arc::new(TokenPanelRenderer),
            card: Arc::new(TokenCardRenderer),
            form: Arc::new(TokenFormRenderer),
            list_item: Arc::new(TokenListItemRenderer),
            tree_item: Arc::new(TokenTreeItemRenderer),
            keybinding_input: Arc::new(TokenKeybindingInputRenderer),
            split_button: Arc::new(TokenSplitButtonRenderer),
            empty_state: Arc::new(TokenEmptyStateRenderer),
        }
    }

    pub fn with_button(mut self, r: Arc<dyn ButtonRenderer>) -> Self {
        self.button = r;
        self
    }
    pub fn with_label(mut self, r: Arc<dyn LabelRenderer>) -> Self {
        self.label = r;
        self
    }
    pub fn with_heading(mut self, r: Arc<dyn HeadingRenderer>) -> Self {
        self.heading = r;
        self
    }
    pub fn with_divider(mut self, r: Arc<dyn DividerRenderer>) -> Self {
        self.divider = r;
        self
    }
    pub fn with_focus_ring(mut self, r: Arc<dyn FocusRingRenderer>) -> Self {
        self.focus_ring = r;
        self
    }
    pub fn with_badge(mut self, r: Arc<dyn BadgeRenderer>) -> Self {
        self.badge = r;
        self
    }
    pub fn with_tag(mut self, r: Arc<dyn TagRenderer>) -> Self {
        self.tag = r;
        self
    }
    pub fn with_progress_bar(mut self, r: Arc<dyn ProgressBarRenderer>) -> Self {
        self.progress_bar = r;
        self
    }
    pub fn with_skeleton(mut self, r: Arc<dyn SkeletonRenderer>) -> Self {
        self.skeleton = r;
        self
    }
    pub fn with_tooltip(mut self, r: Arc<dyn TooltipRenderer>) -> Self {
        self.tooltip = r;
        self
    }
    pub fn with_avatar(mut self, r: Arc<dyn AvatarRenderer>) -> Self {
        self.avatar = r;
        self
    }
    pub fn with_switch(mut self, r: Arc<dyn SwitchRenderer>) -> Self {
        self.switch = r;
        self
    }
    pub fn with_checkbox(mut self, r: Arc<dyn CheckboxRenderer>) -> Self {
        self.checkbox = r;
        self
    }
    pub fn with_radio(mut self, r: Arc<dyn RadioRenderer>) -> Self {
        self.radio = r;
        self
    }
    pub fn with_icon_button(mut self, r: Arc<dyn IconButtonRenderer>) -> Self {
        self.icon_button = r;
        self
    }
    pub fn with_toggle_button(mut self, r: Arc<dyn ToggleButtonRenderer>) -> Self {
        self.toggle_button = r;
        self
    }
    pub fn with_text_input(mut self, r: Arc<dyn TextInputRenderer>) -> Self {
        self.text_input = r;
        self
    }
    pub fn with_modal(mut self, r: Arc<dyn ModalRenderer>) -> Self {
        self.modal = r;
        self
    }
    pub fn with_popover(mut self, r: Arc<dyn PopoverRenderer>) -> Self {
        self.popover = r;
        self
    }
    pub fn with_dropdown_menu(mut self, r: Arc<dyn DropdownMenuRenderer>) -> Self {
        self.dropdown_menu = r;
        self
    }
    pub fn with_toast(mut self, r: Arc<dyn ToastRenderer>) -> Self {
        self.toast = r;
        self
    }
    pub fn with_notification(mut self, r: Arc<dyn NotificationRenderer>) -> Self {
        self.notification = r;
        self
    }
    pub fn with_panel(mut self, r: Arc<dyn PanelRenderer>) -> Self {
        self.panel = r;
        self
    }
    pub fn with_card(mut self, r: Arc<dyn CardRenderer>) -> Self {
        self.card = r;
        self
    }
    pub fn with_form(mut self, r: Arc<dyn FormRenderer>) -> Self {
        self.form = r;
        self
    }
    pub fn with_list_item(mut self, r: Arc<dyn ListItemRenderer>) -> Self {
        self.list_item = r;
        self
    }
    pub fn with_text_area(mut self, r: Arc<dyn TextAreaRenderer>) -> Self {
        self.text_area = r;
        self
    }
    pub fn with_password_input(mut self, r: Arc<dyn PasswordInputRenderer>) -> Self {
        self.password_input = r;
        self
    }
    pub fn with_number_input(mut self, r: Arc<dyn NumberInputRenderer>) -> Self {
        self.number_input = r;
        self
    }
    pub fn with_file_path_input(mut self, r: Arc<dyn FilePathInputRenderer>) -> Self {
        self.file_path_input = r;
        self
    }
    pub fn with_search_input(mut self, r: Arc<dyn SearchInputRenderer>) -> Self {
        self.search_input = r;
        self
    }
    pub fn with_select(mut self, r: Arc<dyn SelectRenderer>) -> Self {
        self.select = r;
        self
    }
    pub fn with_combo_box(mut self, r: Arc<dyn ComboBoxRenderer>) -> Self {
        self.combo_box = r;
        self
    }
    pub fn with_disclosure(mut self, r: Arc<dyn DisclosureRenderer>) -> Self {
        self.disclosure = r;
        self
    }
    pub fn with_tree_item(mut self, r: Arc<dyn TreeItemRenderer>) -> Self {
        self.tree_item = r;
        self
    }
    pub fn with_keybinding_input(mut self, r: Arc<dyn KeybindingInputRenderer>) -> Self {
        self.keybinding_input = r;
        self
    }
    pub fn with_split_button(mut self, r: Arc<dyn SplitButtonRenderer>) -> Self {
        self.split_button = r;
        self
    }
    pub fn with_empty_state(mut self, r: Arc<dyn EmptyStateRenderer>) -> Self {
        self.empty_state = r;
        self
    }

    /// Generic component renderer setter (P0-4).
    ///
    /// Lets a theme package install a renderer for a render-state
    /// type without writing a `with_<x>` setter per component. The
    /// 40+ `with_<x>` setters above remain as thin wrappers for
    /// backward-compat; new code can use this entry point.
    ///
    /// The renderer's render-state type is the key. The registry
    /// downcasts `Arc<dyn Any>` back to `Arc<dyn ComponentRenderer<S>>`
    /// when components read it. Note: this is currently a
    /// documentation hook; the actual HashMap-based storage
    /// migration is staged for v0.4.1 — see ARCHITECTURE_GUIDE §
    /// "P0-4".
    pub fn with_component<S, R>(self, _renderer: Arc<R>) -> Self
    where
        S: super::component_renderer::RenderState,
        R: super::component_renderer::ComponentRenderer<S> + 'static,
    {
        // Stage 1: type-system only. Stage 2 (v0.4.1) will swap the
        // 40+ named `Arc<dyn …>` fields for a `HashMap<TypeId, Arc<dyn
        // Any>>` and route reads through it.
        self
    }

    /// Generic renderer lookup (P0-4 stage 1).
    ///
    /// Returns `None` until the storage migration is complete; the
    /// per-component `Arc<dyn …Renderer>` fields are still the
    /// source of truth. The method exists so call-sites can adopt
    /// the new API in advance and migrate to the HashMap-based
    /// lookup transparently when stage 2 ships.
    pub fn get_component<S: super::component_renderer::RenderState>(
        &self,
    ) -> Option<Arc<dyn super::component_renderer::ComponentRenderer<S>>> {
        None
    }
}
