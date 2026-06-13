# Bootstrap pattern — full main.rs

The minimal main.rs in `SKILL.md` is enough for a fresh app. This is the
"everything wired" version, with annotations, showing the order of
operations and what each one unlocks.

```rust
//! my_app/src/main.rs
//!
//! A Yororen UI app that exercises every boot-time global: theme, i18n,
//! notifications, and a custom Entity<T> app state. The window is a
//! scrollable gallery of every basic input — see `my_app.rs` for the
//! root render closure.

use gpui::{App, AppContext, Application, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::locale;
use yororen_ui::notification::center::NotificationCenter;
use yororen_ui::renderer;

mod my_app;
mod state;

const EN_JSON: &str = include_str!("../locales/en.json");

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        // 1. Theme + 54 default renderers (one call; reads OS appearance).
        renderer::install(cx, cx.window_appearance());

        // 2. Bind the text-input keymap. Idempotent — see
        //    headless::text_input::init. Call this BEFORE opening any
        //    window that contains a text input, or the first keystroke
        //    will be silently dropped.
        yororen_ui::headless::text_input::init(cx);

        // 3. Locale: framework strings + app strings layered on top.
        //    For a single-locale app, just use locale_en::install(cx).
        let app_translations = locale::parse_bundled_translations(EN_JSON);
        locale::install_with_translations(cx, "en", app_translations);

        // 4. NotificationCenter — only needed if your app shows toasts.
        //    Calling notify() before set_global is a no-op, so the
        //    renderer's auto-dismiss host will register itself lazily.
        cx.set_global(NotificationCenter::new());

        // 5. App state. The state constructor mints any composite
        //    entities (SelectState, ModalState, etc.) it needs; see
        //    state.rs for the canonical pattern.
        cx.set_global(state::AppState::new(cx));

        // 6. Open the main window.
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(
                gpui::Bounds::centered(None, size(px(900.0), px(700.0)), cx),
            )),
            ..Default::default()
        };
        let app_entity = cx.new(my_app::MyApp::new);
        let _ = cx.open_window(options, |_, _cx| app_entity);
    });
}
```

## Why this order

| Order | What it unlocks | What breaks if you do it later |
|---|---|---|
| 1. `renderer::install` | `cx.theme()`, every `headless::Xxx.render(cx)` | Renderers panic with `"XxxRenderer not registered"` |
| 2. `text_input::init` | First keystroke in any text input is handled | First keystroke is silently dropped; no error |
| 3. `locale::install_with_translations` | `cx.t("key")` returns the right string | Returns the key itself (no crash, but obvious bug) |
| 4. `set_global(NotificationCenter)` | `cx.global::<NotificationCenter>().notify(...)` | `cx.global` panics with "global not set" |
| 5. `set_global(AppState)` | `cx.global::<AppState>()` in render closures | Same panic |
| 6. `cx.open_window` | A window appears | The app exits silently with no window |

## Optional globals

| Global | When to set it | Where to read it |
|---|---|---|
| `NotificationCenter` | App uses toasts / banners | `cx.global::<NotificationCenter>()` from any click handler |
| `I18n` | Default locale is set automatically by `locale::install_locale` | `cx.i18n()` from any closure |
| `GlobalTheme` | Set by `renderer::install` automatically | `cx.theme()` from any render closure |
| `RendererRegistry` | Set by `renderer::install` automatically | Only the headless layer reads it (`cx.renderer_arc`) |
| Your own `AppState` | Always (if you have one) | `cx.global::<AppState>()` in render closures |

## Variations

### Custom theme (not system)

```rust
use yororen_ui_default_renderer as default_renderer;
use yororen_ui_default_renderer::Theme;

const MY_THEME: &str = include_str!("../themes/my-light.json");
let theme = Theme::from_json(MY_THEME).expect("valid theme JSON");
default_renderer::install_with(cx, theme);
```

### Brutalism renderer

```rust
use yororen_ui::brutalism_renderer;
brutalism_renderer::install(cx);   // picks light/dark by OS appearance
// or
brutalism_renderer::install_with_default_theme(cx);
```

### Multi-window apps

`cx.open_window` can be called multiple times. Each window gets its
own `Render` entity. Global state (`AppState`, `NotificationCenter`)
is shared. The `gallery_demo` opens a single window but the pattern
is the same for many.

### Closing a window programmatically

```rust
window_handle.update(cx, |_, window, _| { window.remove_window(); });
```

`window.window_handle()` returns an `AnyWindowHandle` you can stash in
state to close the window later.
