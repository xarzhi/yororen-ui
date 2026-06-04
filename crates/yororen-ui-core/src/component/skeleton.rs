use crate::renderer::SkeletonRenderer;
use gpui::{
    Animation, AnimationExt, Div, ElementId, Hsla, IntoElement, ParentElement, Pixels, RenderOnce,
    Styled, div,
};

use gpui::InteractiveElement;
use gpui::prelude::FluentBuilder;

use crate::{animation::constants::duration, renderer::SkeletonRenderState, theme::ActiveTheme};

use crate::animation::ease_in_out_clamped;

/// Creates a new skeleton line element.
pub fn skeleton_line() -> SkeletonLine {
    SkeletonLine::new()
}

#[derive(IntoElement)]
pub struct SkeletonLine {
    element_id: ElementId,
    base: Div,
    width: Option<Pixels>,
    height: Pixels,
    tone: Option<Hsla>,
}

impl Default for SkeletonLine {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonLine {
    pub fn new() -> Self {
        Self {
            element_id: "ui:skeleton-line".into(),
            base: div(),
            width: None,
            height: gpui::px(0.),
            tone: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn tone(mut self, tone: impl Into<Hsla>) -> Self {
        self.tone = Some(tone.into());
        self
    }
}

impl ParentElement for SkeletonLine {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for SkeletonLine {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for SkeletonLine {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self.element_id.clone();
        let theme = cx.theme();
        let r: &dyn SkeletonRenderer = &**theme.renderers.get_skeleton().expect("SkeletonRenderer registered");
        let state = SkeletonRenderState {
            block: false,
            block_sharp: false,
        };
        let user_tone = self.tone;
        let height = {
            let h: f32 = self.height.into();
            if h > 0.0 {
                self.height
            } else {
                r.min_height(&state, theme)
            }
        };
        let radius = r.border_radius(&state, theme);
        let bg = user_tone.unwrap_or_else(|| r.bg(&state, theme));
        let motion = &theme.tokens.motion;
        let pulse_min = motion.pulse_min_opacity;
        let pulse_max = motion.pulse_max_opacity;

        let base = self
            .base
            .id(self.element_id)
            .h(height)
            .rounded(radius)
            .bg(bg)
            .when_some(self.width, |this, w| this.w(w))
            .when(self.width.is_none(), |this| this.w_full());

        base.with_animation(
            (id, "pulse"),
            Animation::new(duration::SKELETON_PULSE_1)
                .repeat()
                .with_easing(ease_in_out_clamped),
            move |this, delta| {
                let t = pulse_min + (pulse_max - pulse_min) * delta;
                this.opacity(t)
            },
        )
    }
}

/// Creates a new skeleton block element.
pub fn skeleton_block() -> SkeletonBlock {
    SkeletonBlock::new()
}

#[derive(IntoElement)]
pub struct SkeletonBlock {
    element_id: ElementId,
    base: Div,
    width: Option<Pixels>,
    height: Pixels,
    rounded: bool,
    tone: Option<Hsla>,
}

impl Default for SkeletonBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl SkeletonBlock {
    pub fn new() -> Self {
        Self {
            element_id: "ui:skeleton-block".into(),
            base: div(),
            width: None,
            height: gpui::px(0.),
            rounded: true,
            tone: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn width(mut self, width: Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: Pixels) -> Self {
        self.height = height;
        self
    }

    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    pub fn tone(mut self, tone: impl Into<Hsla>) -> Self {
        self.tone = Some(tone.into());
        self
    }
}

impl ParentElement for SkeletonBlock {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for SkeletonBlock {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for SkeletonBlock {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let id = self.element_id.clone();
        let theme = cx.theme();
        let r: &dyn SkeletonRenderer = &**theme.renderers.get_skeleton().expect("SkeletonRenderer registered");
        let state = SkeletonRenderState {
            block: true,
            block_sharp: !self.rounded,
        };
        let user_tone = self.tone;
        let height = {
            let h: f32 = self.height.into();
            if h > 0.0 {
                self.height
            } else {
                r.min_height(&state, theme)
            }
        };
        let radius = r.border_radius(&state, theme);
        let bg = user_tone.unwrap_or_else(|| r.bg(&state, theme));
        let pulse_min = theme.tokens.motion.pulse_min_opacity;
        let pulse_max = theme.tokens.motion.pulse_max_opacity;

        let base = self
            .base
            .id(self.element_id)
            .h(height)
            .rounded(radius)
            .bg(bg)
            .when_some(self.width, |this, w| this.w(w))
            .when(self.width.is_none(), |this| this.w_full());

        base.with_animation(
            (id, "pulse"),
            Animation::new(duration::SKELETON_PULSE_2)
                .repeat()
                .with_easing(ease_in_out_clamped),
            move |this, delta| {
                let t = pulse_min + (pulse_max - pulse_min) * delta;
                this.opacity(t)
            },
        )
    }
}
