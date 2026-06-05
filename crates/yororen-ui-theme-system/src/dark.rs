//! Dark theme palette (neutral, no-brand).

use gpui::{hsla, rgb};

use yororen_ui_renderer::theme::{
    ActionTheme, ActionVariant, BorderTheme, ContentTheme, ShadowTheme, StatusTheme, StatusVariant,
    SurfaceTheme, Theme,
};

use yororen_ui_core::i18n::TextDirection;

use yororen_ui_renderer::renderers::RendererRegistry;
use yororen_ui_renderer::theme::tokens::DesignTokens;

/// Default dark theme. Neutral palette, no brand colors.
pub fn dark() -> Theme {
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
