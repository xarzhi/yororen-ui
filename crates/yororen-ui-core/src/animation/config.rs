//! Animation configuration module.

use std::time::Duration;

use super::easing::EasingFn;
use super::easing::ease_out_quad;

/// Configuration for animations.
///
/// # What this carries
///
/// - `duration`: how long the animation runs.
/// - `easing`: the easing curve.
/// - `repeat`: whether the animation loops.
///
/// # What this **does not** carry
///
/// `delay` and `reverse` (yoyo) are **not** modeled here. The
/// underlying `gpui::Animation` does not currently expose them, and
/// silently dropping them at `to_gpui_animation` would be a footgun.
///
/// For delayed / yoyo animations, drive them explicitly with
/// [`super::orchestrator`]. That module is the public API for
/// animation choreography that exceeds what `gpui::Animation`
/// itself supports.
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Duration of the animation.
    pub duration: Duration,
    /// Easing function to use.
    pub easing: EasingFn,
    /// Whether the animation should repeat.
    pub repeat: bool,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            duration: Duration::from_millis(200),
            easing: ease_out_quad,
            repeat: false,
        }
    }
}

impl AnimationConfig {
    /// Create a new animation config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the duration.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Set the easing function.
    pub fn with_easing(mut self, easing: EasingFn) -> Self {
        self.easing = easing;
        self
    }

    /// Enable repeat.
    pub fn with_repeat(mut self) -> Self {
        self.repeat = true;
        self
    }

    /// Convert to a `gpui::Animation` honoring the configured
    /// `duration`, `easing` and `repeat`. Note that `delay` and
    /// `reverse` (yoyo) are intentionally **not** part of this
    /// config — see the struct-level docs and
    /// [`super::orchestrator`] for the API surface that does
    /// support them.
    pub fn to_gpui_animation(self) -> gpui::Animation {
        let mut animation = gpui::Animation::new(self.duration).with_easing(self.easing);

        if self.repeat {
            animation = animation.repeat();
        }

        animation
    }

    /// Build an `AnimationConfig` from a theme's `MotionTokens`.
    /// This is the unified entry point — duration comes from
    /// `tokens.motion.duration_normal` and easing from
    /// `tokens.motion.easing_standard`. Components should call this
    /// instead of independently reading
    /// `animation::constants::duration::*` and `easing::*`.
    pub fn from_motion(motion: &crate::theme::tokens::MotionTokens) -> Self {
        Self {
            duration: motion.duration_normal,
            easing: motion.easing_standard,
            repeat: false,
        }
    }
}

/// State tracking for complex animations.
#[derive(Debug, Clone, Default)]
pub struct AnimationState {
    /// Current progress (0.0 to 1.0).
    pub progress: f32,
    /// Whether the animation is currently running.
    pub is_running: bool,
    /// Whether the animation is paused.
    pub is_paused: bool,
}

impl AnimationState {
    /// Create a new animation state.
    pub fn new() -> Self {
        Self {
            progress: 0.0,
            is_running: false,
            is_paused: false,
        }
    }

    /// Reset the state.
    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.is_running = false;
        self.is_paused = false;
    }

    /// Update progress value.
    pub fn update(&mut self, progress: f32) {
        self.progress = progress.clamp(0.0, 1.0);
    }
}
