//! Headless / "hook" style component APIs.
//!
//! Most users interact with `yororen-ui` through the
//! `button(...)` / `text_input(...)` / etc. **builder APIs** in
//! [`component`]. These builder components carry a default visual
//! style that themes can override through the `Renderer` trait
//! fleet.
//!
//! Some users — typically those integrating with their own design
//! system, those building fully custom markup, or those wanting
//! pixel-perfect control — need access to the *headless* behavior
//! (a11y, keyboard, focus, click handling, internal state) without
//! the bundled visual style. That's what this module provides.
//!
//! # Usage
//!
//! Each `use_xxx(id, cx)` call returns a `XxxProps` struct. The
//! caller can then either:
//!
//! 1. Read the props' fields directly (e.g. `props.value`,
//!    `props.checked`) and render their own markup, **or**
//! 2. Call `props.apply(div())` to wire up the standard a11y
//!    handler chain (track_focus + on_click + on_hover where
//!    applicable) and receive a `Stateful<Div>` back.
//!
//! ```ignore
//! let props = use_button("my-button", cx);
//! div()
//!     .id("my-button")
//!     .bg(rgb(0x89B4FA))
//!     .rounded(px(20.))
//!     .apply(button_props.apply)
//!     .child("Custom button")
//! ```
//!
//! All 8 hooks (button / icon_button / switch / checkbox / radio /
//! text_input / toggle_button / label) live in this module so the
//! advanced API stays a single import.

use std::sync::Arc;

use gpui::{
    App, AppContext, ClickEvent, Div, ElementId, Entity, FocusHandle, Hsla, InteractiveElement,
    MouseButton, StatefulInteractiveElement, Window,
};

use crate::component::ClickCallback;
use crate::theme::ActiveTheme;

/// Callback for toggle-style hooks (toggle_button / checkbox / switch / radio).
///
/// The `Option<&ClickEvent>` argument is `Some` for pointer clicks and
/// `None` for keyboard activations so the handler can branch on
/// input source.
pub type ToggleCallback = Arc<dyn Fn(bool, Option<&ClickEvent>, &mut Window, &mut App)>;

/// Callback for text input value changes.
pub type TextChangeCallback = Arc<dyn Fn(String, &mut Window, &mut App)>;

// ---------------------------------------------------------------------------
// use_button
// ---------------------------------------------------------------------------

/// Returned by [`use_button`]. Caller applies `on_click` and
/// (optionally) `hover_state` to its own div. `focus_handle` is
/// exposed so keyboard handlers can be wired in.
#[derive(Clone)]
pub struct ButtonProps {
    pub id: ElementId,
    pub on_click: Option<ClickCallback>,
    pub disabled: bool,
    pub clickable: bool,
    pub focus_handle: FocusHandle,
    pub hover_state: Entity<HoverState>,
}

impl ButtonProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_hovered(&self, cx: &App) -> bool {
        self.hover_state.read(cx).hovered
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
}

/// Hover-state entity shared between `use_button` and the
/// Stateful hover tracker used by the headless `use_button` /
/// `use_icon_button` hooks. The renderer-driven component family
/// (Button, IconButton, etc.) goes through gpui's native
/// `.hover(|s| ...)` callback instead, so this state struct only
/// matters for the headless hooks path.
#[derive(Clone, Default)]
pub struct HoverState {
    pub hovered: bool,
}

/// `use_button(id, cx)` — headless button. Use it when you want
/// `Button`-level behavior (a11y, focus, click) without the
/// bundled visual style. Pair with a custom div.
pub fn use_button(id: impl Into<ElementId>, cx: &mut App) -> ButtonProps {
    let id = id.into();
    let focus_handle = cx.focus_handle();
    let hover_state = cx.new(|_| HoverState::default());
    ButtonProps {
        id,
        on_click: None,
        disabled: false,
        clickable: true,
        focus_handle,
        hover_state,
    }
}

impl ButtonProps {
    /// Mutate the props to attach an `on_click` callback. Mirrors
    /// `Button::on_click`.
    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.on_click = Some(Arc::new(listener));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    /// Apply the button's a11y / click / hover handlers to a
    /// `Div`. The returned `Stateful<Div>` is ready to be
    /// `.child(...)`-ed or have any further style applied.
    pub fn apply(self, el: Div) -> gpui::Stateful<Div> {
        let focus_handle = self.focus_handle.clone();
        let hover_state = self.hover_state.clone();
        let click_fn = self.on_click.clone();
        let disabled = self.disabled;
        let clickable = self.clickable;
        let mut s = el.id(self.id.clone()).track_focus(&focus_handle);
        s = s.on_hover(move |hovered, _window, cx| {
            hover_state.update(cx, |s, _| s.hovered = *hovered);
        });
        if clickable
            && !disabled
            && let Some(f) = click_fn
        {
            s = s.on_click(move |ev, window, cx| {
                if disabled {
                    return;
                }
                f(ev, window, cx);
            });
        }
        s
    }
}

// ---------------------------------------------------------------------------
// use_label
// ---------------------------------------------------------------------------

/// Returned by [`use_label`]. Carries the rendered text + a
/// disabled / muted / strong / mono / inherit_color flag set that
/// the caller can read to style its own div.
#[derive(Clone, Debug)]
pub struct LabelProps {
    pub text: String,
    pub muted: bool,
    pub strong: bool,
    pub inherit_color: bool,
    pub mono: bool,
    pub ellipsis: bool,
    pub wrap: bool,
    pub max_lines: Option<usize>,
}

impl LabelProps {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            muted: false,
            strong: false,
            inherit_color: false,
            mono: false,
            ellipsis: false,
            wrap: false,
            max_lines: None,
        }
    }
    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }
    pub fn strong(mut self, strong: bool) -> Self {
        self.strong = strong;
        self
    }
    pub fn inherit_color(mut self, inherit: bool) -> Self {
        self.inherit_color = inherit;
        self
    }
    pub fn mono(mut self, mono: bool) -> Self {
        self.mono = mono;
        self
    }
    pub fn ellipsis(mut self, ellipsis: bool) -> Self {
        self.ellipsis = ellipsis;
        self
    }
    pub fn wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }
    pub fn max_lines(mut self, max: usize) -> Self {
        self.max_lines = Some(max);
        self
    }

    /// Compute the `Hsla` color the bundled `Label` would have
    /// rendered. Useful for `div().text_color(...)` from a
    /// composer's own markup.
    pub fn color(&self, cx: &App) -> Hsla {
        let theme = cx.theme();
        let r: &dyn crate::renderer::LabelRenderer = &**theme
            .renderers
            .get_label()
            .expect("LabelRenderer registered");
        r.color(
            &crate::renderer::LabelRenderState {
                muted: self.muted,
                strong: self.strong,
                mono: self.mono,
                inherit_color: self.inherit_color,
            },
            theme,
        )
    }
}

/// `use_label(text, cx)` — headless text label. Provides the
/// `text()` / `muted` / `strong` / `mono` / `inherit_color` flags
/// without a div. Pair with your own markup.
pub fn use_label(text: impl Into<String>, _cx: &mut App) -> LabelProps {
    LabelProps::new(text)
}

// ---------------------------------------------------------------------------
// use_switch
// ---------------------------------------------------------------------------

/// Returned by [`use_switch`]. Tracks `checked` / `disabled` and
/// exposes an `on_toggle` for the caller.
#[derive(Clone)]
pub struct SwitchProps {
    pub id: ElementId,
    pub checked: bool,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
}

impl SwitchProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
}

pub fn use_switch(id: impl Into<ElementId>, cx: &mut App) -> SwitchProps {
    SwitchProps {
        id: id.into(),
        checked: false,
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_toggle: None,
    }
}

impl SwitchProps {
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(bool, Option<&ClickEvent>, &mut Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(handler));
        self
    }

    pub fn apply(self, el: Div) -> gpui::Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            let checked = self.checked;
            s = s.on_click(move |ev, window, cx| {
                f(!checked, Some(ev), window, cx);
            });
        }
        s
    }
}

// ---------------------------------------------------------------------------
// use_checkbox
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct CheckboxProps {
    pub id: ElementId,
    pub checked: bool,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
}

impl CheckboxProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
}

pub fn use_checkbox(id: impl Into<ElementId>, cx: &mut App) -> CheckboxProps {
    CheckboxProps {
        id: id.into(),
        checked: false,
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_toggle: None,
    }
}

impl CheckboxProps {
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(bool, Option<&ClickEvent>, &mut Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(handler));
        self
    }

    pub fn apply(self, el: Div) -> gpui::Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            let checked = self.checked;
            s = s.on_click(move |ev, window, cx| {
                f(!checked, Some(ev), window, cx);
            });
        }
        s
    }
}

// ---------------------------------------------------------------------------
// use_radio
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RadioProps {
    pub id: ElementId,
    pub checked: bool,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
}

impl RadioProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
}

pub fn use_radio(id: impl Into<ElementId>, cx: &mut App) -> RadioProps {
    RadioProps {
        id: id.into(),
        checked: false,
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_toggle: None,
    }
}

impl RadioProps {
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(bool, Option<&ClickEvent>, &mut Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(handler));
        self
    }

    pub fn apply(self, el: Div) -> gpui::Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            let checked = self.checked;
            s = s.on_click(move |ev, window, cx| {
                f(!checked, Some(ev), window, cx);
            });
        }
        s
    }
}

// ---------------------------------------------------------------------------
// use_toggle_button
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ToggleButtonProps {
    pub id: ElementId,
    pub label: String,
    pub selected: bool,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
}

impl ToggleButtonProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
}

pub fn use_toggle_button(
    id: impl Into<ElementId>,
    label: impl Into<String>,
    cx: &mut App,
) -> ToggleButtonProps {
    ToggleButtonProps {
        id: id.into(),
        label: label.into(),
        selected: false,
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_toggle: None,
    }
}

impl ToggleButtonProps {
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(bool, Option<&ClickEvent>, &mut Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(handler));
        self
    }

    pub fn apply(self, el: Div) -> gpui::Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            let selected = self.selected;
            s = s.on_click(move |ev, window, cx| {
                f(!selected, Some(ev), window, cx);
            });
        }
        s
    }
}

// ---------------------------------------------------------------------------
// use_icon_button
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct IconButtonProps {
    pub id: ElementId,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_click: Option<ClickCallback>,
}

impl IconButtonProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
}

pub fn use_icon_button(id: impl Into<ElementId>, cx: &mut App) -> IconButtonProps {
    IconButtonProps {
        id: id.into(),
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_click: None,
    }
}

impl IconButtonProps {
    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.on_click = Some(Arc::new(listener));
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn apply(self, el: Div) -> gpui::Stateful<Div> {
        let focus_handle = self.focus_handle.clone();
        let click_fn = self.on_click.clone();
        let disabled = self.disabled;
        let mut s = el.id(self.id.clone()).track_focus(&focus_handle);
        if !disabled && let Some(f) = click_fn {
            s = s.on_click(move |ev, window, cx| {
                f(ev, window, cx);
            });
        }
        s
    }
}

// ---------------------------------------------------------------------------
// use_text_input
// ---------------------------------------------------------------------------

/// Returned by [`use_text_input`]. Carries `value` / `placeholder`
/// / `disabled` and a `FocusHandle` so the caller can render any
/// markup on top.
#[derive(Clone)]
pub struct TextInputProps {
    pub id: ElementId,
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_change: Option<TextChangeCallback>,
    pub on_submit: Option<TextChangeCallback>,
    pub max_length: Option<usize>,
}

impl TextInputProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
}

pub fn use_text_input(id: impl Into<ElementId>, cx: &mut App) -> TextInputProps {
    TextInputProps {
        id: id.into(),
        value: String::new(),
        placeholder: String::new(),
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_change: None,
        on_submit: None,
        max_length: None,
    }
}

impl TextInputProps {
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
    pub fn max_length(mut self, max: usize) -> Self {
        self.max_length = Some(max);
        self
    }
    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(String, &mut Window, &mut App),
    {
        self.on_change = Some(Arc::new(handler));
        self
    }
    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(String, &mut Window, &mut App),
    {
        self.on_submit = Some(Arc::new(handler));
        self
    }

    /// Wire up the input's focus / mouse-down handlers. The
    /// caller is responsible for actually rendering the
    /// `text()` content; this method only wires the interaction
    /// model.
    pub fn apply(self, el: Div) -> gpui::Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        s = s.on_mouse_down(MouseButton::Left, move |_ev, _window, _cx| {
            // The caller's div decides how to render the caret
            // and handle text input events. We only need to
            // mark the element as focusable here.
        });
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn use_label_returns_text(cx: &mut TestAppContext) {
        let props = cx.update(|cx| use_label("hello", cx));
        assert_eq!(props.text, "hello");
        assert!(!props.muted);
        assert!(!props.strong);
    }

    #[gpui::test]
    fn use_label_mutators_chain(cx: &mut TestAppContext) {
        let props = cx.update(|cx| {
            use_label("hi", cx)
                .muted(true)
                .strong(true)
                .mono(true)
                .max_lines(3)
        });
        assert!(props.muted);
        assert!(props.strong);
        assert!(props.mono);
        assert_eq!(props.max_lines, Some(3));
    }

    #[gpui::test]
    fn use_button_default_state(cx: &mut TestAppContext) {
        let props = cx.update(|cx| use_button("my-btn", cx));
        assert!(props.clickable);
        assert!(!props.disabled);
    }

    #[gpui::test]
    fn use_button_on_click_attaches(cx: &mut TestAppContext) {
        let props = cx.update(|cx| use_button("my-btn", cx).on_click(|_ev, _w, _cx| {}));
        assert!(props.on_click.is_some());
    }

    #[gpui::test]
    fn use_switch_default_unchecked(cx: &mut TestAppContext) {
        let props = cx.update(|cx| use_switch("my-switch", cx));
        assert!(!props.checked);
    }

    #[gpui::test]
    fn use_checkbox_chain_sets_disabled(cx: &mut TestAppContext) {
        let props = cx.update(|cx| use_checkbox("c", cx).checked(true).disabled(true));
        assert!(props.checked);
        assert!(props.disabled);
    }

    #[gpui::test]
    fn use_radio_default(cx: &mut TestAppContext) {
        let props = cx.update(|cx| use_radio("r", cx));
        assert!(!props.checked);
    }

    #[gpui::test]
    fn use_toggle_button_label_carried(cx: &mut TestAppContext) {
        let props = cx.update(|cx| use_toggle_button("tb", "Bold", cx));
        assert_eq!(props.label, "Bold");
    }

    #[gpui::test]
    fn use_icon_button_default_clickable(cx: &mut TestAppContext) {
        let props = cx.update(|cx| use_icon_button("ib", cx));
        assert!(!props.disabled);
    }

    #[gpui::test]
    fn use_text_input_default_empty(cx: &mut TestAppContext) {
        let props = cx.update(|cx| use_text_input("in", cx));
        assert_eq!(props.value, "");
        assert_eq!(props.placeholder, "");
        assert_eq!(props.max_length, None);
    }
}
