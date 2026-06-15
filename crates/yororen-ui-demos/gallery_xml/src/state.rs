//! Gallery demo state — pure data, each mutable field
//! wrapped in its own `Entity<T>` so the XML layer's
//! `@bind` and brace expressions can read and write it
//! without races.
//!
//! The shape mirrors `showcase_xml`'s `ShowcaseState`:
//! every toggleable / input field is its own entity
//! so that two switches in different sections don't
//! share state.

use gpui::{App, AppContext, Entity, Global};

/// A simple counter — the action section's "Press me"
/// toggle button bumps it, the footer shows the live
/// value.
#[derive(Debug, Clone, Copy, Default)]
pub struct Counter {
    pub value: usize,
}

/// A todo row — mirrors the showcase_xml design where
/// each row owns its own `done` entity so two checkboxes
/// in different rows never sync with each other.
#[derive(Debug, Clone)]
pub struct TodoItem {
    pub label: String,
    pub done: Entity<bool>,
}

/// The 3 sections the user can switch between via the
/// `state.section` field. `<Match on={section}>` in the
/// XML drives the "what's currently in focus" indicator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Section {
    #[default]
    Actions,
    Display,
    Inputs,
    Controls,
    Lists,
}

/// The application state. Each toggleable flag is its
/// own entity; each text input mirrors its value into
/// the `Entity<String>` so the XML can read and the
/// controller can write.
pub struct GalleryState {
    // ---- Toolbar state ----
    pub toast_count: Entity<Counter>,

    // ---- Actions section ----
    pub toggle_btn_selected: Entity<bool>,
    pub last_action_label: Entity<String>,

    // ---- Display section ----
    pub progress_value: Entity<f32>,
    pub tag_closable_count: Entity<Counter>,

    // ---- Inputs section ----
    pub text_value: Entity<String>,
    pub search_value: Entity<String>,
    pub number_value: Entity<f64>,

    // ---- Controls section ----
    pub checkbox_value: Entity<bool>,
    pub switch_value: Entity<bool>,
    pub radio_value: Entity<usize>,
    pub slider_value: Entity<f32>,

    // ---- Lists section ----
    pub todos: Vec<TodoItem>,

    // ---- Routing / focus ----
    pub section: Entity<Section>,
}

impl GalleryState {
    pub fn new_data(cx: &mut App) -> Self {
        // Each todo row mints its own `done` entity so
        // the per-row `<Checkbox @bind={item.done}>`
        // never collides with another row's state.
        let todos = [
            ("Build the XML layer", true),
            ("Add @bind + Match + For", true),
            ("Add Template / Slot", false),
            ("Replace gallery_demo with XML", false),
            ("Ship v0.7", false),
        ]
        .into_iter()
        .map(|(label, done)| TodoItem {
            label: label.to_string(),
            done: cx.new(|_| done),
        })
        .collect();

        Self {
            toast_count: cx.new(|_| Counter { value: 0 }),
            toggle_btn_selected: cx.new(|_| false),
            last_action_label: cx.new(|_| String::from("(none yet)")),
            progress_value: cx.new(|_| 0.42_f32),
            tag_closable_count: cx.new(|_| Counter { value: 0 }),
            text_value: cx.new(|_| String::new()),
            search_value: cx.new(|_| String::new()),
            number_value: cx.new(|_| 3.14_f64),
            checkbox_value: cx.new(|_| false),
            switch_value: cx.new(|_| true),
            radio_value: cx.new(|_| 1_usize),
            slider_value: cx.new(|_| 0.5_f32),
            todos,
            section: cx.new(|_| Section::default()),
        }
    }
}

/// Global handle to the state entity. Stored once at
/// startup; the view reads it via `cx.global::<StateRef>()`.
#[derive(Clone)]
pub struct StateRef {
    pub state: Entity<GalleryState>,
}

impl Global for StateRef {}
