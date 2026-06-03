//! Global state for the modal a11y demo.

use gpui::{App, AppContext, Entity, Global};

use yororen_ui::component::OverlayCloseReason;

/// Visibility flags for the three modal types the demo exercises.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ModalVisibility {
    pub standard: bool,
    pub required: bool,
    pub no_scroll_lock: bool,
}

pub struct ModalA11yState {
    pub visibility: Entity<ModalVisibility>,
    /// Number of times the standard modal was closed, by reason
    /// (used for a small stats label under the buttons).
    pub close_log: Entity<CloseLog>,
}

/// A small ring buffer of `OverlayCloseReason` values for the
/// "recent closes" list at the bottom of the demo.
#[derive(Clone, Debug, Default)]
pub struct CloseLog {
    pub entries: Vec<(OverlayCloseReason, std::time::Instant)>,
}

impl CloseLog {
    pub fn push(&mut self, reason: OverlayCloseReason) {
        self.entries.push((reason, std::time::Instant::now()));
        if self.entries.len() > 8 {
            let drop_n = self.entries.len() - 8;
            self.entries.drain(0..drop_n);
        }
    }
}

impl Global for ModalA11yState {}

impl ModalA11yState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            visibility: cx.new(|_| ModalVisibility::default()),
            close_log: cx.new(|_| CloseLog::default()),
        }
    }

    pub fn visibility(&self, cx: &gpui::App) -> ModalVisibility {
        *self.visibility.read(cx)
    }

    /// Convenience: update the visibility entity. `f` runs against
    /// the inner `ModalVisibility` value; callers don't need to
    /// thread the `Context` through.
    #[allow(dead_code)]
    pub fn set_visibility(&self, cx: &mut gpui::App, f: impl FnOnce(&mut ModalVisibility)) {
        self.visibility.update(cx, |v, _cx| f(v));
    }

    /// Append a close reason to the global close log.
    #[allow(dead_code)]
    pub fn push_close(&self, cx: &mut gpui::App, reason: OverlayCloseReason) {
        self.close_log.update(cx, |log, _cx| log.push(reason));
    }
}
