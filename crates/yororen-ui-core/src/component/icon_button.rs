use std::sync::Arc;

use gpui::{
    ClickEvent, Div, ElementId, Hsla, InteractiveElement, IntoElement, ParentElement, Pixels,
    RenderOnce, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder,
};

use crate::{
    component::{ClickCallback, HoverCallback, Icon, compute_action_style_with_custom},
    renderer::{ButtonVariant, VariantKey, resolve_custom_variant},
    theme::{ActionVariantKind, ActiveTheme},
};

/// Creates a new icon button element.
///
/// Icon buttons display an icon in a compact, clickable container. They are commonly used for
/// actions like close, settings, or navigation. Use `.variant()` to change visual style.
///
/// # Example
/// ```rust,ignore
/// use yororen_ui::component::{icon_button, Icon, IconName};
/// use yororen_ui::theme::ActionVariantKind;
///
/// let btn = icon_button("my-icon-button")
///     .icon(Icon::new(IconName::Close))
///     .variant(ActionVariantKind::Danger)
///     .on_click(|_ev, _window, _cx| {
///         // handle click
///     });
/// ```
pub fn icon_button(id: impl Into<ElementId>) -> IconButton {
    IconButton::new().id(id)
}

#[derive(IntoElement)]
pub struct IconButton {
    element_id: ElementId,
    base: Div,
    icon: Option<Icon>,

    click_fn: Option<ClickCallback>,
    hover_fn: Option<HoverCallback>,
    clickable: bool,
    disabled: bool,
    variant: ButtonVariant,

    bg: Option<Hsla>,
    hover_bg: Option<Hsla>,
    icon_size: Option<Pixels>,
}

impl IconButton {
    pub fn new() -> Self {
        Self {
            element_id: "ui:icon-button".into(),
            base: div(),
            icon: None,

            click_fn: None,
            hover_fn: None,
            clickable: true,
            disabled: false,
            variant: ButtonVariant::default(),

            bg: None,
            hover_bg: None,
            icon_size: None,
        }
    }
}

impl Default for IconButton {
    fn default() -> Self {
        Self::new()
    }
}

impl IconButton {
    pub fn icon(mut self, icon: impl Into<Icon>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.element_id = id.into();
        self
    }

    /// Alias for `id(...)`. Use `key(...)` when you want to emphasize state identity.
    pub fn key(self, key: impl Into<ElementId>) -> Self {
        self.id(key)
    }

    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

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

    pub fn icon_size(mut self, size: Pixels) -> Self {
        self.icon_size = Some(size);
        self
    }
}

impl ParentElement for IconButton {
    fn extend(&mut self, elements: impl IntoIterator<Item = gpui::AnyElement>) {
        self.base.extend(elements);
    }
}

impl Styled for IconButton {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.base.style()
    }
}

impl InteractiveElement for IconButton {
    fn interactivity(&mut self) -> &mut gpui::Interactivity {
        self.base.interactivity()
    }
}

impl StatefulInteractiveElement for IconButton {}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        let clickable = self.clickable;
        let click_fn = self.click_fn;
        let hover_fn = self.hover_fn;
        let bg = self.bg;
        let hover_bg = self.hover_bg;
        let disabled = self.disabled;
        let variant = self.variant;
        let icon_size = self.icon_size;

        let custom_style = match &variant {
            ButtonVariant::Builtin(_) => None,
            ButtonVariant::Custom(key) => resolve_custom_variant(cx, key),
        };
        let variant_builtin = variant
            .as_builtin()
            .unwrap_or(ActionVariantKind::Neutral);

        let action_style = compute_action_style_with_custom(
            cx.theme(),
            variant_builtin,
            custom_style,
            disabled,
            bg,
            hover_bg,
        );

        self.base
            .id(self.element_id)
            .w(cx.theme().tokens.control.icon_button.min_size)
            .h(cx.theme().tokens.control.icon_button.min_size)
            .rounded_md()
            .flex()
            .items_center()
            .justify_center()
            .text_color(action_style.fg)
            .focusable()
            .focus_visible(|style| style.border_2().border_color(cx.theme().border.focus))
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
            .bg(action_style.bg)
            .hover(move |this| this.bg(action_style.hover_bg))
            .when(self.icon.is_some(), |this| {
                this.child(
                    self.icon
                        .unwrap()
                        .size(icon_size.unwrap_or(cx.theme().tokens.sizes.icon_md))
                        .color(action_style.fg),
                )
            })
    }
}
