//! Locale Loader - Loads locale data from embedded resources

use super::{Language, Locale};
use rust_embed::Embed;

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
        
        // Load embedded locale data
        let file_name = format!("{}.ftl", language.code());
        
        if let Some(content) = LocaleAssets::get(&file_name) {
            let content = std::str::from_utf8(&content.data)?;
            Self::parse_ftl(content, &mut locale)?;
        } else {
            // Use built-in fallback if no file found
            Self::load_builtin(language, &mut locale)?;
        }
        
        Ok(locale)
    }

    /// Parse Fluent FTL format
    fn parse_ftl(content: &str, locale: &mut Locale) -> anyhow::Result<()> {
        // Simple FTL parser (in production, use fluent-bundle crate)
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
                locale.add_message(key, value);
            }
        }
        
        Ok(())
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
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.description", "AI-powered code assistant"),
            ("menu.file", "File"),
            ("menu.edit", "Edit"),
            ("menu.view", "View"),
            ("menu.help", "Help"),
            ("button.new", "New"),
            ("button.save", "Save"),
            ("button.cancel", "Cancel"),
            ("button.ok", "OK"),
            ("error.generic", "An error occurred"),
            ("success.saved", "Saved successfully")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_chinese(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude 代码"),
            ("app.description", "AI 驱动的代码助手"),
            ("menu.file", "文件"),
            ("menu.edit", "编辑"),
            ("menu.view", "视图"),
            ("menu.help", "帮助"),
            ("button.new", "新建"),
            ("button.save", "保存"),
            ("button.cancel", "取消"),
            ("button.ok", "确定"),
            ("error.generic", "发生错误"),
            ("success.saved", "保存成功")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_japanese(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude コード"),
            ("app.description", "AI 駆動のコードアシスタント"),
            ("menu.file", "ファイル"),
            ("menu.edit", "編集"),
            ("menu.view", "表示"),
            ("menu.help", "ヘルプ"),
            ("button.new", "新規"),
            ("button.save", "保存"),
            ("button.cancel", "キャンセル"),
            ("button.ok", "OK"),
            ("error.generic", "エラーが発生しました"),
            ("success.saved", "保存しました")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_spanish(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Código"),
            ("app.description", "Asistente de código impulsado por IA"),
            ("menu.file", "Archivo"),
            ("menu.edit", "Editar"),
            ("menu.view", "Ver"),
            ("menu.help", "Ayuda"),
            ("button.new", "Nuevo"),
            ("button.save", "Guardar"),
            ("button.cancel", "Cancelar"),
            ("button.ok", "OK"),
            ("error.generic", "Se produjo un error"),
            ("success.saved", "Guardado correctamente")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_french(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.description", "Assistant de code alimenté par IA"),
            ("menu.file", "Fichier"),
            ("menu.edit", "Éditer"),
            ("menu.view", "Affichage"),
            ("menu.help", "Aide"),
            ("button.new", "Nouveau"),
            ("button.save", "Enregistrer"),
            ("button.cancel", "Annuler"),
            ("button.ok", "OK"),
            ("error.generic", "Une erreur s'est produite"),
            ("success.saved", "Enregistré avec succès")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_german(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Code"),
            ("app.description", "KI-gestützter Code-Assistent"),
            ("menu.file", "Datei"),
            ("menu.edit", "Bearbeiten"),
            ("menu.view", "Ansicht"),
            ("menu.help", "Hilfe"),
            ("button.new", "Neu"),
            ("button.save", "Speichern"),
            ("button.cancel", "Abbrechen"),
            ("button.ok", "OK"),
            ("error.generic", "Ein Fehler ist aufgetreten"),
            ("success.saved", "Erfolgreich gespeichert")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_russian(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Код"),
            ("app.description", "Ассистент по коду на основе ИИ"),
            ("menu.file", "Файл"),
            ("menu.edit", "Редактировать"),
            ("menu.view", "Вид"),
            ("menu.help", "Помощь"),
            ("button.new", "Новый"),
            ("button.save", "Сохранить"),
            ("button.cancel", "Отмена"),
            ("button.ok", "OK"),
            ("error.generic", "Произошла ошибка"),
            ("success.saved", "Сохранено успешно")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_portuguese(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Código"),
            ("app.description", "Assistente de código alimentado por IA"),
            ("menu.file", "Arquivo"),
            ("menu.edit", "Editar"),
            ("menu.view", "Visualizar"),
            ("menu.help", "Ajuda"),
            ("button.new", "Novo"),
            ("button.save", "Salvar"),
            ("button.cancel", "Cancelar"),
            ("button.ok", "OK"),
            ("error.generic", "Ocorreu um erro"),
            ("success.saved", "Salvo com sucesso")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_italian(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude Codice"),
            ("app.description", "Assistente di codice alimentato da IA"),
            ("menu.file", "File"),
            ("menu.edit", "Modifica"),
            ("menu.view", "Visualizza"),
            ("menu.help", "Aiuto"),
            ("button.new", "Nuovo"),
            ("button.save", "Salva"),
            ("button.cancel", "Annulla"),
            ("button.ok", "OK"),
            ("error.generic", "Si è verificato un errore"),
            ("success.saved", "Salvato con successo")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }

    fn load_korean(locale: &mut Locale) {
        let messages = vec![
            ("app.name", "Claude 코드"),
            ("app.description", "AI 기반 코드 어시스턴트"),
            ("menu.file", "파일"),
            ("menu.edit", "편집"),
            ("menu.view", "보기"),
            ("menu.help", "도움말"),
            ("button.new", "새로 만들기"),
            ("button.save", "저장"),
            ("button.cancel", "취소"),
            ("button.ok", "확인"),
            ("error.generic", "오류가 발생했습니다"),
            ("success.saved", "성공적으로 저장되었습니다")
        ];
        
        for (key, value) in messages {
            locale.add_message(key, value);
        }
    }
}
