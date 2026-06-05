//! Default renderer for [`yororen_ui_core`] headless primitives.
//!
//! This crate holds the *visual* layer of yororen-ui: the 38
//! `XxxRenderer` traits, their `TokenXxxRenderer` default
//! implementations, the bundled `system-light.json` /
//! `system-dark.json` themes, and a one-call install helper.
//!
//! Each `renderers/<name>.rs` file is the home of three things:
//!
//! - the `XxxRenderer` trait (the contract a theme implements),
//! - the `TokenXxxRenderer` struct (the default implementation),
//! - the `DefaultXxx` extension trait (the `headless::XxxProps` →
//!   `Stateful<Div>` sugar that reads this renderer).
//!
//! Apps depend on this crate when they want a stock look.
//! Third-party renderer crates depend on `yororen-ui-core`
//! and call `cx.register_renderer_arc::<markers::X, dyn XxxRenderer>(...)`
//! at install time.
//!
//! Three-layer architecture:
//!
//! ```text
//! themes/*.json ──▶ default-renderer ──▶ core ──▶ gpui-ce
//! ```

// The renderer traits are documented at the public meta
// level (`yororen_ui::renderer::XxxRenderer`); the
// per-renderer file docstrings are kept terse here to avoid
// maintaining two copies. The 38 `TokenXxxRenderer` impl
// bodies are not user-facing and stay undocumented to keep
// the implementation files readable.

pub mod renderers;
pub mod themes;

pub use renderers::button::DefaultButton;
pub use renderers::checkbox::DefaultCheckbox;
pub use renderers::file_path_input::DefaultFilePathInput;
pub use renderers::icon_button::DefaultIconButton;
pub use renderers::keybinding_input::DefaultKeybindingInput;
pub use renderers::label::DefaultLabel;
pub use renderers::number_input::DefaultNumberInput;
pub use renderers::password_input::DefaultPasswordInput;
pub use renderers::radio::DefaultRadio;
pub use renderers::search_input::DefaultSearchInput;
pub use renderers::switch::DefaultSwitch;
pub use renderers::text_area::DefaultTextArea;
pub use renderers::text_input::DefaultTextInput;
pub use renderers::toggle_button::DefaultToggleButton;
pub use renderers::{
    ButtonRenderState, ButtonRenderer, IconButtonRenderState, IconButtonRenderer,
    LabelRenderState, LabelRenderer, ToggleButtonRenderState, ToggleButtonRenderer,
};
pub use yororen_ui_core::theme::Theme;
pub use themes::{install, install_with, register_default_renderers, system_dark, system_for, system_light};
