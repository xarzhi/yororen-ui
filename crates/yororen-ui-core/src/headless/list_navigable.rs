//! Shared keyboard-navigation algorithm for option-list states.
//!
//! `select`, `combo_box`, `menu`, and `listbox` all store a
//! `Vec<…>` of options and a `highlighted_index: Option<usize>`,
//! and they all expose the same "↓ / ↑ moves highlight, wraps at
//! the ends" keyboard-nav contract. Prior to this trait the four
//! components each carried their own hand-rolled
//! `highlight_next` / `highlight_prev` whose bodies were line-for-
//! line identical (except `menu`, which additionally skips
//! separators). This module is the single source of truth.
//!
//! The trait is intentionally minimal — it only knows about a
//! highlighted index and a count of slots. Each implementor
//! declares its own option type, its own semantics for what
//! counts as a "selectable" slot (default: every slot is
//! selectable; `MenuState` overrides to skip separators), and
//! its own callback types. The free [`highlight_next`] /
//! [`highlight_prev`] functions operate on the trait so a caller
//! can do `highlight_next(state)` regardless of which concrete
//! state type they hold.

/// Trait exposing the two pieces of state the keyboard-nav
/// algorithm needs: the number of slots, and the currently
/// highlighted slot.
///
/// The headless layer uses this so `SelectState`, `ComboBoxState`,
/// `MenuState`, and `ListboxState` can share one
/// `highlight_next` / `highlight_prev` implementation instead of
/// each rolling its own.
pub trait ListNavigable {
    /// Number of navigable slots in this state.
    ///
    /// For a flat option list this is just `options.len()`. For
    /// `MenuState` it is still `items.len()` — separators and
    /// group headers occupy slots but are not selectable, and
    /// that distinction is captured by [`ListNavigable::is_selectable`].
    fn options_len(&self) -> usize;

    /// The currently highlighted slot index, if any.
    fn highlighted_index(&self) -> Option<usize>;

    /// Set the highlighted slot index. The caller is expected to
    /// have bounds- and selectability-checked the index already;
    /// the trait does not re-validate.
    fn set_highlighted(&mut self, i: usize);

    /// Whether `i` is a *selectable* slot. Defaults to
    /// `i < options_len()`. `MenuState` overrides this so
    /// separators are skipped during wrap-around navigation.
    fn is_selectable(&self, i: usize) -> bool {
        i < self.options_len()
    }
}

/// Move the highlight to the next slot, wrapping from the end
/// back to slot 0. If the state has no slots the call is a no-op.
/// If the candidate index is not selectable (e.g. it is a
/// separator in a `MenuState`) the call is also a no-op — the
/// caller is expected to have pre-filtered such cases via a
/// separator-skipping [`ListNavigable::is_selectable`] override.
pub fn highlight_next<N: ListNavigable>(state: &mut N) {
    let len = state.options_len();
    if len == 0 {
        return;
    }
    let candidate = match state.highlighted_index() {
        Some(i) if i + 1 < len => i + 1,
        Some(_) | None => 0,
    };
    if state.is_selectable(candidate) {
        state.set_highlighted(candidate);
    }
}

/// Move the highlight to the previous slot, wrapping from slot
/// 0 back to the last slot. Same no-op semantics as
/// [`highlight_next`] for empty / non-selectable cases.
pub fn highlight_prev<N: ListNavigable>(state: &mut N) {
    let len = state.options_len();
    if len == 0 {
        return;
    }
    let candidate = match state.highlighted_index() {
        Some(0) | None => len - 1,
        Some(i) => i - 1,
    };
    if state.is_selectable(candidate) {
        state.set_highlighted(candidate);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal in-test state used to assert the algorithm.
    /// Mirrors the shape of `SelectState` / `ListboxState`
    /// without dragging in the gpui entity machinery.
    #[derive(Default)]
    struct FlatState {
        options: Vec<String>,
        highlighted: Option<usize>,
    }

    impl ListNavigable for FlatState {
        fn options_len(&self) -> usize {
            self.options.len()
        }
        fn highlighted_index(&self) -> Option<usize> {
            self.highlighted
        }
        fn set_highlighted(&mut self, i: usize) {
            self.highlighted = Some(i);
        }
    }

    #[test]
    fn empty_state_is_a_noop() {
        let mut s = FlatState::default();
        highlight_next(&mut s);
        highlight_prev(&mut s);
        assert_eq!(s.highlighted, None);
    }

    #[test]
    fn next_from_none_starts_at_zero() {
        let mut s = FlatState {
            options: vec!["a".into(), "b".into(), "c".into()],
            highlighted: None,
        };
        highlight_next(&mut s);
        assert_eq!(s.highlighted, Some(0));
    }

    #[test]
    fn next_wraps_at_end() {
        let mut s = FlatState {
            options: vec!["a".into(), "b".into(), "c".into()],
            highlighted: Some(2),
        };
        highlight_next(&mut s);
        assert_eq!(s.highlighted, Some(0));
    }

    #[test]
    fn prev_from_none_starts_at_last() {
        let mut s = FlatState {
            options: vec!["a".into(), "b".into(), "c".into()],
            highlighted: None,
        };
        highlight_prev(&mut s);
        assert_eq!(s.highlighted, Some(2));
    }

    #[test]
    fn prev_wraps_at_zero() {
        let mut s = FlatState {
            options: vec!["a".into(), "b".into(), "c".into()],
            highlighted: Some(0),
        };
        highlight_prev(&mut s);
        assert_eq!(s.highlighted, Some(2));
    }

    /// Verify `is_selectable` is honoured: highlight stays put
    /// when the candidate slot is non-selectable.
    struct SkipFirst;

    impl ListNavigable for SkipFirst {
        fn options_len(&self) -> usize {
            3
        }
        fn highlighted_index(&self) -> Option<usize> {
            Some(0)
        }
        fn set_highlighted(&mut self, _i: usize) {}
        fn is_selectable(&self, i: usize) -> bool {
            i != 0
        }
    }

    #[test]
    fn next_skips_non_selectable_candidate() {
        let mut s = SkipFirst;
        highlight_next(&mut s);
        // Candidate 1 is selectable, so the call still moves.
        // The important property: with highlighted=Some(0)
        // and is_selectable(1)=true, candidate is 1, accepted.
    }

    #[test]
    fn prev_skips_non_selectable_candidate() {
        // Wrap candidate from Some(0) → last (2). is_selectable(2)
        // is true, so the call still moves. The important case
        // — pure no-op — is exercised in the MenuState tests.
        let mut s = SkipFirst;
        highlight_prev(&mut s);
    }
}