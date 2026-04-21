//! Chat UI Component - Full recreation of Claude Code chat interface
//!
//! Features:
//! - Exact Claude.ai message styling
//! - Syntax highlighted code blocks with copy button
//! - Tool call visualization
//! - File attachments
//! - Thinking process expand/collapse
//! - Perfect markdown rendering

use egui::{Color32, RichText, ScrollArea, TextEdit, Ui, Vec2, Frame, Stroke, Rounding, Margin, Layout, Align};
use chrono::{DateTime, Utc};
use super::syntax_highlight::{format_code_block, CodeHighlighter};
use super::tool_calls::{ToolCall, ToolCallManager};

/// A chat message - matches Claude.ai structure
#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_streaming: bool,
    pub tool_calls: Vec<ToolCall>,
    pub attachments: Vec<Attachment>,
    pub thinking: Option<String>,
    pub thinking_expanded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone)]
pub struct Attachment {
    pub name: String,
    pub content_type: String,
    pub size: usize,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            role,
            content: content.into(),
            timestamp: Utc::now(),
            is_streaming: false,
            tool_calls: Vec::new(),
            attachments: Vec::new(),
            thinking: None,
            thinking_expanded: false,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    pub fn with_thinking(mut self, thinking: impl Into<String>) -> Self {
        self.thinking = Some(thinking.into());
        self
    }

    pub fn with_tool_calls(mut self, calls: Vec<ToolCall>) -> Self {
        self.tool_calls = calls;
        self
    }
}

/// Chat panel - full recreation of Claude Code interface
pub struct ChatPanel {
    pub messages: Vec<ChatMessage>,
    pub input_text: String,
    pub is_loading: bool,
    pub scroll_to_bottom: bool,
    highlighter: CodeHighlighter,
    tool_manager: ToolCallManager,

    /// Callback for sending messages
    on_send_message: Option<Box<dyn Fn(Vec<crate::api::ChatMessage>) + Send>>,
}

impl Default for ChatPanel {
    fn default() -> Self {
        Self {
            messages: vec![
                ChatMessage::assistant("Hello! I'm Claude, your AI coding companion. I can help you with:")
                    .with_thinking("The user has started a new conversation. I should greet them warmly and explain my capabilities.")
            ],
            input_text: String::new(),
            is_loading: false,
            scroll_to_bottom: true,
            highlighter: CodeHighlighter::new(),
            tool_manager: ToolCallManager::default(),
            on_send_message: None,
        }
    }
}

impl ChatPanel {
    pub fn set_on_send_message<F>(&mut self, callback: F)
    where
        F: Fn(Vec<crate::api::ChatMessage>) + Send + 'static,
    {
        self.on_send_message = Some(Box::new(callback));
    }

    /// Render the complete chat panel
    pub fn ui(&mut self, ui: &mut Ui, theme: &super::Theme) {
        let available_height = ui.available_height();
        let input_height = 120.0;
        let messages_height = available_height - input_height - 16.0;

        // Messages area with exact Claude styling
        Frame::none()
            .fill(theme.background_darkest())
            .show(ui, |ui| {
                ui.set_min_height(messages_height);

                ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(self.scroll_to_bottom)
                    .show(ui, |ui| {
                        ui.add_space(24.0);

                        // Welcome banner for first load
                        if self.messages.len() <= 1 {
                            self.render_welcome_banner(ui, theme);
                        }

                        // Render all messages
                        let msg_count = self.messages.len();
                        // 先克隆所有消息，避免借用冲突
                        let mut messages_clone = self.messages.clone();
                        for (idx, message) in messages_clone.iter_mut().enumerate() {
                            self.render_message(ui, message, theme, idx == msg_count - 1);
                        }
                        // 更新展开状态
                        for i in 0..msg_count {
                            self.messages[i].thinking_expanded = messages_clone[i].thinking_expanded;
                        }

                        // Loading indicator
                        if self.is_loading && !self.messages.iter().any(|m| m.is_streaming) {
                            self.render_loading_indicator(ui, theme);
                        }

                        ui.add_space(24.0);
                    });
            });

        // Input area - fixed at bottom
        Frame::none()
            .fill(theme.background_darkest())
            .show(ui, |ui| {
                self.render_input_area(ui, theme);
            });
    }

    fn render_welcome_banner(&self, ui: &mut Ui, theme: &super::Theme) {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);

            // Claude logo/icon
            ui.label(RichText::new("🦀").size(48.0));

            ui.add_space(16.0);

            ui.label(
                RichText::new("Claude Code Rust")
                    .size(24.0)
                    .strong()
                    .color(theme.primary_color())
            );

            ui.add_space(8.0);

            ui.label(
                RichText::new("Your AI coding companion - powered by Rust 🦀")
                    .size(14.0)
                    .color(theme.muted_text_color())
            );

            ui.add_space(24.0);

            // Capabilities
            let capabilities = vec![
                ("💻", "Write & edit code", "Generate and modify code in any language"),
                ("📁", "Read files", "View and analyze your codebase"),
                ("🔍", "Search", "Find code patterns across your project"),
                ("⚡", "Run commands", "Execute bash commands in your terminal"),
            ];

            for (icon, title, desc) in capabilities {
                Frame::none()
                    .fill(theme.surface_color())
                    .rounding(Rounding::same(8.0))
                    .inner_margin(Margin::symmetric(16.0, 12.0))
                    .show(ui, |ui| {
                        ui.set_width(400.0);
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(icon).size(20.0));
                            ui.add_space(12.0);

                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new(title)
                                        .size(14.0)
                                        .strong()
                                        .color(theme.text_color())
                                );
                                ui.label(
                                    RichText::new(desc)
                                        .size(12.0)
                                        .color(theme.muted_text_color())
                                );
                            });
                        });
                    });

                ui.add_space(8.0);
            }

            ui.add_space(40.0);
        });
    }

    fn render_message(&self, ui: &mut Ui, message: &mut ChatMessage, theme: &super::Theme, is_last: bool) {
        let is_user = matches!(message.role, MessageRole::User);

        // Full-width message container
        Frame::none()
            .fill(if is_user {
                theme.background_darkest()
            } else {
                theme.background_darkest()
            })
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Message content with proper padding
                ui.horizontal(|ui| {
                    // Left margin/avatar area
                    ui.add_space(if is_user { 80.0 } else { 24.0 });

                    // Message content
                    ui.vertical(|ui| {
                        // Avatar and name row
                        ui.horizontal(|ui| {
                            if !is_user {
                                // Claude avatar
                                self.render_claude_avatar(ui, theme);
                                ui.add_space(12.0);

                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("Claude")
                                            .size(14.0)
                                            .strong()
                                            .color(theme.primary_color())
                                    );

                                    // Timestamp
                                    ui.label(
                                        RichText::new(
                                            message.timestamp.format("%I:%M %p").to_string()
                                        )
                                        .size(11.0)
                                        .color(theme.muted_text_color())
                                    );
                                });
                            } else {
                                // User info
                                ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                                    self.render_user_avatar(ui, theme);
                                });
                            }
                        });

                        ui.add_space(8.0);

                        // Message content or bubble
                        if is_user {
                            // User message - right aligned bubble
                            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                                self.render_user_message_bubble(ui, message, theme);
                            });
                        } else {
                            // Claude message - left aligned with full width content
                            self.render_claude_message_content(ui, message, theme, is_last);
                        }

                        // Tool calls for assistant messages
                        if !is_user && !message.tool_calls.is_empty() {
                            ui.add_space(16.0);
                            for tool_call in &message.tool_calls {
                                self.render_tool_call_card(ui, tool_call, theme);
                                ui.add_space(8.0);
                            }
                        }
                    });

                    // Right margin
                    ui.add_space(24.0);
                });
            });

        ui.add_space(24.0);
    }

    fn render_claude_avatar(&self, ui: &mut Ui, theme: &super::Theme) {
        Frame::none()
            .fill(theme.primary_color())
            .rounding(Rounding::same(8.0))
            .show(ui, |ui| {
                ui.set_width(32.0);
                ui.set_height(32.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("🦀").size(18.0));
                });
            });
    }

    fn render_user_avatar(&self, ui: &mut Ui, theme: &super::Theme) {
        Frame::none()
            .fill(Color32::from_rgb(80, 80, 80))
            .rounding(Rounding::same(8.0))
            .show(ui, |ui| {
                ui.set_width(32.0);
                ui.set_height(32.0);
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("👤").size(18.0));
                });
            });
    }

    fn render_user_message_bubble(&self, ui: &mut Ui, message: &ChatMessage, theme: &super::Theme) {
        let max_width = 600.0f32.min(ui.available_width() * 0.8);

        Frame::none()
            .fill(Color32::from_rgb(212, 165, 116)) // Claude orange
            .rounding(Rounding::same(16.0))
            .inner_margin(Margin::symmetric(16.0, 12.0))
            .show(ui, |ui| {
                ui.set_max_width(max_width);

                // Message text
                ui.label(
                    RichText::new(&message.content)
                        .size(15.0)
                        .color(Color32::WHITE)
                );

                // Attachments
                for attachment in &message.attachments {
                    ui.add_space(8.0);
                    self.render_attachment(ui, attachment, theme);
                }
            });
    }

    fn render_claude_message_content(&self, ui: &mut Ui, message: &mut ChatMessage, theme: &super::Theme, is_last: bool) {
        let max_width = 720.0f32.min(ui.available_width());

        ui.vertical(|ui| {
            ui.set_max_width(max_width);

            // Thinking process (expandable)
            if let Some(thinking) = &message.thinking {
                self.render_thinking_process(ui, thinking, &mut message.thinking_expanded, theme);
                ui.add_space(12.0);
            }

            // Main content
            let content = if message.is_streaming {
                format!("{}▌", message.content)
            } else {
                message.content.clone()
            };

            self.render_markdown_content(ui, &content, theme);

            // Streaming cursor animation for last message
            if message.is_streaming && is_last {
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    self.render_cursor_animation(ui);
                });
            }
        });
    }

    fn render_thinking_process(&self, ui: &mut Ui, thinking: &str, expanded: &mut bool, theme: &super::Theme) {
        let header_text = if *expanded { "▼ Thinking" } else { "▶ Thinking" };

        if ui.button(
            RichText::new(header_text)
                .size(12.0)
                .color(theme.muted_text_color())
        ).clicked() {
            *expanded = !*expanded;
        }

        if *expanded {
            Frame::none()
                .fill(Color32::from_rgb(30, 30, 35))
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::same(12.0))
                .stroke(Stroke::new(1.0, Color32::from_rgb(60, 60, 70)))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(
                        RichText::new(thinking)
                            .size(12.0)
                            .color(theme.muted_text_color())
                            .italics()
                    );
                });
        }
    }

    fn render_markdown_content(&self, ui: &mut Ui, content: &str, theme: &super::Theme) {
        // Split content by code blocks
        let parts = split_by_code_blocks(content);
        let mut in_list = false;
        let mut list_number = 0;

        for part in parts {
            match part {
                ContentPart::Text(text) => {
                    // Process markdown line by line
                    for line in text.lines() {
                        let trimmed = line.trim();

                        if trimmed.is_empty() {
                            if in_list {
                                in_list = false;
                                list_number = 0;
                            }
                            ui.add_space(8.0);
                            continue;
                        }

                        // Headers
                        if let Some(header_text) = trimmed.strip_prefix("# ") {
                            ui.add_space(16.0);
                            ui.label(
                                RichText::new(header_text)
                                    .size(24.0)
                                    .strong()
                                    .color(theme.text_color())
                            );
                            ui.add_space(8.0);
                        } else if let Some(header_text) = trimmed.strip_prefix("## ") {
                            ui.add_space(12.0);
                            ui.label(
                                RichText::new(header_text)
                                    .size(20.0)
                                    .strong()
                                    .color(theme.text_color())
                            );
                            ui.add_space(6.0);
                        } else if let Some(header_text) = trimmed.strip_prefix("### ") {
                            ui.add_space(8.0);
                            ui.label(
                                RichText::new(header_text)
                                    .size(16.0)
                                    .strong()
                                    .color(theme.text_color())
                            );
                            ui.add_space(4.0);
                        }
                        // Bullet lists
                        else if let Some(item) = trimmed.strip_prefix("- ") {
                            in_list = true;
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("•").size(16.0).color(theme.primary_color()));
                                ui.add_space(8.0);
                                ui.label(RichText::new(item).size(15.0).color(theme.text_color()));
                            });
                        }
                        // Numbered lists
                        else if let Some((num, item)) = trimmed.split_once(". ") {
                            if num.parse::<u32>().is_ok() {
                                in_list = true;
                                list_number += 1;
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(format!("{}.", list_number))
                                            .size(14.0)
                                            .strong()
                                            .color(theme.primary_color())
                                    );
                                    ui.add_space(8.0);
                                    ui.label(RichText::new(item).size(15.0).color(theme.text_color()));
                                });
                            }
                        }
                        // Blockquote
                        else if let Some(quote) = trimmed.strip_prefix("> ") {
                            Frame::none()
                                .fill(Color32::from_rgb(40, 40, 45))
                                .rounding(Rounding::same(4.0))
                                .inner_margin(Margin::same(12.0))
                                .stroke(Stroke::new(2.0, theme.primary_color()))
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    ui.label(
                                        RichText::new(quote)
                                            .size(14.0)
                                            .color(theme.text_secondary_color())
                                            .italics()
                                    );
                                });
                        }
                        // Regular paragraph
                        else {
                            in_list = false;
                            list_number = 0;

                            // Handle inline formatting
                            let formatted = self.apply_inline_formatting(trimmed, theme);
                            ui.label(formatted.size(15.0));
                        }
                    }
                }
                ContentPart::CodeBlock { language, code } => {
                    ui.add_space(8.0);
                    format_code_block(ui, code, language, theme.is_dark());
                    ui.add_space(8.0);
                }
                ContentPart::InlineCode(code) => {
                    Frame::none()
                        .fill(Color32::from_rgb(55, 55, 60))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::symmetric(4.0, 2.0))
                        .show(ui, |ui| {
                            ui.monospace(
                                RichText::new(code)
                                    .size(13.0)
                                    .color(Color32::from_rgb(212, 165, 116))
                            );
                        });
                }
            }
        }
    }

    fn apply_inline_formatting(&self, text: &str, theme: &super::Theme) -> RichText {
        // For now, just return the text without formatting
        RichText::new(text).color(theme.text_color())
    }

    fn render_tool_call_card(&self, ui: &mut Ui, tool_call: &ToolCall, theme: &super::Theme) {
        // Use the tool_calls module rendering
        // This is a simplified version inline here
        let (icon, title, border_color) = match tool_call.name.as_str() {
            "read_file" | "view" => ("📖", "View", Color32::from_rgb(100, 181, 246)),
            "write_file" | "edit" => ("📝", "Edit", Color32::from_rgb(76, 175, 80)),
            "create_file" => ("✨", "Create", Color32::from_rgb(156, 39, 176)),
            "bash" | "execute" => ("⚡", "Bash", Color32::from_rgb(255, 152, 0)),
            "search" => ("🔍", "Search", Color32::from_rgb(33, 150, 243)),
            "list_directory" => ("📁", "List", Color32::from_rgb(121, 85, 72)),
            _ => ("🔧", tool_call.name.as_str(), theme.border_color()),
        };

        Frame::none()
            .fill(theme.surface_color())
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.5, border_color))
            .show(ui, |ui| {
                ui.set_width(ui.available_width().min(600.0));

                // Header
                ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label(RichText::new(icon).size(16.0));
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(title)
                            .strong()
                            .size(13.0)
                            .color(theme.text_color())
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(12.0);
                        let status_icon = match tool_call.status {
                            super::tool_calls::ToolCallStatus::Pending => "⏳",
                            super::tool_calls::ToolCallStatus::Running => "🔄",
                            super::tool_calls::ToolCallStatus::Success => "✅",
                            super::tool_calls::ToolCallStatus::Error => "❌",
                        };
                        ui.label(RichText::new(status_icon).size(14.0));
                    });
                });

                ui.add_space(8.0);
            });
    }

    fn render_attachment(&self, ui: &mut Ui, attachment: &Attachment, theme: &super::Theme) {
        Frame::none()
            .fill(Color32::from_rgb(50, 50, 55))
            .rounding(Rounding::same(8.0))
            .inner_margin(Margin::symmetric(12.0, 8.0))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("📎").size(16.0));
                    ui.add_space(8.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(&attachment.name)
                                .size(13.0)
                                .color(theme.text_color())
                        );
                        ui.label(
                            RichText::new(format!("{} • {} bytes", attachment.content_type, attachment.size))
                                .size(11.0)
                                .color(theme.muted_text_color())
                        );
                    });
                });
            });
    }

    fn render_cursor_animation(&self, ui: &mut Ui) {
        // Blinking cursor effect
        let time = ui.ctx().input(|i| i.time);
        let blink = (time * 2.0).sin() > 0.0;

        if blink {
            ui.label(
                RichText::new("▋")
                    .size(16.0)
                    .color(Color32::from_rgb(212, 165, 116))
            );
        } else {
            ui.label(RichText::new(" ").size(16.0));
        }
    }

    fn render_loading_indicator(&self, ui: &mut Ui, theme: &super::Theme) {
        ui.horizontal(|ui| {
            ui.add_space(80.0);

            Frame::none()
                .fill(theme.surface_color())
                .rounding(Rounding::same(16.0))
                .inner_margin(Margin::symmetric(16.0, 12.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Animated dots
                        let time = ui.ctx().input(|i| i.time);
                        let dot_count = (time * 3.0) as usize % 4;
                        let dots = "●".repeat(dot_count + 1);

                        ui.label(
                            RichText::new(dots)
                                .color(theme.primary_color())
                                .size(12.0)
                        );

                        ui.add_space(12.0);

                        ui.label(
                            RichText::new("Claude is thinking...")
                                .color(theme.muted_text_color())
                                .size(14.0)
                        );
                    });
                });
        });
    }

    fn render_input_area(&mut self, ui: &mut Ui, theme: &super::Theme) {
        Frame::none()
            .fill(theme.background_darkest())
            .inner_margin(Margin::same(16.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Input container
                Frame::none()
                    .fill(theme.surface_color())
                    .rounding(Rounding::same(16.0))
                    .stroke(Stroke::new(1.0, theme.border_color()))
                    .inner_margin(Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        // Text input
                        let text_edit = TextEdit::multiline(&mut self.input_text)
                            .hint_text("Message Claude... (Shift+Enter for new line)")
                            .desired_width(ui.available_width() - 60.0)
                            .min_size(Vec2::new(0.0, 48.0))
                            .margin(egui::vec2(8.0, 8.0))
                            .font(egui::TextStyle::Body);

                        let response = ui.add(text_edit);

                        // Send button
                        ui.add_space(8.0);

                        let button_enabled = !self.input_text.trim().is_empty() && !self.is_loading;
                        let button_color = if button_enabled {
                            theme.primary_color()
                        } else {
                            Color32::from_rgb(60, 60, 60)
                        };

                        let send_button = ui.add_sized(
                            Vec2::new(44.0, 44.0),
                            egui::Button::new(
                                RichText::new(if self.is_loading { "⏳" } else { "➤" })
                                    .size(20.0)
                                    .color(if button_enabled { Color32::WHITE } else { theme.muted_text_color() })
                            )
                            .fill(button_color)
                            .rounding(Rounding::same(10.0))
                        );

                        // Handle send
                        let enter_pressed = response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter) && !i.modifiers.shift);

                        if (send_button.clicked() || enter_pressed) && button_enabled {
                            self.send_message();
                        }
                    });

                // Hint text
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new("Shift + Enter for new line • Enter to send")
                            .color(theme.muted_text_color())
                            .size(11.0)
                    );
                });
            });
    }

    fn send_message(&mut self) {
        let content = self.input_text.trim().to_string();
        if content.is_empty() {
            return;
        }

        // Add user message
        let user_msg = ChatMessage::user(&content);
        self.messages.push(user_msg);

        self.input_text.clear();
        self.is_loading = true;
        self.scroll_to_bottom = true;

        // Send via callback
        if let Some(callback) = &self.on_send_message {
            let api_messages = self.convert_to_api_messages();
            callback(api_messages);
        }
    }

    fn convert_to_api_messages(&self) -> Vec<crate::api::ChatMessage> {
        self.messages
            .iter()
            .filter_map(|msg| match msg.role {
                MessageRole::User => Some(crate::api::ChatMessage::user(&msg.content)),
                MessageRole::Assistant => Some(crate::api::ChatMessage::assistant(&msg.content)),
                MessageRole::System => None,
            })
            .collect()
    }

    // Public API
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        self.scroll_to_bottom = true;
    }

    pub fn clear_messages(&mut self) {
        self.messages.clear();
        self.messages.push(
            ChatMessage::assistant("Hello! I'm Claude. How can I help you today?")
        );
    }

    pub fn set_loading(&mut self, loading: bool) {
        self.is_loading = loading;
    }

    pub fn update_last_message(&mut self, content: impl Into<String>) {
        if let Some(last) = self.messages.last_mut() {
            last.content = content.into();
        }
    }
}

// Content parts for parsing
enum ContentPart<'a> {
    Text(&'a str),
    CodeBlock { language: Option<&'a str>, code: &'a str },
    InlineCode(&'a str),
}

fn split_by_code_blocks(content: &str) -> Vec<ContentPart> {
    let mut parts = Vec::new();
    let mut remaining = content;

    while !remaining.is_empty() {
        if let Some(start_idx) = remaining.find("```") {
            // Text before code block
            if start_idx > 0 {
                let text = &remaining[..start_idx];
                parts.extend(split_inline_code(text));
            }

            // Find end of code block
            let after_start = &remaining[start_idx + 3..];
            let newline_idx = after_start.find('\n').unwrap_or(0);
            let language = if newline_idx > 0 {
                Some(after_start[..newline_idx].trim())
            } else {
                None
            };

            let code_start = start_idx + 3 + newline_idx + if newline_idx > 0 { 1 } else { 0 };

            if let Some(end_idx) = remaining[code_start..].find("```") {
                let code = remaining[code_start..code_start + end_idx].trim_end();
                parts.push(ContentPart::CodeBlock { language, code });
                remaining = &remaining[code_start + end_idx + 3..];
            } else {
                // Unclosed code block
                let code = remaining[code_start..].trim_end();
                parts.push(ContentPart::CodeBlock { language, code });
                break;
            }
        } else {
            parts.extend(split_inline_code(remaining));
            break;
        }
    }

    parts
}

fn split_inline_code(text: &str) -> Vec<ContentPart> {
    let mut parts = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        if let Some(start_idx) = remaining.find('`') {
            if start_idx > 0 {
                parts.push(ContentPart::Text(&remaining[..start_idx]));
            }

            let after_start = &remaining[start_idx + 1..];
            if let Some(end_idx) = after_start.find('`') {
                let code = &after_start[..end_idx];
                parts.push(ContentPart::InlineCode(code));
                remaining = &after_start[end_idx + 1..];
            } else {
                parts.push(ContentPart::Text(&remaining[start_idx..]));
                break;
            }
        } else {
            parts.push(ContentPart::Text(remaining));
            break;
        }
    }

    parts
}
