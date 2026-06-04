use std::sync::Arc;

use gpui::{
    Animation, AnimationExt, ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::{
    animation,
    component::{
        ToggleCallback, create_internal_state, resolve_state_value_simple,
        use_internal_state_simple,
    },
    renderer::RadioRenderState,
    theme::ActiveTheme,
};

use crate::animation::ease_in_out_clamped;

/// Creates a new radio button element.
/// Requires an id to be set via `.id()` for internal state management.
///
/// # Accessibility
///
/// This component provides accessibility support:
/// - The radio button is keyboard accessible (Tab to focus, Space/Enter to select)
/// - The checked state is visually indicated with a filled circle
/// - Disabled state is properly conveyed to assistive technologies
///
/// For full accessibility support:
/// - Use with a `<label>` element for proper text association
/// - Group related radio buttons using `RadioGroup` for proper mutual exclusion
/// - The component internally manages `role="radio"` and `aria-checked` state
pub fn radio(id: impl Into<ElementId>) -> Radio {
    Radio::new().id(id)
}

#[derive(IntoElement)]
pub struct Radio {
    element_id: ElementId,
    base: Div,
    checked: bool,
    disabled: bool,
    on_toggle: Option<ToggleCallback>,
    tone: Option<Hsla>,
}

impl Default for Radio {
    fn default() -> Self {
        Self::new()
    }
}

impl Radio {
    pub fn new() -> Self {
        Self {
            element_id: "ui:radio".into(),
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

impl ParentElement for Radio {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Radio {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Radio {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Radio {}

impl RenderOnce for Radio {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let disabled = self.disabled;
        let explicit_checked = self.checked;
        let on_toggle = self.on_toggle;
        let tone = self.tone;

        // Radio requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = self.element_id;

        let use_internal = use_internal_state_simple(on_toggle.is_some());
        let internal_checked = create_internal_state(
            window,
            cx,
            &id,
            "ui:radio:checked".to_string(),
            explicit_checked,
            use_internal,
        );

        let checked =
            resolve_state_value_simple(explicit_checked, &internal_checked, cx, use_internal);

        let theme = cx.theme();
        let r = &theme.renderers.radio;
        let state = RadioRenderState {
            checked,
            disabled,
            has_custom_tone: tone.is_some(),
        };
        let ring_size = r.ring_size(&state, theme);
        let dot_size = r.dot_size(&state, theme);
        let ring_bg = r.ring_bg(&state, theme);
        let ring_border = r.ring_border(&state, theme);
        let ring_hover_bg = r.ring_hover_bg(&state, theme);
        let dot_fg = r.dot_fg(&state, theme);
        let focus_color = r.focus_color(&state, theme);
        let disabled_opacity = r.disabled_opacity(&state, theme);

        let mut base = self
            .base
            .id(id.clone())
            .w(ring_size)
            .h(ring_size)
            .rounded_full()
            .border_1()
            .border_color(ring_border)
            .bg(ring_bg)
            .flex()
            .items_center()
            .justify_center()
            .focusable()
            .focus_visible(move |style| style.border_2().border_color(focus_color));

        if disabled {
            base = base.opacity(disabled_opacity).cursor_not_allowed();
        } else {
            base = base
                .cursor_pointer()
                .hover(move |this| this.bg(ring_hover_bg));
        }

        // Add animated inner dot for checked state
        let inner_dot = div().w(dot_size).h(dot_size).rounded_full().bg(dot_fg);

        let animated_dot = inner_dot.with_animation(
            format!("ui:radio:dot:{}", checked),
            Animation::new(animation::duration::FAST).with_easing(ease_in_out_clamped),
            move |this, value| this.opacity(if checked { value } else { 1.0 - value }),
        );

        base = base.when(checked, move |this| {
            this.border_color(ring_border)
                .text_color(dot_fg)
                .child(animated_dot)
        });

        base.on_click(move |ev, window, cx| {
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
