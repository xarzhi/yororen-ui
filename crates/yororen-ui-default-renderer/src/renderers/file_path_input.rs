//! `FilePathInputRenderer` — visual side of `FilePathInput`.
//!
//! v0.3 implementation: reuses `TextInputElement` plus a folder
//! icon at the leading edge and a "browse" button at the
//! trailing edge. Clicking the browse button opens a native
//! file dialog via `App::prompt_for_paths`; the chosen path
//! is written to the state and fired through `on_change`.
//! The user's `on_browse` becomes a post-pick hook that
//! receives the selected path (empty string on cancel).

use std::any::Any;
use std::sync::Arc;

use gpui::{
    AnyElement, App, AppContext, Div, Hsla, InteractiveElement, IntoElement, MouseButton,
    ParentElement, Pixels, Stateful, StatefulInteractiveElement, Styled, Window, div, px,
};
use yororen_ui_core::headless::file_path_input::FilePathInputProps;
use yororen_ui_core::headless::icon::{IconSource, icon};
use yororen_ui_core::headless::text_input::TextInputState;
use yororen_ui_core::renderer::{RendererContext, markers};
use yororen_ui_core::theme::{ActiveTheme, Theme};

use crate::renderers::text_input::{TextInputElement, start_cursor_blink, wire_input_keyboard};
pub use yororen_ui_core::renderer::file_path_input::{
    FilePathInputRenderState, FilePathInputRenderer,
};
use yororen_ui_core::renderer::spec::Edges;

pub struct TokenFilePathInputRenderer;

impl FilePathInputRenderer for TokenFilePathInputRenderer {
    fn bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn focus_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.focus").unwrap_or_default()
    }
    fn hover_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.muted").unwrap_or_default()
    }
    fn active_border(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("border.default").unwrap_or_default()
    }
    fn button_bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        // The browse button is a small affordance inside the
        // input, not a primary action. Match the input surface
        // (white in light, dark gray in dark) so the icon
        // doesn't compete with the typed path.
        theme.get_color("surface.base").unwrap_or_default()
    }
    fn button_fg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        // Use the same color as the typed text. On hover, the
        // theme can override via `custom_button_fg` or by
        // registering a custom `FilePathInputRenderer`.
        theme.get_color("content.primary").unwrap_or_default()
    }
    fn button_hover_bg(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme.get_color("surface.hover").unwrap_or_default()
    }
    fn min_height(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.file_path_input.min_height")
            .unwrap_or(0.0) as f32)
    }
    fn padding(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Edges<Pixels> {
        Edges::symmetric(
            px(theme
                .get_number("tokens.control.file_path_input.horizontal_padding")
                .unwrap_or(0.0) as f32),
            px(theme
                .get_number("tokens.control.input.vertical_padding")
                .unwrap_or(0.0) as f32),
        )
    }
    fn action_gap(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.file_path_input.action_gap")
            .unwrap_or(0.0) as f32)
    }
    fn border_radius(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme.get_number("tokens.radii.md").unwrap_or(0.0) as f32)
    }
    fn icon_size(&self, _state: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(theme
            .get_number("tokens.control.file_path_input.icon_size")
            .unwrap_or(0.0) as f32)
    }
}

pub fn arc_file_path_input<T: FilePathInputRenderer + 'static>(
    r: T,
) -> Arc<dyn FilePathInputRenderer> {
    Arc::new(r)
}
