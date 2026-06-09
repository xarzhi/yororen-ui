//! Default renderer for [`yororen_ui_core`] headless primitives.
//!
//! This crate holds the *visual* layer of yororen-ui: the 38
//! `XxxRenderer` traits (re-exported from `yororen-ui-core`
//! since v0.3.6), their `TokenXxxRenderer` default
//! implementations, the bundled `system-light.json` /
//! `system-dark.json` themes, and a one-call install helper.
//!
//! Each `renderers/<name>.rs` file is the home of:
//!
//! - the `XxxRenderer` trait (re-exported from core),
//! - the `TokenXxxRenderer` struct (the default impl),
//! - the `DefaultXxx::render` dispatch trait. For
//!   state-minting inputs (text_input, password_input,
//!   number_input, search_input, file_path_input,
//!   keybinding_input, text_area) the `DefaultXxx::render`
//!   trait is what wires up the `TextInputState` and IME
//!   pipeline; for stateless components (button, label, etc.)
//!   the headless `XxxProps::render(cx)` inherent method
//!   already covers everything, but the trait is provided
//!   uniformly so all 38 components share the same API.
//!
//! Three-layer architecture:
//!
//! ```text
//! themes/*.json ──▶ default-renderer ──▶ core ──▶ gpui ce
//! ```
//!
//! Apps depend on this crate when they want a stock look.
//! Third-party renderer crates depend on `yororen-ui-core`
//! and call `cx.register_renderer_arc::<markers::X, dyn XxxRenderer>(...)`
//! at install time.

pub mod renderers;
pub mod themes;

pub use renderers::{
    ButtonRenderState, ButtonRenderer, IconButtonRenderState, IconButtonRenderer, LabelRenderState,
    LabelRenderer, ToggleButtonRenderState, ToggleButtonRenderer,
};
pub use renderers::{
    DefaultCheckbox, DefaultFilePathInput, DefaultIcon, DefaultIconButton, DefaultKeybindingInput,
    DefaultLabel, DefaultNumberInput, DefaultPasswordInput, DefaultRadio, DefaultSearchInput,
    DefaultSwitch, DefaultTextArea, DefaultTextInput, DefaultToggleButton,
};
pub use themes::{
    install, install_with, register_default_renderers, system_dark, system_for, system_light,
};
pub use yororen_ui_core::theme::Theme;
