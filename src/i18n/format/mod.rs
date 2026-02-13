//! Number and date/time formatting.
//!
//! This module provides locale-aware formatting for numbers, currencies, percentages,
//! and dates/times.

use super::locale::Locale;

/// Number formatting options.
#[derive(Clone, Debug, Default)]
pub struct NumberFormatOptions {
    /// Minimum decimal places.
    pub min_fraction_digits: Option<usize>,
    /// Maximum decimal places.
    pub max_fraction_digits: Option<usize>,
    /// Whether to use grouping separators.
    pub use_grouping: bool,
    /// Currency code (e.g., "USD", "EUR").
    pub currency: Option<&'static str>,
    /// Currency display style.
    pub currency_display: super::CurrencyDisplay,
}

/// Currency display style.
#[derive(Clone, Debug, Default)]
pub enum CurrencyDisplay {
    #[default]
    Symbol,
    Code,
    Name,
}

/// Number formatter.
pub struct NumberFormatter {
    locale: Locale,
}

impl NumberFormatter {
    /// Create a new number formatter for a locale.
    pub fn new(locale: Locale) -> Self {
        Self { locale }
    }

    /// Format a number as a decimal.
    pub fn format_decimal(&self, value: f64) -> String {
        // Use basic formatting based on locale
        let lang = self.locale.language();

        // Check if locale uses grouping (most do except some Asian locales)
        let use_grouping = !matches!(lang, "ja" | "zh" | "ko");

        if use_grouping {
            let formatted = format!("{:.}", value);
            let parts: Vec<&str> = formatted.split('.').collect();
            let int_part = parts[0];
            let dec_part = parts.get(1);

            // Add grouping separators
            let int_formatted = add_grouping_separators(int_part, lang);

            if let Some(dec) = dec_part {
                format!("{}.{}", int_formatted, dec)
            } else {
                int_formatted
            }
        } else {
            if value.fract() == 0.0 {
                format!("{:.0}", value)
            } else {
                format!("{}", value)
            }
        }
    }

    /// Format a number with options.
    pub fn format_with_options(&self, value: f64, options: &NumberFormatOptions) -> String {
        let result = self.format_decimal(value);

        if let Some(currency) = options.currency {
            let symbol = match options.currency_display {
                CurrencyDisplay::Symbol => get_currency_symbol(currency).to_string(),
                CurrencyDisplay::Code => currency.to_string(),
                CurrencyDisplay::Name => get_currency_name(currency),
            };
            format!("{} {}", symbol, result)
        } else {
            result
        }
    }

    /// Format a number as currency.
    pub fn format_currency(&self, value: f64, currency: &'static str) -> String {
        let options = NumberFormatOptions {
            currency: Some(currency),
            currency_display: CurrencyDisplay::Symbol,
            ..Default::default()
        };
        self.format_with_options(value, &options)
    }

    /// Format a number as a percentage.
    pub fn format_percent(&self, value: f64) -> String {
        let percent = value * 100.0;
        format!("{}%", self.format_decimal(percent))
    }
}

/// Add thousand separators based on locale.
fn add_grouping_separators(s: &str, lang: &str) -> String {
    // Different locales use different group sizes
    let (group_size, separator) = match lang {
        "en" | "zh" => (3, ","),
        "de" | "es" | "fr" | "it" | "ru" => (3, " "),
        "hi" => (3, ","),
        _ => (3, ","),
    };

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    if len <= group_size {
        return s.to_string();
    }

    let mut result = String::new();
    let groups = (len + group_size - 1) / group_size;

    for i in 0..groups {
        let mut start = len - (i + 1) * group_size;
        let end = if i == 0 { len } else { len - i * group_size };
        if start > end {
            start = end;
        }

        let group: String = chars[start..end].iter().collect();

        if i == 0 {
            result = group;
        } else {
            result = format!("{}{}{}", separator, group, result);
        }
    }

    result
}

/// Get currency symbol.
fn get_currency_symbol(currency: &str) -> &str {
    match currency {
        "USD" => "$",
        "EUR" => "€",
        "GBP" => "£",
        "JPY" => "¥",
        "CNY" => "¥",
        "KRW" => "₩",
        "INR" => "₹",
        "RUB" => "₽",
        _ => currency,
    }
}

/// Get currency name (localized).
fn get_currency_name(currency: &str) -> String {
    match currency {
        "USD" => "US Dollar".to_string(),
        "EUR" => "Euro".to_string(),
        "GBP" => "British Pound".to_string(),
        "JPY" => "Japanese Yen".to_string(),
        "CNY" => "Chinese Yuan".to_string(),
        "KRW" => "Korean Won".to_string(),
        "INR" => "Indian Rupee".to_string(),
        "RUB" => "Russian Ruble".to_string(),
        _ => currency.to_string(),
    }
}

/// Date/time formatting options.
#[derive(Clone, Debug, Default)]
pub struct DateTimeFormatOptions {
    /// Date length.
    pub date_length: DateTimeLength,
    /// Time length.
    pub time_length: DateTimeLength,
}

/// Date/time length.
#[derive(Clone, Debug, Default)]
pub enum DateTimeLength {
    #[default]
    Short,
    Medium,
    Long,
    Full,
}

/// Date/time formatter.
pub struct DateTimeFormatter {
    locale: Locale,
}

impl DateTimeFormatter {
    /// Create a new date/time formatter for a locale.
    pub fn new(locale: Locale) -> Self {
        Self { locale }
    }

    /// Format a date (timestamp in seconds).
    pub fn format_date(&self, timestamp: i64) -> String {
        use chrono::{TimeZone, Utc};

        let datetime = Utc.timestamp_opt(timestamp, 0).single();
        if let Some(dt) = datetime {
            let lang = self.locale.language();

            // Format based on locale
            match lang {
                "en" => dt.format("%Y-%m-%d").to_string(),
                "zh" => format!("{}年{}月{}日", dt.format("%Y"), dt.format("%m"), dt.format("%d")),
                "ja" => format!("{}年{}月{}日", dt.format("%Y"), dt.format("%m"), dt.format("%d")),
                "ko" => format!("{}-{}-{}", dt.format("%Y"), dt.format("%m"), dt.format("%d")),
                "de" => dt.format("%d.%m.%Y").to_string(),
                "fr" => dt.format("%d/%m/%Y").to_string(),
                "es" => dt.format("%d/%m/%Y").to_string(),
                "ru" => dt.format("%d.%m.%Y").to_string(),
                "ar" => dt.format("%d/%m/%Y").to_string(), // RTL-aware display needed
                "he" => dt.format("%d/%m/%Y").to_string(),
                _ => dt.format("%Y-%m-%d").to_string(),
            }
        } else {
            "Invalid date".to_string()
        }
    }

    /// Format a time (timestamp in seconds).
    pub fn format_time(&self, timestamp: i64) -> String {
        use chrono::{TimeZone, Utc};

        let datetime = Utc.timestamp_opt(timestamp, 0).single();
        if let Some(dt) = datetime {
            let lang = self.locale.language();

            // Some locales use 12-hour format
            match lang {
                "en" | "ko" | "zh" | "ja" => dt.format("%H:%M").to_string(),
                _ => dt.format("%H:%M").to_string(),
            }
        } else {
            "Invalid time".to_string()
        }
    }

    /// Format a date and time.
    pub fn format_datetime(&self, timestamp: i64) -> String {
        format!(
            "{} {}",
            self.format_date(timestamp),
            self.format_time(timestamp)
        )
    }
}

/// Combined formatter for both numbers and date/time.
pub struct Formatter {
    _locale: Locale,
    number: NumberFormatter,
    datetime: DateTimeFormatter,
}

impl Formatter {
    /// Create a new formatter for a locale.
    pub fn new(locale: Locale) -> Self {
        Self {
            _locale: locale.clone(),
            number: NumberFormatter::new(locale.clone()),
            datetime: DateTimeFormatter::new(locale),
        }
    }

    /// Get the number formatter.
    pub fn number(&self) -> &NumberFormatter {
        &self.number
    }

    /// Get the date/time formatter.
    pub fn datetime(&self) -> &DateTimeFormatter {
        &self.datetime
    }

    /// Format a number.
    pub fn format_number(&self, value: f64) -> String {
        self.number.format_decimal(value)
    }

    /// Format a currency value.
    pub fn format_currency(&self, value: f64, currency: &'static str) -> String {
        self.number.format_currency(value, currency)
    }

    /// Format a percentage.
    pub fn format_percent(&self, value: f64) -> String {
        self.number.format_percent(value)
    }

    /// Format a date.
    pub fn format_date(&self, timestamp: i64) -> String {
        self.datetime.format_date(timestamp)
    }

    /// Format a time.
    pub fn format_time(&self, timestamp: i64) -> String {
        self.datetime.format_time(timestamp)
    }

    /// Format a date and time.
    pub fn format_datetime(&self, timestamp: i64) -> String {
        self.datetime.format_datetime(timestamp)
    }
}

/// Helper to add formatting to I18n.
pub trait I18nFormatter {
    fn formatter(&self) -> Formatter;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_format() {
        let formatter = NumberFormatter::new(Locale::new("en").unwrap());

        assert_eq!(formatter.format_decimal(1000.0), "1,000");
        assert_eq!(formatter.format_decimal(1000000.0), "1,000,000");
        assert_eq!(formatter.format_decimal(100.5), "100.5");
    }

    #[test]
    fn test_currency_format() {
        let formatter = NumberFormatter::new(Locale::new("en").unwrap());

        assert_eq!(formatter.format_currency(100.50, "USD"), "$ 100.5");
        assert_eq!(formatter.format_currency(1000.0, "EUR"), "€ 1,000");
    }

    #[test]
    fn test_date_format() {
        let formatter = DateTimeFormatter::new(Locale::new("en").unwrap());
        // Use a known timestamp: 2024-01-01 00:00:00 UTC
        let timestamp = 1704067200;
        let date = formatter.format_date(timestamp);
        assert!(date.contains("2024"));
    }
}
