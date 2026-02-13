//! Internationalization runtime state and management.

use std::collections::HashMap;
use std::sync::Arc;

use gpui::{App, Global, SharedString};

use super::locale::{Locale, SupportedLocale, TextDirection};

/// Global i18n state that stores the current locale and available translations.
pub struct I18n {
    /// Current active locale.
    pub current_locale: Locale,
    /// Available locales.
    pub available_locales: Vec<SupportedLocale>,
    /// Translation strings indexed by locale.
    translations: HashMap<Locale, Arc<TranslationMap>>,
}

impl Global for I18n {}

impl I18n {
    /// Create a new i18n instance with default English locale.
    pub fn new() -> Self {
        Self::with_locale(Locale::default())
    }

    /// Create a new i18n instance with a specific locale.
    pub fn with_locale(locale: Locale) -> Self {
        Self {
            current_locale: locale,
            available_locales: SupportedLocale::all().to_vec(),
            translations: HashMap::new(),
        }
    }

    /// Set the current locale.
    pub fn set_locale(&mut self, locale: Locale) {
        self.current_locale = locale;
    }

    /// Get the current locale.
    pub fn locale(&self) -> &Locale {
        &self.current_locale
    }

    /// Get the text direction for the current locale.
    pub fn text_direction(&self) -> TextDirection {
        self.current_locale.text_direction()
    }

    /// Check if RTL mode is active.
    pub fn is_rtl(&self) -> bool {
        self.text_direction().is_rtl()
    }

    /// Load translations for a locale.
    pub fn load_translations(&mut self, locale: Locale, translations: TranslationMap) {
        self.translations.insert(locale, Arc::new(translations));
    }

    /// Get translations for the current locale.
    pub fn translations(&self) -> Option<&Arc<TranslationMap>> {
        self.translations.get(&self.current_locale)
    }

    /// Get a translation by key.
    pub fn t(&self, key: &str) -> Option<&str> {
        self.translations()?.get(key)
    }
}

impl Default for I18n {
    fn default() -> Self {
        Self::new()
    }
}

/// Translation map that stores key-value pairs.
#[derive(Clone, Debug, Default)]
pub struct TranslationMap {
    /// Flat key-value map for simple translations.
    values: HashMap<String, String>,
    /// Nested translations.
    nested: HashMap<String, TranslationMap>,
}

impl TranslationMap {
    /// Create a new empty translation map.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert a flat key-value pair.
    pub fn insert(&mut self, key: &str, value: &str) {
        self.values.insert(key.to_string(), value.to_string());
    }

    /// Insert a nested translation map.
    pub fn insert_nested(&mut self, key: &str, map: TranslationMap) {
        self.nested.insert(key.to_string(), map);
    }

    /// Get a translation by key, supporting dot notation for nested keys.
    pub fn get(&self, key: &str) -> Option<&str> {
        // First try direct key
        if let Some(value) = self.values.get(key) {
            return Some(value);
        }

        // Try dot notation for nested keys
        let parts: Vec<&str> = key.split('.').collect();
        if parts.len() < 2 {
            return None;
        }

        let mut current = self;
        for (i, part) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Last part - try to get from values
                return current.values.get(*part).map(|s| s.as_str());
            } else {
                // Navigate to nested map
                current = current.nested.get(*part)?;
            }
        }

        None
    }

    /// Get all flat key-value pairs.
    pub fn values(&self) -> &HashMap<String, String> {
        &self.values
    }

    /// Get all nested maps.
    pub fn nested(&self) -> &HashMap<String, TranslationMap> {
        &self.nested
    }
}

/// Helper to access i18n from app context.
pub trait I18nContext {
    fn i18n(&self) -> &I18n;
}

impl I18nContext for App {
    fn i18n(&self) -> &I18n {
        self.global::<I18n>()
    }
}

/// Helper to translate strings within app context.
pub trait Translate {
    /// Translate a key to a string.
    fn t(&self, key: &str) -> SharedString;

    /// Translate with placeholders.
    fn t_with_args(&self, key: &str, args: &HashMap<&str, &str>) -> SharedString;
}

impl Translate for App {
    fn t(&self, key: &str) -> SharedString {
        let i18n = self.i18n();
        match i18n.t(key) {
            Some(s) => s.to_string().into(),
            None => key.to_string().into(),
        }
    }

    fn t_with_args(&self, key: &str, args: &HashMap<&str, &str>) -> SharedString {
        let i18n = self.i18n();
        let base = match i18n.t(key) {
            Some(s) => s.to_string(),
            None => key.to_string(),
        };

        replace_placeholders(&base, args).into()
    }
}

/// Replace placeholders in a string with values from the args map.
fn replace_placeholders(template: &str, args: &HashMap<&str, &str>) -> String {
    let mut result = template.to_string();
    for (key, value) in args {
        let placeholder = format!("{{{}}}", key);
        result = result.replace(&placeholder, value);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_map_nested() {
        let mut map = TranslationMap::new();
        map.insert("hello", "Hello");

        let mut nested = TranslationMap::new();
        nested.insert("placeholder", "Select…");
        map.insert_nested("select", nested);

        assert_eq!(map.get("hello"), Some("Hello"));
        assert_eq!(map.get("select.placeholder"), Some("Select…"));
    }

    #[test]
    fn test_replace_placeholders() {
        let template = "Hello {name}, you have {count} items";
        let mut args = HashMap::new();
        args.insert("name", "World");
        args.insert("count", "5");

        let result = replace_placeholders(template, &args);
        assert_eq!(result, "Hello World, you have 5 items");
    }
}
