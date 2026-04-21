//! Settings Panel - Application settings UI

use egui::{Color32, RichText, Ui, Vec2};

/// Settings panel state
pub struct SettingsPanel {
    pub current_section: SettingsSection,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub theme: super::Theme,
    pub language: String,
    pub auto_save: bool,
    pub notifications: bool,
    pub telemetry: bool,

    /// Callback for saving settings
    on_save_settings: Option<Box<dyn Fn() + Send>>,
    /// Callback for testing connection
    on_test_connection: Option<Box<dyn Fn() + Send>>,
    /// Test result display
    test_result: Option<(bool, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    General,
    Api,
    Appearance,
    Plugins,
    Advanced,
}

impl Default for SettingsPanel {
    fn default() -> Self {
        Self {
            current_section: SettingsSection::General,
            api_key: String::new(),
            base_url: "https://api.deepseek.com".to_string(),
            model: "deepseek-reasoner".to_string(),
            theme: super::Theme::Dark,
            language: "en".to_string(),
            auto_save: true,
            notifications: true,
            telemetry: false,

            on_save_settings: None,
            on_test_connection: None,
            test_result: None,
        }
    }
}

impl SettingsPanel {
    /// Set save settings callback
    pub fn set_on_save_settings<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.on_save_settings = Some(Box::new(callback));
    }

    /// Set test connection callback
    pub fn set_on_test_connection<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        self.on_test_connection = Some(Box::new(callback));
    }

    /// Set test result display
    pub fn set_test_result(&mut self, success: bool, message: String) {
        self.test_result = Some((success, message));
    }

    /// Load settings from configuration
    pub fn load_from_settings(&mut self, settings: &crate::config::Settings) {
        self.api_key = settings.api.api_key.clone().unwrap_or_default();
        self.base_url = settings.api.get_base_url();
        self.model = settings.model.clone();
    }

    /// Save settings to configuration
    pub fn save_to_settings(&self, settings: &mut crate::config::Settings) {
        settings.api.api_key = if self.api_key.is_empty() {
            None
        } else {
            Some(self.api_key.clone())
        };
        settings.api.base_url = self.base_url.clone();
        settings.model = self.model.clone();
    }

    /// Render the settings panel
    pub fn ui(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.horizontal(|ui| {
            // Left sidebar for sections
            ui.vertical(|ui| {
                ui.set_width(180.0);
                ui.set_min_height(ui.available_height());

                self.render_section_list(ui, theme);
            });

            ui.separator();

            // Right panel for settings content
            ui.vertical(|ui| {
                ui.set_min_width(ui.available_width());
                ui.set_min_height(ui.available_height());

                match self.current_section {
                    SettingsSection::General => self.render_general_settings(ui, theme),
                    SettingsSection::Api => self.render_api_settings(ui, theme),
                    SettingsSection::Appearance => self.render_appearance_settings(ui, theme),
                    SettingsSection::Plugins => self.render_plugin_settings(ui, theme),
                    SettingsSection::Advanced => self.render_advanced_settings(ui, theme),
                }
            });
        });
    }

    fn render_section_list(&mut self, ui: &mut Ui, theme: &super::Theme) {
        let sections = vec![
            (SettingsSection::General, "⚙️", "General"),
            (SettingsSection::Api, "🔑", "API"),
            (SettingsSection::Appearance, "🎨", "Appearance"),
            (SettingsSection::Plugins, "🔌", "Plugins"),
            (SettingsSection::Advanced, "⚡", "Advanced"),
        ];

        for (section, icon, label) in sections {
            let is_selected = self.current_section == section;
            
            let button = egui::Button::new(
                RichText::new(format!("{} {}", icon, label))
                    .color(if is_selected { Color32::WHITE } else { theme.text_color() })
            )
            .fill(if is_selected { theme.primary_color() } else { theme.surface_color() })
            .min_size(Vec2::new(ui.available_width(), 40.0))
            .rounding(8.0);

            if ui.add(button).clicked() {
                self.current_section = section;
            }
            ui.add_space(4.0);
        }
    }

    fn render_general_settings(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.heading(RichText::new("General Settings").color(theme.text_color()));
        ui.add_space(16.0);

        // Language selection
        ui.group(|ui| {
            ui.label(RichText::new("Language").strong().color(theme.text_color()));
            ui.add_space(4.0);
            
            egui::ComboBox::from_id_source("language")
                .selected_text(&self.language)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.language, "en".to_string(), "🇺🇸 English");
                    ui.selectable_value(&mut self.language, "zh".to_string(), "🇨🇳 中文");
                    ui.selectable_value(&mut self.language, "ja".to_string(), "🇯🇵 日本語");
                    ui.selectable_value(&mut self.language, "es".to_string(), "🇪🇸 Español");
                    ui.selectable_value(&mut self.language, "fr".to_string(), "🇫🇷 Français");
                    ui.selectable_value(&mut self.language, "de".to_string(), "🇩🇪 Deutsch");
                });
        });

        ui.add_space(16.0);

        // Auto-save toggle
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.auto_save, "");
            ui.vertical(|ui| {
                ui.label(RichText::new("Auto-save conversations")
                    .strong()
                    .color(theme.text_color()));
                ui.label(RichText::new("Automatically save conversation history")
                    .color(theme.muted_text_color())
                    .size(11.0));
            });
        });

        ui.add_space(8.0);

        // Notifications toggle
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.notifications, "");
            ui.vertical(|ui| {
                ui.label(RichText::new("Enable notifications")
                    .strong()
                    .color(theme.text_color()));
                ui.label(RichText::new("Show notifications for important events")
                    .color(theme.muted_text_color())
                    .size(11.0));
            });
        });

        ui.add_space(8.0);

        // Telemetry toggle
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.telemetry, "");
            ui.vertical(|ui| {
                ui.label(RichText::new("Enable telemetry")
                    .strong()
                    .color(theme.text_color()));
                ui.label(RichText::new("Help improve Claude Code by sharing anonymous usage data")
                    .color(theme.muted_text_color())
                    .size(11.0));
            });
        });
    }

    fn render_api_settings(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.heading(RichText::new("API Configuration").color(theme.text_color()));
        ui.add_space(16.0);

        // API Key
        ui.group(|ui| {
            ui.label(RichText::new("API Key").strong().color(theme.text_color()));
            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                let api_key_edit = egui::TextEdit::singleline(&mut self.api_key)
                    .password(true)
                    .hint_text("Enter your API key")
                    .desired_width(ui.available_width() - 100.0);
                
                ui.add(api_key_edit);
                
                if ui.button("Show").clicked() {
                    // Toggle visibility
                }
            });
            
            ui.label(RichText::new("Your API key is stored securely on your device")
                .color(theme.muted_text_color())
                .size(11.0));
        });

        ui.add_space(16.0);

        // Base URL
        ui.group(|ui| {
            ui.label(RichText::new("Base URL").strong().color(theme.text_color()));
            ui.add_space(4.0);
            
            ui.add(
                egui::TextEdit::singleline(&mut self.base_url)
                    .hint_text("https://api.example.com")
                    .desired_width(ui.available_width())
            );
        });

        ui.add_space(16.0);

        // Model selection
        ui.group(|ui| {
            ui.label(RichText::new("Model").strong().color(theme.text_color()));
            ui.add_space(4.0);
            
            egui::ComboBox::from_id_source("model")
                .selected_text(&self.model)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.model, "deepseek-reasoner".to_string(), "DeepSeek Reasoner");
                    ui.selectable_value(&mut self.model, "deepseek-chat".to_string(), "DeepSeek Chat");
                    ui.selectable_value(&mut self.model, "claude-sonnet".to_string(), "Claude Sonnet");
                    ui.selectable_value(&mut self.model, "claude-opus".to_string(), "Claude Opus");
                    ui.selectable_value(&mut self.model, "claude-haiku".to_string(), "Claude Haiku");
                });
        });

        ui.add_space(16.0);

        // Save button
        let save_button = egui::Button::new(
            RichText::new("💾 Save Settings")
                .strong()
                .color(Color32::WHITE),
        )
        .fill(theme.success_color())
        .min_size(Vec2::new(150.0, 36.0))
        .rounding(8.0);

        if ui.add(save_button).clicked() {
            if let Some(callback) = &self.on_save_settings {
                callback();
            }
        }

        ui.add_space(16.0);

        // Test connection button
        ui.horizontal(|ui| {
            let test_button = egui::Button::new(
                RichText::new("🔄 Test Connection")
                    .strong()
                    .color(Color32::WHITE),
            )
            .fill(theme.primary_color())
            .min_size(Vec2::new(150.0, 36.0))
            .rounding(8.0);

            if ui.add(test_button).clicked() {
                if let Some(callback) = &self.on_test_connection {
                    callback();
                }
            }

            // Show test result
            if let Some((success, message)) = &self.test_result {
                ui.add_space(8.0);
                let (icon, color) = if *success {
                    ("✅", theme.success_color())
                } else {
                    ("❌", theme.error_color())
                };
                ui.label(RichText::new(format!("{} {}", icon, message)).color(color));
            }
        });
    }

    fn render_appearance_settings(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.heading(RichText::new("Appearance").color(theme.text_color()));
        ui.add_space(16.0);

        // Theme selection
        ui.group(|ui| {
            ui.label(RichText::new("Theme").strong().color(theme.text_color()));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let themes = vec![
                    (super::Theme::Light, "☀️", "Light"),
                    (super::Theme::Dark, "🌙", "Dark"),
                    (super::Theme::System, "💻", "System"),
                ];

                for (t, icon, label) in themes {
                    let is_selected = self.theme == t;
                    let button = egui::Button::new(
                        RichText::new(format!("{} {}", icon, label))
                            .color(if is_selected { Color32::WHITE } else { theme.text_color() })
                    )
                    .fill(if is_selected { theme.primary_color() } else { theme.surface_color() })
                    .min_size(Vec2::new(100.0, 60.0))
                    .rounding(8.0);

                    if ui.add(button).clicked() {
                        self.theme = t;
                    }
                    ui.add_space(8.0);
                }
            });
        });

        ui.add_space(16.0);

        // Font size
        ui.group(|ui| {
            ui.label(RichText::new("Font Size").strong().color(theme.text_color()));
            ui.add_space(4.0);
            
            ui.horizontal(|ui| {
                if ui.button("A-").clicked() {
                    // Decrease font size
                }
                ui.label("100%");
                if ui.button("A+").clicked() {
                    // Increase font size
                }
            });
        });

        ui.add_space(16.0);

        // Compact mode
        ui.horizontal(|ui| {
            let mut compact_mode = false;
            ui.checkbox(&mut compact_mode, "");
            ui.vertical(|ui| {
                ui.label(RichText::new("Compact mode")
                    .strong()
                    .color(theme.text_color()));
                ui.label(RichText::new("Reduce padding and margins for a more compact view")
                    .color(theme.muted_text_color())
                    .size(11.0));
            });
        });
    }

    fn render_plugin_settings(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.heading(RichText::new("Plugin Settings").color(theme.text_color()));
        ui.add_space(16.0);

        // Installed plugins list
        let plugins = vec![
            ("File System", "1.0.0", "Access and manage files", true),
            ("Git Integration", "1.2.0", "Git commands and repository management", true),
            ("Code Analysis", "0.9.0", "Static code analysis tools", false),
            ("Terminal", "1.1.0", "Integrated terminal access", true),
        ];

        for (name, version, description, enabled) in plugins {
            let mut is_enabled = enabled;
            
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut is_enabled, "");
                    
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(name)
                                .strong()
                                .color(theme.text_color()));
                            ui.label(RichText::new(format!("v{}", version))
                                .color(theme.muted_text_color())
                                .size(11.0));
                        });
                        ui.label(RichText::new(description)
                            .color(theme.muted_text_color())
                            .size(11.0));
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("⚙️").clicked() {
                            // Open plugin settings
                        }
                        if ui.button("🗑️").clicked() {
                            // Uninstall plugin
                        }
                    });
                });
            });
            ui.add_space(8.0);
        }

        ui.add_space(16.0);

        // Install new plugin button
        let install_button = egui::Button::new(
            RichText::new("➕ Install Plugin")
                .strong()
                .color(Color32::WHITE)
        )
        .fill(theme.primary_color())
        .min_size(Vec2::new(150.0, 36.0))
        .rounding(8.0);

        if ui.add(install_button).clicked() {
            // Open plugin marketplace
        }
    }

    fn render_advanced_settings(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.heading(RichText::new("Advanced Settings").color(theme.text_color()));
        ui.add_space(16.0);

        // Cache settings
        ui.group(|ui| {
            ui.label(RichText::new("Cache").strong().color(theme.text_color()));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                ui.label("Cache size: ");
                ui.label(RichText::new("125 MB").strong().color(theme.text_color()));
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear Cache").clicked() {
                        // Clear cache
                    }
                });
            });
        });

        ui.add_space(16.0);

        // Data export/import
        ui.group(|ui| {
            ui.label(RichText::new("Data Management").strong().color(theme.text_color()));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("📥 Export Data").clicked() {
                    // Export data
                }
                if ui.button("📤 Import Data").clicked() {
                    // Import data
                }
            });
        });

        ui.add_space(16.0);

        // Reset settings
        ui.group(|ui| {
            ui.label(RichText::new("Reset").strong().color(theme.text_color()));
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Reset Settings").clicked() {
                    // Reset to defaults
                }
                if ui.button("Clear All Data").clicked() {
                    // Clear all data
                }
            });
        });

        ui.add_space(16.0);

        // Developer options
        ui.collapsing(RichText::new("Developer Options").color(theme.text_color()), |ui| {
            let mut dev_mode = false;
            ui.checkbox(&mut dev_mode, "Enable developer mode");
            
            let mut debug_logging = false;
            ui.checkbox(&mut debug_logging, "Enable debug logging");
            
            let mut experimental_features = false;
            ui.checkbox(&mut experimental_features, "Enable experimental features");
        });
    }

    /// Get the current theme
    pub fn theme(&self) -> super::Theme {
        self.theme
    }

    /// Set the theme
    pub fn set_theme(&mut self, theme: super::Theme) {
        self.theme = theme;
    }
}
