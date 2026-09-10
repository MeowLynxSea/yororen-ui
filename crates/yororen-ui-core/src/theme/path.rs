//! Dot-path walker for `serde_json::Value`.
//!
//! `Theme::get("a.b.c")` walks through `Value::Object` segments
//! separated by `.` and returns the leaf, or `None` if any
//! segment is missing (or the parent is not an object).
//!
//! The walker is deliberately minimal: no array indexing, no
//! wildcards, no `..` ranges. Renderers consume theme data
//! through these getters, and the format is "path keys into a
//! JSON object tree" — anything more elaborate would force
//! schema decisions onto the value side, which is exactly the
//! coupling the v0.3 split removes.

use serde_json::{Map, Value};

use gpui::Hsla;

/// Walk `root` following the dot-separated `path`. Returns
/// `None` if any segment is missing or hits a non-object. An
/// empty path returns the root.
pub fn walk<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    if path.is_empty() {
        return Some(root);
    }
    let mut current = root;
    for segment in path.split('.') {
        let obj = current.as_object()?;
        current = obj.get(segment)?;
    }
    Some(current)
}

/// Insert (or overwrite) `value` at `path`, creating
/// intermediate `Value::Object` segments as needed. Returns
/// the previous value at that path, or `None` if the path was
/// newly created.
///
/// Fails (returns `None` and leaves the tree unchanged) if any
/// intermediate segment is a non-object — you cannot store a
/// child of a string, for instance.
pub fn set(root: &mut Value, path: &str, value: Value) -> Option<Value> {
    let segments: Vec<&str> = path.split('.').collect();
    if segments.is_empty() {
        // Setting "" replaces the root; the previous root is
        // returned so callers can roll back if they wish.
        return Some(std::mem::replace(root, value));
    }
    set_recursive(root, &segments, value)
}

fn set_recursive(node: &mut Value, segments: &[&str], value: Value) -> Option<Value> {
    if segments.len() == 1 {
        let map = node.as_object_mut()?;
        return map.insert(segments[0].to_string(), value);
    }
    let map = node.as_object_mut()?;
    let entry = map
        .entry(segments[0].to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    set_recursive(entry, &segments[1..], value)
}

/// Parse a color value into `Hsla`. Accepts:
///
/// - `"#rrggbb"` — 6-digit hex (alpha defaults to 1.0)
/// - `"#rrggbbaa"` — 8-digit hex
/// - `{"h": 0.0, "s": 0.0, "l": 0.0, "a": 1.0}` — HSLA object
/// - `[0.0, 0.0, 0.0, 1.0]` — HSLA array of 4 floats
///
/// Returns `None` for any other shape. The sRGB → HSL
/// conversion follows the standard formula.
pub fn value_to_hsla(v: &Value) -> Option<Hsla> {
    if let Some(s) = v.as_str() {
        return parse_hex_color(s);
    }
    if let Some(obj) = v.as_object() {
        let h = obj.get("h").and_then(Value::as_f64)? as f32;
        let s = obj.get("s").and_then(Value::as_f64)? as f32;
        let l = obj.get("l").and_then(Value::as_f64)? as f32;
        let a = obj.get("a").and_then(Value::as_f64).unwrap_or(1.0) as f32;
        return Some(gpui::hsla(h, s, l, a));
    }
    if let Some(arr) = v.as_array()
        && arr.len() == 4
    {
        let h = arr[0].as_f64()? as f32;
        let s = arr[1].as_f64()? as f32;
        let l = arr[2].as_f64()? as f32;
        let a = arr[3].as_f64()? as f32;
        return Some(gpui::hsla(h, s, l, a));
    }
    None
}

fn parse_hex_color(s: &str) -> Option<Hsla> {
    let s = s.strip_prefix('#')?;
    let (r, g, b, a) = match s.len() {
        6 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            (r, g, b, 255u8)
        }
        8 => {
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            let a = u8::from_str_radix(&s[6..8], 16).ok()?;
            (r, g, b, a)
        }
        _ => return None,
    };
    let rgb = gpui::Rgba::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    );
    Some(gpui::rgb_to_hsla(rgb))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn walk_simple_path() {
        let v = json!({"a": {"b": {"c": 42}}});
        assert_eq!(walk(&v, "a.b.c"), Some(&json!(42)));
    }

    #[test]
    fn walk_missing_segment_returns_none() {
        let v = json!({"a": {"b": 1}});
        assert_eq!(walk(&v, "a.x.c"), None);
    }

    #[test]
    fn walk_into_non_object_returns_none() {
        let v = json!({"a": "string"});
        assert_eq!(walk(&v, "a.b"), None);
    }

    #[test]
    fn walk_empty_path_returns_root() {
        let v = json!({"a": 1});
        assert_eq!(walk(&v, ""), Some(&v));
    }

    #[test]
    fn set_creates_intermediate_objects() {
        let mut v = json!({});
        let prev = set(&mut v, "a.b.c", json!(42));
        assert_eq!(prev, None);
        assert_eq!(v, json!({"a": {"b": {"c": 42}}}));
    }

    #[test]
    fn set_overwrites_existing_value() {
        let mut v = json!({"a": {"b": 1}});
        let prev = set(&mut v, "a.b", json!(2));
        assert_eq!(prev, Some(json!(1)));
        assert_eq!(v, json!({"a": {"b": 2}}));
    }

    #[test]
    fn set_fails_when_parent_is_not_object() {
        let mut v = json!({"a": "string"});
        let prev = set(&mut v, "a.b", json!(1));
        assert_eq!(prev, None);
        assert_eq!(v, json!({"a": "string"}));
    }

    #[test]
    fn parse_hex_6_digit() {
        let c = parse_hex_color("#ff0000").unwrap();
        // red: hue 0, sat 1, lightness 0.5, alpha 1
        assert!(c.saturation > 0.5, "expected high saturation, got {}", c.saturation);
        assert!(
            c.lightness > 0.4 && c.lightness < 0.6,
            "expected mid lightness, got {}",
            c.lightness
        );
        assert!(c.alpha > 0.99);
    }

    #[test]
    fn parse_hex_8_digit_with_alpha() {
        let c = parse_hex_color("#00ff0080").unwrap();
        // alpha should be ~0.5
        assert!(c.alpha > 0.49 && c.alpha < 0.51, "alpha {}", c.alpha);
    }

    #[test]
    fn parse_hex_invalid_returns_none() {
        assert_eq!(parse_hex_color("ff0000"), None);
        assert_eq!(parse_hex_color("#xyz"), None);
        assert_eq!(parse_hex_color("#fff"), None); // only 3-digit not supported
    }

    #[test]
    fn value_to_hsla_from_object() {
        let v = json!({"h": 0.5, "s": 0.5, "l": 0.5, "a": 1.0});
        let c = value_to_hsla(&v).unwrap();
        assert!((c.color.hue.into_degrees() / 360.0 - 0.5).abs() < 1e-6);
        assert_eq!(c.saturation, 0.5);
        assert_eq!(c.lightness, 0.5);
        assert_eq!(c.alpha, 1.0);
    }

    #[test]
    fn value_to_hsla_from_array() {
        let v = json!([0.1, 0.2, 0.3, 0.4]);
        let c = value_to_hsla(&v).unwrap();
        assert!((c.color.hue.into_degrees() / 360.0 - 0.1).abs() < 1e-5);
        assert!((c.saturation - 0.2).abs() < 1e-6);
        assert!((c.lightness - 0.3).abs() < 1e-6);
        assert!((c.alpha - 0.4).abs() < 1e-6);
    }

    #[test]
    fn value_to_hsla_rejects_invalid() {
        assert_eq!(value_to_hsla(&json!("not-a-color")), None);
        assert_eq!(value_to_hsla(&json!(42)), None);
        assert_eq!(value_to_hsla(&json!([1, 2])), None); // wrong length
    }
}
