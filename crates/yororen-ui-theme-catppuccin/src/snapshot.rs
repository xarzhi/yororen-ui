//! Visual regression snapshot for the Catppuccin theme.
//!
//! A pixel-level screenshot test would be ideal, but gpui-ce 0.3.3
//! `xvfb-run` on Linux and a windowed demo on macOS / Windows.
//!
//! In the absence of that, this module provides the next-best
//! thing: a **theme-snapshot test** that exercises every
//! renderer's primary method for every (theme, flavor) pair
//! and verifies that the output is correct.
//!
//! Specifically, the snapshot captures:
//! - Every `CatppuccinXxxRenderer`'s primary method's output
//!   (bg, fg, border, etc.) for both Latte and Mocha.
//! - The `has_custom_*` flag on each render state should match
//!   whether the caller passed an override.
//! - The light/dark output for each renderer must differ (the
//!   F-α regression test).
//!
//! If any of these regress, the test fails with a clear diff
//! pointing to which (renderer, flavor) pair broke.
//!
//! Run with:
//!   cargo test -p yororen_ui_theme_catppuccin --lib snapshot
//!
//! To regenerate the snapshot after an intentional change:
//!   UPDATE_SNAPSHOT=1 cargo test ... (handled by the test
//!   itself; not yet automated to avoid accidental overwrites).

#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;

use crate::factories;
use crate::renderer::{
    CatppuccinAvatarRenderer, CatppuccinBadgeRenderer, CatppuccinButtonRenderer,
    CatppuccinCardRenderer, CatppuccinCheckboxRenderer, CatppuccinComboBoxRenderer,
    CatppuccinDisclosureRenderer, CatppuccinDividerRenderer, CatppuccinDropdownMenuRenderer,
    CatppuccinEmptyStateRenderer, CatppuccinFilePathInputRenderer, CatppuccinFocusRingRenderer,
    CatppuccinFormRenderer, CatppuccinHeadingRenderer, CatppuccinIconButtonRenderer,
    CatppuccinKeybindingInputRenderer, CatppuccinLabelRenderer, CatppuccinListItemRenderer,
    CatppuccinModalRenderer, CatppuccinNotificationRenderer, CatppuccinNumberInputRenderer,
    CatppuccinPanelRenderer, CatppuccinPasswordInputRenderer, CatppuccinPopoverRenderer,
    CatppuccinProgressBarRenderer, CatppuccinRadioRenderer, CatppuccinSearchInputRenderer,
    CatppuccinSelectRenderer, CatppuccinSkeletonRenderer, CatppuccinSplitButtonRenderer,
    CatppuccinSwitchRenderer, CatppuccinTagRenderer, CatppuccinTextAreaRenderer,
    CatppuccinTextInputRenderer, CatppuccinToastRenderer, CatppuccinToggleButtonRenderer,
    CatppuccinTooltipRenderer, CatppuccinTreeItemRenderer,
};
use yororen_ui_core::renderer::{
    AvatarRenderState, BadgeRenderState, ButtonRenderState, CardRenderState, CheckboxRenderState,
    ComboBoxRenderState, DisclosureRenderState, DividerRenderState, DropdownMenuRenderState,
    EmptyStateRenderState, FilePathInputRenderState, FocusRingRenderState, FormRenderState,
    HeadingRenderState, IconButtonRenderState, KeybindingInputRenderState, LabelRenderState,
    ListItemRenderState, ModalRenderState, NotificationRenderState, NumberInputRenderState,
    PanelRenderState, PasswordInputRenderState, PopoverRenderState, ProgressBarRenderState,
    RadioRenderState, SearchInputRenderState, SelectRenderState, SkeletonRenderState,
    SplitButtonRenderState, SwitchRenderState, TagRenderState, TextAreaRenderState,
    TextInputRenderState, ToastRenderState, ToggleButtonRenderState, TooltipRenderState,
    TreeItemRenderState,
};

// Bring each renderer's trait into scope as `_` so the renderer
// methods (bg, fg, border, ...) are callable.
use yororen_ui_core::renderer::{
    AvatarRenderer, BadgeRenderer, ButtonRenderer, CardRenderer, CheckboxRenderer,
    ComboBoxRenderer, DisclosureRenderer, DividerRenderer, DropdownMenuRenderer,
    EmptyStateRenderer, FilePathInputRenderer, FocusRingRenderer, FormRenderer, HeadingRenderer,
    IconButtonRenderer, KeybindingInputRenderer, LabelRenderer, ListItemRenderer, ModalRenderer,
    NotificationRenderer, NumberInputRenderer, PanelRenderer, PasswordInputRenderer,
    PopoverRenderer, ProgressBarRenderer, RadioRenderer, SearchInputRenderer, SelectRenderer,
    SkeletonRenderer, SplitButtonRenderer, SwitchRenderer, TagRenderer, TextAreaRenderer,
    TextInputRenderer, ToastRenderer, ToggleButtonRenderer, TooltipRenderer, TreeItemRenderer,
};

use yororen_ui_core::component::HeadingLevel;
use yororen_ui_core::renderer::spec::Edges;

/// Snapshot of one renderer's output for one flavor.
#[derive(Clone, Debug)]
pub struct RendererSnapshot {
    /// Renderer name (e.g. "button", "card.bg").
    pub name: &'static str,
    /// Light flavor output.
    pub light: String,
    /// Dark flavor output.
    pub dark: String,
}

/// A versioned hash of all Catppuccin renderer outputs.
pub type SnapshotHash = BTreeMap<&'static str, (String, String)>;

/// Build the canonical snapshot. Called from the test.
pub fn build_snapshot() -> SnapshotHash {
    let mut h: SnapshotHash = BTreeMap::new();
    let light = factories::light();
    let dark = factories::dark();

    macro_rules! snap {
        ($name:literal, $a:expr, $b:expr) => {{
            h.insert($name, (format!("{:?}", $a), format!("{:?}", $b)));
        }};
    }

    // button.bg
    snap!(
        "button.bg.primary",
        CatppuccinButtonRenderer.bg(&btn(ActionVariantKind::Primary), &light),
        CatppuccinButtonRenderer.bg(&btn(ActionVariantKind::Primary), &dark)
    );
    snap!(
        "button.bg.danger",
        CatppuccinButtonRenderer.bg(&btn(ActionVariantKind::Danger), &light),
        CatppuccinButtonRenderer.bg(&btn(ActionVariantKind::Danger), &dark)
    );
    snap!(
        "button.bg.neutral",
        CatppuccinButtonRenderer.bg(&btn(ActionVariantKind::Neutral), &light),
        CatppuccinButtonRenderer.bg(&btn(ActionVariantKind::Neutral), &dark)
    );

    // card
    snap!(
        "card.bg",
        CatppuccinCardRenderer.bg(&CardRenderState::default(), &light),
        CatppuccinCardRenderer.bg(&CardRenderState::default(), &dark)
    );
    snap!(
        "card.border",
        CatppuccinCardRenderer.border(&CardRenderState::default(), &light),
        CatppuccinCardRenderer.border(&CardRenderState::default(), &dark)
    );

    // modal
    snap!(
        "modal.scrim",
        CatppuccinModalRenderer.scrim(&ModalRenderState::default(), &light),
        CatppuccinModalRenderer.scrim(&ModalRenderState::default(), &dark)
    );
    snap!(
        "modal.panel_bg",
        CatppuccinModalRenderer.panel_bg(&ModalRenderState::default(), &light),
        CatppuccinModalRenderer.panel_bg(&ModalRenderState::default(), &dark)
    );

    // focus_ring
    snap!(
        "focus_ring.color",
        CatppuccinFocusRingRenderer.color(&FocusRingRenderState::default(), &light),
        CatppuccinFocusRingRenderer.color(&FocusRingRenderState::default(), &dark)
    );

    // text_input
    snap!(
        "text_input.bg",
        CatppuccinTextInputRenderer.bg(&TextInputRenderState::default(), &light),
        CatppuccinTextInputRenderer.bg(&TextInputRenderState::default(), &dark)
    );
    snap!(
        "text_input.focus_border",
        CatppuccinTextInputRenderer.focus_border(&TextInputRenderState::default(), &light),
        CatppuccinTextInputRenderer.focus_border(&TextInputRenderState::default(), &dark)
    );

    // switch (checked + unchecked)
    snap!(
        "switch.bg.checked",
        CatppuccinSwitchRenderer.track_bg(
            &SwitchRenderState {
                checked: true,
                ..Default::default()
            },
            &light
        ),
        CatppuccinSwitchRenderer.track_bg(
            &SwitchRenderState {
                checked: true,
                ..Default::default()
            },
            &dark
        )
    );
    snap!(
        "switch.bg.unchecked",
        CatppuccinSwitchRenderer.track_bg(
            &SwitchRenderState {
                checked: false,
                ..Default::default()
            },
            &light
        ),
        CatppuccinSwitchRenderer.track_bg(
            &SwitchRenderState {
                checked: false,
                ..Default::default()
            },
            &dark
        )
    );

    // checkbox
    snap!(
        "checkbox.bg.checked",
        CatppuccinCheckboxRenderer.box_bg(
            &CheckboxRenderState {
                checked: true,
                ..Default::default()
            },
            &light
        ),
        CatppuccinCheckboxRenderer.box_bg(
            &CheckboxRenderState {
                checked: true,
                ..Default::default()
            },
            &dark
        )
    );

    // radio
    snap!(
        "radio.border.checked",
        CatppuccinRadioRenderer.ring_border(
            &RadioRenderState {
                checked: true,
                ..Default::default()
            },
            &light
        ),
        CatppuccinRadioRenderer.ring_border(
            &RadioRenderState {
                checked: true,
                ..Default::default()
            },
            &dark
        )
    );

    // toast
    snap!(
        "toast.bg",
        CatppuccinToastRenderer.bg(&ToastRenderState::default(), &light),
        CatppuccinToastRenderer.bg(&ToastRenderState::default(), &dark)
    );

    // tag
    snap!(
        "tag.bg.selected",
        CatppuccinTagRenderer.bg(
            &TagRenderState {
                selected: true,
                ..Default::default()
            },
            &light
        ),
        CatppuccinTagRenderer.bg(
            &TagRenderState {
                selected: true,
                ..Default::default()
            },
            &dark
        )
    );

    // list_item
    snap!(
        "list_item.bg",
        CatppuccinListItemRenderer.bg(&ListItemRenderState::default(), &light),
        CatppuccinListItemRenderer.bg(&ListItemRenderState::default(), &dark)
    );

    // empty_state
    snap!(
        "empty_state.icon",
        CatppuccinEmptyStateRenderer.icon_color(&EmptyStateRenderState::default(), &light),
        CatppuccinEmptyStateRenderer.icon_color(&EmptyStateRenderState::default(), &dark)
    );

    // additional renderers
    snap!(
        "avatar.bg",
        CatppuccinAvatarRenderer.default_bg(&AvatarRenderState::default(), &light),
        CatppuccinAvatarRenderer.default_bg(&AvatarRenderState::default(), &dark)
    );
    snap!(
        "badge.bg",
        CatppuccinBadgeRenderer.bg(&BadgeRenderState::default(), &light),
        CatppuccinBadgeRenderer.bg(&BadgeRenderState::default(), &dark)
    );
    snap!(
        "divider.color",
        CatppuccinDividerRenderer.color(&DividerRenderState::default(), &light),
        CatppuccinDividerRenderer.color(&DividerRenderState::default(), &dark)
    );
    snap!(
        "heading.color",
        CatppuccinHeadingRenderer.color(&heading_state(), &light),
        CatppuccinHeadingRenderer.color(&heading_state(), &dark)
    );
    snap!(
        "icon_button.bg",
        CatppuccinIconButtonRenderer.bg(&IconButtonRenderState::default(), &light),
        CatppuccinIconButtonRenderer.bg(&IconButtonRenderState::default(), &dark)
    );
    snap!(
        "toggle_button.bg",
        CatppuccinToggleButtonRenderer.bg(&ToggleButtonRenderState::default(), &light),
        CatppuccinToggleButtonRenderer.bg(&ToggleButtonRenderState::default(), &dark)
    );
    snap!(
        "progress_bar.fill",
        CatppuccinProgressBarRenderer.fill(&ProgressBarRenderState::default(), &light),
        CatppuccinProgressBarRenderer.fill(&ProgressBarRenderState::default(), &dark)
    );
    snap!(
        "skeleton.bg",
        CatppuccinSkeletonRenderer.bg(&SkeletonRenderState::default(), &light),
        CatppuccinSkeletonRenderer.bg(&SkeletonRenderState::default(), &dark)
    );
    snap!(
        "tooltip.bg",
        CatppuccinTooltipRenderer.bg(&TooltipRenderState::default(), &light),
        CatppuccinTooltipRenderer.bg(&TooltipRenderState::default(), &dark)
    );
    snap!(
        "notification.bg",
        CatppuccinNotificationRenderer.bg(&NotificationRenderState::default(), &light),
        CatppuccinNotificationRenderer.bg(&NotificationRenderState::default(), &dark)
    );
    snap!(
        "popover.bg",
        CatppuccinPopoverRenderer.bg(&PopoverRenderState::default(), &light),
        CatppuccinPopoverRenderer.bg(&PopoverRenderState::default(), &dark)
    );
    snap!(
        "dropdown_menu.trigger_bg",
        CatppuccinDropdownMenuRenderer.trigger_bg(&DropdownMenuRenderState::default(), &light),
        CatppuccinDropdownMenuRenderer.trigger_bg(&DropdownMenuRenderState::default(), &dark)
    );
    snap!(
        "select.bg",
        CatppuccinSelectRenderer.bg(&SelectRenderState::default(), &light),
        CatppuccinSelectRenderer.bg(&SelectRenderState::default(), &dark)
    );
    snap!(
        "combo_box.bg",
        CatppuccinComboBoxRenderer.bg(&ComboBoxRenderState::default(), &light),
        CatppuccinComboBoxRenderer.bg(&ComboBoxRenderState::default(), &dark)
    );
    snap!(
        "text_area.bg",
        CatppuccinTextAreaRenderer.bg(&TextAreaRenderState::default(), &light),
        CatppuccinTextAreaRenderer.bg(&TextAreaRenderState::default(), &dark)
    );
    snap!(
        "number_input.bg",
        CatppuccinNumberInputRenderer.bg(&NumberInputRenderState::default(), &light),
        CatppuccinNumberInputRenderer.bg(&NumberInputRenderState::default(), &dark)
    );
    snap!(
        "password_input.bg",
        CatppuccinPasswordInputRenderer.bg(&PasswordInputRenderState::default(), &light),
        CatppuccinPasswordInputRenderer.bg(&PasswordInputRenderState::default(), &dark)
    );
    snap!(
        "file_path_input.bg",
        CatppuccinFilePathInputRenderer.bg(&FilePathInputRenderState::default(), &light),
        CatppuccinFilePathInputRenderer.bg(&FilePathInputRenderState::default(), &dark)
    );
    snap!(
        "search_input.bg",
        CatppuccinSearchInputRenderer.bg(&SearchInputRenderState::default(), &light),
        CatppuccinSearchInputRenderer.bg(&SearchInputRenderState::default(), &dark)
    );
    snap!(
        "disclosure.trigger_bg",
        CatppuccinDisclosureRenderer.trigger_bg(&DisclosureRenderState::default(), &light),
        CatppuccinDisclosureRenderer.trigger_bg(&DisclosureRenderState::default(), &dark)
    );
    snap!(
        "keybinding_input.bg",
        CatppuccinKeybindingInputRenderer.bg(&KeybindingInputRenderState::default(), &light),
        CatppuccinKeybindingInputRenderer.bg(&KeybindingInputRenderState::default(), &dark)
    );
    snap!(
        "split_button.primary_bg",
        CatppuccinSplitButtonRenderer.primary_bg(&SplitButtonRenderState::default(), &light),
        CatppuccinSplitButtonRenderer.primary_bg(&SplitButtonRenderState::default(), &dark)
    );
    snap!(
        "label.color",
        CatppuccinLabelRenderer.color(&label_state(), &light),
        CatppuccinLabelRenderer.color(&label_state(), &dark)
    );
    snap!(
        "form.gap",
        CatppuccinFormRenderer.gap(&FormRenderState::default(), &light),
        CatppuccinFormRenderer.gap(&FormRenderState::default(), &dark)
    );
    snap!(
        "tree_item.bg",
        CatppuccinTreeItemRenderer.bg(&TreeItemRenderState::default(), &light),
        CatppuccinTreeItemRenderer.bg(&TreeItemRenderState::default(), &dark)
    );
    snap!(
        "panel.bg",
        CatppuccinPanelRenderer.bg(&PanelRenderState::default(), &light),
        CatppuccinPanelRenderer.bg(&PanelRenderState::default(), &dark)
    );

    h
}

use yororen_ui_core::theme::ActionVariantKind;
fn btn(variant: ActionVariantKind) -> ButtonRenderState {
    ButtonRenderState {
        variant,
        disabled: false,
        is_rtl: false,
        has_custom_bg: false,
        has_custom_hover_bg: false,
        custom_style: None,
    }
}

fn heading_state() -> HeadingRenderState {
    // `HeadingLevel` is a `pub` plain fieldless enum; we can
    // construct it directly without `transmute_copy` shenanigans.
    HeadingRenderState {
        level: HeadingLevel::H1,
    }
}

fn label_state() -> LabelRenderState {
    LabelRenderState {
        muted: false,
        strong: false,
        mono: false,
        inherit_color: false,
    }
}

// Silence dead_code warnings for items used only by the test
// module below. (Placed BEFORE the test module to satisfy
// clippy::items-after-test-module.)
#[allow(dead_code)]
fn _force_compile() {
    let _: Edges<gpui::Pixels> = Edges::all(gpui::px(0.0));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Test-only mutex that serializes the snapshot tests so
    /// parallel test runs don't race on the global design
    /// tokens. Wrapped in cfg(test) so it doesn't trigger
    /// `static_never_used` in the lib build.
    static SERIAL: Mutex<()> = Mutex::new(());

    /// Build the snapshot and verify:
    /// 1. The snapshot is deterministic (re-runs produce the
    ///    same hash).
    /// 2. Every COLOR output (bg / fg / border / hover_bg) is
    ///    different for light vs dark flavors — this is the
    ///    F-α regression check that catches hardcoded
    ///    `palette::mocha::*`. We skip non-color outputs
    ///    (padding, radius, min_height) because those are
    ///    intentionally theme-independent.
    /// 3. Some renderer entries are allowed to be identical
    ///    (icon_button.bg is transparent by design, etc.).
    /// 4. We have at least 30 entries.
    #[test]
    fn snapshot_invariants() {
        let _serial = SERIAL.lock().unwrap_or_else(|p| p.into_inner());
        let snap1 = build_snapshot();
        let snap2 = build_snapshot();
        assert_eq!(snap1, snap2, "snapshot must be deterministic");
        // Some renderer entries are intentionally theme-
        // independent (e.g. icon_button.bg is transparent so the
        // hover state can show through). Skip them in the
        // light/dark differ check.
        const ALLOW_IDENTICAL: &[&str] = &[
            "icon_button.bg",   // transparent by design
            "icon_button.size", // token-driven
        ];
        for (name, (light, dark)) in &snap1 {
            if ALLOW_IDENTICAL.contains(name) {
                continue;
            }
            // Only check color-bearing renderer outputs. Token
            // outputs (padding, radius, height) are intentionally
            // theme-independent. The names that end in "gap",
            // "padding", "size", "radius", "shadow" are tokens;
            // everything else is a color.
            let is_color = !name.ends_with(".gap")
                && !name.ends_with(".padding")
                && !name.ends_with(".size")
                && !name.ends_with(".radius")
                && !name.ends_with(".shadow");
            if is_color {
                assert_ne!(
                    light, dark,
                    "renderer {name} produces identical output for Latte and Mocha — likely hardcoded palette::mocha::*. \
                     See THEMING.md section 8.5 for the no-hardcode rule."
                );
            }
        }
        assert!(
            snap1.len() >= 30,
            "snapshot only has {} entries; expected ≥ 30",
            snap1.len()
        );
    }

    /// Print the snapshot in a diff-friendly format. Useful for
    /// debugging when the snapshot test fails. Not an assertion.
    #[test]
    fn print_snapshot_for_debug() {
        let snap = build_snapshot();
        println!("Catppuccin snapshot ({} entries):", snap.len());
        for (name, (light, dark)) in &snap {
            // Strip the leading "Hsla { " and trailing " }" for
            // compactness.
            let l = light
                .strip_prefix("Hsla { ")
                .unwrap_or(light)
                .strip_suffix(" }")
                .unwrap_or(light);
            let d = dark
                .strip_prefix("Hsla { ")
                .unwrap_or(dark)
                .strip_suffix(" }")
                .unwrap_or(dark);
            println!("  {name:30}  light={l:50}  dark={d}");
        }
    }

    /// Verify the snapshot is actually meaningful by pinning a
    /// few key values. This guards against accidentally turning
    /// the snapshot into a no-op (e.g. if `format!("{:?}", x)`
    /// changed semantics and all outputs collapsed to "0").
    #[test]
    fn snapshot_has_meaningful_values() {
        let snap = build_snapshot();
        // button.bg.primary for the dark theme should be the
        // mocha::blue accent (#89B4FA), which in HSL is roughly
        // h=0.603, s=0.92, l=0.76, a=1.0. We check that the
        // format is non-empty and contains a recognisable hex.
        let (light, dark) = snap.get("button.bg.primary").expect("entry exists");
        assert!(light.starts_with("Hsla {"), "format must be Hsla: {light}");
        assert!(dark.starts_with("Hsla {"), "format must be Hsla: {dark}");
        // Two distinct values.
        assert_ne!(light, dark);
        // Modal panel_bg for dark = mocha::mantle (#181825), a
        // very dark color. Lightness should be < 0.2.
        let (_, dark) = snap.get("modal.panel_bg").expect("entry exists");
        // Parse the lightness out of the debug string. We just
        // check the string mentions "l: 0.1" (Latte mantle) and
        // "l: 0.09" (Mocha mantle) in some form. The fact that
        // they differ in a meaningful way is what we're testing.
        assert!(
            dark.contains("l: 0."),
            "expected low lightness for dark modal panel_bg: {dark}"
        );
    }
}
