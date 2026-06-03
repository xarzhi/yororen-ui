//! Root component for the variant-showcase demo.

use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};

use yororen_ui::component::{button, label};
use yororen_ui::i18n::Translate;
use yororen_ui::renderer::{ButtonVariant, GlobalVariantRegistry, VariantKey};
use yororen_ui::theme::ActiveTheme;

pub struct VariantShowcaseApp;

impl VariantShowcaseApp {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self
    }
}

impl Render for VariantShowcaseApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let save_label = cx.t("common.save");
        let cancel_label = cx.t("common.cancel");

        // Resolve the two custom variants from the global registry to
        // prove they were registered correctly.
        let reg = cx.global::<GlobalVariantRegistry>();
        assert!(reg.0.resolve(&VariantKey::borrowed("ghost")).is_some());
        assert!(reg.0.resolve(&VariantKey::borrowed("branded")).is_some());
        let _ = cancel_label;

        div()
            .size_full()
            .bg(theme.surface.base)
            .flex()
            .flex_col()
            .gap(px(20.))
            .p(px(28.))
            .child(label("Variant Showcase").strong(true).text_size(px(24.)))
            .child(label(
                "5 variants in 5 rows. The first 3 use the v0.4 builtin \
                 variants (Neutral / Primary / Danger). The last 2 are \
                 custom — 'ghost' and 'branded' — registered at startup via \
                 VariantRegistry::register() and resolved through the new \
                 ButtonVariant::Custom path.",
            ))
            .child(variant_row(
                "Neutral (builtin)",
                ButtonKind::BuiltinNeutral,
                &save_label,
            ))
            .child(variant_row(
                "Primary (builtin)",
                ButtonKind::BuiltinPrimary,
                &save_label,
            ))
            .child(variant_row(
                "Danger (builtin)",
                ButtonKind::BuiltinDanger,
                &cancel_label,
            ))
            .child(variant_row(
                "Ghost (custom)",
                ButtonKind::CustomGhost,
                &save_label,
            ))
            .child(variant_row(
                "Branded (custom)",
                ButtonKind::CustomBranded,
                &save_label,
            ))
    }
}

#[derive(Clone, Copy)]
enum ButtonKind {
    BuiltinNeutral,
    BuiltinPrimary,
    BuiltinDanger,
    CustomGhost,
    CustomBranded,
}

fn variant_row(title: &str, kind: ButtonKind, label_text: &str) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(12.))
        .child(label(title.to_string()).strong(true).min_w(px(180.0)))
        .child(build_button(kind, label_text))
}

fn build_button(kind: ButtonKind, label_text: &str) -> gpui::AnyElement {
    match kind {
        ButtonKind::BuiltinNeutral => {
            button("variant:builtin:neutral").child(label_text.to_string())
        }
        ButtonKind::BuiltinPrimary => button("variant:builtin:primary")
            .variant(ButtonVariant::Builtin(
                yororen_ui::theme::ActionVariantKind::Primary,
            ))
            .child(label_text.to_string()),
        ButtonKind::BuiltinDanger => button("variant:builtin:danger")
            .variant(ButtonVariant::Builtin(
                yororen_ui::theme::ActionVariantKind::Danger,
            ))
            .child(label_text.to_string()),
        ButtonKind::CustomGhost => button("variant:custom:ghost")
            .variant(ButtonVariant::Custom(VariantKey::borrowed("ghost")))
            .child(label_text.to_string()),
        ButtonKind::CustomBranded => button("variant:custom:branded")
            .variant(ButtonVariant::Custom(VariantKey::borrowed("branded")))
            .child(label_text.to_string()),
    }
    .into_any_element()
}
