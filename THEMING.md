# Theming Guide

> **Status**: Phase F (v0.5.0). `yororen-ui` ships a Renderer-trait-based theming system
> that lets a third-party package swap component visuals without touching core. This
> guide explains how to write your own theme package.

## 1. Mental model

`yororen-ui-core` is **headless**. It provides:

1. **State, behavior, a11y, RTL, i18n** for every component.
2. **A `Theme` struct** with color slots (`surface`, `content`, `border`, `action`,
   `status`, `shadow`) and a `tokens` namespace (`sizes`, `radii`, `spacing`,
   `typography`, `motion`, `control`).
3. **A `RendererRegistry`** of `Arc<dyn XxxRenderer>` for every visual component.
4. **A default `TokenXxxRenderer` implementation** for each component, which reads
   from the `Theme` and produces the v0.5 baseline visual.

A **theme package** (like `yororen-ui-theme-system` or `yororen-ui-theme-catppuccin`)
plugs in its own `Theme` and its own `RendererRegistry` overrides. The app code does
not change at all.

```
┌─────────────────────────────────────────┐
│           yororen-ui-core               │
│  ┌─ behavior ─┐ ┌─ state ─┐ ┌─ a11y ─┐ │
│  │ click/hover│ │  Entity │ │ ARIA   │ │
│  │ focus/Esc  │ │ observe │ │ focus  │ │
│  └────────────┘ └─────────┘ └────────┘ │
│  ┌─ Renderer traits ───────────────────┐│
│  │ ButtonRenderer, InputRenderer, ...  ││
│  │ (38+ trait definitions)             ││
│  └─────────────────────────────────────┘│
│  ┌─ Default token-based renderers ─────┐│
│  │ TokenButtonRenderer, ...            ││
│  │ (reads from Theme.tokens().*)       ││
│  └─────────────────────────────────────┘│
└─────────────────────────────────────────┘
                  ▲                  ▲
                  │ palette +        │ renderer
                  │ tokens           │ overrides
                  │                  │
   ┌──────────────┴───┐  ┌───────────┴────────────┐
   │ theme-system     │  │ theme-catppuccin        │
   │ (neutral)        │  │ (Latte/Mocha palette)   │
   └──────────────────┘  └────────────────────────┘
```

## 2. The smallest possible theme package

A complete, runnable example: a theme that overrides **just the button** with a custom
color and radius, and reuses the rest of the system theme.

```rust,ignore
// crates/my-theme/src/lib.rs
use std::sync::Arc;
use gpui::{App, WindowAppearance, hsla, px, Hsla, Pixels};
use yororen_ui_core::renderer::{
    ButtonRenderState, ButtonRenderer, RendererRegistry, spec::Edges,
};
use yororen_ui_core::theme::{Theme, GlobalTheme, ThemeSet, ActiveTheme};
use yororen_ui_theme_system as theme_system;

pub struct BrandButtonRenderer;
impl ButtonRenderer for BrandButtonRenderer {
    fn bg(&self, _state: &ButtonRenderState, _theme: &Theme) -> Hsla {
        hsla(0.0, 0.7, 0.5, 1.0) // bright magenta
    }
    fn fg(&self, _state: &ButtonRenderState, _theme: &Theme) -> Hsla {
        hsla(0.0, 0.0, 1.0, 1.0) // white
    }
    fn padding(&self, _state: &ButtonRenderState, _theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(px(20.), px(12.))
    }
    fn border_radius(&self, _state: &ButtonRenderState, _theme: &Theme) -> Pixels {
        px(8.)
    }
    fn border(&self, _: &ButtonRenderState, _: &Theme) -> Option<...> { None }
    fn shadow(&self, _: &ButtonRenderState, _: &Theme) -> Option<...> { None }
    fn min_height(&self, _: &ButtonRenderState, theme: &Theme) -> Pixels {
        theme.tokens.control.button.min_height
    }
    fn disabled_opacity(&self, _: &ButtonRenderState, _: &Theme) -> f32 { 1.0 }
}

pub fn install(cx: &mut App, appearance: WindowAppearance) {
    let mut theme = match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => theme_system::light(),
        _ => theme_system::dark(),
    };
    theme.renderers = RendererRegistry::token_based()
        .with_button(Arc::new(BrandButtonRenderer));
    cx.set_global(GlobalTheme::new_with_themes(appearance, ThemeSet::new(theme)));
}
```

That's it — every `button(...)` in the app will now render in brand magenta with 8 px
radius, while all other components keep their v0.5 baseline.

## 3. Building a Theme from a palette

A typical theme package builds its own `Theme` from a palette of colors:

```rust,ignore
use yororen_ui_core::theme::*;
use yororen_ui_core::i18n::TextDirection;

pub fn my_theme() -> Theme {
    Theme {
        surface: SurfaceTheme {
            canvas: hex(0x1A1B26),   // canvas
            base:   hex(0x24283B),   // panel
            raised: hex(0x2F3447),   // raised panel
            sunken: hex(0x16161E),   // sunken
            hover:  hex(0x363A52),   // hover
        },
        content: ContentTheme {
            primary:   hex(0xC0CAF5),
            secondary: hex(0xA9B1D6),
            tertiary:  hex(0x565F89),
            disabled:  hex(0x414868),
            on_primary:hex(0x1A1B26),
            on_status: hex(0x1A1B26),
        },
        border: BorderTheme {
            default: hex(0x363A52),
            muted:   hex(0x24283B),
            focus:   hex(0x7AA2F7),  // signature accent
            divider: hex(0x24283B),
        },
        action: ActionTheme {
            neutral: ActionVariant { /* 6 colors */ },
            primary: ActionVariant { /* 6 colors */ },
            danger:  ActionVariant { /* 6 colors */ },
        },
        status: StatusTheme { /* 4 variants */ },
        shadow: ShadowTheme { /* elevation_1 / elevation_2 */ },
        text_direction: TextDirection::Ltr,
        tokens: DesignTokens::default(),
        renderers: my_registry(),
    }
}
```

`DesignTokens::default()` is the v0.5 baseline; it gives sensible numbers for
`tokens.control.button.min_height`, `tokens.radii.md`, `tokens.spacing.inset_md`,
etc. A theme package can override individual token fields if it wants different
geometric defaults.

## 4. Renderer overrides

`RendererRegistry` exposes 38+ `with_xxx(renderer)` setters. Each `XxxRenderer`
trait is a small set of methods that return concrete values (`Hsla`, `Pixels`,
`Edges<Pixels>`, etc.). The component's render code calls these methods through
`cx.theme().renderers.<comp>.<method>(&state, theme)`.

Common pattern:

```rust,ignore
pub fn my_registry() -> RendererRegistry {
    RendererRegistry::token_based()
        .with_button(Arc::new(MyButtonRenderer))
        .with_card(Arc::new(MyCardRenderer))
        .with_modal(Arc::new(MyModalRenderer))
        .with_focus_ring(Arc::new(MyFocusRingRenderer))
        // ... any other component you want to override
}
```

Components that you **don't** override keep their `TokenXxxRenderer` default.
This is the right way to ship a "minimal" theme that touches only the components
that need a custom look.

## 5. Renderer API surface

Every renderer method receives a typed `XxxRenderState` and a `&Theme`. The state
struct carries the boolean flags the component knows about (e.g. `disabled`,
`hovered`, `pressed`, `focused`, `is_rtl`).

Example: `ButtonRenderState` has `variant`, `disabled`, `is_rtl`, `has_custom_bg`,
`has_custom_hover_bg`, and `custom_style` (for the `VariantRegistry` path).

The renderer method returns a concrete value:

```rust,ignore
pub trait ButtonRenderer: Send + Sync {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &ButtonRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &ButtonRenderState, theme: &Theme) -> Pixels;
    fn border(&self, state: &ButtonRenderState, theme: &Theme) -> Option<BorderSpec>;
    fn shadow(&self, state: &ButtonRenderState, theme: &Theme) -> Option<ShadowSpec>;
    fn min_height(&self, state: &ButtonRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &ButtonRenderState, theme: &Theme) -> f32;
}
```

The full set of renderers is exported from `yororen_ui_core::renderer` and listed
in `crates/yororen-ui-core/src/renderer/mod.rs`. Each `XxxRenderer` follows the
same shape: 5-12 methods returning concrete visual values.

## 6. Custom variants (VariantRegistry)

The 3 built-in `ActionVariantKind` values (Neutral / Primary / Danger) are
augmented by an open `VariantRegistry` that lets any theme (or any app) register
new variants under string keys.

```rust,ignore
use std::sync::Arc;
use gpui::Hsla;
use yororen_ui_core::renderer::{
    GlobalVariantRegistry, VariantKey, VariantRegistry, VariantState, VariantStyle,
};

#[derive(Debug)]
struct GhostVariant;
impl VariantStyle for GhostVariant {
    fn bg(&self, _: &VariantState) -> Hsla { gpui::hsla(0.0, 0.0, 0.0, 0.0) }
    fn fg(&self, _: &VariantState) -> Hsla { gpui::rgb(0xFFFFFF).into() }
    fn border(&self, _: &VariantState) -> Option<Hsla> { None }
    fn disabled_opacity(&self) -> f32 { 1.0 }
}

let reg = Arc::new(VariantRegistry::with_defaults());
reg.register(VariantKey::borrowed("ghost"), Arc::new(GhostVariant));
cx.set_global(GlobalVariantRegistry(reg));
```

Then in a button builder:

```rust,ignore
use yororen_ui::component::button;
use yororen_ui::renderer::{ButtonVariant, VariantKey};

button("save")
    .variant(ButtonVariant::Custom(VariantKey::borrowed("ghost")))
    .child("Save");
```

The renderer reads `state.custom_style` (populated from the registry) and calls
its `bg` / `fg` / `disabled_opacity` instead of the built-in `action.primary.bg`.
This means custom variants layer cleanly on top of any renderer implementation.

The `yororen-ui-theme-catppuccin` crate ships three custom variants out of the
box: `mocha`, `lavender`, `ghost`.

## 7. Multi-ThemeSet switching

`ThemeSet` is a (light, dark) pair. `install(cx, appearance)` picks the right one
based on `WindowAppearance`. Apps that want a third mode (e.g. "high contrast")
can ship a second theme package and swap them at runtime via
`cx.set_global(GlobalTheme::new_with_themes(appearance, themeset))`.

For per-element theme overrides, use `with_theme(theme, || { ... })` from
`yororen_ui::component` — it temporarily installs the supplied theme as the
global theme for the duration of the closure. The theme-compare demo
(`crates/yororen-ui-demos/theme_compare/`) uses this to show two themes
side-by-side.

## 8. Validation

`yororen_ui_core::theme::validate(&theme) -> Vec<Issue>` runs a battery of
contrast / token-range checks. Run it in CI to catch:

- Foreground/background pairs with contrast ratio below the recommended minimum
- Tokens with negative, NaN, or absurdly large values
- Status text that is unreadable on its background
- Suspicious control geometry (e.g. knob larger than track)

```rust,ignore
use yororen_ui_core::theme::validate;
let issues = validate(&my_theme());
assert!(issues.is_empty(), "theme has {} issues", issues.len());
```

The default `theme-system` and `theme-catppuccin` Themes both pass with zero
issues. If your custom theme triggers a `ContrastTooLow` or `TokenOutOfRange`,
fix the source data before shipping.

## 9. Per-flavor palettes (Catppuccin-style)

`yororen-ui-theme-catppuccin` demonstrates the "4 flavors" pattern: a single
crate ships Latte (light) + Frappé + Macchiato + Mocha (dark) palettes, and
exposes a `light()` / `dark()` / `frappe()` / `macchiato()` / `mocha()` factory
for each. The user picks a flavor at install time:

```rust,ignore
use yororen_ui_theme_catppuccin as catppuccin;

catppuccin::install(cx, cx.window_appearance());  // Latte + Mocha
// or
let theme = catppuccin::frappe();  // single Frappé
```

The palette module in that crate uses 4 small submodules (`latte`, `frappe`,
`macchiato`, `mocha`) that each export 26 color helpers. The factory composes
those colors into a `Theme` via a small `build_theme(base, accent, is_dark)`
helper.

When implementing your own multi-flavor theme, follow the same shape:

- One `palette` module with one submodule per flavor.
- One `factories` module that converts each flavor to a `Theme`.
- A default 2-flavor `themeset()` (light + dark) plus per-flavor helpers.

## 10. Common pitfalls

- **Don't read from `Theme.tokens()`** in a custom renderer. The whole point
  of the trait is that a theme package can ignore tokens. If you want to
  reuse a token value (e.g. `tokens.control.button.min_height`), it's fine,
  but don't bake token references into your renderer logic.
- **Don't bundle `rust-embed` or any asset embedder in a theme package** unless
  you need a custom font or icon set. `yororen-ui-core` ships the
  universal 13-icon set; a theme package should reference those, not embed
  new ones.
- **Don't replace `Theme.renderers` entirely** unless you have an override for
  every one of the 38+ components. Always start from
  `RendererRegistry::token_based()` and use the `with_xxx` setters to swap
  individual entries.
- **Don't depend on `yororen-ui-theme-system` in your theme package** unless
  you specifically want to compose with it. Most theme packages should depend
  only on `yororen-ui-core`.
- **Don't write a custom `ActionVariantKind`** — that's the closed enum that
  is going away. Use `VariantRegistry` for new variants instead.

## 11. File layout

A typical theme package:

```
crates/my-theme/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs              # public API: install(cx, appearance), themeset()
    ├── palette.rs          # color constants and helpers
    ├── factories.rs        # theme factories: light(), dark(), custom()
    ├── renderer.rs         # custom XxxRenderer impls + my_registry()
    └── variant.rs          # custom VariantStyle impls (optional)
```

The `Cargo.toml` should depend only on `yororen-ui-core` (and optionally
`yororen-ui-theme-system` if you want to compose with the system palette).

## 12. Reference: the Catppuccin theme

[`yororen-ui-theme-catppuccin`](crates/yororen-ui-theme-catppuccin/) is the
reference implementation of a "real-world" theme package. Reading its source
top-to-bottom is the fastest way to understand the full set of patterns
documented above.

- `palette.rs` — 4 flavors × 26 colors = 104 helpers.
- `factories.rs` — `build_theme` composes any flavor into a `Theme`.
- `renderer.rs` — 12 custom renderers (button, card, modal, focus_ring,
  text_input, switch, checkbox, radio, toast, tag, list_item, empty_state).
- `variant.rs` — 3 custom variants (mocha, lavender, ghost).

The total crate is ~700 lines and demonstrates every pattern this guide
mentions.
