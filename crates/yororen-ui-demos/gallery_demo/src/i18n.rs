//! Gallery demo's **own** i18n catalog.
//!
//! The locale crates under `yororen-ui-locale-*` ship the **component
//! defaults** (`common.save`, `button.neutral`, `select.placeholder`,
//! …) — strings the components themselves own. Everything else in this
//! gallery (toolbar titles, demo cell labels, section headings, sample
//! data labels, …) is **caller-specific text** and lives here.
//!
//! ## Why a separate catalog
//!
//! The `yororen-ui-locale-*` crates are framework-level. Mixing
//! demo-specific keys into them would mean every downstream consumer
//! pays the cost of catalog bytes they never use, and editing the
//! demo's strings would require rebuilding the framework crates.
//! Keeping the boundary here makes both layers evolve independently.
//!
//! ## Lookup semantics
//!
//! The gallery's `install_for_locale` flow is:
//! 1. `yororen_ui::locale_xx::install(cx)` — overwrites the global
//!    `I18n` with a fresh catalog containing only the component
//!    defaults for that locale.
//! 2. `cx.global_mut::<I18n>().merge_translations(locale, demo_map)`
//!    — layers the demo's own keys on top. `merge_translations`
//!    shadows component defaults if the demo ever needs to (none
//!    today — the keys are namespaced `demo.*` and never collide).
//!
//! All demo keys live under the `demo.*` namespace so a future
//! sweep for "strings that should not be in the framework" is a
//! one-line regex.

use yororen_ui::i18n::{I18n, I18nContext, Locale, TranslationMap, parse_translation_value};

use crate::state::LocaleChoice;

const RAW_EN: &str = include_str!("../translations/en.json");
const RAW_ZH_CN: &str = include_str!("../translations/zh-CN.json");
const RAW_AR: &str = include_str!("../translations/ar.json");

/// Parse a bundled demo JSON file into a `TranslationMap`.
///
/// Panics on malformed JSON — the bundled files are checked-in
/// artifacts, so a syntax error is a build-time bug, not a runtime
/// condition.
fn parse_demo(raw: &str) -> TranslationMap {
    let value: serde_json::Value =
        serde_json::from_str(raw).expect("bundled gallery demo JSON must be valid");
    parse_translation_value(value).expect("bundled gallery demo JSON must be a JSON object")
}

/// Demo-only translation map for the chosen locale.
fn demo_translation_map(choice: LocaleChoice) -> TranslationMap {
    match choice {
        LocaleChoice::En => parse_demo(RAW_EN),
        LocaleChoice::ZhCn => parse_demo(RAW_ZH_CN),
        LocaleChoice::Ar => parse_demo(RAW_AR),
    }
}

/// Install the chosen locale on `cx`, layering the gallery demo's own
/// translations on top of the component defaults from
/// `yororen-ui-locale-*`.
///
/// Idempotent: re-calling with the same `choice` is a no-op-ish merge
/// (the same keys are overwritten with the same values). Re-calling
/// with a different `choice` swaps both the active locale and the
/// demo catalog, so a toolbar toggle can hot-swap languages without
/// restarting the app.
///
/// Pair this with `cx.theme()` / `yororen_ui::locale_xx::install`
/// only when you also need the framework-level translations; the
/// gallery always does, so the `main.rs` and toolbar `on_toggle`
/// paths call this helper exclusively.
pub fn install_for_locale(cx: &mut gpui::App, choice: LocaleChoice) {
    // 1. Component defaults.
    match choice {
        LocaleChoice::En => yororen_ui::locale_en::install(cx),
        LocaleChoice::ZhCn => yororen_ui::locale_zh_cn::install(cx),
        LocaleChoice::Ar => yororen_ui::locale_ar::install(cx),
    }
    // 2. Demo's own keys, merged on top.
    let locale: Locale = cx.i18n().locale().clone();
    let demo_map = demo_translation_map(choice);
    cx.global_mut::<I18n>().merge_translations(locale, demo_map);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_keys_resolve() {
        let map = parse_demo(RAW_EN);
        assert_eq!(
            map.get("demo.title").map(String::from),
            Some("yororen-ui gallery".to_string())
        );
        assert_eq!(
            map.get("demo.section_actions").map(String::from),
            Some("1. Actions".to_string())
        );
    }

    #[test]
    fn demo_zh_keys_resolve() {
        let map = parse_demo(RAW_ZH_CN);
        assert_eq!(
            map.get("demo.title").map(String::from),
            Some("yororen-ui 画廊".to_string())
        );
    }

    #[test]
    fn demo_ar_keys_resolve() {
        let map = parse_demo(RAW_AR);
        assert_eq!(
            map.get("demo.title").map(String::from),
            Some("معرض yororen-ui".to_string())
        );
    }

    /// Every locale must have the same set of `demo.*` keys.
    /// Catches the bug where a key was added to `en.json` but
    /// the other locales silently fall back to the key path
    /// (e.g. the "common.top" → "demo.common.top" rename left
    /// `en` correct but `zh` / `ar` rendering as the raw key).
    #[test]
    fn demo_keys_present_in_all_locales() {
        fn collect(raw: &str) -> std::collections::BTreeSet<String> {
            let map = parse_demo(raw);
            map.values()
                .keys()
                .map(|k| format!("demo.{k}"))
                .collect()
        }
        let en = collect(RAW_EN);
        let zh = collect(RAW_ZH_CN);
        let ar = collect(RAW_AR);
        let in_en_only: Vec<_> = en.difference(&zh).chain(en.difference(&ar)).collect();
        let in_zh_only: Vec<_> = zh.difference(&en).collect();
        let in_ar_only: Vec<_> = ar.difference(&en).collect();
        assert!(
            in_en_only.is_empty(),
            "keys present in en but missing in zh/ar: {in_en_only:?}"
        );
        assert!(
            in_zh_only.is_empty(),
            "keys present in zh but missing in en: {in_zh_only:?}"
        );
        assert!(
            in_ar_only.is_empty(),
            "keys present in ar but missing in en: {in_ar_only:?}"
        );
    }
}
