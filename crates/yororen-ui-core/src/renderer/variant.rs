//! Built-in `ActionVariantKind` — shared by core (headless props
//! like `ButtonProps.variant`) and the default-renderer (where the
//! `TokenButtonRenderer` resolves the kind to a `theme.action.*`
//! palette path).
//!
//! Lifted out of `yororen-ui-default-renderer` so headless builders
//! can name a variant without depending on the renderer crate.

use std::borrow::Cow;

use gpui::Hsla;

use crate::theme::Theme;

/// Logical kind of an action button. Maps to one of the three
/// entries under `theme.action.*` in the JSON theme.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ActionVariantKind {
    #[default]
    Neutral,
    Primary,
    Danger,
}

impl ActionVariantKind {
    /// Lowercase key used to look up `action.<key>.<field>` paths
    /// in the theme JSON. Stable, exposed for diagnostic
    /// messages.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Neutral => "neutral",
            Self::Primary => "primary",
            Self::Danger => "danger",
        }
    }
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

impl From<ActionVariantKind> for BuiltinVariantKey {
    fn from(k: ActionVariantKind) -> Self {
        match k {
            ActionVariantKind::Neutral => Self::Neutral,
            ActionVariantKind::Primary => Self::Primary,
            ActionVariantKind::Danger => Self::Danger,
        }
    }
}

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
    pub fn from_action(v_kind: ActionVariantKind, theme: &Theme) -> Self {
        let key = v_kind.as_str();
        Self {
            bg: theme
                .get_color(&format!("action.{key}.bg"))
                .unwrap_or_default(),
            fg: theme
                .get_color(&format!("action.{key}.fg"))
                .unwrap_or_default(),
            disabled_bg: theme
                .get_color(&format!("action.{key}.disabled_bg"))
                .unwrap_or_default(),
            disabled_fg: theme
                .get_color(&format!("action.{key}.disabled_fg"))
                .unwrap_or_default(),
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
/// `with_defaults_for_theme` and remain immutable. Custom variants
/// live behind a `RwLock` so multiple threads can register and
/// resolve in parallel.
pub struct VariantRegistry {
    builtins: std::collections::HashMap<BuiltinVariantKey, std::sync::Arc<dyn VariantStyle>>,
    customs:
        std::sync::RwLock<std::collections::HashMap<VariantKey, std::sync::Arc<dyn VariantStyle>>>,
}

impl Default for VariantRegistry {
    fn default() -> Self {
        Self::empty()
    }
}

impl VariantRegistry {
    /// Empty registry (no builtins). Useful for tests.
    pub fn empty() -> Self {
        Self {
            builtins: std::collections::HashMap::new(),
            customs: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }

    /// Construct a registry with the 3 built-in variants seeded from
    /// the supplied `Theme`'s `action.*` palettes.
    pub fn with_defaults_for_theme(theme: &Theme) -> Self {
        let mut r = Self::empty();
        for k in [
            ActionVariantKind::Neutral,
            ActionVariantKind::Primary,
            ActionVariantKind::Danger,
        ] {
            r.builtins.insert(
                k.into(),
                std::sync::Arc::new(TokenVariantStyle::from_action(k, theme)),
            );
        }
        r
    }

    /// Add a built-in variant (typically called by `core` once at
    /// startup). Existing entries are kept — `with_builtin` does
    /// not overwrite.
    pub fn with_builtin(
        mut self,
        key: BuiltinVariantKey,
        style: std::sync::Arc<dyn VariantStyle>,
    ) -> Self {
        self.builtins.entry(key).or_insert(style);
        self
    }

    /// Register (or replace) a custom variant.
    pub fn register(&self, key: VariantKey, style: std::sync::Arc<dyn VariantStyle>) {
        if let Ok(mut w) = self.customs.write() {
            w.insert(key, style);
        }
    }

    /// Remove a previously-registered custom variant.
    pub fn unregister(&self, key: &VariantKey) -> Option<std::sync::Arc<dyn VariantStyle>> {
        self.customs.write().ok()?.remove(key)
    }

    /// Resolve a custom variant by key.
    pub fn resolve(&self, key: &VariantKey) -> Option<std::sync::Arc<dyn VariantStyle>> {
        self.customs.read().ok()?.get(key).cloned()
    }

    /// Built-in lookup.
    pub fn builtin(&self, key: BuiltinVariantKey) -> Option<std::sync::Arc<dyn VariantStyle>> {
        self.builtins.get(&key).cloned()
    }

    /// Number of custom variants currently registered.
    pub fn custom_count(&self) -> usize {
        self.customs.read().map(|r| r.len()).unwrap_or(0)
    }
}

/// Global wrapper so the registry can be set on `App` with
/// `cx.set_global(GlobalVariantRegistry(...))`.
pub struct GlobalVariantRegistry(pub std::sync::Arc<VariantRegistry>);

impl gpui::Global for GlobalVariantRegistry {}

/// Variant-aware button variant tag. Built-in variants (Neutral /
/// Primary / Danger) are handled directly by `TokenButtonRenderer`.
/// Custom variants route through the global `VariantRegistry` to
/// a third-party `VariantStyle`.
#[derive(Clone, Debug)]
pub enum ButtonVariant {
    /// Built-in variant handled by the default renderer.
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
    /// `None` for `Custom` variants.
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

/// Compose a base variant with a list of override variants.
///
/// Semantics mirror `class-variance-authority` / CSS class
/// composition: each subsequent override fully replaces the
/// previous one for `bg` / `fg` / `border`. The last override in
/// the list wins. If `overrides` is empty, the composed style is
/// behaviorally identical to `base`. `disabled_opacity` is always
/// taken from `base`, because it expresses the disabled policy of
/// the underlying variant — overrides typically only restyle the
/// active surface, not the disabled fade.
pub fn variant_compose(
    base: std::sync::Arc<dyn VariantStyle>,
    overrides: &[(VariantKey, std::sync::Arc<dyn VariantStyle>)],
) -> std::sync::Arc<dyn VariantStyle> {
    std::sync::Arc::new(ComposedVariantStyle {
        base,
        overrides: overrides.to_vec(),
    })
}

struct ComposedVariantStyle {
    base: std::sync::Arc<dyn VariantStyle>,
    overrides: Vec<(VariantKey, std::sync::Arc<dyn VariantStyle>)>,
}

impl std::fmt::Debug for ComposedVariantStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ComposedVariantStyle")
            .field("overrides_count", &self.overrides.len())
            .finish()
    }
}

impl ComposedVariantStyle {
    fn effective(&self) -> &dyn VariantStyle {
        match self.overrides.last() {
            Some((_, s)) => s.as_ref(),
            None => self.base.as_ref(),
        }
    }
}

impl VariantStyle for ComposedVariantStyle {
    fn bg(&self, state: &VariantState) -> Hsla {
        self.effective().bg(state)
    }
    fn fg(&self, state: &VariantState) -> Hsla {
        self.effective().fg(state)
    }
    fn border(&self, state: &VariantState) -> Option<Hsla> {
        self.effective().border(state)
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
    fn action_variant_as_str() {
        assert_eq!(ActionVariantKind::Neutral.as_str(), "neutral");
        assert_eq!(ActionVariantKind::Primary.as_str(), "primary");
        assert_eq!(ActionVariantKind::Danger.as_str(), "danger");
    }

    #[test]
    fn token_variant_style_from_action_reads_theme_paths() {
        let theme = Theme::from_value(serde_json::json!({
            "action": {
                "primary": { "bg": "#ff0000", "fg": "#ffffff" }
            }
        }));
        let s = TokenVariantStyle::from_action(ActionVariantKind::Primary, &theme);
        // Red's HSL: hue=0. Verify against the theme.
        let bg = s.bg(&VariantState::default());
        assert!(
            bg.saturation > 0.5,
            "primary bg should be saturated red, got {:?}",
            bg
        );
    }

    #[test]
    fn button_variant_default_is_neutral_builtin() {
        let v = ButtonVariant::default();
        assert_eq!(v.as_str(), "neutral");
        assert_eq!(v.as_builtin(), Some(ActionVariantKind::Neutral));
    }

    #[test]
    fn button_variant_custom_key_string_round_trip() {
        let v = ButtonVariant::Custom(VariantKey::borrowed("branded"));
        assert_eq!(v.as_str(), "branded");
        assert!(v.as_builtin().is_none());
    }

    #[test]
    fn empty_registry_has_no_customs() {
        let r = VariantRegistry::empty();
        assert_eq!(r.custom_count(), 0);
        assert!(r.resolve(&VariantKey::borrowed("x")).is_none());
    }

    #[test]
    fn register_and_resolve_custom() {
        let r = VariantRegistry::empty();
        #[derive(Debug)]
        struct Ghost;
        impl VariantStyle for Ghost {
            fn bg(&self, _: &VariantState) -> Hsla {
                gpui::rgb_to_hsla(rgb(0x000000))
            }
            fn fg(&self, _: &VariantState) -> Hsla {
                gpui::rgb_to_hsla(rgb(0xFFFFFF))
            }
            fn border(&self, _: &VariantState) -> Option<Hsla> {
                None
            }
            fn disabled_opacity(&self) -> f32 {
                1.0
            }
        }
        let key = VariantKey::borrowed("ghost");
        r.register(key.clone(), std::sync::Arc::new(Ghost));
        assert_eq!(r.custom_count(), 1);
        let resolved = r.resolve(&key).expect("ghost should be registered");
        assert_eq!(resolved.bg(&VariantState::default()), gpui::rgb_to_hsla(rgb(0x000000)));
    }
}
