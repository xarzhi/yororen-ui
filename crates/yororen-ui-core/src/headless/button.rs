//! Headless `button` — a clickable, focusable element with no
//! bundled visual.
//!
//! ```ignore
//! div()
//!     .bg(red).rounded(8).p_2()
//!     .apply(button("save", cx).on_click(|ev, w, cx| { ... }))
//!     .child("Save")
//! ```
//!
//! `apply` is purely a11y: it wires the focus handle, then
//! the click handler. It does **not** inject any visual
//! feedback. The caller's `el` decides colors, padding,
//! radius, hover, active — every visual concern stays
//! with the caller (or the renderer, via `default_render`).

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Styled, Window, div, px,
};

// The headless `Button` marker is the same type the
// renderer registry keys on (`core::renderer::markers::Button`).
// Re-exporting it from the headless module keeps the
// `use yororen_ui_core::headless::button::Button as ButtonMarker`
// import path working in renderer code that wants the
// renderer-marker.
pub use crate::renderer::markers::Button;

/// Click handler shared by every interactive headless primitive.
pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

/// Returns a `ButtonProps` with a fresh `FocusHandle` minted from `cx`.
///
/// The caller is expected to pass the result to `.apply(div)` (or the
/// renderer's `DefaultButton::default_render`).
pub fn button(id: impl Into<ElementId>, cx: &mut App) -> ButtonProps {
    ButtonProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        on_click: None,
        disabled: false,
        clickable: true,
        variant: crate::renderer::ActionVariantKind::default(),
    }
}

#[derive(Clone)]
pub struct ButtonProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub on_click: Option<ClickCallback>,
    pub disabled: bool,
    pub clickable: bool,
    /// Action variant — `Neutral` (default) / `Primary` / `Danger`.
    /// The renderer dispatches to `action.<variant>.{bg,fg}`.
    pub variant: crate::renderer::ActionVariantKind,
}

impl ButtonProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
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

    pub fn variant(mut self, v: crate::renderer::ActionVariantKind) -> Self {
        self.variant = v;
        self
    }

    /// Wire the headless contract onto the caller's `el`.
    ///
    /// `apply` is purely a11y: it sets the element id,
    /// registers the focus handle, and (if `clickable` and not
    /// `disabled`) attaches the click handler. It does **not**
    /// inject any visual feedback — no opacity dip, no hover /
    /// active style. The caller (or the renderer via
    /// `default_render`) owns the visual.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let focus_handle = self.focus_handle.clone();
        let on_click = self.on_click.clone();
        let disabled = self.disabled;
        let clickable = self.clickable;
        let s = el.id(self.id.clone()).track_focus(&focus_handle);
        if clickable
            && !disabled
            && let Some(f) = on_click
        {
            s.on_click(move |ev, window, cx| {
                if disabled {
                    return;
                }
                f(ev, window, cx);
            })
        } else {
            s
        }
    }

    /// Render the button using the registered `ButtonRenderer`.
    ///
    /// Looks up the `XxxRenderer` registered via
    /// `cx.register_renderer_arc::<ButtonMarker, dyn ButtonRenderer>(…)`
    /// and consumes ALL of its tokens (bg / fg / padding /
    /// border / shadow / min_height / disabled_opacity /
    /// hover_bg / active_bg) to build the `Stateful<Div>`.
    ///
    /// The renderer doesn't need to know about headless — it
    /// just provides the data. The headless owns the
    /// consumption.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::button::{ButtonRenderState, ButtonRenderer};
        use crate::renderer::markers::Button as ButtonMarker;
        use crate::theme::ActiveTheme;

        let r: &Arc<dyn ButtonRenderer> = cx
            .renderer_arc::<ButtonMarker, dyn ButtonRenderer>()
            .expect("ButtonRenderer registered");
        let theme = cx.theme();
        let state = ButtonRenderState {
            variant: self.variant,
            disabled: self.disabled,
            is_rtl: false,
            has_custom_bg: false,
            has_custom_hover_bg: false,
            custom_style: None,
        };

        let bg = r.bg(&state, theme);
        let fg = r.fg(&state, theme);
        let padding = r.padding(&state, theme);
        let radius = r.border_radius(&state, theme);
        let min_h = r.min_height(&state, theme);
        let opacity = if self.disabled {
            r.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let hover_bg = r.hover_bg(&state, theme);
        let active_bg = r.active_bg(&state, theme);
        let border = r.border(&state, theme);
        let shadow = r.shadow(&state, theme);

        let mut el = div()
            .bg(bg)
            .text_color(fg)
            .px(padding.left)
            .py(padding.top)
            .rounded(radius)
            .min_h(min_h)
            .flex()
            .items_center()
            .justify_center()
            .opacity(opacity);

        if let Some(b) = border {
            // gpui-ce 0.3.3 only ships `border_1()..border_10()`
            // helpers (no arbitrary-width API on `Styled`).
            // `Pixels / Pixels -> f32` lets us recover the
            // width as f32, round it, clamp to 0..=10, then
            // dispatch to the matching `border_N` helper.
            // (Brutalism returns 3px → `.border_3()`; the
            // default renderer returns `None` → no border.)
            let w = ((b.width / px(1.0)).round() as i32).clamp(0, 10);
            match w {
                0 => {}
                1 => {
                    el = el.border_1().border_color(b.color);
                }
                2 => {
                    el = el.border_2().border_color(b.color);
                }
                3 => {
                    el = el.border_3().border_color(b.color);
                }
                4 => {
                    el = el.border_4().border_color(b.color);
                }
                5 => {
                    el = el.border_5().border_color(b.color);
                }
                6 => {
                    el = el.border_6().border_color(b.color);
                }
                7 => {
                    el = el.border_7().border_color(b.color);
                }
                8 => {
                    el = el.border_8().border_color(b.color);
                }
                9 => {
                    el = el.border_9().border_color(b.color);
                }
                _ => {
                    el = el.border_10().border_color(b.color);
                }
            }
        }
        if let Some(s) = shadow {
            el = el.shadow(vec![gpui::BoxShadow {
                color: s.color,
                offset: gpui::point(px(0.0), s.offset_y),
                blur_radius: s.blur,
                spread_radius: px(0.0),
            }]);
        }

        self.apply(el)
            .hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}
