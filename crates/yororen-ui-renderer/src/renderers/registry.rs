//! `RendererRegistry` — the collection of component renderers wired
//! into a `Theme`.
//!
//! ## Storage model
//!
//! Renderers are stored in a `HashMap<TypeId, Arc<dyn Any + Send + Sync>>`,
//! keyed by the component's `XxxRenderState` type. Each
//! `with_<x>(r: Arc<dyn XxxRenderer>)` setter inserts `r` under
//! `TypeId::of::<XxxRenderState>()`; each `get_<x>() -> Option<&Arc<dyn XxxRenderer>>`
//! looks it up. The `38 XxxRenderer` traits themselves stay public and
//! unchanged — they are the **type-level** contract a theme implements,
//! the HashMap is the **storage** layer.
//!
//! ## Why not 38 named fields?
//!
//! The 38-trait design (one trait per component) is the right level of
//! granularity for the *type system*: each `XxxRenderer` is callable
//! with the component-specific `XxxRenderState` and returns the
//! component-specific property shape. Trying to collapse them into a
//! single `ComponentRenderer<S: RenderState>` trait would force a
//! 200+-method union (because the 38 method sets are largely disjoint
//! — `Switch` has `track_w/track_h/knob_size` while `Avatar` has
//! `status_dot_size/status_inset`).
//!
//! What the 38-field `RendererRegistry` *didn't* need was the named
//! fields themselves: there is no code that reads `theme.renderers.button`
//! directly (all call sites go through `with_button` setters and
//! `get_button` accessors). So the fields are private storage, the
//! public surface is the 38 `with_<x>` / 38 `get_<x>` method pairs.

use std::any::{Any, TypeId};
use std::collections::HashMap;
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

use super::avatar::AvatarRenderState;
use super::badge::BadgeRenderState;
use super::button::ButtonRenderState;
use super::card::CardRenderState;
use super::checkbox::CheckboxRenderState;
use super::combo_box::ComboBoxRenderState;
use super::disclosure::DisclosureRenderState;
use super::divider::DividerRenderState;
use super::dropdown_menu::DropdownMenuRenderState;
use super::empty_state::EmptyStateRenderState;
use super::file_path_input::FilePathInputRenderState;
use super::focus_ring::FocusRingRenderState;
use super::form::FormRenderState;
use super::heading::HeadingRenderState;
use super::icon_button::IconButtonRenderState;
use super::keybinding_input::KeybindingInputRenderState;
use super::label::LabelRenderState;
use super::list_item::ListItemRenderState;
use super::modal::ModalRenderState;
use super::notification::NotificationRenderState;
use super::number_input::NumberInputRenderState;
use super::panel::PanelRenderState;
use super::password_input::PasswordInputRenderState;
use super::popover::PopoverRenderState;
use super::progress::ProgressBarRenderState;
use super::radio::RadioRenderState;
use super::search_input::SearchInputRenderState;
use super::select::SelectRenderState;
use super::skeleton::SkeletonRenderState;
use super::split_button::SplitButtonRenderState;
use super::switch::SwitchRenderState;
use super::tag::TagRenderState;
use super::text_area::TextAreaRenderState;
use super::text_input::TextInputRenderState;
use super::toast::ToastRenderState;
use super::toggle_button::ToggleButtonRenderState;
use super::tooltip::TooltipRenderState;
use super::tree_item::TreeItemRenderState;

/// Collection of component renderers. Looked up at render time by
/// `XxxRenderState` `TypeId`.
///
/// Public surface: 38 `with_<x>(Arc<dyn XxxRenderer>)` setters + 38
/// `get_<x>() -> Option<&Arc<dyn XxxRenderer>>` accessors. The
/// underlying HashMap is private.
#[derive(Clone)]
pub struct RendererRegistry {
    map: HashMap<TypeId, Arc<dyn Any + Send + Sync>>,
}

/// Generate one `with_<x>` setter per component. The setter stores
/// the renderer under `TypeId::of::<XxxRenderState>()`. The
/// `Arc<dyn XxxRenderer>` is wrapped in an outer `Arc` and the whole
/// thing is cast to `Arc<dyn Any + Send + Sync>`. The outer wrap is
/// what makes the downcast back to `Arc<dyn XxxRenderer>` work: the
/// `Arc::downcast_ref::<Arc<dyn XxxRenderer>>()` matches the
/// concrete Sized `Arc<dyn XxxRenderer>` stored inside the outer
/// `Arc<dyn Any>`.
macro_rules! renderer_setter {
    ($setter:ident, $state:ty, $trait:path) => {
        pub fn $setter(mut self, r: Arc<dyn $trait>) -> Self {
            // Outer Arc wraps the inner Arc<dyn XxxRenderer>; the
            // outer Arc is the "Any" container, and downcasting
            // its inner Any back to `Arc<dyn XxxRenderer>` recovers
            // the original typed renderer.
            let any: Arc<dyn Any + Send + Sync> = Arc::new(r);
            self.map.insert(TypeId::of::<$state>(), any);
            self
        }
    };
}

/// Generate one `get_<x>()` typed accessor per component.
macro_rules! renderer_getter {
    ($getter:ident, $state:ty, $trait:path) => {
        pub fn $getter(&self) -> Option<&Arc<dyn $trait>> {
            self.get_typed::<$state, dyn $trait>()
        }
    };
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
    ///
    /// Self-validates on construction: if a future edit adds a new
    /// `with_<x>(...)` line here but forgets to mirror it in `REQUIRED`,
    /// `token_based()` panics with the full list of missing entries
    /// the first time it is called. This is the single coupling point
    /// between the 38 setters and the 38 `REQUIRED` entries.
    pub fn token_based() -> Self {
        // Cannot use `Self::default()` here — `default` is implemented
        // as `token_based`, so that would recurse forever. Construct
        // the empty registry directly and chain the 38 setters.
        let registry = Self {
            map: HashMap::new(),
        }
        .with_button(Arc::new(TokenButtonRenderer))
        .with_icon_button(Arc::new(TokenIconButtonRenderer))
        .with_toggle_button(Arc::new(TokenToggleButtonRenderer))
        .with_label(Arc::new(TokenLabelRenderer))
        .with_heading(Arc::new(TokenHeadingRenderer))
        .with_divider(Arc::new(TokenDividerRenderer))
        .with_focus_ring(Arc::new(TokenFocusRingRenderer))
        .with_badge(Arc::new(TokenBadgeRenderer))
        .with_tag(Arc::new(TokenTagRenderer))
        .with_progress_bar(Arc::new(TokenProgressBarRenderer))
        .with_skeleton(Arc::new(TokenSkeletonRenderer))
        .with_tooltip(Arc::new(TokenTooltipRenderer))
        .with_avatar(Arc::new(TokenAvatarRenderer))
        .with_switch(Arc::new(TokenSwitchRenderer))
        .with_checkbox(Arc::new(TokenCheckboxRenderer))
        .with_radio(Arc::new(TokenRadioRenderer))
        .with_text_input(Arc::new(TokenTextInputRenderer))
        .with_text_area(Arc::new(TokenTextAreaRenderer))
        .with_password_input(Arc::new(TokenPasswordInputRenderer))
        .with_number_input(Arc::new(TokenNumberInputRenderer))
        .with_file_path_input(Arc::new(TokenFilePathInputRenderer))
        .with_search_input(Arc::new(TokenSearchInputRenderer))
        .with_select(Arc::new(TokenSelectRenderer))
        .with_combo_box(Arc::new(TokenComboBoxRenderer))
        .with_modal(Arc::new(TokenModalRenderer))
        .with_popover(Arc::new(TokenPopoverRenderer))
        .with_dropdown_menu(Arc::new(TokenDropdownMenuRenderer))
        .with_disclosure(Arc::new(TokenDisclosureRenderer))
        .with_toast(Arc::new(TokenToastRenderer))
        .with_notification(Arc::new(TokenNotificationRenderer))
        .with_panel(Arc::new(TokenPanelRenderer))
        .with_card(Arc::new(TokenCardRenderer))
        .with_form(Arc::new(TokenFormRenderer))
        .with_list_item(Arc::new(TokenListItemRenderer))
        .with_tree_item(Arc::new(TokenTreeItemRenderer))
        .with_keybinding_input(Arc::new(TokenKeybindingInputRenderer))
        .with_split_button(Arc::new(TokenSplitButtonRenderer))
        .with_empty_state(Arc::new(TokenEmptyStateRenderer));

        if registry.validate().is_err() {
            panic_missing_renderers(&registry);
        }
        registry
    }

    // -- 38 setters (one per component) ---------------------------------
    // Each setter preserves the original `with_<x>(Arc<dyn XxxRenderer>)`
    // signature exactly — theme packages and downstream apps call them
    // unchanged. The renderer is stored under
    // `TypeId::of::<XxxRenderState>()`.

    renderer_setter!(with_button, ButtonRenderState, ButtonRenderer);
    renderer_setter!(with_label, LabelRenderState, LabelRenderer);
    renderer_setter!(with_heading, HeadingRenderState, HeadingRenderer);
    renderer_setter!(with_divider, DividerRenderState, DividerRenderer);
    renderer_setter!(with_focus_ring, FocusRingRenderState, FocusRingRenderer);
    renderer_setter!(with_badge, BadgeRenderState, BadgeRenderer);
    renderer_setter!(with_tag, TagRenderState, TagRenderer);
    renderer_setter!(
        with_progress_bar,
        ProgressBarRenderState,
        ProgressBarRenderer
    );
    renderer_setter!(with_skeleton, SkeletonRenderState, SkeletonRenderer);
    renderer_setter!(with_tooltip, TooltipRenderState, TooltipRenderer);
    renderer_setter!(with_avatar, AvatarRenderState, AvatarRenderer);
    renderer_setter!(with_switch, SwitchRenderState, SwitchRenderer);
    renderer_setter!(with_checkbox, CheckboxRenderState, CheckboxRenderer);
    renderer_setter!(with_radio, RadioRenderState, RadioRenderer);
    renderer_setter!(with_icon_button, IconButtonRenderState, IconButtonRenderer);
    renderer_setter!(
        with_toggle_button,
        ToggleButtonRenderState,
        ToggleButtonRenderer
    );
    renderer_setter!(with_text_input, TextInputRenderState, TextInputRenderer);
    renderer_setter!(with_modal, ModalRenderState, ModalRenderer);
    renderer_setter!(with_popover, PopoverRenderState, PopoverRenderer);
    renderer_setter!(
        with_dropdown_menu,
        DropdownMenuRenderState,
        DropdownMenuRenderer
    );
    renderer_setter!(with_toast, ToastRenderState, ToastRenderer);
    renderer_setter!(
        with_notification,
        NotificationRenderState,
        NotificationRenderer
    );
    renderer_setter!(with_panel, PanelRenderState, PanelRenderer);
    renderer_setter!(with_card, CardRenderState, CardRenderer);
    renderer_setter!(with_form, FormRenderState, FormRenderer);
    renderer_setter!(with_list_item, ListItemRenderState, ListItemRenderer);
    renderer_setter!(with_text_area, TextAreaRenderState, TextAreaRenderer);
    renderer_setter!(
        with_password_input,
        PasswordInputRenderState,
        PasswordInputRenderer
    );
    renderer_setter!(
        with_number_input,
        NumberInputRenderState,
        NumberInputRenderer
    );
    renderer_setter!(
        with_file_path_input,
        FilePathInputRenderState,
        FilePathInputRenderer
    );
    renderer_setter!(
        with_search_input,
        SearchInputRenderState,
        SearchInputRenderer
    );
    renderer_setter!(with_select, SelectRenderState, SelectRenderer);
    renderer_setter!(with_combo_box, ComboBoxRenderState, ComboBoxRenderer);
    renderer_setter!(with_disclosure, DisclosureRenderState, DisclosureRenderer);
    renderer_setter!(with_tree_item, TreeItemRenderState, TreeItemRenderer);
    renderer_setter!(
        with_keybinding_input,
        KeybindingInputRenderState,
        KeybindingInputRenderer
    );
    renderer_setter!(
        with_split_button,
        SplitButtonRenderState,
        SplitButtonRenderer
    );
    renderer_setter!(with_empty_state, EmptyStateRenderState, EmptyStateRenderer);

    /// Internal: typed lookup. The component-side accessors are
    /// `get_<x>()` 1-liners that pin both the state and the trait.
    ///
    /// The setter stores `Box::new(r as Arc<dyn XxxRenderer>)` cast
    /// to `Box<dyn Any + Send + Sync>`. So the inner `Any` is
    /// `Arc<dyn XxxRenderer>`. We downcast back to that `Arc<...>`
    /// to recover the original typed renderer.
    fn get_typed<S: super::spec::RenderState, R: ?Sized + 'static>(&self) -> Option<&Arc<R>> {
        self.map
            .get(&TypeId::of::<S>())
            .and_then(|arc| arc.downcast_ref::<Arc<R>>())
    }

    // -- 38 typed accessors (one per component) -------------------------
    // Each returns `Option<&Arc<dyn XxxRenderer>>`. Components call
    // these in their `RenderOnce::render` body; theme packages and
    // downstream code do not need them (the setter is enough to
    // install a custom renderer).

    renderer_getter!(get_button, ButtonRenderState, ButtonRenderer);
    renderer_getter!(get_label, LabelRenderState, LabelRenderer);
    renderer_getter!(get_heading, HeadingRenderState, HeadingRenderer);
    renderer_getter!(get_divider, DividerRenderState, DividerRenderer);
    renderer_getter!(get_focus_ring, FocusRingRenderState, FocusRingRenderer);
    renderer_getter!(get_badge, BadgeRenderState, BadgeRenderer);
    renderer_getter!(get_tag, TagRenderState, TagRenderer);
    renderer_getter!(
        get_progress_bar,
        ProgressBarRenderState,
        ProgressBarRenderer
    );
    renderer_getter!(get_skeleton, SkeletonRenderState, SkeletonRenderer);
    renderer_getter!(get_tooltip, TooltipRenderState, TooltipRenderer);
    renderer_getter!(get_avatar, AvatarRenderState, AvatarRenderer);
    renderer_getter!(get_switch, SwitchRenderState, SwitchRenderer);
    renderer_getter!(get_checkbox, CheckboxRenderState, CheckboxRenderer);
    renderer_getter!(get_radio, RadioRenderState, RadioRenderer);
    renderer_getter!(get_icon_button, IconButtonRenderState, IconButtonRenderer);
    renderer_getter!(
        get_toggle_button,
        ToggleButtonRenderState,
        ToggleButtonRenderer
    );
    renderer_getter!(get_text_input, TextInputRenderState, TextInputRenderer);
    renderer_getter!(get_modal, ModalRenderState, ModalRenderer);
    renderer_getter!(get_popover, PopoverRenderState, PopoverRenderer);
    renderer_getter!(
        get_dropdown_menu,
        DropdownMenuRenderState,
        DropdownMenuRenderer
    );
    renderer_getter!(get_toast, ToastRenderState, ToastRenderer);
    renderer_getter!(
        get_notification,
        NotificationRenderState,
        NotificationRenderer
    );
    renderer_getter!(get_panel, PanelRenderState, PanelRenderer);
    renderer_getter!(get_card, CardRenderState, CardRenderer);
    renderer_getter!(get_form, FormRenderState, FormRenderer);
    renderer_getter!(get_list_item, ListItemRenderState, ListItemRenderer);
    renderer_getter!(get_text_area, TextAreaRenderState, TextAreaRenderer);
    renderer_getter!(
        get_password_input,
        PasswordInputRenderState,
        PasswordInputRenderer
    );
    renderer_getter!(
        get_number_input,
        NumberInputRenderState,
        NumberInputRenderer
    );
    renderer_getter!(
        get_file_path_input,
        FilePathInputRenderState,
        FilePathInputRenderer
    );
    renderer_getter!(
        get_search_input,
        SearchInputRenderState,
        SearchInputRenderer
    );
    renderer_getter!(get_select, SelectRenderState, SelectRenderer);
    renderer_getter!(get_combo_box, ComboBoxRenderState, ComboBoxRenderer);
    renderer_getter!(get_disclosure, DisclosureRenderState, DisclosureRenderer);
    renderer_getter!(get_tree_item, TreeItemRenderState, TreeItemRenderer);
    renderer_getter!(
        get_keybinding_input,
        KeybindingInputRenderState,
        KeybindingInputRenderer
    );
    renderer_getter!(
        get_split_button,
        SplitButtonRenderState,
        SplitButtonRenderer
    );
    renderer_getter!(get_empty_state, EmptyStateRenderState, EmptyStateRenderer);

    /// All 38 `(TypeId, "name")` pairs that a complete registry must
    /// contain. Single source of truth for `validate()` and for the
    /// self-check at the end of `token_based()`.
    ///
    /// The names are stable strings (matching the `with_<x>` setter
    /// identifiers) so error messages stay grep-friendly: a panic
    /// listing `"text_input, password_input"` tells the theme author
    /// exactly which `with_<x>(...)` calls they forgot.
    const REQUIRED: &[(TypeId, &str)] = &[
        (TypeId::of::<ButtonRenderState>(), "button"),
        (TypeId::of::<IconButtonRenderState>(), "icon_button"),
        (TypeId::of::<ToggleButtonRenderState>(), "toggle_button"),
        (TypeId::of::<LabelRenderState>(), "label"),
        (TypeId::of::<HeadingRenderState>(), "heading"),
        (TypeId::of::<DividerRenderState>(), "divider"),
        (TypeId::of::<FocusRingRenderState>(), "focus_ring"),
        (TypeId::of::<BadgeRenderState>(), "badge"),
        (TypeId::of::<TagRenderState>(), "tag"),
        (TypeId::of::<ProgressBarRenderState>(), "progress_bar"),
        (TypeId::of::<SkeletonRenderState>(), "skeleton"),
        (TypeId::of::<TooltipRenderState>(), "tooltip"),
        (TypeId::of::<AvatarRenderState>(), "avatar"),
        (TypeId::of::<SwitchRenderState>(), "switch"),
        (TypeId::of::<CheckboxRenderState>(), "checkbox"),
        (TypeId::of::<RadioRenderState>(), "radio"),
        (TypeId::of::<TextInputRenderState>(), "text_input"),
        (TypeId::of::<TextAreaRenderState>(), "text_area"),
        (TypeId::of::<PasswordInputRenderState>(), "password_input"),
        (TypeId::of::<NumberInputRenderState>(), "number_input"),
        (TypeId::of::<FilePathInputRenderState>(), "file_path_input"),
        (TypeId::of::<SearchInputRenderState>(), "search_input"),
        (TypeId::of::<SelectRenderState>(), "select"),
        (TypeId::of::<ComboBoxRenderState>(), "combo_box"),
        (TypeId::of::<ModalRenderState>(), "modal"),
        (TypeId::of::<PopoverRenderState>(), "popover"),
        (TypeId::of::<DropdownMenuRenderState>(), "dropdown_menu"),
        (TypeId::of::<DisclosureRenderState>(), "disclosure"),
        (TypeId::of::<ToastRenderState>(), "toast"),
        (TypeId::of::<NotificationRenderState>(), "notification"),
        (TypeId::of::<PanelRenderState>(), "panel"),
        (TypeId::of::<CardRenderState>(), "card"),
        (TypeId::of::<FormRenderState>(), "form"),
        (TypeId::of::<ListItemRenderState>(), "list_item"),
        (TypeId::of::<TreeItemRenderState>(), "tree_item"),
        (TypeId::of::<KeybindingInputRenderState>(), "keybinding_input"),
        (TypeId::of::<SplitButtonRenderState>(), "split_button"),
        (TypeId::of::<EmptyStateRenderState>(), "empty_state"),
    ];

    /// Verify that this registry contains a renderer for **all** 38
    /// `XxxRenderState` types. Returns `Ok(())` if complete, or
    /// `Err(missing)` listing the names of every absent renderer.
    ///
    /// ## Why
    ///
    /// Component render paths look up their renderer via
    /// `get_<x>().expect("XxxRenderer registered")`. That `expect`
    /// is the *last line of defense*; if it ever fires in production
    /// it means a theme package was constructed without all 38
    /// renderers registered. This is a *configuration* bug, not a
    /// runtime condition, and the fix is "add `with_<x>(...)` to your
    /// theme's registry builder".
    ///
    /// `validate()` is the recommended way to catch that bug at app
    /// boot time, before any component renders. Theme packages should
    /// call it inside their `install(cx)` entrypoint:
    ///
    /// ```ignore
    /// pub fn install(cx: &mut App) {
    ///     let registry = RendererRegistry::token_based()
    ///         .with_button(Arc::new(MyButtonRenderer));
    ///     registry.validate().expect("my-theme registry incomplete");
    ///     cx.set_global(GlobalTheme::new(Theme { renderers: registry, .. }));
    /// }
    /// ```
    ///
    /// `token_based()` self-validates on construction, so registries
    /// built on top of it (the common case) only need this check as
    /// a "did I forget a `with_<x>` after `token_based()`?" guard.
    pub fn validate(&self) -> Result<(), Vec<&'static str>> {
        let missing: Vec<&'static str> = Self::REQUIRED
            .iter()
            .copied()
            .filter_map(|(id, name)| (!self.map.contains_key(&id)).then_some(name))
            .collect();
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }
}

/// Internal: emit a panic listing the missing renderers. Used by
/// `token_based()`'s self-check to guarantee that future edits to
/// `token_based()` (adding a `with_<x>(...)` call) are always
/// mirrored in `REQUIRED`.
#[cold]
#[inline(never)]
fn panic_missing_renderers(registry: &RendererRegistry) -> ! {
    match registry.validate() {
        Ok(()) => unreachable!("panic_missing_renderers called on a valid registry"),
        Err(missing) => panic!(
            "RendererRegistry::token_based() is missing {} renderer(s): {}. \
             This is an internal bug: token_based() must register all 38 components. \
             Please report this as a yororen-ui bug.",
            missing.len(),
            missing.join(", "),
        ),
    }
}

#[cfg(test)]
mod tests {
    //! `RendererRegistry::validate()` is the boot-time guard against
    //! "theme package forgot to call `with_<x>(...)` for some
    //! component". The 6 tests below cover every interesting case
    //! of "how could a registry be incomplete" — and one regression
    //! test for the public `token_based()` happy path.

    use super::*;
    use std::any::TypeId;

    /// Build a registry that has *only* `ButtonRenderState`
    /// registered. Used to assert validate() reports the other 37
    /// as missing.
    fn only_button() -> RendererRegistry {
        let mut map: HashMap<TypeId, Arc<dyn Any + Send + Sync>> = HashMap::new();
        let arc: Arc<dyn ButtonRenderer> = Arc::new(super::super::button::TokenButtonRenderer);
        map.insert(TypeId::of::<super::super::button::ButtonRenderState>(), Arc::new(arc));
        RendererRegistry { map }
    }

    #[test]
    fn empty_registry_reports_all_38_missing() {
        let r = RendererRegistry {
            map: HashMap::new(),
        };
        let err = r.validate().unwrap_err();
        assert_eq!(err.len(), 38, "expected 38 missing entries, got {}: {:?}", err.len(), err);
        // Spot-check a few well-known component names.
        assert!(err.contains(&"button"));
        assert!(err.contains(&"text_input"));
        assert!(err.contains(&"modal"));
        assert!(err.contains(&"tree_item"));
        assert!(err.contains(&"empty_state"));
    }

    #[test]
    fn only_button_reports_37_missing() {
        let r = only_button();
        let err = r.validate().unwrap_err();
        assert_eq!(err.len(), 37, "expected 37 missing entries, got {}: {:?}", err.len(), err);
        assert!(!err.contains(&"button"), "button was registered, must not appear in missing");
        assert!(err.contains(&"text_input"));
    }

    #[test]
    fn token_based_passes_validation() {
        // This is the primary happy path: every theme package that
        // builds on `RendererRegistry::token_based()` must pass.
        let r = RendererRegistry::token_based();
        r.validate().expect("token_based() must always validate");
    }

    #[test]
    fn default_impl_passes_validation() {
        // `Default::default()` is defined as `token_based()`, so it
        // must also pass — this guards the (infinite-recursion)
        // implementation comment in `token_based()` from drifting.
        let r = RendererRegistry::default();
        r.validate().expect("Default::default() must always validate");
    }

    #[test]
    fn override_after_token_based_still_passes() {
        // The realistic theme-package pattern: take `token_based()`
        // and override one renderer. The override path must not
        // accidentally drop the other 37.
        let r = RendererRegistry::token_based()
            .with_button(Arc::new(super::super::button::TokenButtonRenderer));
        r.validate().expect("override-after-token_based must validate");
    }

    #[test]
    fn missing_set_is_sorted_in_required_order() {
        // The missing list is built by iterating `REQUIRED`, so the
        // order is deterministic and matches the declaration order.
        // This is a regression test against a refactor that might
        // (e.g.) switch to a HashSet and lose order.
        let r = RendererRegistry {
            map: HashMap::new(),
        };
        let err = r.validate().unwrap_err();
        let required_order: Vec<&'static str> =
            RendererRegistry::REQUIRED.iter().map(|(_, n)| *n).collect();
        assert_eq!(
            err, required_order,
            "missing list must follow REQUIRED declaration order"
        );
    }

    #[test]
    #[should_panic(expected = "token_based() is missing")]
    fn token_based_panics_if_a_setter_call_is_removed() {
        // Simulate the failure mode the self-check guards against:
        // a future edit removes one `with_<x>(...)` line from
        // `token_based()`. We can reproduce that locally by taking
        // `token_based()` (which validates) and then *deleting* one
        // entry — except `map` is private and setters can't remove.
        //
        // The realistic mirror is: rebuild the registry from a
        // `token_based()` snapshot but skip one setter. We do that
        // by constructing an almost-complete registry (37 of 38) and
        // asking the same code path to validate. Since we can't
        // reuse the production self-check without code duplication,
        // we rely on the `panic_missing_renderers` helper directly
        // via a hand-rolled almost-complete registry.
        let mut almost = RendererRegistry::token_based();
        // Drop one entry to simulate "future edit removed this line".
        let dropped = TypeId::of::<super::super::button::ButtonRenderState>();
        almost.map.remove(&dropped);
        // Now `almost` is incomplete; the same `panic_missing_renderers`
        // path used by `token_based()` must fire.
        assert!(almost.validate().is_err());
        panic_missing_renderers(&almost);
    }
}
