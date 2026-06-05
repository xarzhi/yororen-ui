//! Headless `label` — pure text + flag set, no visual.

use gpui::{App, Div, ElementId, InteractiveElement, Stateful};

#[derive(Clone, Debug)]
pub struct LabelProps {
    pub id: ElementId,
    pub text: String,
    pub muted: bool,
    pub strong: bool,
    pub inherit_color: bool,
    pub mono: bool,
    pub ellipsis: bool,
    pub wrap: bool,
    pub max_lines: Option<usize>,
}

pub fn label(id: impl Into<ElementId>, text: impl Into<String>, _cx: &mut App) -> LabelProps {
    LabelProps {
        id: id.into(),
        text: text.into(),
        muted: false,
        strong: false,
        inherit_color: false,
        mono: false,
        ellipsis: false,
        wrap: false,
        max_lines: None,
    }
}

impl LabelProps {
    pub fn muted(mut self, v: bool) -> Self {
        self.muted = v;
        self
    }
    pub fn strong(mut self, v: bool) -> Self {
        self.strong = v;
        self
    }
    pub fn inherit_color(mut self, v: bool) -> Self {
        self.inherit_color = v;
        self
    }
    pub fn mono(mut self, v: bool) -> Self {
        self.mono = v;
        self
    }
    pub fn ellipsis(mut self, v: bool) -> Self {
        self.ellipsis = v;
        self
    }
    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
    pub fn max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }

    /// Headless apply. The caller decides the text color / weight /
    /// family; this only adds the `id`. The text itself is read by
    /// the caller via `props.text` and composed as a child of `el`.
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }
}
