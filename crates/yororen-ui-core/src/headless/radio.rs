//! Headless `radio` — checked / disabled / on_toggle, no
//! visual.
//!
//! `apply` is purely a11y: focus + click. The caller (or the
//! renderer via `default_render`) owns every visual concern,
//! including hover / active feedback.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, ParentElement, Stateful,
    StatefulInteractiveElement, Styled, Window, div,
};

use super::switch::ToggleCallback;
use crate::renderer::RendererContext;
use crate::renderer::markers::Radio as RadioMarker;
use crate::renderer::radio::{RadioRenderState, RadioRenderer};
use crate::theme::ActiveTheme;

#[derive(Clone)]
pub struct RadioProps {
    pub id: ElementId,
    pub checked: bool,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
    /// `true` if the caller supplied a custom checked-state
    /// color (consumed by `RadioRenderer.has_custom_tone`).
    pub has_custom_tone: bool,
    /// Caller-supplied override for the checked-state dot /
    /// ring color. `None` → renderer falls back to
    /// `action.primary.bg`.
    pub custom_tone: Option<gpui::Hsla>,
}

pub fn radio(id: impl Into<ElementId>, cx: &mut App) -> RadioProps {
    RadioProps {
        id: id.into(),
        checked: false,
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_toggle: None,
        has_custom_tone: false,
        custom_tone: None,
    }
}

impl RadioProps {
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

    /// Render the radio using the registered `RadioRenderer`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let r: &Arc<dyn RadioRenderer> = cx
            .renderer_arc::<RadioMarker, dyn RadioRenderer>()
            .expect("RadioRenderer registered");
        let state = RadioRenderState {
            checked: self.checked,
            disabled: self.disabled,
            has_custom_tone: self.has_custom_tone,
            custom_tone: self.custom_tone,
        };
        let bg = r.ring_bg(&state, theme);
        let border = r.ring_border(&state, theme);
        let ring_size = r.ring_size(&state, theme);
        let dot_size = r.dot_size(&state, theme);
        let dot_fg = r.dot_fg(&state, theme);
        let pill_radius = gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32);
        let mut el = div()
            .bg(bg)
            .border_1()
            .border_color(border)
            .size(ring_size)
            .rounded(pill_radius)
            .flex()
            .items_center()
            .justify_center();
        if self.checked {
            el = el.child(div().bg(dot_fg).size(dot_size).rounded(pill_radius));
        }
        let hover_bg = r.ring_hover_bg(&state, theme);
        let active_bg = r.ring_active_bg(&state, theme);
        self.apply(el)
            .hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}
