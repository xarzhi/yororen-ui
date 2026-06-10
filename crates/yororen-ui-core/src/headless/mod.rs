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
//! button("save", cx).on_click(...).render(cx)
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
pub mod combo_box;
pub mod context_menu_trigger;
pub mod disclosure;
pub mod divider;
pub mod drag_handle;
pub mod dropdown_menu;
pub mod empty_state;
pub mod file_path_input;
pub mod focus_ring;
pub mod form;
pub mod form_field;
pub mod heading;
pub mod icon;
pub mod icon_button;
pub mod image;
pub mod keybinding_display;
pub mod keybinding_input;
pub mod label;
pub mod list_item;
pub mod listbox;
pub mod menu;
pub mod modal;
pub mod number_input;
pub mod overlay;
pub mod panel;
pub mod password_input;
pub mod popover;
pub mod progress;
pub mod radio;
pub mod radio_group;
pub mod search_input;
pub mod select;
pub mod shortcut_hint;
pub mod skeleton;
pub mod slider;
pub mod spacer;
pub mod split_button;
pub mod switch;
pub mod table;
pub mod tag;
pub mod text;
pub mod text_area;
pub mod text_area_element;
pub mod text_input;
pub mod text_input_element;
pub mod toggle_button;
pub mod tooltip;
pub mod tree;
pub mod tree_item;
pub mod virtual_list;

// Re-export each component's factory function at the top
// level so callers can write `use yororen_ui::headless::button;`
// and get the `button(id, cx)` function (not the module).
pub use avatar::avatar;
pub use badge::badge;
pub use button::button;
pub use checkbox::checkbox;
pub use combo_box::combo_box;
pub use divider::divider;
pub use heading::heading;
pub use icon::icon;
pub use icon_button::icon_button;
pub use label::label;
pub use list_item::list_item;
pub use progress::progress;
pub use radio::radio;
pub use skeleton::skeleton;
pub use slider::slider;
pub use switch::switch;
pub use tag::tag;
pub use text::text;
pub use text_input::text_input;
pub use toggle_button::toggle_button;

// Marker types (for `cx.renderer_arc::<markers::Button, ...>`).
// These are the *component* markers, not the `headless::*Props`
// types — re-exporting them from `headless` keeps the typical
// `use yororen_ui::headless::*` pattern in app code.
pub use crate::renderer::markers;
// `ButtonProps`, `LabelProps`, etc. (the props structs
// returned by the factory functions) are not re-exported
// at the top level because callers usually don't need to
// name the struct — `.apply(div)` consumes the value.
// Power users can import the struct explicitly via
// `use yororen_ui::headless::button::ButtonProps;`.
