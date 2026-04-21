//! Sidebar Component - Navigation sidebar for the GUI
//!
//! Claude-style sidebar with conversation list and navigation

use egui::{Color32, RichText, Ui, Vec2, Frame, Rounding, Margin, Stroke};

/// Sidebar state and configuration
pub struct Sidebar {
    pub selected_tab: Tab,
    pub collapsed: bool,
    pub width: f32,
    pub conversations: Vec<ConversationItem>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Chat,
    History,
    Plugins,
    Settings,
    Tools,
}

#[derive(Debug, Clone)]
pub struct ConversationItem {
    pub id: String,
    pub title: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub message_count: usize,
}

impl Default for Sidebar {
    fn default() -> Self {
        Self {
            selected_tab: Tab::Chat,
            collapsed: false,
            width: 260.0,
            conversations: vec![
                ConversationItem {
                    id: "1".to_string(),
                    title: "New Conversation".to_string(),
                    timestamp: chrono::Utc::now(),
                    message_count: 0,
                },
            ],
        }
    }
}

impl Sidebar {
    /// Render the sidebar
    pub fn ui(&mut self, ui: &mut Ui, theme: &super::Theme) {
        let width = if self.collapsed { 60.0 } else { self.width };

        egui::SidePanel::left("sidebar")
            .resizable(!self.collapsed)
            .min_width(width)
            .max_width(400.0)
            .default_width(width)
            .show_inside(ui, |ui| {
                Frame::none()
                    .fill(theme.background_darkest())
                    .show(ui, |ui| {
                        ui.set_width(width);
                        ui.set_min_height(ui.available_height());

                        // Header with collapse button
                        ui.horizontal(|ui| {
                            if !self.collapsed {
                                ui.heading(RichText::new("Claude")
                                    .color(theme.primary_color())
                                    .size(20.0)
                                    .strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    let collapse_btn = egui::Button::new(
                                        RichText::new("◀").color(theme.muted_text_color())
                                    )
                                    .fill(theme.surface_color())
                                    .rounding(Rounding::same(6.0));
                                    if ui.add(collapse_btn).clicked() {
                                        self.collapsed = true;
                                    }
                                });
                            } else {
                                let expand_btn = egui::Button::new(
                                    RichText::new("▶").color(theme.primary_color())
                                )
                                .fill(theme.surface_color())
                                .rounding(Rounding::same(6.0));
                                if ui.add(expand_btn).clicked() {
                                    self.collapsed = false;
                                }
                            }
                        });

                        ui.add_space(20.0);

                        if self.collapsed {
                            self.render_collapsed(ui, theme);
                        } else {
                            self.render_expanded(ui, theme);
                        }
                    });
            });
    }

    fn render_collapsed(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.vertical_centered(|ui| {
            // New chat button
            let new_chat_btn = egui::Button::new(
                RichText::new("➕").color(Color32::WHITE)
            )
            .fill(theme.primary_color())
            .min_size(Vec2::new(40.0, 40.0))
            .rounding(Rounding::same(10.0));

            if ui.add(new_chat_btn).clicked() {
                self.selected_tab = Tab::Chat;
                self.create_new_conversation();
            }
            ui.add_space(16.0);

            // Tab buttons
            let tabs = vec![
                (Tab::Chat, "💬", "Chat"),
                (Tab::History, "📜", "History"),
                (Tab::Plugins, "🔌", "Plugins"),
                (Tab::Tools, "🛠️", "Tools"),
                (Tab::Settings, "⚙️", "Settings"),
            ];

            for (tab, icon, _tooltip) in tabs {
                let is_selected = self.selected_tab == tab;
                let bg_color = if is_selected {
                    theme.primary_color()
                } else {
                    theme.surface_color()
                };
                let text_color = if is_selected {
                    Color32::WHITE
                } else {
                    theme.text_color()
                };

                let button = egui::Button::new(RichText::new(icon).size(20.0).color(text_color))
                    .fill(bg_color)
                    .min_size(Vec2::new(44.0, 44.0))
                    .rounding(Rounding::same(10.0));

                if ui.add(button).clicked() {
                    self.selected_tab = tab;
                }
                ui.add_space(6.0);
            }
        });
    }

    fn render_expanded(&mut self, ui: &mut Ui, theme: &super::Theme) {
        // New conversation button
        let new_chat_button = egui::Button::new(
            RichText::new("+  New Chat")
                .strong()
                .color(Color32::WHITE)
        )
        .fill(theme.primary_color())
        .min_size(Vec2::new(ui.available_width(), 44.0))
        .rounding(Rounding::same(10.0));

        if ui.add(new_chat_button).clicked() {
            self.create_new_conversation();
        }

        ui.add_space(20.0);

        // Tab buttons with modern styling
        ui.horizontal(|ui| {
            let tabs = vec![
                (Tab::Chat, "💬", "Chat"),
                (Tab::History, "📜", ""),
                (Tab::Plugins, "🔌", ""),
                (Tab::Tools, "🛠️", ""),
            ];

            for (tab, icon, label) in tabs {
                let is_selected = self.selected_tab == tab;
                let bg_color = if is_selected {
                    theme.elevated_color()
                } else {
                    theme.background_darkest()
                };
                let text_color = if is_selected {
                    theme.primary_color()
                } else {
                    theme.muted_text_color()
                };

                let btn_text = if label.is_empty() {
                    icon.to_string()
                } else {
                    format!("{} {}", icon, label)
                };

                let button = egui::Button::new(
                    RichText::new(btn_text).color(text_color).size(13.0)
                )
                .fill(bg_color)
                .min_size(Vec2::new(if label.is_empty() { 36.0 } else { 70.0 }, 32.0))
                .rounding(Rounding::same(8.0));

                if ui.add(button).clicked() {
                    self.selected_tab = tab;
                }
                ui.add_space(4.0);
            }
        });

        ui.add_space(12.0);

        // Separator
        ui.add(egui::Separator::default().spacing(8.0));

        // Tab content
        match self.selected_tab {
            Tab::Chat => self.render_conversations_list(ui, theme),
            Tab::History => self.render_history(ui, theme),
            Tab::Plugins => self.render_plugins(ui, theme),
            Tab::Tools => self.render_tools(ui, theme),
            Tab::Settings => self.render_settings_link(ui, theme),
        }

        ui.add_space(16.0);
        ui.add(egui::Separator::default().spacing(8.0));
        ui.add_space(8.0);

        // Settings button at bottom
        let settings_button = egui::Button::new(
            RichText::new("⚙️  Settings")
                .color(theme.text_color())
                .size(13.0)
        )
        .fill(theme.surface_color())
        .stroke(Stroke::new(1.0, theme.border_color()))
        .min_size(Vec2::new(ui.available_width(), 40.0))
        .rounding(Rounding::same(8.0));

        if ui.add(settings_button).clicked() {
            self.selected_tab = Tab::Settings;
        }
    }

    fn render_conversations_list(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.label(RichText::new("Recent conversations")
            .strong()
            .color(theme.muted_text_color())
            .size(12.0));
        ui.add_space(12.0);

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                for conversation in &self.conversations {
                    self.render_conversation_item(ui, conversation, theme);
                }
            });
    }

    fn render_conversation_item(&self, ui: &mut Ui, conversation: &ConversationItem, theme: &super::Theme) {
        let is_selected = conversation.id == "1"; // First conversation is selected

        let bg_color = if is_selected {
            theme.elevated_color()
        } else {
            theme.background_darkest()
        };

        let stroke = if is_selected {
            Stroke::new(1.0, theme.border_color())
        } else {
            Stroke::NONE
        };

        let button = egui::Button::new(
            RichText::new(format!("💬  {}", conversation.title))
                .color(theme.text_color())
                .size(13.0)
        )
        .fill(bg_color)
        .stroke(stroke)
        .min_size(Vec2::new(ui.available_width(), 48.0))
        .rounding(Rounding::same(10.0));

        ui.add(button);
        ui.add_space(6.0);
    }

    fn render_history(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.label(RichText::new("History")
            .strong()
            .color(theme.muted_text_color())
            .size(12.0));
        ui.add_space(12.0);

        let history_items = vec![
            ("Today", "3 chats"),
            ("Yesterday", "5 chats"),
            ("Last 7 days", "12 chats"),
            ("Last 30 days", "28 chats"),
        ];

        for (item, count) in history_items {
            ui.horizontal(|ui| {
                let button = egui::Button::new(
                    RichText::new(format!("📅  {}", item))
                        .color(theme.text_color())
                        .size(13.0)
                )
                .fill(theme.surface_color())
                .min_size(Vec2::new(ui.available_width() - 50.0, 40.0))
                .rounding(Rounding::same(8.0));

                ui.add(button);

                ui.label(RichText::new(count)
                    .color(theme.muted_text_color())
                    .size(11.0));
            });
            ui.add_space(6.0);
        }
    }

    fn render_plugins(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.label(RichText::new("Installed plugins")
            .strong()
            .color(theme.muted_text_color())
            .size(12.0));
        ui.add_space(12.0);

        let plugins = vec![
            ("🔌  File System", "Enabled", true),
            ("🔌  Git Integration", "Enabled", true),
            ("🔌  Code Analysis", "Disabled", false),
            ("🔌  Terminal", "Enabled", true),
        ];

        for (name, status, is_enabled) in plugins {
            let status_color = if is_enabled {
                theme.success_color()
            } else {
                theme.muted_text_color()
            };

            Frame::none()
                .fill(theme.surface_color())
                .rounding(Rounding::same(8.0))
                .inner_margin(Margin::symmetric(12.0, 8.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(name)
                            .color(theme.text_color())
                            .size(13.0));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(RichText::new(status)
                                .color(status_color)
                                .size(11.0));
                        });
                    });
                });
            ui.add_space(6.0);
        }

        ui.add_space(12.0);

        let install_btn = egui::Button::new(
            RichText::new("➕  Install Plugin")
                .color(theme.primary_color())
                .size(13.0)
        )
        .fill(theme.background_darkest())
        .stroke(Stroke::new(1.0, theme.border_color()))
        .min_size(Vec2::new(ui.available_width(), 40.0))
        .rounding(Rounding::same(8.0));

        if ui.add(install_btn).clicked() {
            // Open plugin marketplace
        }
    }

    fn render_tools(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.label(RichText::new("Quick tools")
            .strong()
            .color(theme.muted_text_color())
            .size(12.0));
        ui.add_space(12.0);

        let tools = vec![
            ("📁", "File Explorer", "Browse files"),
            ("🔍", "Search", "Search in codebase"),
            ("⚡", "Terminal", "Execute commands"),
            ("📝", "Editor", "Open code editor"),
        ];

        for (icon, name, desc) in tools {
            Frame::none()
                .fill(theme.surface_color())
                .rounding(Rounding::same(10.0))
                .inner_margin(Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(icon).size(20.0));
                        ui.add_space(8.0);

                        ui.vertical(|ui| {
                            ui.label(RichText::new(name)
                                .color(theme.text_color())
                                .size(13.0)
                                .strong());
                            ui.label(RichText::new(desc)
                                .color(theme.muted_text_color())
                                .size(11.0));
                        });
                    });
                });
            ui.add_space(6.0);
        }
    }

    fn render_settings_link(&mut self, ui: &mut Ui, theme: &super::Theme) {
        ui.label(RichText::new("Quick Settings")
            .strong()
            .color(theme.muted_text_color())
            .size(12.0));
        ui.add_space(12.0);

        let settings = vec![
            ("🔑", "API Configuration", "Configure API keys"),
            ("🎨", "Appearance", "Theme and colors"),
            ("🔔", "Notifications", "Alert preferences"),
            ("💾", "Data & Storage", "Manage your data"),
        ];

        for (icon, name, desc) in settings {
            Frame::none()
                .fill(theme.surface_color())
                .rounding(Rounding::same(10.0))
                .inner_margin(Margin::symmetric(12.0, 10.0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(icon).size(18.0));
                        ui.add_space(8.0);

                        ui.vertical(|ui| {
                            ui.label(RichText::new(name)
                                .color(theme.text_color())
                                .size(13.0));
                            ui.label(RichText::new(desc)
                                .color(theme.muted_text_color())
                                .size(11.0));
                        });
                    });
                });
            ui.add_space(6.0);
        }
    }

    fn create_new_conversation(&mut self) {
        let new_conversation = ConversationItem {
            id: uuid::Uuid::new_v4().to_string(),
            title: format!("Conversation {}", self.conversations.len() + 1),
            timestamp: chrono::Utc::now(),
            message_count: 0,
        };
        self.conversations.push(new_conversation);
    }

    /// Get the currently selected tab
    pub fn selected_tab(&self) -> Tab {
        self.selected_tab
    }

    /// Set the selected tab
    pub fn set_selected_tab(&mut self, tab: Tab) {
        self.selected_tab = tab;
    }

    /// Toggle sidebar collapse state
    pub fn toggle_collapse(&mut self) {
        self.collapsed = !self.collapsed;
    }
}
