---
name: yororen-ui-app-core
description: App bootstrap and core architecture for end users building a gpui desktop app with Yororen UI (yororen_ui). Use when generating or refactoring main.rs, window setup, the one-call renderer install, the i18n bootstrap, NotificationCenter global, custom theme JSON, project module layout (state.rs, *app.rs, components/), Entity<T> state pattern, or theme reads via cx.theme(). Not for contributing to yororen-ui itself.
---

# Yororen UI App Core

Scaffold the foundation of a Yororen UI app: the one-call bootstrap, the
three globals you'll set, the project layout the demos converge on, and
the cross-cutting concerns (state, theme, i18n, notifications, animation,
a11y, assets) that every app needs.

## 1. The one-call bootstrap

Every Yororen UI app starts the same way. `main.rs` is short on purpose:

```rust
use gpui::{App, AppContext, Application, WindowBounds, WindowOptions, px, size};
use yororen_ui::assets::UiAsset;
use yororen_ui::renderer;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // 1. Theme + 54 default renderers, in one call.
        renderer::install(cx, cx.window_appearance());

        // 2. Bind the text-input keymap (idempotent — see §9).
        yororen_ui::headless::text_input::init(cx);

        // 3. Locale (en / zh-CN / ar shipped; or your own).
        yororen_ui::locale_en::install(cx);

        // 4. Notification center — only if you use toasts.
        cx.set_global(NotificationCenter::new());

        // 5. Your app's global state (or skip if not needed).
        cx.set_global(state::AppState::new(cx));

        // 6. Open the main window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(
                gpui::Bounds::centered(None, size(px(800.0), px(600.0)), cx)
            )),
            ..Default::default()
        };
        let _ = cx.open_window(options, |_, cx| cx.new(|_cx| my_app::MyApp));
    });
}
```

That's the whole boot. Every other file in your app reads from these
globals and renders. There is no `app.run` configuration beyond this.

A complete, with-everything-wired variant lives in
[`references/bootstrap-pattern.md`](references/bootstrap-pattern.md).

### When to swap the renderer

The `renderer::install` call above uses the **default** renderer (the
`yororen-ui-default-renderer` crate, with `system-light.json` /
`system-dark.json` chosen by OS appearance). Three legitimate reasons
to change it:

| If you want… | Use |
|---|---|
| A custom theme, but the default look | `default_renderer::install_with(cx, my_theme)` |
| A completely different visual (sharp corners, hard shadows) | `yororen_ui::brutalism_renderer::install(cx)` (feature-gated) |
| A custom renderer for one component | `cx.register_renderer_arc::<m::Button, dyn ButtonRenderer>(Arc::new(MyButtonRenderer))` |

See `$yororen-ui-recipes` for "Choosing a renderer".

## 2. Project layout

The five live demos (`counter`, `layers_demo`, `inputs_demo`, `gallery_demo`,
`theme_showcase`) all converge on the same shape:

```
my_app/
├── Cargo.toml
├── locales/                       # your app's i18n JSON (optional)
│   ├── en.json
│   └── zh-CN.json
├── themes/                        # your app's theme JSON (optional)
│   └── my-light.json
└── src/
    ├── main.rs                    # 10–30 lines; the bootstrap above
    ├── state.rs                   # global state + composite entity fields
    ├── my_app.rs                  # root Render impl
    └── components/                # sub-screens, panels, modals
        ├── settings_modal.rs
        └── ...
```

`main.rs` is **only** the bootstrap. `state.rs` is **only** the
`Entity<T>`-wrapping-`Global` data model. `*_app.rs` is the root
`Render` impl. Everything else (composite states, helper elements,
panel layouts) lives in `components/`.

## 3. The state pattern

Yororen UI uses gpui's `Entity<T>` for all app state. The render layer
tracks which entities a render closure read, so a single `cx.notify()`
on the entity is enough to invalidate the window — no manual
`EntityId` plumbing.

```rust
// state.rs
use gpui::{App, AppContext, Entity, Global};

#[derive(Debug, Clone, Copy, Default)]
pub struct Counter { pub value: i32 }

pub struct AppState { pub counter: Entity<Counter> }

impl AppState {
    pub fn new(cx: &mut App) -> Self {
        Self { counter: cx.new(|_| Counter::default()) }
    }
}
impl Global for AppState {}   // mark as gpui global
```

A render closure reads the state and chains mutations through `update`:

```rust
// my_app.rs
use gpui::{Context, IntoElement, ParentElement, Render, Window, div};
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;

use crate::state::AppState;

pub struct MyApp;
impl Render for MyApp {
    fn render(&mut self, _w: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<AppState>();
        let count = state.counter.read(cx).value;
        let inc = state.counter.clone();

        div().size_full().p_4()
            .child(label("count", count.to_string(), cx).render(cx))
            .child(
                button("inc", cx)
                    .on_click(move |_, _, cx| {
                        inc.update(cx, |c, cx| { c.value += 1; cx.notify(); });
                    })
                    .render(cx)
                    .child("+"),
            )
    }
}
```

Two things to internalize:

1. **Every `move` closure that touches your state needs its own `Entity<T>`
   clone.** The closure owns the clone; the original entity handle stays
   in the global. A single `state.counter.clone()` per closure is
   cheap (it's an `Arc` bump).
2. **Mutate, then `cx.notify()`.** gpui-ce 0.3 does not auto-notify
   on entity update. The `&mut Context` inside `update` is what
   triggers re-render.

For a fuller version, see the [`counter` demo in
`crates/yororen-ui-demos/counter/`](../../crates/yororen-ui-demos/counter/).

## 4. The `Theme` global and `cx.theme()`

The `Theme` is `pub struct Theme(pub serde_json::Value)`. There is no
Rust schema — the renderer reads paths like `action.primary.bg` and
your code can do the same.

```rust
use yororen_ui::theme::ActiveTheme;   // brings cx.theme() into scope

let surface = cx.theme().get_color("surface.base").unwrap_or_default();
let pad     = cx.theme().get_number("tokens.control.button.horizontal_padding").unwrap_or(16.0);
```

`ActiveTheme` is implemented for **both** `App` **and**
`Context<'_, T>`, so `cx.theme()` works in any render closure without
casting.

### Authoring a custom theme

Write a JSON file, parse it, pass to `install_with`:

```rust
use yororen_ui_default_renderer as default_renderer;
use yororen_ui_default_renderer::Theme;

const MY_THEME: &str = include_str!("../themes/my-light.json");

fn main() {
    // ...
    app.run(|cx: &mut App| {
        let theme = Theme::from_json(MY_THEME).expect("valid theme JSON");
        default_renderer::install_with(cx, theme);
        // ...
    });
}
```

A minimal valid theme covers the top-level keys the renderers read:
`surface`, `content`, `border`, `action.{neutral,primary,danger}`,
`status.{success,warning,danger,error,info,neutral}`, `tokens.control.<component>`.
Missing keys fall back to renderer defaults via `get_color`'s
`Option<Hsla>` return type.

For live theme switching at runtime (e.g. a "Next theme" toolbar button),
call `yororen_ui::theme::install(cx, new_theme)` inside the render
closure — every frame. The `theme_showcase` demo does this.

## 5. i18n

Three pieces:

- **Framework strings** ship as a `TranslationMap` per locale in
  `yororen-ui-locale-{en,zh-CN,ar}`. Install one with
  `yororen_ui::locale::install_locale(cx, "en"|"zh-CN"|"ar")` (or
  `yororen_ui::locale_en::install(cx)` for the short form).
- **App strings** are a `TranslationMap` you author (JSON via
  `include_str!`, parsed by `locale::parse_bundled_translations`).
  Layer them on top with
  `locale::install_with_translations(cx, "en", app_map)`.
- **Lookups** are `cx.t("key.path")` everywhere; the `Translate` trait
  is implemented for `App`. The full key is dot-separated; plural
  categories use `tn("items", 5)` which falls back to `items.other` if
  the count-specific form is missing.

```rust
// main.rs — install + layer
use yororen_ui::locale;
const EN: &str = include_str!("../locales/en.json");
let app_map = locale::parse_bundled_translations(EN);
locale::install_with_translations(cx, "en", app_map);

// anywhere
let save_label: SharedString = cx.t("common.save");
```

The `ar` locale is RTL; `cx.i18n().text_direction()` returns
`TextDirection::Rtl` for it. Components that read this direction via
the `rtl` helpers will automatically flip padding / margin / icon
positions. If your custom layout uses `.left_*` / `.right_*` directly
instead of the start/end helpers, it will render wrong in RTL.

## 6. Notifications

If your app shows toasts, set the global and use it:

```rust
use yororen_ui::notification::center::NotificationCenter;
use yororen_ui::notification::Notification;
use yororen_ui::notification::ToastKind;

cx.set_global(NotificationCenter::new());           // once at boot

// somewhere in a click handler:
let center = cx.global::<NotificationCenter>().clone();
center.notify(
    Notification::new("Saved!").title("Done").kind(ToastKind::Success),
    cx,
);
```

The `NotificationCenter` lives in `yororen-ui-core/src/notification/`
and is a thin state machine: it owns the queue, schedules auto-dismiss
timers, and exposes `items()` for the renderer to paint. The renderer
(visual) lives in `yororen-ui-default-renderer`'s notification host;
your app is responsible for wiring the host into the render tree (the
`gallery_demo` shows the exact pattern, including the
`gpui::deferred(...).with_priority(3)` wrap that puts toasts above
modals).

### Sticky semantics

`Notification::sticky(true)` upgrades `dismiss` to `Manual` automatically
(both in the builder and in the scheduler, dual-guarded). Use it for
notifications the user must explicitly dismiss — error alerts that
need acknowledgment, etc. Default is non-sticky with a 4-second timer.

## 7. Animation primitives

The single concept you need: **`AnimatedVisibility`**. Every stateful
composite (`SelectState`, `ModalState`, `PopoverState`, `TooltipState`,
`DropdownMenuState`, `ComboBoxState`, `MenuState`) owns an
`animation: AnimatedVisibility` field, and the renderer reads
`is_visible()` / `progress()` / `phase()` to paint enter/exit transitions.

```rust
// Open the popover; renderer will animate it in.
popover_state.update(cx, |s, _| s.open());
// → AnimatedVisibility::show() — progress lerps 0 → 1, target=true

// Close
popover_state.update(cx, |s, _| s.close());
// → AnimatedVisibility::hide() — progress lerps 1 → 0, target=false
```

`AnimatedVisibility` itself is just `target: bool, progress: f32` plus
an `update(dt)` step. The orchestrator (in
`yororen-ui-core/src/animation/`) drives the timer; the renderer reads
the progress to scale / translate / fade. Default config is 200ms
ease-out quadratic; presets (`fade_in`, `scale_in`, `fade_slide_in_*`,
`bounce_*`, `elastic_*`) are in
`yororen-ui-core/src/animation/preset.rs`.

## 8. a11y primitives

Four modules in `yororen-ui-core/src/a11y/`. They are **opt-in per
element** — you don't get a global "a11y mode"; you compose what you
need.

| Module | When to use | Pattern |
|---|---|---|
| `FocusTrap` | modals that block navigation | `div().focus_trap().on_escape(\|_,_,_\| close()).child(content)` |
| `ClickOutsideGuard` | popovers / dropdowns / context menus | `el.apply(div()).child(content)` then guard close |
| `ScrollLockGuard` | nested modals (prevent body scroll) | `let _lock = ScrollLockGuard::acquire();` in the open path |
| `FocusableList` / `cycle_focus` | keyboard nav between focusable items | `list.push(id); ring.cycle(current_id, Next)` |

## 9. The text-input keymap

If your app uses **any** text input (`text_input`, `password_input`,
`search_input`, `number_input`, `file_path_input`, `keybinding_input`,
`text_area`, or `combo_box` — anything that embeds a text input), call
**once** at boot:

```rust
yororen_ui::headless::text_input::init(cx);
```

This binds 15 keyboard actions (Backspace, Delete, Enter, Escape,
Left, Right, Shift+Left, Shift+Right, Cmd-A, Cmd-V, Cmd-C, Cmd-X,
Home, End, Ctrl-Cmd-Space for the character palette) to the
`"UITextInput"` keymap context. It's idempotent via a `OnceLock`, so
calling it twice is a no-op.

If your app has its own global keymap that conflicts (an editor that
uses `secondary-v` for paste globally), simply don't call `init` — the
text inputs will still work, you just lose the default key bindings.

## 10. Assets

`yororen_ui::assets::UiAsset` is a `rust_embed`-backed `AssetSource`
that ships an embedded SVG icon set. Register it on the `Application`:

```rust
use yororen_ui::assets::UiAsset;
let app = Application::new().with_assets(UiAsset);
```

Reference icons from any `headless::icon` call:

```rust
use yororen_ui::headless::icon::icon;
use yororen_ui::headless::icon::IconSource;

icon("search", IconSource::Builtin("search".into()), cx)
    .size(px(16.))
    .render(cx)
```

For a custom icon set, implement `AssetSource` and use
`Application::new().with_assets(my_source)` instead. For multiple
sources, use `CompositeAssetSource` (defined in
`yororen-ui-core/src/assets.rs`).

## 11. Verification checklist

After wiring the bootstrap, verify before moving on:

- [ ] `cargo check` is clean
- [ ] The window opens at the expected size
- [ ] A simple `headless::button` with `.render(cx)` shows up with the
      default colors (proves the renderer + theme are wired)
- [ ] `cx.theme().get_color("surface.base")` returns `Some(_)` (proves
      the theme is loaded)
- [ ] `cx.t("common.save")` returns `"Save"` (proves i18n is loaded)
- [ ] If using text inputs, `init(cx)` was called and the first keystroke
      inserts text (proves the keymap is bound)

If any of these fail, the issue is almost always **out-of-order init**
(themed component rendered before `install` ran, or `cx.set_global`
called after the first `Render`).

## 12. Related skills

- `$yororen-ui-user` — entry point, dispatch, hard rules
- `$yororen-ui-state-inputs` — inputs, forms, modals, composites
- `$yororen-ui-recipes` — full working examples
