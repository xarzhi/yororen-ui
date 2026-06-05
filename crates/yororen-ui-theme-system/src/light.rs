//! Light theme palette (neutral, no-brand).

use gpui::{hsla, rgb};

use yororen_ui_renderer::theme::{
    ActionTheme, ActionVariant, BorderTheme, ContentTheme, ShadowTheme, StatusTheme, StatusVariant,
    SurfaceTheme, Theme,
};

use yororen_ui_core::i18n::TextDirection;

use yororen_ui_renderer::renderers::RendererRegistry;
use yororen_ui_renderer::theme::tokens::DesignTokens;

/// Default light theme. Neutral palette, no brand colors.
pub fn light() -> Theme {
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
