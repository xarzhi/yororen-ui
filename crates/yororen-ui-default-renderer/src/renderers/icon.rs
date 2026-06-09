//! `IconRenderer` — visual side of `Icon`.
//!
//! Bridges the headless [`IconProps`] (id + source + size + color)
//! to a concrete `gpui::Svg` element. SVG bytes are loaded via the
//! app's global `AssetSource` (the `yororen-ui-core::UiAsset`
//! bundle registered at startup).
//!
//! Mirrors the v0.2 reference implementation
//! (`src/component/icon.rs` at tag `v0.2.0`):
//!
//! ```ignore
//! svg().path(self.path).size(self.size).id(self.element_id).text_color(color)
//! ```
//!
//! The bundled icons use `stroke="currentColor"`, which `resvg`
//! resolves to a fixed color at parse time. gpui's SVG renderer
//! ignores the SVG's own stroke color — it renders the SVG to an
//! alpha mask and then draws a `MonochromeSprite` with the color
//! we pass. So the caller's `IconProps::color` is the only color
//! that matters at paint time.
//!
//! `gpui::Svg` reads its own local `style.text.color` at paint
//! time; it does NOT inherit `text_color` from a parent element
//! via `Window::text_style_stack`. We must call `text_color(c)`
//! on the SVG explicitly.

use gpui::{AnyElement, App, InteractiveElement, IntoElement, Styled, Window, svg};

use yororen_ui_core::headless::icon::{IconProps, IconSource};

/// Resolves an `IconSource` to an asset path the global
/// `AssetSource` can load.
///
/// `Builtin` icons live under the embedded `icons/` folder of
/// `yororen-ui-core::assets`; `Resource` paths are passed through
/// unchanged. The `.svg` extension is required — `paint_svg`
/// silently fails (`log_err`) when the asset is missing, leaving
/// the icon invisible.
fn resolve_icon_path(source: &IconSource) -> gpui::SharedString {
    match source {
        IconSource::Builtin(name) => gpui::SharedString::from(format!("icons/{}.svg", name)),
        IconSource::Resource(path) => path.clone(),
    }
}

pub trait DefaultIcon: Sized {
    fn default_render(self, _cx: &mut App, _window: &mut Window) -> AnyElement;
}

impl DefaultIcon for IconProps {
    fn default_render(self, _cx: &mut App, _window: &mut Window) -> AnyElement {
        let path = resolve_icon_path(&self.source);
        let size = self.size.unwrap_or(gpui::px(14.0));
        let color = self.color.unwrap_or_else(|| gpui::rgb(0x0A0A0A).into());
        // v0.2 pattern: chain `.path().size()` first (which stay
        // on `Svg`), then call `.id()` last (which produces
        // `Stateful<Svg>`) and finally `.text_color()` on
        // `Stateful<Svg>`. `Stateful<E>: Styled` so the
        // `text_color` setter is in scope.
        svg()
            .path(path)
            .size(size)
            .id(self.id)
            .text_color(color)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_path_includes_svg_extension() {
        // Regression: `paint_svg` silently fails (`log_err`) when
        // the asset path is missing the `.svg` extension, leaving
        // the icon invisible. Make sure the resolver never
        // returns a bare name without the extension.
        let p = resolve_icon_path(&IconSource::Builtin("search".into()));
        assert_eq!(p.as_ref(), "icons/search.svg");

        let p = resolve_icon_path(&IconSource::Builtin("close".into()));
        assert_eq!(p.as_ref(), "icons/close.svg");

        let p = resolve_icon_path(&IconSource::Builtin("folder".into()));
        assert_eq!(p.as_ref(), "icons/folder.svg");
    }

    #[test]
    fn resource_path_passes_through() {
        // `IconSource::Resource` is an explicit asset path the
        // caller owns; the resolver must not add any prefix.
        let p = resolve_icon_path(&IconSource::Resource("icons/custom-thing.svg".into()));
        assert_eq!(p.as_ref(), "icons/custom-thing.svg");
    }
}
