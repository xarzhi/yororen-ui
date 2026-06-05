//! Headless notification center.
//!
//! The visual toast stack that draws incoming notifications lives in
//! `yororen-ui-renderer::notification::host`; this module is the
//! pure data + state machine (`Notification`, `NotificationId`,
//! `NotificationCenter`, `DismissStrategy`).

pub mod center;

#[doc = include_str!("README.md")]
pub mod docs {}

pub use center::*;
