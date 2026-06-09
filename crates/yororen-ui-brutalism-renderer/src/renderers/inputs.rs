//! Brutalist input renderers: `TextInput`, `TextArea`,
//! `PasswordInput`, `NumberInput`, `FilePathInput`,
//! `SearchInput`, `Select`, `ComboBox`, `KeybindingInput`.
//!
//! The v0.3 headless wiring (text shaping, IME, focus, keymap)
//! lives in the default-renderer; this module only provides the
//! visual contract via the `XxxRenderer` traits.

use gpui::{Hsla, Pixels, hsla, px};
use yororen_ui_core::theme::Theme;
use yororen_ui_default_renderer::renderers::spec::Edges;

use crate::style::{BRUTAL_BORDER, BRUTAL_RADIUS, brutal_border_color};

// =====================================================================
// Shared brutalist input colors.
// =====================================================================

fn brutal_input_bg(disabled: bool, theme: &Theme) -> Hsla {
    if disabled {
        theme
            .get_color("surface.sunken")
            .unwrap_or(BRUTAL_BORDER)
    } else {
        theme
            .get_color("surface.base")
            .unwrap_or(BRUTAL_BORDER)
    }
}

fn brutal_input_border(disabled: bool, theme: &Theme) -> Hsla {
    if disabled {
        theme
            .get_color("border.muted")
            .unwrap_or(brutal_border_color(theme))
    } else {
        brutal_border_color(theme)
    }
}

fn brutal_input_focus_border(theme: &Theme) -> Hsla {
    theme
        .get_color("border.focus")
        .unwrap_or(BRUTAL_BORDER)
}

fn brutal_input_text_color(disabled: bool, theme: &Theme) -> Hsla {
    if disabled {
        theme
            .get_color("content.disabled")
            .unwrap_or(BRUTAL_BORDER)
    } else {
        theme
            .get_color("content.primary")
            .unwrap_or(BRUTAL_BORDER)
    }
}

fn brutal_input_hint_color(theme: &Theme) -> Hsla {
    theme
        .get_color("content.tertiary")
        .unwrap_or(BRUTAL_BORDER)
}

fn brutal_input_min_height(theme: &Theme) -> Pixels {
    px(
        theme
            .get_number("tokens.control.input.min_height")
            .unwrap_or(42.0) as f32,
    )
}

fn brutal_input_padding(theme: &Theme) -> Edges<Pixels> {
    let h = theme
        .get_number("tokens.control.input.horizontal_padding")
        .unwrap_or(12.0) as f32;
    let v = theme
        .get_number("tokens.control.input.vertical_padding")
        .unwrap_or(10.0) as f32;
    Edges::symmetric(px(h), px(v))
}

// =====================================================================
// TextInput
// =====================================================================

pub use yororen_ui_default_renderer::renderers::text_input::{
    TextInputRenderState, TextInputRenderer,
};

pub struct BrutalTextInputRenderer;

impl TextInputRenderer for BrutalTextInputRenderer {
    fn bg(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(state.disabled, theme)
    }
    fn border(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(state.disabled, theme)
    }
    fn focus_border(&self, _: &TextInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn hover_border(&self, _: &TextInputRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn active_border(&self, _: &TextInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn text_color(&self, state: &TextInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_text_color(state.disabled, theme)
    }
    fn hint_color(&self, _: &TextInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_hint_color(theme)
    }
    fn cursor_color(&self, _: &TextInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn selection_color(&self, _: &TextInputRenderState, theme: &Theme) -> Hsla {
        // Brutalism: high-contrast selection (50% alpha).
        let c = brutal_input_focus_border(theme);
        hsla(c.h, c.s, c.l, 0.5)
    }
    fn min_height(&self, _: &TextInputRenderState, theme: &Theme) -> Pixels {
        brutal_input_min_height(theme)
    }
    fn padding(&self, _: &TextInputRenderState, theme: &Theme) -> Edges<Pixels> {
        brutal_input_padding(theme)
    }
    fn border_radius(&self, _: &TextInputRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn disabled_opacity(&self, state: &TextInputRenderState, _: &Theme) -> f32 {
        if state.disabled { 0.6 } else { 1.0 }
    }
}

// =====================================================================
// TextArea — fewer methods (the painter reads hint/cursor/selection
// directly from the theme); brutalism maps them the same way.
// =====================================================================

pub use yororen_ui_default_renderer::renderers::text_area::{
    TextAreaRenderState, TextAreaRenderer,
};

pub struct BrutalTextAreaRenderer;

impl TextAreaRenderer for BrutalTextAreaRenderer {
    fn bg(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(state.disabled, theme)
    }
    fn border(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(state.disabled, theme)
    }
    fn focus_border(&self, _: &TextAreaRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn hover_border(&self, _: &TextAreaRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn active_border(&self, _: &TextAreaRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn text_color(&self, state: &TextAreaRenderState, theme: &Theme) -> Hsla {
        brutal_input_text_color(state.disabled, theme)
    }
    fn min_height(&self, _: &TextAreaRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.text_area.min_height")
                .unwrap_or(90.0) as f32,
        )
    }
    fn padding(&self, _: &TextAreaRenderState, theme: &Theme) -> Edges<Pixels> {
        let p = theme
            .get_number("tokens.control.text_area.padding")
            .unwrap_or(12.0) as f32;
        Edges::all(px(p))
    }
    fn border_radius(&self, _: &TextAreaRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

// =====================================================================
// PasswordInput
// =====================================================================

pub use yororen_ui_default_renderer::renderers::password_input::{
    PasswordInputRenderState, PasswordInputRenderer,
};

pub struct BrutalPasswordInputRenderer;

impl PasswordInputRenderer for BrutalPasswordInputRenderer {
    fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(state.disabled, theme)
    }
    fn border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(state.disabled, theme)
    }
    fn focus_border(&self, _: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn hover_border(&self, _: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn active_border(&self, _: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_text_color(state.disabled, theme)
    }
    fn min_height(&self, _: &PasswordInputRenderState, theme: &Theme) -> Pixels {
        brutal_input_min_height(theme)
    }
    fn padding(&self, _: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels> {
        brutal_input_padding(theme)
    }
    fn border_radius(&self, _: &PasswordInputRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

// =====================================================================
// NumberInput
// =====================================================================

pub use yororen_ui_default_renderer::renderers::number_input::{
    NumberInputRenderState, NumberInputRenderer,
};

pub struct BrutalNumberInputRenderer;

impl NumberInputRenderer for BrutalNumberInputRenderer {
    fn bg(&self, _: &NumberInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(false, theme)
    }
    fn border(&self, _: &NumberInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(false, theme)
    }
    fn focus_border(&self, _: &NumberInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn hover_border(&self, _: &NumberInputRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn active_border(&self, _: &NumberInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn min_height(&self, _: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.number_input.min_height")
                .unwrap_or(42.0) as f32,
        )
    }
    fn padding(&self, _: &NumberInputRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.number_input.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme
            .get_number("tokens.control.input.vertical_padding")
            .unwrap_or(10.0) as f32;
        Edges::symmetric(px(h), px(v))
    }
    fn stepper_button_size(&self, _: &NumberInputRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.number_input.stepper_button_size")
                .unwrap_or(32.0) as f32,
        )
    }
    fn border_radius(&self, _: &NumberInputRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

// =====================================================================
// FilePathInput
// =====================================================================

pub use yororen_ui_default_renderer::renderers::file_path_input::{
    FilePathInputRenderState, FilePathInputRenderer,
};

pub struct BrutalFilePathInputRenderer;

impl FilePathInputRenderer for BrutalFilePathInputRenderer {
    fn bg(&self, _: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(false, theme)
    }
    fn border(&self, _: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(false, theme)
    }
    fn focus_border(&self, _: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn hover_border(&self, _: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn active_border(&self, _: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn button_bg(&self, _: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn button_hover_bg(&self, _: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.hover_bg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn button_fg(&self, _: &FilePathInputRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("action.neutral.fg")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn min_height(&self, _: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.file_path_input.min_height")
                .unwrap_or(42.0) as f32,
        )
    }
    fn padding(&self, _: &FilePathInputRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.file_path_input.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme
            .get_number("tokens.control.input.vertical_padding")
            .unwrap_or(10.0) as f32;
        Edges::symmetric(px(h), px(v))
    }
    fn action_gap(&self, _: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.file_path_input.action_gap")
                .unwrap_or(8.0) as f32,
        )
    }
    fn border_radius(&self, _: &FilePathInputRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn icon_size(&self, _: &FilePathInputRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.file_path_input.icon_size")
                .unwrap_or(20.0) as f32,
        )
    }
}

// =====================================================================
// SearchInput
// =====================================================================

pub use yororen_ui_default_renderer::renderers::search_input::{
    SearchInputRenderState, SearchInputRenderer,
};

pub struct BrutalSearchInputRenderer;

impl SearchInputRenderer for BrutalSearchInputRenderer {
    fn bg(&self, _: &SearchInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(false, theme)
    }
    fn border(&self, _: &SearchInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(false, theme)
    }
    fn focus_border(&self, _: &SearchInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn hover_border(&self, _: &SearchInputRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn active_border(&self, _: &SearchInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn icon_color(&self, _: &SearchInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_hint_color(theme)
    }
    fn fg(&self, _: &SearchInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_text_color(false, theme)
    }
    fn min_height(&self, _: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.search_input.min_height")
                .unwrap_or(42.0) as f32,
        )
    }
    fn padding(&self, _: &SearchInputRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.search_input.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme
            .get_number("tokens.control.input.vertical_padding")
            .unwrap_or(10.0) as f32;
        Edges::symmetric(px(h), px(v))
    }
    fn border_radius(&self, _: &SearchInputRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn input_gap(&self, _: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.search_input.input_gap")
                .unwrap_or(8.0) as f32,
        )
    }
    fn icon_size(&self, _: &SearchInputRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.search_input.icon_size")
                .unwrap_or(20.0) as f32,
        )
    }
}

// =====================================================================
// Select
// =====================================================================

pub use yororen_ui_default_renderer::renderers::select::{SelectRenderState, SelectRenderer};

pub struct BrutalSelectRenderer;

impl SelectRenderer for BrutalSelectRenderer {
    fn bg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(state.disabled, theme)
    }
    fn border(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(state.disabled, theme)
    }
    fn focus_border(&self, _: &SelectRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn fg(&self, state: &SelectRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("content.disabled")
                .unwrap_or(BRUTAL_BORDER)
        } else if state.has_value {
            theme
                .get_color("content.primary")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("content.tertiary")
                .unwrap_or(BRUTAL_BORDER)
        }
    }
    fn hint_color(&self, _: &SelectRenderState, theme: &Theme) -> Hsla {
        brutal_input_hint_color(theme)
    }
    fn min_height(&self, _: &SelectRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.select.min_height")
                .unwrap_or(42.0) as f32,
        )
    }
    fn padding(&self, _: &SelectRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.select.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme
            .get_number("tokens.spacing.tight")
            .unwrap_or(4.0) as f32;
        Edges::symmetric(px(h), px(v))
    }
    fn border_radius(&self, _: &SelectRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
    fn chevron_rotation(&self, state: &SelectRenderState, _: &Theme) -> f32 {
        if state.open { 180.0 } else { 0.0 }
    }
}

// =====================================================================
// ComboBox
// =====================================================================

pub use yororen_ui_default_renderer::renderers::combo_box::{
    ComboBoxRenderState, ComboBoxRenderer,
};

pub struct BrutalComboBoxRenderer;

impl ComboBoxRenderer for BrutalComboBoxRenderer {
    fn bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(state.disabled, theme)
    }
    fn border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(state.disabled, theme)
    }
    fn focus_border(&self, _: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        if state.disabled {
            theme
                .get_color("content.disabled")
                .unwrap_or(BRUTAL_BORDER)
        } else if state.has_value {
            theme
                .get_color("content.primary")
                .unwrap_or(BRUTAL_BORDER)
        } else {
            theme
                .get_color("content.tertiary")
                .unwrap_or(BRUTAL_BORDER)
        }
    }
    fn search_bg(&self, _: &ComboBoxRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.base")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn min_height(&self, _: &ComboBoxRenderState, theme: &Theme) -> Pixels {
        px(
            theme
                .get_number("tokens.control.combo_box.min_height")
                .unwrap_or(42.0) as f32,
        )
    }
    fn padding(&self, _: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels> {
        let h = theme
            .get_number("tokens.control.combo_box.horizontal_padding")
            .unwrap_or(12.0) as f32;
        let v = theme
            .get_number("tokens.spacing.tight")
            .unwrap_or(4.0) as f32;
        Edges::symmetric(px(h), px(v))
    }
    fn border_radius(&self, _: &ComboBoxRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}

// =====================================================================
// KeybindingInput
// =====================================================================

pub use yororen_ui_default_renderer::renderers::keybinding_input::{
    KeybindingInputRenderState, KeybindingInputRenderer,
};

pub struct BrutalKeybindingInputRenderer;

impl KeybindingInputRenderer for BrutalKeybindingInputRenderer {
    fn bg(&self, _: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_bg(false, theme)
    }
    fn border(&self, _: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_border(false, theme)
    }
    fn focus_border(&self, _: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn hover_border(&self, _: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        brutal_border_color(theme)
    }
    fn active_border(&self, _: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        brutal_input_focus_border(theme)
    }
    fn kbd_bg(&self, _: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("surface.hover")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn kbd_fg(&self, _: &KeybindingInputRenderState, theme: &Theme) -> Hsla {
        theme
            .get_color("content.primary")
            .unwrap_or(BRUTAL_BORDER)
    }
    fn min_height(&self, _: &KeybindingInputRenderState, theme: &Theme) -> Pixels {
        brutal_input_min_height(theme)
    }
    fn border_radius(&self, _: &KeybindingInputRenderState, _: &Theme) -> Pixels {
        px(BRUTAL_RADIUS)
    }
}
