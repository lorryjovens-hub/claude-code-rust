//! Locales Module - Locale data loading

use super::{Language, Locale, loader::LocaleLoader};

/// Load locale data for a language
pub fn load_locale(language: Language) -> anyhow::Result<Locale> {
    LocaleLoader::load(language)
}

/// Get all available locale codes
pub fn available_locales() -> Vec<&'static str> {
    vec![
        "en", "zh", "ja", "es", "fr", "de", "ru", "pt", "it", "ko"
    ]
}

/// Check if a locale is available
pub fn is_locale_available(code: &str) -> bool {
    available_locales().contains(&code)
}
