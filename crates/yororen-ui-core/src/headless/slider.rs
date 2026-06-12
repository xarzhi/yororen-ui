//! Headless `slider` — value in [min, max] with on_change.

use std::sync::Arc;

use gpui::{
    App, AppContext, Div, DragMoveEvent, ElementId, Empty, InteractiveElement, MouseButton,
    MouseDownEvent, Stateful, StatefulInteractiveElement, Window,
};

pub type SliderCallback = Arc<dyn Fn(f32, &mut Window, &mut App)>;

#[derive(Clone)]
pub struct SliderProps {
    pub id: ElementId,
    pub value: f32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub disabled: bool,
    pub on_change: Option<SliderCallback>,
}

pub fn slider(id: impl Into<ElementId>, _cx: &mut App) -> SliderProps {
    SliderProps {
        id: id.into(),
        value: 0.0,
        min: 0.0,
        max: 1.0,
        step: 0.01,
        disabled: false,
        on_change: None,
    }
}

impl SliderProps {
    pub fn value(mut self, v: f32) -> Self {
        self.value = v;
        self
    }
    pub fn range(mut self, min: f32, max: f32) -> Self {
        self.min = min;
        self.max = max;
        self
    }
    pub fn step(mut self, s: f32) -> Self {
        self.step = s;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn on_change<F>(mut self, f: F) -> Self
    where
        F: 'static + Send + Sync + Fn(f32, &mut Window, &mut App),
    {
        self.on_change = Some(Arc::new(f));
        self
    }
    pub fn apply(self, el: Div) -> Stateful<Div> {
        el.id(self.id)
    }

    /// Convert a window-relative x coordinate (and the slider bounds)
    /// into a stepped, clamped value.
    fn value_from_x(x: f32, track_w: f32, min: f32, max: f32, step: f32) -> f32 {
        let pct = (x / track_w).clamp(0.0, 1.0);
        let raw = min + pct * (max - min);
        let stepped = (raw / step).round() * step;
        stepped.clamp(min, max)
    }

    /// Render the slider using the registered `SliderRenderer`.
    ///
    /// The renderer produces the visual tree (track + fill + knob);
    /// the headless layer layers drag behaviour on top.
    ///
    /// Dragging is implemented with GPUI's drag API: `on_drag` starts an
    /// empty drag preview so that `on_drag_move` continues to fire while
    /// the mouse is anywhere on screen, not only while it is over the
    /// slider track.
    pub fn render(self, cx: &App) -> Stateful<Div> {
        use crate::renderer::RendererContext;
        use crate::renderer::markers::Slider as SliderMarker;
        use crate::renderer::slider::SliderRenderer;

        let r: &Arc<dyn SliderRenderer> = cx
            .renderer_arc::<SliderMarker, dyn SliderRenderer>()
            .expect("SliderRenderer registered");

        let output = r.compose(&self, cx);
        let bounds = output.track_bounds;
        let mut styled = output.visual;

        if !self.disabled {
            let min = self.min;
            let max = self.max;
            let step = self.step;

            if let Some(on_change) = self.on_change.clone() {
                let bounds_for_down = bounds.clone();
                let on_change_for_down = on_change.clone();

                // Clicking anywhere on the slider jumps the handle to that
                // position.
                styled = styled.on_mouse_down(
                    MouseButton::Left,
                    move |event: &MouseDownEvent, window, cx| {
                        let b = *bounds_for_down.lock().unwrap();
                        if let Some(b) = b {
                            let x: f32 = (event.position.x - b.left()).into();
                            let track_w: f32 = b.size.width.into();
                            let value = Self::value_from_x(x, track_w, min, max, step);
                            on_change_for_down(value, window, cx);
                        }
                    },
                );

                // Use an empty drag preview so `on_drag_move` receives
                // global mouse moves while the user is dragging, even if
                // the cursor leaves the slider bounds.
                styled = styled
                    .on_drag((), move |_, _, _: &mut Window, cx: &mut App| {
                        cx.new(|_| Empty)
                    })
                    .on_drag_move(move |event: &DragMoveEvent<()>, window, cx| {
                        let b = event.bounds;
                        let x: f32 = (event.event.position.x - b.left()).into();
                        let track_w: f32 = b.size.width.into();
                        let value = Self::value_from_x(x, track_w, min, max, step);
                        on_change(value, window, cx);
                    });
            }
        }

        styled
    }
}
