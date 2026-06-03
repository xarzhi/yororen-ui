//! Accessibility (A11y) module for Yororen UI.
//!
//! This module provides accessibility utilities including:
//! - ARIA role and attribute definitions
//! - Focus management components (FocusTrap)
//! - Click-outside detection (`ClickOutsideGuard`,
//!   `ClickOutsideCapture`, `on_click_outside`)
//! - Body scroll lock (`ScrollLockGuard`)
//! - Keyboard navigation helpers

mod aria;
mod click_outside;
mod focus_trap;
mod scroll_lock;

pub use aria::*;
pub use click_outside::*;
pub use focus_trap::*;
pub use scroll_lock::*;
