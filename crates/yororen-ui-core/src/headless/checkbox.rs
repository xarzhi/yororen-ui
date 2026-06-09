//! Headless `checkbox` — checked / disabled state + on_toggle, no
//! visual.
//!
//! `apply` is purely a11y: focus + click. The caller (or the
//! renderer via `default_render`) owns every visual concern,
//! including hover / active feedback.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, ParentElement, Stateful,
    StatefulInteractiveElement, Styled, Window, div, px,
};

use super::switch::ToggleCallback;
use crate::renderer::RendererContext;
use crate::renderer::checkbox::{CheckboxRenderState, CheckboxRenderer};
use crate::renderer::markers::Checkbox as CheckboxMarker;
use crate::theme::ActiveTheme;

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

    /// Wire the headless contract onto the caller's `el`.
    ///
    /// Purely a11y: id, focus, click (which fires
    /// `on_toggle(!checked, ...)`). No visual feedback
    /// injected — caller / renderer owns hover / active.
    pub fn apply(self, el: Div) -> Stateful<Div> {
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

    /// Render the checkbox using the registered `CheckboxRenderer`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let r: &Arc<dyn CheckboxRenderer> = cx
            .renderer_arc::<CheckboxMarker, dyn CheckboxRenderer>()
            .expect("CheckboxRenderer registered");
        let state = CheckboxRenderState {
            checked: self.checked,
            disabled: self.disabled,
            has_custom_tone: self.has_custom_tone,
            custom_tone: self.custom_tone,
        };
        let bg = r.box_bg(&state, theme);
        let border = r.box_border(&state, theme);
        let size = r.box_size(&state, theme);
        let check_size = r.check_size(&state, theme);
        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border)
            .size(size)
            .rounded(px(4.))
            .flex()
            .items_center()
            .justify_center();
        if self.checked {
            el = el.child(div().bg(border).size(check_size).rounded(px(2.)));
        }
        let hover_bg = r.box_hover_bg(&state, theme);
        let active_bg = r.box_active_bg(&state, theme);
        self.apply(el)
            .hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}
