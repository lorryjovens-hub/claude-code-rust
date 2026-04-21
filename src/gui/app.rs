//! Main Application - GUI Application State and Logic

use eframe::Frame;
use egui::{CentralPanel, Context, SidePanel, TopBottomPanel};

use super::{chat::ChatPanel, sidebar::{Sidebar, Tab}, settings::SettingsPanel, Theme, GuiMessage};

/// Main application state
pub struct ClaudeCodeApp {
    theme: Theme,
    sidebar: Sidebar,
    chat_panel: ChatPanel,
    settings_panel: SettingsPanel,
    show_settings: bool,
    status_message: Option<String>,
    status_timer: Option<std::time::Instant>,

    /// Message sender for async tasks
    message_tx: Option<tokio::sync::mpsc::Sender<GuiMessage>>,
    /// Message receiver for GUI updates
    message_rx: Option<tokio::sync::mpsc::Receiver<GuiMessage>>,
    /// API client instance
    api_client: Option<crate::api::ApiClient>,
    /// Current settings
    settings: crate::config::Settings,
    /// Pending streaming message ID (for updating)
    pending_message_id: Option<String>,
}

impl Default for ClaudeCodeApp {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            sidebar: Sidebar::default(),
            chat_panel: ChatPanel::default(),
            settings_panel: SettingsPanel::default(),
            show_settings: false,
            status_message: None,
            status_timer: None,

            message_tx: None,
            message_rx: None,
            api_client: None,
            settings: crate::config::Settings::default(),
            pending_message_id: None,
        }
    }
}

impl ClaudeCodeApp {
    /// Create a new application instance
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let (message_tx, message_rx) = tokio::sync::mpsc::channel(100);

        // Load settings
        let settings = crate::config::Settings::load().unwrap_or_default();

        // Create API client
        let api_client = Some(crate::api::ApiClient::new(settings.clone()));

        let mut app = Self {
            message_tx: Some(message_tx),
            message_rx: Some(message_rx),
            api_client,
            settings,
            ..Self::default()
        };

        // Load settings into settings panel
        app.settings_panel.load_from_settings(&app.settings);

        // Set up chat panel callback
        let tx_clone = app.message_tx.clone().unwrap();
        app.chat_panel.set_on_send_message(move |messages| {
            let tx = tx_clone.clone();
            tokio::spawn(async move {
                tx.send(GuiMessage::SendMessage { messages }).await.ok();
            });
        });

        // Set up settings panel callbacks
        let tx_clone = app.message_tx.clone().unwrap();
        app.settings_panel.set_on_save_settings(move || {
            let tx = tx_clone.clone();
            tokio::spawn(async move {
                tx.send(GuiMessage::SettingsUpdated).await.ok();
            });
        });

        let tx_clone = app.message_tx.clone().unwrap();
        let settings_clone = app.settings.clone();
        app.settings_panel.set_on_test_connection(move || {
            let tx = tx_clone.clone();
            let settings = settings_clone.clone();

            tokio::spawn(async move {
                let result = Self::test_api_connection(settings).await;
                tx.send(GuiMessage::TestConnectionResult {
                    success: result.0,
                    message: result.1,
                })
                .await
                .ok();
            });
        });

        // Apply theme
        app.theme.apply(&cc.egui_ctx);

        // Load custom fonts if needed
        Self::configure_fonts(&cc.egui_ctx);

        app.show_status("Ready");

        app
    }

    /// Configure custom fonts
    fn configure_fonts(ctx: &Context) {
        let fonts = egui::FontDefinitions::default();
        
        // Add custom fonts here if needed
        // fonts.font_data.insert("my_font".to_owned(), ...);
        
        ctx.set_fonts(fonts);
    }

    /// Show a status message
    fn show_status(&mut self, message: impl Into<String>) {
        self.status_message = Some(message.into());
        self.status_timer = Some(std::time::Instant::now());
    }

    /// Process incoming messages
    fn process_messages(&mut self, ctx: &Context) {
        let mut pending_messages = Vec::new();

        // Take messages out first to avoid borrow conflict
        if let Some(rx) = &mut self.message_rx {
            while let Ok(msg) = rx.try_recv() {
                pending_messages.push(msg);
            }
        }

        // Process the messages
        for msg in pending_messages {
            match msg {
                GuiMessage::SendMessage { messages } => {
                    self.handle_send_message(messages);
                }
                GuiMessage::StreamChunk { content, done } => {
                    self.handle_stream_chunk(content, done);
                }
                GuiMessage::ApiError { error } => {
                    self.handle_api_error(error);
                }
                GuiMessage::TestConnectionResult { success, message } => {
                    self.handle_test_result(success, message);
                }
                GuiMessage::SettingsUpdated => {
                    self.save_settings();
                }
            }
        }

        // Request repaint if we have pending work
        if self.chat_panel.is_loading || self.pending_message_id.is_some() {
            ctx.request_repaint();
        }
    }

    /// Handle sending a message to the API
    fn handle_send_message(&mut self, messages: Vec<crate::api::ChatMessage>) {
        let api_client = match self.api_client.clone() {
            Some(client) => client,
            None => {
                self.show_status("API client not initialized");
                self.chat_panel.is_loading = false;
                return;
            }
        };

        let tx = self.message_tx.clone().unwrap();

        // Create placeholder assistant message
        let mut assistant_msg = crate::gui::chat::ChatMessage::assistant("");
        assistant_msg.is_streaming = true;
        self.pending_message_id = Some(assistant_msg.id.clone());
        self.chat_panel.messages.push(assistant_msg);

        // Spawn async task for API call
        tokio::spawn(async move {
            if api_client.get_api_key().is_none() {
                tx.send(GuiMessage::ApiError {
                    error: "API key not configured. Please set it in settings.".to_string(),
                })
                .await
                .ok();
                return;
            }

            match api_client.chat_stream(messages, None).await {
                Ok(response) => {
                    use futures::StreamExt;

                    // Handle streaming response
                    let mut stream = response.bytes_stream();
                    let mut buffer = String::new();

                    while let Some(chunk) = stream.next().await {
                        match chunk {
                            Ok(bytes) => {
                                // Parse SSE stream
                                let text = String::from_utf8_lossy(&bytes);
                                for line in text.lines() {
                                    if line.starts_with("data: ") {
                                        let data = &line[6..];
                                        if data == "[DONE]" {
                                            tx.send(GuiMessage::StreamChunk {
                                                content: buffer.clone(),
                                                done: true,
                                            })
                                            .await
                                            .ok();
                                            return;
                                        }

                                        if let Ok(chunk) =
                                            serde_json::from_str::<crate::api::StreamChunk>(data)
                                        {
                                            if let Some(choice) = chunk.choices.first() {
                                                if let Some(content) = &choice.delta.content {
                                                    buffer.push_str(content);
                                                    tx.send(GuiMessage::StreamChunk {
                                                        content: buffer.clone(),
                                                        done: false,
                                                    })
                                                    .await
                                                    .ok();
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tx.send(GuiMessage::ApiError {
                                    error: format!("Stream error: {}", e),
                                })
                                .await
                                .ok();
                                return;
                            }
                        }
                    }
                }
                Err(e) => {
                    tx.send(GuiMessage::ApiError {
                        error: format!("API error: {}", e),
                    })
                    .await
                    .ok();
                }
            }
        });
    }

    /// Handle streaming response chunks
    fn handle_stream_chunk(&mut self, content: String, done: bool) {
        if let Some(msg_id) = &self.pending_message_id {
            if let Some(msg) = self.chat_panel.messages.iter_mut().find(|m| m.id == *msg_id) {
                msg.content = content;
                msg.is_streaming = !done;

                if done {
                    self.pending_message_id = None;
                    self.chat_panel.is_loading = false;
                }
            }
        }
        self.chat_panel.scroll_to_bottom = true;
    }

    /// Handle API errors
    fn handle_api_error(&mut self, error: String) {
        // Add error message as system message
        self.chat_panel
            .messages
            .push(crate::gui::chat::ChatMessage::system(&format!(
                "Error: {}",
                error
            )));

        // Remove pending message if exists
        if let Some(msg_id) = &self.pending_message_id {
            self.chat_panel.messages.retain(|m| m.id != *msg_id);
            self.pending_message_id = None;
        }

        self.chat_panel.is_loading = false;
        self.show_status("API error occurred");
        self.chat_panel.scroll_to_bottom = true;
    }

    /// Handle test connection results
    fn handle_test_result(&mut self, success: bool, message: String) {
        self.settings_panel.set_test_result(success, message.clone());
        self.show_status(&message);
    }

    /// Save settings to disk
    fn save_settings(&mut self) {
        self.settings_panel.save_to_settings(&mut self.settings);

        if let Err(e) = self.settings.save() {
            self.show_status(&format!("Failed to save settings: {}", e));
            return;
        }

        // Recreate API client with new settings
        self.api_client = Some(crate::api::ApiClient::new(self.settings.clone()));

        self.show_status("Settings saved successfully");
    }

    /// Test the API connection
    async fn test_api_connection(settings: crate::config::Settings) -> (bool, String) {
        let client = crate::api::ApiClient::new(settings);

        if client.get_api_key().is_none() {
            return (false, "API key not configured".to_string());
        }

        // Send a simple test message
        let messages = vec![crate::api::ChatMessage::user(
            "Hi, please respond with 'Connection successful' if you receive this.",
        )];

        match client.chat(messages, None).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    if let Some(content) = &choice.message.content {
                        if content.contains("Connection successful") || !content.is_empty() {
                            return (true, "Connection successful!".to_string());
                        }
                    }
                }
                (true, "Connected, but unexpected response".to_string())
            }
            Err(e) => (false, format!("Connection failed: {}", e)),
        }
    }

    /// Update status timer
    fn update_status(&mut self) {
        if let Some(timer) = self.status_timer {
            if timer.elapsed().as_secs() > 3 {
                self.status_message = None;
                self.status_timer = None;
            }
        }
    }
}

impl eframe::App for ClaudeCodeApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        // Process pending messages
        self.process_messages(ctx);

        // Apply theme
        self.theme.apply(ctx);

        // Update status
        self.update_status();

        // Top panel - Title bar
        TopBottomPanel::top("top_panel")
            .exact_height(48.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Title
                    ui.heading(
                        egui::RichText::new("Claude Code")
                            .color(self.theme.primary_color())
                            .size(20.0)
                    );
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Window controls
                        if ui.button("➖").clicked() {
                            // Minimize window
                        }
                        if ui.button("⬜").clicked() {
                            // Maximize/restore window
                        }
                        if ui.button("✕").clicked() {
                            // Close window
                        }
                        
                        ui.add_space(8.0);
                        
                        // Settings button
                        let settings_text = if self.show_settings { "✓ ⚙️" } else { "⚙️" };
                        if ui.button(settings_text).clicked() {
                            self.show_settings = !self.show_settings;
                        }
                        
                        ui.add_space(8.0);
                        
                        // Theme toggle
                        let theme_icon = match self.theme {
                            Theme::Light => "☀️",
                            Theme::Dark => "🌙",
                            Theme::System => "💻",
                        };
                        if ui.button(theme_icon).clicked() {
                            self.theme = match self.theme {
                                Theme::Light => Theme::Dark,
                                Theme::Dark => Theme::System,
                                Theme::System => Theme::Light,
                            };
                            self.settings_panel.set_theme(self.theme);
                        }
                    });
                });
            });

        // Main content area
        if self.show_settings {
            // Show settings panel
            CentralPanel::default().show(ctx, |ui| {
                self.settings_panel.ui(ui, &self.theme);
            });
        } else {
            // Show main chat interface
            SidePanel::left("sidebar_panel")
                .resizable(true)
                .default_width(260.0)
                .min_width(200.0)
                .max_width(400.0)
                .show(ctx, |ui| {
                    self.sidebar.ui(ui, &self.theme);
                });

            CentralPanel::default().show(ctx, |ui| {
                match self.sidebar.selected_tab() {
                    Tab::Chat => {
                        self.chat_panel.ui(ui, &self.theme);
                    }
                    Tab::Settings => {
                        self.settings_panel.ui(ui, &self.theme);
                    }
                    _ => {
                        // Other tabs - show placeholder
                        ui.vertical_centered(|ui| {
                            ui.add_space(ui.available_height() / 2.0 - 50.0);
                            ui.heading(
                                egui::RichText::new("Coming Soon")
                                    .color(self.theme.muted_text_color())
                                    .size(24.0)
                            );
                            ui.label(
                                egui::RichText::new("This feature is under development")
                                    .color(self.theme.muted_text_color())
                            );
                        });
                    }
                }
            });
        }

        // Bottom panel - Status bar
        TopBottomPanel::bottom("bottom_panel")
            .exact_height(28.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Status message
                    if let Some(ref message) = self.status_message {
                        ui.label(
                            egui::RichText::new(message)
                                .color(self.theme.info_color())
                                .size(11.0)
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("Ready")
                                .color(self.theme.muted_text_color())
                                .size(11.0)
                        );
                    }
                    
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Version info
                        ui.label(
                            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .color(self.theme.muted_text_color())
                                .size(11.0)
                        );
                        
                        ui.add_space(16.0);
                        
                        // Connection status
                        ui.label(
                            egui::RichText::new("● Connected")
                                .color(self.theme.success_color())
                                .size(11.0)
                        );
                    });
                });
            });
    }

    fn on_exit(&mut self, _ctx: Option<&eframe::glow::Context>) {
        // Save app state before exit
        // In a real implementation, save to disk
    }
}

/// Run the GUI application
pub fn run_gui() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Claude Code",
        options,
        Box::new(|cc| Ok(Box::new(ClaudeCodeApp::new(cc)))),
    )
}
