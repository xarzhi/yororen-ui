//! Keyboard navigation utilities.
//!
//! a11y module. Provides helpers for moving focus through a
//! container of focusable elements when the user presses Tab or
//! Shift+Tab.
//!
//! The helpers in this module:
//!
//! - [`FocusableList`]: a manually-maintained ordered list of
//!   focusable element IDs. The caller appends elements as they
//!   render, and the helpers cycle through the list on Tab /
//!   Shift+Tab. This is the same pattern Radix / Headless UI use.
//! - [`cycle_focus`]: pure function that picks the next / previous
//!   element from a list, wrapping at the boundaries.
//! - [`FocusRing`]: helper to define a focus ring on a container
//!   (Next / Previous / First / Last navigation).
//!
//! # Usage
//!
//! ```ignore
//! use yororen_ui::a11y::{FocusableList, cycle_focus, FocusDirection};
//!
//! let mut list = FocusableList::new();
//! list.push("field-1");
//! list.push("field-2");
//! list.push("submit-button");
//!
//! // On Tab:
//! let next = cycle_focus(&list, "field-1", FocusDirection::Next);
//! // -> Some("field-2")
//! ```

use std::sync::Arc;

/// Manually-maintained ordered list of focusable element IDs.
///
/// This is the v0.5 fallback for the absence of a DOM-style
/// child-walking API in gpui-ce 0.3.3. Callers add their focusable
/// elements to the list as they render, then use
/// [`cycle_focus`] to determine the next/previous element on
/// Tab / Shift+Tab.
///
/// A list can be shared across renders via `Arc<Mutex<...>>` if
/// the caller needs stable focus state across frames. For
/// short-lived containers, a per-render `Vec` is sufficient.
#[derive(Clone, Debug, Default)]
pub struct FocusableList {
    /// The element IDs in tab order.
    elements: Vec<Arc<str>>,
}

impl FocusableList {
    /// Create an empty focusable list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a focusable list with the given initial elements.
    pub fn from_elements<I, S>(elements: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        Self {
            elements: elements.into_iter().map(Into::into).collect(),
        }
    }

    /// Append a focusable element to the end of the list.
    pub fn push<S>(&mut self, id: S)
    where
        S: Into<Arc<str>>,
    {
        self.elements.push(id.into());
    }

    /// Insert a focusable element at `index`, shifting later
    /// elements right. Panics if `index > len`.
    pub fn insert<S>(&mut self, index: usize, id: S)
    where
        S: Into<Arc<str>>,
    {
        self.elements.insert(index, id.into());
    }

    /// Remove the element with the given ID. Returns the removed
    /// position, or `None` if the ID was not in the list.
    pub fn remove(&mut self, id: &str) -> Option<usize> {
        if let Some(pos) = self.elements.iter().position(|e| e.as_ref() == id) {
            self.elements.remove(pos);
            Some(pos)
        } else {
            None
        }
    }

    /// Get the position of `id` in the list, or `None` if absent.
    pub fn position(&self, id: &str) -> Option<usize> {
        self.elements.iter().position(|e| e.as_ref() == id)
    }

    /// Get the element at `index`, or `None` if out of bounds.
    pub fn get(&self, index: usize) -> Option<&str> {
        self.elements.get(index).map(|e| e.as_ref())
    }

    /// Number of focusable elements in the list.
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Whether the list is empty.
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// First element in tab order.
    pub fn first(&self) -> Option<&str> {
        self.elements.first().map(|e| e.as_ref())
    }

    /// Last element in tab order.
    pub fn last(&self) -> Option<&str> {
        self.elements.last().map(|e| e.as_ref())
    }

    /// Iterate over the focusable elements in tab order.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.elements.iter().map(|e| e.as_ref())
    }
}

/// Direction for focus navigation. Re-exported from `focus_trap`
/// so callers can use either import path.
pub use crate::a11y::focus_trap::FocusDirection;

/// Cycle focus through a [`FocusableList`] in the given direction.
///
/// - `current` is the element ID that currently has focus (or
///   the empty string if nothing is focused). If `current` is
///   not in the list, the first element is returned (or the
///   last for `Previous`).
/// - `direction` is `Next`, `Previous`, `First`, or `Last`.
/// - At the boundaries, the focus wraps: the last element
///   followed by `Next` returns the first, and vice versa.
///
/// Returns `None` if the list is empty.
pub fn cycle_focus<'a>(
    list: &'a FocusableList,
    current: &str,
    direction: FocusDirection,
) -> Option<&'a str> {
    if list.is_empty() {
        return None;
    }
    let pos = list.position(current);
    match direction {
        FocusDirection::Next => match pos {
            Some(p) => Some(list.get((p + 1) % list.len()).unwrap()),
            None => list.first(),
        },
        FocusDirection::Previous => match pos {
            Some(0) => list.last(),
            Some(p) => Some(list.get(p - 1).unwrap()),
            None => list.last(),
        },
        FocusDirection::First => list.first(),
        FocusDirection::Last => list.last(),
    }
}

/// Focus ring for a container.
///
/// A focus ring is an ordered list of focusable elements that
/// participate in keyboard navigation. It also tracks the
/// currently-focused element so that "next" / "previous" can be
/// computed from a [`cycle_focus`] call.
///
/// A focus ring is built up by `push`-ing focusable elements
/// during the render of the container. It's typically held as a
/// per-render local (or a `use_keyed_state` for persistent rings).
#[derive(Clone, Debug, Default)]
pub struct FocusRing {
    list: FocusableList,
}

impl FocusRing {
    /// Create a new empty focus ring.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new focus ring with the given initial elements.
    pub fn from_elements<I, S>(elements: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        Self {
            list: FocusableList::from_elements(elements),
        }
    }

    /// Append a focusable element to the ring.
    pub fn push<S>(&mut self, id: S)
    where
        S: Into<Arc<str>>,
    {
        self.list.push(id);
    }

    /// Compute the next / previous / first / last element in the
    /// ring, given the currently-focused element.
    pub fn cycle(&self, current: &str, direction: FocusDirection) -> Option<&str> {
        cycle_focus(&self.list, current, direction)
    }

    /// Read-only accessor for the underlying list.
    pub fn list(&self) -> &FocusableList {
        &self.list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_list() -> FocusableList {
        FocusableList::from_elements(["a", "b", "c"])
    }

    #[test]
    fn cycle_next_wraps_at_end() {
        let list = sample_list();
        assert_eq!(cycle_focus(&list, "a", FocusDirection::Next), Some("b"));
        assert_eq!(cycle_focus(&list, "b", FocusDirection::Next), Some("c"));
        assert_eq!(cycle_focus(&list, "c", FocusDirection::Next), Some("a"));
    }

    #[test]
    fn cycle_previous_wraps_at_start() {
        let list = sample_list();
        assert_eq!(cycle_focus(&list, "a", FocusDirection::Previous), Some("c"));
        assert_eq!(cycle_focus(&list, "b", FocusDirection::Previous), Some("a"));
        assert_eq!(cycle_focus(&list, "c", FocusDirection::Previous), Some("b"));
    }

    #[test]
    fn first_and_last_jump_to_ends() {
        let list = sample_list();
        assert_eq!(cycle_focus(&list, "b", FocusDirection::First), Some("a"));
        assert_eq!(cycle_focus(&list, "b", FocusDirection::Last), Some("c"));
    }

    #[test]
    fn unknown_current_falls_back_to_first() {
        let list = sample_list();
        assert_eq!(cycle_focus(&list, "nope", FocusDirection::Next), Some("a"));
        assert_eq!(
            cycle_focus(&list, "nope", FocusDirection::Previous),
            Some("c")
        );
    }

    #[test]
    fn empty_list_returns_none() {
        let list = FocusableList::new();
        assert!(cycle_focus(&list, "anything", FocusDirection::Next).is_none());
        assert!(cycle_focus(&list, "anything", FocusDirection::Previous).is_none());
        assert!(cycle_focus(&list, "anything", FocusDirection::First).is_none());
        assert!(cycle_focus(&list, "anything", FocusDirection::Last).is_none());
    }

    #[test]
    fn list_push_grows() {
        let mut list = FocusableList::new();
        assert!(list.is_empty());
        list.push("a");
        list.push("b");
        assert_eq!(list.len(), 2);
        assert_eq!(list.first(), Some("a"));
        assert_eq!(list.last(), Some("b"));
    }

    #[test]
    fn list_remove_returns_position() {
        let mut list = sample_list();
        assert_eq!(list.remove("b"), Some(1));
        assert_eq!(list.len(), 2);
        assert_eq!(list.position("b"), None);
        // Removing an absent element returns None.
        assert_eq!(list.remove("nope"), None);
    }

    #[test]
    fn list_position_returns_index() {
        let list = sample_list();
        assert_eq!(list.position("a"), Some(0));
        assert_eq!(list.position("c"), Some(2));
        assert_eq!(list.position("d"), None);
    }

    #[test]
    fn focus_ring_cycle_delegates() {
        let mut ring = FocusRing::new();
        ring.push("x");
        ring.push("y");
        ring.push("z");
        assert_eq!(ring.cycle("x", FocusDirection::Next), Some("y"));
        assert_eq!(ring.cycle("z", FocusDirection::Next), Some("x"));
        assert_eq!(ring.cycle("y", FocusDirection::Previous), Some("x"));
    }

    #[test]
    fn focus_ring_from_elements() {
        let ring = FocusRing::from_elements(["p", "q"]);
        assert_eq!(ring.cycle("p", FocusDirection::Next), Some("q"));
        assert_eq!(ring.cycle("q", FocusDirection::Previous), Some("p"));
    }
}
