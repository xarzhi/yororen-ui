//! Headless `icon_button` — focusable clickable element with no
//! bundled visual.
//!
//! `apply` is purely a11y: focus handle + click handler. The
//! caller (or the renderer via `default_render`) owns every
//! visual concern, including hover / active feedback.
//!
//! For the common case where the caller wants a styled icon
//! button, set the icon via `.icon(source)` and call
//! `.render(cx)`. The headless wraps the icon with the
//! renderer's tokens (size / radius / bg / fg). The icon's
//! colour is read from the renderer's `fg` so it stays
//! readable on the variant's bg.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Pixels, Stateful,
    StatefulInteractiveElement, Window,
};

use super::icon::IconSource;
use crate::renderer::icon_button::IconButtonRenderer;

/// Click handler shared by every interactive headless primitive.
pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct IconButtonProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub on_click: Option<ClickCallback>,
    pub disabled: bool,
    pub variant: crate::renderer::ActionVariantKind,
    /// Optional icon source. Used by `render` to render the
    /// icon inline. `apply` is unaffected — the caller can
    /// still chain `.child(icon(...))` for the a11y path.
    pub icon: Option<IconSource>,
    /// Pixel size of the icon, when `icon` is set.
    pub icon_size: Pixels,
}

pub fn icon_button(id: impl Into<ElementId>, cx: &mut App) -> IconButtonProps {
    IconButtonProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        on_click: None,
        disabled: false,
        variant: crate::renderer::ActionVariantKind::default(),
        icon: None,
        icon_size: gpui::px(16.),
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

    /// Set the icon source. Consumed by `render` — the icon is
    /// rendered inline with the variant's `fg` colour (so the
    /// icon stays readable on the bg without the caller having
    /// to know the right colour).
    pub fn icon(mut self, source: IconSource) -> Self {
        self.icon = Some(source);
        self
    }

    /// Override the icon's pixel size. Default: 16.
    pub fn icon_size(mut self, size: Pixels) -> Self {
        self.icon_size = size;
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
    ///
    /// Data flow is one-way: the renderer takes the full
    /// `IconButtonProps` and returns a fully-built `Stateful<Div>`
    /// (visuals + optional icon + hover / active + id + focus).
    /// This method only layers `on_click` on top. **No** token
    /// values are pulled from the renderer in headless.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::IconButton as IconButtonMarker;

        let r: &Arc<dyn IconButtonRenderer> = cx
            .renderer_arc::<IconButtonMarker, dyn IconButtonRenderer>()
            .expect("IconButtonRenderer registered");

        let mut styled = r.compose(&self, &self.focus_handle, cx);
        if !self.disabled
            && let Some(f) = self.on_click.clone()
        {
            styled = styled.on_click(move |ev, window, cx| f(ev, window, cx));
        }
        styled
    }
}
