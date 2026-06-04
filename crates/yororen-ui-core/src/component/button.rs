use std::sync::Arc;

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::component::{ClickCallback, HoverCallback};
use crate::renderer::variant::VariantState;
use crate::renderer::{
    ButtonRenderState, ButtonVariant, Edges, VariantKey, resolve_custom_variant,
};
use crate::theme::{ActionVariantKind, ActiveTheme};

/// Creates a new button element.
///
/// Buttons trigger actions or navigation. Use `.variant()` to change visual
/// style. Built-in variants come from [`ActionVariantKind`]
/// (`Primary`, `Danger`); custom variants are resolved through the
/// global [`crate::renderer::VariantRegistry`] (see also
/// [`.custom_variant`]).
///
/// # Example
/// ```rust,ignore
/// use yororen_ui::component::{button, ActionVariantKind};
///
/// let btn = button("my-button")
///     .variant(ActionVariantKind::Primary)
///     .child("Click me")
///     .on_click(|_ev, _window, _cx| {
///         // handle click
///     });
/// ```
///
/// Note: The first argument must be a unique, stable element ID.
pub fn button(id: impl Into<ElementId>) -> Button {
    Button::new().id(id)
}

#[derive(IntoElement)]
pub struct Button {
    element_id: ElementId,
    base: Div,

    click_fn: Option<ClickCallback>,
    hover_fn: Option<HoverCallback>,
    clickable: bool,
    disabled: bool,
    variant: ButtonVariant,

    bg: Option<Hsla>,
    hover_bg: Option<Hsla>,
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl Button {
    /// Creates a new button with default styles.
    ///
    /// Default height is 36px with horizontal and vertical padding.
    /// Default variant is [`ActionVariantKind::Neutral`].
    pub fn new() -> Self {
        Self {
            element_id: "ui:button".into(),
            base: div().px_4().py_2(),
            click_fn: None,
            hover_fn: None,
            clickable: true,
            disabled: false,
            variant: ButtonVariant::default(),
            bg: None,
            hover_bg: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the visual variant. Accepts a built-in [`ActionVariantKind`]
    /// (e.g. `.variant(ActionVariantKind::Primary)`) or a
    /// [`ButtonVariant::Custom`] for variants registered through
    /// [`crate::renderer::VariantRegistry`].
    pub fn variant(mut self, variant: impl Into<ButtonVariant>) -> Self {
        self.variant = variant.into();
        self
    }

    /// Convenience: set the variant to a custom registry key.
    pub fn custom_variant(self, key: impl Into<VariantKey>) -> Self {
        self.variant(ButtonVariant::Custom(key.into()))
    }

    pub fn on_click<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(&ClickEvent, &mut gpui::Window, &mut gpui::App),
    {
        self.click_fn = Some(Arc::new(listener));
        self
    }

    pub fn on_hover<F>(mut self, listener: F) -> Self
    where
        F: 'static + Fn(bool, &mut gpui::Window, &mut gpui::App),
    {
        self.hover_fn = Some(Arc::new(listener));
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
}

impl Styled for Button {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl ParentElement for Button {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl InteractiveElement for Button {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for Button {}

impl RenderOnce for Button {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let clickable = self.clickable;
        let disabled = self.disabled;
        let click_fn = self.click_fn;
        let hover_fn = self.hover_fn;
        let bg = self.bg;
        let hover_bg = self.hover_bg;
        let variant = self.variant;
        let direction = cx.theme().text_direction;

        // Resolve a custom variant up-front so the renderer (and the
        // inline hover fallback below) can both consult it.
        let custom_style: Option<Arc<dyn crate::renderer::VariantStyle>> = match &variant {
            ButtonVariant::Builtin(_) => None,
            ButtonVariant::Custom(key) => resolve_custom_variant(cx, key),
        };
        let variant_builtin: ActionVariantKind =
            variant.as_builtin().unwrap_or(ActionVariantKind::Neutral);

        // button visuals go through ButtonRenderer.
        let theme = cx.theme();
        let r = &theme.renderers.button;
        let state = ButtonRenderState {
            variant: variant_builtin,
            disabled,
            is_rtl: direction.is_rtl(),
            has_custom_bg: bg.is_some(),
            has_custom_hover_bg: hover_bg.is_some(),
            custom_style: custom_style.clone(),
        };
        let bg_color = r.bg(&state, theme);
        let fg_color = r.fg(&state, theme);
        let padding: Edges<gpui::Pixels> = r.padding(&state, theme);
        let radius = r.border_radius(&state, theme);
        let min_height = r.min_height(&state, theme);

        // For hover, prefer the user's custom hover_bg; otherwise let
        // the custom variant provide its base bg as a sensible hover
        // default, falling back to the built-in theme hover palette
        // (mirroring v0.3 behavior).
        let hover_bg_color = hover_bg.unwrap_or_else(|| {
            if let Some(s) = &custom_style {
                s.bg(&VariantState { disabled })
            } else {
                let av = theme.action_variant(variant_builtin);
                if disabled {
                    av.disabled_bg
                } else {
                    av.hover_bg
                }
            }
        });

        self.base
            .id(self.element_id)
            .h(min_height)
            .rounded(radius)
            .px(padding.left)
            .py(padding.top)
            .flex()
            .when(direction.is_rtl(), |this| this.flex_row_reverse())
            .when(!direction.is_rtl(), |this| this.flex_row())
            .items_center()
            .justify_center()
            .bg(bg_color)
            .text_color(fg_color)
            .when(clickable && !disabled, |this| this.cursor_pointer())
            .when(disabled, |this| this.cursor_not_allowed())
            .on_click(move |ev, window, cx| {
                if disabled {
                    return;
                }
                if clickable && let Some(f) = &click_fn {
                    f(ev, window, cx);
                }
            })
            .when(hover_fn.is_some(), move |this| {
                let hover_fn = hover_fn;
                this.on_hover(move |active, window, cx| {
                    if let Some(handler) = &hover_fn {
                        handler(*active, window, cx);
                    }
                })
            })
            .hover(move |this| this.bg(hover_bg_color))
    }
}
