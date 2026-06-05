//! Headless core for yororen-ui.
//!
//! Three-layer split:
//!
//! ```text
//! theme-* ──▶ renderer ──▶ core ──▶ gpui-ce
//! ```
//!
//! This crate owns **only**:
//! - [`headless`] — pure data + state-machine `XxxProps` for every
//!   primitive, no visual decisions.
//! - [`a11y`] — accessibility helpers (label / description / focused).
//! - [`animation`] — animation state machine (no visuals).
//! - [`assets`] — embedded SVG icon set, surfaced via `gpui::AssetSource`.
//! - [`i18n`] — `I18n` global + `Locale` + translation lookups.
//! - [`notification`] — `NotificationCenter` state machine (the visual
//!   toast host lives in `yororen-ui-renderer`).
//! - [`rtl`] — `TextDirection` plumbing shared with `i18n`.
//!
//! Anything visual (palette, geometry tokens, renderer traits) lives
//! in `yororen-ui-renderer`. Anything that *picks a concrete palette*
//! lives in a `yororen-ui-theme-*` crate.

#![warn(missing_docs)]

pub mod a11y;
pub mod animation;
pub mod assets;
pub mod headless;
pub mod i18n;
pub mod notification;
pub mod renderer;
pub mod rtl;
pub mod theme;
