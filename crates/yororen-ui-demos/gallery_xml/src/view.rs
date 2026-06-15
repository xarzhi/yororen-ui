//! The `GalleryApp` view. The per-section UI lives in XML
//! files under `src/ui/sections/`; this file owns the
//! scroll-root, the top-level section virtual list, the
//! modal overlay, and the notification host.

use gpui::{AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Render, Styled, Window, div, hsla, px};
use yororen_ui::theme::ActiveTheme;

use yororen_ui::headless::divider::divider;
use yororen_ui::headless::modal::modal;
use yororen_ui::headless::virtual_list::virtual_list;
use yororen_ui::notification::center::NotificationCenter;
use yororen_ui::xml_file;

use crate::controller::Controller;
use crate::state::StateRef;
use crate::theme_switcher::install_renderer;

/// Number of rows in the top-level section virtual list.
pub const SECTION_ROW_COUNT: usize = 9;

pub struct GalleryApp {
    controller: Controller,
}

impl GalleryApp {
    pub fn new(cx: &mut Context<Self>, controller: Controller) -> Self {
        // Re-render whenever the underlying state entity changes.
        let state = cx.global::<StateRef>().state.clone();
        cx.observe(&state, |_this, _state, cx| cx.notify()).detach();
        Self { controller }
    }
}

impl Render for GalleryApp {
    #[allow(clippy::explicit_auto_deref)]
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // 1. Per-render renderer + theme install. Cheap and
        //    guarantees the window reflects the latest toolbar
        //    toggle.
        let renderer = self.controller.current_renderer(cx);
        let dark = self.controller.dark_mode(cx);
        install_renderer(&mut **cx, renderer, dark);

        // 2. Register the host window with the notification center
        //    so toasts can auto-dismiss.
        if let Some(center) = cx.try_global::<NotificationCenter>() {
            center.register_host_window(window.window_handle());
        }

        // 3. Surface color for the window background.
        let surface = cx.theme().get_color("surface.base").unwrap_or_default();

        // 4. Modal scrim/panel at scroll-root level so it survives
        //    virtualization.
        let modal_overlay = build_modal_overlay(self, window, cx);

        // 5. Section virtual list. The row closure dispatches to the
        //    matching XML section file.
        let entity = cx.entity().clone();
        let section_vl = virtual_list(
            "gallery-sections-vl",
            &self.controller.section_list_controller(cx),
            cx,
        )
        .row(move |ix, window, cx| section_row(ix, &entity, window, cx))
        .render(cx)
        .size_full()
        .border_color(surface)
        .rounded(px(0.));

        div()
            .id("gallery-scroll")
            .relative()
            .size_full()
            .bg(surface)
            .child(section_vl)
            .child(modal_overlay)
            .child(crate::notifications_host::deferred_host(cx))
    }
}

fn section_row(
    ix: usize,
    entity: &gpui::Entity<GalleryApp>,
    window: &mut Window,
    cx: &mut gpui::App,
) -> AnyElement {
    let entity = entity.clone();
    entity.update(cx, |app, cx| {
        let controller = app.controller.clone();
        let inner: AnyElement = match ix {
            0 => {
                let tb = xml_file!(cx = cx, window = window, "ui/sections/toolbar.xml");
                let div_pair = div()
                    .flex()
                    .flex_col()
                    .child(tb)
                    .child(divider("toolbar-divider", cx).render(cx));
                div_pair.into_any_element()
            }
            1 => xml_file!(cx = cx, window = window, "ui/sections/actions.xml").into_any_element(),
            2 => xml_file!(cx = cx, window = window, "ui/sections/display.xml").into_any_element(),
            3 => xml_file!(cx = cx, window = window, "ui/sections/surfaces.xml").into_any_element(),
            4 => xml_file!(cx = cx, window = window, "ui/sections/inputs.xml").into_any_element(),
            5 => xml_file!(cx = cx, window = window, "ui/sections/controls.xml").into_any_element(),
            6 => xml_file!(cx = cx, window = window, "ui/sections/overlays.xml").into_any_element(),
            7 => xml_file!(cx = cx, window = window, "ui/sections/lists.xml").into_any_element(),
            8 => xml_file!(cx = cx, window = window, "ui/sections/footer.xml").into_any_element(),
            _ => div().into_any_element(),
        };

        let mut wrapper = div().px(px(24.)).pb(px(24.));
        if ix == 0 {
            wrapper = wrapper.pt(px(24.));
        }
        wrapper.child(inner).into_any_element()
    })
}

fn build_modal_overlay(
    app: &GalleryApp,
    window: &mut Window,
    cx: &mut Context<GalleryApp>,
) -> gpui::Deferred {
    if !app.controller.is_modal_open(cx) {
        return gpui::deferred(div()).with_priority(2);
    }

    let controller = app.controller.clone();
    let modal_panel = modal("ov-modal", app.controller.modal_state(cx))
        .child(xml_file!(
            cx = cx,
            window = window,
            "ui/sections/modal_body.xml"
        ))
        .render(cx)
        .w(px(360.));

    gpui::deferred(
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .bg(hsla(0.0, 0.0, 0.0, 0.55))
            .child(modal_panel),
    )
    .with_priority(2)
}
