//! Tool Call Visualization - Display tool calls in chat interface
//!
//! Recreates Claude Code's tool call cards with:
//! - Expandable/collapsible details
//! - File read/write visualization
//! - Bash command execution display
//! - File diff viewer
//! - Tool result display

use egui::{Color32, RichText, Ui, Frame, Rounding, Margin, Stroke, Layout, Align};
use std::time::{SystemTime, UNIX_EPOCH};

/// A tool call instance
#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
    pub status: ToolCallStatus,
    pub result: Option<String>,
    pub timestamp: u64,
    pub expanded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCallStatus {
    Pending,
    Running,
    Success,
    Error,
}

impl ToolCall {
    pub fn new(name: impl Into<String>, arguments: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            arguments: arguments.into(),
            status: ToolCallStatus::Pending,
            result: None,
            timestamp,
            expanded: true,
        }
    }

    pub fn with_result(mut self, result: impl Into<String>) -> Self {
        self.result = Some(result.into());
        self.status = ToolCallStatus::Success;
        self
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.result = Some(error.into());
        self.status = ToolCallStatus::Error;
        self
    }
}

/// Tool call manager
pub struct ToolCallManager {
    pub calls: Vec<ToolCall>,
}

impl Default for ToolCallManager {
    fn default() -> Self {
        Self {
            calls: Vec::new(),
        }
    }
}

impl ToolCallManager {
    pub fn add_call(&mut self, call: ToolCall) {
        self.calls.push(call);
    }

    pub fn update_call_result(&mut self, id: &str, result: String, success: bool) {
        if let Some(call) = self.calls.iter_mut().find(|c| c.id == id) {
            call.result = Some(result);
            call.status = if success {
                ToolCallStatus::Success
            } else {
                ToolCallStatus::Error
            };
        }
    }

    pub fn render_tool_calls(&mut self, ui: &mut Ui, theme: &super::Theme) {
        // 为了避免借用冲突，我们先克隆需要的数据
        let calls = std::mem::take(&mut self.calls);
        for mut call in calls {
            self.render_tool_call_card(ui, &mut call, theme);
            self.calls.push(call);
            ui.add_space(8.0);
        }
    }

    fn render_tool_call_card(&self, ui: &mut Ui, call: &mut ToolCall, theme: &super::Theme) {
        let (icon, title, bg_color, border_color) = match call.name.as_str() {
            "read_file" | "file_read" | "view" => {
                ("📖", "View", theme.surface_color(), Color32::from_rgb(100, 181, 246))
            }
            "write_file" | "file_write" | "edit" => {
                ("📝", "Edit", theme.surface_color(), Color32::from_rgb(76, 175, 80))
            }
            "create_file" | "create" => {
                ("✨", "Create", theme.surface_color(), Color32::from_rgb(156, 39, 176))
            }
            "bash" | "execute" | "execute_bash" => {
                ("⚡", "Bash", theme.surface_color(), Color32::from_rgb(255, 152, 0))
            }
            "search" | "grep" | "search_files" => {
                ("🔍", "Search", theme.surface_color(), Color32::from_rgb(33, 150, 243))
            }
            "list_directory" | "ls" | "list" => {
                ("📁", "List", theme.surface_color(), Color32::from_rgb(121, 85, 72))
            }
            _ => {
                ("🔧", call.name.as_str(), theme.surface_color(), theme.border_color())
            }
        };

        // Status indicator
        let status_icon = match call.status {
            ToolCallStatus::Pending => "⏳",
            ToolCallStatus::Running => "🔄",
            ToolCallStatus::Success => "✅",
            ToolCallStatus::Error => "❌",
        };

        let status_color = match call.status {
            ToolCallStatus::Pending => Color32::from_rgb(255, 193, 7),
            ToolCallStatus::Running => Color32::from_rgb(33, 150, 243),
            ToolCallStatus::Success => Color32::from_rgb(76, 175, 80),
            ToolCallStatus::Error => Color32::from_rgb(244, 67, 54),
        };

        Frame::none()
            .fill(bg_color)
            .rounding(Rounding::same(8.0))
            .stroke(Stroke::new(1.5, border_color))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Header
                ui.horizontal(|ui| {
                    ui.add_space(12.0);

                    // Tool icon
                    ui.label(RichText::new(icon).size(18.0));
                    ui.add_space(8.0);

                    // Tool name
                    ui.label(
                        RichText::new(title)
                            .strong()
                            .color(theme.text_color())
                            .size(14.0)
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.add_space(12.0);

                        // Status indicator
                        ui.label(
                            RichText::new(status_icon)
                                .color(status_color)
                                .size(16.0)
                        );

                        ui.add_space(8.0);

                        // Expand/collapse button
                        let expand_text = if call.expanded { "▼" } else { "▶" };
                        if ui.button(expand_text).clicked() {
                            call.expanded = !call.expanded;
                        }
                    });
                });

                // Expanded content
                if call.expanded {
                    ui.add_space(8.0);
                    ui.add(egui::Separator::default().spacing(4.0));
                    ui.add_space(8.0);

                    // Parse and display arguments
                    if let Ok(args) = serde_json::from_str::<serde_json::Value>(&call.arguments) {
                        self.render_tool_arguments(ui, &call.name, &args, theme);
                    } else {
                        ui.monospace(
                            RichText::new(&call.arguments)
                                .color(theme.muted_text_color())
                                .size(12.0)
                        );
                    }

                    // Result
                    if let Some(result) = &call.result {
                        ui.add_space(8.0);
                        ui.add(egui::Separator::default().spacing(4.0));
                        ui.add_space(8.0);

                        ui.label(
                            RichText::new("Result:")
                                .strong()
                                .color(theme.text_color())
                                .size(12.0)
                        );

                        ui.add_space(4.0);

                        // Result with styling
                        let result_color = if call.status == ToolCallStatus::Error {
                            theme.error_color()
                        } else {
                            theme.success_color()
                        };

                        Frame::none()
                            .fill(Color32::from_rgb(25, 25, 25))
                            .rounding(Rounding::same(4.0))
                            .inner_margin(Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());

                                // Truncate if too long
                                let display_result = if result.len() > 2000 {
                                    format!("{}... (truncated)", &result[..2000])
                                } else {
                                    result.clone()
                                };

                                ui.monospace(
                                    RichText::new(display_result)
                                        .color(result_color)
                                        .size(11.0)
                                );
                            });
                    }
                }

                ui.add_space(8.0);
            });
    }

    fn render_tool_arguments(&self, ui: &mut Ui, tool_name: &str, args: &serde_json::Value, theme: &super::Theme) {
        match tool_name {
            "read_file" | "file_read" | "view" => {
                if let Some(path) = args["path"].as_str() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Path:").color(theme.muted_text_color()).size(12.0));
                        ui.monospace(
                            RichText::new(path)
                                .color(theme.primary_color())
                                .size(12.0)
                        );
                    });
                }
                if let Some(offset) = args["offset"].as_i64() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Offset:").color(theme.muted_text_color()).size(12.0));
                        ui.label(RichText::new(offset.to_string()).color(theme.text_color()).size(12.0));
                    });
                }
                if let Some(limit) = args["limit"].as_i64() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Limit:").color(theme.muted_text_color()).size(12.0));
                        ui.label(RichText::new(limit.to_string()).color(theme.text_color()).size(12.0));
                    });
                }
            }
            "write_file" | "file_write" | "edit" | "create_file" | "create" => {
                if let Some(path) = args["path"].as_str() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Path:").color(theme.muted_text_color()).size(12.0));
                        ui.monospace(
                            RichText::new(path)
                                .color(theme.primary_color())
                                .size(12.0)
                        );
                    });
                }
                if let Some(content) = args["content"].as_str() {
                    ui.add_space(4.0);
                    ui.label(RichText::new("Content:").color(theme.muted_text_color()).size(12.0));

                    Frame::none()
                        .fill(Color32::from_rgb(25, 25, 25))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::same(8.0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            let preview = if content.len() > 500 {
                                format!("{}... ({} bytes)", &content[..500], content.len())
                            } else {
                                content.to_string()
                            };
                            ui.monospace(
                                RichText::new(preview)
                                    .color(theme.code_text_color())
                                    .size(11.0)
                            );
                        });
                }
            }
            "bash" | "execute" | "execute_bash" => {
                if let Some(command) = args["command"].as_str() {
                    ui.label(RichText::new("Command:").color(theme.muted_text_color()).size(12.0));

                    Frame::none()
                        .fill(Color32::from_rgb(30, 20, 10))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(Margin::same(8.0))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(100, 70, 40)))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("$").color(Color32::from_rgb(212, 165, 116)).size(12.0));
                                ui.add_space(4.0);
                                ui.monospace(
                                    RichText::new(command)
                                        .color(Color32::from_rgb(230, 230, 230))
                                        .size(12.0)
                                );
                            });
                        });
                }
                if let Some(timeout) = args["timeout"].as_i64() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Timeout:").color(theme.muted_text_color()).size(12.0));
                        ui.label(RichText::new(format!("{}s", timeout)).color(theme.text_color()).size(12.0));
                    });
                }
            }
            "search" | "grep" | "search_files" => {
                if let Some(query) = args["query"].as_str() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Query:").color(theme.muted_text_color()).size(12.0));
                        ui.monospace(
                            RichText::new(format!("'{}'", query))
                                .color(Color32::from_rgb(255, 193, 7))
                                .size(12.0)
                        );
                    });
                }
                if let Some(path) = args["path"].as_str() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("In:").color(theme.muted_text_color()).size(12.0));
                        ui.monospace(
                            RichText::new(path)
                                .color(theme.primary_color())
                                .size(12.0)
                        );
                    });
                }
            }
            "list_directory" | "ls" | "list" => {
                if let Some(path) = args["path"].as_str() {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Directory:").color(theme.muted_text_color()).size(12.0));
                        ui.monospace(
                            RichText::new(path)
                                .color(theme.primary_color())
                                .size(12.0)
                        );
                    });
                }
            }
            _ => {
                // Generic display for unknown tools
                if let Some(obj) = args.as_object() {
                    for (key, value) in obj {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(format!("{}:", key)).color(theme.muted_text_color()).size(12.0));
                            ui.monospace(
                                RichText::new(format!("{}", value))
                                    .color(theme.text_color())
                                    .size(12.0)
                            );
                        });
                    }
                }
            }
        }
    }

    /// Render a diff view for file changes
    pub fn render_diff(ui: &mut Ui, old_content: &str, new_content: &str, theme: &super::Theme) {
        use similar::{ChangeTag, TextDiff};

        let diff = TextDiff::from_lines(old_content, new_content);

        Frame::none()
            .fill(Color32::from_rgb(20, 20, 20))
            .rounding(Rounding::same(4.0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                for change in diff.iter_all_changes() {
                    let (prefix, color) = match change.tag() {
                        ChangeTag::Delete => ("-", Color32::from_rgb(255, 100, 100)),
                        ChangeTag::Insert => ("+", Color32::from_rgb(100, 255, 100)),
                        ChangeTag::Equal => (" ", Color32::from_rgb(150, 150, 150)),
                    };

                    ui.horizontal(|ui| {
                        ui.monospace(
                            RichText::new(prefix)
                                .color(color)
                                .size(11.0)
                        );
                        ui.add_space(4.0);
                        ui.monospace(
                            RichText::new(change.value().trim_end())
                                .color(color)
                                .size(11.0)
                        );
                    });
                }
            });
    }
}

/// Create a sample tool call for demo purposes
pub fn demo_tool_calls() -> Vec<ToolCall> {
    vec![
        ToolCall::new(
            "read_file",
            r#"{"path": "src/main.rs", "offset": 1, "limit": 50}"#,
        ).with_result("use std::io;\n\nfn main() {\n    println!(\"Hello, World!\");\n}"),

        ToolCall::new(
            "search",
            r#"{"query": "fn main", "path": "src"}"#,
        ).with_result("src/main.rs:3: fn main() {\nsrc/lib.rs:10: pub fn main() -> Result<()>"),

        ToolCall::new(
            "bash",
            r#"{"command": "cargo build --release", "timeout": 120}"#,
        ).with_result("Compiling...\nFinished release [optimized] target(s) in 12.34s"),
    ]
}
