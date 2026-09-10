//! `Theme` — the open key-value palette that drives the renderer layer.
//!
//! The v0.3 design decouples "what colors / sizes a renderer can
//! ask for" from "what shape those values take". `Theme` is
//! simply a wrapper around [`serde_json::Value`]; renderers
//! decide which paths to read (e.g. `"action.primary.bg"` or
//! `"tokens.control.button.radius"`) and fall back to their own
//! defaults if the path is absent. There is no fixed schema
//! and no compile-time guarantee that a given path exists —
//! the theme JSON file is the only source of truth.
//!
//! ```ignore
//! let json = include_str!("../themes/system-light.json");
//! let theme = Theme::from_json(json)?;
//! assert_eq!(theme.get_string("themeColor"), Some("#3b82f6"));
//! assert_eq!(theme.get_color("action.primary.bg"), Some(...));
//! ```

use gpui::Hsla;
use serde_json::Value;

mod global;
mod path;

pub use global::{ActiveTheme, GlobalTheme, install};
pub use path::value_to_hsla;

/// Open key-value palette.
///
/// `Theme(pub Value)` so callers can introspect / mutate the
/// underlying tree if they need to. The convenience methods
/// (`get`, `get_string`, `get_color`, …) cover the renderer
/// hot path; the raw `Value` is exposed for advanced use.
#[derive(Clone, Debug, Default)]
pub struct Theme(pub Value);

impl Theme {
    /// Empty theme. Equivalent to `Theme::from_value(Value::Null)`.
    pub fn new() -> Self {
        Theme(Value::Object(Default::default()))
    }

    /// Parse a theme from a JSON string. Any valid JSON shape
    /// is accepted — typically a top-level object, but arrays
    /// and scalars are tolerated and the renderers will simply
    /// find no paths.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        Ok(Theme(serde_json::from_str(s)?))
    }

    /// Wrap an existing `serde_json::Value`.
    pub fn from_value(v: Value) -> Self {
        Theme(v)
    }

    /// Borrow the underlying `serde_json::Value`.
    pub fn value(&self) -> &Value {
        &self.0
    }

    /// Walk a dot-separated path. Returns the leaf value or
    /// `None` if any segment is missing.
    ///
    /// ```ignore
    /// theme.get("action.primary.bg") // Some(&Value::String("#3b82f6".into()))
    /// theme.get("missing.path")      // None
    /// ```
    pub fn get(&self, path: &str) -> Option<&Value> {
        path::walk(&self.0, path)
    }

    /// Path → string. Returns `None` if the path is missing or
    /// the leaf is not a string.
    pub fn get_string(&self, path: &str) -> Option<&str> {
        self.get(path).and_then(Value::as_str)
    }

    /// Path → bool. Returns `None` if the path is missing or
    /// the leaf is not a bool.
    pub fn get_bool(&self, path: &str) -> Option<bool> {
        self.get(path).and_then(Value::as_bool)
    }

    /// Path → number. JSON numbers parse as `f64`; integer
    /// values come back as the same float.
    pub fn get_number(&self, path: &str) -> Option<f64> {
        self.get(path).and_then(Value::as_f64)
    }

    /// Path → color (`Hsla`). Accepts:
    ///
    /// - `"#rrggbb"` (alpha defaults to 1.0)
    /// - `"#rrggbbaa"`
    /// - `{"h": …, "s": …, "l": …, "a": …}`
    /// - `[h, s, l, a]`
    pub fn get_color(&self, path: &str) -> Option<Hsla> {
        self.get(path).and_then(value_to_hsla)
    }

    /// Path → `Value` with a typed `T` deserialization. Useful
    /// for theme packages that ship strongly-typed extensions
    /// alongside the open palette.
    pub fn get_typed<T: serde::de::DeserializeOwned>(&self, path: &str) -> Option<T> {
        self.get(path)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Insert / overwrite a value at `path`, creating
    /// intermediate objects as needed. Returns the previous
    /// value at that path, or `None` if it was newly created
    /// (or the parent path is a non-object and the insert
    /// failed).
    pub fn set(&mut self, path: &str, value: Value) -> Option<Value> {
        path::set(&mut self.0, path, value)
    }
}

impl From<Value> for Theme {
    fn from(v: Value) -> Self {
        Theme(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn empty_theme_has_no_paths() {
        let t = Theme::new();
        assert_eq!(t.get("anything"), None);
        assert_eq!(t.get_string("x"), None);
    }

    #[test]
    fn round_trip_json() {
        let json = r##"{"action":{"primary":{"bg":"#3b82f6"}}}"##;
        let t = Theme::from_json(json).unwrap();
        assert_eq!(t.get_string("action.primary.bg"), Some("#3b82f6"));
    }

    #[test]
    fn invalid_json_returns_err() {
        assert!(Theme::from_json("not json").is_err());
    }

    #[test]
    fn get_color_from_string_hex() {
        let t = Theme::from_value(json!({"x": "#ff0000"}));
        let c = t.get_color("x").unwrap();
        assert!(
            c.saturation > 0.5,
            "red should have high saturation, got {}",
            c.saturation
        );
    }

    #[test]
    fn get_number_int_or_float() {
        let t = Theme::from_value(json!({"a": 6, "b": 1.5}));
        assert_eq!(t.get_number("a"), Some(6.0));
        assert_eq!(t.get_number("b"), Some(1.5));
    }

    #[test]
    fn get_bool_strict() {
        let t = Theme::from_value(json!({"a": true, "b": 1}));
        assert_eq!(t.get_bool("a"), Some(true));
        assert_eq!(t.get_bool("b"), None); // 1 is not bool
        assert_eq!(t.get_bool("missing"), None);
    }

    #[test]
    fn set_then_get_round_trip() {
        let mut t = Theme::new();
        t.set("a.b.c", json!("hello"));
        assert_eq!(t.get_string("a.b.c"), Some("hello"));
    }

    #[test]
    fn get_typed_for_strongly_typed_extension() {
        #[derive(serde::Deserialize, PartialEq, Debug)]
        struct Pad {
            x: f64,
            y: f64,
        }
        let t = Theme::from_value(json!({"btn": {"x": 12.0, "y": 6.0}}));
        let pad: Pad = t.get_typed("btn").unwrap();
        assert_eq!(pad, Pad { x: 12.0, y: 6.0 });
    }
}
