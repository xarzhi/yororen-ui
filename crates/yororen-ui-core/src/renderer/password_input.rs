//! `PasswordInputRenderer` — visual side of `PasswordInput`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct PasswordInputRenderState {
    pub disabled: bool,
    pub focused: bool,
    pub has_custom_bg: bool,
    pub has_custom_border: bool,
    pub has_custom_focus_border: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait PasswordInputRenderer: Any + Send + Sync {
    fn bg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn hover_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn active_border(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &PasswordInputRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &PasswordInputRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &PasswordInputRenderState, theme: &Theme) -> Pixels;
}
