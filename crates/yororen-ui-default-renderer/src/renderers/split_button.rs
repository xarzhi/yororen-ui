//! `TokenSplitButtonRenderer` — default `SplitButtonRenderer` impl.
//!
//! Composes a complete split-button widget (primary action +
//! chevron toggle + optional floating dropdown) by delegating
//! to the `Button`, `ListItem` and `Panel` renderers. The
//! visual logic (variant tokens, item hover, popover bg/border/
//! shadow) is read from the active theme — the headless
//! `SplitButtonProps` only carries data.
//!
//! Layout: the returned `Div` is `relative`, with the trigger
//! row as its first child and (when `state.is_open()`) an
//! `absolute` dropdown body as the second child. This puts the
//! dropdown on its own stacking layer above the surrounding
//! page flow without shifting sibling components.

use std::sync::Arc;

use gpui::{
    App, BoxShadow, ClickEvent, Div, ElementId, InteractiveElement, ParentElement, Pixels,
    StatefulInteractiveElement, Styled, Window, deferred, div, point, px,
};

use yororen_ui_core::animation::SlideDirection;
use yororen_ui_core::headless::button::ButtonProps;
use yororen_ui_core::headless::dropdown_menu::DropdownItem;
use yororen_ui_core::headless::list_item::ListItemProps;
use yororen_ui_core::headless::split_button::{ClickCallback, SplitButtonProps};
use yororen_ui_core::renderer::variant::ActionVariantKind;
use yororen_ui_core::theme::Theme;

use crate::animation::AnimatedPresenceElement;

pub use yororen_ui_core::renderer::split_button::{SplitButtonRenderState, SplitButtonRenderer};

pub struct TokenSplitButtonRenderer;

// Inherent helpers — *not* part of the trait surface.
impl TokenSplitButtonRenderer {
    pub fn primary_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn primary_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn chevron_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.bg").unwrap_or_default()
    }
    pub fn chevron_fg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme.get_color("action.neutral.fg").unwrap_or_default()
    }
    pub fn chevron_hover_bg(&self, _state: &SplitButtonRenderState, theme: &Theme) -> gpui::Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or_default()
    }
    pub fn min_height(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.button.min_height")
            .unwrap_or(36.0) as f32)
    }
    pub fn border_radius(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(6.0) as f32)
    }
    pub fn gap(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.separator_w")
            .unwrap_or(2.0) as f32)
    }
    pub fn chevron_width(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.chevron_width")
            .unwrap_or(32.0) as f32)
    }
    pub fn menu_width(&self, _state: &SplitButtonRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.split_button.menu_width")
            .unwrap_or(180.0) as f32)
    }
}

impl SplitButtonRenderer for TokenSplitButtonRenderer {
    fn compose(&self, props: &SplitButtonProps, cx: &App) -> Div {
        use yororen_ui_core::theme::ActiveTheme;
        let theme = cx.theme();
        let (open, visible) = props
            .state
            .as_ref()
            .map(|s| {
                let s = s.read(cx);
                (s.is_open(), s.is_visible())
            })
            .unwrap_or((false, false));
        let state = SplitButtonRenderState {
            open,
            disabled: props.disabled,
        };

        // ---- Primary button (caption only, click = props.primary) ----
        let primary_id: ElementId = format!("{:?}-primary", props.id).into();
        let primary = ButtonProps {
            id: primary_id,
            focus_handle: props.primary_focus.clone(),
            on_click: Some(props.primary.clone()),
            disabled: props.disabled,
            clickable: true,
            variant: ActionVariantKind::Neutral,
            caption: props.caption.clone(),
            icon: None,
            icon_size: px(16.),
        }
        .render(cx);

        // ---- Chevron button (toggles dropdown_state) ----
        let state_for_chevron = props.state.clone();
        let chevron_click: ClickCallback =
            Arc::new(move |_ev: &ClickEvent, _w: &mut Window, cx: &mut App| {
                if let Some(s) = state_for_chevron.as_ref() {
                    s.update(cx, |st, _cx| st.toggle());
                }
            });
        let chevron_label = if open { "▴" } else { "▾" };
        let chevron_id: ElementId = format!("{:?}-chevron", props.id).into();
        let chevron_w = self.chevron_width(&state, theme);
        let chevron = ButtonProps {
            id: chevron_id,
            focus_handle: props.chevron_focus.clone(),
            on_click: Some(chevron_click),
            disabled: props.disabled,
            clickable: true,
            variant: ActionVariantKind::Neutral,
            caption: Some(chevron_label.into()),
            icon: None,
            icon_size: px(16.),
        }
        .render(cx)
        .w(chevron_w)
        .px(px(0.));

        let gap = self.gap(&state, theme);
        let trigger_row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(gap)
            .child(primary)
            .child(chevron);

        // `on_mouse_down_out` on the outer wrapper closes the menu
        // when the user clicks anywhere outside the trigger + menu
        // (the absolute menu body is still a layout child of this
        // div, so clicks on items are absorbed by the items and
        // don't fire `_out`). Clicks on the trigger are likewise
        // absorbed by the primary / chevron buttons. Honoured only
        // when the caller opted in via `dismiss_on_outside_click`.
        let state_for_close = props.state.clone();
        let dismiss_outside = props
            .state
            .as_ref()
            .map(|s| s.read(cx).dismiss_on_outside_click)
            .unwrap_or(true);

        // ---- Dropdown body (only when open) ----
        // Wrapped in `gpui::deferred(...)` so the popover paints
        // *after* every other sibling in the tree. Without this,
        // `.absolute()` only removes the element from layout flow
        // — paint order remains DOM order, so any later sibling
        // (e.g. the next row in the section) would draw on top of
        // the menu and you'd see "through" it.
        let mut root = div().relative().child(trigger_row);
        if dismiss_outside {
            root = root.on_mouse_down_out(move |_ev, _w, cx| {
                if let Some(st) = state_for_close.as_ref() {
                    st.update(cx, |s, _cx| s.close());
                }
            });
        }
        if visible {
            // Dropdown bg prefers `surface.popover` (a dedicated
            // contrast colour the JSON theme can override) and
            // falls back to `surface.raised` so older theme
            // packages still render with a sensible elevation.
            let panel_bg = theme
                .get_color("surface.popover")
                .or_else(|| theme.get_color("surface.raised"))
                .unwrap_or_default();
            let panel_border = theme.get_color("border.default").unwrap_or_default();
            let panel_radius = px(theme.get_number("tokens.radii.lg").unwrap_or(8.0) as f32);
            let panel_pad = px(theme.get_number("tokens.spacing.inset_xs").unwrap_or(4.0) as f32);
            let item_hover_bg = theme.get_color("surface.hover").unwrap_or_default();
            let shadow_color = theme.get_color("shadow.elevation_2").unwrap_or_default();
            let divider_color = theme.get_color("border.divider").unwrap_or_default();
            let menu_w = self.menu_width(&state, theme);
            let min_h = self.min_height(&state, theme);
            // Place the menu just below the trigger row.
            let menu_offset = min_h + px(4.);

            let mut menu = div()
                .absolute()
                .top(menu_offset)
                .left_0()
                .w(menu_w)
                .bg(panel_bg)
                .border_1()
                .border_color(panel_border)
                .rounded(panel_radius)
                .p(panel_pad)
                .flex()
                .flex_col()
                .gap(px(2.))
                .shadow(vec![BoxShadow {
                    color: shadow_color,
                    offset: point(px(0.), px(4.)),
                    blur_radius: px(12.),
                    spread_radius: px(0.),
                }])
                // popover pattern: occlude (the
                // `InteractiveElement` trait method) blocks
                // events from reaching elements painted behind
                // the menu (stops a click on an option from
                // also firing on the cell directly below the
                // split button).
                .occlude();

            for it in &props.items {
                match it {
                    DropdownItem::Item(item) => {
                        let item_id_str = item.id.clone();
                        let item_label = item.label.clone();
                        let item_disabled = item.disabled;
                        let state_for_click = props.state.clone();
                        let on_select_for_click = props.on_select.clone();
                        let item_id_for_callback = item_id_str.clone();

                        let row_id: ElementId =
                            format!("{:?}-item-{}", props.id, item_id_str).into();
                        let list_item_el = ListItemProps {
                            id: row_id,
                            title: item_label,
                            description: None,
                            leading_icon: None,
                            trailing_icon: None,
                            selected: false,
                            disabled: item_disabled,
                            on_click: None,
                        }
                        .render(cx);

                        let item_el = if !item_disabled {
                            list_item_el
                                .w_full()
                                // Override list_item's `surface.base`
                                // default so items blend with the
                                // menu container instead of stamping
                                // a contrasting rectangle on it.
                                .bg(panel_bg)
                                .cursor_pointer()
                                .hover(move |s| s.bg(item_hover_bg))
                                .on_click(move |_ev, window, cx| {
                                    if let Some(st) = state_for_click.as_ref() {
                                        st.update(cx, |s, _cx| s.close());
                                    }
                                    if let Some(cb) = on_select_for_click.as_ref() {
                                        cb(item_id_for_callback.clone(), window, cx);
                                    }
                                })
                        } else {
                            list_item_el.w_full().bg(panel_bg)
                        };
                        menu = menu.child(item_el);
                    }
                    DropdownItem::Separator => {
                        menu = menu.child(div().h(px(1.)).bg(divider_color).my(px(2.)));
                    }
                    DropdownItem::Group(_) => {
                        // Groups are not rendered specially in v0.3.
                    }
                }
            }

            let distance = px(theme.get_number("motion.slide_distance").unwrap_or(10.0) as f32);
            let state_entity = props
                .state
                .clone()
                .expect("visible implies state is present");
            // The animation wrapper is absolutely positioned at the
            // top-left of the root relative container so the menu
            // inside keeps its original `top/left` offset.
            root.child(
                deferred(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .child(AnimatedPresenceElement::new(
                            state_entity,
                            (props.id.clone(), "menu"),
                            SlideDirection::Down,
                            distance,
                            div().child(menu),
                        )),
                )
                .with_priority(1),
            )
        } else {
            root
        }
    }
}

pub fn arc_split_button<T: SplitButtonRenderer + 'static>(r: T) -> Arc<dyn SplitButtonRenderer> {
    Arc::new(r)
}
