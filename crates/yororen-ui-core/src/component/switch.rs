use std::sync::Arc;

use gpui::{
    Animation, AnimationExt, ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, StatefulInteractiveElement, Styled, div,
};

use crate::{
    animation,
    component::{
        ToggleCallback, create_internal_state, is_uncontrolled_simple, resolve_state_value_simple,
    },
    renderer::SwitchRenderState,
    theme::ActiveTheme,
};

use crate::animation::ease_in_out_clamped;

/// Creates a new switch element.
/// Requires an id to be set via `.id()` for internal state management.
///
/// # Accessibility
///
/// This component provides accessibility support:
/// - The switch is keyboard accessible (Tab to focus, Space/Enter to toggle)
/// - The on/off state is visually indicated by the thumb position
/// - Disabled state is properly conveyed to assistive technologies
///
/// For full accessibility support:
/// - Use with a `<label>` element for proper text association
/// - The component internally manages `role="switch"` and `aria-checked` state
/// - Switches are commonly used for on/off settings rather than selections
pub fn switch(id: impl Into<ElementId>) -> Switch {
    Switch::new().id(id)
}

#[derive(IntoElement)]
pub struct Switch {
    element_id: ElementId,
    base: Div,
    checked: bool,
    disabled: bool,
    on_toggle: Option<ToggleCallback>,
    tone: Option<Hsla>,
}

impl Default for Switch {
    fn default() -> Self {
        Self::new()
    }
}

impl Switch {
    pub fn new() -> Self {
        Self {
            element_id: "ui:switch".into(),
            base: div(),
            checked: false,
            disabled: false,
            on_toggle: None,
            tone: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn tone(mut self, tone: impl Into<Hsla>) -> Self {
        self.tone = Some(tone.into());
        self
    }

    pub fn on_toggle<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(bool, Option<&ClickEvent>, &mut gpui::Window, &mut gpui::App),
    {
        self.on_toggle = Some(Arc::new(handler));
        self
    }
}

impl ParentElement for Switch {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Switch {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Switch {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Switch {}

impl RenderOnce for Switch {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let disabled = self.disabled;
        let explicit_checked = self.checked;
        let on_toggle = self.on_toggle;
        let tone = self.tone;

        // Switch requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = self.element_id;

        let use_internal = is_uncontrolled_simple(on_toggle.is_some());
        let internal_checked = create_internal_state(
            window,
            cx,
            &id,
            "ui:switch:checked".to_string(),
            explicit_checked,
            use_internal,
        );

        let checked =
            resolve_state_value_simple(explicit_checked, &internal_checked, cx, use_internal);

        let theme = cx.theme();
        let r = &theme.renderers.switch;
        let state = SwitchRenderState {
            checked,
            disabled,
            has_custom_tone: tone.is_some(),
        };
        let track_bg = r.track_bg(&state, theme);
        let track_border = r.track_border(&state, theme);
        let track_hover_bg = r.track_hover_bg(&state, theme);
        let knob_bg = r.knob_bg(&state, theme);
        let focus_color = r.focus_color(&state, theme);
        let disabled_opacity = r.disabled_opacity(&state, theme);

        let track_w: f32 = r.track_w(&state, theme).into();
        let track_h: f32 = r.track_h(&state, theme).into();
        let knob_size: f32 = r.knob_size(&state, theme).into();
        let padding: f32 = r.padding(&state, theme).into();
        let travel = track_w - 2.0 * padding - knob_size;
        let knob_top = (track_h - knob_size) / 2.0;

        let mut base = self
            .base
            .id(id.clone())
            .w(r.track_w(&state, theme))
            .h(r.track_h(&state, theme))
            .rounded_full()
            .border_1()
            .border_color(track_border)
            .bg(track_bg)
            .p(r.padding(&state, theme))
            .relative()
            .focusable()
            .focus_visible(move |style| style.border_2().border_color(focus_color));

        if disabled {
            base = base.opacity(disabled_opacity).cursor_not_allowed();
        } else {
            base = base
                .cursor_pointer()
                .hover(move |this| this.bg(track_hover_bg));
        }

        let direction = cx.theme().text_direction;
        let is_rtl = direction.is_rtl();

        // Create animated knob with position transition
        // Initial position: left at 2px (padding), vertically centered
        let knob_size_px: Pixels = r.knob_size(&state, theme);
        let knob = div()
            .w(knob_size_px)
            .h(knob_size_px)
            .rounded_full()
            .bg(knob_bg)
            .absolute()
            .top(gpui::px(knob_top));

        let animated_knob = knob.with_animation(
            format!("ui:switch:knob:{}", checked),
            Animation::new(animation::duration::FAST).with_easing(ease_in_out_clamped),
            move |this, value| {
                let position = if checked { value } else { 1.0 - value };
                if is_rtl {
                    // RTL: off = right (logical start), on = left (logical end)
                    this.left(gpui::px(padding + travel - position * travel))
                } else {
                    this.left(gpui::px(padding + position * travel))
                }
            },
        );

        base.child(animated_knob).on_click(move |ev, window, cx| {
            if disabled {
                return;
            }

            if use_internal {
                if let Some(internal_checked) = &internal_checked {
                    internal_checked.update(cx, |value, _cx| *value = !*value);
                }
            } else if let Some(handler) = &on_toggle {
                handler(!explicit_checked, Some(ev), window, cx);
            }
        })
    }
}
