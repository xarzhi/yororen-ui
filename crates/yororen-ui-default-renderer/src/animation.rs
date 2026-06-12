//! Renderer-side animation helpers.
//!
//! These types live in the renderer crate because they deal with
//! pixels, transforms, and `gpui::Element` — all visual concerns.
//! The data layer (`AnimatedVisibility`) stays in
//! `yororen_ui_core::animation`.

use std::time::{Duration, Instant};

use gpui::{
    AnyElement, App, Div, Element, ElementId, Entity, GlobalElementId, InspectorElementId,
    InteractiveElement, IntoElement, LayoutId, Pixels, Styled, Window, px,
};
use yororen_ui_core::animation::{AnimatedPresenceState, AnimationConfig, SlideDirection};

/// A custom element that keeps a child mounted while it exits and
/// applies a fade + slide animation driven by an
/// [`AnimatedVisibility`] held in a `gpui::Entity<S>`.
///
/// This is the renderer-side half of the presence animation system.
/// The headless state owns *when* to open/close; this element owns
/// *how it looks* while doing so.
pub struct AnimatedPresenceElement<S: AnimatedPresenceState> {
    pub state: Entity<S>,
    pub id: ElementId,
    /// Direction the child slides *from* while entering (and *to*
    /// while exiting).
    pub direction: SlideDirection,
    pub distance: Pixels,
    child: Option<Div>,
}

impl<S: AnimatedPresenceState> AnimatedPresenceElement<S> {
    pub fn new(
        state: Entity<S>,
        id: impl Into<ElementId>,
        direction: SlideDirection,
        distance: Pixels,
        child: Div,
    ) -> Self {
        Self {
            state,
            id: id.into(),
            direction,
            distance,
            child: Some(child),
        }
    }
}

impl<S: AnimatedPresenceState> IntoElement for AnimatedPresenceElement<S> {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

#[derive(Clone)]
struct PresenceElementState {
    start: Instant,
    previous_target: bool,
    previous_progress: f32,
}

impl<S: AnimatedPresenceState> Element for AnimatedPresenceElement<S> {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let now = Instant::now();

        // Read or initialize per-element animation bookkeeping.
        let (mut el_state, _) = window.with_element_state(
            global_id.unwrap(),
            |state: Option<PresenceElementState>, _window| {
                let state = state.unwrap_or(PresenceElementState {
                    start: now,
                    previous_target: false,
                    previous_progress: 0.0,
                });
                ((state.clone(), ()), state)
            },
        );

        // Read/update the shared visibility state. We update it here
        // so the entity always reflects the current progress and the
        // app can query `is_visible()` to decide whether to keep the
        // overlay mounted.
        let (progress, target, _is_animating, enter_config, exit_config) = self.state.update(
            cx,
            |s, _cx| {
                let v = s.visibility_mut();
                let target = v.target;

                if el_state.previous_target != target {
                    // Target changed since last frame: reset the
                    // element-local animation clock, but keep the
                    // current progress so the transition is smooth.
                    el_state.start = now;
                    el_state.previous_target = target;
                }

                let dt = el_state.start.elapsed();
                v.update(dt);

                (
                    v.progress,
                    v.target,
                    v.is_animating(),
                    v.enter_config.clone(),
                    v.exit_config.clone(),
                )
            },
        );

        // Request another frame whenever progress changed. This keeps
        // the animation running while it is active, and schedules one
        // final frame after it reaches a boundary (e.g. progress == 0)
        // so the parent can re-read `is_visible()` and unmount.
        if el_state.previous_progress != progress {
            window.request_animation_frame();
            el_state.previous_progress = progress;
        }

        // Save the element-local clock back.
        window.with_element_state(global_id.unwrap(), |_state, _window| ((), el_state));

        let config = if target { enter_config } else { exit_config };
        let eased = (config.easing)(progress);

        // Apply fade + slide transform based on the current phase.
        let distance_f: f32 = self.distance.into();
        let translate = distance_f * (1.0 - eased);

        let child = self.child.take().expect("AnimatedPresenceElement::request_layout called once");
        let mut styled = child.id(self.id.clone()).opacity(progress);

        match self.direction {
            SlideDirection::Left => styled = styled.ml(px(-translate)),
            SlideDirection::Right => styled = styled.ml(px(translate)),
            SlideDirection::Up => styled = styled.mt(px(-translate)),
            SlideDirection::Down => styled = styled.mt(px(translate)),
        }

        let mut element = styled.into_any_element();
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

/// Apply a one-shot fade-in animation to a `Div`.
///
/// This is the simplest animation path: the element mounts, fades in,
/// and stays at full opacity. It is used by components whose state is
/// not entity-backed (e.g. the `Overlay` scrim when the caller mounts
/// it conditionally).
pub fn fade_in_on_mount(
    el: Div,
    id: impl Into<ElementId>,
    duration: Duration,
    easing: fn(f32) -> f32,
) -> gpui::AnimationElement<Div> {
    use gpui::AnimationExt;
    let animation = gpui::Animation::new(duration).with_easing(easing);
    el.with_animation(id, animation, move |this, progress| {
        let eased = easing(progress);
        this.opacity(eased)
    })
}

// =====================================================================
// Boolean transition elements — used by toggle controls (switch,
// checkbox) whose state is a plain `bool` rather than an entity-backed
// `AnimatedVisibility`.
// =====================================================================

#[derive(Clone)]
struct BooleanElementState {
    start: Instant,
    previous_value: bool,
    previous_progress: f32,
}

impl BooleanElementState {
    fn progress(&self, value: bool, config: &AnimationConfig) -> f32 {
        let duration_secs = config.duration.as_secs_f32();
        let rate = if duration_secs > 0.0 {
            self.start.elapsed().as_secs_f32() / duration_secs
        } else {
            1.0
        };
        if value {
            (self.previous_progress + rate).min(1.0)
        } else {
            (self.previous_progress - rate).max(0.0)
        }
    }
}

/// A custom element that animates a child's opacity based on a
/// boolean value.
///
/// When `value` is `true` the child fades in; when `value` is `false`
/// it fades out. The element keeps its own element-local animation
/// clock so the transition is smooth even though the underlying state
/// is not entity-backed.
pub struct AnimatedOpacityElement {
    pub id: ElementId,
    pub value: bool,
    child: Option<Div>,
    pub config: AnimationConfig,
}

impl AnimatedOpacityElement {
    pub fn new(id: impl Into<ElementId>, value: bool, child: Div) -> Self {
        Self {
            id: id.into(),
            value,
            child: Some(child),
            config: AnimationConfig::default(),
        }
    }

    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }
}

impl IntoElement for AnimatedOpacityElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for AnimatedOpacityElement {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let now = Instant::now();
        let (progress, is_animating) = window.with_element_state(
            global_id.unwrap(),
            |state: Option<BooleanElementState>, _window| {
                let mut state = state.unwrap_or(BooleanElementState {
                    start: now,
                    previous_value: self.value,
                    previous_progress: if self.value { 1.0 } else { 0.0 },
                });
                if state.previous_value != self.value {
                    state.start = now;
                    state.previous_value = self.value;
                }
                let progress = state.progress(self.value, &self.config);
                let is_animating =
                    (self.value && progress < 1.0) || (!self.value && progress > 0.0);
                state.previous_progress = progress;
                ((progress, is_animating), state)
            },
        );

        if is_animating {
            window.request_animation_frame();
        }

        let eased = (self.config.easing)(progress);
        let child = self
            .child
            .take()
            .expect("AnimatedOpacityElement::request_layout called once");
        let mut styled = child.id(self.id.clone()).opacity(eased);
        if progress <= 0.0 {
            styled = styled.invisible();
        }
        let mut element = styled.into_any_element();
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

/// A custom element that animates a child's left margin based on a
/// boolean value.
///
/// When `value` is `true` the child slides right by `distance`; when
/// `value` is `false` it slides back to `margin-left: 0`. This is used
/// for switch knobs.
pub struct AnimatedMarginElement {
    pub id: ElementId,
    pub value: bool,
    pub distance: Pixels,
    child: Option<Div>,
    pub config: AnimationConfig,
}

impl AnimatedMarginElement {
    pub fn new(id: impl Into<ElementId>, value: bool, distance: Pixels, child: Div) -> Self {
        Self {
            id: id.into(),
            value,
            distance,
            child: Some(child),
            config: AnimationConfig::default(),
        }
    }

    pub fn with_config(mut self, config: AnimationConfig) -> Self {
        self.config = config;
        self
    }
}

impl IntoElement for AnimatedMarginElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for AnimatedMarginElement {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let now = Instant::now();
        let (progress, is_animating) = window.with_element_state(
            global_id.unwrap(),
            |state: Option<BooleanElementState>, _window| {
                let mut state = state.unwrap_or(BooleanElementState {
                    start: now,
                    previous_value: self.value,
                    previous_progress: if self.value { 1.0 } else { 0.0 },
                });
                if state.previous_value != self.value {
                    state.start = now;
                    state.previous_value = self.value;
                }
                let progress = state.progress(self.value, &self.config);
                let is_animating =
                    (self.value && progress < 1.0) || (!self.value && progress > 0.0);
                state.previous_progress = progress;
                ((progress, is_animating), state)
            },
        );

        if is_animating {
            window.request_animation_frame();
        }

        let eased = (self.config.easing)(progress);
        let distance_f: f32 = self.distance.into();
        let translate = distance_f * eased;

        let child = self
            .child
            .take()
            .expect("AnimatedMarginElement::request_layout called once");
        let mut element = child.id(self.id.clone()).ml(px(translate)).into_any_element();
        (element.request_layout(window, cx), element)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: gpui::Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}
