use std::sync::Arc;

use gpui::{
    App, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::{
    component::{IconName, TextInputState, compute_input_style, icon, icon_button, text_input},
    renderer::variant::VariantState,
    renderer::{ButtonVariant, VariantKey},
    theme::{ActionVariant, ActiveTheme},
};

/// Creates a new search input element.
/// Requires an id to be set via `.id()` for internal state management.
pub fn search_input(id: impl Into<ElementId>) -> SearchInput {
    SearchInput::new().id(id)
}

type ChangeFn = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;
type SubmitFn = Arc<dyn Fn(SharedString, &mut gpui::Window, &mut App)>;

#[derive(IntoElement)]
pub struct SearchInput {
    element_id: ElementId,
    base: Div,
    placeholder: SharedString,

    disabled: bool,
    variant: ButtonVariant,

    bg: Option<Hsla>,
    border: Option<Hsla>,
    focus_border: Option<Hsla>,
    text_color: Option<Hsla>,
    height: Option<gpui::AbsoluteLength>,

    on_change: Option<ChangeFn>,
    on_submit: Option<SubmitFn>,
}

impl Default for SearchInput {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchInput {
    pub fn new() -> Self {
        Self {
            element_id: "ui:search-input".into(),
            base: div(),
            placeholder: "".into(),

            disabled: false,
            variant: ButtonVariant::default(),

            bg: None,
            border: None,
            focus_border: None,
            text_color: None,
            height: None,

            on_change: None,
            on_submit: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the visual variant of the trailing clear (`x`) button.
    /// Built-in [`ActionVariantKind`] values map to the theme palette;
    /// custom variants registered through
    /// [`crate::renderer::VariantRegistry`] are resolved at render time.
    pub fn variant(mut self, variant: impl Into<ButtonVariant>) -> Self {
        self.variant = variant.into();
        self
    }

    /// Convenience: set the variant to a custom registry key.
    pub fn custom_variant(self, key: impl Into<VariantKey>) -> Self {
        self.variant(ButtonVariant::Custom(key.into()))
    }

    pub fn on_change<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_change = Some(Arc::new(handler));
        self
    }

    pub fn on_submit<F>(mut self, handler: F) -> Self
    where
        F: 'static + Fn(SharedString, &mut gpui::Window, &mut App),
    {
        self.on_submit = Some(Arc::new(handler));
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

    pub fn focus_border(mut self, color: impl Into<Hsla>) -> Self {
        self.focus_border = Some(color.into());
        self
    }

    pub fn text_color(mut self, color: impl Into<Hsla>) -> Self {
        self.text_color = Some(color.into());
        self
    }

    pub fn height(mut self, height: gpui::AbsoluteLength) -> Self {
        self.height = Some(height);
        self
    }

    /// Generate a child element ID by combining this component's element ID with a suffix.
    pub fn child_id(&self, suffix: &str) -> ElementId {
        (self.element_id.clone(), suffix.to_string()).into()
    }
}

impl ParentElement for SearchInput {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for SearchInput {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for SearchInput {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for SearchInput {}

impl RenderOnce for SearchInput {
    fn render(self, window: &mut gpui::Window, cx: &mut App) -> impl IntoElement {
        // SearchInput requires an element ID for keyed state management.
        // Use `.id()` to provide a stable ID, or a unique ID will be generated automatically.
        let id = self.element_id.clone();
        let placeholder = self.placeholder;
        let disabled = self.disabled;
        let height = self
            .height
            .unwrap_or_else(|| cx.theme().tokens.control.button.min_height.into());
        let input_style =
            compute_input_style(cx.theme(), disabled, self.bg, self.border, self.focus_border, self.text_color);
        let on_change = self.on_change;
        let on_submit = self.on_submit;
        let variant = self.variant;

        let input_id: ElementId = (id.clone(), "ui:search-input:input").into();
        let clear_id: ElementId = (id.clone(), "ui:search-input:clear").into();

        let theme = cx.theme().clone();
        let hint = theme.content.tertiary;

        // Resolve the clear button's variant. Built-in values fall
        // through to the theme; custom variants are looked up in the
        // global VariantRegistry.
        let resolved = crate::component::ResolvedVariant::resolve(&variant, cx);
        let custom_style = resolved.custom_style;
        let variant_builtin = resolved.builtin;
        let theme_action_variant = cx.theme().action_variant(variant_builtin).clone();
        let action_variant = if let Some(s) = &custom_style {
            ActionVariant {
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

        let input_state =
            window.use_keyed_state(input_id.clone(), cx, |_, cx| TextInputState::new(cx));

        let on_change_for_input = {
            let input_state = input_state.clone();
            let on_change = on_change.clone();
            move |value: SharedString, window: &mut gpui::Window, cx: &mut App| {
                // Sync to our input_state
                input_state.update(cx, |state, cx| {
                    state.set_content(value.clone());
                    cx.notify();
                });
                // Call external handler
                if let Some(handler) = &on_change {
                    handler(value, window, cx);
                }
            }
        };

        let clear_visible = !input_state.read(cx).content().is_empty();

        let on_change_for_clear = on_change;

        let on_submit_for_input = on_submit.clone();

        let direction = cx.theme().text_direction;
        let mut base = self
            .base
            .id(id.clone())
            .flex()
            .when(direction.is_rtl(), |this| this.flex_row_reverse())
            .when(!direction.is_rtl(), |this| this.flex_row())
            .items_center()
            .gap_1()
            .h(height)
            .px_2()
            .bg(input_style.bg)
            .border_1()
            .border_color(input_style.border)
            .rounded_md()
            .focus_visible(|style| style.border_2().border_color(input_style.focus_border))
            .when(disabled, |this| this.opacity(0.6).cursor_not_allowed())
            .child(
                icon(IconName::Search)
                    .size(cx.theme().tokens.sizes.icon_md)
                    .color(hint),
            )
            .child(
                div().flex_1().h(height).child(
                    text_input(input_id)
                        .placeholder(placeholder)
                        .disabled(disabled)
                        .height(height)
                        .px_1()
                        .bg(theme.surface.base.alpha(0.0))
                        .border(theme.border.default.alpha(0.0))
                        .focus_border(theme.border.default.alpha(0.0))
                        .text_color(input_style.text_color)
                        .on_change(on_change_for_input)
                        .on_submit({
                            let on_submit = on_submit_for_input;
                            move |value, window, cx| {
                                if let Some(handler) = &on_submit {
                                    handler(value, window, cx);
                                }
                            }
                        }),
                ),
            );

        // Conditionally add clear button
        if clear_visible && !disabled {
            let clear_size = cx.theme().tokens.sizes.control_h_xs;
            base = base.child(
                div()
                    .w(clear_size)
                    .h(clear_size)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(
                        icon_button(clear_id)
                            .variant(variant.clone())
                            .icon(icon(IconName::Close))
                            .icon_size(cx.theme().tokens.sizes.icon_md)
                            .w(clear_size)
                            .h(clear_size)
                            .rounded_md()
                            .bg(action_variant.bg.alpha(0.0))
                            .hover_bg(action_variant.hover_bg)
                            .on_click({
                                let input_state = input_state.clone();
                                let on_change = on_change_for_clear;
                                move |_ev, window, cx| {
                                    input_state.update(cx, |state, cx| {
                                        state.set_content(SharedString::new_static(""));
                                        cx.notify();
                                    });

                                    if let Some(handler) = &on_change {
                                        handler(SharedString::new_static(""), window, cx);
                                    }
                                }
                            }),
                    ),
            );
        }

        base
    }
}
