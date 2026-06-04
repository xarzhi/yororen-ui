use crate::renderer::LabelRenderer;
use gpui::{
    Div, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, div, prelude::FluentBuilder,
};

use crate::renderer::LabelRenderState;
use crate::rtl;
use crate::theme::ActiveTheme;

pub fn label(text: impl Into<SharedString>) -> Label {
    Label::new(text)
}

#[derive(IntoElement)]
pub struct Label {
    element_id: ElementId,
    base: Div,
    text: SharedString,

    muted: bool,
    strong: bool,
    inherit_color: bool,
    mono: bool,
    ellipsis: bool,
    wrap: bool,
    max_lines: Option<usize>,

    preview_lines: Option<usize>,
}

impl Label {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            element_id: "ui:label".into(),
            base: div(),
            text: text.into(),

            muted: false,
            strong: false,
            inherit_color: false,
            mono: false,
            ellipsis: false,
            wrap: false,
            max_lines: None,

            preview_lines: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn muted(mut self, value: bool) -> Self {
        self.muted = value;
        self
    }

    pub fn strong(mut self, value: bool) -> Self {
        self.strong = value;
        self
    }

    pub fn inherit_color(mut self, value: bool) -> Self {
        self.inherit_color = value;
        self
    }

    pub fn mono(mut self, value: bool) -> Self {
        self.mono = value;
        self
    }

    pub fn ellipsis(mut self, value: bool) -> Self {
        self.ellipsis = value;
        self
    }

    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }

    pub fn lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }

    /// Render a multi-line preview that clamps to `lines`.
    ///
    /// This is designed for previews of potentially multi-paragraph content (news, descriptions).
    /// It will:
    ///
    /// - Keep only the first paragraph (split on a blank line).
    /// - Trim trailing whitespace.
    /// - Apply line clamping to `lines`.
    ///
    /// Use the original full text in a modal/popover when the user clicks "read more".
    pub fn preview_lines(mut self, lines: usize) -> Self {
        self.preview_lines = Some(lines);
        self
    }
}

impl ParentElement for Label {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Label {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let direction = cx.theme().text_direction;
        let theme = cx.theme();
        let r: &dyn LabelRenderer = &**theme.renderers.get_label().expect("LabelRenderer registered");
        let state = LabelRenderState {
            muted: self.muted,
            strong: self.strong,
            mono: self.mono,
            inherit_color: self.inherit_color,
        };
        let color = r.color(&state, theme);
        let strong_weight = r.strong_weight(&state, theme);
        let mono_family = r.family_mono(&state, theme);
        let mut temp = self.base;
        let has_custom_align = temp
            .style()
            .text
            .as_ref()
            .is_some_and(|t| t.text_align.is_some());
        let mut base = temp
            .id(self.element_id)
            .when(!has_custom_align, |this| {
                this.text_align(rtl::text_align_start(direction))
            })
            .when(self.strong, |this| this.font_weight(strong_weight))
            .when(self.mono, |this| this.font_family(mono_family))
            .when(self.ellipsis, |this| this.truncate())
            // If wrap is enabled and ellipsis is not, allow text to wrap naturally
            .when(self.wrap && !self.ellipsis, |this| {
                this.overflow_x_hidden()
                    .overflow_y_hidden()
                    .whitespace_normal()
            })
            // If both are provided, `preview_lines` wins: it also controls the line clamp.
            .when_some(self.preview_lines, |this, lines| {
                this.relative().line_clamp(lines)
            })
            .when(self.preview_lines.is_none(), |this| {
                this.when_some(self.max_lines, |this, lines| this.line_clamp(lines))
            });

        if let Some(_lines) = self.preview_lines {
            let full = self.text.as_ref();
            let mut paragraphs = full.split("\n\n");
            let first_paragraph = paragraphs.next().unwrap_or("");

            let trimmed = first_paragraph.trim_end();

            let preview_text: SharedString = if trimmed.is_empty() {
                self.text
            } else {
                // Prevent previews from accidentally showing the next paragraph.
                SharedString::from(trimmed.replace('\n', " "))
            };

            base = base.child(preview_text);
        } else {
            base = base.child(self.text);
        }

        if self.inherit_color {
            base
        } else {
            base.text_color(color)
        }
    }
}
