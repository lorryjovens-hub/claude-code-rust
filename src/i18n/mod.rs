//! Internationalization (i18n) Module - Multi-language support
//!
//! This module provides localization support for Claude Code using
//! Mozilla's Fluent localization system.

pub mod translator;
pub mod loader;
pub mod locales;

pub use translator::Translator;
pub use loader::LocaleLoader;

use std::sync::OnceLock;

/// Global translator instance
static TRANSLATOR: OnceLock<Translator> = OnceLock::new();

/// Initialize the global translator
pub fn init(lang: &str) -> &'static Translator {
    TRANSLATOR.get_or_init(|| {
        Translator::new(lang).expect("Failed to initialize translator")
    })
}

/// Get the global translator instance
pub fn translator() -> &'static Translator {
    TRANSLATOR.get().expect("Translator not initialized")
}

/// Translate a message
pub fn t(key: &str) -> String {
    translator().translate(key)
}

/// Translate a message with arguments
pub fn t_args(key: &str, args: &[(&str, &str)]) -> String {
    translator().translate_with_args(key, args)
}

/// Supported languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Chinese,
    Japanese,
    Spanish,
    French,
    German,
    Russian,
    Portuguese,
    Italian,
    Korean,
}

impl Language {
    /// Get the language code
    pub fn code(&self) -> &'static str {
        match self {
            Language::English => "en",
            Language::Chinese => "zh",
            Language::Japanese => "ja",
            Language::Spanish => "es",
            Language::French => "fr",
            Language::German => "de",
            Language::Russian => "ru",
            Language::Portuguese => "pt",
            Language::Italian => "it",
            Language::Korean => "ko",
        }
    }

    /// Get the language name in its native form
    pub fn native_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "中文",
            Language::Japanese => "日本語",
            Language::Spanish => "Español",
            Language::French => "Français",
            Language::German => "Deutsch",
            Language::Russian => "Русский",
            Language::Portuguese => "Português",
            Language::Italian => "Italiano",
            Language::Korean => "한국어",
        }
    }

    /// Get the language name in English
    pub fn english_name(&self) -> &'static str {
        match self {
            Language::English => "English",
            Language::Chinese => "Chinese",
            Language::Japanese => "Japanese",
            Language::Spanish => "Spanish",
            Language::French => "French",
            Language::German => "German",
            Language::Russian => "Russian",
            Language::Portuguese => "Portuguese",
            Language::Italian => "Italian",
            Language::Korean => "Korean",
        }
    }

    /// Get all supported languages
    pub fn all() -> Vec<Language> {
        vec![
            Language::English,
            Language::Chinese,
            Language::Japanese,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Russian,
            Language::Portuguese,
            Language::Italian,
            Language::Korean,
        ]
    }
}

impl std::str::FromStr for Language {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "en" | "english" => Ok(Language::English),
            "zh" | "chinese" | "中文" => Ok(Language::Chinese),
            "ja" | "japanese" | "日本語" => Ok(Language::Japanese),
            "es" | "spanish" | "español" => Ok(Language::Spanish),
            "fr" | "french" | "français" => Ok(Language::French),
            "de" | "german" | "deutsch" => Ok(Language::German),
            "ru" | "russian" | "русский" => Ok(Language::Russian),
            "pt" | "portuguese" | "português" => Ok(Language::Portuguese),
            "it" | "italian" | "italiano" => Ok(Language::Italian),
            "ko" | "korean" | "한국어" => Ok(Language::Korean),
            _ => Err(format!("Unsupported language: {}", s)),
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code())
    }
}

/// Locale data for a specific language
#[derive(Debug, Clone)]
pub struct Locale {
    pub language: Language,
    pub messages: std::collections::HashMap<String, String>,
}

impl Locale {
    /// Create a new locale
    pub fn new(language: Language) -> Self {
        Self {
            language,
            messages: std::collections::HashMap::new(),
        }
    }

    /// Add a message
    pub fn add_message(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.messages.insert(key.into(), value.into());
    }

    /// Get a message
    pub fn get(&self, key: &str) -> Option<&String> {
        self.messages.get(key)
    }
}
