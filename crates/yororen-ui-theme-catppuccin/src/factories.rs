//! Theme factories: `light()` = Latte, `dark()` = Mocha, plus
//! `frappe()` and `macchiato()` for the medium-dark and dark Catppuccin
//! flavors. All four return a `yororen_ui_renderer::theme::Theme` with
//! the Catppuccin `RendererRegistry` already wired in.

use gpui::{hsla, rgb};

use yororen_ui_core::i18n::TextDirection;
use yororen_ui_renderer::theme::tokens::DesignTokens;
use yororen_ui_renderer::theme::{
    ActionTheme, ActionVariant, BorderTheme, ContentTheme, ShadowTheme, StatusTheme, StatusVariant,
    SurfaceTheme, Theme,
};

use crate::palette;
use crate::renderer;

/// Build a `Theme` for one Catppuccin flavor. The four `light()`,
/// `frappe()`, `macchiato()`, `mocha()` factories all delegate here,
/// passing the right pair of base + accent palettes.
fn build_theme(base: BasePalette, accent: AccentPalette, is_dark: bool) -> Theme {
    let content = ContentTheme {
        primary: base.text,
        secondary: base.subtext1,
        tertiary: base.subtext0,
        disabled: base.overlay0,
        on_primary: base.base,
        on_status: if is_dark { base.base } else { base.crust },
    };

    let action = ActionTheme {
        neutral: ActionVariant {
            bg: base.surface0,
            hover_bg: base.surface1,
            active_bg: base.surface2,
            fg: content.primary,
            disabled_bg: base.surface0,
            disabled_fg: content.disabled,
        },
        primary: ActionVariant {
            bg: accent.blue,
            hover_bg: accent.sapphire,
            active_bg: accent.lavender,
            fg: base.base,
            disabled_bg: base.surface1,
            disabled_fg: content.disabled,
        },
        danger: ActionVariant {
            bg: accent.red,
            hover_bg: accent.maroon,
            active_bg: accent.peach,
            fg: base.base,
            disabled_bg: base.surface1,
            disabled_fg: content.disabled,
        },
    };

    Theme {
        surface: SurfaceTheme {
            canvas: base.crust,
            base: base.base,
            raised: base.mantle,
            sunken: base.crust,
            hover: base.surface0,
        },
        content: content.clone(),
        border: BorderTheme {
            default: base.surface1,
            muted: base.surface0,
            focus: accent.mauve,
            divider: base.surface0,
        },
        action,
        status: StatusTheme {
            success: StatusVariant {
                bg: accent.green,
                fg: content.on_status,
            },
            warning: StatusVariant {
                bg: accent.yellow,
                fg: content.on_status,
            },
            error: StatusVariant {
                bg: accent.red,
                fg: content.on_status,
            },
            info: StatusVariant {
                bg: accent.sapphire,
                fg: content.on_status,
            },
        },
        shadow: ShadowTheme {
            elevation_1: hsla(0.0, 0.0, 0.0, if is_dark { 0.35 } else { 0.10 }),
            elevation_2: hsla(0.0, 0.0, 0.0, if is_dark { 0.50 } else { 0.18 }),
        },
        text_direction: TextDirection::Ltr,
        tokens: DesignTokens::default(),
        renderers: renderer::catppuccin_registry(),
    }
}

/// Snapshot of the 13 "structural" colors (backgrounds, text, dividers,
/// borders) used by the base theme. Accent colors come from a
/// separate [`AccentPalette`].
#[derive(Clone, Copy)]
struct BasePalette {
    text: gpui::Hsla,
    subtext1: gpui::Hsla,
    subtext0: gpui::Hsla,
    overlay0: gpui::Hsla,
    base: gpui::Hsla,
    mantle: gpui::Hsla,
    crust: gpui::Hsla,
    surface0: gpui::Hsla,
    surface1: gpui::Hsla,
    surface2: gpui::Hsla,
}

/// Snapshot of the 14 "accent" colors (the saturated pastels used for
/// primary / danger / status). Picking a different accent from the
/// base lets a theme package remix colors without rewriting the
/// whole palette.
#[derive(Clone, Copy)]
struct AccentPalette {
    blue: gpui::Hsla,
    sapphire: gpui::Hsla,
    lavender: gpui::Hsla,
    mauve: gpui::Hsla,
    red: gpui::Hsla,
    maroon: gpui::Hsla,
    peach: gpui::Hsla,
    yellow: gpui::Hsla,
    green: gpui::Hsla,
}

impl BasePalette {
    fn latte() -> Self {
        Self {
            text: palette::latte::text(),
            subtext1: palette::latte::subtext1(),
            subtext0: palette::latte::subtext0(),
            overlay0: palette::latte::overlay0(),
            base: palette::latte::base(),
            mantle: palette::latte::mantle(),
            crust: palette::latte::crust(),
            surface0: palette::latte::surface0(),
            surface1: palette::latte::surface1(),
            surface2: palette::latte::surface2(),
        }
    }
    fn frappe() -> Self {
        Self {
            text: palette::frappe::text(),
            subtext1: palette::frappe::subtext1(),
            subtext0: palette::frappe::subtext0(),
            overlay0: palette::frappe::overlay0(),
            base: palette::frappe::base(),
            mantle: palette::frappe::mantle(),
            crust: palette::frappe::crust(),
            surface0: palette::frappe::surface0(),
            surface1: palette::frappe::surface1(),
            surface2: palette::frappe::surface2(),
        }
    }
    fn macchiato() -> Self {
        Self {
            text: palette::macchiato::text(),
            subtext1: palette::macchiato::subtext1(),
            subtext0: palette::macchiato::subtext0(),
            overlay0: palette::macchiato::overlay0(),
            base: palette::macchiato::base(),
            mantle: palette::macchiato::mantle(),
            crust: palette::macchiato::crust(),
            surface0: palette::macchiato::surface0(),
            surface1: palette::macchiato::surface1(),
            surface2: palette::macchiato::surface2(),
        }
    }
    fn mocha() -> Self {
        Self {
            text: palette::mocha::text(),
            subtext1: palette::mocha::subtext1(),
            subtext0: palette::mocha::subtext0(),
            overlay0: palette::mocha::overlay0(),
            base: palette::mocha::base(),
            mantle: palette::mocha::mantle(),
            crust: palette::mocha::crust(),
            surface0: palette::mocha::surface0(),
            surface1: palette::mocha::surface1(),
            surface2: palette::mocha::surface2(),
        }
    }
}

impl AccentPalette {
    fn latte() -> Self {
        Self {
            blue: palette::latte::blue(),
            sapphire: palette::latte::sapphire(),
            lavender: palette::latte::lavender(),
            mauve: palette::latte::mauve(),
            red: palette::latte::red(),
            maroon: palette::latte::maroon(),
            peach: palette::latte::peach(),
            yellow: palette::latte::yellow(),
            green: palette::latte::green(),
        }
    }
    fn frappe() -> Self {
        Self {
            blue: palette::frappe::blue(),
            sapphire: palette::frappe::sapphire(),
            lavender: palette::frappe::lavender(),
            mauve: palette::frappe::mauve(),
            red: palette::frappe::red(),
            maroon: palette::frappe::maroon(),
            peach: palette::frappe::peach(),
            yellow: palette::frappe::yellow(),
            green: palette::frappe::green(),
        }
    }
    fn macchiato() -> Self {
        Self {
            blue: palette::macchiato::blue(),
            sapphire: palette::macchiato::sapphire(),
            lavender: palette::macchiato::lavender(),
            mauve: palette::macchiato::mauve(),
            red: palette::macchiato::red(),
            maroon: palette::macchiato::maroon(),
            peach: palette::macchiato::peach(),
            yellow: palette::macchiato::yellow(),
            green: palette::macchiato::green(),
        }
    }
    fn mocha() -> Self {
        Self {
            blue: palette::mocha::blue(),
            sapphire: palette::mocha::sapphire(),
            lavender: palette::mocha::lavender(),
            mauve: palette::mocha::mauve(),
            red: palette::mocha::red(),
            maroon: palette::mocha::maroon(),
            peach: palette::mocha::peach(),
            yellow: palette::mocha::yellow(),
            green: palette::mocha::green(),
        }
    }
}

/// Light theme using the Latte palette.
pub fn light() -> Theme {
    build_theme(BasePalette::latte(), AccentPalette::latte(), false)
}

/// Frappé theme (medium-dark).
pub fn frappe() -> Theme {
    build_theme(BasePalette::frappe(), AccentPalette::frappe(), true)
}

/// Macchiato theme (darker than Frappé, lighter than Mocha).
pub fn macchiato() -> Theme {
    build_theme(BasePalette::macchiato(), AccentPalette::macchiato(), true)
}

/// Mocha theme (darkest, most popular Catppuccin flavor).
pub fn mocha() -> Theme {
    build_theme(BasePalette::mocha(), AccentPalette::mocha(), true)
}

/// Default dark theme: alias for `mocha()`. Kept for the public API
/// surface so `catppuccin::dark()` is discoverable.
pub fn dark() -> Theme {
    mocha()
}

/// Alias for [`light`] returning a strongly-typed `LatteTheme` for
/// documentation / downcasting. Today this is just a `Theme`; the
/// alias is provided so future versions can add Latte-specific
/// extensions without breaking the public API.
pub type LatteTheme = Theme;

/// Alias for Frappé.
pub type FrappeTheme = Theme;

/// Alias for Macchiato.
pub type MacchiatoTheme = Theme;

/// Alias for Mocha.
pub type MochaTheme = Theme;

/// Build a Latte-flavoured `Theme` (light).
pub fn latte_theme() -> Theme {
    light()
}

/// Build a Frappé-flavoured `Theme`.
pub fn frappe_theme() -> Theme {
    frappe()
}

/// Build a Macchiato-flavoured `Theme`.
pub fn macchiato_theme() -> Theme {
    macchiato()
}

/// Build a Mocha-flavoured `Theme` (dark).
pub fn mocha_theme() -> Theme {
    mocha()
}

// Silence dead_code: the per-flavor typed alias (`LatteTheme` etc.)
// are public re-exports.
#[allow(dead_code)]
fn _force_compile(_: u8) {
    let _ = rgb(0x000000);
}
