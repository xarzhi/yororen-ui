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
    use crate::theme::{
        ActionTheme, ActionVariant, BorderTheme, ContentTheme, DesignTokens, RendererRegistry,
        ShadowTheme, StatusTheme, StatusVariant, SurfaceTheme, Theme,
    };
    use crate::i18n::TextDirection;
    use gpui::{hsla, rgb};

    /// Internal test fixtures. `core` is headless: the production default
    /// themes live in `yororen_ui_theme_system`. We re-create the
    /// equivalent palettes here so contrast / token-range assertions still
    /// have something to chew on.
    fn fixture_themes() -> [(&'static str, Theme); 2] {
        [
            ("dark", fixture_dark()),
            ("light", fixture_light()),
        ]
    }

    fn fixture_dark() -> Theme {
        let content = ContentTheme {
            primary: rgb(0xF2F2F3).into(),
            secondary: rgb(0xC8C8CC).into(),
            tertiary: rgb(0x9B9BA1).into(),
            disabled: rgb(0x6F6F76).into(),
            on_primary: rgb(0x0B0B0D).into(),
            on_status: rgb(0x0B0B0D).into(),
        };
        Theme {
            surface: SurfaceTheme {
                canvas: rgb(0x0F0F11).into(),
                base: rgb(0x151518).into(),
                raised: rgb(0x1D1D21).into(),
                sunken: rgb(0x111113).into(),
                hover: rgb(0x232327).into(),
            },
            content: content.clone(),
            border: BorderTheme {
                default: rgb(0x2A2A2F).into(),
                muted: rgb(0x1E1E22).into(),
                focus: rgb(0x8BB0FF).into(),
                divider: rgb(0x1E1E22).into(),
            },
            action: ActionTheme {
                neutral: ActionVariant {
                    bg: rgb(0x1D1D21).into(),
                    hover_bg: rgb(0x24242A).into(),
                    active_bg: rgb(0x2A2A31).into(),
                    fg: content.primary,
                    disabled_bg: rgb(0x1A1A1D).into(),
                    disabled_fg: content.disabled,
                },
                primary: ActionVariant {
                    bg: rgb(0xF4F4F6).into(),
                    hover_bg: rgb(0xFFFFFF).into(),
                    active_bg: rgb(0xE9E9EC).into(),
                    fg: content.on_primary,
                    disabled_bg: rgb(0xE0E0E4).into(),
                    disabled_fg: rgb(0x5B5B61).into(),
                },
                danger: ActionVariant {
                    bg: rgb(0xFFB4AE).into(),
                    hover_bg: rgb(0xFFA099).into(),
                    active_bg: rgb(0xFF8A82).into(),
                    fg: content.on_status,
                    disabled_bg: rgb(0xE0B3AF).into(),
                    disabled_fg: rgb(0x5B5B61).into(),
                },
            },
            status: StatusTheme {
                success: StatusVariant {
                    bg: rgb(0xB9F5C9).into(),
                    fg: content.on_status,
                },
                warning: StatusVariant {
                    bg: rgb(0xFFE1A6).into(),
                    fg: content.on_status,
                },
                error: StatusVariant {
                    bg: rgb(0xFFB4AE).into(),
                    fg: content.on_status,
                },
                info: StatusVariant {
                    bg: rgb(0xB6D9FF).into(),
                    fg: content.on_status,
                },
            },
            shadow: ShadowTheme {
                elevation_1: hsla(0.0, 0.0, 0.0, 0.3),
                elevation_2: hsla(0.0, 0.0, 0.0, 0.45),
            },
            text_direction: TextDirection::Ltr,
            tokens: DesignTokens::default(),
            renderers: RendererRegistry::token_based(),
        }
    }

    fn fixture_light() -> Theme {
        let content = ContentTheme {
            primary: rgb(0x141416).into(),
            secondary: rgb(0x3E3E45).into(),
            tertiary: rgb(0x6B6B73).into(),
            disabled: rgb(0x9A9AA2).into(),
            on_primary: rgb(0xFFFFFF).into(),
            on_status: rgb(0x0B0B0D).into(),
        };
        Theme {
            surface: SurfaceTheme {
                canvas: rgb(0xF4F4F6).into(),
                base: rgb(0xFFFFFF).into(),
                raised: rgb(0xFBFBFD).into(),
                sunken: rgb(0xEFEFF2).into(),
                hover: rgb(0xE6E6EA).into(),
            },
            content: content.clone(),
            border: BorderTheme {
                default: rgb(0xD8D8DD).into(),
                muted: rgb(0xE3E3E8).into(),
                focus: rgb(0x2F63FF).into(),
                divider: rgb(0xE3E3E8).into(),
            },
            action: ActionTheme {
                neutral: ActionVariant {
                    bg: rgb(0xF1F1F3).into(),
                    hover_bg: rgb(0xE6E6EA).into(),
                    active_bg: rgb(0xDADADF).into(),
                    fg: content.primary,
                    disabled_bg: rgb(0xE7E7EA).into(),
                    disabled_fg: content.disabled,
                },
                primary: ActionVariant {
                    bg: rgb(0x121214).into(),
                    hover_bg: rgb(0x0C0C0D).into(),
                    active_bg: rgb(0x000000).into(),
                    fg: content.on_primary,
                    disabled_bg: rgb(0x2A2A2E).into(),
                    disabled_fg: rgb(0xD0D0D6).into(),
                },
                danger: ActionVariant {
                    bg: rgb(0xFFB4AE).into(),
                    hover_bg: rgb(0xFFA099).into(),
                    active_bg: rgb(0xFF8A82).into(),
                    fg: content.on_status,
                    disabled_bg: rgb(0xF0CBC7).into(),
                    disabled_fg: content.disabled,
                },
            },
            status: StatusTheme {
                success: StatusVariant {
                    bg: rgb(0xB9F5C9).into(),
                    fg: content.on_status,
                },
                warning: StatusVariant {
                    bg: rgb(0xFFE1A6).into(),
                    fg: content.on_status,
                },
                error: StatusVariant {
                    bg: rgb(0xFFB4AE).into(),
                    fg: content.on_status,
                },
                info: StatusVariant {
                    bg: rgb(0xB6D9FF).into(),
                    fg: content.on_status,
                },
            },
            shadow: ShadowTheme {
                elevation_1: hsla(0.0, 0.0, 0.0, 0.18),
                elevation_2: hsla(0.0, 0.0, 0.0, 0.3),
            },
            text_direction: TextDirection::Ltr,
            tokens: DesignTokens::default(),
            renderers: RendererRegistry::token_based(),
        }
    }

    #[test]
    fn fixture_themes_have_no_issues() {
        for (name, theme) in fixture_themes() {
            let issues = validate(&theme);
            assert!(
                issues.is_empty(),
                "{name} theme has validation issues: {:#?}",
                issues
            );
        }
    }
}
