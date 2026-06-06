//! Headless `checkbox` — checked / disabled state + on_toggle, no
//! visual.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

use super::switch::ToggleCallback;

#[derive(Clone)]
pub struct CheckboxProps {
    pub id: ElementId,
    pub checked: bool,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
    /// `true` if the caller supplied a custom checked-state color
    /// (consumed by `CheckboxRenderer.has_custom_tone`).
    pub has_custom_tone: bool,
    /// Caller-supplied override for the checked-state fill /
    /// border color. `None` → renderer falls back to the
    /// `action.primary` palette.
    pub custom_tone: Option<gpui::Hsla>,
    /// See `headless::button::ButtonProps::raw_hover`.
    pub raw_hover: bool,
}

pub fn checkbox(id: impl Into<ElementId>, cx: &mut App) -> CheckboxProps {
    CheckboxProps {
        id: id.into(),
        checked: false,
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_toggle: None,
        has_custom_tone: false,
        custom_tone: None,
        raw_hover: true,
    }
}

impl CheckboxProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
    pub fn checked(mut self, v: bool) -> Self {
        self.checked = v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn has_custom_tone(mut self, v: bool) -> Self {
        self.has_custom_tone = v;
        self
    }
    pub fn custom_tone(mut self, c: gpui::Hsla) -> Self {
        self.custom_tone = Some(c);
        self.has_custom_tone = true;
        self
    }
    pub fn on_toggle<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(bool, Option<&ClickEvent>, &mut Window, &mut App),
    {
        self.on_toggle = Some(Arc::new(f));
        self
    }
    pub fn raw_hover(mut self, raw: bool) -> Self {
        self.raw_hover = raw;
        self
    }

    pub fn apply(self, el: Div) -> Stateful<Div> {
        let mut s = el.id(self.id.clone()).track_focus(&self.focus_handle);
        if self.raw_hover && !self.disabled {
            s = s
                .hover(|mut style| { style.opacity = Some(0.9); style })
                .active(|mut style| { style.opacity = Some(0.85); style });
        }
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
