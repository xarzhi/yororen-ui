//! Material Design 3 (M3) theme package for `yororen-ui`.
//!
//! This crate is the v0.6 reference implementation of a "second
//! official theme" — it shows that the same `Renderer` trait
//! fleet from `core` can ship a completely different visual
//! language (Material's pill buttons, 4-px filled text fields,
//! 28-px dialog corners, M3 state layers) without modifying
//! `yororen-ui-core`.
//!
//! # Quick start
//!
//! ```rust,ignore
//! use yororen_ui_theme_material as material;
//!
//! material::install(cx, window.appearance());
//! ```
//!
//! # What this crate ships
//!
//! - A [`palette`] module with the M3 light / dark baseline schemes
//!   (purple seed `#6750A4`).
//! - [`light`](factories::light) / [`dark`](factories::dark) factories
//!   for the two baseline schemes.
//! - 20 [`MaterialXxxRenderer`](renderer) implementations covering
//!   the most common components (button, icon_button, label, heading,
//!   divider, focus_ring, badge, tag, list_item, switch, checkbox,
//!   radio, text_input, modal, popover, toast, tooltip, panel, card,
//!   avatar).
//! - A [`material_registry`](renderer::material_registry) helper that
//!   returns a `RendererRegistry` with the M3 renderers installed for
//!   those 20 components; other components fall back to the default
//!   `TokenXxxRenderer` from `core`.
//! - An [`install`](factories::install) helper that puts everything on
//!   the `App` for you.
//!
//! # M3 state layers
//!
//! M3's interaction states are expressed as **state layers** — flat
//! primary-color overlays at low alpha (8% hover, 10% press). This
//! crate ships a `palette::apply_state_layer` helper that
//! alpha-composites the overlay onto any surface so a renderer can
//! derive hover / pressed / focus backgrounds without ad-hoc RGB
//! math.

pub mod palette;
pub mod renderer;

mod factories;

pub use factories::{dark, install, light, themeset};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn light_dark_themes_distinct() {
        let l = light();
        let d = dark();
        assert_ne!(l.surface.base, d.surface.base);
    }

    #[test]
    fn themeset_has_both() {
        let ts = themeset();
        assert!(ts.dark.is_some());
    }
}
