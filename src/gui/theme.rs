//! Theme System - Claude Code UI themes
//!
//! Based on Claude.ai design system with warm orange/brown accent colors

use egui::{Color32, FontFamily, FontId, TextStyle, Visuals, Stroke, Rounding};

/// Application theme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
    System,
}

/// Claude brand colors
pub mod colors {
    use super::Color32;

    // Primary accent - warm orange/brown
    pub const CLAUDE_ORANGE: Color32 = Color32::from_rgb(212, 165, 116);
    pub const CLAUDE_ORANGE_DARK: Color32 = Color32::from_rgb(180, 135, 90);
    pub const CLAUDE_ORANGE_LIGHT: Color32 = Color32::from_rgb(235, 195, 150);

    // Background colors
    pub const BG_DARKEST: Color32 = Color32::from_rgb(13, 13, 13);      // #0D0D0D
    pub const BG_DARKER: Color32 = Color32::from_rgb(18, 18, 18);      // #121212
    pub const BG_DARK: Color32 = Color32::from_rgb(26, 26, 26);        // #1A1A1A
    pub const BG_SURFACE: Color32 = Color32::from_rgb(35, 35, 35);     // #232323
    pub const BG_ELEVATED: Color32 = Color32::from_rgb(45, 45, 45);    // #2D2D2D

    // Border colors
    pub const BORDER_DARK: Color32 = Color32::from_rgb(42, 42, 42);    // #2A2A2A
    pub const BORDER: Color32 = Color32::from_rgb(55, 55, 55);         // #373737
    pub const BORDER_LIGHT: Color32 = Color32::from_rgb(70, 70, 70);   // #464646

    // Text colors
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(232, 232, 232);   // #E8E8E8
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(180, 180, 180); // #B4B4B4
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(120, 120, 120);     // #787878
    pub const TEXT_DISABLED: Color32 = Color32::from_rgb(80, 80, 80);     // #505050

    // Semantic colors
    pub const SUCCESS: Color32 = Color32::from_rgb(76, 175, 80);
    pub const WARNING: Color32 = Color32::from_rgb(255, 152, 0);
    pub const ERROR: Color32 = Color32::from_rgb(244, 67, 54);
    pub const INFO: Color32 = Color32::from_rgb(100, 181, 246);

    // Code block colors
    pub const CODE_BG: Color32 = Color32::from_rgb(30, 30, 30);
    pub const CODE_TEXT: Color32 = Color32::from_rgb(220, 220, 220);
    pub const INLINE_CODE_BG: Color32 = Color32::from_rgb(50, 50, 50);

    // Light theme colors
    pub const BG_LIGHT: Color32 = Color32::from_rgb(250, 250, 250);
    pub const BG_LIGHT_SURFACE: Color32 = Color32::from_rgb(255, 255, 255);
    pub const BORDER_LIGHT_MODE: Color32 = Color32::from_rgb(224, 224, 224);
    pub const TEXT_LIGHT_PRIMARY: Color32 = Color32::from_rgb(33, 33, 33);
    pub const TEXT_LIGHT_SECONDARY: Color32 = Color32::from_rgb(100, 100, 100);
}

impl Theme {
    /// Apply theme to egui context
    pub fn apply(&self, ctx: &egui::Context) {
        let visuals = match self {
            Theme::Light => self.light_visuals(),
            Theme::Dark => self.dark_visuals(),
            Theme::System => self.dark_visuals(), // Default to dark
        };

        ctx.set_visuals(visuals);
        self.configure_style(ctx);
    }

    /// Dark theme visuals - Claude style
    fn dark_visuals(&self) -> Visuals {
        use colors::*;

        Visuals {
            dark_mode: true,
            override_text_color: Some(TEXT_PRIMARY),
            hyperlink_color: CLAUDE_ORANGE,
            faint_bg_color: BG_DARK,
            extreme_bg_color: BG_DARKEST,
            code_bg_color: CODE_BG,
            warn_fg_color: WARNING,
            error_fg_color: ERROR,

            // Window
            window_fill: BG_SURFACE,
            window_stroke: Stroke::new(1.0, BORDER),
            window_rounding: Rounding::same(12.0),
            window_shadow: egui::epaint::Shadow {
                offset: egui::vec2(8.0, 8.0),
                blur: 16.0,
                spread: 0.0,
                color: Color32::from_black_alpha(128),
            },

            // Panel
            panel_fill: BG_DARK,

            // Menu
            menu_rounding: Rounding::same(8.0),

            // Color space
            numeric_color_space: egui::style::NumericColorSpace::GammaByte,

            // Widgets
            widgets: egui::style::Widgets {
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: BG_DARK,
                    weak_bg_fill: BG_SURFACE,
                    bg_stroke: Stroke::new(1.0, BORDER),
                    fg_stroke: Stroke::new(1.0, TEXT_SECONDARY),
                    rounding: Rounding::same(8.0),
                    expansion: 0.0,
                },
                inactive: egui::style::WidgetVisuals {
                    bg_fill: BG_SURFACE,
                    weak_bg_fill: BG_ELEVATED,
                    bg_stroke: Stroke::new(1.0, BORDER),
                    fg_stroke: Stroke::new(1.0, TEXT_PRIMARY),
                    rounding: Rounding::same(8.0),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: BG_ELEVATED,
                    weak_bg_fill: Color32::from_rgb(55, 55, 55),
                    bg_stroke: Stroke::new(1.0, BORDER_LIGHT),
                    fg_stroke: Stroke::new(1.5, CLAUDE_ORANGE),
                    rounding: Rounding::same(8.0),
                    expansion: 1.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: CLAUDE_ORANGE_DARK,
                    weak_bg_fill: CLAUDE_ORANGE,
                    bg_stroke: Stroke::new(1.0, CLAUDE_ORANGE_LIGHT),
                    fg_stroke: Stroke::new(1.5, Color32::WHITE),
                    rounding: Rounding::same(8.0),
                    expansion: 1.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: BG_ELEVATED,
                    weak_bg_fill: BG_SURFACE,
                    bg_stroke: Stroke::new(1.0, CLAUDE_ORANGE),
                    fg_stroke: Stroke::new(1.0, CLAUDE_ORANGE),
                    rounding: Rounding::same(8.0),
                    expansion: 0.0,
                },
            },

            // Selection
            selection: egui::style::Selection {
                bg_fill: Color32::from_rgb(212, 165, 116).linear_multiply(0.3),
                stroke: Stroke::new(1.0, CLAUDE_ORANGE),
            },

            // Other settings
            popup_shadow: egui::epaint::Shadow {
                offset: egui::vec2(4.0, 4.0),
                blur: 12.0,
                spread: 0.0,
                color: Color32::from_black_alpha(128),
            },
            resize_corner_size: 12.0,
            text_cursor: egui::style::TextCursorStyle::default(),
            clip_rect_margin: 0.0,
            button_frame: true,
            collapsing_header_frame: false,
            indent_has_left_vline: true,
            striped: true,
            slider_trailing_fill: true,
            handle_shape: egui::style::HandleShape::Circle,
            interact_cursor: None,
            image_loading_spinners: true,
            window_highlight_topmost: true,
        }
    }

    /// Light theme visuals
    fn light_visuals(&self) -> Visuals {
        use colors::*;

        Visuals {
            dark_mode: false,
            override_text_color: Some(TEXT_LIGHT_PRIMARY),
            hyperlink_color: CLAUDE_ORANGE_DARK,
            faint_bg_color: BG_LIGHT,
            extreme_bg_color: Color32::WHITE,
            code_bg_color: Color32::from_rgb(245, 245, 245),
            warn_fg_color: Color32::from_rgb(230, 126, 34),
            error_fg_color: Color32::from_rgb(231, 76, 60),

            window_fill: BG_LIGHT_SURFACE,
            window_stroke: Stroke::new(1.0, BORDER_LIGHT_MODE),
            window_rounding: Rounding::same(12.0),
            window_shadow: egui::epaint::Shadow {
                offset: egui::vec2(8.0, 8.0),
                blur: 16.0,
                spread: 0.0,
                color: Color32::from_black_alpha(64),
            },

            panel_fill: BG_LIGHT,
            menu_rounding: Rounding::same(8.0),
            numeric_color_space: egui::style::NumericColorSpace::GammaByte,

            widgets: egui::style::Widgets {
                noninteractive: egui::style::WidgetVisuals {
                    bg_fill: BG_LIGHT,
                    weak_bg_fill: Color32::WHITE,
                    bg_stroke: Stroke::new(1.0, BORDER_LIGHT_MODE),
                    fg_stroke: Stroke::new(1.0, TEXT_LIGHT_SECONDARY),
                    rounding: Rounding::same(8.0),
                    expansion: 0.0,
                },
                inactive: egui::style::WidgetVisuals {
                    bg_fill: Color32::WHITE,
                    weak_bg_fill: BG_LIGHT,
                    bg_stroke: Stroke::new(1.0, BORDER_LIGHT_MODE),
                    fg_stroke: Stroke::new(1.0, TEXT_LIGHT_PRIMARY),
                    rounding: Rounding::same(8.0),
                    expansion: 0.0,
                },
                hovered: egui::style::WidgetVisuals {
                    bg_fill: BG_LIGHT,
                    weak_bg_fill: Color32::from_rgb(240, 240, 240),
                    bg_stroke: Stroke::new(1.0, Color32::from_rgb(200, 200, 200)),
                    fg_stroke: Stroke::new(1.5, CLAUDE_ORANGE_DARK),
                    rounding: Rounding::same(8.0),
                    expansion: 1.0,
                },
                active: egui::style::WidgetVisuals {
                    bg_fill: CLAUDE_ORANGE,
                    weak_bg_fill: CLAUDE_ORANGE_LIGHT,
                    bg_stroke: Stroke::new(1.0, CLAUDE_ORANGE_DARK),
                    fg_stroke: Stroke::new(1.5, Color32::WHITE),
                    rounding: Rounding::same(8.0),
                    expansion: 1.0,
                },
                open: egui::style::WidgetVisuals {
                    bg_fill: BG_LIGHT,
                    weak_bg_fill: Color32::WHITE,
                    bg_stroke: Stroke::new(1.0, CLAUDE_ORANGE),
                    fg_stroke: Stroke::new(1.0, CLAUDE_ORANGE),
                    rounding: Rounding::same(8.0),
                    expansion: 0.0,
                },
            },

            selection: egui::style::Selection {
                bg_fill: Color32::from_rgb(212, 165, 116).linear_multiply(0.2),
                stroke: Stroke::new(1.0, CLAUDE_ORANGE_DARK),
            },

            popup_shadow: egui::epaint::Shadow {
                offset: egui::vec2(4.0, 4.0),
                blur: 12.0,
                spread: 0.0,
                color: Color32::from_black_alpha(64),
            },
            resize_corner_size: 12.0,
            text_cursor: egui::style::TextCursorStyle::default(),
            clip_rect_margin: 0.0,
            button_frame: true,
            collapsing_header_frame: false,
            indent_has_left_vline: true,
            striped: true,
            slider_trailing_fill: true,
            handle_shape: egui::style::HandleShape::Circle,
            interact_cursor: None,
            image_loading_spinners: true,
            window_highlight_topmost: true,
        }
    }

    /// Configure global style
    fn configure_style(&self, ctx: &egui::Context) {
        let mut style = (*ctx.style()).clone();

        style.text_styles = [
            (TextStyle::Heading, FontId::new(24.0, FontFamily::Proportional)),
            (TextStyle::Name("heading2".into()), FontId::new(20.0, FontFamily::Proportional)),
            (TextStyle::Name("heading3".into()), FontId::new(16.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
            (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
            (TextStyle::Small, FontId::new(12.0, FontFamily::Proportional)),
        ]
        .into();

        // Spacing
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.window_margin = egui::Margin::same(16.0);
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.indent = 16.0;
        style.spacing.interact_size = egui::vec2(40.0, 24.0);

        ctx.set_style(style);
    }

    // === Color getters ===

    /// Primary accent color (Claude orange)
    pub fn primary_color(&self) -> Color32 {
        colors::CLAUDE_ORANGE
    }

    /// Darker variant of primary
    pub fn primary_dark(&self) -> Color32 {
        colors::CLAUDE_ORANGE_DARK
    }

    /// Lighter variant of primary
    pub fn primary_light(&self) -> Color32 {
        colors::CLAUDE_ORANGE_LIGHT
    }

    /// Secondary color
    pub fn secondary_color(&self) -> Color32 {
        colors::CLAUDE_ORANGE_LIGHT
    }

    /// Background color
    pub fn background_color(&self) -> Color32 {
        match self {
            Theme::Light => colors::BG_LIGHT,
            Theme::Dark | Theme::System => colors::BG_DARK,
        }
    }

    /// Darkest background (for contrast)
    pub fn background_darkest(&self) -> Color32 {
        match self {
            Theme::Light => colors::BG_LIGHT_SURFACE,
            Theme::Dark | Theme::System => colors::BG_DARKEST,
        }
    }

    /// Surface color (cards, panels)
    pub fn surface_color(&self) -> Color32 {
        match self {
            Theme::Light => colors::BG_LIGHT_SURFACE,
            Theme::Dark | Theme::System => colors::BG_SURFACE,
        }
    }

    /// Elevated surface (hovered items)
    pub fn elevated_color(&self) -> Color32 {
        match self {
            Theme::Light => Color32::WHITE,
            Theme::Dark | Theme::System => colors::BG_ELEVATED,
        }
    }

    /// Primary text color
    pub fn text_color(&self) -> Color32 {
        match self {
            Theme::Light => colors::TEXT_LIGHT_PRIMARY,
            Theme::Dark | Theme::System => colors::TEXT_PRIMARY,
        }
    }

    /// Secondary text color
    pub fn text_secondary_color(&self) -> Color32 {
        match self {
            Theme::Light => colors::TEXT_LIGHT_SECONDARY,
            Theme::Dark | Theme::System => colors::TEXT_SECONDARY,
        }
    }

    /// Muted/disabled text
    pub fn muted_text_color(&self) -> Color32 {
        match self {
            Theme::Light => Color32::from_rgb(150, 150, 150),
            Theme::Dark | Theme::System => colors::TEXT_MUTED,
        }
    }

    /// Border color
    pub fn border_color(&self) -> Color32 {
        match self {
            Theme::Light => colors::BORDER_LIGHT_MODE,
            Theme::Dark | Theme::System => colors::BORDER,
        }
    }

    /// Success color
    pub fn success_color(&self) -> Color32 {
        colors::SUCCESS
    }

    /// Warning color
    pub fn warning_color(&self) -> Color32 {
        colors::WARNING
    }

    /// Error color
    pub fn error_color(&self) -> Color32 {
        colors::ERROR
    }

    /// Info color
    pub fn info_color(&self) -> Color32 {
        colors::INFO
    }

    /// Code block background
    pub fn code_bg_color(&self) -> Color32 {
        colors::CODE_BG
    }

    /// Code text color
    pub fn code_text_color(&self) -> Color32 {
        colors::CODE_TEXT
    }

    /// Inline code background
    pub fn inline_code_bg_color(&self) -> Color32 {
        colors::INLINE_CODE_BG
    }

    /// Get user message bubble color
    pub fn user_message_bg(&self) -> Color32 {
        colors::CLAUDE_ORANGE_DARK
    }

    /// Get assistant message bubble color
    pub fn assistant_message_bg(&self) -> Color32 {
        self.surface_color()
    }

    /// Get system message color
    pub fn system_message_color(&self) -> Color32 {
        self.muted_text_color()
    }

    /// Check if dark mode
    pub fn is_dark(&self) -> bool {
        matches!(self, Theme::Dark | Theme::System)
    }
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}
