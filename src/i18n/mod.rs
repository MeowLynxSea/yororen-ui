//! Internationalization (i18n) module.
//!
//! This module provides internationalization support including:
//! - Translation key-value system with nested keys
//! - Runtime language switching
//! - Plural forms support (CLDR rules)
//! - Placeholder replacement
//! - Number and date/time formatting
//! - RTL (right-to-left) support预留
//!
//! # Usage
//!
//! ## Basic Translation
//!
//! ```ignore
//! use gpui::App;
//! use yororen_ui::i18n::I18nContext;
//!
//! // Get translated string
//! let text = cx.t("select.placeholder");
//! ```
//!
//! ## With Placeholders
//!
//! ```ignore
//! use std::collections::HashMap;
//! use gpui::App;
//! use yororen_ui::i18n::I18nContext;
//!
//! let mut args = HashMap::new();
//! args.insert("name", "World");
//! let text = cx.t_with_args("greeting", &args);
//! ```
//!
//! ## Plural Forms
//!
//! In your translation JSON file:
//! ```json
//! {
//!   "items": {
//!     "one": "{count} item",
//!     "other": "{count} items"
//!   }
//! }
//! ```
//!
//! ```ignore
//! use gpui::App;
//! use yororen_ui::i18n::I18nContext;
//!
//! let text = cx.tn("items", n = 5);
//! ```

pub mod defaults;
pub mod format;
pub mod loader;
pub mod locale;
pub mod runtime;
pub mod translate;

pub use format::{
    CurrencyDisplay, DateTimeFormatOptions, DateTimeFormatter, DateTimeLength, Formatter,
    I18nFormatter, NumberFormatOptions, NumberFormatter,
};
pub use loader::{
    EmbeddedLoader, FallbackLoader, FileLoader, LoadError, LocaleFiles, TranslationLoader,
};
pub use locale::{Locale, SupportedLocale, TextDirection};
pub use runtime::{I18n, I18nContext, Translate, TranslationMap};
pub use translate::{PluralCategory, TranslatedString, Translator};

// Re-export commonly used types
pub use locale::Locale as I18nLocale;
