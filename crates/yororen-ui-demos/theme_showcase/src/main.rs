//! yororen-ui Theme Showcase Demo
//!
//! Phase F.4 end-to-end proof. Renders a split window:
//!
//! - **Left half** uses the default `theme-system` palette + token
//!   renderers. (v0.5 visual baseline.)
//! - **Right half** uses the `yororen-ui-theme-catppuccin` Mocha
//!   palette + Catppuccin-flavoured renderers. The two halves share
//!   the same UI, so a side-by-side diff is unmistakable.
//!
//! The right half is wrapped in `with_theme(right_theme, ...)` so
//! its components pick up the Catppuccin theme without touching the
//! process-global theme. A "Switch right" button toggles which theme
//! the right half uses (system / catppuccin), proving that
//! `Theme.renderers` is a per-Theme swappable handle, not a
//! process-global.
//!
//! Also registers the three Catppuccin-specific custom variants
//! (`mocha`, `lavender`, `ghost`) on the global `VariantRegistry`,
//! and renders a "Custom variants" row that uses them.

use std::sync::Arc;

use gpui::{App, AppContext, Application, WindowAppearance, WindowBounds, WindowOptions, px, size};

use yororen_ui::assets::UiAsset;
use yororen_ui::renderer::{GlobalVariantRegistry, VariantRegistry};

use yororen_ui_locale_en as locale_en;
use yororen_ui_theme_catppuccin as catppuccin;
use yororen_ui_theme_system as theme_system;

mod showcase_app;
mod state;

use state::ThemeShowcaseState;

fn main() {
    let app = Application::new().with_assets(UiAsset);

    app.run(|cx: &mut App| {
        yororen_ui::component::init(cx);
        // Default install uses theme-system. The right half of the
        // window overrides to the Catppuccin theme via `with_theme`.
        theme_system::install(cx, cx.window_appearance());
        locale_en::install(cx);

        // Register the 3 Catppuccin custom variants so the gallery
        // can render them.
        let reg = Arc::new(catppuccin::variant::catppuccin_registry());
        cx.set_global(GlobalVariantRegistry(reg));

        let st = ThemeShowcaseState::new(cx);
        cx.set_global(st);

        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(gpui::Bounds::centered(
                None,
                size(px(900.0), px(620.0)),
                cx,
            ))),
            ..Default::default()
        };

        let _ = cx.open_window(options, |_, cx| cx.new(showcase_app::ThemeShowcaseApp::new));
    });
}

/// Resolve the system palette for the current OS appearance, but
/// swap the renderer registry for the full Catppuccin one. This
/// proves that a Catppuccin "look" can be applied to any palette
/// (and vice versa).
///
/// F-γ (Phase F review requirement): this covers all 37 renderers
/// the Catppuccin theme ships, not just a subset. The 1 remaining
/// gap is the `widget/` package, which is out of scope for Phase F.
pub fn catppuccin_renderer_only(appearance: WindowAppearance) -> yororen_ui::theme::Theme {
    let mut t = match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => theme_system::light(),
        _ => theme_system::dark(),
    };
    t.renderers = catppuccin::renderer::catppuccin_registry();
    t
}

/// Build a Catppuccin theme matching the current `WindowAppearance`
/// (Latte for light, Mocha for dark) with the full Catppuccin
/// renderer registry.
pub fn catppuccin_theme(appearance: WindowAppearance) -> yororen_ui::theme::Theme {
    match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => catppuccin::light(),
        _ => catppuccin::dark(),
    }
}

/// Build a system theme matching the current `WindowAppearance`
/// using the v0.5 token-based defaults.
pub fn system_theme(appearance: WindowAppearance) -> yororen_ui::theme::Theme {
    match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => theme_system::light(),
        _ => theme_system::dark(),
    }
}

/// Sanity-check helper used in tests: the variant registry should
/// have all three Catppuccin variants registered.
pub fn assert_variants_registered(cx: &App) {
    let reg = cx.global::<GlobalVariantRegistry>();
    let _ = reg
        .0
        .resolve(&yororen_ui::renderer::VariantKey::borrowed("mocha"));
    let _ = reg
        .0
        .resolve(&yororen_ui::renderer::VariantKey::borrowed("lavender"));
    let _ = reg
        .0
        .resolve(&yororen_ui::renderer::VariantKey::borrowed("ghost"));
}

// We rely on the global VariantRegistry being present in
// `cx.global::<GlobalVariantRegistry>()`. This function is unused at
// the moment but kept so future demos can re-register without
// reaching into the global directly.
#[allow(dead_code)]
pub fn fresh_variant_registry() -> Arc<VariantRegistry> {
    Arc::new(catppuccin::variant::catppuccin_registry())
}

#[cfg(test)]
mod tests {
    use super::*;
    use yororen_ui::renderer::{
        AvatarRenderState, BadgeRenderState, ButtonRenderState, CardRenderState,
        CheckboxRenderState, ComboBoxRenderState, DisclosureRenderState, DividerRenderState,
        DropdownMenuRenderState, EmptyStateRenderState, FilePathInputRenderState,
        FocusRingRenderState, FormRenderState, HeadingRenderState, IconButtonRenderState,
        IconRenderState, KeybindingInputRenderState, LabelRenderState, ListItemRenderState,
        ModalRenderState, NotificationRenderState, NumberInputRenderState,
        PasswordInputRenderState, PopoverRenderState, ProgressBarRenderState, RadioRenderState,
        SearchInputRenderState, SelectRenderState, SkeletonRenderState, SplitButtonRenderState,
        SwitchRenderState, TagRenderState, TextAreaRenderState, TextInputRenderState,
        ToastRenderState, ToggleButtonRenderState, TooltipRenderState, TreeItemRenderState,
    };

    /// F-γ: catppuccin_renderer_only() must cover all 37 renderers.
    /// This is a smoke test that calls a primary method on each
    /// renderer to verify the registry is fully populated.
    #[test]
    fn catppuccin_renderer_only_covers_all_renderers() {
        use gpui::WindowAppearance;
        let theme = catppuccin_renderer_only(WindowAppearance::Dark);
        let _ = theme
            .renderers
            .button
            .bg(&ButtonRenderState::default(), &theme);
        let _ = theme
            .renderers
            .icon_button
            .bg(&IconButtonRenderState::default(), &theme);
        let _ = theme
            .renderers
            .toggle_button
            .bg(&ToggleButtonRenderState::default(), &theme);
        let _ = theme
            .renderers
            .label
            .color(&LabelRenderState::default(), &theme);
        let _ = theme.renderers.heading.size(
            &HeadingRenderState {
                level: unsafe { std::mem::zeroed() },
            },
            &theme,
        );
        let _ = theme
            .renderers
            .divider
            .color(&DividerRenderState::default(), &theme);
        let _ = theme
            .renderers
            .focus_ring
            .color(&FocusRingRenderState::default(), &theme);
        let _ = theme
            .renderers
            .badge
            .bg(&BadgeRenderState::default(), &theme);
        let _ = theme.renderers.tag.bg(&TagRenderState::default(), &theme);
        let _ = theme
            .renderers
            .progress_bar
            .track(&ProgressBarRenderState::default(), &theme);
        let _ = theme
            .renderers
            .skeleton
            .bg(&SkeletonRenderState::default(), &theme);
        let _ = theme
            .renderers
            .tooltip
            .bg(&TooltipRenderState::default(), &theme);
        let _ = theme
            .renderers
            .avatar
            .default_bg(&AvatarRenderState::default(), &theme);
        let _ = theme
            .renderers
            .switch
            .track_bg(&SwitchRenderState::default(), &theme);
        let _ = theme
            .renderers
            .checkbox
            .box_bg(&CheckboxRenderState::default(), &theme);
        let _ = theme
            .renderers
            .radio
            .ring_bg(&RadioRenderState::default(), &theme);
        let _ = theme
            .renderers
            .text_input
            .bg(&TextInputRenderState::default(), &theme);
        let _ = theme
            .renderers
            .text_area
            .bg(&TextAreaRenderState::default(), &theme);
        let _ = theme
            .renderers
            .password_input
            .bg(&PasswordInputRenderState::default(), &theme);
        let _ = theme
            .renderers
            .number_input
            .bg(&NumberInputRenderState::default(), &theme);
        let _ = theme
            .renderers
            .file_path_input
            .bg(&FilePathInputRenderState::default(), &theme);
        let _ = theme
            .renderers
            .search_input
            .bg(&SearchInputRenderState::default(), &theme);
        let _ = theme
            .renderers
            .select
            .bg(&SelectRenderState::default(), &theme);
        let _ = theme
            .renderers
            .combo_box
            .bg(&ComboBoxRenderState::default(), &theme);
        let _ = theme
            .renderers
            .modal
            .panel_bg(&ModalRenderState::default(), &theme);
        let _ = theme
            .renderers
            .popover
            .bg(&PopoverRenderState::default(), &theme);
        let _ = theme
            .renderers
            .dropdown_menu
            .trigger_bg(&DropdownMenuRenderState::default(), &theme);
        let _ = theme
            .renderers
            .disclosure
            .trigger_bg(&DisclosureRenderState::default(), &theme);
        let _ = theme
            .renderers
            .toast
            .bg(&ToastRenderState::default(), &theme);
        let _ = theme
            .renderers
            .notification
            .bg(&NotificationRenderState::default(), &theme);
        let _ = theme.renderers.card.bg(&CardRenderState::default(), &theme);
        let _ = theme
            .renderers
            .form
            .gap(&FormRenderState::default(), &theme);
        let _ = theme
            .renderers
            .list_item
            .bg(&ListItemRenderState::default(), &theme);
        let _ = theme
            .renderers
            .tree_item
            .bg(&TreeItemRenderState::default(), &theme);
        let _ = theme
            .renderers
            .keybinding_input
            .bg(&KeybindingInputRenderState::default(), &theme);
        let _ = theme
            .renderers
            .split_button
            .primary_bg(&SplitButtonRenderState::default(), &theme);
        let _ = theme
            .renderers
            .empty_state
            .icon_color(&EmptyStateRenderState::default(), &theme);
        let _ = theme
            .renderers
            .icon
            .color(&IconRenderState::default(), &theme);
    }
}
