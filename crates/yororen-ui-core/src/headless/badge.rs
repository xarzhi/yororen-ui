//! Headless `badge` — small inline label with a variant flag set.

use gpui::{Div, ElementId, InteractiveElement, SharedString, Stateful};

#[derive(Clone, Debug)]
pub struct BadgeProps {
    pub id: ElementId,
    pub text: String,
    pub variant: BadgeVariant,
    pub icon: Option<SharedString>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeVariant {
    #[default]
    Neutral,
    Success,
    Warning,
    Danger,
    Info,
}

pub fn badge(id: impl Into<ElementId>, text: impl Into<String>, _cx: &mut gpui::App) -> BadgeProps {
    BadgeProps {
        id: id.into(),
        text: text.into(),
        variant: BadgeVariant::default(),
        icon: None,
    }
}

impl BadgeProps {
    pub fn variant(mut self, v: BadgeVariant) -> Self {
        self.variant = v;
        self
    }
    pub fn icon(mut self, i: impl Into<SharedString>) -> Self {
        self.icon = Some(i.into());
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
