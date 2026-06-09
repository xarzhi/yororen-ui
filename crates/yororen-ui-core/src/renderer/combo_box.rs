//! `ComboBoxRenderer` — visual side of `ComboBox`.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::Edges;
use crate::theme::Theme;

#[derive(Clone, Copy, Debug, Default)]
pub struct ComboBoxRenderState {
    pub open: bool,
    pub disabled: bool,
    pub has_value: bool,
    pub custom_bg: Option<Hsla>,
    pub custom_border: Option<Hsla>,
    pub custom_focus_border: Option<Hsla>,
    pub custom_fg: Option<Hsla>,
}

pub trait ComboBoxRenderer: Any + Send + Sync {
    fn bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn focus_border(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn search_bg(&self, state: &ComboBoxRenderState, theme: &Theme) -> Hsla;
    fn min_height(&self, state: &ComboBoxRenderState, theme: &Theme) -> Pixels;
    fn padding(&self, state: &ComboBoxRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &ComboBoxRenderState, theme: &Theme) -> Pixels;
}
