//! Light / dark `Theme` factories for the Material 3 package.
//!
//! Each factory builds a `Theme` whose `palette` slots (surface,
//! content, border, action, status) are populated from the M3
//! baseline schemes in [`palette`]. Visual styling is provided by
//! the M3-flavored `Renderer` implementations in [`renderer`]; the
//! factories install the renderer registry by default.

use gpui::{App, Hsla, WindowAppearance};

use yororen_ui_core::i18n::TextDirection;
use yororen_ui_core::theme::tokens::DesignTokens;
use yororen_ui_core::theme::{
    ActionTheme, ActionVariant, BorderTheme, ContentTheme, GlobalTheme, ShadowTheme, StatusTheme,
    StatusVariant, SurfaceTheme, Theme,
};

use crate::palette::{dark, light, rgb};

/// M3 "elevated" tone: a surface that sits 1dp above the base
/// surface. M3 uses a tonal elevation system; for the simple
/// renderer set we add a small `surface_tint` layer of primary at
/// ~5% alpha to the base.
fn elevated(base: Hsla, primary: Hsla, alpha: f32) -> Hsla {
    crate::palette::apply_state_layer(base, primary, alpha)
}

fn build_palette(light: bool) -> (SurfaceTheme, ContentTheme, BorderTheme, ActionTheme, StatusTheme, Hsla) {
    let (primary, on_primary, _primary_container, _on_primary_container) = if light {
        (rgb(light::PRIMARY), rgb(light::ON_PRIMARY), rgb(light::PRIMARY_CONTAINER), rgb(light::ON_PRIMARY_CONTAINER))
    } else {
        (rgb(dark::PRIMARY), rgb(dark::ON_PRIMARY), rgb(dark::PRIMARY_CONTAINER), rgb(dark::ON_PRIMARY_CONTAINER))
    };
    let (_secondary, _on_secondary, _secondary_container, _on_secondary_container) = if light {
        (rgb(light::SECONDARY), rgb(light::ON_SECONDARY), rgb(light::SECONDARY_CONTAINER), rgb(light::ON_SECONDARY_CONTAINER))
    } else {
        (rgb(dark::SECONDARY), rgb(dark::ON_SECONDARY), rgb(dark::SECONDARY_CONTAINER), rgb(dark::ON_SECONDARY_CONTAINER))
    };
    let (_tertiary, _on_tertiary, _tertiary_container, _on_tertiary_container) = if light {
        (rgb(light::TERTIARY), rgb(light::ON_TERTIARY), rgb(light::TERTIARY_CONTAINER), rgb(light::ON_TERTIARY_CONTAINER))
    } else {
        (rgb(dark::TERTIARY), rgb(dark::ON_TERTIARY), rgb(dark::TERTIARY_CONTAINER), rgb(dark::ON_TERTIARY_CONTAINER))
    };
    let (error, on_error, _error_container, _on_error_container) = if light {
        (rgb(light::ERROR), rgb(light::ON_ERROR), rgb(light::ERROR_CONTAINER), rgb(light::ON_ERROR_CONTAINER))
    } else {
        (rgb(dark::ERROR), rgb(dark::ON_ERROR), rgb(dark::ERROR_CONTAINER), rgb(dark::ON_ERROR_CONTAINER))
    };
    let (background, on_background, surface, on_surface, _surface_variant, on_surface_variant, _outline, outline_variant) = if light {
        (
            rgb(light::BACKGROUND), rgb(light::ON_BACKGROUND),
            rgb(light::SURFACE), rgb(light::ON_SURFACE),
            rgb(light::SURFACE_VARIANT), rgb(light::ON_SURFACE_VARIANT),
            rgb(light::OUTLINE), rgb(light::OUTLINE_VARIANT),
        )
    } else {
        (
            rgb(dark::BACKGROUND), rgb(dark::ON_BACKGROUND),
            rgb(dark::SURFACE), rgb(dark::ON_SURFACE),
            rgb(dark::SURFACE_VARIANT), rgb(dark::ON_SURFACE_VARIANT),
            rgb(dark::OUTLINE), rgb(dark::OUTLINE_VARIANT),
        )
    };

    // M3 surfaces: `canvas` is the lowest layer (background), `base`
    // is the default surface, `raised` is the surface +1dp tinted
    // with primary at 5%, `sunken` is the dialog / scrim background.
    let surface_theme = SurfaceTheme {
        canvas: background,
        base: surface,
        raised: elevated(surface, primary, 0.05),
        sunken: elevated(surface, primary, 0.10),
        hover: elevated(surface, primary, 0.08),
    };
    let content_theme = ContentTheme {
        primary: on_surface,
        secondary: on_surface_variant,
        tertiary: on_surface_variant,
        disabled: on_surface_variant,
        on_primary,
        on_status: on_background,
    };
    let border_theme = BorderTheme {
        default: outline_variant,
        muted: outline_variant,
        focus: primary,
        divider: outline_variant,
    };
    // M3 "filled" / "elevated" / "tonal" button styles. We use
    // the "filled" style (action.primary = primary bg, on_primary
    // fg), "outlined" style (action.neutral = transparent, content
    // fg, primary border), and "text" style (no background).
    // For the "danger" variant we reuse the M3 error color.
    let action_theme = ActionTheme {
        neutral: ActionVariant {
            // M3 "outlined" / "text" button look: transparent on
            // surface, but the renderer is responsible for adding
            // the actual outline. We keep `bg` = surface, `fg` =
            // primary so the builder-level bg/fg read like a real
            // M3 button when not overridden.
            bg: surface,
            hover_bg: elevated(surface, primary, 0.08),
            active_bg: elevated(surface, primary, 0.12),
            fg: primary,
            disabled_bg: surface,
            disabled_fg: on_surface_variant,
        },
        primary: ActionVariant {
            // M3 "filled" button.
            bg: primary,
            hover_bg: elevated(primary, on_primary, 0.08),
            active_bg: elevated(primary, on_primary, 0.12),
            fg: on_primary,
            disabled_bg: elevated(surface, on_surface_variant, 0.12),
            disabled_fg: on_surface_variant,
        },
        danger: ActionVariant {
            // M3 error / destructive action.
            bg: error,
            hover_bg: elevated(error, on_error, 0.08),
            active_bg: elevated(error, on_error, 0.12),
            fg: on_error,
            disabled_bg: elevated(surface, on_error, 0.12),
            disabled_fg: on_surface_variant,
        },
    };
    // M3 status uses container + on-container. We approximate
    // with the on-color text on a tinted surface.
    let status_theme = StatusTheme {
        success: StatusVariant {
            bg: elevated(surface, primary, 0.12), // M3 has no "success" — use a tinted primary
            fg: on_background,
        },
        warning: StatusVariant {
            bg: elevated(surface, primary, 0.16),
            fg: on_background,
        },
        error: StatusVariant {
            bg: error,
            fg: on_error,
        },
        info: StatusVariant {
            bg: elevated(surface, primary, 0.08),
            fg: on_background,
        },
    };
    (surface_theme, content_theme, border_theme, action_theme, status_theme, primary)
}

fn build_shadow(light: bool, primary: Hsla) -> ShadowTheme {
    // M3 elevates shadows tinted with the primary color. We
    // approximate with a single tinted shadow.
    let _ = light;
    ShadowTheme {
        elevation_1: crate::palette::shadow_for_elevation(1, primary),
        elevation_2: crate::palette::shadow_for_elevation(3, primary),
    }
}

/// Build a `Theme` for the M3 Light scheme.
pub fn light() -> Theme {
    let (surface, content, border, action, status, primary) = build_palette(true);
    let shadow = build_shadow(true, primary);
    Theme {
        surface,
        content,
        border,
        action,
        status,
        shadow,
        text_direction: TextDirection::Ltr,
        tokens: DesignTokens::default(),
        renderers: crate::renderer::material_registry(),
    }
}

/// Build a `Theme` for the M3 Dark scheme.
pub fn dark() -> Theme {
    let (surface, content, border, action, status, primary) = build_palette(false);
    let shadow = build_shadow(false, primary);
    Theme {
        surface,
        content,
        border,
        action,
        status,
        shadow,
        text_direction: TextDirection::Ltr,
        tokens: DesignTokens::default(),
        renderers: crate::renderer::material_registry(),
    }
}

/// Install the Material 3 theme on the given `App`.
pub fn install(cx: &mut App, appearance: WindowAppearance) {
    let theme = match appearance {
        WindowAppearance::Light | WindowAppearance::VibrantLight => light(),
        WindowAppearance::Dark | WindowAppearance::VibrantDark => dark(),
    };
    cx.set_global(GlobalTheme::new(theme));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_and_dark_distinct() {
        let l = light();
        let d = dark();
        assert_ne!(l.surface.base, d.surface.base);
    }

    #[test]
    fn light_uses_purple40() {
        let theme = light();
        // M3 light scheme has the purple40 primary as the
        // `border.focus` color and the `action.primary.bg`.
        let purple40 = rgb(light::PRIMARY);
        assert_eq!(theme.border.focus, purple40);
    }

    #[test]
    fn dark_uses_purple80() {
        let theme = dark();
        let purple80 = rgb(dark::PRIMARY);
        assert_eq!(theme.border.focus, purple80);
    }

    #[test]
    fn renderers_installed() {
        // Sanity: the renderer registry is the material one, not
        // the default `token_based()`.
        let l = light();
        // Catppuccin uses 12-px button radius; material uses pill.
        let state = yororen_ui_core::renderer::ButtonRenderState::default();
        let r = l.renderers.button.border_radius(&state, &l);
        assert!(r.to_f64() > 100.0, "expected pill radius (~999 px), got {}", r.to_f64());
    }
}
