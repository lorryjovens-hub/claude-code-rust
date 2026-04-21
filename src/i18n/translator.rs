//! Translator - Message translation engine

use super::{Language, Locale, locales};

/// Translator for internationalization
pub struct Translator {
    current_language: Language,
    current_locale: Locale,
    fallback_locale: Locale,
}

impl Translator {
    /// Create a new translator with the specified language
    pub fn new(lang_code: &str) -> anyhow::Result<Self> {
        let language = lang_code
            .parse::<Language>()
            .map_err(anyhow::Error::msg)?;
        let current_locale = locales::load_locale(language)?;
        let fallback_locale = locales::load_locale(Language::English)?;

        Ok(Self {
            current_language: language,
            current_locale,
            fallback_locale,
        })
    }

    /// Get the current language
    pub fn language(&self) -> Language {
        self.current_language
    }

    /// Get the current language code
    pub fn language_code(&self) -> &str {
        self.current_language.code()
    }

    /// Change the current language
    pub fn set_language(&mut self, lang_code: &str) -> anyhow::Result<()> {
        let language = lang_code
            .parse::<Language>()
            .map_err(anyhow::Error::msg)?;
        self.current_locale = locales::load_locale(language)?;
        self.current_language = language;
        Ok(())
    }

    /// Translate a message key
    pub fn translate(&self, key: &str) -> String {
        self.translate_with_args(key, &[])
    }

    /// Translate a message key with arguments
    pub fn translate_with_args(&self, key: &str, args: &[(&str, &str)]) -> String {
        // Try current locale first
        let message = self.current_locale.get(key)
            .or_else(|| self.fallback_locale.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string());

        // Replace placeholders with arguments
        let mut result = message;
        for (arg_key, arg_value) in args {
            result = result.replace(&format!("{{{}}}", arg_key), arg_value);
        }

        result
    }

    /// Translate a message and return a default if not found
    pub fn translate_or(&self, key: &str, default: &str) -> String {
        self.current_locale.get(key)
            .or_else(|| self.fallback_locale.get(key))
            .cloned()
            .unwrap_or_else(|| default.to_string())
    }

    /// Check if a key exists in the current locale
    pub fn has_key(&self, key: &str) -> bool {
        self.current_locale.get(key).is_some() || 
        self.fallback_locale.get(key).is_some()
    }

    /// Get all available keys
    pub fn keys(&self) -> Vec<&String> {
        let mut keys: Vec<&String> = self.current_locale.messages.keys().collect();
        keys.extend(self.fallback_locale.messages.keys());
        keys
    }

    /// Format a number according to locale
    pub fn format_number(&self, num: impl Into<f64>) -> String {
        let num = num.into();
        match self.current_language {
            Language::German | Language::French | Language::Russian | Language::Portuguese | Language::Italian => {
                // Use comma as decimal separator
                format!("{:.2}", num).replace('.', ",")
            }
            _ => format!("{:.2}", num),
        }
    }

    /// Format a date according to locale
    pub fn format_date(&self, date: &chrono::DateTime<chrono::Local>) -> String {
        match self.current_language {
            Language::English => date.format("%m/%d/%Y").to_string(),
            Language::Chinese => date.format("%Y年%m月%d日").to_string(),
            Language::Japanese => date.format("%Y年%m月%d日").to_string(),
            Language::German => date.format("%d.%m.%Y").to_string(),
            _ => date.format("%d/%m/%Y").to_string(),
        }
    }

    /// Format a datetime according to locale
    pub fn format_datetime(&self, datetime: &chrono::DateTime<chrono::Local>) -> String {
        match self.current_language {
            Language::English => datetime.format("%m/%d/%Y %I:%M %p").to_string(),
            Language::Chinese => datetime.format("%Y年%m月%d日 %H:%M").to_string(),
            Language::Japanese => datetime.format("%Y年%m月%d日 %H:%M").to_string(),
            Language::German => datetime.format("%d.%m.%Y %H:%M").to_string(),
            _ => datetime.format("%d/%m/%Y %H:%M").to_string(),
        }
    }

    /// Get the text direction (LTR or RTL)
    pub fn text_direction(&self) -> TextDirection {
        match self.current_language {
            // Add RTL languages here when supported
            _ => TextDirection::Ltr,
        }
    }

    /// Get plural form for a number
    pub fn plural_form(&self, count: i64) -> PluralForm {
        match self.current_language {
            Language::Chinese | Language::Japanese | Language::Korean => {
                // No plural forms
                PluralForm::Other
            }
            Language::English | Language::German | Language::Spanish | Language::Italian | Language::Portuguese => {
                if count == 1 {
                    PluralForm::One
                } else {
                    PluralForm::Other
                }
            }
            Language::French => {
                if count <= 1 {
                    PluralForm::One
                } else {
                    PluralForm::Other
                }
            }
            Language::Russian => {
                // Russian has more complex plural rules
                let rem100 = count % 100;
                let rem10 = count % 10;
                
                if rem10 == 1 && rem100 != 11 {
                    PluralForm::One
                } else if (2..=4).contains(&rem10) && !(12..=14).contains(&rem100) {
                    PluralForm::Few
                } else {
                    PluralForm::Many
                }
            }
        }
    }

    /// Translate with plural form
    pub fn translate_plural(&self, key_one: &str, key_other: &str, count: i64) -> String {
        let key = match self.plural_form(count) {
            PluralForm::One => key_one,
            _ => key_other,
        };
        
        self.translate_with_args(key, &[("count", &count.to_string())])
    }
}

/// Text direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextDirection {
    Ltr,
    Rtl,
}

/// Plural forms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluralForm {
    Zero,
    One,
    Two,
    Few,
    Many,
    Other,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate() {
        let translator = Translator::new("en").unwrap();
        assert_eq!(translator.translate("app.name"), "Claude Code");
    }

    #[test]
    fn test_translate_with_args() {
        let translator = Translator::new("en").unwrap();
        let result = translator.translate_with_args("welcome.user", &[("name", "Alice")]);
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_plural_forms() {
        let translator = Translator::new("en").unwrap();
        assert_eq!(translator.plural_form(1), PluralForm::One);
        assert_eq!(translator.plural_form(0), PluralForm::Other);
        assert_eq!(translator.plural_form(5), PluralForm::Other);
    }
}
