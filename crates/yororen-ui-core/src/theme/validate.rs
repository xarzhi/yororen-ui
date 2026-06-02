//! Theme validation utilities.
//!
//! Run `validate(&theme)` to produce a list of issues for a given theme.
//! Use this in CI for third-party theme packages to catch common mistakes
//! (insufficient contrast, out-of-range sizes, etc.) before shipping.

use gpui::{Hsla, Rgba};

use super::Theme;

/// A single issue found while validating a theme.
#[derive(Clone, Debug)]
pub struct Issue {
    pub kind: IssueKind,
    pub message: String,
}

#[derive(Clone, Debug)]
pub enum IssueKind {
    /// Foreground/background pair has contrast ratio below the recommended minimum.
    ContrastTooLow {
        fg: Hsla,
        bg: Hsla,
        ratio: f32,
        min: f32,
    },
    /// A token field is in an unexpected range (negative, NaN, absurdly large).
    TokenOutOfRange {
        token_path: &'static str,
        value: f32,
    },
    /// Status text on its background is unreadable.
    StatusTextInvisible,
    /// A control's knob/track geometric ratio is suspicious.
    MalformedControlGeometry {
        control: &'static str,
        detail: String,
    },
}

/// Validate a theme and return a list of issues.
///
/// The core default themes are guaranteed to produce zero issues.
pub fn validate(theme: &Theme) -> Vec<Issue> {
    let mut issues = Vec::new();

    // --- contrast checks ---
    let pairs: [(&str, Hsla, Hsla, f32); 8] = [
        (
            "surface.base/content.primary",
            theme.surface.base,
            theme.content.primary,
            4.5,
        ),
        (
            "action.neutral",
            theme.action.neutral.bg,
            theme.action.neutral.fg,
            4.5,
        ),
        (
            "action.primary",
            theme.action.primary.bg,
            theme.action.primary.fg,
            4.5,
        ),
        (
            "action.danger",
            theme.action.danger.bg,
            theme.action.danger.fg,
            4.5,
        ),
        (
            "status.success",
            theme.status.success.bg,
            theme.status.success.fg,
            4.5,
        ),
        (
            "status.warning",
            theme.status.warning.bg,
            theme.status.warning.fg,
            4.5,
        ),
        (
            "status.error",
            theme.status.error.bg,
            theme.status.error.fg,
            4.5,
        ),
        (
            "status.info",
            theme.status.info.bg,
            theme.status.info.fg,
            4.5,
        ),
    ];
    for (label, bg, fg, min) in pairs {
        let ratio = contrast_ratio(fg, bg);
        if ratio < min {
            issues.push(Issue {
                kind: IssueKind::ContrastTooLow { fg, bg, ratio, min },
                message: format!(
                    "{label}: contrast ratio {ratio:.2} is below recommended minimum {min:.2}"
                ),
            });
        }
    }

    let focus_ratio = contrast_ratio(theme.border.focus, theme.surface.base);
    if focus_ratio < 3.0 {
        issues.push(Issue {
            kind: IssueKind::ContrastTooLow {
                fg: theme.border.focus,
                bg: theme.surface.base,
                ratio: focus_ratio,
                min: 3.0,
            },
            message: format!(
                "border.focus on surface.base: contrast ratio {focus_ratio:.2} is below 3.0"
            ),
        });
    }

    // --- control geometry sanity ---
    let knob: f32 = theme.tokens.control.switch.knob_size.into();
    let track: f32 = theme.tokens.control.switch.track_h.into();
    let pad: f32 = theme.tokens.control.switch.padding.into();
    if knob > track - pad * 2.0 {
        issues.push(Issue {
            kind: IssueKind::MalformedControlGeometry {
                control: "switch",
                detail: "knob_size is larger than track_h minus padding*2".into(),
            },
            message: "switch.knob_size must fit within track_h - padding*2".into(),
        });
    }

    // --- token range ---
    if !theme.tokens.motion.pulse_min_opacity.is_finite()
        || theme.tokens.motion.pulse_min_opacity < 0.0
        || theme.tokens.motion.pulse_min_opacity > 1.0
    {
        issues.push(Issue {
            kind: IssueKind::TokenOutOfRange {
                token_path: "motion.pulse_min_opacity",
                value: theme.tokens.motion.pulse_min_opacity,
            },
            message: format!(
                "motion.pulse_min_opacity must be in [0.0, 1.0], got {}",
                theme.tokens.motion.pulse_min_opacity
            ),
        });
    }
    if !theme.tokens.motion.pulse_max_opacity.is_finite()
        || theme.tokens.motion.pulse_max_opacity < 0.0
        || theme.tokens.motion.pulse_max_opacity > 1.0
    {
        issues.push(Issue {
            kind: IssueKind::TokenOutOfRange {
                token_path: "motion.pulse_max_opacity",
                value: theme.tokens.motion.pulse_max_opacity,
            },
            message: format!(
                "motion.pulse_max_opacity must be in [0.0, 1.0], got {}",
                theme.tokens.motion.pulse_max_opacity
            ),
        });
    }

    issues
}

fn relative_luminance(color: Hsla) -> f32 {
    let rgb = Rgba::from(color);
    let linear = |c: f32| {
        if c <= 0.03928 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    };
    let r = linear(rgb.r);
    let g = linear(rgb.g);
    let b = linear(rgb.b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn contrast_ratio(a: Hsla, b: Hsla) -> f32 {
    let l1 = relative_luminance(a);
    let l2 = relative_luminance(b);
    let (lighter, darker) = if l1 >= l2 { (l1, l2) } else { (l2, l1) };
    (lighter + 0.05) / (darker + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    #[test]
    fn default_themes_have_no_issues() {
        for (name, theme) in [
            ("dark", Theme::default_dark()),
            ("light", Theme::default_light()),
        ] {
            let issues = validate(&theme);
            assert!(
                issues.is_empty(),
                "{name} theme has validation issues: {:#?}",
                issues
            );
        }
    }
}
