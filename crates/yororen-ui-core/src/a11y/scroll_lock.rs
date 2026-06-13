//! Body scroll lock — the "you can't scroll the page behind a modal"
//! pattern. The mechanism is process-global (a counter) so nested
//! modals can each call `lock()` and only the outermost unlock
//! actually restores scroll.
//!
//! # Why a counter?
//!
//! Naive boolean flags break with nested modals: opening modal B
//! while modal A is open would set the flag again, then closing B
//! would clear it while A is still open and the page starts
//! scrolling. Using a counter:
//!
//! - `lock()` increments the counter.
//! - `unlock()` decrements it.
//! - The lock is "active" iff the counter > 0.
//! - Only the call that decrements the counter to 0 actually
//!   restores the previous state.
//!
//! This is the same shape React Aria / Radix use.

use std::sync::atomic::{AtomicUsize, Ordering};

/// Process-global scroll-lock counter. Incremented by
/// [`ScrollLock::lock`], decremented by [`ScrollLock::unlock`].
/// Lock is "active" iff the counter is non-zero.
static SCROLL_LOCK_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Returns `true` if at least one caller has called `lock()` and
/// has not yet released the lock. Idempotent across nested locks.
pub fn is_locked() -> bool {
    SCROLL_LOCK_COUNT.load(Ordering::Acquire) > 0
}

/// Returns the current lock count. Useful for diagnostics and
/// tests.
pub fn current_lock_count() -> usize {
    SCROLL_LOCK_COUNT.load(Ordering::Acquire)
}

/// Reset the lock counter to 0. **Test-only** — production code
/// should always pair `lock()` with `unlock()`.
#[cfg(test)]
pub fn reset_for_test() {
    SCROLL_LOCK_COUNT.store(0, Ordering::Release);
}

/// RAII-style scroll-lock guard. Locks on construction, unlocks on
/// drop. Drop the guard (or call [`release`](Self::release)) to
/// restore the previous state.
///
/// ```ignore
/// fn render_modal(&self, cx: &mut App) -> impl IntoElement {
///     let _lock = ScrollLock::acquire();
///     modal_content()
/// }
/// ```
///
/// For component code that already returns a single value, prefer
/// [`ScrollLockGuard::new`]. The `acquire` alias is provided for
/// symmetry with `acquire` / `release` patterns in the rest of the
/// crate.
#[derive(Debug)]
#[must_use = "Drop the guard to release the scroll lock. Use `forget()` to leak it intentionally."]
pub struct ScrollLockGuard {
    /// `true` if the guard is still holding the lock. Set to
    /// `false` by [`release`](Self::release) or
    /// [`forget`](Self::forget) so a subsequent drop is a no-op.
    armed: bool,
}

impl ScrollLockGuard {
    /// Acquire a new scroll lock. Increments the global counter.
    pub fn acquire() -> Self {
        SCROLL_LOCK_COUNT.fetch_add(1, Ordering::AcqRel);
        Self { armed: true }
    }

    /// Alias for [`acquire`](Self::acquire) for symmetry with the
    /// other "new" patterns in the codebase.
    pub fn new() -> Self {
        Self::acquire()
    }

    /// Release the lock early without waiting for drop. Idempotent:
    /// calling `release` twice is a no-op.
    pub fn release(mut self) {
        if self.armed {
            self.armed = false;
            let prev = SCROLL_LOCK_COUNT.fetch_sub(1, Ordering::AcqRel);
            debug_assert!(prev > 0, "scroll-lock underflow");
        }
        // self is consumed here; drop() will see armed=false.
    }

    /// Forget the lock without releasing. The global counter is
    /// left incremented. Use only if you intentionally want a
    /// permanent lock (e.g. test scenarios).
    pub fn forget(mut self) {
        self.armed = false;
    }
}

impl Default for ScrollLockGuard {
    fn default() -> Self {
        Self::acquire()
    }
}

impl Drop for ScrollLockGuard {
    fn drop(&mut self) {
        if self.armed {
            // fetch_sub returns the previous value; if it was 1
            // before, the counter is now 0 — the outermost lock
            // just released.
            let prev = SCROLL_LOCK_COUNT.fetch_sub(1, Ordering::AcqRel);
            debug_assert!(prev > 0, "scroll-lock underflow");
        }
    }
}

/// Compatibility alias. The original plan called this `ScrollLock`;
/// renamed to `ScrollLockGuard` so it can be used as a type name
/// without colliding with the static helpers.
pub type ScrollLock = ScrollLockGuard;

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Test-only mutex that serializes the scroll_lock tests so the
    /// global counter is owned by exactly one test at a time.
    /// Without this, parallel `cargo test` runs cause flaky failures
    /// from counter races. The Mutex itself is never poisoned
    /// because we never panic while holding it (each test's body
    /// is straightforward `let g = ...; drop(g);`).
    static SERIAL: Mutex<()> = Mutex::new(());

    #[test]
    fn lock_count_starts_at_zero() {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        reset_for_test();
        assert_eq!(current_lock_count(), 0);
        assert!(!is_locked());
    }

    #[test]
    fn single_lock_activates_and_deactivates() {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        // Reset to a known baseline for a deterministic test.
        reset_for_test();
        let g = ScrollLockGuard::acquire();
        assert_eq!(current_lock_count(), 1);
        assert!(is_locked());
        drop(g);
        assert_eq!(current_lock_count(), 0);
        assert!(!is_locked());
    }

    #[test]
    fn nested_locks_only_release_at_zero() {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        reset_for_test();
        let outer = ScrollLockGuard::acquire();
        let inner = ScrollLockGuard::acquire();
        assert_eq!(current_lock_count(), 2);
        assert!(is_locked());
        drop(inner);
        // Inner released but outer still holds the lock.
        assert!(is_locked());
        assert_eq!(current_lock_count(), 1);
        drop(outer);
        assert!(!is_locked());
        assert_eq!(current_lock_count(), 0);
    }

    #[test]
    fn release_is_idempotent() {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        reset_for_test();
        let g = ScrollLockGuard::acquire();
        assert_eq!(current_lock_count(), 1);
        // release() consumes self and decrements the counter.
        g.release();
        assert_eq!(current_lock_count(), 0);
    }

    #[test]
    fn forget_leaves_counter_incremented() {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        reset_for_test();
        let g = ScrollLockGuard::acquire();
        assert_eq!(current_lock_count(), 1);
        g.forget();
        // The Drop doesn't fire because forget consumed self with
        // armed = false; the counter is left incremented.
        assert_eq!(current_lock_count(), 1);
        // Clean up so the next test sees a clean baseline.
        SCROLL_LOCK_COUNT.store(0, std::sync::atomic::Ordering::Release);
    }
}
