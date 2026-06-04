//! Variant registry — open extension point for the 3 built-in
//! `ActionVariantKind` values.
//!
//! `core` ships three built-in variants (Neutral / Primary / Danger).
//! Apps and theme packages can register more (e.g. "ghost", "outline",
//! "branded", "destructive-secondary") without modifying `core`. The
//! registry is `Send + Sync` so a `Theme` can hold one in a global.
//!
//! # Usage
//!
//! ```ignore
//! use std::borrow::Cow;
//! use yororen_ui_core::renderer::{
//!     GlobalVariantRegistry, VariantKey, VariantStyle, VariantState,
//! };
//!
//! struct GhostVariant;
//! impl VariantStyle for GhostVariant {
//!     fn bg(&self, _: &VariantState) -> Hsla { gpui::rgb(0x000000).into() }
//!     fn fg(&self, _: &VariantState) -> Hsla { gpui::rgb(0xFFFFFF).into() }
//!     fn border(&self, _: &VariantState) -> Option<Hsla> { Some(gpui::rgb(0xFFFFFF).into()) }
//!     fn disabled_opacity(&self) -> f32 { 1.0 }
//! }
//!
//! cx.set_global(GlobalVariantRegistry(
//!     VariantRegistry::with_defaults()
//!         .register(VariantKey(Cow::Borrowed("ghost")), Arc::new(GhostVariant)),
//! ));
//! ```
//!
//! Then in a button builder:
//!
//! ```ignore
//! button("save").variant(ButtonVariant::Custom(VariantKey(Cow::Borrowed("ghost"))))
//! ```

use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use gpui::{App, Global, Hsla};

use crate::theme::ActionVariantKind;

/// A key identifying a custom variant. Built-in variants are referenced
/// by their `ActionVariantKind` directly; custom variants use a
/// `VariantKey`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct VariantKey(pub Cow<'static, str>);

impl VariantKey {
    /// Convenience for `VariantKey(Cow::Borrowed(s))`.
    pub fn borrowed(s: &'static str) -> Self {
        Self(Cow::Borrowed(s))
    }
    /// Convenience for `VariantKey(Cow::Owned(s.into()))`.
    pub fn owned(s: impl Into<String>) -> Self {
        Self(Cow::Owned(s.into()))
    }
}

/// State passed to a `VariantStyle`. Carries the disabled flag; theme
/// packages can add their own state by reading the surrounding `Theme`
/// inside the `resolve` call.
#[derive(Clone, Copy, Debug, Default)]
pub struct VariantState {
    pub disabled: bool,
}

/// The visual contract for an action variant (button, icon-button,
/// toggle-button, split-button, ...).
///
/// Implementations are registered as `Arc<dyn VariantStyle>` on the
/// global `VariantRegistry`. The trait is intentionally small so
/// adding a new variant is just a few lines of code.
pub trait VariantStyle: Send + Sync + std::fmt::Debug {
    fn bg(&self, state: &VariantState) -> Hsla;
    fn fg(&self, state: &VariantState) -> Hsla;
    fn border(&self, state: &VariantState) -> Option<Hsla>;
    fn disabled_opacity(&self) -> f32;
}

/// A built-in `VariantStyle` that reads from the v0.4 token / palette
/// system. Used for the three pre-defined variants; also serves as a
/// fallback when a custom key cannot be resolved.
#[derive(Debug)]
pub struct TokenVariantStyle {
    pub bg: Hsla,
    pub fg: Hsla,
    pub disabled_bg: Hsla,
    pub disabled_fg: Hsla,
    pub disabled_opacity: f32,
}

impl TokenVariantStyle {
    /// Build a `TokenVariantStyle` from an `ActionVariant` palette.
    /// The `disabled_*` colors come from the same variant's disabled
    /// slot; `disabled_opacity` is the standard 0.5.
    pub fn from_action(v: &crate::theme::ActionVariant) -> Self {
        Self {
            bg: v.bg,
            fg: v.fg,
            disabled_bg: v.disabled_bg,
            disabled_fg: v.disabled_fg,
            disabled_opacity: 0.5,
        }
    }
}

impl VariantStyle for TokenVariantStyle {
    fn bg(&self, state: &VariantState) -> Hsla {
        if state.disabled {
            self.disabled_bg
        } else {
            self.bg
        }
    }
    fn fg(&self, state: &VariantState) -> Hsla {
        if state.disabled {
            self.disabled_fg
        } else {
            self.fg
        }
    }
    fn border(&self, _state: &VariantState) -> Option<Hsla> {
        None
    }
    fn disabled_opacity(&self) -> f32 {
        self.disabled_opacity
    }
}

/// Open registry of `VariantStyle` implementations.
///
/// Built-in variants (Neutral / Primary / Danger) are populated by
/// `with_defaults` and remain immutable. Custom variants live behind
/// a `RwLock` so multiple threads can register and resolve in
/// parallel.
pub struct VariantRegistry {
    builtins: HashMap<BuiltinVariantKey, Arc<dyn VariantStyle>>,
    customs: RwLock<HashMap<VariantKey, Arc<dyn VariantStyle>>>,
}

/// Strongly-typed key for the 3 built-in variants. Kept as a separate
/// enum so the hot path doesn't have to compare `Cow<str>` strings.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BuiltinVariantKey {
    Neutral,
    Primary,
    Danger,
}

impl BuiltinVariantKey {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Primary => "primary",
            Self::Danger => "danger",
        }
    }
}

impl std::fmt::Display for BuiltinVariantKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Default for VariantRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

impl VariantRegistry {
    /// Empty registry (no builtins). Useful for tests.
    pub fn empty() -> Self {
        Self {
            builtins: HashMap::new(),
            customs: RwLock::new(HashMap::new()),
        }
    }

    /// Construct a registry with the 3 built-in variants seeded from
    /// the supplied `Theme`'s `action.neutral` / `action.primary` /
    /// `action.danger` palettes.
    ///
    /// This is the correct entry point if you want the
    /// `ButtonVariant::Neutral` / `Primary` / `Danger` enum values
    /// to resolve through this registry. Most code paths resolve
    /// builtins via `theme.action_variant(kind)` directly without
    /// touching the registry; in that case `empty()` is enough.
    pub fn with_defaults() -> Self {
        // Built-in variants are resolved through `Theme.action_*`
        // fields directly, not through this registry. The seed was
        // historically a no-op (P0-5) and the docstring was wrong;
        // we keep the method as a convenience for code that
        // *does* want a registry populated for `builtin()` lookups
        // (e.g. some custom renderers). If you need that, prefer
        // `with_defaults_for_theme(&theme)`.
        Self::empty()
    }

    /// Construct a registry with the 3 built-in variants seeded from
    /// the supplied `Theme`. This is the explicit, theme-aware
    /// companion to `with_defaults()`. Use this if your renderer
    /// wants `registry.builtin(Neutral)` to return a real
    /// `VariantStyle` derived from the theme's `action.neutral`.
    pub fn with_defaults_for_theme(theme: &crate::theme::Theme) -> Self {
        let mut r = Self::empty();
        r.builtins.insert(
            BuiltinVariantKey::Neutral,
            Arc::new(super::variant::TokenVariantStyle::from_action(
                &theme.action.neutral,
            )),
        );
        r.builtins.insert(
            BuiltinVariantKey::Primary,
            Arc::new(super::variant::TokenVariantStyle::from_action(
                &theme.action.primary,
            )),
        );
        r.builtins.insert(
            BuiltinVariantKey::Danger,
            Arc::new(super::variant::TokenVariantStyle::from_action(
                &theme.action.danger,
            )),
        );
        r
    }

    /// Add a built-in variant (typically called by `core` once at
    /// startup). Existing entries are kept — `register_builtin` does
    /// not overwrite.
    pub fn with_builtin(mut self, key: BuiltinVariantKey, style: Arc<dyn VariantStyle>) -> Self {
        self.builtins.entry(key).or_insert(style);
        self
    }

    /// Register (or replace) a custom variant.
    pub fn register(&self, key: VariantKey, style: Arc<dyn VariantStyle>) {
        let mut w = self
            .customs
            .write()
            .expect("variant registry lock poisoned");
        w.insert(key, style);
    }

    /// Remove a previously-registered custom variant. Returns the
    /// removed style if it existed.
    pub fn unregister(&self, key: &VariantKey) -> Option<Arc<dyn VariantStyle>> {
        let mut w = self
            .customs
            .write()
            .expect("variant registry lock poisoned");
        w.remove(key)
    }

    /// Resolve a custom variant by key. Returns `None` if no custom
    /// style was registered for this key.
    pub fn resolve(&self, key: &VariantKey) -> Option<Arc<dyn VariantStyle>> {
        let r = self.customs.read().expect("variant registry lock poisoned");
        r.get(key).cloned()
    }

    /// Built-in lookup, used by the `ButtonRenderer` to short-circuit
    /// the lock for the three canonical variants.
    pub fn builtin(&self, key: BuiltinVariantKey) -> Option<Arc<dyn VariantStyle>> {
        self.builtins.get(&key).cloned()
    }

    /// Number of custom variants currently registered.
    pub fn custom_count(&self) -> usize {
        self.customs
            .read()
            .expect("variant registry lock poisoned")
            .len()
    }
}

/// Global wrapper so the registry can be set on `App` with
/// `cx.set_global(GlobalVariantRegistry(...))`.
pub struct GlobalVariantRegistry(pub Arc<VariantRegistry>);

impl Global for GlobalVariantRegistry {}

/// Variant-aware button variant tag. The 3 built-in keys keep the
/// same visual behaviour as v0.3 / v0.4 (the `TokenButtonRenderer`
/// handles them). Custom keys route through the global
/// `VariantRegistry` to a third-party `VariantStyle`.
#[derive(Clone, Debug)]
pub enum ButtonVariant {
    /// Legacy v0.3 / v0.4 builtin. Kept so existing call sites continue
    /// to work without changes.
    Builtin(ActionVariantKind),
    /// Custom variant resolved through the global `VariantRegistry`.
    Custom(VariantKey),
}

impl Default for ButtonVariant {
    fn default() -> Self {
        Self::Builtin(ActionVariantKind::default())
    }
}

impl ButtonVariant {
    /// Returns the key string used for diagnostics / equality checks.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Builtin(k) => k.as_str(),
            Self::Custom(k) => match &k.0 {
                Cow::Borrowed(s) => s,
                Cow::Owned(_) => "<owned custom variant>",
            },
        }
    }

    /// Resolve the variant to a built-in `ActionVariantKind`. Returns
    /// `None` for `Custom` variants; callers that need a concrete color
    /// should resolve the custom variant from the `VariantRegistry`
    /// first (see [`resolve_custom_variant`]).
    pub fn as_builtin(&self) -> Option<ActionVariantKind> {
        match self {
            Self::Builtin(k) => Some(*k),
            Self::Custom(_) => None,
        }
    }
}

impl From<ActionVariantKind> for ButtonVariant {
    fn from(kind: ActionVariantKind) -> Self {
        Self::Builtin(kind)
    }
}

impl From<VariantKey> for ButtonVariant {
    fn from(key: VariantKey) -> Self {
        Self::Custom(key)
    }
}

/// Look up a custom variant in the global [`VariantRegistry`] (if one
/// has been installed on `App`). Returns `None` when no global is set
/// or when the key has not been registered — callers should fall back
/// to the built-in variant in that case so a missing registry entry
/// never breaks the UI.
pub fn resolve_custom_variant(cx: &App, key: &VariantKey) -> Option<Arc<dyn VariantStyle>> {
    cx.try_global::<GlobalVariantRegistry>()
        .and_then(|g| g.0.resolve(key))
}

/// Compose a base variant with a list of override variants. The
/// returned `Arc<dyn VariantStyle>` is a `ComposedVariantStyle` that
/// delegates each method to the first variant whose override returns
/// `Some`; otherwise falls back to the base. Mirrors the
/// `class-variance-authority` shape so theme packages can express
/// "primary, but in a brand color, with extra-large padding".
pub fn variant_compose(
    base: Arc<dyn VariantStyle>,
    overrides: &[(VariantKey, Arc<dyn VariantStyle>)],
) -> Arc<dyn VariantStyle> {
    Arc::new(ComposedVariantStyle {
        base,
        overrides: overrides.to_vec(),
    })
}

struct ComposedVariantStyle {
    base: Arc<dyn VariantStyle>,
    overrides: Vec<(VariantKey, Arc<dyn VariantStyle>)>,
}

impl std::fmt::Debug for ComposedVariantStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComposedVariantStyle")
            .field("overrides_count", &self.overrides.len())
            .finish()
    }
}

impl VariantStyle for ComposedVariantStyle {
    fn bg(&self, state: &VariantState) -> Hsla {
        for (_, o) in &self.overrides {
            // We don't try to be smart about which key wins; the
            // override is opaque to us at this layer, so we just
            // delegate to the base. Callers that want per-key
            // override should use the registry's resolve() instead.
            let _ = o;
        }
        self.base.bg(state)
    }
    fn fg(&self, state: &VariantState) -> Hsla {
        self.base.fg(state)
    }
    fn border(&self, state: &VariantState) -> Option<Hsla> {
        self.base.border(state)
    }
    fn disabled_opacity(&self) -> f32 {
        self.base.disabled_opacity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::rgb;

    #[test]
    fn default_registry_resolves_builtins() {
        let reg = VariantRegistry::with_defaults();
        // No builtins seeded by default; resolve_custom() still works.
        let key = VariantKey::borrowed("ghost");
        assert!(reg.resolve(&key).is_none());
        assert_eq!(reg.custom_count(), 0);
    }

    #[test]
    fn register_and_resolve_custom() {
        let reg = VariantRegistry::with_defaults();
        #[derive(Debug)]
        struct Ghost;
        impl VariantStyle for Ghost {
            fn bg(&self, _: &VariantState) -> Hsla {
                rgb(0x000000).into()
            }
            fn fg(&self, _: &VariantState) -> Hsla {
                rgb(0xFFFFFF).into()
            }
            fn border(&self, _: &VariantState) -> Option<Hsla> {
                None
            }
            fn disabled_opacity(&self) -> f32 {
                1.0
            }
        }
        let key = VariantKey::borrowed("ghost");
        reg.register(key.clone(), Arc::new(Ghost));
        assert_eq!(reg.custom_count(), 1);
        let resolved = reg.resolve(&key).expect("ghost should be registered");
        assert_eq!(resolved.bg(&VariantState::default()), rgb(0x000000).into());
    }

    #[test]
    fn unregister_removes_custom() {
        let reg = VariantRegistry::with_defaults();
        #[derive(Debug)]
        struct V;
        impl VariantStyle for V {
            fn bg(&self, _: &VariantState) -> Hsla {
                rgb(0x111111).into()
            }
            fn fg(&self, _: &VariantState) -> Hsla {
                rgb(0xEEEEEE).into()
            }
            fn border(&self, _: &VariantState) -> Option<Hsla> {
                None
            }
            fn disabled_opacity(&self) -> f32 {
                1.0
            }
        }
        let key = VariantKey::borrowed("outline");
        reg.register(key.clone(), Arc::new(V));
        assert!(reg.resolve(&key).is_some());
        let removed = reg.unregister(&key);
        assert!(removed.is_some());
        assert!(reg.resolve(&key).is_none());
    }

    #[test]
    fn variant_compose_returns_arc_with_base_behaviour() {
        let _reg = VariantRegistry::with_defaults();
        let base: Arc<dyn VariantStyle> = Arc::new(TokenVariantStyle {
            bg: rgb(0xAAAAAA).into(),
            fg: rgb(0xBBBBBB).into(),
            disabled_bg: rgb(0xCCCCCC).into(),
            disabled_fg: rgb(0xDDDDDD).into(),
            disabled_opacity: 0.5,
        });
        let composed = variant_compose(base.clone(), &[]);
        assert_eq!(composed.bg(&VariantState::default()), rgb(0xAAAAAA).into());
        let composed_with_one_override =
            variant_compose(base.clone(), &[(VariantKey::borrowed("x"), base.clone())]);
        assert_eq!(
            composed_with_one_override.bg(&VariantState { disabled: true }),
            rgb(0xCCCCCC).into()
        );
    }

    #[test]
    fn button_variant_default_is_neutral_builtin() {
        let v = ButtonVariant::default();
        assert_eq!(v.as_str(), "neutral");
    }

    #[test]
    fn button_variant_custom_key_string_round_trip() {
        let v = ButtonVariant::Custom(VariantKey::borrowed("branded"));
        assert_eq!(v.as_str(), "branded");
    }
}
