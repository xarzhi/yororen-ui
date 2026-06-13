//! yororen-ui Simple State Pattern
//!
//! This demo uses `gpui::Entity<T>` for app state instead of `Arc<Mutex<T>>`.
//!
//! Why:
//! - GPUI already tracks which `Entity`s a window reads during render.
//! - When an entity is mutated, `cx.notify()` can invalidate the window efficiently.
//! - No manual `EntityId` plumbing is needed for basic state updates.
//!
//! `Entity::update(...)` does not implicitly notify the window —
//! call `cx.notify()` (on the entity context) after mutation.

use gpui::{App, AppContext, Entity, Global};

#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    pub value: i32,
}

/// Global wrapper so components can access the same `Entity<Counter>` via `cx.global::<CounterState>()`.
///
/// `CounterState` itself is stored as a GPUI global, but the mutable data lives in the entity.
pub struct CounterState {
    pub counter: Entity<Counter>,
}

impl CounterState {
    pub fn new(cx: &mut App) -> Self {
        Self {
            counter: cx.new(|_| Counter::default()),
        }
    }
}

impl Global for CounterState {}
