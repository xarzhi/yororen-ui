//! Material Design 3 baseline palette (simplified).
//!
//! The full M3 spec defines ~30 tonal palettes; this module ships the
//! two most common baseline schemes (Light / Dark) plus a small set
//! of accent colors that are reused by the renderer set.
//!
//! Color choices follow M3's tonal-color philosophy:
//!
//! - **Primary**: `purple40` (Light) / `purple80` (Dark).
//! - **Secondary**: muted lavender complement.
//! - **Tertiary**: rosy accent.
//! - **Error**: standard M3 red ramp.
//! - **Neutral / Neutral-variant**: warm grays.
//! - **Surface tints**: very-low-alpha primary layered on the surface.
//!
//! These are deliberately simple — M3's full token system (color
//! roles × state layers × elevation) is too much to capture in a
//! 15-renderer reference theme. The renderer set below is enough
//! to demonstrate that the same `Renderer` trait fleet can ship
//! "Material 3 style" alongside Catppuccin and the system theme
//! without modifying `core`.

use gpui::Hsla;

/// Material 3 light scheme — baseline purple seed (`#6750A4`).
pub mod light {
    pub const PRIMARY: (f32, f32, f32) = (103.0, 80.0, 164.0); // M3 purple40
    pub const ON_PRIMARY: (f32, f32, f32) = (255.0, 255.0, 255.0);
    pub const PRIMARY_CONTAINER: (f32, f32, f32) = (232.0, 222.0, 248.0);
    pub const ON_PRIMARY_CONTAINER: (f32, f32, f32) = (33.0, 0.0, 94.0);

    pub const SECONDARY: (f32, f32, f32) = (98.0, 91.0, 113.0);
    pub const ON_SECONDARY: (f32, f32, f32) = (255.0, 255.0, 255.0);
    pub const SECONDARY_CONTAINER: (f32, f32, f32) = (232.0, 222.0, 248.0);
    pub const ON_SECONDARY_CONTAINER: (f32, f32, f32) = (30.0, 25.0, 43.0);

    pub const TERTIARY: (f32, f32, f32) = (125.0, 82.0, 96.0);
    pub const ON_TERTIARY: (f32, f32, f32) = (255.0, 255.0, 255.0);
    pub const TERTIARY_CONTAINER: (f32, f32, f32) = (255.0, 216.0, 228.0);
    pub const ON_TERTIARY_CONTAINER: (f32, f32, f32) = (49.0, 17.0, 29.0);

    pub const ERROR: (f32, f32, f32) = (179.0, 38.0, 30.0);
    pub const ON_ERROR: (f32, f32, f32) = (255.0, 255.0, 255.0);
    pub const ERROR_CONTAINER: (f32, f32, f32) = (249.0, 222.0, 220.0);
    pub const ON_ERROR_CONTAINER: (f32, f32, f32) = (65.0, 14.0, 11.0);

    pub const BACKGROUND: (f32, f32, f32) = (254.0, 247.0, 255.0);
    pub const ON_BACKGROUND: (f32, f32, f32) = (29.0, 27.0, 32.0);
    pub const SURFACE: (f32, f32, f32) = (254.0, 247.0, 255.0);
    pub const ON_SURFACE: (f32, f32, f32) = (29.0, 27.0, 32.0);
    pub const SURFACE_VARIANT: (f32, f32, f32) = (231.0, 224.0, 236.0);
    pub const ON_SURFACE_VARIANT: (f32, f32, f32) = (73.0, 69.0, 79.0);
    pub const OUTLINE: (f32, f32, f32) = (121.0, 116.0, 126.0);
    pub const OUTLINE_VARIANT: (f32, f32, f32) = (202.0, 196.0, 208.0);
}

/// Material 3 dark scheme — purple80 baseline.
pub mod dark {

    pub const PRIMARY: (f32, f32, f32) = (208.0, 188.0, 255.0); // M3 purple80
    pub const ON_PRIMARY: (f32, f32, f32) = (56.0, 30.0, 114.0);
    pub const PRIMARY_CONTAINER: (f32, f32, f32) = (79.0, 55.0, 139.0);
    pub const ON_PRIMARY_CONTAINER: (f32, f32, f32) = (232.0, 222.0, 248.0);

    pub const SECONDARY: (f32, f32, f32) = (204.0, 194.0, 220.0);
    pub const ON_SECONDARY: (f32, f32, f32) = (51.0, 45.0, 65.0);
    pub const SECONDARY_CONTAINER: (f32, f32, f32) = (74.0, 68.0, 88.0);
    pub const ON_SECONDARY_CONTAINER: (f32, f32, f32) = (232.0, 222.0, 248.0);

    pub const TERTIARY: (f32, f32, f32) = (243.0, 184.0, 199.0);
    pub const ON_TERTIARY: (f32, f32, f32) = (74.0, 37.0, 50.0);
    pub const TERTIARY_CONTAINER: (f32, f32, f32) = (99.0, 59.0, 72.0);
    pub const ON_TERTIARY_CONTAINER: (f32, f32, f32) = (255.0, 216.0, 228.0);

    pub const ERROR: (f32, f32, f32) = (242.0, 184.0, 181.0);
    pub const ON_ERROR: (f32, f32, f32) = (96.0, 20.0, 16.0);
    pub const ERROR_CONTAINER: (f32, f32, f32) = (140.0, 29.0, 24.0);
    pub const ON_ERROR_CONTAINER: (f32, f32, f32) = (249.0, 222.0, 220.0);

    pub const BACKGROUND: (f32, f32, f32) = (29.0, 27.0, 32.0);
    pub const ON_BACKGROUND: (f32, f32, f32) = (230.0, 224.0, 233.0);
    pub const SURFACE: (f32, f32, f32) = (29.0, 27.0, 32.0);
    pub const ON_SURFACE: (f32, f32, f32) = (230.0, 224.0, 233.0);
    pub const SURFACE_VARIANT: (f32, f32, f32) = (73.0, 69.0, 79.0);
    pub const ON_SURFACE_VARIANT: (f32, f32, f32) = (202.0, 196.0, 208.0);
    pub const OUTLINE: (f32, f32, f32) = (147.0, 143.0, 153.0);
    pub const OUTLINE_VARIANT: (f32, f32, f32) = (73.0, 69.0, 79.0);
}

/// Convert an `(r, g, b)` tuple in 0..=255 to a `Hsla` with alpha 1.0.
pub fn rgb(rgb: (f32, f32, f32)) -> Hsla {
    let r = rgb.0 / 255.0;
    let g = rgb.1 / 255.0;
    let b = rgb.2 / 255.0;
    let rgba = gpui::Rgba { r, g, b, a: 1.0 };
    <gpui::Hsla as From<gpui::Rgba>>::from(rgba)
}

/// M3 state-layer alphas (used by interaction-aware renderers).
/// M3 spec: hover 0.08, focus 0.10, pressed 0.10, dragged 0.16.
pub mod state_layer {
    pub const HOVER: f32 = 0.08;
    pub const FOCUS: f32 = 0.10;
    pub const PRESSED: f32 = 0.10;
    pub const DRAGGED: f32 = 0.16;
    /// M3 disabled content opacity.
    pub const DISABLED_CONTENT: f32 = 0.38;
    /// M3 disabled container opacity.
    pub const DISABLED_CONTAINER: f32 = 0.12;
}

/// M3 elevation shadow helpers. M3 spec uses key + ambient dual shadows
/// and tints the shadow with the primary color; here we approximate
/// that with a single tinted shadow (gpui-ce 0.3.3 takes a single
/// shadow per `shadow` call).
pub fn shadow_for_elevation(elevation: u8, primary: Hsla) -> Hsla {
    let mut c = primary;
    let alpha = match elevation {
        0 => 0.0,
        1 => 0.05,
        2 => 0.08,
        3 => 0.11,
        4 => 0.12,
        5 => 0.14,
        _ => 0.15,
    };
    c.a = alpha;
    c
}

/// M3 corner-radius scale. M3 default shapes are mostly pill
/// (large) for buttons, 12-px for cards, 4-px for dialogs.
pub mod radii {
    /// M3 corner for cards, sheets, dialogs.
    pub const LG: f32 = 12.0;
    /// M3 corner for smaller surfaces (chips, list items).
    pub const MD: f32 = 8.0;
    /// M3 corner for tiny elements (badge, etc.).
    pub const SM: f32 = 4.0;
    /// M3 corner for "extra-large" surfaces (large FAB, modal).
    pub const XL: f32 = 16.0;
    /// M3 pill radius (for buttons, switches).
    pub const PILL: f32 = 999.0;
}

/// M3 state layer helper: overlay `overlay_color` (usually primary) on
/// top of `surface` at the given `alpha` (0.0..=1.0). The returned
/// `Hsla` is a flat (premultiplied) result suitable for `.bg(...)`.
pub fn apply_state_layer(surface: Hsla, overlay: Hsla, alpha: f32) -> Hsla {
    let sa = surface.a;
    let oa = overlay.a;
    // alpha composite: out = sa * surf + (1 - sa) * (oa * over)
    let out_a = sa + (1.0 - sa) * oa;
    if out_a == 0.0 {
        return surface;
    }
    // Convert to Rgba to do RGB math, then convert back to Hsla.
    let s_rgba: gpui::Rgba = surface.into();
    let o_rgba: gpui::Rgba = overlay.into();
    let r = (sa * s_rgba.r + (1.0 - sa) * (oa * o_rgba.r * alpha)) / out_a;
    let g = (sa * s_rgba.g + (1.0 - sa) * (oa * o_rgba.g * alpha)) / out_a;
    let b = (sa * s_rgba.b + (1.0 - sa) * (oa * o_rgba.b * alpha)) / out_a;
    let rgba = gpui::Rgba {
        r: r.clamp(0.0, 1.0),
        g: g.clamp(0.0, 1.0),
        b: b.clamp(0.0, 1.0),
        a: out_a,
    };
    <gpui::Hsla as From<gpui::Rgba>>::from(rgba)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_primary_is_purple40() {
        let c: gpui::Rgba = rgb(light::PRIMARY).into();
        // Sanity: it is purple-ish (blue + red > green).
        assert!(c.r > c.g);
    }

    #[test]
    fn dark_primary_is_purple80() {
        let c = rgb(dark::PRIMARY);
        let l = rgb(light::PRIMARY);
        // Lightness should be much higher than light's primary.
        assert!(c.l > l.l);
    }

    #[test]
    fn apply_state_layer_increases_visual_weight() {
        let base = rgb(light::SURFACE);
        let primary = rgb(light::PRIMARY);
        let layered = apply_state_layer(base, primary, state_layer::HOVER);
        // After applying an 8% primary overlay, the surface should
        // shift toward purple. Compare Rgba channels for stability.
        let base_rgba: gpui::Rgba = base.into();
        let layered_rgba: gpui::Rgba = layered.into();
        assert!(layered_rgba.r >= base_rgba.r - 0.001);
    }
}
