//! Application-wide theme system
//!
//! Provides a comprehensive theming system for all UI elements with
//! multiple built-in themes and the ability to create custom themes.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete application theme defining all UI colors
#[derive(Debug, Clone)]
pub struct AppTheme {
    pub name: String,
    pub description: String,

    // Window colors
    pub window_border: Color,
    pub window_border_focused: Color,
    pub window_background: Color,
    pub window_title: Color,

    // Text colors
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub text_selected: Color,

    // Background colors
    pub background_primary: Color,
    pub background_secondary: Color,
    pub background_selected: Color,
    pub background_hover: Color,

    // Editor colors
    pub editor_border: Color,
    pub editor_label: Color,
    pub editor_label_focused: Color,
    pub editor_text: Color,
    pub editor_cursor: Color,
    pub editor_status: Color,
    pub editor_background: Color,

    // Browser/List colors
    pub browser_border: Color,
    pub browser_title: Color,
    pub browser_item_normal: Color,
    pub browser_item_selected: Color,
    pub browser_item_focused: Color,
    pub browser_background: Color,
    pub browser_scrollbar: Color,

    // Form colors
    pub form_border: Color,
    pub form_label: Color,
    pub form_label_focused: Color,
    pub form_field_background: Color,
    pub form_field_text: Color,
    pub form_checkbox_checked: Color,
    pub form_checkbox_unchecked: Color,
    pub form_error: Color,

    // Menu/Popup colors
    pub menu_border: Color,
    pub menu_background: Color,
    pub menu_item_normal: Color,
    pub menu_item_selected: Color,
    pub menu_item_focused: Color,
    pub menu_separator: Color,

    // Status/Indicator colors
    pub status_info: Color,
    pub status_success: Color,
    pub status_warning: Color,
    pub status_error: Color,
    pub status_background: Color,

    // Interactive elements
    pub button_normal: Color,
    pub button_hover: Color,
    pub button_active: Color,
    pub button_disabled: Color,

    // Game-specific colors
    pub command_echo: Color,
    pub selection_background: Color,
    pub link_color: Color,
    pub speech_color: Color,
    pub whisper_color: Color,
    pub thought_color: Color,

    // Widget defaults
    pub injury_default_color: Color,
}

impl AppTheme {
    /// Get a color by semantic name (for dynamic lookups)
    pub fn get_color(&self, name: &str) -> Option<Color> {
        match name {
            "window_border" => Some(self.window_border),
            "window_border_focused" => Some(self.window_border_focused),
            "window_background" => Some(self.window_background),
            "text_primary" => Some(self.text_primary),
            "text_selected" => Some(self.text_selected),
            "background_selected" => Some(self.background_selected),
            "editor_cursor" => Some(self.editor_cursor),
            "status_error" => Some(self.status_error),
            "link_color" => Some(self.link_color),
            "injury_default_color" => Some(self.injury_default_color),
            _ => None,
        }
    }

    /// Convert EditorTheme colors to use AppTheme
    pub fn to_editor_theme(&self) -> EditorTheme {
        EditorTheme {
            border_color: self.editor_border,
            label_color: self.editor_label,
            focused_label_color: self.editor_label_focused,
            text_color: self.editor_text,
            cursor_color: self.editor_cursor,
            status_color: self.editor_status,
        }
    }
}

fn color_to_rgb_components(color: Color) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(index) => indexed_color_to_rgb(index),
        Color::Reset => (0, 0, 0),
        Color::Black => (0, 0, 0),
        Color::Red => (205, 0, 0),
        Color::Green => (0, 205, 0),
        Color::Yellow => (205, 205, 0),
        Color::Blue => (0, 0, 238),
        Color::Magenta => (205, 0, 205),
        Color::Cyan => (0, 205, 205),
        Color::Gray => (229, 229, 229),
        Color::DarkGray => (127, 127, 127),
        Color::LightRed => (255, 102, 102),
        Color::LightGreen => (102, 255, 102),
        Color::LightYellow => (255, 255, 102),
        Color::LightBlue => (173, 216, 230),
        Color::LightMagenta => (255, 119, 255),
        Color::LightCyan => (224, 255, 255),
        Color::White => (255, 255, 255),
    }
}

fn indexed_color_to_rgb(index: u8) -> (u8, u8, u8) {
    const STANDARD_COLORS: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];

    if index < 16 {
        return STANDARD_COLORS[index as usize];
    }

    if index <= 231 {
        let level = index as usize - 16;
        let r = level / 36;
        let g = (level % 36) / 6;
        let b = level % 6;
        let levels = [0, 95, 135, 175, 215, 255];
        return (levels[r], levels[g], levels[b]);
    }

    let gray = 8 + (index.saturating_sub(232)) * 10;
    (gray, gray, gray)
}

fn blend_colors(base: Color, other: Color, ratio: f32) -> Color {
    let ratio = ratio.clamp(0.0, 1.0);
    let (br, bg, bb) = color_to_rgb_components(base);
    let (or, og, ob) = color_to_rgb_components(other);
    let blend_component = |a: u8, b: u8| -> u8 {
        let value = (a as f32) * (1.0 - ratio) + (b as f32) * ratio;
        value.round().clamp(0.0, 255.0) as u8
    };

    Color::Rgb(
        blend_component(br, or),
        blend_component(bg, og),
        blend_component(bb, ob),
    )
}

fn derive_injury_default_color(window_background: Color, text_secondary: Color) -> Color {
    blend_colors(window_background, text_secondary, 0.25)
}

/// Subset of theme for window editor (backwards compatibility)
#[derive(Debug, Clone)]
pub struct EditorTheme {
    pub border_color: Color,
    pub label_color: Color,
    pub focused_label_color: Color,
    pub text_color: Color,
    pub cursor_color: Color,
    pub status_color: Color,
}

/// Built-in theme presets
pub struct ThemePresets;

impl ThemePresets {
    /// Get all available built-in themes
    pub fn all() -> HashMap<String, AppTheme> {
        let mut themes = HashMap::new();
        themes.insert("dark".to_string(), Self::dark());
        themes.insert("light".to_string(), Self::light());
        themes.insert("nord".to_string(), Self::nord());
        themes.insert("dracula".to_string(), Self::dracula());
        themes.insert("solarized-dark".to_string(), Self::solarized_dark());
        themes.insert("solarized-light".to_string(), Self::solarized_light());
        themes.insert("monokai".to_string(), Self::monokai());
        themes.insert("gruvbox-dark".to_string(), Self::gruvbox_dark());
        themes.insert("night-owl".to_string(), Self::night_owl());
        themes.insert("catppuccin".to_string(), Self::catppuccin());
        themes.insert("cyberpunk".to_string(), Self::cyberpunk());
        themes.insert("retro-terminal".to_string(), Self::retro_terminal());
        themes.insert("apex".to_string(), Self::apex());
        themes.insert("minimalist-warm".to_string(), Self::minimalist_warm());
        themes.insert("forest-creek".to_string(), Self::forest_creek());
        themes.insert("synthwave".to_string(), Self::synthwave());
        themes
    }

    /// Default dark theme (current VellumFE style)
    pub fn dark() -> AppTheme {
        let mut theme = AppTheme {
            name: "Dark".to_string(),
            description: "Classic dark theme with cyan accents".to_string(),

            // Windows
            window_border: Color::Cyan,
            window_border_focused: Color::Yellow,
            window_background: Color::Black,
            window_title: Color::White,

            // Text
            text_primary: Color::White,
            text_secondary: Color::Gray,
            text_disabled: Color::DarkGray,
            text_selected: Color::Yellow,

            // Backgrounds
            background_primary: Color::Black,
            background_secondary: Color::Rgb(20, 20, 20),
            background_selected: Color::Rgb(74, 74, 74),
            background_hover: Color::Rgb(40, 40, 40),

            // Editor
            editor_border: Color::Cyan,
            editor_label: Color::Cyan,
            editor_label_focused: Color::Rgb(255, 215, 0), // Gold
            editor_text: Color::White,
            editor_cursor: Color::Yellow,
            editor_status: Color::Yellow,
            editor_background: Color::Black,

            // Browser
            browser_border: Color::Cyan,
            browser_title: Color::White,
            browser_item_normal: Color::White,
            browser_item_selected: Color::Black,
            browser_item_focused: Color::Yellow,
            browser_background: Color::Black,
            browser_scrollbar: Color::Cyan,

            // Form
            form_border: Color::Cyan,
            form_label: Color::Rgb(100, 149, 237), // Cornflower blue
            form_label_focused: Color::Yellow,
            form_field_background: Color::Rgb(30, 30, 30),
            form_field_text: Color::Cyan,
            form_checkbox_checked: Color::Green,
            form_checkbox_unchecked: Color::Gray,
            form_error: Color::Red,

            // Menu
            menu_border: Color::Cyan,
            menu_background: Color::Black,
            menu_item_normal: Color::White,
            menu_item_selected: Color::Black,
            menu_item_focused: Color::Yellow,
            menu_separator: Color::DarkGray,

            // Status
            status_info: Color::Cyan,
            status_success: Color::Green,
            status_warning: Color::Yellow,
            status_error: Color::Red,
            status_background: Color::Black,

            // Interactive
            button_normal: Color::Cyan,
            button_hover: Color::Yellow,
            button_active: Color::Green,
            button_disabled: Color::DarkGray,

            // Game-specific
            command_echo: Color::White,
            selection_background: Color::Rgb(74, 74, 74),
            link_color: Color::Rgb(71, 122, 179),
            speech_color: Color::Rgb(83, 166, 132),
            whisper_color: Color::Rgb(96, 180, 191),
            thought_color: Color::Rgb(255, 128, 128),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Light theme for daytime use
    pub fn light() -> AppTheme {
        let mut theme = AppTheme {
            name: "Light".to_string(),
            description: "Bright light theme for daytime use".to_string(),

            // Windows
            window_border: Color::Blue,
            window_border_focused: Color::Rgb(255, 140, 0), // Dark orange
            window_background: Color::White,
            window_title: Color::Black,

            // Text
            text_primary: Color::Black,
            text_secondary: Color::Rgb(80, 80, 80),
            text_disabled: Color::Rgb(160, 160, 160),
            text_selected: Color::Rgb(0, 0, 139), // Dark blue

            // Backgrounds
            background_primary: Color::White,
            background_secondary: Color::Rgb(245, 245, 245),
            background_selected: Color::Rgb(200, 220, 255),
            background_hover: Color::Rgb(230, 230, 230),

            // Editor
            editor_border: Color::Blue,
            editor_label: Color::Blue,
            editor_label_focused: Color::Rgb(255, 140, 0),
            editor_text: Color::Black,
            editor_cursor: Color::Rgb(255, 140, 0),
            editor_status: Color::Rgb(0, 100, 0),
            editor_background: Color::White,

            // Browser
            browser_border: Color::Blue,
            browser_title: Color::Black,
            browser_item_normal: Color::Black,
            browser_item_selected: Color::White,
            browser_item_focused: Color::Rgb(0, 0, 139),
            browser_background: Color::White,
            browser_scrollbar: Color::Blue,

            // Form
            form_border: Color::Blue,
            form_label: Color::Rgb(0, 0, 139),
            form_label_focused: Color::Rgb(255, 140, 0),
            form_field_background: Color::Rgb(250, 250, 250),
            form_field_text: Color::Black,
            form_checkbox_checked: Color::Rgb(0, 128, 0),
            form_checkbox_unchecked: Color::Rgb(128, 128, 128),
            form_error: Color::Rgb(200, 0, 0),

            // Menu
            menu_border: Color::Blue,
            menu_background: Color::White,
            menu_item_normal: Color::Black,
            menu_item_selected: Color::White,
            menu_item_focused: Color::Rgb(0, 0, 139),
            menu_separator: Color::Rgb(200, 200, 200),

            // Status
            status_info: Color::Blue,
            status_success: Color::Rgb(0, 128, 0),
            status_warning: Color::Rgb(200, 100, 0),
            status_error: Color::Rgb(200, 0, 0),
            status_background: Color::Rgb(245, 245, 245),

            // Interactive
            button_normal: Color::Blue,
            button_hover: Color::Rgb(255, 140, 0),
            button_active: Color::Rgb(0, 128, 0),
            button_disabled: Color::Rgb(180, 180, 180),

            // Game-specific
            command_echo: Color::Black,
            selection_background: Color::Rgb(200, 220, 255),
            link_color: Color::Rgb(0, 0, 238),
            speech_color: Color::Rgb(0, 128, 0),
            whisper_color: Color::Rgb(0, 128, 128),
            thought_color: Color::Rgb(200, 50, 50),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Nord theme - Arctic, north-bluish color palette
    pub fn nord() -> AppTheme {
        let mut theme = AppTheme {
            name: "Nord".to_string(),
            description: "Arctic-inspired color palette".to_string(),

            window_border: Color::Rgb(136, 192, 208), // Nord frost
            window_border_focused: Color::Rgb(143, 188, 187), // Nord frost
            window_background: Color::Rgb(46, 52, 64), // Nord polar night
            window_title: Color::Rgb(236, 239, 244),  // Nord snow storm

            text_primary: Color::Rgb(236, 239, 244),
            text_secondary: Color::Rgb(216, 222, 233),
            text_disabled: Color::Rgb(76, 86, 106),
            text_selected: Color::Rgb(136, 192, 208),

            background_primary: Color::Rgb(46, 52, 64),
            background_secondary: Color::Rgb(59, 66, 82),
            background_selected: Color::Rgb(76, 86, 106),
            background_hover: Color::Rgb(67, 76, 94),

            editor_border: Color::Rgb(136, 192, 208),
            editor_label: Color::Rgb(136, 192, 208),
            editor_label_focused: Color::Rgb(163, 190, 140),
            editor_text: Color::Rgb(236, 239, 244),
            editor_cursor: Color::Rgb(235, 203, 139),
            editor_status: Color::Rgb(163, 190, 140),
            editor_background: Color::Rgb(46, 52, 64),

            browser_border: Color::Rgb(136, 192, 208),
            browser_title: Color::Rgb(236, 239, 244),
            browser_item_normal: Color::Rgb(236, 239, 244),
            browser_item_selected: Color::Rgb(46, 52, 64),
            browser_item_focused: Color::Rgb(136, 192, 208),
            browser_background: Color::Rgb(46, 52, 64),
            browser_scrollbar: Color::Rgb(136, 192, 208),

            form_border: Color::Rgb(136, 192, 208),
            form_label: Color::Rgb(129, 161, 193),
            form_label_focused: Color::Rgb(235, 203, 139),
            form_field_background: Color::Rgb(59, 66, 82),
            form_field_text: Color::Rgb(236, 239, 244),
            form_checkbox_checked: Color::Rgb(163, 190, 140),
            form_checkbox_unchecked: Color::Rgb(76, 86, 106),
            form_error: Color::Rgb(191, 97, 106),

            menu_border: Color::Rgb(136, 192, 208),
            menu_background: Color::Rgb(46, 52, 64),
            menu_item_normal: Color::Rgb(236, 239, 244),
            menu_item_selected: Color::Rgb(46, 52, 64),
            menu_item_focused: Color::Rgb(136, 192, 208),
            menu_separator: Color::Rgb(76, 86, 106),

            status_info: Color::Rgb(136, 192, 208),
            status_success: Color::Rgb(163, 190, 140),
            status_warning: Color::Rgb(235, 203, 139),
            status_error: Color::Rgb(191, 97, 106),
            status_background: Color::Rgb(46, 52, 64),

            button_normal: Color::Rgb(136, 192, 208),
            button_hover: Color::Rgb(163, 190, 140),
            button_active: Color::Rgb(163, 190, 140),
            button_disabled: Color::Rgb(76, 86, 106),

            command_echo: Color::Rgb(236, 239, 244),
            selection_background: Color::Rgb(76, 86, 106),
            link_color: Color::Rgb(136, 192, 208),
            speech_color: Color::Rgb(163, 190, 140),
            whisper_color: Color::Rgb(129, 161, 193),
            thought_color: Color::Rgb(180, 142, 173),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Dracula theme - Dark theme with purple accents
    pub fn dracula() -> AppTheme {
        let mut theme = AppTheme {
            name: "Dracula".to_string(),
            description: "Dark theme with vibrant purple accents".to_string(),

            window_border: Color::Rgb(189, 147, 249), // Purple
            window_border_focused: Color::Rgb(255, 121, 198), // Pink
            window_background: Color::Rgb(40, 42, 54), // Background
            window_title: Color::Rgb(248, 248, 242),  // Foreground

            text_primary: Color::Rgb(248, 248, 242),
            text_secondary: Color::Rgb(98, 114, 164),
            text_disabled: Color::Rgb(68, 71, 90),
            text_selected: Color::Rgb(255, 121, 198),

            background_primary: Color::Rgb(40, 42, 54),
            background_secondary: Color::Rgb(68, 71, 90),
            background_selected: Color::Rgb(68, 71, 90),
            background_hover: Color::Rgb(68, 71, 90),

            editor_border: Color::Rgb(189, 147, 249),
            editor_label: Color::Rgb(139, 233, 253),
            editor_label_focused: Color::Rgb(255, 121, 198),
            editor_text: Color::Rgb(248, 248, 242),
            editor_cursor: Color::Rgb(255, 121, 198),
            editor_status: Color::Rgb(80, 250, 123),
            editor_background: Color::Rgb(40, 42, 54),

            browser_border: Color::Rgb(189, 147, 249),
            browser_title: Color::Rgb(248, 248, 242),
            browser_item_normal: Color::Rgb(248, 248, 242),
            browser_item_selected: Color::Rgb(40, 42, 54),
            browser_item_focused: Color::Rgb(255, 121, 198),
            browser_background: Color::Rgb(40, 42, 54),
            browser_scrollbar: Color::Rgb(189, 147, 249),

            form_border: Color::Rgb(189, 147, 249),
            form_label: Color::Rgb(139, 233, 253),
            form_label_focused: Color::Rgb(255, 121, 198),
            form_field_background: Color::Rgb(68, 71, 90),
            form_field_text: Color::Rgb(248, 248, 242),
            form_checkbox_checked: Color::Rgb(80, 250, 123),
            form_checkbox_unchecked: Color::Rgb(98, 114, 164),
            form_error: Color::Rgb(255, 85, 85),

            menu_border: Color::Rgb(189, 147, 249),
            menu_background: Color::Rgb(40, 42, 54),
            menu_item_normal: Color::Rgb(248, 248, 242),
            menu_item_selected: Color::Rgb(40, 42, 54),
            menu_item_focused: Color::Rgb(255, 121, 198),
            menu_separator: Color::Rgb(98, 114, 164),

            status_info: Color::Rgb(139, 233, 253),
            status_success: Color::Rgb(80, 250, 123),
            status_warning: Color::Rgb(241, 250, 140),
            status_error: Color::Rgb(255, 85, 85),
            status_background: Color::Rgb(40, 42, 54),

            button_normal: Color::Rgb(189, 147, 249),
            button_hover: Color::Rgb(255, 121, 198),
            button_active: Color::Rgb(80, 250, 123),
            button_disabled: Color::Rgb(98, 114, 164),

            command_echo: Color::Rgb(248, 248, 242),
            selection_background: Color::Rgb(68, 71, 90),
            link_color: Color::Rgb(189, 147, 249),
            speech_color: Color::Rgb(80, 250, 123),
            whisper_color: Color::Rgb(139, 233, 253),
            thought_color: Color::Rgb(255, 121, 198),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Solarized Dark
    pub fn solarized_dark() -> AppTheme {
        let mut theme = AppTheme {
            name: "Solarized Dark".to_string(),
            description: "Precision colors for machines and people".to_string(),

            window_border: Color::Rgb(38, 139, 210), // Blue
            window_border_focused: Color::Rgb(203, 75, 22), // Orange
            window_background: Color::Rgb(0, 43, 54), // Base03
            window_title: Color::Rgb(147, 161, 161), // Base1

            text_primary: Color::Rgb(131, 148, 150),
            text_secondary: Color::Rgb(88, 110, 117),
            text_disabled: Color::Rgb(7, 54, 66),
            text_selected: Color::Rgb(203, 75, 22),

            background_primary: Color::Rgb(0, 43, 54),
            background_secondary: Color::Rgb(7, 54, 66),
            background_selected: Color::Rgb(7, 54, 66),
            background_hover: Color::Rgb(7, 54, 66),

            editor_border: Color::Rgb(38, 139, 210),
            editor_label: Color::Rgb(42, 161, 152),
            editor_label_focused: Color::Rgb(203, 75, 22),
            editor_text: Color::Rgb(131, 148, 150),
            editor_cursor: Color::Rgb(203, 75, 22),
            editor_status: Color::Rgb(133, 153, 0),
            editor_background: Color::Rgb(0, 43, 54),

            browser_border: Color::Rgb(38, 139, 210),
            browser_title: Color::Rgb(147, 161, 161),
            browser_item_normal: Color::Rgb(131, 148, 150),
            browser_item_selected: Color::Rgb(0, 43, 54),
            browser_item_focused: Color::Rgb(203, 75, 22),
            browser_background: Color::Rgb(0, 43, 54),
            browser_scrollbar: Color::Rgb(38, 139, 210),

            form_border: Color::Rgb(38, 139, 210),
            form_label: Color::Rgb(42, 161, 152),
            form_label_focused: Color::Rgb(203, 75, 22),
            form_field_background: Color::Rgb(7, 54, 66),
            form_field_text: Color::Rgb(131, 148, 150),
            form_checkbox_checked: Color::Rgb(133, 153, 0),
            form_checkbox_unchecked: Color::Rgb(88, 110, 117),
            form_error: Color::Rgb(220, 50, 47),

            menu_border: Color::Rgb(38, 139, 210),
            menu_background: Color::Rgb(0, 43, 54),
            menu_item_normal: Color::Rgb(131, 148, 150),
            menu_item_selected: Color::Rgb(0, 43, 54),
            menu_item_focused: Color::Rgb(203, 75, 22),
            menu_separator: Color::Rgb(7, 54, 66),

            status_info: Color::Rgb(38, 139, 210),
            status_success: Color::Rgb(133, 153, 0),
            status_warning: Color::Rgb(181, 137, 0),
            status_error: Color::Rgb(220, 50, 47),
            status_background: Color::Rgb(0, 43, 54),

            button_normal: Color::Rgb(38, 139, 210),
            button_hover: Color::Rgb(203, 75, 22),
            button_active: Color::Rgb(133, 153, 0),
            button_disabled: Color::Rgb(88, 110, 117),

            command_echo: Color::Rgb(131, 148, 150),
            selection_background: Color::Rgb(7, 54, 66),
            link_color: Color::Rgb(38, 139, 210),
            speech_color: Color::Rgb(133, 153, 0),
            whisper_color: Color::Rgb(42, 161, 152),
            thought_color: Color::Rgb(108, 113, 196),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Solarized Light
    pub fn solarized_light() -> AppTheme {
        let mut theme = AppTheme {
            name: "Solarized Light".to_string(),
            description: "Precision colors for machines and people (light)".to_string(),

            window_border: Color::Rgb(38, 139, 210),
            window_border_focused: Color::Rgb(203, 75, 22),
            window_background: Color::Rgb(253, 246, 227), // Base3
            window_title: Color::Rgb(101, 123, 131),      // Base00

            text_primary: Color::Rgb(101, 123, 131),
            text_secondary: Color::Rgb(147, 161, 161),
            text_disabled: Color::Rgb(238, 232, 213),
            text_selected: Color::Rgb(203, 75, 22),

            background_primary: Color::Rgb(253, 246, 227),
            background_secondary: Color::Rgb(238, 232, 213),
            background_selected: Color::Rgb(238, 232, 213),
            background_hover: Color::Rgb(238, 232, 213),

            editor_border: Color::Rgb(38, 139, 210),
            editor_label: Color::Rgb(42, 161, 152),
            editor_label_focused: Color::Rgb(203, 75, 22),
            editor_text: Color::Rgb(101, 123, 131),
            editor_cursor: Color::Rgb(203, 75, 22),
            editor_status: Color::Rgb(133, 153, 0),
            editor_background: Color::Rgb(253, 246, 227),

            browser_border: Color::Rgb(38, 139, 210),
            browser_title: Color::Rgb(101, 123, 131),
            browser_item_normal: Color::Rgb(101, 123, 131),
            browser_item_selected: Color::Rgb(253, 246, 227),
            browser_item_focused: Color::Rgb(203, 75, 22),
            browser_background: Color::Rgb(253, 246, 227),
            browser_scrollbar: Color::Rgb(38, 139, 210),

            form_border: Color::Rgb(38, 139, 210),
            form_label: Color::Rgb(42, 161, 152),
            form_label_focused: Color::Rgb(203, 75, 22),
            form_field_background: Color::Rgb(238, 232, 213),
            form_field_text: Color::Rgb(101, 123, 131),
            form_checkbox_checked: Color::Rgb(133, 153, 0),
            form_checkbox_unchecked: Color::Rgb(147, 161, 161),
            form_error: Color::Rgb(220, 50, 47),

            menu_border: Color::Rgb(38, 139, 210),
            menu_background: Color::Rgb(253, 246, 227),
            menu_item_normal: Color::Rgb(101, 123, 131),
            menu_item_selected: Color::Rgb(253, 246, 227),
            menu_item_focused: Color::Rgb(203, 75, 22),
            menu_separator: Color::Rgb(238, 232, 213),

            status_info: Color::Rgb(38, 139, 210),
            status_success: Color::Rgb(133, 153, 0),
            status_warning: Color::Rgb(181, 137, 0),
            status_error: Color::Rgb(220, 50, 47),
            status_background: Color::Rgb(253, 246, 227),

            button_normal: Color::Rgb(38, 139, 210),
            button_hover: Color::Rgb(203, 75, 22),
            button_active: Color::Rgb(133, 153, 0),
            button_disabled: Color::Rgb(147, 161, 161),

            command_echo: Color::Rgb(101, 123, 131),
            selection_background: Color::Rgb(238, 232, 213),
            link_color: Color::Rgb(38, 139, 210),
            speech_color: Color::Rgb(133, 153, 0),
            whisper_color: Color::Rgb(42, 161, 152),
            thought_color: Color::Rgb(108, 113, 196),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Monokai theme
    pub fn monokai() -> AppTheme {
        let mut theme = AppTheme {
            name: "Monokai".to_string(),
            description: "Vibrant coding theme with warm colors".to_string(),

            window_border: Color::Rgb(102, 217, 239),
            window_border_focused: Color::Rgb(249, 38, 114),
            window_background: Color::Rgb(39, 40, 34),
            window_title: Color::Rgb(248, 248, 240),

            text_primary: Color::Rgb(248, 248, 240),
            text_secondary: Color::Rgb(117, 113, 94),
            text_disabled: Color::Rgb(73, 72, 62),
            text_selected: Color::Rgb(249, 38, 114),

            background_primary: Color::Rgb(39, 40, 34),
            background_secondary: Color::Rgb(73, 72, 62),
            background_selected: Color::Rgb(73, 72, 62),
            background_hover: Color::Rgb(73, 72, 62),

            editor_border: Color::Rgb(102, 217, 239),
            editor_label: Color::Rgb(102, 217, 239),
            editor_label_focused: Color::Rgb(249, 38, 114),
            editor_text: Color::Rgb(248, 248, 240),
            editor_cursor: Color::Rgb(249, 38, 114),
            editor_status: Color::Rgb(166, 226, 46),
            editor_background: Color::Rgb(39, 40, 34),

            browser_border: Color::Rgb(102, 217, 239),
            browser_title: Color::Rgb(248, 248, 240),
            browser_item_normal: Color::Rgb(248, 248, 240),
            browser_item_selected: Color::Rgb(39, 40, 34),
            browser_item_focused: Color::Rgb(249, 38, 114),
            browser_background: Color::Rgb(39, 40, 34),
            browser_scrollbar: Color::Rgb(102, 217, 239),

            form_border: Color::Rgb(102, 217, 239),
            form_label: Color::Rgb(102, 217, 239),
            form_label_focused: Color::Rgb(249, 38, 114),
            form_field_background: Color::Rgb(73, 72, 62),
            form_field_text: Color::Rgb(248, 248, 240),
            form_checkbox_checked: Color::Rgb(166, 226, 46),
            form_checkbox_unchecked: Color::Rgb(117, 113, 94),
            form_error: Color::Rgb(249, 38, 114),

            menu_border: Color::Rgb(102, 217, 239),
            menu_background: Color::Rgb(39, 40, 34),
            menu_item_normal: Color::Rgb(248, 248, 240),
            menu_item_selected: Color::Rgb(39, 40, 34),
            menu_item_focused: Color::Rgb(249, 38, 114),
            menu_separator: Color::Rgb(117, 113, 94),

            status_info: Color::Rgb(102, 217, 239),
            status_success: Color::Rgb(166, 226, 46),
            status_warning: Color::Rgb(253, 151, 31),
            status_error: Color::Rgb(249, 38, 114),
            status_background: Color::Rgb(39, 40, 34),

            button_normal: Color::Rgb(102, 217, 239),
            button_hover: Color::Rgb(249, 38, 114),
            button_active: Color::Rgb(166, 226, 46),
            button_disabled: Color::Rgb(117, 113, 94),

            command_echo: Color::Rgb(248, 248, 240),
            selection_background: Color::Rgb(73, 72, 62),
            link_color: Color::Rgb(102, 217, 239),
            speech_color: Color::Rgb(166, 226, 46),
            whisper_color: Color::Rgb(102, 217, 239),
            thought_color: Color::Rgb(174, 129, 255),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Gruvbox Dark theme
    pub fn gruvbox_dark() -> AppTheme {
        let mut theme = AppTheme {
            name: "Gruvbox Dark".to_string(),
            description: "Retro groove with warm earthy colors".to_string(),

            window_border: Color::Rgb(131, 165, 152),
            window_border_focused: Color::Rgb(254, 128, 25),
            window_background: Color::Rgb(40, 40, 40),
            window_title: Color::Rgb(235, 219, 178),

            text_primary: Color::Rgb(235, 219, 178),
            text_secondary: Color::Rgb(168, 153, 132),
            text_disabled: Color::Rgb(60, 56, 54),
            text_selected: Color::Rgb(254, 128, 25),

            background_primary: Color::Rgb(40, 40, 40),
            background_secondary: Color::Rgb(60, 56, 54),
            background_selected: Color::Rgb(80, 73, 69),
            background_hover: Color::Rgb(60, 56, 54),

            editor_border: Color::Rgb(131, 165, 152),
            editor_label: Color::Rgb(184, 187, 38),
            editor_label_focused: Color::Rgb(254, 128, 25),
            editor_text: Color::Rgb(235, 219, 178),
            editor_cursor: Color::Rgb(254, 128, 25),
            editor_status: Color::Rgb(184, 187, 38),
            editor_background: Color::Rgb(40, 40, 40),

            browser_border: Color::Rgb(131, 165, 152),
            browser_title: Color::Rgb(235, 219, 178),
            browser_item_normal: Color::Rgb(235, 219, 178),
            browser_item_selected: Color::Rgb(40, 40, 40),
            browser_item_focused: Color::Rgb(254, 128, 25),
            browser_background: Color::Rgb(40, 40, 40),
            browser_scrollbar: Color::Rgb(131, 165, 152),

            form_border: Color::Rgb(131, 165, 152),
            form_label: Color::Rgb(184, 187, 38),
            form_label_focused: Color::Rgb(254, 128, 25),
            form_field_background: Color::Rgb(60, 56, 54),
            form_field_text: Color::Rgb(235, 219, 178),
            form_checkbox_checked: Color::Rgb(184, 187, 38),
            form_checkbox_unchecked: Color::Rgb(168, 153, 132),
            form_error: Color::Rgb(251, 73, 52),

            menu_border: Color::Rgb(131, 165, 152),
            menu_background: Color::Rgb(40, 40, 40),
            menu_item_normal: Color::Rgb(235, 219, 178),
            menu_item_selected: Color::Rgb(40, 40, 40),
            menu_item_focused: Color::Rgb(254, 128, 25),
            menu_separator: Color::Rgb(80, 73, 69),

            status_info: Color::Rgb(131, 165, 152),
            status_success: Color::Rgb(184, 187, 38),
            status_warning: Color::Rgb(250, 189, 47),
            status_error: Color::Rgb(251, 73, 52),
            status_background: Color::Rgb(40, 40, 40),

            button_normal: Color::Rgb(131, 165, 152),
            button_hover: Color::Rgb(254, 128, 25),
            button_active: Color::Rgb(184, 187, 38),
            button_disabled: Color::Rgb(168, 153, 132),

            command_echo: Color::Rgb(235, 219, 178),
            selection_background: Color::Rgb(80, 73, 69),
            link_color: Color::Rgb(131, 165, 152),
            speech_color: Color::Rgb(184, 187, 38),
            whisper_color: Color::Rgb(142, 192, 124),
            thought_color: Color::Rgb(211, 134, 155),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Night Owl – deep ocean blues with neon highlights
    pub fn night_owl() -> AppTheme {
        let mut theme = AppTheme {
            name: "Night Owl".to_string(),
            description: "Deep indigo background with bright neon highlights".to_string(),

            window_border: Color::Rgb(41, 137, 222),
            window_border_focused: Color::Rgb(128, 255, 183),
            window_background: Color::Rgb(1, 22, 39),
            window_title: Color::Rgb(226, 232, 240),

            text_primary: Color::Rgb(226, 232, 240),
            text_secondary: Color::Rgb(131, 153, 186),
            text_disabled: Color::Rgb(20, 30, 44),
            text_selected: Color::Rgb(41, 137, 222),

            background_primary: Color::Rgb(1, 22, 39),
            background_secondary: Color::Rgb(10, 39, 69),
            background_selected: Color::Rgb(16, 54, 100),
            background_hover: Color::Rgb(10, 39, 69),

            editor_border: Color::Rgb(41, 137, 222),
            editor_label: Color::Rgb(41, 137, 222),
            editor_label_focused: Color::Rgb(255, 179, 64),
            editor_text: Color::Rgb(226, 232, 240),
            editor_cursor: Color::Rgb(128, 255, 183),
            editor_status: Color::Rgb(189, 195, 199),
            editor_background: Color::Rgb(1, 22, 39),

            browser_border: Color::Rgb(41, 137, 222),
            browser_title: Color::Rgb(226, 232, 240),
            browser_item_normal: Color::Rgb(226, 232, 240),
            browser_item_selected: Color::Rgb(1, 22, 39),
            browser_item_focused: Color::Rgb(128, 255, 183),
            browser_background: Color::Rgb(1, 22, 39),
            browser_scrollbar: Color::Rgb(41, 137, 222),

            form_border: Color::Rgb(41, 137, 222),
            form_label: Color::Rgb(131, 153, 186),
            form_label_focused: Color::Rgb(255, 179, 64),
            form_field_background: Color::Rgb(10, 39, 69),
            form_field_text: Color::Rgb(226, 232, 240),
            form_checkbox_checked: Color::Rgb(128, 255, 183),
            form_checkbox_unchecked: Color::Rgb(20, 30, 44),
            form_error: Color::Rgb(255, 99, 132),

            menu_border: Color::Rgb(41, 137, 222),
            menu_background: Color::Rgb(1, 22, 39),
            menu_item_normal: Color::Rgb(226, 232, 240),
            menu_item_selected: Color::Rgb(16, 54, 100),
            menu_item_focused: Color::Rgb(128, 255, 183),
            menu_separator: Color::Rgb(20, 30, 44),

            status_info: Color::Rgb(77, 189, 252),
            status_success: Color::Rgb(103, 255, 173),
            status_warning: Color::Rgb(255, 179, 64),
            status_error: Color::Rgb(255, 100, 115),
            status_background: Color::Rgb(1, 22, 39),

            button_normal: Color::Rgb(41, 137, 222),
            button_hover: Color::Rgb(255, 179, 64),
            button_active: Color::Rgb(103, 255, 173),
            button_disabled: Color::Rgb(20, 30, 44),

            command_echo: Color::Rgb(226, 232, 240),
            selection_background: Color::Rgb(16, 54, 100),
            link_color: Color::Rgb(84, 147, 253),
            speech_color: Color::Rgb(103, 255, 173),
            whisper_color: Color::Rgb(128, 255, 183),
            thought_color: Color::Rgb(255, 179, 64),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Catppuccin Mocha-inspired palette
    pub fn catppuccin() -> AppTheme {
        let mut theme = AppTheme {
            name: "Catppuccin".to_string(),
            description: "Mocha pastels with soft rosy and violet tones".to_string(),

            window_border: Color::Rgb(203, 166, 247),
            window_border_focused: Color::Rgb(245, 194, 231),
            window_background: Color::Rgb(30, 25, 50),
            window_title: Color::Rgb(245, 222, 224),

            text_primary: Color::Rgb(245, 224, 220),
            text_secondary: Color::Rgb(192, 158, 255),
            text_disabled: Color::Rgb(124, 115, 138),
            text_selected: Color::Rgb(245, 194, 231),

            background_primary: Color::Rgb(30, 25, 50),
            background_secondary: Color::Rgb(45, 40, 66),
            background_selected: Color::Rgb(79, 63, 111),
            background_hover: Color::Rgb(42, 35, 68),

            editor_border: Color::Rgb(203, 166, 247),
            editor_label: Color::Rgb(203, 166, 247),
            editor_label_focused: Color::Rgb(245, 194, 231),
            editor_text: Color::Rgb(245, 224, 220),
            editor_cursor: Color::Rgb(243, 139, 168),
            editor_status: Color::Rgb(166, 227, 161),
            editor_background: Color::Rgb(30, 25, 50),

            browser_border: Color::Rgb(203, 166, 247),
            browser_title: Color::Rgb(245, 224, 220),
            browser_item_normal: Color::Rgb(245, 224, 220),
            browser_item_selected: Color::Rgb(30, 25, 50),
            browser_item_focused: Color::Rgb(243, 139, 168),
            browser_background: Color::Rgb(30, 25, 50),
            browser_scrollbar: Color::Rgb(203, 166, 247),

            form_border: Color::Rgb(203, 166, 247),
            form_label: Color::Rgb(243, 139, 168),
            form_label_focused: Color::Rgb(245, 194, 231),
            form_field_background: Color::Rgb(45, 40, 66),
            form_field_text: Color::Rgb(245, 224, 220),
            form_checkbox_checked: Color::Rgb(166, 227, 161),
            form_checkbox_unchecked: Color::Rgb(192, 158, 255),
            form_error: Color::Rgb(245, 139, 168),

            menu_border: Color::Rgb(203, 166, 247),
            menu_background: Color::Rgb(30, 25, 50),
            menu_item_normal: Color::Rgb(245, 224, 220),
            menu_item_selected: Color::Rgb(79, 63, 111),
            menu_item_focused: Color::Rgb(245, 194, 231),
            menu_separator: Color::Rgb(80, 74, 107),

            status_info: Color::Rgb(166, 227, 161),
            status_success: Color::Rgb(148, 226, 213),
            status_warning: Color::Rgb(255, 176, 92),
            status_error: Color::Rgb(245, 139, 168),
            status_background: Color::Rgb(30, 25, 50),

            button_normal: Color::Rgb(203, 166, 247),
            button_hover: Color::Rgb(245, 194, 231),
            button_active: Color::Rgb(166, 227, 161),
            button_disabled: Color::Rgb(124, 115, 138),

            command_echo: Color::Rgb(245, 224, 220),
            selection_background: Color::Rgb(79, 63, 111),
            link_color: Color::Rgb(181, 205, 255),
            speech_color: Color::Rgb(245, 194, 231),
            whisper_color: Color::Rgb(164, 214, 255),
            thought_color: Color::Rgb(203, 166, 247),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Cyberpunk neons on a midnight background
    pub fn cyberpunk() -> AppTheme {
        let mut theme = AppTheme {
            name: "Cyberpunk".to_string(),
            description: "Vibrant neon on pitch-black backgrounds".to_string(),

            window_border: Color::Rgb(255, 0, 128),
            window_border_focused: Color::Rgb(15, 251, 222),
            window_background: Color::Rgb(5, 1, 15),
            window_title: Color::Rgb(254, 254, 254),

            text_primary: Color::Rgb(254, 254, 254),
            text_secondary: Color::Rgb(162, 166, 201),
            text_disabled: Color::Rgb(27, 28, 46),
            text_selected: Color::Rgb(255, 0, 128),

            background_primary: Color::Rgb(5, 1, 15),
            background_secondary: Color::Rgb(16, 11, 29),
            background_selected: Color::Rgb(27, 14, 44),
            background_hover: Color::Rgb(16, 11, 29),

            editor_border: Color::Rgb(255, 0, 128),
            editor_label: Color::Rgb(255, 157, 92),
            editor_label_focused: Color::Rgb(15, 251, 222),
            editor_text: Color::Rgb(254, 254, 254),
            editor_cursor: Color::Rgb(255, 207, 0),
            editor_status: Color::Rgb(133, 255, 203),
            editor_background: Color::Rgb(5, 1, 15),

            browser_border: Color::Rgb(255, 0, 128),
            browser_title: Color::Rgb(254, 254, 254),
            browser_item_normal: Color::Rgb(254, 254, 254),
            browser_item_selected: Color::Rgb(5, 1, 15),
            browser_item_focused: Color::Rgb(15, 251, 222),
            browser_background: Color::Rgb(5, 1, 15),
            browser_scrollbar: Color::Rgb(255, 0, 128),

            form_border: Color::Rgb(255, 0, 128),
            form_label: Color::Rgb(255, 157, 92),
            form_label_focused: Color::Rgb(15, 251, 222),
            form_field_background: Color::Rgb(16, 11, 29),
            form_field_text: Color::Rgb(254, 254, 254),
            form_checkbox_checked: Color::Rgb(255, 207, 0),
            form_checkbox_unchecked: Color::Rgb(162, 166, 201),
            form_error: Color::Rgb(255, 107, 159),

            menu_border: Color::Rgb(255, 0, 128),
            menu_background: Color::Rgb(5, 1, 15),
            menu_item_normal: Color::Rgb(254, 254, 254),
            menu_item_selected: Color::Rgb(27, 14, 44),
            menu_item_focused: Color::Rgb(15, 251, 222),
            menu_separator: Color::Rgb(42, 28, 51),

            status_info: Color::Rgb(15, 251, 222),
            status_success: Color::Rgb(133, 255, 203),
            status_warning: Color::Rgb(255, 207, 0),
            status_error: Color::Rgb(255, 107, 159),
            status_background: Color::Rgb(5, 1, 15),

            button_normal: Color::Rgb(255, 0, 128),
            button_hover: Color::Rgb(255, 157, 92),
            button_active: Color::Rgb(15, 251, 222),
            button_disabled: Color::Rgb(42, 28, 51),

            command_echo: Color::Rgb(254, 254, 254),
            selection_background: Color::Rgb(27, 14, 44),
            link_color: Color::Rgb(137, 180, 255),
            speech_color: Color::Rgb(255, 157, 92),
            whisper_color: Color::Rgb(15, 251, 222),
            thought_color: Color::Rgb(255, 107, 159),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Retro terminal palette (amber/green on black)
    pub fn retro_terminal() -> AppTheme {
        let mut theme = AppTheme {
            name: "Retro Terminal".to_string(),
            description: "Monochrome amber/green on black for retro vibes".to_string(),

            window_border: Color::Rgb(255, 170, 51),
            window_border_focused: Color::Rgb(255, 255, 255),
            window_background: Color::Rgb(4, 12, 11),
            window_title: Color::Rgb(255, 215, 130),

            text_primary: Color::Rgb(255, 249, 199),
            text_secondary: Color::Rgb(195, 165, 93),
            text_disabled: Color::Rgb(63, 59, 50),
            text_selected: Color::Rgb(255, 255, 255),

            background_primary: Color::Rgb(2, 8, 3),
            background_secondary: Color::Rgb(10, 16, 9),
            background_selected: Color::Rgb(27, 40, 15),
            background_hover: Color::Rgb(13, 21, 12),

            editor_border: Color::Rgb(255, 170, 51),
            editor_label: Color::Rgb(255, 215, 130),
            editor_label_focused: Color::Rgb(255, 255, 255),
            editor_text: Color::Rgb(255, 249, 199),
            editor_cursor: Color::Rgb(255, 255, 255),
            editor_status: Color::Rgb(255, 215, 130),
            editor_background: Color::Rgb(2, 8, 3),

            browser_border: Color::Rgb(255, 170, 51),
            browser_title: Color::Rgb(255, 249, 199),
            browser_item_normal: Color::Rgb(255, 249, 199),
            browser_item_selected: Color::Rgb(2, 8, 3),
            browser_item_focused: Color::Rgb(255, 255, 255),
            browser_background: Color::Rgb(2, 8, 3),
            browser_scrollbar: Color::Rgb(255, 170, 51),

            form_border: Color::Rgb(255, 170, 51),
            form_label: Color::Rgb(255, 215, 130),
            form_label_focused: Color::Rgb(255, 255, 255),
            form_field_background: Color::Rgb(10, 16, 9),
            form_field_text: Color::Rgb(255, 249, 199),
            form_checkbox_checked: Color::Rgb(255, 215, 130),
            form_checkbox_unchecked: Color::Rgb(195, 165, 93),
            form_error: Color::Rgb(255, 127, 0),

            menu_border: Color::Rgb(255, 170, 51),
            menu_background: Color::Rgb(2, 8, 3),
            menu_item_normal: Color::Rgb(255, 249, 199),
            menu_item_selected: Color::Rgb(27, 40, 15),
            menu_item_focused: Color::Rgb(255, 255, 255),
            menu_separator: Color::Rgb(27, 40, 15),

            status_info: Color::Rgb(255, 215, 130),
            status_success: Color::Rgb(160, 255, 139),
            status_warning: Color::Rgb(255, 159, 0),
            status_error: Color::Rgb(255, 61, 48),
            status_background: Color::Rgb(2, 8, 3),

            button_normal: Color::Rgb(255, 170, 51),
            button_hover: Color::Rgb(255, 215, 130),
            button_active: Color::Rgb(160, 255, 139),
            button_disabled: Color::Rgb(63, 59, 50),

            command_echo: Color::Rgb(255, 249, 199),
            selection_background: Color::Rgb(27, 40, 15),
            link_color: Color::Rgb(255, 215, 130),
            speech_color: Color::Rgb(160, 255, 139),
            whisper_color: Color::Rgb(255, 255, 255),
            thought_color: Color::Rgb(255, 159, 0),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Apex / Space Station: muted dark slate with neon cyan/orange highlights
    pub fn apex() -> AppTheme {
        let mut theme = AppTheme {
            name: "Apex".to_string(),
            description: "Space station gray with neon cyan & amber accents".to_string(),

            window_border: Color::Rgb(88, 199, 255),
            window_border_focused: Color::Rgb(255, 178, 92),
            window_background: Color::Rgb(5, 10, 17),
            window_title: Color::Rgb(232, 244, 255),

            text_primary: Color::Rgb(232, 244, 255),
            text_secondary: Color::Rgb(141, 169, 195),
            text_disabled: Color::Rgb(43, 58, 78),
            text_selected: Color::Rgb(255, 178, 92),

            background_primary: Color::Rgb(5, 10, 17),
            background_secondary: Color::Rgb(13, 24, 41),
            background_selected: Color::Rgb(25, 44, 72),
            background_hover: Color::Rgb(13, 24, 41),

            editor_border: Color::Rgb(88, 199, 255),
            editor_label: Color::Rgb(88, 199, 255),
            editor_label_focused: Color::Rgb(255, 178, 92),
            editor_text: Color::Rgb(232, 244, 255),
            editor_cursor: Color::Rgb(88, 199, 255),
            editor_status: Color::Rgb(202, 229, 255),
            editor_background: Color::Rgb(5, 10, 17),

            browser_border: Color::Rgb(88, 199, 255),
            browser_title: Color::Rgb(232, 244, 255),
            browser_item_normal: Color::Rgb(232, 244, 255),
            browser_item_selected: Color::Rgb(5, 10, 17),
            browser_item_focused: Color::Rgb(255, 178, 92),
            browser_background: Color::Rgb(5, 10, 17),
            browser_scrollbar: Color::Rgb(88, 199, 255),

            form_border: Color::Rgb(88, 199, 255),
            form_label: Color::Rgb(141, 169, 195),
            form_label_focused: Color::Rgb(255, 178, 92),
            form_field_background: Color::Rgb(13, 24, 41),
            form_field_text: Color::Rgb(232, 244, 255),
            form_checkbox_checked: Color::Rgb(255, 178, 92),
            form_checkbox_unchecked: Color::Rgb(78, 107, 143),
            form_error: Color::Rgb(255, 99, 132),

            menu_border: Color::Rgb(88, 199, 255),
            menu_background: Color::Rgb(5, 10, 17),
            menu_item_normal: Color::Rgb(232, 244, 255),
            menu_item_selected: Color::Rgb(25, 44, 72),
            menu_item_focused: Color::Rgb(255, 178, 92),
            menu_separator: Color::Rgb(35, 54, 76),

            status_info: Color::Rgb(88, 199, 255),
            status_success: Color::Rgb(133, 255, 202),
            status_warning: Color::Rgb(255, 178, 92),
            status_error: Color::Rgb(255, 99, 132),
            status_background: Color::Rgb(5, 10, 17),

            button_normal: Color::Rgb(88, 199, 255),
            button_hover: Color::Rgb(255, 178, 92),
            button_active: Color::Rgb(133, 255, 202),
            button_disabled: Color::Rgb(35, 54, 76),

            command_echo: Color::Rgb(232, 244, 255),
            selection_background: Color::Rgb(25, 44, 72),
            link_color: Color::Rgb(81, 180, 255),
            speech_color: Color::Rgb(133, 255, 202),
            whisper_color: Color::Rgb(88, 199, 255),
            thought_color: Color::Rgb(255, 178, 92),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Minimalist warm: clean paper tones with brown-orange highlights
    pub fn minimalist_warm() -> AppTheme {
        let mut theme = AppTheme {
            name: "Minimalist Warm".to_string(),
            description: "Beige paper with warm brown and amber accents".to_string(),

            window_border: Color::Rgb(136, 95, 64),
            window_border_focused: Color::Rgb(222, 141, 88),
            window_background: Color::Rgb(248, 243, 233),
            window_title: Color::Rgb(61, 42, 31),

            text_primary: Color::Rgb(61, 42, 31),
            text_secondary: Color::Rgb(117, 92, 70),
            text_disabled: Color::Rgb(190, 176, 161),
            text_selected: Color::Rgb(222, 141, 88),

            background_primary: Color::Rgb(248, 243, 233),
            background_secondary: Color::Rgb(239, 229, 216),
            background_selected: Color::Rgb(229, 211, 193),
            background_hover: Color::Rgb(239, 229, 216),

            editor_border: Color::Rgb(136, 95, 64),
            editor_label: Color::Rgb(136, 95, 64),
            editor_label_focused: Color::Rgb(222, 141, 88),
            editor_text: Color::Rgb(61, 42, 31),
            editor_cursor: Color::Rgb(222, 141, 88),
            editor_status: Color::Rgb(117, 92, 70),
            editor_background: Color::Rgb(248, 243, 233),

            browser_border: Color::Rgb(136, 95, 64),
            browser_title: Color::Rgb(61, 42, 31),
            browser_item_normal: Color::Rgb(61, 42, 31),
            browser_item_selected: Color::Rgb(248, 243, 233),
            browser_item_focused: Color::Rgb(222, 141, 88),
            browser_background: Color::Rgb(248, 243, 233),
            browser_scrollbar: Color::Rgb(136, 95, 64),

            form_border: Color::Rgb(136, 95, 64),
            form_label: Color::Rgb(117, 92, 70),
            form_label_focused: Color::Rgb(222, 141, 88),
            form_field_background: Color::Rgb(239, 229, 216),
            form_field_text: Color::Rgb(61, 42, 31),
            form_checkbox_checked: Color::Rgb(222, 141, 88),
            form_checkbox_unchecked: Color::Rgb(152, 125, 101),
            form_error: Color::Rgb(197, 62, 62),

            menu_border: Color::Rgb(136, 95, 64),
            menu_background: Color::Rgb(248, 243, 233),
            menu_item_normal: Color::Rgb(61, 42, 31),
            menu_item_selected: Color::Rgb(229, 211, 193),
            menu_item_focused: Color::Rgb(222, 141, 88),
            menu_separator: Color::Rgb(217, 194, 170),

            status_info: Color::Rgb(136, 95, 64),
            status_success: Color::Rgb(129, 186, 116),
            status_warning: Color::Rgb(222, 141, 88),
            status_error: Color::Rgb(197, 62, 62),
            status_background: Color::Rgb(248, 243, 233),

            button_normal: Color::Rgb(136, 95, 64),
            button_hover: Color::Rgb(222, 141, 88),
            button_active: Color::Rgb(129, 186, 116),
            button_disabled: Color::Rgb(190, 176, 161),

            command_echo: Color::Rgb(61, 42, 31),
            selection_background: Color::Rgb(229, 211, 193),
            link_color: Color::Rgb(79, 115, 160),
            speech_color: Color::Rgb(129, 186, 116),
            whisper_color: Color::Rgb(117, 92, 70),
            thought_color: Color::Rgb(222, 141, 88),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Forest Creek: deep greens with amber moss highlights
    pub fn forest_creek() -> AppTheme {
        let mut theme = AppTheme {
            name: "Forest Creek".to_string(),
            description: "Deep forest greens with mossy amber highlights".to_string(),

            window_border: Color::Rgb(100, 178, 152),
            window_border_focused: Color::Rgb(255, 189, 105),
            window_background: Color::Rgb(5, 20, 14),
            window_title: Color::Rgb(216, 239, 226),

            text_primary: Color::Rgb(216, 239, 226),
            text_secondary: Color::Rgb(146, 184, 162),
            text_disabled: Color::Rgb(45, 74, 63),
            text_selected: Color::Rgb(255, 189, 105),

            background_primary: Color::Rgb(5, 20, 14),
            background_secondary: Color::Rgb(11, 40, 24),
            background_selected: Color::Rgb(32, 72, 48),
            background_hover: Color::Rgb(11, 40, 24),

            editor_border: Color::Rgb(100, 178, 152),
            editor_label: Color::Rgb(100, 178, 152),
            editor_label_focused: Color::Rgb(255, 189, 105),
            editor_text: Color::Rgb(216, 239, 226),
            editor_cursor: Color::Rgb(255, 189, 105),
            editor_status: Color::Rgb(180, 213, 188),
            editor_background: Color::Rgb(5, 20, 14),

            browser_border: Color::Rgb(100, 178, 152),
            browser_title: Color::Rgb(216, 239, 226),
            browser_item_normal: Color::Rgb(216, 239, 226),
            browser_item_selected: Color::Rgb(5, 20, 14),
            browser_item_focused: Color::Rgb(255, 189, 105),
            browser_background: Color::Rgb(5, 20, 14),
            browser_scrollbar: Color::Rgb(100, 178, 152),

            form_border: Color::Rgb(100, 178, 152),
            form_label: Color::Rgb(146, 184, 162),
            form_label_focused: Color::Rgb(255, 189, 105),
            form_field_background: Color::Rgb(11, 40, 24),
            form_field_text: Color::Rgb(216, 239, 226),
            form_checkbox_checked: Color::Rgb(255, 189, 105),
            form_checkbox_unchecked: Color::Rgb(83, 113, 101),
            form_error: Color::Rgb(231, 129, 97),

            menu_border: Color::Rgb(100, 178, 152),
            menu_background: Color::Rgb(5, 20, 14),
            menu_item_normal: Color::Rgb(216, 239, 226),
            menu_item_selected: Color::Rgb(32, 72, 48),
            menu_item_focused: Color::Rgb(255, 189, 105),
            menu_separator: Color::Rgb(44, 87, 70),

            status_info: Color::Rgb(100, 178, 152),
            status_success: Color::Rgb(146, 184, 162),
            status_warning: Color::Rgb(255, 189, 105),
            status_error: Color::Rgb(231, 129, 97),
            status_background: Color::Rgb(5, 20, 14),

            button_normal: Color::Rgb(100, 178, 152),
            button_hover: Color::Rgb(255, 189, 105),
            button_active: Color::Rgb(146, 184, 162),
            button_disabled: Color::Rgb(45, 74, 63),

            command_echo: Color::Rgb(216, 239, 226),
            selection_background: Color::Rgb(32, 72, 48),
            link_color: Color::Rgb(113, 204, 177),
            speech_color: Color::Rgb(146, 184, 162),
            whisper_color: Color::Rgb(89, 148, 118),
            thought_color: Color::Rgb(255, 189, 105),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Synthwave: neon magenta + cyan on deep violet
    pub fn synthwave() -> AppTheme {
        let mut theme = AppTheme {
            name: "Synthwave".to_string(),
            description: "Neon magenta & cyan gradients on a violet noir background".to_string(),

            window_border: Color::Rgb(255, 95, 206),
            window_border_focused: Color::Rgb(92, 255, 255),
            window_background: Color::Rgb(14, 1, 40),
            window_title: Color::Rgb(255, 214, 255),

            text_primary: Color::Rgb(255, 214, 255),
            text_secondary: Color::Rgb(173, 158, 255),
            text_disabled: Color::Rgb(52, 24, 86),
            text_selected: Color::Rgb(92, 255, 255),

            background_primary: Color::Rgb(14, 1, 40),
            background_secondary: Color::Rgb(23, 6, 58),
            background_selected: Color::Rgb(35, 8, 76),
            background_hover: Color::Rgb(23, 6, 58),

            editor_border: Color::Rgb(255, 95, 206),
            editor_label: Color::Rgb(255, 95, 206),
            editor_label_focused: Color::Rgb(92, 255, 255),
            editor_text: Color::Rgb(255, 214, 255),
            editor_cursor: Color::Rgb(255, 207, 109),
            editor_status: Color::Rgb(173, 158, 255),
            editor_background: Color::Rgb(14, 1, 40),

            browser_border: Color::Rgb(255, 95, 206),
            browser_title: Color::Rgb(255, 214, 255),
            browser_item_normal: Color::Rgb(255, 214, 255),
            browser_item_selected: Color::Rgb(14, 1, 40),
            browser_item_focused: Color::Rgb(92, 255, 255),
            browser_background: Color::Rgb(14, 1, 40),
            browser_scrollbar: Color::Rgb(255, 95, 206),

            form_border: Color::Rgb(255, 95, 206),
            form_label: Color::Rgb(173, 158, 255),
            form_label_focused: Color::Rgb(92, 255, 255),
            form_field_background: Color::Rgb(23, 6, 58),
            form_field_text: Color::Rgb(255, 214, 255),
            form_checkbox_checked: Color::Rgb(255, 207, 109),
            form_checkbox_unchecked: Color::Rgb(116, 59, 128),
            form_error: Color::Rgb(255, 49, 112),

            menu_border: Color::Rgb(255, 95, 206),
            menu_background: Color::Rgb(14, 1, 40),
            menu_item_normal: Color::Rgb(255, 214, 255),
            menu_item_selected: Color::Rgb(35, 8, 76),
            menu_item_focused: Color::Rgb(92, 255, 255),
            menu_separator: Color::Rgb(46, 18, 75),

            status_info: Color::Rgb(92, 255, 255),
            status_success: Color::Rgb(173, 255, 129),
            status_warning: Color::Rgb(255, 207, 109),
            status_error: Color::Rgb(255, 49, 112),
            status_background: Color::Rgb(14, 1, 40),

            button_normal: Color::Rgb(255, 95, 206),
            button_hover: Color::Rgb(92, 255, 255),
            button_active: Color::Rgb(255, 207, 109),
            button_disabled: Color::Rgb(52, 24, 86),

            command_echo: Color::Rgb(255, 214, 255),
            selection_background: Color::Rgb(35, 8, 76),
            link_color: Color::Rgb(99, 176, 255),
            speech_color: Color::Rgb(255, 207, 109),
            whisper_color: Color::Rgb(92, 255, 255),
            thought_color: Color::Rgb(255, 95, 206),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Load custom themes from ~/.two-face/themes/ directory
    pub fn load_custom_themes(config_base: Option<&str>) -> HashMap<String, AppTheme> {
        use std::fs;
        use std::path::PathBuf;

        let mut custom_themes = HashMap::new();

        // Determine themes directory path
        let themes_dir = if let Some(base) = config_base {
            PathBuf::from(base).join("themes")
        } else {
            match dirs::home_dir() {
                Some(home) => home.join(".two-face").join("themes"),
                None => {
                    tracing::warn!("Could not determine home directory for custom themes");
                    return custom_themes;
                }
            }
        };

        // Check if themes directory exists
        if !themes_dir.exists() {
            tracing::debug!("Custom themes directory does not exist: {:?}", themes_dir);
            return custom_themes;
        }

        // Read all .toml files in the directory
        match fs::read_dir(&themes_dir) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                        match crate::frontend::tui::theme_editor::ThemeData::load_from_file(&path) {
                            Ok(theme_data) => {
                                if let Some(app_theme) = theme_data.to_app_theme() {
                                    tracing::info!("Loaded custom theme: {}", theme_data.name);
                                    custom_themes.insert(theme_data.name.clone(), app_theme);
                                } else {
                                    tracing::warn!(
                                        "Failed to convert theme data to AppTheme: {:?}",
                                        path
                                    );
                                }
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to load custom theme from {:?}: {}",
                                    path,
                                    e
                                );
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to read custom themes directory {:?}: {}",
                    themes_dir,
                    e
                );
            }
        }

        custom_themes
    }

    /// Get all available themes (built-in + custom)
    pub fn all_with_custom(config_base: Option<&str>) -> HashMap<String, AppTheme> {
        let mut themes = Self::all();
        let custom = Self::load_custom_themes(config_base);
        themes.extend(custom);
        themes
    }
}

impl Default for AppTheme {
    fn default() -> Self {
        ThemePresets::dark()
    }
}
