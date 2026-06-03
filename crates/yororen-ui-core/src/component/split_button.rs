use std::sync::Arc;

use gpui::{
    Animation, AnimationExt, ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement,
    ParentElement, Pixels, RenderOnce, StatefulInteractiveElement, Styled, div,
    prelude::FluentBuilder,
};

use crate::animation::constants::duration;
use crate::component::{ArrowDirection, Icon, IconName, button};
use crate::renderer::{ButtonVariant, VariantKey, resolve_custom_variant};
use crate::renderer::variant::VariantState;
use crate::theme::{ActionVariantKind, ActiveTheme};

use crate::animation::ease_out_quint_clamped;

/// Creates a new split button element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn split_button(id: impl Into<ElementId>) -> SplitButton {
    SplitButton::new().id(id)
}

type ClickFn = Box<dyn Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App)>;
type ActionFn = Arc<dyn Fn(SplitButtonAction, &ClickEvent, &mut gpui::Window, &mut gpui::App)>;

#[derive(Clone, Debug)]
pub struct SplitButtonOption {
    id: String,
    label: String,
}

impl SplitButtonOption {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SplitButtonAction {
    Primary,
    Option(String),
}

#[derive(IntoElement)]
pub struct SplitButton {
    element_id: ElementId,
    base: Div,
    label: String,
    on_primary: Option<ClickFn>,
    on_action: Option<ActionFn>,
    options: Vec<SplitButtonOption>,
    disabled: bool,
    variant: ButtonVariant,
    bg: Option<Hsla>,
    hover_bg: Option<Hsla>,
    menu_width: Option<Pixels>,
}

impl Default for SplitButton {
    fn default() -> Self {
        Self::new()
    }
}

impl SplitButton {
    pub fn new() -> Self {
        Self {
            element_id: "ui:split-button".into(),
            base: div(),
            label: "Action".to_string(),
            on_primary: None,
            on_action: None,
            options: Vec::new(),
            disabled: false,
            variant: ButtonVariant::default(),
            bg: None,
            hover_bg: None,
            menu_width: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the visual variant of the primary action surface. Built-in
    /// [`ActionVariantKind`] values drive the outer container's bg /
    /// hover_bg / fg; custom variants registered through
    /// [`crate::renderer::VariantRegistry`] are resolved at render time.
    /// Menu items always use the neutral palette regardless of variant.
    pub fn variant(mut self, variant: impl Into<ButtonVariant>) -> Self {
        self.variant = variant.into();
        self
    }

    /// Convenience: set the variant to a custom registry key.
    pub fn custom_variant(self, key: impl Into<VariantKey>) -> Self {
        self.variant(ButtonVariant::Custom(key.into()))
    }

    pub fn on_primary<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_primary = Some(Box::new(handler));
        self
    }

    pub fn on_action<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SplitButtonAction, &ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.on_action = Some(Arc::new(handler));
        self
    }

    pub fn option(mut self, id: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push(SplitButtonOption::new(id, label));
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = SplitButtonOption>) -> Self {
        self.options.extend(options);
        self
    }

    pub fn bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.bg = Some(fill.into());
        self
    }

    pub fn hover_bg(mut self, fill: impl Into<Hsla>) -> Self {
        self.hover_bg = Some(fill.into());
        self
    }

    pub fn menu_width(mut self, width: Pixels) -> Self {
        self.menu_width = Some(width);
        self
    }

    /// Generate a child element ID by combining this component's element ID with a suffix.
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }
}

impl ParentElement for SplitButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for SplitButton {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for SplitButton {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for SplitButton {}

impl RenderOnce for SplitButton {
    fn render(self, window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        // Extract element_id
        let id = self.element_id.clone();

        let disabled = self.disabled;
        let on_primary = self.on_primary;
        let on_action = self.on_action;
        let options = self.options;
        let label = self.label;
        let variant = self.variant;

        // Resolve the user's variant: built-in ActionVariantKind reads
        // straight from the theme; custom variants are looked up in the
        // global VariantRegistry. The lookup only affects the outer
        // container's bg / hover_bg / fg (the inner primary button is
        // forced transparent so the outer shows through).
        let custom_style: Option<Arc<dyn crate::renderer::VariantStyle>> = match &variant {
            ButtonVariant::Builtin(_) => None,
            ButtonVariant::Custom(key) => resolve_custom_variant(cx, key),
        };
        let variant_builtin = variant
            .as_builtin()
            .unwrap_or(ActionVariantKind::Neutral);
        let theme_action_variant = cx.theme().action_variant(variant_builtin).clone();
        let action_variant = if let Some(s) = &custom_style {
            crate::theme::ActionVariant {
                bg: s.bg(&VariantState { disabled }),
                hover_bg: s.bg(&VariantState { disabled }),
                active_bg: s.bg(&VariantState { disabled }),
                fg: s.fg(&VariantState { disabled }),
                disabled_bg: s.bg(&VariantState { disabled: true }),
                disabled_fg: s.fg(&VariantState { disabled: true }),
            }
        } else {
            theme_action_variant
        };
        let bg = self.bg.unwrap_or(action_variant.bg);
        let hover_bg = self.hover_bg.unwrap_or(action_variant.hover_bg);
        let menu_width = self.menu_width;
        let neutral_bg = cx.theme().action.neutral.bg.alpha(0.0);
        let neutral_hover_bg = cx.theme().action.neutral.hover_bg;
        let neutral_fg = cx.theme().action.neutral.fg;
        let border_default = cx.theme().border.default;
        let border_divider = cx.theme().border.divider;
        let surface_raised = cx.theme().surface.raised;

        // SplitButton requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.

        let primary_id: ElementId = (id.clone(), "primary").into();
        let toggle_id: ElementId = (id.clone(), "toggle").into();

        let menu_open = window.use_keyed_state(id.clone(), cx, |_window, _cx| false);
        let is_open = *menu_open.read(cx);

        let menu_open_for_button = menu_open.clone();
        let menu_open_for_outside = menu_open.clone();
        let menu_open_for_options = menu_open.clone();
        let menu_open_for_primary = menu_open.clone();
        let on_action_for_menu = on_action.clone();

        let has_options = !options.is_empty();

        let text_color = action_variant.fg;

        let direction = cx.theme().text_direction;
        let is_rtl = direction.is_rtl();

        self.base
            .id(id.clone())
            .relative()
            .flex()
            .when(is_rtl, |this| this.flex_row_reverse())
            .when(!is_rtl, |this| this.flex_row())
            .items_center()
            .rounded_lg()
            .border_1()
            .border_color(border_default)
            .bg(bg)
            .text_color(text_color)
            .when(is_open, |this| this.bg(hover_bg))
            .when(is_open && has_options, |this| {
                let id = id.clone();
                let on_action = on_action_for_menu.clone();
                let menu = div()
                    .id("ui:split-button:menu")
                    .absolute()
                    .top_full()
                    .when(is_rtl, |this| this.left_0())
                    .when(!is_rtl, |this| this.right_0())
                    .mt(cx.theme().tokens.control.popover.offset)
                    .rounded_md()
                    .border_1()
                    .border_color(border_default)
                    .bg(surface_raised)
                    .shadow_md()
                    .py_1()
                    .when_some(menu_width, |this, width| this.w(width))
                    .occlude()
                    .on_mouse_down_out(move |_ev, _window, cx| {
                        menu_open_for_outside.update(cx, |open, _cx| *open = false);
                    })
                    .children(options.into_iter().map(move |option| {
                        let option_id = option.id.clone();
                        let option_label = option.label.clone();
                        let on_action = on_action.clone();
                        let menu_open_for_option = menu_open_for_options.clone();
                        let element_id = id.clone();
                        button((element_id, format!("option-{}", option_id)))
                            .w_full()
                            .px_3()
                            .py_2()
                            .bg(neutral_bg)
                            .hover_bg(neutral_hover_bg)
                            .text_color(neutral_fg)
                            .on_click(move |ev, window, cx| {
                                menu_open_for_option.update(cx, |open, _cx| *open = false);
                                if let Some(handler) = &on_action {
                                    handler(
                                        SplitButtonAction::Option(option_id.clone()),
                                        ev,
                                        window,
                                        cx,
                                    );
                                }
                            })
                            .child(option_label)
                    }));

                let popover_offset: f32 = cx.theme().tokens.control.popover.offset.into();
                let popover_slide: f32 = cx.theme().tokens.motion.slide_distance;
                let animated_menu = menu.with_animation(
                    format!("ui:split-button:menu-{}", is_open),
                    Animation::new(duration::MENU_OPEN).with_easing(ease_out_quint_clamped),
                    move |this, value| {
                        this.opacity(value)
                            .mt(gpui::px(popover_offset - popover_slide * value))
                    },
                );

                this.child(gpui::deferred(animated_menu).with_priority(100))
            })
            .child(
                button(primary_id)
                    .variant(variant.clone())
                    .h(cx.theme().tokens.control.button.min_height)
                    .px_4()
                    .py_2()
                    .rounded_lg()
                    .when(is_rtl, |this| this.rounded_l_none())
                    .when(!is_rtl, |this| this.rounded_r_none())
                    .bg(neutral_bg)
                    .hover_bg(hover_bg)
                    .when(disabled, |this| this.cursor_not_allowed())
                    .on_click(move |ev, window, cx| {
                        if disabled {
                            return;
                        }
                        if let Some(handler) = &on_primary {
                            handler(ev, window, cx);
                        }
                        if let Some(handler) = &on_action {
                            handler(SplitButtonAction::Primary, ev, window, cx);
                        }
                        if has_options {
                            menu_open_for_primary.update(cx, |open, _cx| *open = false);
                        }
                    })
                    .child(label),
            )
            .when(has_options, |this| {
                this.child(
                    div()
                        .w(cx.theme().tokens.control.split_button.separator_w)
                        .h_full()
                        .bg(border_divider),
                )
                .child(
                    button(toggle_id)
                        .w(cx.theme().tokens.control.split_button.chevron_width)
                        .h(cx.theme().tokens.control.button.min_height)
                        .rounded_lg()
                        .when(is_rtl, |this| this.rounded_r_none())
                        .when(!is_rtl, |this| this.rounded_l_none())
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(neutral_bg)
                        .hover_bg(hover_bg)
                        .when(disabled, |this| this.cursor_not_allowed())
                        .on_click(move |_ev, _window, cx| {
                            if disabled {
                                return;
                            }

                            menu_open_for_button.update(cx, |open, _cx| *open = !*open);
                            cx.stop_propagation();
                        })
                        .child(
                            Icon::new(IconName::Arrow(ArrowDirection::Down))
                                .size(cx.theme().tokens.sizes.icon_sm)
                                .color(text_color),
                        ),
                )
            })
    }
}
