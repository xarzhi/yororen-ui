//! `FilePathInputRenderer` — visual side of `FilePathInput`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct FilePathInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait FilePathInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn button_bg(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn button_hover_bg(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn button_fg(&self, state: &FilePathInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &FilePathInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &FilePathInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn action_gap(&self, state: &FilePathInputRenderState, theme: &Theme) -> Pixels;
    fn border_radius(&self, state: &FilePathInputRenderState, theme: &Theme) -> Pixels;
    fn icon_size(&self, state: &FilePathInputRenderState, theme: &Theme) -> Pixels;
}
