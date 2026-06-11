//! Section 5 — Controls. Each component is wrapped in a labelled
//! `cell` so the user can identify every instance.

use std::sync::Arc;
use std::sync::Mutex;

use gpui::{
    App, Bounds, Context, Div, Element, GlobalElementId, InteractiveElement, IntoElement,
    LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, ParentElement, Path, PathBuilder,
    Pixels, Point, Style, Styled, Window, div, fill, hsla, point, px, size,
};

use yororen_ui::headless::checkbox::checkbox;
use yororen_ui::headless::label::label;
use yororen_ui::headless::radio::radio;
use yororen_ui::headless::radio_group::radio_group;
use yororen_ui::headless::slider::slider;
use yororen_ui::headless::switch::switch;

use crate::sections::cell;
use crate::sections::input_cell;
use crate::state::GalleryApp;

pub fn render(app: &mut GalleryApp, cx: &mut Context<GalleryApp>) -> Div {
    let entity = cx.entity().clone();

    // --- checkbox ---
    let entity_cb = entity.clone();
    let cb = checkbox("ctrl-cb", cx)
        .checked(app.checkbox_value)
        .on_toggle(move |v, _ev, _w, cx| {
            entity_cb.update(cx, |s, _cx| s.checkbox_value = v);
        })
        .render(cx);

    // --- switch ---
    let entity_sw = entity.clone();
    let sw = switch("ctrl-sw", cx)
        .checked(app.switch_value)
        .on_toggle(move |v, _ev, _w, cx| {
            entity_sw.update(cx, |s, _cx| s.switch_value = v);
        })
        .render(cx);

    // --- 3 radio buttons in a radio_group ---
    let rg_label = label("rg-current", format!("selected: {}", app.radio_value), cx)
        .muted(true)
        .render(cx);
    let rg_with_label = radio_group("ctrl-rg", cx)
        .name("choice")
        .selected(app.radio_value)
        .apply(div().flex().flex_row().gap(px(8.)).items_center())
        .child(rg_label);
    let rg_with_radios = (0..3).fold(rg_with_label, |acc, i| {
        let entity_r = entity.clone();
        acc.child(
            radio(format!("ctrl-radio-{i}"), cx)
                .checked(app.radio_value == i)
                .on_toggle(move |_v, _ev, _w, cx| {
                    entity_r.update(cx, |s, _cx| s.radio_value = i);
                })
                .render(cx),
        )
    });

    // --- slider (headless slider has no default_render; we
    //     build track + fill + knob + drag handlers manually).
    //     Coordinates are local: the custom `SliderElement`
    //     stores its bounds in `prepaint` so the click
    //     handlers can convert the window-relative click
    //     position to a local x via `Bounds::localize`.
    let entity_sl = entity.clone();
    let slider_value = app.slider_value;
    let slider_min = 0.0_f32;
    let slider_max = 100.0_f32;
    let slider_step = 1.0_f32;
    let slider_track_w = 240.0_f32;
    let slider_track_h = 8.0_f32;
    let slider_knob_size = 16.0_f32;
    let slider_value_for_calc = slider_value;
    let pct = ((slider_value_for_calc - slider_min) / (slider_max - slider_min)).clamp(0.0, 1.0);
    let track_bg = hsla(0.0, 0.0, 0.85, 1.0);
    let fill_bg: gpui::Hsla = gpui::rgb(0x0A0A0A).into();
    let knob_bg: gpui::Hsla = gpui::rgb(0x0A0A0A).into();

    let entity_for_slider = entity_sl.clone();
    let entity_for_slider_move = entity_sl.clone();
    let slider_bounds: Arc<Mutex<Option<Bounds<Pixels>>>> = Arc::new(Mutex::new(None));
    let slider_bounds_for_down = slider_bounds.clone();
    let slider_bounds_for_move = slider_bounds.clone();
    let slider_min_d = slider_min;
    let slider_max_d = slider_max;
    let slider_step_d = slider_step;
    let slider_track_w_d = slider_track_w;

    let slider_element = SliderElement {
        bounds: slider_bounds.clone(),
        pct,
        track_w: slider_track_w,
        track_h: slider_track_h,
        knob_size: slider_knob_size,
        track_bg,
        fill_bg,
        knob_bg,
    };

    // We wrap the Element in a Div that captures the mouse
    // events. The Element itself doesn't dispatch mouse
    // events; the Div's on_mouse_down / on_mouse_move use
    // the bounds the Element stored in prepaint.
    let slider_div = div()
        .w(px(slider_track_w))
        .h(px(24.))
        .child(slider_element)
        .on_mouse_down(MouseButton::Left, move |event: &MouseDownEvent, _window, cx| {
            let b = slider_bounds_for_down.lock().unwrap().clone();
            if let Some(b) = b
                && let Some(local) = b.localize(&event.position)
            {
                let x: f32 = local.x.into();
                let pct = (x / slider_track_w_d).clamp(0.0, 1.0);
                let raw = slider_min_d + pct * (slider_max_d - slider_min_d);
                let stepped = (raw / slider_step_d).round() * slider_step_d;
                let value = stepped.clamp(slider_min_d, slider_max_d);
                entity_for_slider.update(cx, |s, _cx| s.slider_value = value);
            }
        })
        .on_mouse_move(move |event: &MouseMoveEvent, _window, cx| {
            if !event.dragging() {
                return;
            }
            let b = slider_bounds_for_move.lock().unwrap().clone();
            if let Some(b) = b
                && let Some(local) = b.localize(&event.position)
            {
                let x: f32 = local.x.into();
                let pct = (x / slider_track_w_d).clamp(0.0, 1.0);
                let raw = slider_min_d + pct * (slider_max_d - slider_min_d);
                let stepped = (raw / slider_step_d).round() * slider_step_d;
                let value = stepped.clamp(slider_min_d, slider_max_d);
                entity_for_slider_move.update(cx, |s, _cx| s.slider_value = value);
            }
        });

    let _ = (slider_value_for_calc, pct); // keep these in scope for clarity

    let slider_track = slider("ctrl-slider", cx)
        .value(slider_value)
        .range(slider_min, slider_max)
        .step(slider_step)
        .on_change(move |v, _w, cx| {
            entity_sl.update(cx, |s, _cx| s.slider_value = v);
        })
        .apply(div().relative().child(slider_div.into_any_element()));

    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .child(div().flex().flex_row().flex_wrap().gap(px(12.)).items_center().child(input_cell("checkbox", cb, &format!("value: {}", app.checkbox_value), cx)).child(input_cell("switch", sw, &format!("value: {}", app.switch_value), cx)))
        .child(cell("radio_group (3 radios)", rg_with_radios, cx))
        .child(div().flex().flex_col().gap(px(4.)).child(cell("slider (track + knob)", slider_track, cx)).child(label("slider-lbl", format!("slider: {slider_value:.1}"), cx).muted(true).render(cx)))
}

/// A custom `Element` that paints a slider track + fill +
/// knob. The key job is `prepaint`: it stores the laid-out
/// `bounds` in the shared `Arc<Mutex<Option<Bounds>>>` so
/// the wrapping div's `on_mouse_down` / `on_mouse_move`
/// closures can convert the window-absolute click position
/// to a local x via `Bounds::localize`. Without this, the
/// slider's value mapping is wrong whenever the slider is
/// not flush with the window's left edge.
struct SliderElement {
    bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    pct: f32,
    track_w: f32,
    track_h: f32,
    knob_size: f32,
    track_bg: gpui::Hsla,
    fill_bg: gpui::Hsla,
    knob_bg: gpui::Hsla,
}

impl Element for SliderElement {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<gpui::ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = px(self.track_w).into();
        style.size.height = px(24.0).into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
        *self.bounds.lock().unwrap() = Some(bounds);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        let track_y = bounds.top() + px((24.0 - self.track_h) / 2.0);
        let knob_y = bounds.top() + px((24.0 - self.knob_size) / 2.0);
        let fill_w = px(self.pct * (self.track_w - self.knob_size));
        let knob_x = bounds.left() + px(self.pct * (self.track_w - self.knob_size));

        let track_quad = fill(
            Bounds::new(point(bounds.left(), track_y), size(px(self.track_w), px(self.track_h))),
            self.track_bg,
        );
        window.paint_quad(track_quad);

        let fill_quad = fill(
            Bounds::new(point(bounds.left(), track_y), size(fill_w, px(self.track_h))),
            self.fill_bg,
        );
        window.paint_quad(fill_quad);

        let knob_center = point(
            knob_x + px(self.knob_size / 2.0),
            knob_y + px(self.knob_size / 2.0),
        );
        let knob_path = circle_path(knob_center, px(self.knob_size / 2.0));
        window.paint_path(knob_path, self.knob_bg);
    }
}

/// Build a closed circular `Path<Pixels>` of the given `radius`
/// around `center`, made of four quarter arcs. `paint_path`
/// is the gpui-ce primitive for any non-rectangular fill —
/// `fill()` / `paint_quad()` only paint axis-aligned rectangles.
fn circle_path(center: Point<Pixels>, radius: Pixels) -> Path<Pixels> {
    let mut builder = PathBuilder::fill();
    let r = radius;
    let cx = center.x;
    let cy = center.y;
    builder.move_to(point(cx + r, cy));
    builder.arc_to(point(r, r), px(0.), false, true, point(cx, cy + r));
    builder.arc_to(point(r, r), px(0.), false, true, point(cx - r, cy));
    builder.arc_to(point(r, r), px(0.), false, true, point(cx, cy - r));
    builder.arc_to(point(r, r), px(0.), false, true, point(cx + r, cy));
    builder.close();
    builder.build().expect("valid circle path")
}

impl IntoElement for SliderElement {
    type Element = Self;
    fn into_element(self) -> Self::Element {
        self
    }
}
