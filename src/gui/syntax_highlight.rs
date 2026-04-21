//! Syntax Highlighting - Code block syntax highlighting using syntect

use egui::{Color32, TextFormat, FontId, FontFamily, Stroke};
use syntect::{
    easy::HighlightLines,
    highlighting::{ThemeSet, Style as SyntectStyle, Color as SyntectColor},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use std::sync::OnceLock;

/// Global syntax set (initialized once)
static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
static THEME_SET: OnceLock<ThemeSet> = OnceLock::new();

/// Initialize syntax highlighting resources
fn init_syntax_set() -> SyntaxSet {
    SyntaxSet::load_defaults_newlines()
}

fn init_theme_set() -> ThemeSet {
    ThemeSet::load_defaults()
}

/// Get or initialize syntax set
pub fn get_syntax_set() -> &'static SyntaxSet {
    SYNTAX_SET.get_or_init(init_syntax_set)
}

/// Get or initialize theme set
pub fn get_theme_set() -> &'static ThemeSet {
    THEME_SET.get_or_init(init_theme_set)
}

/// Get theme name based on dark/light mode
pub fn get_theme_name(is_dark: bool) -> &'static str {
    if is_dark {
        "base16-ocean.dark"
    } else {
        "base16-ocean.light"
    }
}

/// Highlighter for code blocks
pub struct CodeHighlighter {
    syntax_set: &'static SyntaxSet,
    theme_set: &'static ThemeSet,
}

impl Default for CodeHighlighter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeHighlighter {
    /// Create a new code highlighter
    pub fn new() -> Self {
        Self {
            syntax_set: get_syntax_set(),
            theme_set: get_theme_set(),
        }
    }

    /// Detect language from code block header
    pub fn detect_language(&self, header: &str) -> Option<&syntect::parsing::SyntaxReference> {
        let lang = header.trim().trim_start_matches("```").trim();

        if lang.is_empty() {
            return None;
        }

        // Try to find syntax by name or extension
        self.syntax_set.find_syntax_by_token(lang)
            .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
            .or_else(|| self.syntax_set.find_syntax_by_name(lang))
    }

    /// Highlight code and return formatted text
    pub fn highlight(&self,
        code: &str,
        language: Option<&str>,
        is_dark: bool,
    ) -> Vec<(egui::text::LayoutJob, usize)> {  // (job, line_count)
        let theme_name = get_theme_name(is_dark);
        let theme = &self.theme_set.themes[theme_name];

        let syntax = language
            .and_then(|lang| {
                self.syntax_set.find_syntax_by_token(lang)
                    .or_else(|| self.syntax_set.find_syntax_by_extension(lang))
            })
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());

        let mut highlighter = HighlightLines::new(syntax, theme);

        let mut lines = Vec::new();

        for line in LinesWithEndings::from(code) {
            let highlighted = highlighter.highlight_line(line, self.syntax_set)
                .unwrap_or_default();

            let mut job = egui::text::LayoutJob::default();

            for (style, text) in highlighted {
                let format = syntect_style_to_egui_format(style, is_dark);
                job.append(text.trim_end_matches('\n'), 0.0, format);
            }

            lines.push((job, 1));
        }

        lines
    }

    /// Get plain text layout (fallback when highlighting fails)
    pub fn plain_text(&self, code: &str, is_dark: bool) -> egui::text::LayoutJob {
        let mut job = egui::text::LayoutJob::default();

        let text_color = if is_dark {
            Color32::from_rgb(220, 220, 220)
        } else {
            Color32::from_rgb(50, 50, 50)
        };

        job.append(code, 0.0, TextFormat {
            font_id: FontId::new(13.0, FontFamily::Monospace),
            color: text_color,
            ..Default::default()
        });

        job
    }

    /// Get available language names for dropdown
    pub fn available_languages(&self) -> Vec<(&str, &str)> {
        let mut languages: Vec<(&str, &str)> = self.syntax_set
            .syntaxes()
            .iter()
            .map(|s| (s.name.as_str(), s.file_extensions.first().map(|e| e.as_str()).unwrap_or("")))
            .collect();

        languages.sort_by(|a, b| a.0.cmp(b.0));
        languages.dedup_by(|a, b| a.0 == b.0);

        languages
    }
}

/// Convert syntect style to egui text format
fn syntect_style_to_egui_format(style: SyntectStyle, is_dark: bool) -> TextFormat {
    let SyntectColor { r, g, b, a } = style.foreground;

    // Use font's background color or calculate one
    let background = if style.background.a > 0 {
        let bg = style.background;
        Some(Color32::from_rgba_premultiplied(bg.r, bg.g, bg.b, bg.a))
    } else {
        None
    };

    let color = Color32::from_rgba_premultiplied(r, g, b, a);

    // Adjust colors for better visibility in dark/light themes
    let adjusted_color = if is_dark {
        // Boost brightness for dark themes
        boost_brightness(color, 1.1)
    } else {
        color
    };

    let underline = if style.font_style == syntect::highlighting::FontStyle::UNDERLINE {
        Stroke::new(1.0, adjusted_color)
    } else {
        Stroke::NONE
    };

    TextFormat {
        font_id: FontId::new(13.0, FontFamily::Monospace),
        color: adjusted_color,
        background: background.unwrap_or(Color32::TRANSPARENT),
        italics: style.font_style == syntect::highlighting::FontStyle::ITALIC,
        underline,
        ..Default::default()
    }
}

/// Boost color brightness slightly
fn boost_brightness(color: Color32, factor: f32) -> Color32 {
    let [r, g, b, a] = color.to_array();

    Color32::from_rgba_premultiplied(
        (r as f32 * factor).min(255.0) as u8,
        (g as f32 * factor).min(255.0) as u8,
        (b as f32 * factor).min(255.0) as u8,
        a,
    )
}

/// Get a simple colored code block without full highlighting
pub fn get_simple_code_style(language: &str, is_dark: bool) -> (Color32, Color32) {
    // (background, text_color)

    let language_colors: [(&str, (Color32, Color32)); 12] = [
        ("rust", (Color32::from_rgb(50, 30, 30), Color32::from_rgb(255, 180, 180))),
        ("python", (Color32::from_rgb(30, 50, 70), Color32::from_rgb(180, 210, 255))),
        ("javascript", (Color32::from_rgb(50, 50, 30), Color32::from_rgb(255, 255, 180))),
        ("typescript", (Color32::from_rgb(30, 50, 70), Color32::from_rgb(100, 150, 255))),
        ("html", (Color32::from_rgb(50, 30, 30), Color32::from_rgb(255, 150, 150))),
        ("css", (Color32::from_rgb(30, 40, 60), Color32::from_rgb(150, 200, 255))),
        ("json", (Color32::from_rgb(40, 45, 40), Color32::from_rgb(180, 255, 180))),
        ("yaml", (Color32::from_rgb(40, 50, 40), Color32::from_rgb(200, 255, 200))),
        ("toml", (Color32::from_rgb(50, 40, 30), Color32::from_rgb(255, 200, 150))),
        ("bash", (Color32::from_rgb(30, 30, 30), Color32::from_rgb(200, 200, 200))),
        ("shell", (Color32::from_rgb(30, 30, 30), Color32::from_rgb(200, 200, 200))),
        ("markdown", (Color32::from_rgb(45, 45, 45), Color32::from_rgb(220, 220, 220))),
    ];

    let lang_lower = language.to_lowercase();

    for (lang, colors) in &language_colors {
        if lang_lower.contains(lang) {
            return *colors;
        }
    }

    // Default colors
    if is_dark {
        (Color32::from_rgb(35, 35, 35), Color32::from_rgb(220, 220, 220))
    } else {
        (Color32::from_rgb(245, 245, 245), Color32::from_rgb(50, 50, 50))
    }
}

/// Format a code block with optional highlighting
pub fn format_code_block(
    ui: &mut egui::Ui,
    code: &str,
    language: Option<&str>,
    is_dark: bool,
) {
    let highlighter = CodeHighlighter::new();
    let (bg_color, text_color) = language
        .map(|l| get_simple_code_style(l, is_dark))
        .unwrap_or_else(|| {
            if is_dark {
                (Color32::from_rgb(35, 35, 35), Color32::from_rgb(220, 220, 220))
            } else {
                (Color32::from_rgb(245, 245, 245), Color32::from_rgb(50, 50, 50))
            }
        });

    // Code block container
    egui::Frame::none()
        .fill(bg_color)
        .inner_margin(12.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());

            // Language badge
            if let Some(lang) = language {
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(lang.to_uppercase())
                            .size(10.0)
                            .color(text_color.linear_multiply(0.7))
                            .monospace()
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.small_button("📋").clicked() {
                            ui.output_mut(|o| o.copied_text = code.to_string());
                        }
                    });
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);
            }

            // Try syntax highlighting, fallback to plain text
            let lines = highlighter.highlight(code, language, is_dark);

            if lines.is_empty() {
                // Fallback: plain text
                let job = highlighter.plain_text(code, is_dark);
                ui.label(job);
            } else {
                // Render highlighted lines
                for (line_job, _) in lines {
                    ui.label(line_job);
                }
            }
        });
}
