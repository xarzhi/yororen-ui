//! `Theme` path helpers — small typed wrappers around
//! `Theme::get_color` / `get_number` that pick a path shape
//! the renderer crate actually consumes.
//!
//! The JSON schema is open; these helpers are the place where
//! the renderer makes a *concrete* choice about which keys to
//! read. If a different renderer wants different paths, it
//! writes its own helpers and ignores this file.
//!
//! Conventions used here:
//!
//! - `action.<variant>.<field>` — color slot for an action
//!   variant. `variant` is `neutral` / `primary` / `danger`;
//!   `field` is e.g. `bg` / `hover_bg` / `fg` / `disabled_bg`.
//! - `surface.<field>` — base canvas colors.
//! - `content.<field>` — text colors.
//! - `border.<field>` — border colors.
//! - `status.<kind>.{bg,fg}` — status pill colors.
//! - `shadow.<field>` — elevation shadow color.
//! - `tokens.sizes.<field>` — generic sizes (control heights, icon sizes, avatar sizes).
//! - `tokens.radii.<key>` — radius scale.
//! - `tokens.spacing.<field>` — gap / inset scale.
//! - `tokens.typography.<field>` — font sizes, weights, families.
//! - `tokens.motion.<field>` — animation durations.
//! - `tokens.control.<component>.<field>` — component-specific
//!   geometry / size tokens.

use gpui::Hsla;
use serde_json::Value;

use yororen_ui_core::theme::Theme;

use super::button::ActionVariantKind;

// ---------------------------------------------------------------------------
// Color helpers
// ---------------------------------------------------------------------------

/// `action.<variant>.<field>` color. Returns `Hsla::default()` if
/// the path is missing — renderers that want a different
/// fallback should call `theme.get_color` directly.
pub fn action_color(theme: &Theme, variant: ActionVariantKind, field: &str) -> Hsla {
    let key = format!("action.{}.{}", variant.as_str(), field);
    theme.get_color(&key).unwrap_or_default()
}

/// `surface.<field>` color.
pub fn surface(theme: &Theme, field: &str) -> Hsla {
    let key = format!("surface.{}", field);
    theme.get_color(&key).unwrap_or_default()
}

/// `content.<field>` color.
pub fn content(theme: &Theme, field: &str) -> Hsla {
    let key = format!("content.{}", field);
    theme.get_color(&key).unwrap_or_default()
}

/// `border.<field>` color.
pub fn border(theme: &Theme, field: &str) -> Hsla {
    let key = format!("border.{}", field);
    theme.get_color(&key).unwrap_or_default()
}

/// `status.<kind>.{bg|fg}` color.
pub fn status(theme: &Theme, kind: &str, field: &str) -> Hsla {
    let key = format!("status.{}.{}", kind, field);
    theme.get_color(&key).unwrap_or_default()
}

/// `shadow.<field>` color.
pub fn shadow(theme: &Theme, field: &str) -> Hsla {
    let key = format!("shadow.{}", field);
    theme.get_color(&key).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Number helpers (return `Option<f64>`; renderers convert to `Pixels` etc.)
// ---------------------------------------------------------------------------

fn get_num(theme: &Theme, path: &str) -> Option<f64> {
    theme.get_number(path)
}

pub fn control_button(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.button.{}", field))
}
pub fn control_input(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.input.{}", field))
}
pub fn control_switch(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.switch.{}", field))
}
pub fn control_checkbox(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.checkbox.{}", field))
}
pub fn control_radio(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.radio.{}", field))
}
pub fn control_select(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.select.{}", field))
}
pub fn control_combo_box(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.combo_box.{}", field))
}
pub fn control_slider(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.slider.{}", field))
}
pub fn control_toast(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.toast.{}", field))
}
pub fn control_modal(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.modal.{}", field))
}
pub fn control_popover(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.popover.{}", field))
}
pub fn control_dropdown(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.dropdown.{}", field))
}
pub fn control_badge(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.badge.{}", field))
}
pub fn control_tag(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.tag.{}", field))
}
pub fn control_skeleton(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.skeleton.{}", field))
}
pub fn control_progress(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.progress.{}", field))
}
pub fn control_avatar(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.avatar.{}", field))
}
pub fn control_tooltip(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.tooltip.{}", field))
}
pub fn control_disclosure(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.disclosure.{}", field))
}
pub fn control_keybinding_input(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.keybinding_input.{}", field))
}
pub fn control_split_button(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.split_button.{}", field))
}
pub fn control_search_input(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.search_input.{}", field))
}
pub fn control_number_input(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.number_input.{}", field))
}
pub fn control_file_path_input(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.file_path_input.{}", field))
}
pub fn control_icon_button(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.icon_button.{}", field))
}
pub fn control_toggle_button(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.toggle_button.{}", field))
}
pub fn control_empty_state(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.empty_state.{}", field))
}
pub fn control_list_item(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.list_item.{}", field))
}
pub fn control_tree_item(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.tree_item.{}", field))
}
pub fn control_card(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.card.{}", field))
}
pub fn control_divider(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.divider.{}", field))
}
pub fn control_form(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.form.{}", field))
}
pub fn control_notification(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.notification.{}", field))
}
pub fn control_focus_ring(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.control.focus_ring.{}", field))
}

pub fn sizes(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.sizes.{}", field))
}
pub fn radii(theme: &Theme, key: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.radii.{}", key))
}
pub fn radii_md(theme: &Theme) -> Option<f64> {
    radii(theme, "md")
}
pub fn spacing(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.spacing.{}", field))
}
pub fn typography_num(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.typography.{}", field))
}
pub fn motion(theme: &Theme, field: &str) -> Option<f64> {
    get_num(theme, &format!("tokens.motion.{}", field))
}

// ---------------------------------------------------------------------------
// String helpers
// ---------------------------------------------------------------------------

/// `tokens.typography.family_<key>` (e.g. `family_mono`,
/// `family_default`). Returns `None` for missing paths so the
/// caller can fall back to a hard-coded string.
pub fn typography_string(theme: &Theme, field: &str) -> Option<String> {
    let key = format!("tokens.typography.{}", field);
    theme.get(&key).and_then(|v| match v {
        Value::String(s) => Some(s.clone()),
        _ => None,
    })
}
