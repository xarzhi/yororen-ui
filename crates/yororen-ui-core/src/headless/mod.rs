//! Headless primitives for yororen-ui.
//!
//! Every function in this module returns a `XxxProps` struct that
//! describes the *behavior* of a UI element (focus, click, key
//! dispatch, internal state). The caller provides the *visual* by
//! composing the props with a `div()` via `.apply(props)`:
//!
//! ```ignore
//! use yororen_ui::headless::button;
//! use yororen_ui_renderer::DefaultButton;
//!
//! // Pure headless — caller chooses every visual:
//! div().bg(red).rounded(8).apply(button("save", cx).on_click(...)).child("Save")
//!
//! // Default rendered — uses the installed GlobalTheme:
//! button("save", cx).on_click(...).default_render(cx)
//! ```
//!
//! The `headless/` module is the **only** way to construct
//! interactive elements. There is no "pre-rendered `Button`" struct
//! in `yororen-ui-core`; the visual lives in the optional
//! `yororen-ui-renderer` crate, and the palette lives in the
//! `yororen-ui-theme-*` crates.
//!
//! ## Composites
//!
//! Multi-part components (popover, modal, select, dropdown menu,
//! tooltip, …) own a piece of state on a `gpui::Entity`. Callers
//! create the entity with `cx.new(|_| XxxState::new())`, read/write
//! it via the returned `Entity<XxxState>`, and ask the renderer to
//! produce a default-styled view. There is no pre-baked
//! `Modal` / `Popover` struct that builds its own trigger and
//! content divs — the caller passes them in.

pub mod avatar;
pub mod badge;
pub mod button;
pub mod button_group;
pub mod card;
pub mod checkbox;
pub mod clickable_surface;
pub mod context_menu_trigger;
pub mod disclosure;
pub mod divider;
pub mod drag_handle;
pub mod empty_state;
pub mod focus_ring;
pub mod form;
pub mod form_field;
pub mod heading;
pub mod icon;
pub mod icon_button;
pub mod image;
pub mod keybinding_display;
pub mod label;
pub mod list_item;
pub mod overlay;
pub mod panel;
pub mod progress;
pub mod radio;
pub mod radio_group;
pub mod shortcut_hint;
pub mod skeleton;
pub mod slider;
pub mod spacer;
pub mod split_button;
pub mod switch;
pub mod table;
pub mod tag;
pub mod text;
pub mod text_input;
pub mod toggle_button;
pub mod tree;
pub mod tree_item;
pub mod virtual_list;
