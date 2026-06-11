//! Headless `slider` — value in [min, max] with on_change.

use std::sync::Arc;

use gpui::{
    App, Div, ElementId, InteractiveElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    Stateful, Window,
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

    /// Render the slider using the registered `SliderRenderer`.
    ///
    /// The renderer produces the visual tree (track + fill + knob);
    /// the headless layer layers drag behaviour on top.
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
            let on_change = self.on_change.clone();
            let bounds_for_down = bounds.clone();
            let bounds_for_move = bounds.clone();

            styled = styled.on_mouse_down(
                MouseButton::Left,
                move |event: &MouseDownEvent, window, cx| {
                    let b = bounds_for_down.lock().unwrap().clone();
                    if let Some(b) = b
                        && let Some(local) = b.localize(&event.position)
                    {
                        let x: f32 = local.x.into();
                        let track_w: f32 = b.size.width.into();
                        let pct = (x / track_w).clamp(0.0, 1.0);
                        let raw = min + pct * (max - min);
                        let stepped = (raw / step).round() * step;
                        let value = stepped.clamp(min, max);
                        if let Some(f) = on_change.clone() {
                            f(value, window, cx);
                        }
                    }
                },
            );

            if let Some(f) = self.on_change.clone() {
                styled = styled.on_mouse_move(move |event: &MouseMoveEvent, window, cx| {
                    if !event.dragging() {
                        return;
                    }
                    let b = bounds_for_move.lock().unwrap().clone();
                    if let Some(b) = b
                        && let Some(local) = b.localize(&event.position)
                    {
                        let x: f32 = local.x.into();
                        let track_w: f32 = b.size.width.into();
                        let pct = (x / track_w).clamp(0.0, 1.0);
                        let raw = min + pct * (max - min);
                        let stepped = (raw / step).round() * step;
                        let value = stepped.clamp(min, max);
                        f(value, window, cx);
                    }
                });
            }
        }

        styled
    }
}
