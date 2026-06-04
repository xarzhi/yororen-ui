use crate::renderer::CheckboxRenderer;
use std::sync::Arc;

use gpui::{
    Animation, AnimationExt, ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::{
    animation,
    component::{
        IconName, ToggleCallback, create_internal_state, icon, is_uncontrolled_simple,
        resolve_state_value_simple,
    },
    renderer::CheckboxRenderState,
    theme::ActiveTheme,
};

use crate::animation::ease_in_out_clamped;

/// Creates a new checkbox element.
/// Requires an id to be set via `.id()` for internal state management.
///
/// # Accessibility
///
/// This component provides accessibility support:
/// - The checkbox is keyboard accessible (Tab to focus, Space to toggle)
/// - The checked state is visually indicated with a checkmark
/// - Disabled state is properly conveyed to assistive technologies
///
/// For full accessibility support:
/// - Use with a `<label>` element for proper text association
/// - The component internally manages `role="checkbox"` and `aria-checked` state
/// - Use `aria-disabled` when the checkbox is visually disabled but focusable
pub fn checkbox(id: impl Into<ElementId>) -> Checkbox {
    Checkbox::new().id(id)
}

#[derive(IntoElement)]
pub struct Checkbox {
    element_id: ElementId,
    base: Div,
    checked: bool,
    disabled: bool,
    on_toggle: Option<ToggleCallback>,
    tone: Option<Hsla>,
}

impl Default for Checkbox {
    fn default() -> Self {
        Self::new()
    }
}

impl Checkbox {
    pub fn new() -> Self {
        Self {
            element_id: "ui:checkbox".into(),
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

impl ParentElement for Checkbox {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Checkbox {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for Checkbox {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Checkbox {}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let disabled = self.disabled;
        let explicit_checked = self.checked;
        let on_toggle = self.on_toggle;
        let tone = self.tone;

        // Checkbox requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = self.element_id;

        let use_internal = is_uncontrolled_simple(on_toggle.is_some());
        let internal_checked = create_internal_state(
            window,
            cx,
            &id,
            "ui:checkbox:checked".to_string(),
            explicit_checked,
            use_internal,
        );

        let checked =
            resolve_state_value_simple(explicit_checked, &internal_checked, cx, use_internal);

        let theme = cx.theme();
        let r: &dyn CheckboxRenderer = &**theme.renderers.get_checkbox().expect("CheckboxRenderer registered");
        let state = CheckboxRenderState {
            checked,
            disabled,
            has_custom_tone: tone.is_some(),
        };
        let box_size = r.box_size(&state, theme);
        let check_size = r.check_size(&state, theme);
        let box_bg = r.box_bg(&state, theme);
        let box_border = r.box_border(&state, theme);
        let box_hover_bg = r.box_hover_bg(&state, theme);
        let check_fg = r.check_fg(&state, theme);
        let focus_color = r.focus_color(&state, theme);
        let disabled_opacity = r.disabled_opacity(&state, theme);

        let mut base = self
            .base
            .id(id.clone())
            .w(box_size)
            .h(box_size)
            .rounded_sm()
            .border_1()
            .border_color(box_border)
            .bg(box_bg)
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
                .hover(move |this| this.bg(box_hover_bg));
        }

        // Animate check icon with opacity effect (wrap in div for animation support)
        let check_wrapper = div().child(icon(IconName::Check).size(check_size).color(check_fg));
        let animated_check = check_wrapper.with_animation(
            format!("ui:checkbox:check:{}", checked),
            Animation::new(animation::duration::FAST).with_easing(ease_in_out_clamped),
            move |this, value| this.opacity(if checked { value } else { 1.0 - value * 0.3 }),
        );

        base = base.when(checked, |this| this.child(animated_check));

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
