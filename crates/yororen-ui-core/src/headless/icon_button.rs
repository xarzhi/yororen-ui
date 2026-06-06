//! Headless `icon_button` — focusable clickable element with no
//! bundled visual.

use std::sync::Arc;

use gpui::{
    App, ClickEvent, Div, ElementId, FocusHandle, InteractiveElement, Stateful,
    StatefulInteractiveElement, Window,
};

/// Click handler shared by every interactive headless primitive.
pub type ClickCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + Send + Sync>;

#[derive(Clone)]
pub struct IconButtonProps {
    pub id: ElementId,
    pub focus_handle: FocusHandle,
    pub on_click: Option<ClickCallback>,
    pub disabled: bool,
    pub variant: crate::renderer::ActionVariantKind,
    /// When `true` (the default), `apply` adds a built-in
    /// opacity hover/active feedback. See
    /// `headless::button::ButtonProps::raw_hover` for the
    /// rationale; same pattern.
    pub raw_hover: bool,
}

pub fn icon_button(id: impl Into<ElementId>, cx: &mut App) -> IconButtonProps {
    IconButtonProps {
        id: id.into(),
        focus_handle: cx.focus_handle(),
        on_click: None,
        disabled: false,
        variant: crate::renderer::ActionVariantKind::default(),
        raw_hover: true,
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
    pub fn raw_hover(mut self, raw: bool) -> Self {
        self.raw_hover = raw;
        self
    }

    pub fn apply(self, el: Div) -> Stateful<Div> {
        let focus_handle = self.focus_handle.clone();
        let on_click = self.on_click.clone();
        let disabled = self.disabled;
        let raw_hover = self.raw_hover;
        let s = el.id(self.id.clone()).track_focus(&focus_handle);
        let s = if raw_hover && !disabled {
            s.hover(|mut style| { style.opacity = Some(0.9); style })
                .active(|mut style| { style.opacity = Some(0.85); style })
        } else {
            s
        };
        if !disabled && let Some(f) = on_click {
            s.on_click(move |ev, window, cx| f(ev, window, cx))
        } else {
            s
        }
    }
}
