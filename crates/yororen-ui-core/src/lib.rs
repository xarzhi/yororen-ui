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
//! - [`renderer`] — `RendererRegistry` + `cx.register_renderer_arc` /
//!   `cx.renderer_arc` API for third-party renderer crates.
//! - [`theme`] — the JSON-backed `Theme` (no schema) + `cx.theme()`
//!   accessor.
//! - [`rtl`] — `TextDirection` plumbing shared with `i18n`.
//!
//! Anything visual (palette, geometry tokens, renderer traits) lives
//! in `yororen-ui-default-renderer`. Anything that *picks a
//! concrete palette* lives in a JSON theme file (`themes/*.json`).

// The 50+ headless modules each have their own doc comments
// at the module level. The `XxxRenderState` struct fields and
// the `XxxProps` builder methods are mechanical and stay
// terse to keep the file readable; the meta-crate
// (`yororen-ui::headless::Xxx`) re-exports the public API
// with proper docs.
#![allow(missing_docs)]

pub mod a11y;
pub mod animation;
pub mod assets;
pub mod headless;
pub mod i18n;
pub mod notification;
pub mod renderer;
pub mod rtl;
pub mod theme;
