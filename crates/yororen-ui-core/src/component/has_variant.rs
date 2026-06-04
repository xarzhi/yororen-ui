//! `HasVariant` — shared helpers for the seven button-like
//! components (button / icon_button / toggle_button /
//! split_button / drag_handle / context_menu_trigger /
//! clickable_surface), plus search_input which renders a
//! built-in icon button with the same variant model.
//!
//! Each of those components carries a `ButtonVariant` field
//! that can be either `Builtin(ActionVariantKind)` or
//! `Custom(VariantKey)`. The custom path resolves through
//! `resolve_custom_variant(...)` and yields an
//! `Option<Arc<dyn VariantStyle>>` that the renderer reads.
//!
//! These helpers centralize the resolve logic so the eight
//! components don't have to repeat the same `match` block in
//! their render methods. They are not a full ControlButton
//! mixin (that would touch the layout), just the
//! variant-resolution half.

use std::sync::Arc;

use crate::renderer::{ButtonVariant, VariantStyle, resolve_custom_variant};
use crate::theme::ActionVariantKind;
use gpui::App;

/// Resolved variant state, ready to hand to a renderer.
///
/// `builtin` is always populated — it falls back to
/// `ActionVariantKind::Neutral` when the variant is a custom
/// key (renderers still need some built-in slot for layout
/// defaults like font weight or icon color).
///
/// `custom_style` is `Some` only when the user picked a custom
/// variant key AND the global `VariantRegistry` resolved it.
/// The 7+ component render paths thread this into their
/// `*RenderState` so the renderer can prefer the custom style
/// over the built-in look.
pub struct ResolvedVariant {
    pub builtin: ActionVariantKind,
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

impl ResolvedVariant {
    /// Resolve a `ButtonVariant` against the running app. Call this
    /// at the top of `render` (after the theme is fetched) so the
    /// resolved `builtin` / `custom_style` can be threaded into
    /// the renderer's `*RenderState`.
    pub fn resolve(variant: &ButtonVariant, cx: &App) -> Self {
        let custom_style = match variant {
            ButtonVariant::Builtin(_) => None,
            ButtonVariant::Custom(key) => resolve_custom_variant(cx, key),
        };
        let builtin = variant.as_builtin().unwrap_or(ActionVariantKind::Neutral);
        Self {
            builtin,
            custom_style,
        }
    }
}
