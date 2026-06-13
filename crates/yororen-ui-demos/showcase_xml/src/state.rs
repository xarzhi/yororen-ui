//! Showcase application state — plain data owned by
//! an `Entity` (held by the controller).
//!
//! The `xml!` macro's `@bind={entity}` and brace
//! expressions read individual sub-entities, so each
//! field that needs two-way binding or event-driven
//! mutation is itself wrapped in an entity. Field
//! reads that are pure values (like `todos: Vec<TodoItem>`)
//! stay plain — they're cloned per render, which is
//! fine for a small fixed list.

use gpui::{App, AppContext, Entity, Global};

/// A simple counter.
#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    pub value: i32,
}

/// Connection status — used to drive the `<Match>` demo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// A todo item.
#[derive(Debug, Clone)]
pub struct TodoItem {
    pub label: String,
    #[allow(dead_code)]
    pub done: bool,
}

/// The application state. Each mutable field that the
/// UI binds against is wrapped in its own `Entity<T>`
/// so the macro can read it via `cx.read(...)` and
/// the controller can update it via `cx.update(...)`.
pub struct ShowcaseState {
    pub counter: Entity<Counter>,
    pub flag: Entity<bool>,
    pub name: Entity<String>,
    pub todos: Vec<TodoItem>,
    pub connection: Entity<ConnectionStatus>,
}

impl ShowcaseState {
    /// Build the data inside a new entity. Callers
    /// wrap with `cx.new(|cx| ShowcaseState::new_data(cx))`.
    pub fn new_data(cx: &mut App) -> Self {
        let mut todos = Vec::new();
        for (label, done) in [
            ("Wire up the XML macro", true),
            ("Build a counter demo", true),
            ("Add Match / State / event modifiers", false),
            ("Separate logic from layout", false),
            ("Write a Phase 3 README", false),
        ] {
            todos.push(TodoItem {
                label: label.to_string(),
                done,
            });
        }
        Self {
            counter: cx.new(|_| Counter { value: 0 }),
            flag: cx.new(|_| false),
            name: cx.new(|_| String::from("")),
            todos,
            connection: cx.new(|_| ConnectionStatus::Disconnected),
        }
    }
}

/// Global handle to the state entity. Stored once at
/// startup; the view reads it via
/// `cx.global::<StateRef>()`.
#[derive(Clone)]
pub struct StateRef {
    pub state: Entity<ShowcaseState>,
}

impl Global for StateRef {}