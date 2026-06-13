//! Showcase application state — plain data owned by
//! an `Entity` (held by the controller).
//!
//! The `xml!` macro's `@bind={entity}` and brace
//! expressions read individual sub-entities, so each
//! field that needs two-way binding or event-driven
//! mutation is itself wrapped in an entity.
//!
//! Each toggleable flag has its **own** entity — this
//! way multiple switches / checkboxes / per-row
//! checkboxes don't all read the same value.

use gpui::{App, AppContext, Entity, Global};

/// A simple counter.
#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    pub value: i32,
}

/// Connection status — used to drive the `<Match>` demo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

/// A todo item — each row owns its own `done` entity
/// so two checkboxes in different rows don't share
/// state. (`Entity<bool>` per row lets the macro's
/// `@bind={item.done}` target the right slot.)
#[derive(Debug, Clone)]
pub struct TodoItem {
    pub label: String,
    pub done: Entity<bool>,
}

/// The application state. Each mutable field that the
/// UI binds against is wrapped in its own `Entity<T>`
/// so the macro can read it via `cx.read(...)` and
/// the controller can update it via `cx.update(...)`.
pub struct ShowcaseState {
    pub counter: Entity<Counter>,
    pub notifications: Entity<bool>,
    pub agree: Entity<bool>,
    pub name: Entity<String>,
    /// Monotonic counter used to mint a fresh
    /// `TextInputState` whenever the user presses
    /// Clear — the renderer's keyed state is keyed
    /// by the input's `id`, so bumping it forces a
    /// fresh, empty input.
    pub name_input_key: Entity<Counter>,
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
                done: cx.new(|_| done),
            });
        }
        Self {
            counter: cx.new(|_| Counter { value: 0 }),
            notifications: cx.new(|_| false),
            agree: cx.new(|_| false),
            name: cx.new(|_| String::from("")),
            name_input_key: cx.new(|_| Counter { value: 0 }),
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
