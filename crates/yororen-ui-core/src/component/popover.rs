use std::sync::Arc;

use gpui::prelude::FluentBuilder;
use gpui::{
    Animation, AnimationExt, Bounds, ClickEvent, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, Styled, div,
};

use crate::component::{BoundsTrackerElement, desired_menu_left};
use crate::i18n::{I18n, TextDirection};
use crate::{animation::constants::duration, theme::ActiveTheme};

use crate::animation::ease_out_quint_clamped;

/// Defines the placement position of a popover relative to its trigger element.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PopoverPlacement {
    /// Positions the popover below the trigger, aligned to the start (left in LTR).
    BottomStart,
    /// Positions the popover below the trigger, aligned to the end (right in LTR).
    BottomEnd,
}

/// Creates a new popover element.
///
/// Popovers display floating content relative to a trigger element. Use `.trigger()` to set
/// the clickable element and `.content()` to set the popover body.
///
/// # Example
/// ```rust,ignore
/// use gpui::px;
/// use yororen_ui::component::{button, popover};
///
/// let p = popover()
///     .trigger(button().child("Open"))
///     .content(div().p_4().child("Popover content"))
///     .width(px(200.));
/// ```
pub fn popover(id: impl Into<ElementId>) -> Popover {
    Popover::new(id)
}

type CloseFn = Arc<dyn Fn(&mut gpui::Window, &mut gpui::App)>;

#[derive(IntoElement)]
pub struct Popover {
    element_id: ElementId,
    base: gpui::Div,

    open: bool,
    placement: PopoverPlacement,
    width: Option<gpui::Pixels>,

    trigger: Option<gpui::AnyElement>,
    content: Option<gpui::AnyElement>,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    on_close: Option<CloseFn>,
    /// Whether the Escape key dismisses the popover. Default: true.
    /// Set to false for non-dismissable popovers
    /// (e.g. persistent notification stacks).
    dismiss_on_escape: bool,
}

impl Default for Popover {
    fn default() -> Self {
        Self::new("ui:popover")
    }
}

impl Popover {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            element_id: id.into(),
            base: div(),

            open: false,
            placement: PopoverPlacement::BottomStart,
            width: None,

            trigger: None,
            content: None,

            bg: None,
            border: None,
            on_close: None,
            dismiss_on_escape: true,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Returns the element's ID.
    pub fn element_id(&self) -> &ElementId {
        &self.element_id
    }

    /// Generates a child element ID by combining the base element ID with a suffix.
    ///
    /// This is useful for creating unique IDs for child elements while maintaining
    /// a clear relationship to the parent component's ID.
    ///
    /// # Example
    /// ```rust,ignore
    /// let popover = popover("my-popover");
    /// let trigger_id = popover.child_id("trigger"); // "my-popover-trigger"
    /// let content_id = popover.child_id("content"); // "my-popover-content"
    /// ```
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn placement(mut self, placement: PopoverPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn width(mut self, width: gpui::Pixels) -> Self {
        self.width = Some(width);
        self
    }

    pub fn bg(mut self, color: impl Into<Hsla>) -> Self {
        self.bg = Some(color.into());
        self
    }

    pub fn border(mut self, color: impl Into<Hsla>) -> Self {
        self.border = Some(color.into());
        self
    }

    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }

    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    pub fn on_close<F>(mut self, f: F) -> Self
    where
        F: 'static + Fn(&mut gpui::Window, &mut gpui::App),
    {
        self.on_close = Some(Arc::new(f));
        self
    }

    /// Set whether the Escape key dismisses the popover. Default:
    /// `true`. Set to `false` for non-dismissable popovers
    /// (e.g. persistent notification stacks).
    pub fn dismiss_on_escape(mut self, dismiss: bool) -> Self {
        self.dismiss_on_escape = dismiss;
        self
    }
}

impl ParentElement for Popover {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for Popover {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl RenderOnce for Popover {
    #[allow(clippy::let_and_return)]
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let element_id = self.element_id;
        let id = element_id.clone();

        // Track trigger bounds for overflow protection. Must use keyed state so the entity
        // persists across renders — `BoundsTrackerElement` writes the bounds in `prepaint`
        // (which happens *after* render), so the same `Entity<Bounds<...>>` must be read in
        // render to see the previous frame's layout result.
        let trigger_bounds_state =
            window.use_keyed_state((id.clone(), "ui:popover:trigger-bounds"), cx, |_, _| {
                Bounds::<Pixels>::default()
            });

        let theme = cx.theme();
        let bg = self.bg.unwrap_or(theme.surface.raised);
        let border = self.border.unwrap_or(theme.border.default);

        let is_open = self.open;
        let placement = self.placement;
        let width = self.width;
        let on_close = self.on_close;
        let dismiss_on_escape = self.dismiss_on_escape;

        // Snapshot token values up-front so the closure doesn't borrow `theme`
        // (which keeps a reference to `cx`) while `cx` itself is moved in.
        let menu_width_default = {
            theme.tokens.control.popover.min_width + theme.tokens.control.popover.max_width / 2.0
        };
        let popover_offset: f32 = theme.tokens.control.popover.offset.into();
        let popover_slide: f32 = theme.tokens.motion.slide_distance;

        let trigger = self.trigger.unwrap_or_else(|| div().into_any_element());
        let content = self.content.unwrap_or_else(|| div().into_any_element());

        // Like Select/ComboBox, Popover is a relative container and the menu is an absolute child
        // rendered via `gpui::deferred(...)` so it is painted above.
        let trigger = self
            .base
            .id(element_id)
            .relative()
            .child(BoundsTrackerElement {
                bounds_state: trigger_bounds_state.clone(),
                inner: trigger.into_any_element(),
            })
            .when(is_open, move |this| {
                let direction = cx
                    .try_global::<I18n>()
                    .map(|i18n| i18n.text_direction())
                    .unwrap_or(TextDirection::Ltr);

                // Resolve menu width for clamping.
                let menu_width_px = width.unwrap_or(menu_width_default);
                let trigger_bounds = *trigger_bounds_state.read(cx);
                let align_end = placement == PopoverPlacement::BottomEnd;
                let menu_left =
                    desired_menu_left(trigger_bounds, menu_width_px, direction, align_end, window);
                let relative_left = menu_left - trigger_bounds.left();

                let menu = div()
                    .id((id.clone(), "ui:popover:menu"))
                    .absolute()
                    .top_full()
                    .left_0()
                    .when(relative_left != Pixels::ZERO, |this| {
                        this.left(relative_left)
                    })
                    .mt(gpui::px(popover_offset))
                    .rounded_md()
                    .overflow_hidden()
                    .border_1()
                    .border_color(border)
                    .bg(bg)
                    .shadow_md()
                    .py_1()
                    .w(menu_width_px)
                    .occlude()
                    .on_mouse_down_out({
                        let on_close = on_close.clone();
                        move |_ev, window, cx| {
                            if let Some(on_close) = &on_close {
                                on_close(window, cx);
                            }
                        }
                    })
                    .when(dismiss_on_escape, |this| {
                        this.capture_key_down({
                            let on_close = on_close.clone();
                            move |event: &gpui::KeyDownEvent, window, cx| {
                                if event.keystroke.key.eq_ignore_ascii_case("escape") {
                                    cx.stop_propagation();
                                    if let Some(on_close) = &on_close {
                                        on_close(window, cx);
                                    }
                                }
                            }
                        })
                    })
                    .child(content);

                let animated = menu.with_animation(
                    format!("ui:popover:menu:{}", is_open),
                    Animation::new(duration::MENU_OPEN).with_easing(ease_out_quint_clamped),
                    move |this, value| {
                        this.opacity(value)
                            .mt(gpui::px(popover_offset - popover_slide * value))
                    },
                );

                this.child(gpui::deferred(animated).with_priority(100))
            });

        trigger
    }
}
// Keep a stable signature for downstream; on_trigger click handling stays with caller.
#[allow(dead_code)]
fn _click_passthrough(_ev: &ClickEvent) {}
