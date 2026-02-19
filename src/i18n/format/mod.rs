//! Number and date/time formatting.
//!
//! This module provides locale-aware formatting for numbers, currencies, percentages,
//! and dates/times.

use super::locale::Locale;

use std::borrow::Cow;

/// Locale-specific separators and digit shape.
#[derive(Clone, Copy, Debug)]
struct NumberSymbols {
    decimal: char,
    group: char,
    minus: char,
    /// Whether to use Arabic-Indic digits (٠١٢٣٤٥٦٧٨٩).
    use_arabic_indic_digits: bool,
}

impl NumberSymbols {
    fn for_locale(locale: &Locale) -> Self {
        match locale.language() {
            // Arabic: Arabic-Indic digits + Arabic separators.
            // Note: Decimal separator in Arabic locales is typically "٫" (U+066B)
            // and group separator is "٬" (U+066C).
            "ar" => Self {
                decimal: '٫',
                group: '٬',
                minus: '−',
                use_arabic_indic_digits: true,
            },
            // French family commonly uses comma for decimals and space (or NBSP) for grouping.
            "fr" | "de" | "es" | "it" | "ru" => Self {
                decimal: ',',
                group: ' ',
                minus: '−',
                use_arabic_indic_digits: false,
            },
            _ => Self {
                decimal: '.',
                group: ',',
                minus: '-',
                use_arabic_indic_digits: false,
            },
        }
    }
}

/// Number formatting options.
#[derive(Clone, Debug)]
pub struct NumberFormatOptions {
    /// Minimum decimal places.
    pub min_fraction_digits: Option<usize>,
    /// Maximum decimal places.
    pub max_fraction_digits: Option<usize>,
    /// Whether to use grouping separators.
    pub use_grouping: bool,
    /// Currency code (e.g., "USD", "EUR").
    pub currency: Option<&'static str>,
    /// Put the currency symbol/code after the number.
    ///
    /// If `None`, a locale-aware default is used.
    pub currency_as_suffix: Option<bool>,
    /// Currency display style.
    pub currency_display: super::CurrencyDisplay,
}

impl Default for NumberFormatOptions {
    fn default() -> Self {
        Self {
            min_fraction_digits: None,
            max_fraction_digits: None,
            use_grouping: true,
            currency: None,
            currency_as_suffix: None,
            currency_display: super::CurrencyDisplay::default(),
        }
    }
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
        self.format_decimal_with_options(value, &NumberFormatOptions::default())
    }

    fn format_decimal_with_options(&self, value: f64, options: &NumberFormatOptions) -> String {
        let symbols = NumberSymbols::for_locale(&self.locale);
        let lang = self.locale.language();
        let use_grouping = options.use_grouping
            && !matches!(lang, "ja" | "zh" | "ko")
            && value.is_finite();

        if value.is_nan() {
            return "NaN".to_string();
        }
        if value.is_infinite() {
            if value.is_sign_negative() {
                return format!("{}∞", symbols.minus);
            }
            return "∞".to_string();
        }

        let mut value = value;
        let mut sign = "";
        if value.is_sign_negative() {
            sign = "-";
            value = -value;
        }

        // Determine fraction digits.
        // If not specified, keep Rust's default formatting for non-integers.
        let formatted = match (options.min_fraction_digits, options.max_fraction_digits) {
            (Some(min), Some(max)) => {
                let max = max.max(min);
                let mut s = format!("{value:.max$}");
                if max > min {
                    // Trim trailing zeros but keep at least min digits.
                    if let Some(dot) = s.find('.') {
                        let mut frac = s[dot + 1..].to_string();
                        while frac.len() > min && frac.ends_with('0') {
                            frac.pop();
                        }
                        if frac.is_empty() {
                            s.truncate(dot);
                        } else {
                            s.truncate(dot + 1);
                            s.push_str(&frac);
                        }
                    }
                }
                s
            }
            (None, Some(max)) => format!("{value:.max$}"),
            (Some(min), None) => {
                if value.fract() == 0.0 {
                    format!("{value:.0}")
                } else {
                    let mut s = format!("{value}");
                    // Ensure at least min digits by padding if necessary.
                    if let Some(dot) = s.find('.') {
                        let current = s.len() - (dot + 1);
                        if current < min {
                            s.push_str(&"0".repeat(min - current));
                        }
                    } else {
                        s.push(symbols.decimal);
                        s.push_str(&"0".repeat(min));
                    }
                    s
                }
            }
            (None, None) => {
                if value.fract() == 0.0 {
                    format!("{value:.0}")
                } else {
                    format!("{value}")
                }
            }
        };

        let (int_part, frac_part) = formatted
            .split_once('.')
            .map(|(a, b)| (a, Some(b)))
            .unwrap_or((formatted.as_str(), None));

        let int_part = if use_grouping {
            add_grouping_separators(int_part, symbols.group)
        } else {
            int_part.to_string()
        };

        let mut out = String::new();
        if sign == "-" {
            out.push(symbols.minus);
        }
        out.push_str(&int_part);
        if let Some(frac) = frac_part {
            out.push(symbols.decimal);
            out.push_str(frac);
        }

        if symbols.use_arabic_indic_digits {
            out = latin_to_arabic_indic_digits(&out);
        }

        out
    }

    /// Format a number with options.
    pub fn format_with_options(&self, value: f64, options: &NumberFormatOptions) -> String {
        let result = self.format_decimal_with_options(value, options);
        let Some(currency) = options.currency else {
            return result;
        };

        let symbol = match options.currency_display {
            CurrencyDisplay::Symbol => get_currency_symbol(currency, &self.locale).to_string(),
            CurrencyDisplay::Code => currency.to_string(),
            CurrencyDisplay::Name => get_currency_name(currency, &self.locale),
        };

        let as_suffix = options
            .currency_as_suffix
            .unwrap_or_else(|| currency_should_be_suffix(&self.locale));

        if as_suffix {
            format!("{result} {symbol}")
        } else {
            format!("{symbol} {result}")
        }
    }

    /// Format a number as currency.
    pub fn format_currency(&self, value: f64, currency: &'static str) -> String {
        let options = NumberFormatOptions {
            currency: Some(currency),
            currency_display: CurrencyDisplay::Symbol,
            use_grouping: true,
            // Pragmatic default: many currencies usually display 2 fraction digits, except
            // some zero-decimal currencies (JPY/KRW).
            min_fraction_digits: Some(currency_default_fraction_digits(currency)),
            max_fraction_digits: Some(currency_default_fraction_digits(currency)),
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
fn add_grouping_separators(s: &str, separator: char) -> String {
    let group_size = 3;

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();

    if len <= group_size {
        return s.to_string();
    }

    // Build from the end to avoid underflow.
    let mut result = String::new();
    let mut remaining = len;

    while remaining > 0 {
        let start = remaining.saturating_sub(group_size);
        let group: String = chars[start..remaining].iter().collect();

        if result.is_empty() {
            result = group;
        } else {
            result = format!("{}{}{}", group, separator, result);
        }

        remaining = start;
    }

    result
}

fn latin_to_arabic_indic_digits(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '0' => '٠',
            '1' => '١',
            '2' => '٢',
            '3' => '٣',
            '4' => '٤',
            '5' => '٥',
            '6' => '٦',
            '7' => '٧',
            '8' => '٨',
            '9' => '٩',
            _ => c,
        })
        .collect()
}

fn currency_should_be_suffix(locale: &Locale) -> bool {
    // A small pragmatic default: English often prefixes, many European languages suffix.
    match locale.language() {
        "fr" | "de" | "es" | "it" | "ru" => true,
        // Arabic commonly uses suffix in many contexts (e.g. ١٠٠ ر.س), but varies.
        "ar" => true,
        _ => false,
    }
}

fn currency_default_fraction_digits(currency: &str) -> usize {
    match currency {
        "JPY" | "KRW" => 0,
        _ => 2,
    }
}

/// Get currency symbol.
fn get_currency_symbol<'a>(currency: &'a str, locale: &Locale) -> Cow<'a, str> {
    match currency {
        "USD" => Cow::Borrowed("$"),
        "EUR" => Cow::Borrowed("€"),
        "GBP" => Cow::Borrowed("£"),
        "JPY" => Cow::Borrowed("¥"),
        "CNY" => Cow::Borrowed("¥"),
        "KRW" => Cow::Borrowed("₩"),
        "INR" => Cow::Borrowed("₹"),
        "RUB" => Cow::Borrowed("₽"),
        "SAR" => {
            if locale.language() == "ar" {
                Cow::Borrowed("ر.س")
            } else {
                Cow::Borrowed("SAR")
            }
        }
        _ => Cow::Borrowed(currency),
    }
}

/// Get currency name (localized).
fn get_currency_name(currency: &str, locale: &Locale) -> String {
    match currency {
        "USD" => match locale.language() {
            "zh" => "美元".to_string(),
            "ar" => "دولار أمريكي".to_string(),
            _ => "US Dollar".to_string(),
        },
        "EUR" => match locale.language() {
            "zh" => "欧元".to_string(),
            "ar" => "يورو".to_string(),
            _ => "Euro".to_string(),
        },
        "GBP" => match locale.language() {
            "zh" => "英镑".to_string(),
            "ar" => "جنيه إسترليني".to_string(),
            _ => "British Pound".to_string(),
        },
        "JPY" => match locale.language() {
            "zh" => "日元".to_string(),
            "ar" => "ين ياباني".to_string(),
            _ => "Japanese Yen".to_string(),
        },
        "CNY" => match locale.language() {
            "zh" => "人民币".to_string(),
            "ar" => "يوان صيني".to_string(),
            _ => "Chinese Yuan".to_string(),
        },
        "KRW" => match locale.language() {
            "zh" => "韩元".to_string(),
            "ar" => "وون كوري".to_string(),
            _ => "Korean Won".to_string(),
        },
        "INR" => match locale.language() {
            "zh" => "印度卢比".to_string(),
            "ar" => "روبية هندية".to_string(),
            _ => "Indian Rupee".to_string(),
        },
        "RUB" => match locale.language() {
            "zh" => "俄罗斯卢布".to_string(),
            "ar" => "روبل روسي".to_string(),
            _ => "Russian Ruble".to_string(),
        },
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
                "zh" => format!(
                    "{}年{}月{}日",
                    dt.format("%Y"),
                    dt.format("%m"),
                    dt.format("%d")
                ),
                "ja" => format!(
                    "{}年{}月{}日",
                    dt.format("%Y"),
                    dt.format("%m"),
                    dt.format("%d")
                ),
                "ko" => format!(
                    "{}-{}-{}",
                    dt.format("%Y"),
                    dt.format("%m"),
                    dt.format("%d")
                ),
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

        assert_eq!(formatter.format_currency(100.50, "USD"), "$ 100.50");
        assert_eq!(formatter.format_currency(1000.0, "EUR"), "€ 1,000.00");
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
