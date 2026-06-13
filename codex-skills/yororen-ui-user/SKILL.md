---
name: yororen-ui-user
description: High-quality app code generation for end users building Rust desktop GUIs with gpui + Yororen UI (yororen_ui). Use when a user asks to build, scaffold, or modify an application using Yororen UI/gpui, or when working in a Rust project that depends on yororen_ui. Triggers include "build a counter with yororen ui", "make a form with TextInput", "add a modal", "add a theme switcher", "use yororen_ui". Routes to sibling skills ($yororen-ui-app-core, $yororen-ui-state-inputs, $yororen-ui-recipes) for deep dives. Not for contributing to yororen-ui itself.
---

# Yororen UI (end-user)

Generate application code that uses Yororen UI correctly: pick the right
sub-skill, follow the 3-layer split, wire the one-call bootstrap, and trust
the framework's built-in patterns for state, inputs, themes, and i18n.

If the user is editing the yororen-ui library itself, stop and ask for an
app repository.

## 1. Mental model — the 3-layer split

Yororen UI ships as three independent layers you compose at the import
boundary:

```
theme JSON  ─▶  renderer (TokenXxxRenderer)  ─▶  headless (XxxProps)  ─▶  gpui-ce
```

| Layer | Crate | Role |
|---|---|---|
| **headless** | `yororen-ui-core` | data + state machine + a11y. No visual decisions. |
| **renderer** | `yororen-ui-default-renderer` (or `-brutalism-`) | turns props into a styled `Div`. 54 trait impls. |
| **theme** | a JSON file | palette + tokens the renderer reads by path. |

The headless layer is the source of truth. The renderer is swappable.
The theme is data. Read more in
[`references/three-layer-architecture.md`](references/three-layer-architecture.md).

## 2. Workflow decision tree

When the user asks for code, route to the right sub-skill. The boundary is
about *what kind of work* the user is doing, not which file gets edited.

| If the user wants to… | Use this sub-skill |
|---|---|
| Scaffold a new app / `main.rs` / window / theme / i18n / state module layout | `$yororen-ui-app-core` |
| Add or fix an input, form, modal, dropdown, popover, listbox, tree, virtual list | `$yororen-ui-state-inputs` |
| Copy a complete working example (counter / layers / inputs / gallery / theme switch) | `$yororen-ui-recipes` |

If the request spans two or three (e.g. "build a settings modal in my
counter app"), prefer `$yororen-ui-recipes` first (it links to the others).

If the repository is **the yororen-ui library itself**, refuse. Ask the
user to point you at their app repo.

## 3. Toolchain

Pin these. Mismatched gpui-ce versions cause compile errors that look like
library bugs.

```toml
# Cargo.toml
[dependencies]
gpui      = { package = "gpui-ce", version = "0.3" }
yororen_ui = "0.3"
```

- **Rust edition 2024** (matches the yororen-ui workspace).
- **gpui-ce** is a community fork of zed's `gpui`. Do not depend on the
  upstream `gpui` crate — it will resolve to a different version and
  break.
- Match the `gpui-ce` minor version to whatever `yororen_ui` resolves to.
  `cargo tree -i gpui-ce` will tell you.

For a brand-new project, prefer a tagged git dep for reproducibility:

```toml
yororen_ui = { git = "https://github.com/MeowLynxSea/yororen-ui.git", tag = "v0.3.0" }
```

For local development against the workspace, `path = "../../yororen-ui"`
is fine — but only when the user explicitly asks to track local changes.
Do not introduce `path = ...` for end users.

## 4. Hard rules

These are the rules the framework was built to enforce. Violate them and
the app will either not compile or quietly misbehave.

1. **One-call bootstrap.** Always start the app with
   `yororen_ui::renderer::install(cx, cx.window_appearance())` (or
   `default_renderer::install_with(cx, my_theme)` for a custom JSON theme).
   This single call sets the global `Theme` and registers all 54 default
   renderers.

2. **The `Theme` global is read-only from app code.** You read it via
   `cx.theme().get_color("action.primary.bg")` and friends. The renderer
   is the only thing that should *write* the theme, and it does that
   once at install time (or per-render, for live theme switching).

3. **State goes in `Entity<T>`, not `Arc<Mutex<T>>`.** gpui-ce already
   tracks which entities a render closure reads, so `cx.notify()` on the
   entity is enough to invalidate the window. `Arc<Mutex<T>>` is
   unnecessary and complicates `cx.notify()` plumbing.

4. **Stateful composites own their `Entity<XxxState>`.** `select`,
   `combo_box`, `modal`, `popover`, `dropdown_menu`, `tooltip`, `listbox`,
   `menu`, `overlay` each ship with a `XxxState` struct whose `::new(cx)`
   mints the entity. You keep the `Entity<XxxState>` in your app state
   and call `state.update(cx, |s, _| s.open())` etc.

5. **Inputs are uncontrolled by default.** Wire
   `.on_change(|new, _w, cx| { ... })` and trust the component to own its
   caret / selection / IME state. If you need to reset the field (open
   edit modal, clear after submit), the component itself is the source
   of truth — derive the new value from your own state, don't fight the
   component.

6. **Don't call `render()` inside an event handler.** Render closures
   run on the GPUI main thread; side effects inside them (timers,
   `cx.spawn`, file I/O) belong in event handlers, not in the
   render path.

7. **Stable identity.** Every stateful child in a list or a virtualized
   view needs a stable `.id(...)` / `.key(...)`. List reordering without
   identity silently desyncs focus and animation state.

## 5. Project layout

The demos converge on this shape; copy it unless the user has a reason
not to.

```
my_app/
├── Cargo.toml
├── locales/             # your app's i18n JSON, optional
│   ├── en.json
│   └── zh-CN.json
├── themes/              # your app's theme JSON, optional
│   └── my-light.json
└── src/
    ├── main.rs          # bootstrap only (10–30 lines)
    ├── state.rs         # global state + composite entity fields
    ├── my_app.rs        # root Render impl
    └── components/      # sub-screens, panels, modals
        ├── settings_modal.rs
        └── ...
```

`main.rs` does *only*:

```rust
fn main() {
    let app = Application::new().with_assets(UiAsset);
    app.run(|cx: &mut App| {
        yororen_ui::renderer::install(cx, cx.window_appearance());
        yororen_ui::locale_en::install(cx);           // or locale::install_locale(cx, "zh-CN")
        cx.set_global(state::AppState::new(cx));      // if you have one
        cx.set_global(NotificationCenter::new());     // only if you use toasts
        // cx.open_window(...) goes here
    });
}
```

See `$yororen-ui-app-core` for the full bootstrap, theme authoring, i18n
plumbing, and the `state.rs` pattern.

## 6. Output standards

When you write code for the user:

- **One module at a time.** Don't dump a 600-line file in one message —
  show the relevant slice, then offer the rest.
- **Use the nested import path.** `use yororen_ui::headless::button::button;`
  imports the factory function. `use yororen_ui::headless::button;` imports
  the *module* (which is almost never what you want).
- **Use the existing API names.** When in doubt, check the headless
  factory in `crates/yororen-ui-core/src/headless/*.rs` of the dependency
  source checkout.
- **Run before committing.** After you generate code, run `cargo check`
  on the user's project. The dylib ABI and trait bounds shift enough
  that "it looks right" is not enough.
- **Explain non-obvious choices.** The `Entity<XxxState>` pattern,
  `cx.entity().clone()` for closures, `gpui::deferred(...).with_priority(N)`
  for overlay z-order — these are framework-specific. A one-line
  comment next to the pattern saves a future reader an hour.

## 7. Related skills

- `$yororen-ui-app-core` — bootstrap, project layout, theme, i18n, state
- `$yororen-ui-state-inputs` — inputs, forms, modals, composites, `cx.entity()` pattern
- `$yororen-ui-recipes` — full working examples (counter, layers, inputs, gallery, theme)

## 8. When NOT to use this skill

- The repository is `yororen-ui` itself (or a fork). Refuse, point at the
  repo's `CONTRIBUTING.md`.
- The user is asking about a non-gpui Rust GUI library (egui, iced, slint,
  tauri). Yororen UI is a gpui binding.
