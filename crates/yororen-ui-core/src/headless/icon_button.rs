//! Headless `icon_button` — focusable clickable element with no
//! bundled visual.
//!
//! `apply` is purely a11y: focus handle + click handler. The
//! caller (or the renderer via `default_render`) owns every
//! visual concern, including hover / active feedback.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Styled, Window, div,
};

use crate::renderer::RendererContext;
use crate::renderer::icon_button::{IconButtonRenderState, IconButtonRenderer};
use crate::renderer::markers::IconButton as IconButtonMarker;
use crate::theme::ActiveTheme;

/// Click handler shared by every interactive headless primitive.
pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct IconButtonProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub on_click: Option<ClickCallback>,
    pub disabled: bool,
    pub variant: crate::renderer::ActionVariantKind,
}

pub fn icon_button(id: impl Into<ElementId>, cx: &mut App) -> IconButtonProps {
    IconButtonProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        on_click: None,
        disabled: false,
        variant: crate::renderer::ActionVariantKind::default(),
    }
}

impl IconButtonProps {
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }
    pub fn is_focused(&self, window: &Window) -> bool {
        self.focus_handle.is_focused(window)
    }
    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(&ClickEvent, &mut Window, &mut App),
    {
        self.on_click = Some(Arc::new(f));
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn variant(mut self, v: crate::renderer::ActionVariantKind) -> Self {
        self.variant = v;
        self
    }

    /// Wire the headless contract onto the caller's `el`.
    ///
    /// Purely a11y: id, focus, click. No visual feedback
    /// injected — caller / renderer owns hover / active.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let focus_handle = self.focus_handle.clone();
        let on_click = self.on_click.clone();
        let disabled = self.disabled;
        let s = el.id(self.id.clone()).track_focus(&focus_handle);
        if !disabled && let Some(f) = on_click {
            s.on_click(move |ev, window, cx| f(ev, window, cx))
        } else {
            s
        }
    }

    /// Render the icon button using the registered `IconButtonRenderer`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let r: &Arc<dyn IconButtonRenderer> = cx
            .renderer_arc::<IconButtonMarker, dyn IconButtonRenderer>()
            .expect("IconButtonRenderer registered");
        let state = IconButtonRenderState {
            variant: self.variant,
            disabled: self.disabled,
            has_custom_bg: false,
            has_custom_hover_bg: false,
            custom_style: None,
        };
        let bg = r.bg(&state, theme);
        let radius = r.border_radius(&state, theme);
        let opacity = if self.disabled {
            r.disabled_opacity(&state, theme)
        } else {
            1.0
        };
        let el = div()
            .bg(bg)
            .rounded(radius)
            .size(gpui::px(36.))
            .opacity(opacity)
            .flex()
            .items_center()
            .justify_center();
        let hover_bg = r.hover_bg(&state, theme);
        let active_bg = r.active_bg(&state, theme);
        self.apply(el)
            .hover(|s| s.bg(hover_bg))
            .active(|s| s.bg(active_bg))
    }
}
