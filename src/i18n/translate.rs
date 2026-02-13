//! Translation functions and macros.

use std::collections::HashMap;
use std::fmt;

use gpui::SharedString;

use super::locale::Locale;
use super::runtime::TranslationMap;

/// Plural category for a specific count.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PluralCategory {
    /// Zero
    Zero,
    /// One (singular)
    One,
    /// Two
    Two,
    /// Few
    Few,
    /// Many
    Many,
    /// Other (default)
    Other,
}

impl PluralCategory {
    /// Get the plural category for a number in a specific locale.
    /// Uses simple rules based on language family.
    pub fn for_number(n: u64, locale: &Locale) -> Self {
        let lang = locale.language();

        // Use language-specific plural rules (simplified CLDR rules)
        match lang {
            // Arabic: 0=zero, 1=one, 2=two, 3-10=few, 11-99=many, rest=other
            "ar" => {
                if n == 0 {
                    PluralCategory::Zero
                } else if n == 1 {
                    PluralCategory::One
                } else if n == 2 {
                    PluralCategory::Two
                } else if (3..=10).contains(&n) {
                    PluralCategory::Few
                } else if (11..=99).contains(&n) {
                    PluralCategory::Many
                } else {
                    PluralCategory::Other
                }
            }

            // Chinese, Japanese, Korean, Vietnamese: no plural
            "zh" | "ja" | "ko" | "vi" | "th" => PluralCategory::Other,

            // French, Portuguese, Italian, Spanish: 0,1=one, rest=other
            "fr" | "pt" | "it" | "es" | "ca" | "gl" => {
                if n == 0 || n == 1 {
                    PluralCategory::One
                } else {
                    PluralCategory::Other
                }
            }

            // German, Dutch: 1=one, rest=other
            "de" | "nl" | "sv" | "da" | "no" | "fi" | "et" | "el" => {
                if n == 1 {
                    PluralCategory::One
                } else {
                    PluralCategory::Other
                }
            }

            // Polish: 1=one, 2-4=few, 5-21=other (simplified)
            "pl" | "cs" | "sk" | "sl" | "uk" | "ru" | "bg" | "sr" | "hr" => {
                if n == 1 {
                    PluralCategory::One
                } else if (2..=4).contains(&n) {
                    PluralCategory::Few
                } else {
                    PluralCategory::Other
                }
            }

            // Romanian: 1=one, 2-19=few if not 1 (simplified)
            "ro" | "hu" => {
                if n == 1 {
                    PluralCategory::One
                } else if (2..=19).contains(&n) {
                    PluralCategory::Few
                } else {
                    PluralCategory::Other
                }
            }

            // English: 1=one, rest=other
            "en" | "en-US" | "en-GB" => {
                if n == 1 {
                    PluralCategory::One
                } else {
                    PluralCategory::Other
                }
            }

            // Default fallback to English rules
            _ => {
                if n == 1 {
                    PluralCategory::One
                } else {
                    PluralCategory::Other
                }
            }
        }
    }
}

impl fmt::Display for PluralCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluralCategory::Zero => write!(f, "zero"),
            PluralCategory::One => write!(f, "one"),
            PluralCategory::Two => write!(f, "two"),
            PluralCategory::Few => write!(f, "few"),
            PluralCategory::Many => write!(f, "many"),
            PluralCategory::Other => write!(f, "other"),
        }
    }
}

/// Translation result that can handle plural forms.
pub struct TranslatedString {
    value: String,
}

impl TranslatedString {
    /// Create a new translated string.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Replace placeholders in the string.
    pub fn with_args(mut self, args: &HashMap<&str, impl fmt::Display>) -> Self {
        for (key, value) in args {
            let placeholder = format!("{{{}}}", key);
            self.value = self.value.replace(&placeholder, &value.to_string());
        }
        self
    }

    /// Convert to SharedString.
    pub fn into_shared(self) -> SharedString {
        self.value.into()
    }
}

impl From<String> for TranslatedString {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl<'a> From<&'a str> for TranslatedString {
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

/// Trait for translating strings.
pub trait Translator {
    /// Get a translation by key.
    fn t(&self, key: &str) -> Option<&str>;

    /// Get a translation with plural forms.
    fn tn(&self, key: &str, n: usize) -> Option<&str>;

    /// Get a translation with placeholders replaced.
    fn tf(&self, key: &str, args: &HashMap<&str, impl fmt::Display>) -> String;
}

impl Translator for TranslationMap {
    fn t(&self, key: &str) -> Option<&str> {
        self.get(key)
    }

    fn tn(&self, key: &str, n: usize) -> Option<&str> {
        // Try plural form first: key.one, key.few, key.many, key.other
        let category = PluralCategory::for_number(n as u64, &Locale::default());

        // Try specific category
        let plural_key = format!("{}.{}", key, category);
        if let Some(value) = self.get(&plural_key) {
            return Some(value);
        }

        // Fall back to "other" category
        let other_key = format!("{}.other", key);
        self.get(&other_key)
    }

    fn tf(&self, key: &str, args: &HashMap<&str, impl fmt::Display>) -> String {
        let base = self.t(key).unwrap_or(key);
        let mut result = base.to_string();

        for (key, value) in args {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, &value.to_string());
        }

        result
    }
}

/// Macro for translating strings.
///
/// # Example
/// ```
/// use yororen_ui::i18n::t;
///
/// let translated = t!("select.placeholder");
/// ```
#[macro_export]
macro_rules! t {
    ($key:expr) => {{
        // This macro is designed to be used within an App context
        // The actual implementation uses the App::t method
        $key
    }};
}

/// Macro for translating strings with arguments.
///
/// # Example
/// ```
/// use std::collections::HashMap;
/// use yororen_ui::i18n::t;
///
/// let mut args = HashMap::new();
/// args.insert("name", "World");
/// args.insert("count", 5);
/// let translated = tf!("items.count", &args);
/// ```
#[macro_export]
macro_rules! tf {
    ($key:expr, $args:expr) => {{
        $key
    }};
}

/// Macro for plural forms.
///
/// # Example
/// ```
/// use yororen_ui::i18n::tn;
///
/// let translated = tn!("items.count", n = 5);
/// ```
#[macro_export]
macro_rules! tn {
    ($key:expr, n = $n:expr) => {{
        $key
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plural_english() {
        let locale = Locale::new("en").unwrap();

        assert_eq!(PluralCategory::for_number(0, &locale), PluralCategory::Other);
        assert_eq!(PluralCategory::for_number(1, &locale), PluralCategory::One);
        assert_eq!(PluralCategory::for_number(2, &locale), PluralCategory::Other);
        assert_eq!(PluralCategory::for_number(5, &locale), PluralCategory::Other);
    }

    #[test]
    fn test_plural_arabic() {
        let locale = Locale::new("ar").unwrap();

        assert_eq!(PluralCategory::for_number(0, &locale), PluralCategory::Zero);
        assert_eq!(PluralCategory::for_number(1, &locale), PluralCategory::One);
        assert_eq!(PluralCategory::for_number(2, &locale), PluralCategory::Two);
        assert_eq!(PluralCategory::for_number(5, &locale), PluralCategory::Few);
    }

    #[test]
    fn test_translated_string_args() {
        let mut args: HashMap<&str, Box<dyn fmt::Display>> = HashMap::new();
        args.insert("name", Box::new("World"));
        args.insert("count", Box::new(5));

        let s = TranslatedString::new("Hello {name}, you have {count} items")
            .with_args(&args);

        assert_eq!(s.into_shared().to_string(), "Hello World, you have 5 items");
    }
}
