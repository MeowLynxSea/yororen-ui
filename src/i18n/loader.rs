//! Translation resource loader for loading JSON locale files.

use std::fmt;
use std::path::Path;

use rust_embed::RustEmbed;

use super::locale::Locale;
use super::runtime::TranslationMap;

/// Error type for translation loading.
#[derive(Debug)]
pub enum LoadError {
    /// Failed to parse JSON.
    ParseError(String),
    /// Locale not found.
    LocaleNotFound(String),
    /// Embed error.
    EmbedError(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            LoadError::LocaleNotFound(locale) => write!(f, "Locale not found: {}", locale),
            LoadError::EmbedError(msg) => write!(f, "Embed error: {}", msg),
        }
    }
}

impl std::error::Error for LoadError {}

/// Trait for loading translation resources.
pub trait TranslationLoader {
    /// Load translations for a specific locale.
    fn load(&self, locale: &Locale) -> Result<TranslationMap, LoadError>;

    /// Check if a locale is available.
    fn is_available(&self, locale: &Locale) -> bool;
}

/// Embedded locale files.
#[derive(RustEmbed)]
#[folder = "locales/"]
pub struct LocaleFiles;

/// Loader that loads translations from embedded JSON files.
pub struct EmbeddedLoader;

impl EmbeddedLoader {
    /// Create a new embedded loader.
    pub fn new() -> Self {
        Self
    }

    /// Get the filename for a locale.
    fn filename_for_locale(locale: &Locale) -> String {
        let lang = locale.language();
        if let Some(region) = locale.region() {
            format!("{}-{}.json", lang, region)
        } else {
            format!("{}.json", lang)
        }
    }
}

impl Default for EmbeddedLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationLoader for EmbeddedLoader {
    fn load(&self, locale: &Locale) -> Result<TranslationMap, LoadError> {
        let filename = Self::filename_for_locale(locale);

        let file = LocaleFiles::get(&filename)
            .ok_or_else(|| LoadError::LocaleNotFound(filename.clone()))?;

        let content = file.data.into_owned();

        let json_str = std::str::from_utf8(&content)
            .map_err(|e| LoadError::EmbedError(e.to_string()))?;

        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| LoadError::ParseError(e.to_string()))?;

        parse_json_to_translation_map(value)
    }

    fn is_available(&self, locale: &Locale) -> bool {
        let filename = Self::filename_for_locale(locale);
        LocaleFiles::get(&filename).is_some()
    }
}

/// Parse JSON value into a translation map.
fn parse_json_to_translation_map(value: serde_json::Value) -> Result<TranslationMap, LoadError> {
    let mut map = TranslationMap::new();

    match value {
        serde_json::Value::Object(obj) => {
            for (key, val) in obj {
                match val {
                    serde_json::Value::String(s) => {
                        map.insert(&key, &s);
                    }
                    serde_json::Value::Object(nested) => {
                        let nested_map = parse_json_object_to_map(nested)?;
                        map.insert_nested(&key, nested_map);
                    }
                    _ => {
                        // Skip non-string, non-object values
                    }
                }
            }
        }
        _ => {
            return Err(LoadError::ParseError("Expected JSON object".to_string()));
        }
    }

    Ok(map)
}

/// Parse a JSON object to a translation map.
fn parse_json_object_to_map(
    obj: serde_json::Map<String, serde_json::Value>,
) -> Result<TranslationMap, LoadError> {
    let mut map = TranslationMap::new();

    for (key, val) in obj {
        match val {
            serde_json::Value::String(s) => {
                map.insert(&key, &s);
            }
            serde_json::Value::Object(nested) => {
                let nested_map = parse_json_object_to_map(nested)?;
                map.insert_nested(&key, nested_map);
            }
            _ => {}
        }
    }

    Ok(map)
}

/// Loader that loads translations from external files.
pub struct FileLoader {
    base_path: String,
}

impl FileLoader {
    /// Create a new file loader with the given base path.
    pub fn new(base_path: impl Into<String>) -> Self {
        Self {
            base_path: base_path.into(),
        }
    }
}

impl TranslationLoader for FileLoader {
    fn load(&self, locale: &Locale) -> Result<TranslationMap, LoadError> {
        let filename = EmbeddedLoader::filename_for_locale(locale);
        let path = Path::new(&self.base_path).join(&filename);

        let content = std::fs::read_to_string(&path)
            .map_err(|e| LoadError::EmbedError(format!("Failed to read file: {}", e)))?;

        let value: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| LoadError::ParseError(e.to_string()))?;

        parse_json_to_translation_map(value)
    }

    fn is_available(&self, locale: &Locale) -> bool {
        let filename = EmbeddedLoader::filename_for_locale(locale);
        let path = Path::new(&self.base_path).join(&filename);
        path.exists()
    }
}

/// Create a loader that tries embedded first, then falls back to file system.
pub struct FallbackLoader {
    embedded: EmbeddedLoader,
    file: Option<FileLoader>,
}

impl FallbackLoader {
    /// Create a new fallback loader.
    pub fn new(file_base_path: Option<impl Into<String>>) -> Self {
        Self {
            embedded: EmbeddedLoader::new(),
            file: file_base_path.map(FileLoader::new),
        }
    }
}

impl TranslationLoader for FallbackLoader {
    fn load(&self, locale: &Locale) -> Result<TranslationMap, LoadError> {
        // Try embedded first
        if self.embedded.is_available(locale) {
            return self.embedded.load(locale);
        }

        // Fall back to file system
        if let Some(ref file) = self.file && file.is_available(locale) {
            return file.load(locale);
        }

        Err(LoadError::LocaleNotFound(locale.to_string()))
    }

    fn is_available(&self, locale: &Locale) -> bool {
        self.embedded.is_available(locale) || self.file.as_ref().is_some_and(|f| f.is_available(locale))
    }
}
