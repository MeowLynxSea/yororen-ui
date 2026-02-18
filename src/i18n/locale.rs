//! Language environment and text direction definitions.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Text direction for layout (LTR or RTL).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextDirection {
    /// Left-to-right (e.g., English, Chinese)
    #[default]
    Ltr,
    /// Right-to-left (e.g., Arabic, Hebrew)
    Rtl,
}

impl TextDirection {
    /// Check if this direction is RTL.
    pub fn is_rtl(&self) -> bool {
        matches!(self, TextDirection::Rtl)
    }
}

impl fmt::Display for TextDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextDirection::Ltr => write!(f, "ltr"),
            TextDirection::Rtl => write!(f, "rtl"),
        }
    }
}

/// Error type for locale parsing failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocaleParseError;

impl fmt::Display for LocaleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse locale from tag")
    }
}

impl std::error::Error for LocaleParseError {}

/// Simple locale identifier that supports language and region tags.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub struct Locale {
    /// Language code (e.g., "en", "zh", "ar")
    language: String,
    /// Region code (e.g., "US", "CN")
    region: Option<String>,
    /// Variant (e.g., "Hans", "Hant")
    variant: Option<String>,
}

impl Locale {
    /// Create a new locale from a language tag string (e.g., "en", "zh-CN", "ar-SA").
    pub fn new(tag: &str) -> Result<Self, LocaleParseError> {
        let parts: Vec<&str> = tag.split('-').collect();

        let language = parts.first().map(|s| s.to_string()).ok_or(LocaleParseError)?;
        let region = parts.get(1).map(|s| s.to_string());
        let variant = parts.get(2).map(|s| s.to_string());

        Ok(Self {
            language,
            region,
            variant,
        })
    }

    /// Get the language code (e.g., "en", "zh", "ar").
    pub fn language(&self) -> &str {
        &self.language
    }

    /// Get the region code if available (e.g., "US", "CN").
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }

    /// Get the variant if available (e.g., "Hans", "Hant").
    pub fn variant(&self) -> Option<&str> {
        self.variant.as_deref()
    }

    /// Get the text direction for this locale.
    pub fn text_direction(&self) -> TextDirection {
        // Arabic, Hebrew, Farsi, Urdu are RTL languages
        match self.language.as_str() {
            "ar" | "he" | "fa" | "ur" | "yi" | "ps" => TextDirection::Rtl,
            _ => TextDirection::Ltr,
        }
    }

    /// Get the display name for this locale in English.
    pub fn display_name(&self) -> String {
        if let Some(region) = &self.region {
            if let Some(variant) = &self.variant {
                format!("{}-{}-{}", self.language, region, variant)
            } else {
                format!("{}-{}", self.language, region)
            }
        } else {
            self.language.clone()
        }
    }

    /// Convert to BCP 47 tag string.
    pub fn to_tag(&self) -> String {
        self.display_name()
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new("en").unwrap()
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl FromStr for Locale {
    type Err = LocaleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

/// Common locales supported by the application.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SupportedLocale {
    /// English (United States)
    EnUS,
    /// English (Great Britain)
    EnGB,
    /// Chinese (Simplified)
    ZhCN,
    /// Chinese (Traditional)
    ZhTW,
    /// Japanese
    Ja,
    /// Arabic
    Ar,
    /// Hebrew
    He,
    /// French
    Fr,
    /// German
    De,
    /// Spanish
    Es,
    /// Korean
    Ko,
}

impl SupportedLocale {
    /// Convert to `Locale` type.
    pub fn to_locale(&self) -> Locale {
        match self {
            SupportedLocale::EnUS => Locale::new("en-US").unwrap(),
            SupportedLocale::EnGB => Locale::new("en-GB").unwrap(),
            SupportedLocale::ZhCN => Locale::new("zh-CN").unwrap(),
            SupportedLocale::ZhTW => Locale::new("zh-TW").unwrap(),
            SupportedLocale::Ja => Locale::new("ja").unwrap(),
            SupportedLocale::Ar => Locale::new("ar").unwrap(),
            SupportedLocale::He => Locale::new("he").unwrap(),
            SupportedLocale::Fr => Locale::new("fr").unwrap(),
            SupportedLocale::De => Locale::new("de").unwrap(),
            SupportedLocale::Es => Locale::new("es").unwrap(),
            SupportedLocale::Ko => Locale::new("ko").unwrap(),
        }
    }

    /// Get all supported locales.
    pub fn all() -> &'static [SupportedLocale] {
        &[
            SupportedLocale::EnUS,
            SupportedLocale::EnGB,
            SupportedLocale::ZhCN,
            SupportedLocale::ZhTW,
            SupportedLocale::Ja,
            SupportedLocale::Ar,
            SupportedLocale::He,
            SupportedLocale::Fr,
            SupportedLocale::De,
            SupportedLocale::Es,
            SupportedLocale::Ko,
        ]
    }

    /// Try to match a locale string to a supported locale.
    pub fn match_locale(tag: &str) -> Option<SupportedLocale> {
        // Try exact match first
        if let Some(&locale) = Self::all().iter().find(|&&l| l.to_locale().to_tag() == tag) {
            return Some(locale);
        }

        // Try language-only match
        let lang = tag.split('-').next()?;
        Self::all().iter().find(|&&locale| locale.to_locale().language() == lang).copied()
    }
}

impl From<SupportedLocale> for Locale {
    fn from(locale: SupportedLocale) -> Self {
        locale.to_locale()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_locale_creation() {
        let locale = Locale::new("zh-CN").unwrap();
        assert_eq!(locale.language(), "zh");
        assert_eq!(locale.region(), Some("CN"));
    }

    #[test]
    fn test_text_direction() {
        let en = Locale::new("en").unwrap();
        assert_eq!(en.text_direction(), TextDirection::Ltr);

        let ar = Locale::new("ar").unwrap();
        assert_eq!(ar.text_direction(), TextDirection::Rtl);
    }

    #[test]
    fn test_supported_locale_match() {
        assert!(SupportedLocale::match_locale("zh-CN").is_some());
        assert!(SupportedLocale::match_locale("en").is_some());
        assert!(SupportedLocale::match_locale("unknown").is_none());
    }
}
