# v0.2 → v0.3 Migration Guide

v0.3 is a **breaking release** that lands the headless core,
JSON-backed theme, and a new core renderer registry. Themes
are no longer Rust crates — they're JSON files. Renderer
crates register themselves against the new core
`RendererRegistry` via `cx.register_renderer_arc`.

> TL;DR — switch `yororen-ui-theme-system` /
> `yororen-ui-theme-{catppuccin,material,mini}` for a JSON file
> + `Theme::from_json(...)`; update the install call to
> `yororen_ui::renderer::install(cx, appearance)`; nothing else
> has to move.

---

## 1. Workspace structure change

| Before (v0.2) | After (v0.3) |
|---|---|
| `yororen_ui` (single crate) | `yororen-ui-core` + `yororen-ui-default-renderer` + `yororen-ui` (meta) |
| `yororen-ui-theme-system` (Rust crate) | `themes/system-light.json` (bundled in default-renderer) |
| `yororen-ui-theme-catppuccin` (Rust crate) | `themes/catppuccin-*.json` (your own file) |
| `yororen-ui-theme-material` (Rust crate) | `themes/material-*.json` (your own file) |
| `yororen-ui-theme-mini` (Rust crate) | (no theme file — mini only needs 2 colors) |
| `yororen-ui-renderer` | `yororen-ui-default-renderer` (renamed) |
| — | `yororen-ui-mini-renderer` (new, opt-in) |

The 4 theme-* crates are deleted. The default-renderer ships
two JSON files (`system-light.json` / `system-dark.json`)
that you load with `Theme::from_json(include_str!(...))` and
apply via `cx.set_global(GlobalTheme::new(theme))`. The
meta-crate re-exports the default renderer's `install(cx, appearance)`
helper that does both at once.

```toml
# Cargo.toml — easy migration (recommended)
[dependencies]
yororen-ui = "0.3"
# No theme package dep needed. Themes are JSON files.

# Cargo.toml — bring your own renderer (third-party)
[dependencies]
yororen-ui = "0.3"
my-renderer = "0.1"   # registers itself with cx.register_renderer_arc
```

---

## 2. Install the theme

**Before (v0.2):**
```rust
use yororen_ui::theme_system;

theme_system::install(cx, cx.window_appearance());
```

**After (v0.3):**
```rust
use yororen_ui::renderer;

renderer::install(cx, cx.window_appearance());
```

`renderer::install` does two things: sets the global
`Theme` (loaded from `themes/system-light.json` or
`themes/system-dark.json`), and registers the 38 default
`TokenXxxRenderer` impls against the core
`RendererRegistry`.

If you want a custom JSON theme, call
`renderer::install_with(cx, Theme::from_json(my_json)?)`
instead.

---

## 3. JSON-backed Theme (no Rust theme struct)

The `Theme` type in v0.3 is a thin wrapper over
`serde_json::Value`. There is no compile-time schema; the
renderer reads paths like `"action.primary.bg"` /
`"tokens.control.button.min_height"`. The bundled
`system-light.json` / `system-dark.json` files cover every
path the 38 default renderers consume; custom themes can
add new paths and renderers will fall back to defaults for
missing ones.

```rust
// Authoring your own theme
let theme_json = serde_json::json!({
    "action": {
        "primary": { "bg": "#3b82f6", "fg": "#ffffff" }
    },
    "tokens": { "control": { "button": { "min_height": 36 } } },
    "themeColor": "#3b82f6"   // mini-renderer reads this
});
let theme = Theme::from_value(theme_json);
renderer::install_with(cx, theme);
```

---

## 4. Renderer registration moves to core

The `RendererRegistry` is no longer inside `Theme`. It lives
in `yororen_ui_core::renderer::RendererRegistry` and is
installed as a global. Renderers register themselves via:

```rust
use yororen_ui_core::renderer::{markers, RendererContext};
use yororen_ui_core::headless::button::ButtonProps;

cx.register_renderer_arc::<markers::Button, dyn ButtonRenderer>(
    Arc::new(MyButtonRenderer),
);
```

And components retrieve the registered renderer via:

```rust
let r: &Arc<dyn ButtonRenderer> = cx
    .renderer_arc::<markers::Button, dyn ButtonRenderer>()
    .expect("ButtonRenderer registered");
```

The 38 markers (`Button`, `Label`, `TextInput`, …) live in
`yororen_ui_core::renderer::markers`. Third-party renderer
crates can add their own markers for custom components.

The old `theme.renderers.get_button()` API still works for
backward compat (the `Theme` struct still carries a
default-populated `renderers` field), but new code should
use the `cx.renderer_arc` API.

---

## 5. Optional: mini renderer

If you want the smallest possible skin — 2 colors + hardcoded
geometry — opt into the mini renderer:

```toml
[dependencies]
yororen-ui = { version = "0.3", features = ["mini"] }
```

```rust
// After the default install, layer mini on top.
yororen_ui::renderer::install(cx, appearance);
yororen_ui::mini_renderer::install(cx);
```

The mini renderer overrides the 4 components it cares about
(`Button`, `IconButton`, `ToggleButton`, `Label`); every
other component continues to come from the default
renderer. Themes are still JSON — `themes/mini-default.json`
ships in the mini renderer and is loaded via
`mini_renderer::install_with_default_theme(cx)`.

---

## 6. Removed: `yororen-ui-theme-system` etc.

The 4 separate theme crates are gone. The default system
theme is now `themes/system-light.json` /
`themes/system-dark.json` in the default-renderer crate.
If you authored a custom theme crate, convert it:

```rust
// Before: yororen_ui_theme_my_theme::light() -> Theme
// After:
let theme = Theme::from_json(include_str!("../themes/my-light.json"))?;
yororen_ui::renderer::install_with(cx, theme);
```

---

## 7. Headless API is unchanged

`headless::button("id", cx).on_click(...).default_render(cx)`
still works — `default_render` is provided by the
default-renderer crate, which is part of the meta re-export
under the `renderer` module.

Apps that want full visual control can still use the
`headless::*` props + their own `div()` composition:

```rust
use yororen_ui::headless::button;

button("save", cx)
    .on_click(|_, _, _| { /* ... */ })
    .apply(div().bg(my_red).p_2())
    .child("Save")
```

`apply` is purely a11y — it wires focus + click and nothing
else. The caller owns every visual concern, including
hover / active styling. To get an opacity dip on hover,
chain `.hover(|s| s.opacity(0.9))` etc. after `apply`; the
`.raw_hover(...)` knob from v0.3.0 has been removed because
the headless core no longer injects any visual feedback.

---

## 8. Summary checklist

- [ ] Bump `yororen-ui` to `0.3` in your `Cargo.toml`
- [ ] Drop the `yororen-ui-theme-*` deps
- [ ] Replace `theme_system::install(cx, ...)` with `yororen_ui::renderer::install(cx, ...)`
- [ ] Move custom theme Rust code into a JSON file under `themes/`
- [ ] If you wrote a custom renderer trait, register it via `cx.register_renderer_arc::<markers::X, dyn XxxRenderer>(...)`
- [ ] (Optional) Opt into the mini renderer with the `mini` feature for a 2-color skin
