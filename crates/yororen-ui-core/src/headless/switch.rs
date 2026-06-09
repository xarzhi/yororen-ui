//! Headless `switch` — checked / disabled state + on_toggle, no
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

use crate::renderer::RendererContext;
use crate::renderer::markers::Switch as SwitchMarker;
use crate::renderer::switch::{SwitchRenderState, SwitchRenderer};
use crate::theme::ActiveTheme;

/// Callback for toggle-style hooks (switch / checkbox / radio / toggle_button).
///
/// The `Option<&ClickEvent>` argument is `Some` for pointer clicks
/// and `None` for keyboard activations.
pub type ToggleCallback = Arc<dyn Fn(bool, Option<&ClickEvent>, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct SwitchProps {
    pub id: ElementId,
    pub checked: bool,
    pub disabled: bool,
    pub focus_handle: FocusHandle,
    pub on_toggle: Option<ToggleCallback>,
    /// `true` if the caller supplied a custom checked-state
    /// track color (consumed by `SwitchRenderer.has_custom_tone`).
    pub has_custom_tone: bool,
    /// Caller-supplied override for the checked-state track
    /// color. `None` → renderer falls back to
    /// `action.primary.bg`.
    pub custom_tone: Option<gpui::Hsla>,
}

pub fn switch(id: impl Into<ElementId>, cx: &mut App) -> SwitchProps {
    SwitchProps {
        id: id.into(),
        checked: false,
        disabled: false,
        focus_handle: cx.focus_handle(),
        on_toggle: None,
        has_custom_tone: false,
        custom_tone: None,
    }
}

impl SwitchProps {
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

    /// Render the switch using the registered `SwitchRenderer`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let r: &Arc<dyn SwitchRenderer> = cx
            .renderer_arc::<SwitchMarker, dyn SwitchRenderer>()
            .expect("SwitchRenderer registered");
        let state = SwitchRenderState {
            checked: self.checked,
            disabled: self.disabled,
            has_custom_tone: self.has_custom_tone,
            custom_tone: self.custom_tone,
        };
        let track = r.track_bg(&state, theme);
        let knob = r.knob_bg(&state, theme);
        let w = r.track_w(&state, theme);
        let h = r.track_h(&state, theme);
        let knob_size = r.knob_size(&state, theme);
        let pad = r.padding(&state, theme);
        let pill_radius = gpui::px(theme.get_number("tokens.radii.pill").unwrap_or(0.0) as f32);
        let mut el = div()
            .bg(track)
            .w(w)
            .h(h)
            .rounded(pill_radius)
            .p(pad)
            .flex()
            .items_center();
        if self.checked {
            el = el.justify_end();
        } else {
            el = el.justify_start();
        }
        el = el.child(div().bg(knob).size(knob_size).rounded(pill_radius));
        let track_hover = r.track_hover_bg(&state, theme);
        let track_active = r.track_active_bg(&state, theme);
        self.apply(el)
            .hover(|s| s.bg(track_hover))
            .active(|s| s.bg(track_active))
    }
}
