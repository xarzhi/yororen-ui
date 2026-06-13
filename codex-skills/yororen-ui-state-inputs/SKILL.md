---
name: yororen-ui-state-inputs
description: State management and interactive components for end users building Yororen UI apps with gpui. Use when implementing TextInput/TextArea/SearchInput/PasswordInput/NumberInput/FilePathInput/KeybindingInput/ComboBox, building forms (form + form_field + submit_button), wiring on_change/on_submit/on_toggle/on_pick handlers, opening/closing modals/popovers/dropdowns/tooltips/select/menus, configuring the AnimatedVisibility lifecycle on composites, using virtual_list with infinite loading, or debugging typing lag and focus issues. Not for contributing to yororen-ui itself.
---

# Yororen UI State + Inputs

Yororen UI's inputs and composites are all the same shape under the hood:
a headless props builder with a `.render(cx)` (or `.render(cx, window)`)
that produces a fully wired element. This skill covers the wiring.

## 1. Mental model

The framework separates **what a component knows** from **who owns its
state**:

- **The component** owns interaction state. For a text input, that is
  caret position, selection, scroll, blink epoch, IME composition. For
  a `Select`, that is `open: bool`, `highlighted_index: usize`,
  `animation: AnimatedVisibility`. None of this is in your app state.
- **Your app** owns business state. The text inside the input, the
  currently selected option's *value*, whether the form is currently
  submitting. You wire the component's `on_change` / `on_pick` /
  `on_close` callbacks to write into your own `Entity<MyState>`.

This split is what makes v0.3 inputs work without feedback loops: the
component is the single source of truth for interaction state, your
app is the single source of truth for business state, and the
`on_change` callback is the only bridge between them.

```
component internal state     your app state
(caret, selection, value     (text, validation,
 as it types, animation,      submit status, …)
 open/closed, …)
        ▲                              ▲
        │                              │
        │       on_change /            │
        │       on_pick /              │
        │       on_close /             │
        │       on_toggle              │
        │                              │
        └──── write your state ────────┘
              via Entity<T>::update
```

## 2. The three render pathways

Every headless factory exposes both `.apply(div)` and `.render(cx)`.
They do very different things.

| API | What it returns | What it does |
|---|---|---|
| `props.apply(div)` | `Stateful<Div>` | Sets `id`, `track_focus`, `on_click`. **No visual feedback.** |
| `props.render(cx)` | `Stateful<Div>` (or `AnyElement` for inputs) | Looks up the registered renderer, calls `compose`, then wires the same a11y callbacks on top. |
| (Custom) | anything you build | You write the painter yourself (see `material_button` in `layers_demo`) |

### When to use which

- **Default look, fastest path**: `.render(cx)`. The renderer paints
  bg / border / padding / radius / hover / active from the theme.
- **Caller controls every visual**: `.apply(div())...child("Save")`.
  You write the `div()`, the renderer only contributes the focus ring
  and click handler.
- **Bespoke animation / brand identity**: hand-roll a `gpui::Element`
  (the `MaterialRippleElement` in `layers_demo/src/material_button.rs`
  is the canonical example). You still call `props.apply(...)` to keep
  the focus + click wiring.

The `layers_demo` puts all three in one window — read it to see them
side by side.

## 3. The seven text inputs

All seven share a single state machine (`TextInputCore` in
`yororen-ui-core/src/headless/text_input_core.rs`): caret, selection,
scroll, blink, IME. They differ only in their wrapper UI (placeholder
icon, mask char, stepper, browse button, etc.) and their `on_change`
signature.

| Factory | Builder extras | `on_change` |
|---|---|---|
| `text_input(id)` | `.placeholder`, `.disabled`, `.max_length` | `Fn(&str, &mut Window, &mut App)` |
| `password_input(id)` | `.mask_char` | `Fn(&str, &mut Window, &mut App)` |
| `search_input(id)` | `.placeholder` | `Fn(&str, …)` + `.on_clear(F)` (renderer fires after Escape) |
| `number_input(id)` | `.min`, `.max`, `.step`, `.value(seed)` | `Fn(f64, …)` + `.on_increment(F)`, `.on_decrement(F)` |
| `file_path_input(id)` | `.placeholder` | `Fn(&str, …)` + `.on_browse(F)` (renderer fires after native picker) |
| `text_area(id)` | `.max_length` | `Fn(&str, …)` (Enter inserts `\n`) |
| `keybinding_input(id)` | `.mode(Idle or Capturing)` | `Fn(&str, …)` + `.on_start_capture(F)`, `.on_cancel_capture(F)` |

All seven return `AnyElement` from `.render(cx, window)` (they need
`window` for the IME handler registration).

### The `cx.entity().clone()` pattern

Every `on_change` is a `move` closure that needs to reach back into
your app state. The canonical way is:

```rust
use yororen_ui::headless::text_input::text_input;

text_input("email")
    .placeholder("you@example.com")
    .on_change({
        let entity = cx.entity();          // Entity<MyApp> from Context<MyApp>
        move |new: &str, _window, cx| {
            entity.update(cx, |s, _cx| {
                s.email = new.to_string();
            });
        }
    })
    .render(cx, window)
```

Why `cx.entity()` and not `state.global_clone()` or anything else:

- It's the only way to get an `Entity<MyApp>` from inside a
  `Context<MyApp>` render closure.
- Cloning it is cheap (`Entity` is internally `Arc`).
- Each `move` closure that needs it must clone it once at construction
  time; you cannot borrow it.

The `inputs_demo` (`crates/yororen-ui-demos/inputs_demo/src/inputs_app.rs`)
has all seven inputs wired this way. Copy from it.

### Required boot step

`text_input::init(cx)` binds the keyboard keymap (Backspace, Delete,
Left, Right, Shift+Left/Right, Cmd-A/V/C/X, Home, End, Enter, Escape,
Ctrl-Cmd-Space for the character palette) to the `"UITextInput"`
context. Call it once at boot, before opening any window that contains
a text input. It's idempotent. If you don't call it, the inputs still
*render* but the first keystroke will be silently dropped.

```rust
yororen_ui::headless::text_input::init(cx);
```

## 4. Stateful composites

`select`, `combo_box`, `modal`, `popover`, `dropdown_menu`, `tooltip`,
`listbox`, `menu`, `overlay` — every one of these is a **stateful
composite**: a headless props builder that takes a caller-owned
`Entity<XxxState>` and a renderer that reads the state's
`is_open()` / `is_visible()` to decide what to paint.

The lifecycle is uniform:

```text
state.update(cx, |s, _| s.open())    // user clicks trigger
   → AnimatedVisibility::show()      // target=true
   → renderer paints enter animation
state.update(cx, |s, _| s.close())   // user picks / presses Escape
   → AnimatedVisibility::hide()      // target=false
   → renderer paints exit animation
```

### The shape of every composite state

```rust
// Pseudo-code — the actual public API is one method per state.

state = XxxState::new(cx)         // mints Entity<XxxState>
state.update(cx, |s, _| {
    s.open()                      // show animation
    s.close()                     // hide animation
    s.toggle()                    // open ? close : open
});
state.read(cx).is_open()          // query current target
state.read(cx).is_visible()       // query target || progress > 0
state.set_on_change(F)            // wire the callback
state.set_on_close(F)             // for overlay-family
state.set_on_select(F)            // for menu-family
```

The renderer is responsible for painting the trigger; the caller is
responsible for the `on_*` callback.

### Select + ComboBox — the `pick` 3-in-1

Both expose a single `pick(value, window, cx)` method that does
`set_value + close + invoke_change` atomically:

```rust
use yororen_ui::headless::select::{select, SelectOption, SelectState};

let entity_for_pick = entity.clone();
let state: Entity<SelectState> = /* from app state */;
let state_for_pick = state.clone();
state.update(cx, |s, _| {
    s.set_options(vec![
        SelectOption::new("apple", "Apple"),
        SelectOption::new("pear",  "Pear"),
    ]);
    s.set_on_change(move |value, _w, cx| {
        let v = value.to_string();
        entity_for_pick.update(cx, |s, _cx| s.picked = v);
    });
});

// Then in the click handler for an option row:
state_for_pick.update(cx, |s, cx| {
    s.pick(SharedString::from("apple"), window, &mut *cx);
});
```

The `&mut *cx` is the in-place coercion of `&mut Context<App>` to
`&mut App` (via `DerefMut`). It's required because `pick` needs
`&mut App` to schedule the animation tick.

### Modal — focus trap + scrim + Escape

The modal renderer wires FocusTrap, Escape, scrim-click for you. You
just own the state and the body:

```rust
use yororen_ui::headless::modal::{modal, ModalCloseReason, ModalState};

let modal = modal("settings", app.modal_state.clone())
    .child(/* title */)
    .child(/* body */)
    .child(/* footer with close button */)
    .render(cx);

// Close handler:
app.modal_state.update(cx, |s, _| s.close());
// Or with reason:
app.modal_state.update(cx, |s, cx| {
    s.invoke_close(ModalCloseReason::Programmatic, window, &mut *cx);
});
```

A modal needs to be rendered at the **scroll-root level** (sibling
to your main content, not inside it) and wrapped in
`gpui::deferred(...).with_priority(2)` so it paints above the page
content but below the toast host. The `gallery_demo` shows the
exact placement.

### Popover + Dropdown + Menu — trigger + content

All three follow the same shape: pass a trigger element, pass a
content element, the renderer places the content next to the trigger.

```rust
use yororen_ui::headless::popover::{popover, PopoverState};

let popover = popover("user-menu", app.popover_state.clone())
    .trigger(button("user-btn", cx).on_click(...).render(cx))
    .content(/* menu element */)
    .render(cx);
```

`dropdown_menu` is the same shape with a built-in items API
(`state.set_items(vec![...])`); `menu` is the body-only variant for
use inside popovers.

### Tooltip — delay + placement

```rust
use yororen_ui::headless::tooltip::{tooltip, TooltipPlacement, TooltipState};

let tip = tooltip("help", "Click to save (⌘S)", app.tooltip_state.clone())
    .trigger(button("save", cx).render(cx))
    .placement(TooltipPlacement::Bottom)
    .render(cx);
```

`TooltipState::set_delay_ms(400)` for the show delay; renderer hides
on trigger blur or Escape.

## 5. Form + form_field

Forms are a thin layer over `form_field`. The `form` props stores
field values + errors; `form_field` is a labelled wrapper for a
single input.

```rust
use yororen_ui::headless::form::{form, FormValue};
use yororen_ui::headless::form_field::form_field;

let entity_for_form = entity.clone();
let form_el = form("settings", cx)
    .value("email", app.email.clone())
    .error("email", app.email_error.as_deref())
    .submit("Save")
    .on_submit(move |vals: HashMap<SharedString, String>, _w, cx| {
        entity_for_form.update(cx, |s, _cx| {
            s.submit_count += 1;
            if let Some(e) = vals.get("email") {
                s.email = e.to_string();
                s.email_error = if e.contains('@') { None } else { Some("must contain @".into()) };
            }
        });
    });

// The submit button is auto-generated:
let submit_btn = form_el.submit_button(cx).expect("submit label was set");

// Each field is a form_field:
let email_field = form_field("settings-email", "email", cx)
    .label(cx.t("demo.form.email_label"))
    .required(true)
    .input(text_input("email").placeholder("you@example.com").render(cx, window))
    .render(cx);
```

The renderer's job is to lay the fields out (label above input,
error below, required marker). Your job is the validation logic in
`on_submit`.

## 6. Listbox, Tree, Table

### Listbox

A scrollable single-select list. The shared keyboard-nav
algorithm lives in `ListNavigable`; `ListboxState` reuses it
via `highlight_next` / `highlight_prev`. The renderer paints
one row per option and wires each row's click to
`state.pick(value, …)` which writes `selected_value` and fires
`on_change`.

```rust
use yororen_ui::headless::listbox::{listbox, ListboxOption, ListboxState};

// `cx` here is `&mut gpui::App`. Build the entity once per
// component instance and store it on your model.
let listbox_state = cx.new(|_| ListboxState::new(cx));
listbox_state.update(cx, |s, _cx| {
    s.set_options(vec![
        ListboxOption::new("a", "Apple"),
        ListboxOption::new("b", "Banana"),
    ]);
    s.set_on_change(|value, _window, cx| {
        // `cx` inside this callback is `&mut App`.
        // Update your model here. `value` is a `SharedString`.
        let _ = value;
        let _ = cx;
    });
});

// `.render(cx)` looks up the registered `ListboxRenderer`
// (default / brutalism) and returns a `Stateful<Div>` containing
// one row per option. Caller can chain `.child(...)` to add
// trailing elements (e.g. a footer hint).
listbox("fruit", listbox_state).render(cx)
```

### Tree

Stateless data + stateful expansion. Build a `TreeData`, then emit
`tree_item` per row:

```rust
use yororen_ui::headless::tree::{tree, TreeData, tree_node_id};
use yororen_ui::headless::tree_item::tree_item;

let mut data = TreeData::new();
data.add(None, tree_node_id("root"), "Root");
data.add(Some(tree_node_id("root")), tree_node_id("child"), "Child");

let mut expanded = std::collections::BTreeSet::new();
expanded.insert(tree_node_id("root"));

tree("my-tree", cx)
    .data(data)
    .expanded(/* the BTreeSet */)
    .selected(tree_node_id("child"))
```

For each row, emit a `tree_item` with its depth, has_children, etc.
The renderer handles indentation and the chevron.

### Table

Column-driven grid:

```rust
use yororen_ui::headless::table::{table, TableColumn};

let table_el = table("users", cx)
    .column(TableColumn::new("name", "Name").width(200.0))
    .column(TableColumn::new("age", "Age"))
    .row(vec!["Alice".into(), "30".into()])
    .row(vec!["Bob".into(), "25".into()])
    .selected(0)
    .on_select(|row_idx, _w, cx| { /* ... */ });
```

## 7. Virtual list

For long lists, use `virtual_list` (variable-height) or
`uniform_virtual_list` (fixed-height, faster). The `controller` carries
the data; the `.row(closure)` paints one item at a given index.

```rust
use yororen_ui::headless::virtual_list::{virtual_list, VirtualListController};

// 1. Keep a controller in your app state; bump it when the data changes.
let controller = cx.new(|_| MyController::new());
controller.update(cx, |c, _| c.reset(10_000));   // 10k items

// 2. Render:
let entity_for_vl = entity.clone();
virtual_list("rows", &controller, cx)
    .row(move |ix, _window, cx| {
        // emit a single row element for index `ix`
        let app = entity_for_vl.read(cx);
        list_item(format!("row-{ix}"), &app.row_label(ix), cx)
            .selected(app.selected == ix)
            .on_click(/* ... */)
            .render(cx).into_any_element()
    })
    .on_visible_range_change(move |range, total, _w, cx| {
        // Lazy-load more data here. `total` is the current controller
        // size; bump it and notify to extend the list.
    })
    .render(cx)
```

`uniform_virtual_list(id, count, &controller, cx)` is the same idea
for fixed-height rows; it leans on gpui's `uniform_list` for speed.

## 8. Verifying input wiring

After wiring an input, verify before moving on:

- [ ] Clicking the input focuses it (cursor appears, border highlights)
- [ ] Typing inserts text (the renderer's status line shows the new value)
- [ ] Pasting works (Cmd-V)
- [ ] Backspace deletes one character
- [ ] Arrow keys move the caret
- [ ] `on_change` fires for each keystroke (your `state.field` updates)
- [ ] `on_submit` fires on Enter (text inputs + form)
- [ ] Escape clears (search_input) or closes (combo, modal, popover)
- [ ] For composites: `state.read(cx).is_open()` toggles correctly
- [ ] The `&mut *cx` coercion compiles in `pick` / `invoke_close` calls

If any of these fail, the failure mode is almost always one of:
- `text_input::init(cx)` was not called before opening the window
- The `on_change` closure was given `&mut App` instead of the
  captured `Entity<MyApp>`
- A composite's `Entity<XxxState>` was re-minted (a different
  `cx.new(|_| SelectState::default())` per render) — state resets
  every frame

## 9. Related skills

- `$yororen-ui-user` — entry point, hard rules
- `$yororen-ui-app-core` — bootstrap, state pattern, theme, i18n
- `$yororen-ui-recipes` — full working examples
