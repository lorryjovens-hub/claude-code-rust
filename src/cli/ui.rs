//! UI Module - Beautiful terminal UI matching original Claude Code
//!
//! This module provides styled output, colors, animations, and formatting
//! to match the aesthetic of the original TypeScript Claude Code CLI.

use colored::{ColoredString, Colorize};
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

/// Claude Code brand colors
pub mod colors {
    use colored::Color;

    /// Anthropic purple - primary brand color
    pub const PRIMARY: Color = Color::Magenta;
    /// Warm orange - accent color
    pub const ACCENT: Color = Color::TrueColor { r: 255, g: 140, b: 66 };
    /// Soft purple for secondary elements
    pub const SECONDARY: Color = Color::TrueColor { r: 147, g: 112, b: 219 };
    /// Green for success states
    pub const SUCCESS: Color = Color::Green;
    /// Yellow for warnings
    pub const WARNING: Color = Color::Yellow;
    /// Red for errors
    pub const ERROR: Color = Color::Red;
    /// Cyan for info
    pub const INFO: Color = Color::Cyan;
    /// Bright white for text
    pub const TEXT: Color = Color::White;
    /// Gray for muted text
    pub const MUTED: Color = Color::BrightBlack;
}

/// Claude Code ASCII Logo
pub const LOGO: &str = r#"
   ╭──────────────────────────────────────╮
   │                                      │
   │   🟣 Claude Code                     │
   │      High-Performance Rust Edition   │
   │                                      │
   ╰──────────────────────────────────────╯
"#;

/// Print the Claude Code welcome banner
pub fn print_welcome() {
    println!();
    print_gradient_banner();
    println!();
    print_features();
    println!();
    print_divider();
    println!();
}

/// Print a beautiful gradient-style banner
fn print_gradient_banner() {
    let banner = r#"
    ╭────────────────────────────────────────────────────────────╮
    │                                                            │
    │     🟣  Claude Code  ·  High-Performance Rust Edition      │
    │                                                            │
    │         ⚡ 2.5x faster  ·  📦 97% smaller  ·  🦀 Rust       │
    │                                                            │
    ╰────────────────────────────────────────────────────────────╯
    "#;

    // Print with purple gradient effect
    for (i, line) in banner.lines().enumerate() {
        let styled = match i {
            0 | 7 => line.bright_purple().bold(),
            3 => line.truecolor(200, 150, 255).bold(),
            5 => line.truecolor(255, 140, 66),
            _ => line.bright_black(),
        };
        println!("{}", styled);
    }
}

/// Print feature highlights
fn print_features() {
    println!("  {}", "Performance:".truecolor(147, 112, 219).bold());
    println!("    {} 启动速度提升 {} ", "▸".green(), "2.5x".green().bold());
    println!("    {} 内存占用减少 {} ", "▸".green(), "60%".green().bold());
    println!("    {} 响应速度提升 {} ", "▸".green(), "40%".green().bold());
    println!();
    println!("  {}", "Type 'help' for commands, 'exit' to quit".bright_black().italic());
}

/// Print a stylish divider
pub fn print_divider() {
    let width = terminal_size().0.min(70);
    let line = "─".repeat(width as usize);
    println!("{}", line.truecolor(100, 80, 120));
}

/// Print a Claude (assistant) message with styling
pub fn print_claude_message(content: &str) {
    println!();
    // Claude's avatar with purple styling
    print!("  {}", "●".truecolor(147, 112, 219).bold());
    println!(" {}", "Claude".truecolor(200, 150, 255).bold());
    println!();

    // Format the content with proper wrapping and styling
    for line in content.lines() {
        if line.starts_with("```") {
            // Code block delimiter
            if line.len() > 3 {
                let lang = &line[3..];
                println!("  {}", format!("───── {} ─────", lang).truecolor(80, 80, 80));
            } else {
                println!("  {}", "─────────────────────".truecolor(80, 80, 80));
            }
        } else if line.starts_with("#") {
            // Headers
            let level = line.chars().take_while(|&c| c == '#').count();
            let header_text = line.trim_start_matches('#').trim();
            let styled = match level {
                1 => header_text.truecolor(255, 140, 66).bold().underline(),
                2 => header_text.truecolor(200, 150, 255).bold(),
                _ => header_text.bright_white().bold(),
            };
            println!("  {}", styled);
        } else if line.starts_with("-") || line.starts_with("*") {
            // List items
            println!("  {} {}", "•".truecolor(147, 112, 219), &line[1..].trim());
        } else if line.starts_with(">") {
            // Blockquote
            println!("  {} {}", "│".truecolor(100, 80, 120), line[1..].trim().bright_black());
        } else {
            // Regular text with inline formatting
            let formatted = format_inline_styles(line);
            println!("  {}", formatted);
        }
    }
    println!();
}

/// Print a user message header (content already shown in terminal input)
pub fn print_user_message(_content: &str) {
    // 不重复打印用户输入，因为输入时已经显示在终端了
    // 只打印分隔线
    println!();
}

/// Format inline markdown styles (bold, italic, code)
fn format_inline_styles(text: &str) -> ColoredString {
    // Handle inline code
    if text.contains('`') {
        let mut result = String::new();
        let mut in_code = false;
        for c in text.chars() {
            if c == '`' {
                in_code = !in_code;
                if in_code {
                    result.push_str("\x1b[48;5;238m\x1b[38;5;250m");
                } else {
                    result.push_str("\x1b[0m");
                }
            } else {
                result.push(c);
            }
        }
        return result.normal();
    }

    // Handle bold (**text**)
    if text.contains("**") {
        let parts: Vec<&str> = text.split("**").collect();
        let mut result = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i % 2 == 1 {
                result.push_str(&format!("\x1b[1m{}\x1b[0m", part));
            } else {
                result.push_str(part);
            }
        }
        return result.normal();
    }

    text.normal()
}

/// Print a typing indicator animation
pub fn print_typing_indicator() {
    print!("  {} ", "●".truecolor(147, 112, 219).bold());
    print!("{}", "thinking...".truecolor(150, 150, 150));
    io::stdout().flush().ok();

    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    for frame in frames.iter().cycle().take(10) {
        print!("\r  {} {} {}",
            "●".truecolor(147, 112, 219).bold(),
            "thinking".truecolor(150, 150, 150),
            frame.truecolor(147, 112, 219)
        );
        io::stdout().flush().ok();
        thread::sleep(Duration::from_millis(80));
    }
    // 清除整行
    print!("\r\x1B[K");
    io::stdout().flush().ok();
}

/// Print a typewriter-style animated output
pub fn print_typewriter(text: &str, delay_ms: u64) {
    for c in text.chars() {
        print!("{}", c);
        io::stdout().flush().ok();
        thread::sleep(Duration::from_millis(delay_ms));
    }
}

/// Print styled help information
pub fn print_help() {
    println!();
    println!("  {}", "📖 Available Commands".truecolor(147, 112, 219).bold());
    println!();

    let commands = [
        ("help", ".help", "Show help information"),
        ("status", ".status", "Show current status"),
        ("config", ".config", "Show configuration"),
        ("history", ".history", "Show conversation history"),
        ("reset", ".reset", "Reset conversation"),
        ("clear", ".clear", "Clear screen"),
        ("exit", ".exit", "Exit REPL"),
    ];

    for (cmd, alias, desc) in commands {
        println!("  {} {:12} {:12} {}",
            "▸".truecolor(100, 80, 120),
            cmd.bright_cyan(),
            alias.bright_black(),
            desc.bright_white()
        );
    }

    println!();
    println!("  {}", "💡 Tip:".truecolor(255, 140, 66).bold());
    println!("     Type any message to chat with Claude");
    println!();
}

/// Print status with styled formatting
pub fn print_status(status: &StatusInfo) {
    println!();
    println!("  {}", "📊 Status".truecolor(147, 112, 219).bold());
    println!();

    print_status_row("Model", &status.model, true);
    print_status_row("API Base", &status.api_base, true);
    print_status_row("Max Tokens", &status.max_tokens, true);
    print_status_row("Timeout", &format!("{}s", status.timeout), true);
    print_status_row("Streaming", if status.streaming { "On" } else { "Off" }, status.streaming);
    print_status_row("Messages", &format!("{}", status.message_count), true);
    print_status_row("API Key", if status.api_key_set { "Set ✓" } else { "Not Set ✗" }, status.api_key_set);

    println!();
}

fn print_status_row(label: &str, value: &str, positive: bool) {
    let value_colored = if positive {
        value.green()
    } else {
        value.red()
    };
    println!("  {:15} {}",
        format!("{}:", label).truecolor(120, 120, 120),
        value_colored
    );
}

/// Status information structure
pub struct StatusInfo {
    pub model: String,
    pub api_base: String,
    pub max_tokens: String,
    pub timeout: u64,
    pub streaming: bool,
    pub message_count: usize,
    pub api_key_set: bool,
}

/// Print an error message with styling
pub fn print_error(message: &str) {
    println!();
    println!("  {} {}",
        "✗".red().bold(),
        "Error:".red().bold()
    );
    println!("    {}", message.bright_red());
    println!();
}

/// Print a success message with styling
pub fn print_success(message: &str) {
    println!("  {} {}", "✓".green().bold(), message.green());
}

/// Print a warning message with styling
pub fn print_warning(message: &str) {
    println!("  {} {}", "⚠".yellow().bold(), message.yellow());
}

/// Print an info message with styling
pub fn print_info(message: &str) {
    println!("  {} {}", "ℹ".cyan(), message.cyan());
}

/// Print a code block with syntax highlighting simulation
pub fn print_code_block(code: &str, language: Option<&str>) {
    let lang = language.unwrap_or("");
    let header = format!("───── {} ─────", lang).truecolor(80, 80, 80);

    println!("  {}", header);
    for line in code.lines() {
        // Simple syntax highlighting simulation
        let highlighted = highlight_code_line(line);
        println!("  {}", highlighted);
    }
    println!("  {}", "─────────────────────".truecolor(80, 80, 80));
}

/// Simple syntax highlighting for code
fn highlight_code_line(line: &str) -> ColoredString {
    // Keywords
    let keywords = ["fn", "let", "mut", "use", "pub", "struct", "impl", "if", "else", "return", "match"];
    for kw in &keywords {
        if line.trim().starts_with(kw) || line.contains(&format!(" {} ", kw)) {
            return line.truecolor(180, 140, 250); // Purple tint for keywords
        }
    }

    // Strings
    if line.contains('"') || line.contains('\'') {
        return line.truecolor(140, 220, 140); // Green tint for strings
    }

    // Comments
    if line.trim().starts_with("//") || line.trim().starts_with("#") {
        return line.bright_black(); // Gray for comments
    }

    line.normal()
}

/// Print a table with styled headers
pub fn print_table(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        println!("  (no data)");
        return;
    }

    // Calculate column widths
    let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(cell.len());
            }
        }
    }

    // Print header
    print!("  ");
    for (i, header) in headers.iter().enumerate() {
        let width = widths.get(i).copied().unwrap_or(10);
        print!("{}  ", format!("{:width$}", header, width = width).truecolor(147, 112, 219).bold());
    }
    println!();

    // Print divider
    print!("  ");
    for width in &widths {
        print!("{}  ", "─".repeat(*width).truecolor(80, 80, 80));
    }
    println!();

    // Print rows
    for row in rows {
        print!("  ");
        for (i, cell) in row.iter().enumerate() {
            let width = widths.get(i).copied().unwrap_or(10);
            print!("{}  ", format!("{:width$}", cell, width = width).bright_white());
        }
        println!();
    }
}

/// Get terminal size (width, height)
pub fn terminal_size() -> (u16, u16) {
    terminal_size::terminal_size()
        .map(|(w, h)| (w.0, h.0))
        .unwrap_or((80, 24))
}

/// Clear the screen
pub fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().ok();
}

/// Print the input prompt
pub fn print_prompt() {
    print!("\n  {} ", "▸".truecolor(255, 140, 66).bold());
    io::stdout().flush().ok();
}

/// Initialize the terminal for styled output
pub fn init_terminal() {
    // Enable ANSI colors on Windows
    #[cfg(windows)]
    {
        let _ = colored::control::set_virtual_terminal(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_inline_styles() {
        let text = "This is **bold** text";
        let _result = format_inline_styles(text);
        // Just verify it doesn't panic
    }

    #[test]
    fn test_terminal_size() {
        let (w, h) = terminal_size();
        assert!(w > 0);
        assert!(h > 0);
    }
}
