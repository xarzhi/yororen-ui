# Yororen UI

<p align="center">
  <a href="README_zh_CN.md">中文版</a> | <strong>English</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-Apache%202.0-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/rust-edition%202024-yellow.svg" alt="Rust Edition">
  <img src="https://img.shields.io/badge/gpui-based-2a2a2a.svg" alt="Powered by gpui">
</p>

<p align="center">
  <strong>Yororen UI</strong> is a headless-first Rust UI library built on top of <a href="https://github.com/zed-industries/zed"><code>gpui</code></a> (Zed). It ships a three-layer architecture — headless primitives, theme JSON, and swappable visual renderers — so apps stay portable across designs.
</p>

<p align="center">
  It is designed to be consumed by a <code>gpui</code> application crate, while keeping the UI layer self-contained (theme, components, widgets, and embedded icon assets).
</p>

---

## Features

<table>
  <tr>
    <td><strong>54 Components</strong></td>
    <td>Buttons, inputs, badges, tooltips, icons, headings, cards, modals, tree controls, virtualized lists, and more</td>
  </tr>
  <tr>
    <td><strong>Three-Layer Architecture</strong></td>
    <td>Headless primitives + JSON themes + swappable visual renderers (default + brutalism)</td>
  </tr>
  <tr>
    <td><strong>Theme System</strong></td>
    <td>Themes are pure JSON — swap palettes at runtime with one <code>install()</code> call</td>
  </tr>
  <tr>
    <td><strong>Animation System</strong></td>
    <td>Configurable animations with presets, easing functions, and orchestrator</td>
  </tr>
  <tr>
    <td><strong>Internationalization</strong></td>
    <td>Multi-language support (English, Chinese, Arabic), text direction support (LTR/RTL)</td>
  </tr>
  <tr>
    <td><strong>Accessibility</strong></td>
    <td>ARIA support, focus management, keyboard navigation, focus trap</td>
  </tr>
  <tr>
    <td><strong>Embedded Assets</strong></td>
    <td>20+ SVG icons embedded via <code>rust-embed</code> (<code>assets/icons/**</code>)</td>
  </tr>
  <tr>
    <td><strong>Notification System</strong></td>
    <td>Toast notifications with various styles, queue management, and interactive actions</td>
  </tr>
</table>

---

## Quick Start

### 1) Install the renderer

```rust
use gpui::App;
use yororen_ui::renderer;

fn init_ui(cx: &mut App) {
    // Loads the bundled system theme (light/dark by OS appearance)
    // and registers all 54 default TokenXxxRenderer impls.
    renderer::install(cx, cx.window_appearance());
}
```

### 2) Use the headless components

```rust
use gpui::{App, AppContext, Entity, Global};
use yororen_ui::headless::button::button;
use yororen_ui::headless::label::label;

#[derive(Default)]
pub struct Counter { pub value: i32 }
pub struct AppState { pub counter: Entity<Counter> }

impl AppState {
    pub fn new(cx: &mut App) -> Self {
        Self { counter: cx.new(|_| Counter::default()) }
    }
}
impl Global for AppState {}

fn render(cx: &mut gpui::Context<MyApp>) -> impl gpui::IntoElement {
    let count = cx.global::<AppState>().counter.read(cx).value;
    let inc = cx.global::<AppState>().counter.clone();

    gpui::div().size_full().flex().items_center().justify_center().gap_2()
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
```

### 3) Install i18n (Locale + RTL)

```rust
use gpui::App;
use yororen_ui::locale_en;

fn init_i18n(cx: &mut App) {
    locale_en::install(cx);
}
```

### 4) Provide Assets (Icons)

```rust
use gpui::Application;
use yororen_ui::assets::UiAsset;

let app = Application::new().with_assets(UiAsset);
```

If your app has its own assets, compose them with `CompositeAssetSource` so Yororen UI's icons are a fallback layer.

---

## Demo Applications

We provide five official demo applications under `crates/yororen-ui-demos/`:

| Demo | What it shows | Run Command |
|------|---------------|-------------|
| [Counter](#counter) | Minimal bootstrap, single `Entity<T>` global, three buttons | `cargo run -p counter-demo` |
| [Layers Demo](#layers-demo) | The three render pathways (headless / default-render / custom painter) side by side | `cargo run -p layers-demo` |
| [Inputs Demo](#inputs-demo) | All seven text-input components wired with `cx.entity().clone()` `on_change` closures | `cargo run -p inputs-demo` |
| [Gallery Demo](#gallery-demo) | The full 54-component showcase, theme switching, i18n, notifications, virtualized list | `cargo run -p gallery-demo` |
| [Theme Showcase](#theme-showcase) | Per-render `theme::install` for live theme switching | `cargo run -p theme-showcase-demo` |

### Counter

<!-- Screenshot slot: replace demo/screenshots/counter.png when refreshing -->
<img src="demo/screenshots/counter.png" alt="Counter Demo" width="600">

A minimal counter application demonstrating the most fundamental Yororen UI concepts.

**Key Features:**
- Single `Entity<Counter>` global state
- Button click event handling via `on_click`
- Reactive UI updates via `cx.notify()`

**Best For:** Developers new to Yororen UI as their first learning example.

### Layers Demo

<!-- Screenshot slot: replace demo/screenshots/layers-demo.png when refreshing -->
<img src="demo/screenshots/layers-demo.png" alt="Layers Demo" width="600">

A side-by-side comparison of the three render pathways: pure headless (caller paints), default-renderer (theme JSON), and a hand-rolled `MaterialButton` painter with a true ripple animation.

**Key Features:**
- Same headless `button` rendered three different ways
- Bespoke `gpui::Element` painter (see `material_button.rs`)
- A fourth panel showing a text input with `.render(cx, window)`

**Best For:** Understanding what `.apply` does (a11y only) vs `.render(cx)` (full visual).

### Inputs Demo

<!-- Screenshot slot: replace demo/screenshots/inputs-demo.png when refreshing -->
<img src="demo/screenshots/inputs-demo.png" alt="Inputs Demo" width="600">

Seven panels, one per text input. Shows the canonical `cx.entity().clone()` pattern for wiring `on_change` closures.

**Key Features:**
- All seven text inputs: TextInput, PasswordInput, NumberInput, TextArea, SearchInput, FilePathInput, KeybindingInput
- Stepper callbacks on `number_input`
- `KeybindingInputMode` transitions (Idle / Capturing)

**Best For:** Building forms, settings pages, anything input-heavy.

### Gallery Demo

<!-- Screenshot slot: replace demo/screenshots/gallery-demo.png when refreshing -->
<img src="demo/screenshots/gallery-demo.png" alt="Gallery Demo" width="600">

The kitchen-sink reference. Every component rendered in one window, with a working theme switcher (default vs brutalism, light vs dark) and locale toggle (en / zh-CN / ar).

**Key Features:**
- All 54 components in one window
- `cell()` / `input_cell()` helpers that wrap each component in a labelled card with a status line
- Live theme + locale switching via the toolbar
- `NotificationCenter` host rendered with `gpui::deferred(...).with_priority(3)`

**Best For:** Any non-trivial pattern — start here when building a real app.

### Theme Showcase

<!-- Screenshot slot: replace demo/screenshots/theme-showcase.png when refreshing -->
<img src="demo/screenshots/theme-showcase.png" alt="Theme Showcase" width="600">

A single window that demonstrates live theme switching: the same headless button and `TokenButtonRenderer` are reused while the JSON the renderer reads is swapped on every "Next" click.

**Key Features:**
- Bundled `system-light` / `system-dark` themes
- Inline CATPPUCCIN and MATERIAL themes (user-defined JSON)
- One-click theme rotation

**Best For:** Building a "Next theme" toolbar, A/B theme testing.

---

## Built with Yororen UI

Projects and applications built using Yororen UI.

### Yororen Accelerator

<img src="demo/screenshots/accelerator-1.png" alt="Yororen Accelerator" width="380">
<img src="demo/screenshots/accelerator-2.png" alt="Yororen Accelerator" width="380">

<img src="demo/screenshots/accelerator-3.png" alt="Yororen Accelerator" width="380">
<img src="demo/screenshots/accelerator-4.png" alt="Yororen Accelerator" width="380">

A network acceleration tool with native-transparent TCP forwarding + relay passthrough, built with Yororen UI.

**Key Highlights:**
- Complex dashboard with real-time statistics
- Custom window chrome with native-like experience
- Rich data tables and virtualized lists
- Server management and configuration UI

---

## What's Inside

### Crates

<table>
  <tr>
    <td><code>yororen-ui-core</code></td>
    <td>Headless primitives, theme JSON access, i18n, a11y, RTL, animation, assets, notification center</td>
  </tr>
  <tr>
    <td><code>yororen-ui-default-renderer</code></td>
    <td>54 <code>TokenXxxRenderer</code> default impls + bundled <code>system-light.json</code> / <code>system-dark.json</code> themes + <code>renderer::install</code> bootstrap</td>
  </tr>
  <tr>
    <td><code>yororen-ui-brutalism-renderer</code></td>
    <td>Alternative renderer — sharp corners, thick black borders, hard offset shadows, monospace typography + bundled brutalism themes</td>
  </tr>
  <tr>
    <td><code>yororen-ui</code></td>
    <td>Meta-crate that re-exports the three layers + locale catalogs (<code>en</code>, <code>zh-CN</code>, <code>ar</code>)</td>
  </tr>
</table>

### Three-Layer Architecture

```
headless components  ──▶  theme (JSON)  ──▶  renderer (visual)
       ▲                                          │
       └──────── registers at install ────────────┘
```

- **Headless**: data + control + a11y. No visual.
- **Theme**: a single `serde_json::Value` you can swap at runtime.
- **Renderer**: a per-component trait that reads the theme and produces visual divs.

A custom renderer only needs to implement the 54 `XxxRenderer` traits — it doesn't touch the headless layer.

### Component Categories

| Category | Components |
|----------|------------|
| **Foundation** | Button, IconButton, Icon, Label, Text, Heading, Spacer, Divider, Card, FocusRing |
| **Inputs** | TextInput, PasswordInput, NumberInput, TextArea, SearchInput, FilePathInput, KeybindingInput |
| **Selection** | Checkbox, Radio, RadioGroup, Switch, Slider, Select, ComboBox |
| **Display** | Badge, Avatar, Image, ProgressBar, Skeleton, Tag, EmptyState |
| **Overlays** | Tooltip, Popover, Modal, Toast, DropdownMenu, Menu, Disclosure, Overlay |
| **Surfaces** | Panel, Card, Tooltip, Avatar, Image |
| **Lists / Tables** | ListItem, TreeItem, Tree, Table, Form, FormField, VirtualList, UniformVirtualList |
| **Interaction** | ToggleButton, SplitButton, ButtonGroup, ShortcutHint, KeybindingDisplay |

### Icons

```rust
use yororen_ui::icon::IconName;
// Use as the icon source for any component that takes an icon.
```

Icon paths map to embedded SVG assets under `assets/icons/`. For app-specific icons, drop the SVG into your own `AssetSource` and reference it via the icon API.

---

## Requirements

<ul>
  <li><strong>Rust edition:</strong> 2024</li>
  <li><code>gpui</code> is provided via <a href="https://crates.io/crates/gpui-ce"><code>gpui-ce</code></a> crate on crates.io</li>
</ul>

---

## Installation

### From crates.io (recommended)

```toml
[dependencies]
yororen_ui = "0.3"
```

The `xml` feature is enabled by default so `xml!` / `xml_file!` work out of the box. To opt out:

```toml
[dependencies]
yororen_ui = { version = "0.3", default-features = false }
```

If you disabled default features and still want XML support, enable it explicitly:

```toml
[dependencies]
yororen_ui = { version = "0.3", default-features = false, features = ["xml"] }
```

### From GitHub (latest development)

```toml
[dependencies]
yororen_ui = { git = "https://github.com/MeowLynxSea/yororen-ui.git", tag = "v0.3.0" }
```

### From a local path (development)

```toml
[dependencies]
yororen_ui = { path = "../yororen-ui" }
```

---

## Dependencies

<code>gpui-ce</code> is distributed via crates.io with semantic versioning. Make sure your application uses a compatible version:

```toml
[dependencies]
gpui = { package = "gpui-ce", version = "0.3" }
```

In this repository, <code>gpui-ce</code> is specified in <code>Cargo.toml</code>.

---

## License

<ul>
  <li>Yororen UI is licensed under the <strong>Apache License, Version 2.0</strong>. See <code>LICENSE</code>.</li>
  <li>This project is built on top of <code>gpui</code> (Zed Industries), also Apache-2.0.</li>
</ul>

See <code>NOTICE</code> for attribution details.

---

## Contributing

Issues and PRs are welcome.

When changing visuals:
<ul>
  <li>Include screenshots or a short recording</li>
  <li>Keep changes <code>rustfmt</code> clean</li>
</ul>

---

## Wiki

See <a href="https://github.com/MeowLynxSea/yororen-ui/wiki" target="_blank">Yororen UI Wiki</a> for detailed documentation, guides, and component references.

---

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=MeowLynxSea/yororen-ui&type=date&legend=top-left)](https://www.star-history.com/#MeowLynxSea/yororen-ui&type=date&legend=top-left)