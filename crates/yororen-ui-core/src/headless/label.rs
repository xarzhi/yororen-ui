//! Headless `label` — pure text + flag set, no visual.

use std::sync::Arc;

use gpui::{App, Div, ElementId, InteractiveElement, ParentElement, Stateful, Styled};

use crate::renderer::RendererContext;
use crate::renderer::label::{LabelRenderState, LabelRenderer};
use crate::renderer::markers::Label as LabelMarker;
use crate::theme::ActiveTheme;

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

    /// Render the label using the registered `LabelRenderer`.
    ///
    /// Looks up the renderer via
    /// `cx.renderer_arc::<LabelMarker, dyn LabelRenderer>()` and
    /// consumes all of its tokens (color / strong_weight /
    /// family_mono) to build the `Stateful<Div>`.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        let theme = cx.theme();
        let r: &Arc<dyn LabelRenderer> = cx
            .renderer_arc::<LabelMarker, dyn LabelRenderer>()
            .expect("LabelRenderer registered");
        let state = LabelRenderState {
            muted: self.muted,
            strong: self.strong,
            mono: self.mono,
            inherit_color: self.inherit_color,
            ellipsis: self.ellipsis,
            wrap: self.wrap,
            max_lines: self.max_lines,
        };
        let color = r.color(&state, theme);
        let weight = r.strong_weight(&state, theme);
        let family = r.family_mono(&state, theme);
        let mut el = gpui::div();
        if !self.inherit_color {
            el = el.text_color(color);
        }
        if self.strong {
            el = el.font_weight(weight);
        }
        if self.mono {
            el = el.font_family(family);
        }
        if self.ellipsis {
            el = el.overflow_hidden().text_ellipsis().whitespace_nowrap();
        }
        if self.wrap {
            el = el.whitespace_normal();
        }
        if let Some(n) = self.max_lines {
            el = el.line_clamp(n).overflow_hidden();
        }
        el = el.child(self.text.clone());
        self.apply(el)
    }
}
