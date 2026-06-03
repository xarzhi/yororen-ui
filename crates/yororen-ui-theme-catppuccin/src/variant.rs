//! Catppuccin custom variants.
//!
//! Three variants are registered in addition to the three built-in
//! `ActionVariantKind` values (Neutral / Primary / Danger):
//!
//! - `"mocha"` — same as Primary but the accent is the Mocha `blue`
//!   (used to demonstrate how a flavor's color becomes a custom
//!   variant).
//! - `"lavender"` — uses the Catppuccin `lavender` accent.
//! - `"ghost"` — a transparent background button. On hover it picks
//!   up the surface tint.
//!
//! Use [`register_all`] to install every Catppuccin variant on the
//! global `VariantRegistry`, then reference the variant from a
//! button builder:
//!
//! ```rust,ignore
//! use yororen_ui::component::button;
//! use yororen_ui::renderer::ButtonVariant;
//! use yororen_ui_core::renderer::VariantKey;
//!
//! button("save")
//!     .variant(ButtonVariant::Custom(VariantKey::borrowed("lavender")))
//!     .child("Save");
//! ```

use std::sync::Arc;

use gpui::{App, Hsla};

use yororen_ui_core::renderer::{
    GlobalVariantRegistry, VariantKey, VariantRegistry, VariantState, VariantStyle,
};

use crate::palette;

/// The canonical `VariantKey` for the `mocha` variant.
pub fn key_mocha() -> VariantKey { VariantKey::borrowed("mocha") }

/// The canonical `VariantKey` for the `lavender` variant.
pub fn key_lavender() -> VariantKey { VariantKey::borrowed("lavender") }

/// The canonical `VariantKey` for the `ghost` variant.
pub fn key_ghost() -> VariantKey { VariantKey::borrowed("ghost") }

/// "mocha" variant: Catppuccin Mocha's `blue` accent on the
/// default surface. The `fg` color is `mocha::base()` so the button
/// reads as an "inverted" pill (saturated background, dark text).
#[derive(Debug)]
pub struct MochaVariant;

impl VariantStyle for MochaVariant {
    fn bg(&self, _state: &VariantState) -> Hsla {
        palette::mocha::blue()
    }
    fn fg(&self, _state: &VariantState) -> Hsla {
        palette::mocha::base()
    }
    fn border(&self, _state: &VariantState) -> Option<Hsla> {
        None
    }
    fn disabled_opacity(&self) -> f32 {
        0.55
    }
}

/// "lavender" variant: uses Catppuccin's `lavender` accent.
#[derive(Debug)]
pub struct LavenderVariant;

impl VariantStyle for LavenderVariant {
    fn bg(&self, _state: &VariantState) -> Hsla {
        palette::mocha::lavender()
    }
    fn fg(&self, _state: &VariantState) -> Hsla {
        palette::mocha::base()
    }
    fn border(&self, _state: &VariantState) -> Option<Hsla> {
        None
    }
    fn disabled_opacity(&self) -> f32 {
        0.55
    }
}

/// "ghost" variant: transparent on the base surface, picks up the
/// surface tint on hover (handled by the renderer, not by the variant
/// trait). Disabled state drops opacity to 0.4.
#[derive(Debug)]
pub struct GhostVariant;

impl VariantStyle for GhostVariant {
    fn bg(&self, _state: &VariantState) -> Hsla {
        let mut c = palette::mocha::surface0();
        c.a = 0.0;
        c
    }
    fn fg(&self, _state: &VariantState) -> Hsla {
        palette::mocha::text()
    }
    fn border(&self, _state: &VariantState) -> Option<Hsla> {
        // Use the surface1 color at low alpha so the ghost button
        // has a faint outline against the surface.
        let mut c = palette::mocha::surface1();
        c.a = 0.4;
        Some(c)
    }
    fn disabled_opacity(&self) -> f32 {
        0.4
    }
}

/// Build a `VariantRegistry` pre-populated with the three Catppuccin
/// variants. Apps that want to extend the registry further can chain
/// `.register(...)` calls on the returned value.
pub fn catppuccin_registry() -> VariantRegistry {
    let reg = VariantRegistry::with_defaults();
    reg.register(key_mocha(), Arc::new(MochaVariant));
    reg.register(key_lavender(), Arc::new(LavenderVariant));
    reg.register(key_ghost(), Arc::new(GhostVariant));
    reg
}

/// Install every Catppuccin variant on the given `App` as the
/// global `VariantRegistry`. Existing custom registrations on the
/// global are not preserved (callers that want to merge should
/// pre-populate the registry and pass it to
/// [`cx.set_global(GlobalVariantRegistry(registry))`]
/// themselves).
pub fn register_all(cx: &mut App) {
    let registry = Arc::new(catppuccin_registry());
    cx.set_global(GlobalVariantRegistry(registry));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_have_expected_names() {
        assert_eq!(key_mocha().0.as_ref(), "mocha");
        assert_eq!(key_lavender().0.as_ref(), "lavender");
        assert_eq!(key_ghost().0.as_ref(), "ghost");
    }

    #[test]
    fn mocha_variant_uses_mocha_palette() {
        let v = MochaVariant;
        let bg = v.bg(&VariantState::default());
        let mocha_blue = palette::mocha::blue();
        assert_eq!(bg, mocha_blue);
        let fg = v.fg(&VariantState::default());
        assert_eq!(fg, palette::mocha::base());
    }

    #[test]
    fn lavender_variant_uses_lavender() {
        let v = LavenderVariant;
        let bg = v.bg(&VariantState::default());
        assert_eq!(bg, palette::mocha::lavender());
    }

    #[test]
    fn ghost_variant_is_transparent_with_surface_outline() {
        let v = GhostVariant;
        let bg = v.bg(&VariantState::default());
        assert_eq!(bg.a, 0.0);
        let border = v.border(&VariantState::default()).expect("ghost has a border");
        // Border should be surface1 with low alpha.
        assert!(border.a < 0.5);
    }

    #[test]
    fn catppuccin_registry_has_three() {
        let reg = catppuccin_registry();
        assert!(reg.resolve(&key_mocha()).is_some());
        assert!(reg.resolve(&key_lavender()).is_some());
        assert!(reg.resolve(&key_ghost()).is_some());
        assert_eq!(reg.custom_count(), 3);
    }
}
