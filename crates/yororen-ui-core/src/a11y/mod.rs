//! Accessibility (A11y) module for Yororen UI.
//!
//! This module provides accessibility utilities including:
//! - Focus management components (`FocusTrap`, `FocusTrapState`)
//! - Click-outside detection (`ClickOutsideGuard`,
//!   `ClickOutsideCapture`, `on_click_outside`)
//! - Body scroll lock (`ScrollLockGuard`)
//! - Keyboard navigation helpers (`FocusableList`, `cycle_focus`,
//!   `FocusRing`)

mod click_outside;
mod focus_trap;
mod keyboard_nav;
mod scroll_lock;

pub use click_outside::*;
pub use focus_trap::*;
pub use keyboard_nav::*;
pub use scroll_lock::*;
