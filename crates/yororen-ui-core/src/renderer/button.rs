//! `ButtonRenderer` trait — the data contract that every
//! third-party renderer (default, brutalism, …) implements to
//! describe a `Button`'s visual.
//!
//! Lives in core (not the default-renderer) so headless
//! `ButtonProps::render` can call these methods. The actual
//! rendering logic that builds the `Stateful<Div>` lives in
//! headless's `ButtonProps::render`; the renderer's job is
//! only to provide the *values* the headless consumes.
//!
//! One trait, one component. The other 37 component renderer
//! traits follow the same shape.

use std::any::Any;

use gpui::{Hsla, Pixels};

use crate::renderer::spec::{BorderSpec, Edges, ShadowSpec};
use crate::renderer::variant::VariantStyle;
use crate::theme::Theme;

/// State passed to a `ButtonRenderer`. Fields are deliberately
/// minimal — a renderer can read more from the `Theme` if it
/// needs to.
#[derive(Clone, Debug, Default)]
pub struct ButtonRenderState {
    pub variant: ActionVariantKind,
    pub disabled: bool,
    pub is_rtl: bool,
    /// `true` if the user supplied `.bg(...)` on the builder.
    pub has_custom_bg: bool,
    /// `true` if the user supplied `.hover_bg(...)` on the
    /// builder.
    pub has_custom_hover_bg: bool,
    /// Pre-resolved custom variant from the global
    /// `VariantRegistry`. When `Some`, the renderer should
    /// delegate color decisions (bg/fg/border/disabled_opacity)
    /// to the contained `VariantStyle` instead of reading
    /// `theme.get_color("action.<v>.<field>")`. When `None`,
    /// the renderer falls back to the built-in token path.
    pub custom_style: Option<Arc<dyn VariantStyle>>,
}

pub use crate::renderer::variant::ActionVariantKind;
use std::sync::Arc;

/// Renderer for the `Button` component. Implementations decide
/// what the button looks like in every state.
///
/// Default: `TokenButtonRenderer` (in the default-renderer
/// crate). Theme packages / renderer crates override this by
/// registering their own `ButtonRenderer` impl via
/// `cx.register_renderer_arc::<ButtonMarker, dyn ButtonRenderer>(…)`.
pub trait ButtonRenderer: Any + Send + Sync {
    fn bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla;
    fn fg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla;
    fn padding(&self, state: &ButtonRenderState, theme: &Theme) -> Edges<Pixels>;
    fn border_radius(&self, state: &ButtonRenderState, theme: &Theme) -> Pixels;
    fn border(&self, state: &ButtonRenderState, theme: &Theme) -> Option<BorderSpec>;
    fn shadow(&self, state: &ButtonRenderState, theme: &Theme) -> Option<ShadowSpec>;
    fn min_height(&self, state: &ButtonRenderState, theme: &Theme) -> Pixels;
    fn disabled_opacity(&self, state: &ButtonRenderState, theme: &Theme) -> f32;
    /// Background colour while the mouse is hovering. Used by
    /// the headless `render()` for `.hover(|s| s.bg(…))`.
    fn hover_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla;
    /// Background colour while the button is being pressed.
    /// Used for `.active(|s| …)`.
    fn active_bg(&self, state: &ButtonRenderState, theme: &Theme) -> Hsla;
}
