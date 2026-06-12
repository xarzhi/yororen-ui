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
use yororen_ui_core::animation::{AnimatedPresenceState, SlideDirection};

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
                });
                ((state.clone(), ()), state)
            },
        );

        // Read/update the shared visibility state. We update it here
        // so the entity always reflects the current progress and the
        // app can query `is_visible()` to decide whether to keep the
        // overlay mounted.
        let (progress, target, is_animating, enter_config, exit_config) = self.state.update(
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

        // Save the element-local clock back.
        window.with_element_state(global_id.unwrap(), |_state, _window| ((), el_state));

        // Keep requesting frames while the transition is running.
        if is_animating {
            window.request_animation_frame();
        }

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
