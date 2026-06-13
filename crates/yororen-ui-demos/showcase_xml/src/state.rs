//! Showcase application state — counters, flags, items.

use gpui::{App, AppContext, Entity, Global};

/// A simple counter.
#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    pub value: i32,
}

/// A toggleable flag — just a `bool` (the `@bind` macro
/// currently only knows how to round-trip a bare `bool`
/// or `String` entity).
pub type Flag = bool;

/// A todo item.
#[derive(Debug, Clone)]
pub struct TodoItem {
    pub label: String,
    pub done: bool,
}

/// The application state — every field is a separate
/// `Entity<T>` so the macros can two-way-bind any
/// individual one with `@bind={...}`.
pub struct ShowcaseState {
    pub counter: Entity<Counter>,
    pub flag: Entity<Flag>,
    pub name: Entity<String>,
    pub todos: Vec<TodoItem>,
}

impl ShowcaseState {
    pub fn new(cx: &mut App) -> Self {
        let mut todos = Vec::new();
        for (label, done) in [
            ("Wire up the XML macro", true),
            ("Build a counter demo", true),
            ("Add a checkbox + switch", false),
            ("Write a Phase 2 README", false),
        ] {
            todos.push(TodoItem { label: label.to_string(), done });
        }
        Self {
            counter: cx.new(|_| Counter { value: 0 }),
            flag: cx.new(|_| false),
            name: cx.new(|_| String::from("")),
            todos,
        }
    }
}

impl Global for ShowcaseState {}
