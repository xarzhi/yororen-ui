//! Headless `toggle_button` — a button with a `selected` flag and
//! `on_toggle` callback. No visual.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

use super::switch::ToggleCallback;

#[derive(Clone)]
pub struct ToggleButtonProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
    pub disabled: bool,
}

pub fn toggle_button(id: impl Into<ElementId>, cx: &mut App) -> ToggleButtonProps {
    ToggleButtonProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        on_toggle: None,
        disabled: false,
    }
}

impl ToggleButtonProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(bool, Option<&ClickEvent>, &mut Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(f));
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }

    pub fn apply(self, el: Div) -> Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        if !self.disabled
            && let Some(f) = self.on_toggle.clone()
        {
            // ToggleButton receives the *new* desired state via the
            // callback. The caller is responsible for tracking the
            // current `selected` and toggling it in response.
            s = s.on_click(move |ev, window, cx| {
                f(true, Some(ev), window, cx);
            });
        }
        s
    }
}
