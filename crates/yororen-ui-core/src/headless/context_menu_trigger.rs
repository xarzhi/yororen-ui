//! Headless `context_menu_trigger` — a span with a right-click
//! handler that requests a menu from the caller.

use std::sync::Arc;

use gpui::{App, Div, ElementId, MouseButton, Stateful, StatefulInteractiveElement, Window};

pub type ContextMenuCallback =
    Arc<dyn Fn(&gpui::Point<gpui::Pixels>, &mut gpui::Window, &mut App)>;

#[derive(Clone)]
pub struct ContextMenuTriggerProps {
    pub id: ElementId,
    pub disabled: bool,
    pub on_show: Option<ContextMenuCallback>,
}

pub fn context_menu_trigger(
    id: impl Into<ElementId>,
    _cx: &mut App,
) -> ContextMenuTriggerProps {
    ContextMenuTriggerProps {
        id: id.into(),
        disabled: false,
        on_show: None,
    }
}

impl ContextMenuTriggerProps {
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn on_show<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(&gpui::Point<gpui::Pixels>, &mut gpui::Window, &mut App),
    {
        self.on_show = Some(Arc::new(f));
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        let mut s = el.id(self.id.clone());
        if !self.disabled
            && let Some(f) = self.on_show.clone()
        {
            s = s.on_mouse_down(MouseButton::Right, move |ev, window, cx| {
                f(&ev.position, window, cx);
            });
        }
        s
    }
}
