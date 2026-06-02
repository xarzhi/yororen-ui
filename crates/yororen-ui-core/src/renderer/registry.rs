//! `RendererRegistry` — the collection of component renderers wired into
//! a `Theme`. Phase B ships the button entry; Phase C adds the remaining
//! 30+ components one trait per file.

use std::sync::Arc;

use super::avatar::{AvatarRenderer, TokenAvatarRenderer};
use super::badge::{BadgeRenderer, TokenBadgeRenderer};
use super::button::{ButtonRenderer, TokenButtonRenderer};
use super::divider::{DividerRenderer, TokenDividerRenderer};
use super::focus_ring::{FocusRingRenderer, TokenFocusRingRenderer};
use super::heading::{HeadingRenderer, TokenHeadingRenderer};
use super::label::{LabelRenderer, TokenLabelRenderer};
use super::progress::{ProgressBarRenderer, TokenProgressBarRenderer};
use super::skeleton::{SkeletonRenderer, TokenSkeletonRenderer};
use super::tag::{TagRenderer, TokenTagRenderer};
use super::tooltip::{TooltipRenderer, TokenTooltipRenderer};

#[derive(Clone)]
pub struct RendererRegistry {
    pub button: Arc<dyn ButtonRenderer>,
    pub label: Arc<dyn LabelRenderer>,
    pub heading: Arc<dyn HeadingRenderer>,
    pub divider: Arc<dyn DividerRenderer>,
    pub focus_ring: Arc<dyn FocusRingRenderer>,
    pub badge: Arc<dyn BadgeRenderer>,
    pub tag: Arc<dyn TagRenderer>,
    pub progress_bar: Arc<dyn ProgressBarRenderer>,
    pub skeleton: Arc<dyn SkeletonRenderer>,
    pub tooltip: Arc<dyn TooltipRenderer>,
    pub avatar: Arc<dyn AvatarRenderer>,
}

impl std::fmt::Debug for RendererRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererRegistry").finish_non_exhaustive()
    }
}

impl Default for RendererRegistry {
    fn default() -> Self {
        Self::token_based()
    }
}

impl RendererRegistry {
    /// All renderers set to the default `TokenXxxRenderer` implementations.
    /// This is the v0.3 / v0.4 visual baseline.
    pub fn token_based() -> Self {
        Self {
            button: Arc::new(TokenButtonRenderer),
            label: Arc::new(TokenLabelRenderer),
            heading: Arc::new(TokenHeadingRenderer),
            divider: Arc::new(TokenDividerRenderer),
            focus_ring: Arc::new(TokenFocusRingRenderer),
            badge: Arc::new(TokenBadgeRenderer),
            tag: Arc::new(TokenTagRenderer),
            progress_bar: Arc::new(TokenProgressBarRenderer),
            skeleton: Arc::new(TokenSkeletonRenderer),
            tooltip: Arc::new(TokenTooltipRenderer),
            avatar: Arc::new(TokenAvatarRenderer),
        }
    }

    pub fn with_button(mut self, r: Arc<dyn ButtonRenderer>) -> Self {
        self.button = r;
        self
    }
    pub fn with_label(mut self, r: Arc<dyn LabelRenderer>) -> Self {
        self.label = r;
        self
    }
    pub fn with_heading(mut self, r: Arc<dyn HeadingRenderer>) -> Self {
        self.heading = r;
        self
    }
    pub fn with_divider(mut self, r: Arc<dyn DividerRenderer>) -> Self {
        self.divider = r;
        self
    }
    pub fn with_focus_ring(mut self, r: Arc<dyn FocusRingRenderer>) -> Self {
        self.focus_ring = r;
        self
    }
    pub fn with_badge(mut self, r: Arc<dyn BadgeRenderer>) -> Self {
        self.badge = r;
        self
    }
    pub fn with_tag(mut self, r: Arc<dyn TagRenderer>) -> Self {
        self.tag = r;
        self
    }
    pub fn with_progress_bar(mut self, r: Arc<dyn ProgressBarRenderer>) -> Self {
        self.progress_bar = r;
        self
    }
    pub fn with_skeleton(mut self, r: Arc<dyn SkeletonRenderer>) -> Self {
        self.skeleton = r;
        self
    }
    pub fn with_tooltip(mut self, r: Arc<dyn TooltipRenderer>) -> Self {
        self.tooltip = r;
        self
    }
    pub fn with_avatar(mut self, r: Arc<dyn AvatarRenderer>) -> Self {
        self.avatar = r;
        self
    }
}
