# The 3-layer split in detail

This is the deep-dive behind the one-paragraph summary in `SKILL.md`.
The rest of the docs assume you have internalized it.

## Layers, top to bottom

```
┌──────────────────────────────────────────────────────────────────┐
│ theme JSON (system-light.json, brutalism-light.json, your.json)  │
│   • serde_json::Value                                              │
│   • palette: surface / content / border / action / status         │
│   • tokens: sizes / radii / spacing / typography / motion /       │
│             control (per-component geometry)                       │
│   • Theme::from_json(include_str!("themes/my.json"))              │
└──────────────────────────────────────────────────────────────────┘
                              │
                              ▼   (cx.theme().get_color / get_typed)
┌──────────────────────────────────────────────────────────────────┐
│ renderer crate (yororen-ui-default-renderer or -brutalism-)       │
│   • 54 `XxxRenderer` traits defined in core                       │
│   • 54 `TokenXxxRenderer` impls (default) or `BrutalXxxRenderer`  │
│   • Each trait is one `compose(&self, &props, cx) -> Stateful<Div>`│
│   • Theme reads only — no I/O, no global state                   │
│   • install() registers all 54 impls via RendererRegistry         │
└──────────────────────────────────────────────────────────────────┘
                              │
                              ▼   (cx.renderer_arc::<Marker, dyn Trait>())
┌──────────────────────────────────────────────────────────────────┐
│ headless (yororen-ui-core/src/headless)                           │
│   • 54 factory functions: button(id, cx) -> ButtonProps           │
│   • Each props: builder methods + .apply(div) + .render(cx)      │
│   • .apply(div) is purely a11y (id + focus + click)               │
│   • .render(cx) looks up the registered renderer and composes     │
│   • Stateful composites own Entity<XxxState>                     │
│   • animation/a11y/i18n/notification are sibling modules          │
└──────────────────────────────────────────────────────────────────┘
                              │
                              ▼
                        gpui-ce 0.3
```

## What goes where, concretely

| Concern | Where it lives | Why |
|---|---|---|
| Click handler, focus wiring, keymap binding | `headless::XxxProps` | Same a11y contract for every renderer |
| Caret, selection, IME, scroll, blink | `headless::text_input_core::TextInputCore` | Shared by `text_input` + `combo_box` |
| Open / close animation state | `headless::XxxState::animation: AnimatedVisibility` | Composites own their own lifecycle |
| bg / fg / border / padding / radius / hover / active | `renderer` | Themable; theme is data |
| Colors and per-component geometry | `theme` JSON | Per-deployment, per-locale, per-brand |
| Notification queue, sticky flag, auto-dismiss | `notification::NotificationCenter` | App-level state, not visual |
| RTL flip logic | `rtl` module (uses `i18n::TextDirection`) | Locale-driven layout adjustment |
| `cx.t("key.path")` lookups | `i18n::Translate` trait | Implemented for `App` |
| Scroll lock counter (nested modals) | `a11y::ScrollLockGuard` | Process-global, RAII |
| Click-outside, focus trap, keyboard nav | `a11y::*` | Compositional — opt in per element |

## Why the split matters

The most important consequence is **renderers are swappable**. The
same `headless::button("save", cx).on_click(...).render(cx)` produces:

- a modern rounded-corner button with the **default** renderer + `system-light.json`
- a sharp-cornered, thick-bordered, hard-shadowed button with the **brutalism** renderer + `brutalism-light.json`
- a hand-rolled Material ripple button if the caller chooses to write
  the painter themselves (see the `material_button` example in the
  `layers_demo`)

Same headless. Same state. Same a11y. Different visual.

A second consequence is **themes are data**, not code. A user can author
a JSON file, point `Theme::from_json(include_str!("themes/catppuccin.json"))?`
at it, and hand it to `default_renderer::install_with(cx, theme)`. The
renderer reads whatever paths the JSON happens to contain; missing paths
fall back to defaults (see `Theme::get_color` returning `None` for
unknown keys, and renderers treating `None` as "use the default").

A third consequence is **the headless core has zero visual dependencies**.
This is why `yororen-ui-core` can be a self-contained crate with no
dependency on `yororen-ui-default-renderer`. It also means a test
harness can spin up headless components in isolation — no theme install
required, because the headless layer never reads `cx.theme()`.

## When the layers collide

- **Theme override at render time** (live switching): call
  `yororen_ui::theme::install(cx, new_theme)` inside a `Render` impl,
  every frame. This is the only safe way to swap themes at runtime
  (the gallery demo does it via a toolbar button).
- **Custom renderer for one component**: register it with
  `cx.register_renderer_arc::<m::Button, dyn ButtonRenderer>(Arc::new(MyButtonRenderer))`.
  The headless layer and theme don't notice; `headless::button.render(cx)`
  now finds `MyButtonRenderer` instead of `TokenButtonRenderer`. To make
  this stick, register the renderer in your app's bootstrap (not inside
  a render closure — the registry is global).
- **Custom headless component**: define a new factory function in
  `yororen-ui-core/src/headless/`. You also need a new `marker!()` in
  `renderer/markers.rs` and a new `XxxRenderer` trait in
  `renderer/xxx.rs`. This is a *core* contribution, not an end-user
  task; see `CONTRIBUTING.md` in the repo.
