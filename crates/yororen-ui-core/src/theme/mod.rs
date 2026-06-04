use std::sync::Arc;

use gpui::{App, Global, Hsla};

use crate::i18n::TextDirection;

pub mod tokens;
pub mod validate;

pub use crate::renderer::RendererRegistry;
pub use tokens::{
    DesignTokens, EasingFn, MotionTokens, RadiiTokens, SizeTokens, SpacingTokens, TypographyTokens,
};
pub use validate::{Issue, IssueKind, validate};

#[derive(Clone, Debug)]
pub struct Theme {
    pub surface: SurfaceTheme,
    pub content: ContentTheme,
    pub border: BorderTheme,
    pub action: ActionTheme,
    pub status: StatusTheme,
    pub shadow: ShadowTheme,
    /// Text direction (LTR or RTL)
    pub text_direction: TextDirection,
    /// Design tokens — single source of truth for component geometry, typography,
    /// spacing, radii, and motion. Themes override these to reshape the UI
    /// without touching component logic.
    pub tokens: DesignTokens,
    /// Per-component renderers. Phase B spike: only `button`. Phase C
    /// generalizes this to 30+ components.
    pub renderers: RendererRegistry,
}

#[derive(Clone, Debug, Default)]
pub struct SurfaceTheme {
    pub canvas: Hsla,
    pub base: Hsla,
    pub raised: Hsla,
    pub sunken: Hsla,
    pub hover: Hsla,
}

#[derive(Clone, Debug, Default)]
pub struct ContentTheme {
    pub primary: Hsla,
    pub secondary: Hsla,
    pub tertiary: Hsla,
    pub disabled: Hsla,
    pub on_primary: Hsla,
    pub on_status: Hsla,
}

#[derive(Clone, Debug, Default)]
pub struct BorderTheme {
    pub default: Hsla,
    pub muted: Hsla,
    pub focus: Hsla,
    pub divider: Hsla,
}

#[derive(Clone, Debug, Default)]
pub struct ActionTheme {
    pub neutral: ActionVariant,
    pub primary: ActionVariant,
    pub danger: ActionVariant,
}

#[derive(Clone, Debug, Default)]
pub struct ActionVariant {
    pub bg: Hsla,
    pub hover_bg: Hsla,
    pub active_bg: Hsla,
    pub fg: Hsla,
    pub disabled_bg: Hsla,
    pub disabled_fg: Hsla,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
pub enum ActionVariantKind {
    #[default]
    Neutral,
    Primary,
    Danger,
}

impl ActionVariantKind {
    /// Canonical lowercase string used in diagnostics and as the
    /// legacy `VariantRegistry` builtin key.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Primary => "primary",
            Self::Danger => "danger",
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct StatusTheme {
    pub success: StatusVariant,
    pub warning: StatusVariant,
    pub error: StatusVariant,
    pub info: StatusVariant,
}

#[derive(Clone, Debug, Default)]
pub struct StatusVariant {
    pub bg: Hsla,
    pub fg: Hsla,
}

#[derive(Clone, Debug, Default)]
pub struct ShadowTheme {
    pub elevation_1: Hsla,
    pub elevation_2: Hsla,
}

impl Theme {
    /// Check if RTL mode is enabled.
    pub fn is_rtl(&self) -> bool {
        self.text_direction.is_rtl()
    }

    /// Get the default text direction.
    pub fn default_text_direction() -> TextDirection {
        TextDirection::Ltr
    }

    pub fn action_variant(&self, variant: ActionVariantKind) -> &ActionVariant {
        match variant {
            ActionVariantKind::Neutral => &self.action.neutral,
            ActionVariantKind::Primary => &self.action.primary,
            ActionVariantKind::Danger => &self.action.danger,
        }
    }
}

// Compile-time proof that `Theme` is `Send + Sync`.
//
// `Theme` is stored inside `GlobalTheme` and shared across gpui worker
// threads. The unsoundness risk is that a future field could introduce
// interior mutability that is *not* `Send + Sync` (e.g. `RefCell<…>`)
// and break that assumption silently. This assertion makes any such
// regression a hard compile error.
//
// Manually verified (2026-06-04): all fields are `Send + Sync`:
//   - palette fields: `Hsla` (`Copy`, trivially `Send + Sync`)
//   - `text_direction: TextDirection` (`enum`)
//   - `tokens: DesignTokens` — all leaf fields are `Pixels` / `Duration`
//   - `renderers: RendererRegistry` — 40+ `Arc<dyn …Renderer>` where
//     every `*Renderer` trait is declared `: Send + Sync`.
//
// If you add a new field, re-verify and update this comment.
const _: fn() = || {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Theme>();
    assert_send_sync::<GlobalTheme>();
};

pub struct GlobalTheme {
    theme: Arc<Theme>,
}

impl Global for GlobalTheme {}

impl GlobalTheme {
    /// Install `theme` as the single process-global theme.
    ///
    /// `core` is headless: it does not ship a default palette. Use a
    /// theme package (e.g. `yororen_ui_theme_system::install`) to
    /// obtain a `Theme` and pass it here.
    ///
    /// As of the headless-core cutover, this is the only identity.
    /// The previous `ThemeSet` (light/dark factory) and
    /// `new_with_themes(appearance, …)` were removed because three
    /// parallel identity systems caused boundary confusion. The
    /// model now is: the app picks the right `Theme` for the OS
    /// appearance, then sets it once.
    pub fn new(theme: impl Into<Arc<Theme>>) -> Self {
        Self {
            theme: theme.into(),
        }
    }

    fn theme(cx: &App) -> &Arc<Theme> {
        &cx.global::<Self>().theme
    }

    /// Read-only accessor for the active `Arc<Theme>`. Useful when an
    /// app needs to clone / mutate the theme (e.g. to flip
    /// `text_direction` for an RTL locale) without going through
    /// `cx.global::<GlobalTheme>()`.
    pub fn current(&self) -> &Arc<Theme> {
        &self.theme
    }

    /// Consume the wrapper and return the underlying `Arc<Theme>`.
    /// Useful when re-wrapping the same theme with different
    /// `WindowAppearance` selection.
    pub fn into_arc(self) -> Arc<Theme> {
        self.theme
    }
}

pub trait ActiveTheme {
    fn theme(&self) -> &Arc<Theme>;
}

#[derive(Clone, Copy)]
pub struct InteractiveColors {
    pub bg: Hsla,
    pub hover_bg: Hsla,
    pub active_bg: Hsla,
    pub fg: Hsla,
    pub disabled_bg: Hsla,
    pub disabled_fg: Hsla,
}

pub fn interactive_colors(theme: &Theme) -> InteractiveColors {
    let neutral = &theme.action.neutral;
    InteractiveColors {
        bg: neutral.bg,
        hover_bg: neutral.hover_bg,
        active_bg: neutral.active_bg,
        fg: neutral.fg,
        disabled_bg: neutral.disabled_bg,
        disabled_fg: neutral.disabled_fg,
    }
}

impl ActiveTheme for App {
    fn theme(&self) -> &Arc<Theme> {
        GlobalTheme::theme(self)
    }
}

// Performance note:
//
// `cx.theme()` is hot in every render. It returns `&Arc<Theme>`,
// not `&Theme`, so:
//   - `let theme = cx.theme();` is a single reference copy (no
//     atomic increment).
//   - `let theme = cx.theme().clone();` is one `Arc::clone` (one
//     atomic increment). It does NOT recursively clone
//     `Theme.renderers` — `Arc::clone` only touches the outer
//     `Arc<Theme>`'s refcount.
//
// The earlier concern that "1000 nodes = 40k Arc clones" was based
// on the misreading that `Theme::clone` was deep. It is not:
// `Theme` is wrapped in `Arc<Theme>` inside `GlobalTheme`, and
// that is what `cx.theme()` returns. Renderer methods that take
// `&Theme` are reached by a single deref.
//
// Where to watch out: `cx.global::<GlobalTheme>().current()` (or
// any code path that explicitly clones the inner `Theme`) will
// invoke `Theme::clone`, which is `#[derive(Clone)]` and DOES
// recursively clone every field — including the 40+ `Arc<dyn …>`
// in `RendererRegistry`. So:
//   - Prefer `cx.theme()` (or `cx.theme().clone()`) for read-only
//     access.
//   - Never call `cx.global::<GlobalTheme>().current().as_ref().clone()`
//     inside a render loop; that allocates 40+ Arc::clone per
//     frame.

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Rgba, hsla, rgb};

    /// Internal test fixtures for theme-contrast tests. `core` is headless
    /// and does not export a default palette; these fixtures are used only
    /// to exercise the contrast helper. The real, exposed default themes
    /// live in `yororen_ui_theme_system`.
    fn fixture_themes() -> [(&'static str, Theme); 2] {
        [("dark", fixture_dark()), ("light", fixture_light())]
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

    fn assert_contrast_at_least(label: &str, a: Hsla, b: Hsla, min: f32) {
        let ratio = contrast_ratio(a, b);
        assert!(ratio >= min, "{label} contrast {ratio:.2} below {min:.2}");
    }

    #[test]
    fn theme_contrast_requirements() {
        for (name, theme) in fixture_themes() {
            assert_contrast_at_least(
                &format!("{name}: surface.base/content.primary"),
                theme.surface.base,
                theme.content.primary,
                4.5,
            );
            assert_contrast_at_least(
                &format!("{name}: action.neutral"),
                theme.action.neutral.bg,
                theme.action.neutral.fg,
                4.5,
            );
            assert_contrast_at_least(
                &format!("{name}: action.primary"),
                theme.action.primary.bg,
                theme.action.primary.fg,
                4.5,
            );
            assert_contrast_at_least(
                &format!("{name}: status.success"),
                theme.status.success.bg,
                theme.status.success.fg,
                4.5,
            );
            assert_contrast_at_least(
                &format!("{name}: status.warning"),
                theme.status.warning.bg,
                theme.status.warning.fg,
                4.5,
            );
            assert_contrast_at_least(
                &format!("{name}: status.error"),
                theme.status.error.bg,
                theme.status.error.fg,
                4.5,
            );
            assert_contrast_at_least(
                &format!("{name}: status.info"),
                theme.status.info.bg,
                theme.status.info.fg,
                4.5,
            );
            assert_contrast_at_least(
                &format!("{name}: border.focus"),
                theme.surface.base,
                theme.border.focus,
                3.0,
            );
        }
    }
}
