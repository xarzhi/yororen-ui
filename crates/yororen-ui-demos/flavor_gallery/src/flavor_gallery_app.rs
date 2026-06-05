//! Root component for the flavor gallery demo.
//!
//! `with_theme` was removed (panic-unsafe). The demo now
//! shows one flavor at a time; a tab strip at the top switches the
//! global theme. The columns-per-flavor layout was the only consumer
//! of per-element theme overrides, so it cannot survive without
//! `with_theme`.

use gpui::prelude::FluentBuilder;
use gpui::{
    Context, Entity, IntoElement, ParentElement, Render, SharedString, Styled, Window, div, px,
};

use yororen_ui::component::{
    ComboBoxOption, OverlayCloseReason, SelectOption, button, combo_box, label, modal_actions_row,
    modal_dialog, select,
};
use yororen_ui::theme::{ActionVariantKind, ActiveTheme, GlobalTheme};

use crate::state::{ActiveModal, FlavorGalleryState, FlavorKind};
use crate::theme_for;

const COLUMN_WIDTH: f32 = 290.0;
const GAP: f32 = 12.0;

pub struct FlavorGalleryApp;

impl FlavorGalleryApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for FlavorGalleryApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let state = cx.global::<FlavorGalleryState>();
        let active_modal = *state.active_modal.read(cx);
        let theme = cx.theme();
        let appearance = cx.window_appearance();
        let active_flavor = *state.active_flavor.read(cx);

        let top_bar = div()
            .flex()
            .gap(px(GAP))
            .items_center()
            .child(label("Active flavor:").strong(true))
            .children(FlavorKind::ALL.iter().map(|kind| {
                let label_text: SharedString = kind.to_string().into();
                let is_active = *kind == active_flavor;
                let state_for_switch = state.active_flavor.clone();
                button(format!("flavor:{}", kind.as_str()))
                    .when(matches!(active_modal, ActiveModal::None), |this| this)
                    .variant(if is_active {
                        ActionVariantKind::Primary
                    } else {
                        ActionVariantKind::Neutral
                    })
                    .child(label_text)
                    .on_click(move |_ev, _w, cx| {
                        let theme = theme_for(*kind, appearance);
                        cx.set_global(GlobalTheme::new(theme));
                        state_for_switch.update(cx, |v, _| *v = *kind);
                        cx.refresh_windows();
                    })
                    .into_any_element()
            }))
            .child(label(
                " (single global theme; the per-flavor 5-column layout was removed with `with_theme`)",
            ));

        let column = render_column(active_flavor, state.active_modal.clone());

        let modal_overlay = if let ActiveModal::Column(kind) = active_modal {
            Some(render_modal(kind, state.active_modal.clone()))
        } else {
            None
        };

        div()
            .size_full()
            .bg(theme.surface.canvas)
            .flex()
            .flex_col()
            .gap(px(GAP))
            .p(px(20.0))
            .child(top_bar)
            .child(div().flex().flex_row().gap(px(GAP)).child(column))
            .when_some(modal_overlay, |this, overlay| this.child(overlay))
    }
}

fn render_column(kind: FlavorKind, state_for_open: Entity<ActiveModal>) -> gpui::AnyElement {
    let column_title: SharedString = format!("{} column", kind).into();
    let inner_button_id: SharedString = format!("flavor:{}:show-modal", kind.as_str()).into();
    let inner_select_id: SharedString = format!("flavor:{}:select", kind.as_str()).into();
    let inner_combo_id: SharedString = format!("flavor:{}:combo", kind.as_str()).into();

    div()
        .w(px(COLUMN_WIDTH))
        .flex()
        .flex_col()
        .gap(px(8.0))
        .p(px(12.0))
        .rounded_lg()
        .border_1()
        .child(label(column_title.clone()).strong(true).text_size(px(16.0)))
        .child(
            label(
                "Select honors Esc via dismiss_on_escape. \
                 Open it, then press Esc to close.",
            )
            .muted(true),
        )
        .child(
            select(inner_select_id.clone())
                .options([
                    SelectOption::new().value("apple").label("Apple"),
                    SelectOption::new().value("banana").label("Banana"),
                    SelectOption::new().value("cherry").label("Cherry"),
                ])
                .placeholder("Pick a fruit…"),
        )
        .child(
            label(
                "Combo box also honors Esc. \
                 Try typing then pressing Esc.",
            )
            .muted(true),
        )
        .child(
            combo_box(inner_combo_id.clone())
                .options([
                    ComboBoxOption::new("cat", "Cat"),
                    ComboBoxOption::new("dog", "Dog"),
                    ComboBoxOption::new("fish", "Fish"),
                ])
                .placeholder("Pick a pet…"),
        )
        .child(label(
            "Modal dialog: full a11y shell (focus trap, Esc, scrim, X).",
        ))
        .child(
            button(inner_button_id.clone())
                .variant(ActionVariantKind::Primary)
                .child("Show modal")
                .on_click(move |_ev, _w, cx| {
                    state_for_open.update(cx, |v, _| {
                        *v = ActiveModal::Column(kind);
                    });
                    cx.refresh_windows();
                }),
        )
        .into_any_element()
}

fn render_modal(kind: FlavorKind, state_for_close: Entity<ActiveModal>) -> gpui::AnyElement {
    modal_dialog(format!("flavor:{}:modal", kind.as_str()))
        .title(format!("{} modal", kind))
        .content(label(format!(
            "Modal rendered for the {} flavor. \
             Press Esc / click the scrim / click the X to close. \
             All three paths route to a single on_close.",
            kind
        )))
        .actions(modal_actions_row(
            yororen_ui::i18n::TextDirection::Ltr,
            [
                button(format!("flavor:{}:modal:cancel", kind.as_str()))
                    .child("Cancel")
                    .into_any_element(),
                button(format!("flavor:{}:modal:ok", kind.as_str()))
                    .variant(ActionVariantKind::Primary)
                    .child("OK")
                    .into_any_element(),
            ],
        ))
        .open(true)
        .on_close(move |reason: &OverlayCloseReason, _w, cx| {
            eprintln!("[{} modal] closed via {:?}", kind, reason);
            state_for_close.update(cx, |v, _| {
                *v = ActiveModal::None;
            });
            cx.refresh_windows();
        })
        .into_any_element()
}
