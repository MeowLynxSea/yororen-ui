use std::path::Path;

use serde_json::Value;
use yororen_ui::i18n::{I18n, LoadError, Locale, TranslationMap};

pub fn load_demo_i18n(locale: Locale) -> Result<I18n, LoadError> {
    let mut i18n = I18n::with_embedded(locale.clone());
    let map = load_demo_translation_map(&locale)?;
    // Merge demo translations on top of core translations.
    // This prevents demo-only maps from accidentally overriding core keys like `common.save`.
    i18n.merge_translations(locale, map);
    Ok(i18n)
}

fn load_demo_translation_map(locale: &Locale) -> Result<TranslationMap, LoadError> {
    let filename = if let Some(region) = locale.region() {
        format!("{}-{}.json", locale.language(), region)
    } else {
        format!("{}.json", locale.language())
    };

    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("locales")
        .join(filename);

    let content = std::fs::read_to_string(&path)
        .map_err(|e| LoadError::EmbedError(format!("Failed to read demo locale file {path:?}: {e}")))?;

    let value: Value =
        serde_json::from_str(&content).map_err(|e| LoadError::ParseError(e.to_string()))?;

    parse_json_to_translation_map(value)
}

fn parse_json_to_translation_map(value: Value) -> Result<TranslationMap, LoadError> {
    let mut map = TranslationMap::new();

    match value {
        Value::Object(obj) => {
            for (key, val) in obj {
                match val {
                    Value::String(s) => map.insert(&key, &s),
                    Value::Object(nested) => {
                        let nested_map = parse_json_object_to_map(nested)?;
                        map.insert_nested(&key, nested_map);
                    }
                    _ => {}
                }
            }
        }
        _ => return Err(LoadError::ParseError("Expected JSON object".to_string())),
    }

    Ok(map)
}

fn parse_json_object_to_map(
    obj: serde_json::Map<String, Value>,
) -> Result<TranslationMap, LoadError> {
    let mut map = TranslationMap::new();

    for (key, val) in obj {
        match val {
            Value::String(s) => map.insert(&key, &s),
            Value::Object(nested) => {
                let nested_map = parse_json_object_to_map(nested)?;
                map.insert_nested(&key, nested_map);
            }
            _ => {}
        }
    }

    Ok(map)
}
