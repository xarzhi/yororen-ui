//! Default renderer for [`yororen_ui_core`] headless primitives.
//!
//! This crate holds the *visual* layer of yororen-ui — the `XxxRenderer`
//! trait fleet, the `DesignTokens` token tree, the `Theme` and
//! `GlobalTheme` types, and the `XxxPropsExt` extension traits that
//! render a headless `XxxProps` into a default-styled `Stateful<Div>`
//! using the currently installed `GlobalTheme`.
//!
//! It depends on `yororen-ui-core` for the headless `XxxProps` shapes
//! and on `gpui-ce` for the underlying element/window primitives. It
//! does **not** know about any concrete palette — that lives in the
//! `yororen-ui-theme-*` packages, which depend on this crate.
//!
//! Three-layer architecture:
//!
//! ```text
//! theme-* ──▶ renderer ──▶ core ──▶ gpui-ce
//! ```
//!
//! Apps that want a stock look depend on `yororen-ui` (which
//! re-exports `core + renderer + theme-system`) or on the
//! `renderer` + a `theme-*` crate of their choice. Apps that want
//! full visual control depend on `yororen-ui-core` only.

#![warn(missing_docs)]

pub mod renderers;
pub mod theme;

pub use theme::{GlobalTheme, Theme};
