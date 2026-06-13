# Writing a custom renderer

The renderer is the swappable visual layer. A renderer crate
implements the 54 `XxxRenderer` traits defined in
`yororen-ui-core/src/renderer/`, one impl per trait. The brutalism
renderer (`yororen-ui-brutalism-renderer`) is the canonical
"complete custom renderer" example — start by reading its source.

This file walks through what you'd write to build a renderer of your
own, from the `Cargo.toml` to the install function.

## 1. Crate layout

```
my_renderer/
├── Cargo.toml
├── themes/
│   └── my-light.json
└── src/
    ├── lib.rs              # install(), register_renderers()
    ├── style.rs            # shared constants (radii, borders, …)
    └── renderers/
        ├── mod.rs
        ├── actions.rs      # Button, IconButton, ToggleButton, SplitButton, ButtonGroup
        ├── controls.rs     # Switch, Checkbox, Radio, RadioGroup, Slider
        ├── display.rs      # Label, Heading, Divider, Badge, Tag, Icon, Spacer, …
        ├── inputs.rs       # TextInput, PasswordInput, SearchInput, NumberInput, …
        ├── lists.rs        # ListItem, TreeItem, Tree, Table, Form, FormField
        ├── notifications.rs
        ├── overlays.rs     # Modal, Popover, DropdownMenu, Menu, Tooltip, Disclosure, Overlay
        └── surfaces.rs     # Card, Panel, Avatar, Image, …
```

The grouping follows the `yororen-ui-brutalism-renderer` convention
but is not enforced. The actual constraint is that you implement the
54 traits — one `impl XxxRenderer for MyXxxRenderer` per file.

## 2. Cargo.toml

```toml
[package]
name        = "my_renderer"
edition     = "2024"
version     = "0.1.0"

[dependencies]
gpui              = { package = "gpui-ce", version = "0.3" }
yororen-ui-core   = "0.3"            # the trait definitions
# Do NOT depend on yororen-ui-default-renderer — that would be circular
# once the user replaces the default with you.
```

## 3. The `lib.rs` install function

```rust
use std::sync::Arc;
use gpui::App;
use yororen_ui_core::renderer::markers as m;
use yororen_ui_core::renderer::{
    ButtonRenderer, IconButtonRenderer, /* … all 54 traits … */
};
use yororen_ui_core::theme::{Theme, install as install_theme};

mod renderers;
use renderers::*;   // brings every MyXxxRenderer into scope

const MY_THEME: &str = include_str!("../themes/my-light.json");

/// Install the renderer with a custom theme.
pub fn install_with(cx: &mut App, theme: Theme) {
    install_theme(cx, theme);
    register_renderers(cx);
}

/// Install the renderer with the bundled light theme.
pub fn install(cx: &mut App) {
    let theme = Theme::from_json(MY_THEME).expect("my-light.json is valid");
    install_with(cx, theme);
}

/// Register all 54 trait impls against the core registry.
pub fn register_renderers(cx: &mut App) {
    cx.register_renderer_arc::<m::Button,     dyn ButtonRenderer>     (Arc::new(MyButtonRenderer));
    cx.register_renderer_arc::<m::IconButton, dyn IconButtonRenderer> (Arc::new(MyIconButtonRenderer));
    // … 52 more …
}
```

## 4. A single renderer impl

The `ButtonRenderer` trait requires exactly one method: `compose`. The
trait lives in `yororen-ui-core/src/renderer/button.rs` and looks like
this (paraphrased):

```rust
pub trait ButtonRenderer: 'static {
    fn compose(
        &self,
        props: &ButtonProps,
        focus_handle: &FocusHandle,
        cx: &App,
    ) -> Stateful<Div>;
}
```

A minimal impl:

```rust
// renderers/actions.rs
use gpui::{div, hsla, px, IntoElement, ParentElement, Stateful, Styled};
use yororen_ui_core::headless::button::ButtonProps;
use yororen_ui_core::renderer::button::ButtonRenderer;
use yororen_ui_core::theme::ActiveTheme;

pub struct MyButtonRenderer;

impl ButtonRenderer for MyButtonRenderer {
    fn compose(
        &self,
        props: &ButtonProps,
        focus_handle: &gpui::FocusHandle,
        cx: &gpui::App,
    ) -> Stateful<Div> {
        // 1. Read theme tokens.
        let theme = cx.theme();
        let bg = theme.get_color("action.primary.bg").unwrap_or_default();
        let fg = theme.get_color("action.primary.fg").unwrap_or_default();
        let pad_x = theme.get_number("tokens.control.button.horizontal_padding").unwrap_or(16.0);

        // 2. Build the styled div.
        let el = div()
            .id(props.id.clone())
            .bg(bg)
            .text_color(fg)
            .px(px(pad_x))
            .py(px(8.0))
            .rounded(px(6.0))
            .track_focus(focus_handle)
            .cursor(gpui::CursorStyle::PointingHand);

        // 3. Add hover/active feedback.
        let el = el
            .hover(|s| s.bg(theme.get_color("action.primary.hover_bg").unwrap_or_default()))
            .active(|s| s.bg(theme.get_color("action.primary.active_bg").unwrap_or_default()));

        // 4. Convert Div to Stateful<Div> via the headless `apply` is
        //    NOT what you do here — `compose` returns the visual; the
        //    headless layer wires `on_click` and `id` on top via
        //    `XxxProps::render`. Don't add `on_click` here.
        el
    }
}
```

Notes:

- **The renderer does not wire `on_click`.** That's the headless
  layer's job (via `XxxProps::render`, which calls your `compose` and
  then adds the `on_click` chain). If you add `on_click` here, it
  fires twice.
- **Use `cx.theme()` to read tokens.** If a token is missing, the
  `get_*` calls return `None`; `.unwrap_or_default()` falls back to
  transparent / zero / empty. The `default` renderer treats missing
  tokens as transparent, so the convention is to always pass a
  fallback.
- **`.track_focus(focus_handle)` is the focus ring.** This is what
  makes the keyboard focus indicator visible.
- **`.id(props.id.clone())` is required** — without it, the headless
  layer's `track_focus(&focus_handle)` won't bind to a stable
  element, and the focus ring will flicker across renders.

## 5. Composites — `compose` takes the entity

For stateful composites (select, modal, popover, etc.), the render
trait receives the `Entity<XxxState>`:

```rust
pub trait SelectRenderer: 'static {
    fn compose(
        &self,
        props: &SelectProps,        // owns the Entity<SelectState>
        cx: &App,
    ) -> Stateful<Div>;
}
```

The renderer reads `props.state.read(cx).is_open()` /
`is_visible()` / `value()` / `highlighted_index()` to decide what to
paint. The renderer's render closure is also responsible for
registering mouse handlers that call `state.update(cx, |s, _| s.toggle())`
on the trigger and `state.update(cx, |s, cx| s.pick(value, w, &mut *cx))`
on the option rows.

The brutalism renderer's `select.rs` is the canonical example. Read
it before writing your own — the entity plumbing is the easy part to
get wrong.

## 6. State-required fields on `XxxProps`

The headless `XxxProps` has fields like `id`, `on_click`, `disabled`,
`variant`, `caption`, `icon` for `ButtonProps`; `placeholder`,
`disabled`, `max_length` for `TextInputProps`; etc. Renderers must
read **all** of them — leaving a field unused is a bug that the
default renderer catches with regression tests but a fresh custom
renderer will silently miss.

A safe pattern: read every field, then branch on it. Look at
`yororen-ui-default-renderer/src/renderers/button.rs` for a complete
example.

## 7. Themes are data, not types

Your renderer reads JSON paths from the theme. The user can hand you
their own theme — you don't get to enforce a Rust schema. Always
treat missing keys as "use my default":

```rust
let radius = theme.get_number("tokens.control.button.radius").unwrap_or(6.0);
```

If you want a strict theme, document the required keys in your
readme. The framework's `RendererRegistry::validate()` checks that
all 54 markers are registered, not that the theme has all keys.

## 8. Register at boot, not in render

`cx.register_renderer_arc` mutates the global `RendererRegistry`.
Calling it from a `Render::render` closure is undefined behavior
(state churns every frame). Call it once at app boot, in the same
function that calls `install_theme`.

```rust
fn main() {
    let app = Application::new().with_assets(UiAsset);
    app.run(|cx: &mut App| {
        my_renderer::install(cx);    // sets theme + registers all 54
        // … open windows …
    });
}
```

## 9. Testing

For unit tests, you can `compose` a renderer with a `Theme::from_json`
and a stub `ButtonProps` without spinning up a full app — the
headless layer's props are pure data.

For integration tests, copy the `theme_showcase` shape: build a tiny
app that uses your renderer, open one window, render a few
components, and `cx.theme()` to verify the visual output.

## 10. Reference implementations

- **`yororen-ui-brutalism-renderer/`** — the canonical full custom
  renderer. 54 trait impls, two bundled themes, per-component geometry
  tokens, sharp corners + hard shadows + monospace. Read its
  `lib.rs` first to see the install shape, then pick one renderer
  (e.g. `renderers/actions.rs::BrutalButtonRenderer`) to see the
  compose pattern.
- **`yororen-ui-default-renderer/`** — the same 54 trait impls with a
  modern rounded-corner look. Useful as a "what does a complete
  renderer look like" reference; the code is heavier because it
  handles every variant of every component.
- **`layers_demo/src/material_button.rs`** — a single bespoke
  `gpui::Element` that wraps the headless `button` for a Material
  ripple animation. Not a trait impl; this is the "caller writes
  the painter" pattern. Read it to understand what a renderer is
  *not* required to do.
