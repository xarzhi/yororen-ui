//! Catppuccin color palettes (Latte / Frappé / Macchiato / Mocha).
//!
//! All colors are the official Catppuccin hex values from
//! <https://github.com/catppuccin/palette> (MIT-licensed). Helpers return
//! `gpui::Hsla` so a theme package can plug them straight into the v0.5
//! Renderer trait methods.
//!
//! Four flavors:
//! - **Latte**: light theme.
//! - **Frappé**: medium-dark theme.
//! - **Macchiato**: darker theme.
//! - **Mocha**: darkest theme (most popular, default for Catppuccin
//!   wallpapers / terminal themes).
//!
//! Each flavor exposes the same 26 color slots, so a theme package can
//! pick any flavor and reach the right token by the same name.

use gpui::{Hsla, hsla, rgb};

/// Catppuccin Latte (light).
pub mod latte {
    use super::*;

    pub fn rosewater() -> Hsla {
        rgb(0xDC8A78).into()
    }
    pub fn flamingo() -> Hsla {
        rgb(0xDD7878).into()
    }
    pub fn pink() -> Hsla {
        rgb(0xEA76CB).into()
    }
    pub fn mauve() -> Hsla {
        rgb(0x8839EF).into()
    }
    pub fn red() -> Hsla {
        rgb(0xD20F39).into()
    }
    pub fn maroon() -> Hsla {
        rgb(0xE64553).into()
    }
    pub fn peach() -> Hsla {
        rgb(0xFE640B).into()
    }
    pub fn yellow() -> Hsla {
        rgb(0xDF8E1D).into()
    }
    pub fn green() -> Hsla {
        rgb(0x40A02B).into()
    }
    pub fn teal() -> Hsla {
        rgb(0x179299).into()
    }
    pub fn sky() -> Hsla {
        rgb(0x04A5E5).into()
    }
    pub fn sapphire() -> Hsla {
        rgb(0x209FB5).into()
    }
    pub fn blue() -> Hsla {
        rgb(0x1E66F5).into()
    }
    pub fn lavender() -> Hsla {
        rgb(0x7287FD).into()
    }
    pub fn text() -> Hsla {
        rgb(0x4C4F69).into()
    }
    pub fn subtext1() -> Hsla {
        rgb(0x5C5F77).into()
    }
    pub fn subtext0() -> Hsla {
        rgb(0x6C6F85).into()
    }
    pub fn overlay2() -> Hsla {
        rgb(0x7C7F93).into()
    }
    pub fn overlay1() -> Hsla {
        rgb(0x8C8FA1).into()
    }
    pub fn overlay0() -> Hsla {
        rgb(0x9CA0B0).into()
    }
    pub fn surface2() -> Hsla {
        rgb(0xACB0BE).into()
    }
    pub fn surface1() -> Hsla {
        rgb(0xBCC0CC).into()
    }
    pub fn surface0() -> Hsla {
        rgb(0xCCD0DA).into()
    }
    pub fn base() -> Hsla {
        rgb(0xEFF1F5).into()
    }
    pub fn mantle() -> Hsla {
        rgb(0xE6E9EF).into()
    }
    pub fn crust() -> Hsla {
        rgb(0xDCE0E8).into()
    }
}

/// Catppuccin Frappé.
pub mod frappe {
    use super::*;

    pub fn rosewater() -> Hsla {
        rgb(0xF2D5CF).into()
    }
    pub fn flamingo() -> Hsla {
        rgb(0xEEBEBE).into()
    }
    pub fn pink() -> Hsla {
        rgb(0xF4B8E4).into()
    }
    pub fn mauve() -> Hsla {
        rgb(0xCA9EE6).into()
    }
    pub fn red() -> Hsla {
        rgb(0xE78284).into()
    }
    pub fn maroon() -> Hsla {
        rgb(0xEA999C).into()
    }
    pub fn peach() -> Hsla {
        rgb(0xEF9F76).into()
    }
    pub fn yellow() -> Hsla {
        rgb(0xE5C890).into()
    }
    pub fn green() -> Hsla {
        rgb(0xA6D189).into()
    }
    pub fn teal() -> Hsla {
        rgb(0x81C8BE).into()
    }
    pub fn sky() -> Hsla {
        rgb(0x99D1DB).into()
    }
    pub fn sapphire() -> Hsla {
        rgb(0x85C1DC).into()
    }
    pub fn blue() -> Hsla {
        rgb(0x8CAAEE).into()
    }
    pub fn lavender() -> Hsla {
        rgb(0xBABBF1).into()
    }
    pub fn text() -> Hsla {
        rgb(0xC6D0F5).into()
    }
    pub fn subtext1() -> Hsla {
        rgb(0xB5BFE2).into()
    }
    pub fn subtext0() -> Hsla {
        rgb(0xA5ADCE).into()
    }
    pub fn overlay2() -> Hsla {
        rgb(0x949CBB).into()
    }
    pub fn overlay1() -> Hsla {
        rgb(0x838BA7).into()
    }
    pub fn overlay0() -> Hsla {
        rgb(0x737994).into()
    }
    pub fn surface2() -> Hsla {
        rgb(0x626880).into()
    }
    pub fn surface1() -> Hsla {
        rgb(0x51576D).into()
    }
    pub fn surface0() -> Hsla {
        rgb(0x414559).into()
    }
    pub fn base() -> Hsla {
        rgb(0x303446).into()
    }
    pub fn mantle() -> Hsla {
        rgb(0x292C3C).into()
    }
    pub fn crust() -> Hsla {
        rgb(0x232634).into()
    }
}

/// Catppuccin Macchiato.
pub mod macchiato {
    use super::*;

    pub fn rosewater() -> Hsla {
        rgb(0xF4DBD6).into()
    }
    pub fn flamingo() -> Hsla {
        rgb(0xF0C6C6).into()
    }
    pub fn pink() -> Hsla {
        rgb(0xF5BDE6).into()
    }
    pub fn mauve() -> Hsla {
        rgb(0xC6A0F6).into()
    }
    pub fn red() -> Hsla {
        rgb(0xED8796).into()
    }
    pub fn maroon() -> Hsla {
        rgb(0xEE99A0).into()
    }
    pub fn peach() -> Hsla {
        rgb(0xF5A97F).into()
    }
    pub fn yellow() -> Hsla {
        rgb(0xEED49F).into()
    }
    pub fn green() -> Hsla {
        rgb(0xA6DA95).into()
    }
    pub fn teal() -> Hsla {
        rgb(0x8BD5CA).into()
    }
    pub fn sky() -> Hsla {
        rgb(0x91D7E3).into()
    }
    pub fn sapphire() -> Hsla {
        rgb(0x7DC4E4).into()
    }
    pub fn blue() -> Hsla {
        rgb(0x8AADF4).into()
    }
    pub fn lavender() -> Hsla {
        rgb(0xB7BDF8).into()
    }
    pub fn text() -> Hsla {
        rgb(0xCAD3F5).into()
    }
    pub fn subtext1() -> Hsla {
        rgb(0xB8C0E0).into()
    }
    pub fn subtext0() -> Hsla {
        rgb(0xA5ADCB).into()
    }
    pub fn overlay2() -> Hsla {
        rgb(0x939AB7).into()
    }
    pub fn overlay1() -> Hsla {
        rgb(0x8087A2).into()
    }
    pub fn overlay0() -> Hsla {
        rgb(0x6E738D).into()
    }
    pub fn surface2() -> Hsla {
        rgb(0x5B6078).into()
    }
    pub fn surface1() -> Hsla {
        rgb(0x494D64).into()
    }
    pub fn surface0() -> Hsla {
        rgb(0x363A4F).into()
    }
    pub fn base() -> Hsla {
        rgb(0x24273A).into()
    }
    pub fn mantle() -> Hsla {
        rgb(0x1E2030).into()
    }
    pub fn crust() -> Hsla {
        rgb(0x181926).into()
    }
}

/// Catppuccin Mocha (default / most popular).
pub mod mocha {
    use super::*;

    pub fn rosewater() -> Hsla {
        rgb(0xF5E0DC).into()
    }
    pub fn flamingo() -> Hsla {
        rgb(0xF2CDCD).into()
    }
    pub fn pink() -> Hsla {
        rgb(0xF5C2E7).into()
    }
    pub fn mauve() -> Hsla {
        rgb(0xCBA6F7).into()
    }
    pub fn red() -> Hsla {
        rgb(0xF38BA8).into()
    }
    pub fn maroon() -> Hsla {
        rgb(0xEBA0AC).into()
    }
    pub fn peach() -> Hsla {
        rgb(0xFAB387).into()
    }
    pub fn yellow() -> Hsla {
        rgb(0xF9E2AF).into()
    }
    pub fn green() -> Hsla {
        rgb(0xA6E3A1).into()
    }
    pub fn teal() -> Hsla {
        rgb(0x94E2D5).into()
    }
    pub fn sky() -> Hsla {
        rgb(0x89DCEB).into()
    }
    pub fn sapphire() -> Hsla {
        rgb(0x74C7EC).into()
    }
    pub fn blue() -> Hsla {
        rgb(0x89B4FA).into()
    }
    pub fn lavender() -> Hsla {
        rgb(0xB4BEFE).into()
    }
    pub fn text() -> Hsla {
        rgb(0xCDD6F4).into()
    }
    pub fn subtext1() -> Hsla {
        rgb(0xBAC2DE).into()
    }
    pub fn subtext0() -> Hsla {
        rgb(0xA6ADC8).into()
    }
    pub fn overlay2() -> Hsla {
        rgb(0x9399B2).into()
    }
    pub fn overlay1() -> Hsla {
        rgb(0x7F849C).into()
    }
    pub fn overlay0() -> Hsla {
        rgb(0x6C7086).into()
    }
    pub fn surface2() -> Hsla {
        rgb(0x585B70).into()
    }
    pub fn surface1() -> Hsla {
        rgb(0x45475A).into()
    }
    pub fn surface0() -> Hsla {
        rgb(0x313244).into()
    }
    pub fn base() -> Hsla {
        rgb(0x1E1E2E).into()
    }
    pub fn mantle() -> Hsla {
        rgb(0x181825).into()
    }
    pub fn crust() -> Hsla {
        rgb(0x11111B).into()
    }
}

/// Helper: convert an `Hsla` to one with a forced alpha. Used to build
/// scrim / overlay tints from a base color.
pub fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: alpha, ..color }
}

/// A semi-transparent black scrim (over a light surface) for scrims and
/// overlays. Equivalent to `hsla(0.0, 0.0, 0.0, 0.45)`.
pub fn dark_scrim() -> Hsla {
    hsla(0.0, 0.0, 0.0, 0.45)
}
