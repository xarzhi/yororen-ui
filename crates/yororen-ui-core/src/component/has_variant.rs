//! `HasVariant` — shared helpers for the seven button-like
//! components (button / icon_button / toggle_button /
//! split_button / drag_handle / context_menu_trigger /
//! clickable_surface).
//!
//! Each of those components carries a `ButtonVariant` field
//! that can be either `Builtin(ActionVariantKind)` or
//! `Custom(VariantKey)`. The custom path resolves through
//! `resolve_custom_variant(...)` and yields an
//! `Option<Arc<dyn VariantStyle>>` that the renderer reads.
//!
//! These helpers centralize the resolve logic so the seven
//! components don't have to repeat the same `match` block in
//! their render methods. They are not a full ControlButton
//! mixin (that would touch the layout), just the
//! variant-resolution half.

use std::sync::Arc;

use crate::renderer::{ButtonVariant, VariantStyle, resolve_custom_variant};
use crate::theme::{ActionVariantKind, ActiveTheme, Theme};
use gpui::App;

/// Resolved variant state, ready to hand to a renderer.
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

/// Theme helper: a renderable `Hsla` for the resolved variant.
/// Returns the custom style's `bg` if present, otherwise the
/// built-in theme slot.
pub fn variant_bg(resolved: &ResolvedVariant, theme: &Theme) -> gpui::Hsla {
    use crate::renderer::VariantState;
    if let Some(s) = &resolved.custom_style {
        s.bg(&VariantState {
            disabled: false,
        })
    } else {
        let v = theme.action_variant(resolved.builtin);
        v.bg
    }
}
