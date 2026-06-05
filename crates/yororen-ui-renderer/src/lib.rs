//! Default renderer for [`yororen_ui_core`] headless primitives.
//!
//! This crate holds the *visual* layer of yororen-ui — the 38
//! `XxxRenderer` traits and their `TokenXxxRenderer` default
//! implementations, the `DesignTokens` token tree, and the
//! `Theme` / `GlobalTheme` types.
//!
//! Each `renderers/<name>.rs` file is the home of three things:
//!
//! - the `XxxRenderer` trait (the contract a theme implements),
//! - the `TokenXxxRenderer` struct (the default implementation),
//! - the `DefaultXxx` extension trait (the `headless::XxxProps` →
//!   `Stateful<Div>` sugar that reads this renderer).
//!
//! Apps depend on this crate when they want a stock look. Apps that
//! want full visual control depend on `yororen-ui-core` only.
//!
//! Three-layer architecture:
//!
//! ```text
//! theme-* ──▶ renderer ──▶ core ──▶ gpui-ce
//! ```

#![warn(missing_docs)]

pub mod renderers;
pub mod theme;

pub use renderers::button::DefaultButton;
pub use renderers::checkbox::DefaultCheckbox;
pub use renderers::icon_button::DefaultIconButton;
pub use renderers::label::DefaultLabel;
pub use renderers::radio::DefaultRadio;
pub use renderers::switch::DefaultSwitch;
pub use renderers::text_input::DefaultTextInput;
pub use renderers::toggle_button::DefaultToggleButton;

pub use theme::{GlobalTheme, Theme};
