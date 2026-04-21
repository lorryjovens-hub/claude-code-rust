//! REPL Module - Interactive Read-Eval-Print Loop
//!
//! Beautiful REPL interface matching the original Claude Code aesthetic

use crate::api::{ApiClient, ChatMessage};
use crate::cli::ui;
use crate::state::AppState;
use colored::Colorize;
use std::io::{self, BufRead, Write};

pub struct Repl {
    state: AppState,
    conversation_history: Vec<ChatMessage>,
}

impl Repl {
    pub fn new(state: AppState) -> Self {
        ui::init_terminal();
        Self {
            state,
            conversation_history: Vec::new(),
        }
    }

    pub fn start(&mut self, initial_prompt: Option<String>) -> anyhow::Result<()> {
        ui::print_welcome();

        if let Some(prompt) = initial_prompt {
            self.process_input(&prompt)?;
        }

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        loop {
            ui::print_prompt();
            stdout.flush()?;

            let mut input = String::new();
            stdin.lock().read_line(&mut input)?;
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            match input {
                "exit" | "quit" | ".exit" | ":q" => {
                    println!("\n  {} {}\n",
                        "👋".yellow(),
                        "Goodbye!".truecolor(255, 140, 66).bold()
                    );
                    break;
                }
                "help" | ".help" | ":h" => ui::print_help(),
                "status" | ".status" => self.print_status(),
                "clear" | ".clear" | ":c" => ui::clear_screen(),
                "history" | ".history" => self.print_history(),
                "reset" | ".reset" => self.reset_conversation(),
                "config" | ".config" => self.print_config(),
                _ => self.process_input(input)?,
            }
        }

        Ok(())
    }

    fn process_input(&mut self, input: &str) -> anyhow::Result<()> {
        // Show user message with styling
        ui::print_user_message(input);

        let client = ApiClient::new(self.state.settings.clone());

        let api_key = match client.get_api_key() {
            Some(key) => key,
            None => {
                ui::print_error("API key not configured\n\nSet it with:\n  claude-code config set api_key \"your-api-key\"");
                return Ok(());
            }
        };

        self.conversation_history.push(ChatMessage::user(input));

        // Show typing indicator
        ui::print_typing_indicator();

        let messages = self.conversation_history.clone();
        let base_url = client.get_base_url();
        let model = client.get_model().to_string();
        let max_tokens = self.state.settings.api.max_tokens;

        let request_body = serde_json::json!({
            "model": model,
            "messages": messages,
            "max_tokens": max_tokens,
            "stream": false,
            "temperature": 0.7
        });

        let http_client = reqwest::blocking::Client::new();
        let url = format!("{}/v1/chat/completions", base_url);

        match http_client
            .post(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
        {
            Ok(resp) => {
                if !resp.status().is_success() {
                    let status = resp.status();
                    let body = resp.text().unwrap_or_default();
                    ui::print_error(&format!("API error ({}): {}", status, body));
                    return Ok(());
                }

                let json: serde_json::Value = resp.json().unwrap_or(serde_json::json!({}));

                if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                    if let Some(choice) = choices.first() {
                        if let Some(content) = choice.get("message")
                            .and_then(|m| m.get("content"))
                            .and_then(|c| c.as_str())
                        {
                            // Print Claude's response with beautiful formatting
                            ui::print_claude_message(content);
                            self.conversation_history.push(ChatMessage::assistant(content.to_string()));

                            // Print token usage if available
                            if let Some(usage) = json.get("usage") {
                                if let (Some(prompt), Some(completion)) = (
                                    usage.get("prompt_tokens").and_then(|t| t.as_u64()),
                                    usage.get("completion_tokens").and_then(|t| t.as_u64()),
                                ) {
                                    let total = prompt + completion;
                                    println!("  {} {} prompt · {} generated · {} total",
                                        "◦".truecolor(100, 100, 100),
                                        prompt.to_string().truecolor(150, 150, 150),
                                        completion.to_string().truecolor(150, 150, 150),
                                        total.to_string().truecolor(180, 180, 180)
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => {
                ui::print_error(&format!("Request failed: {}", e));
            }
        }

        Ok(())
    }

    fn print_status(&self) {
        let status = ui::StatusInfo {
            model: self.state.settings.model.clone(),
            api_base: self.state.settings.api.base_url.clone(),
            max_tokens: self.state.settings.api.max_tokens.to_string(),
            timeout: self.state.settings.api.timeout,
            streaming: self.state.settings.api.streaming,
            message_count: self.conversation_history.len(),
            api_key_set: self.state.settings.api.get_api_key().is_some(),
        };
        ui::print_status(&status);
    }

    fn print_history(&self) {
        println!();
        if self.conversation_history.is_empty() {
            println!("  {} {}",
                "◦".truecolor(100, 100, 100),
                "No conversation history".bright_black()
            );
        } else {
            println!("  {} {}",
                "◦".truecolor(147, 112, 219),
                format!("Conversation history ({} messages)", self.conversation_history.len())
                    .truecolor(147, 112, 219).bold()
            );
            println!();

            for (i, msg) in self.conversation_history.iter().enumerate() {
                let (icon, color) = match msg.role.as_str() {
                    "user" => ("●", "truecolor(255, 140, 66)"),
                    "assistant" => ("●", "truecolor(147, 112, 219)"),
                    _ => ("●", "bright_black"),
                };

                let role_label = match msg.role.as_str() {
                    "user" => "You".truecolor(255, 180, 100),
                    "assistant" => "Claude".truecolor(200, 150, 255),
                    _ => "Unknown".bright_black(),
                };

                let content = msg.content.as_deref().unwrap_or("");
                let preview: String = content.chars().take(50).collect();
                let suffix = if content.len() > 50 { "..." } else { "" };

                println!("  {}. {}  {}{}",
                    (i + 1).to_string().truecolor(100, 100, 100),
                    role_label,
                    preview.bright_white(),
                    suffix.bright_black()
                );
            }
        }
        println!();
    }

    fn print_config(&self) {
        println!();
        println!("  {} {}",
            "⚙".truecolor(147, 112, 219),
            "Configuration".truecolor(147, 112, 219).bold()
        );
        println!();

        match serde_json::to_string_pretty(&self.state.settings) {
            Ok(json) => {
                for line in json.lines() {
                    println!("  {}", line.bright_white());
                }
            }
            Err(_) => {
                ui::print_error("Failed to serialize configuration");
            }
        }
        println!();
    }

    fn reset_conversation(&mut self) {
        self.conversation_history.clear();
        ui::print_success("Conversation reset");
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repl_creation() {
        let state = AppState::default();
        let repl = Repl::new(state);
        assert!(repl.conversation_history.is_empty());
    }
}
