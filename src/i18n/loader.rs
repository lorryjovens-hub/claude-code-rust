//! Locale Loader - Loads locale data from embedded resources

use super::{Language, Locale};
use regex::Regex;
use rust_embed::Embed;
use std::sync::OnceLock;

/// Embedded locale files
#[derive(Embed)]
#[folder = "locales/"]
struct LocaleAssets;

/// Loader for locale data
pub struct LocaleLoader;

impl LocaleLoader {
    /// Load locale data for a language
    pub fn load(language: Language) -> anyhow::Result<Locale> {
        let mut locale = Locale::new(language);

        // Load embedded locale data if present.
        let file_name = format!("{}.ftl", language.code());
        if let Some(content) = LocaleAssets::get(&file_name) {
            let content = std::str::from_utf8(&content.data)?;
            Self::parse_ftl(content, &mut locale)?;
        }

        // Fall back to built-in messages when no embedded locale exists.
        if locale.messages.is_empty() {
            Self::load_builtin(language, &mut locale)?;
        }

        Ok(locale)
    }

    /// Parse Fluent FTL format.
    fn parse_ftl(content: &str, locale: &mut Locale) -> anyhow::Result<()> {
        for line in content.lines() {
            let line = line.trim();

            // Skip comments and empty lines
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse key = value
            if let Some(pos) = line.find('=') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();
                if key.is_empty() {
                    continue;
                }

                // Keep native FTL key style and a dot-notation alias.
                let normalized_value = Self::normalize_placeholders(value);
                locale.add_message(key, normalized_value.clone());

                let alias_key = key.replace('-', ".");
                if alias_key != key {
                    locale.add_message(alias_key, normalized_value);
                }
            }
        }

        Ok(())
    }

    /// Convert Fluent-style placeholders `{ $name }` to internal `{name}` format.
    fn normalize_placeholders(value: &str) -> String {
        static PLACEHOLDER_RE: OnceLock<Regex> = OnceLock::new();
        let regex = PLACEHOLDER_RE.get_or_init(|| {
            Regex::new(r"\{\s*\$([A-Za-z0-9_]+)\s*\}")
                .expect("placeholder regex must be valid")
        });

        regex.replace_all(value, "{$1}").into_owned()
    }

    /// Load built-in locale data
    fn load_builtin(language: Language, locale: &mut Locale) -> anyhow::Result<()> {
        match language {
            Language::English => Self::load_english(locale),
            Language::Chinese => Self::load_chinese(locale),
            Language::Japanese => Self::load_japanese(locale),
            Language::Spanish => Self::load_spanish(locale),
            Language::French => Self::load_french(locale),
            Language::German => Self::load_german(locale),
            Language::Russian => Self::load_russian(locale),
            Language::Portuguese => Self::load_portuguese(locale),
            Language::Italian => Self::load_italian(locale),
            Language::Korean => Self::load_korean(locale),
        }
        Ok(())
    }

    fn load_english(locale: &mut Locale) {
        Self::insert_messages(
            locale,
            &[
                ("app.name", "Claude Code"),
                ("app.description", "AI-powered coding assistant"),
                ("app.version", "Version {version}"),
                ("menu.file", "File"),
                ("menu.edit", "Edit"),
                ("menu.view", "View"),
                ("menu.help", "Help"),
                ("action.new", "New"),
                ("action.open", "Open"),
                ("action.save", "Save"),
                ("action.save.as", "Save As"),
                ("action.exit", "Exit"),
                ("dialog.confirm", "Are you sure?"),
                ("dialog.yes", "Yes"),
                ("dialog.no", "No"),
                ("dialog.cancel", "Cancel"),
                ("dialog.ok", "OK"),
                ("error.generic", "An error occurred"),
                ("error.not.found", "Not found"),
                ("error.permission.denied", "Permission denied"),
                ("status.ready", "Ready"),
                ("status.loading", "Loading..."),
                ("status.saving", "Saving..."),
                ("status.done", "Done"),
                ("welcome.message", "Welcome to Claude Code!"),
                ("welcome.user", "Welcome, {name}!"),
                ("plugin.install", "Install"),
                ("plugin.uninstall", "Uninstall"),
                ("plugin.update", "Update"),
                ("plugin.installed", "Installed"),
                ("plugin.not.installed", "Not installed"),
            ],
        );
    }

    fn load_chinese(locale: &mut Locale) {
        // Embedded zh.ftl is preferred; this fallback keeps behavior stable when it is missing.
        Self::load_english(locale);
    }

    fn load_japanese(locale: &mut Locale) {
        Self::load_english(locale);
    }

    fn load_spanish(locale: &mut Locale) {
        Self::load_english(locale);
    }

    fn load_french(locale: &mut Locale) {
        Self::load_english(locale);
    }

    fn load_german(locale: &mut Locale) {
        Self::load_english(locale);
    }

    fn load_russian(locale: &mut Locale) {
        Self::load_english(locale);
    }

    fn load_portuguese(locale: &mut Locale) {
        Self::load_english(locale);
    }

    fn load_italian(locale: &mut Locale) {
        Self::load_english(locale);
    }

    fn load_korean(locale: &mut Locale) {
        Self::load_english(locale);
    }

    fn insert_messages(locale: &mut Locale, messages: &[(&str, &str)]) {
        for (key, value) in messages {
            locale.add_message(*key, *value);
        }
    }
}
