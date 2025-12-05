//! Frontend-agnostic theme loading and serialization.
//!
//! This module handles loading custom themes from TOML files and converting
//! between the TOML representation (hex colors) and the AppTheme struct (Color types).
//! Both TUI and GUI frontends can use this module to load custom themes.

use crate::frontend::common::Color;
use crate::theme::AppTheme;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Serializable theme data using hex color strings.
/// This is the format used in TOML theme files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeData {
    pub name: String,
    pub description: String,

    // Window colors
    pub window_border: String,
    pub window_border_focused: String,
    pub window_background: String,
    pub window_title: String,

    // Text colors
    pub text_primary: String,
    pub text_secondary: String,
    pub text_disabled: String,
    pub text_selected: String,

    // Background colors
    pub background_primary: String,
    pub background_secondary: String,
    pub background_selected: String,
    pub background_hover: String,

    // Browser/List colors
    pub browser_border: String,
    pub browser_title: String,
    pub browser_item_normal: String,
    pub browser_item_selected: String,
    pub browser_item_focused: String,
    pub browser_background: String,
    pub browser_scrollbar: String,

    // Form colors
    pub form_border: String,
    pub form_label: String,
    pub form_label_focused: String,
    pub form_field_background: String,
    pub form_field_text: String,
    pub form_checkbox_checked: String,
    pub form_checkbox_unchecked: String,
    pub form_error: String,

    // Editor colors
    pub editor_border: String,
    pub editor_label: String,
    pub editor_label_focused: String,
    pub editor_text: String,
    pub editor_cursor: String,
    pub editor_status: String,
    pub editor_background: String,

    // Menu colors
    pub menu_border: String,
    pub menu_background: String,
    pub menu_item_normal: String,
    pub menu_item_selected: String,
    pub menu_item_focused: String,
    pub menu_separator: String,

    // Status bar colors
    pub status_info: String,
    pub status_success: String,
    pub status_warning: String,
    pub status_error: String,
    pub status_background: String,

    // Button colors
    pub button_normal: String,
    pub button_hover: String,
    pub button_active: String,
    pub button_disabled: String,

    // Command and messaging colors
    pub command_echo: String,
    pub selection_background: String,
    pub link_color: String,
    pub speech_color: String,
    pub whisper_color: String,
    pub thought_color: String,

    // Injury doll colors
    pub injury_default_color: String,

    // Legacy/compatibility colors
    pub border_color: String,
    pub label_color: String,
    pub focused_label_color: String,
    pub text_color: String,
}

impl Default for ThemeData {
    fn default() -> Self {
        Self::from_theme(&crate::theme::ThemePresets::dark())
    }
}

impl ThemeData {
    /// Create ThemeData from an existing AppTheme
    pub fn from_theme(theme: &AppTheme) -> Self {
        Self {
            name: theme.name.clone(),
            description: theme.description.clone(),

            window_border: Self::color_to_hex(&theme.window_border),
            window_border_focused: Self::color_to_hex(&theme.window_border_focused),
            window_background: Self::color_to_hex(&theme.window_background),
            window_title: Self::color_to_hex(&theme.window_title),

            text_primary: Self::color_to_hex(&theme.text_primary),
            text_secondary: Self::color_to_hex(&theme.text_secondary),
            text_disabled: Self::color_to_hex(&theme.text_disabled),
            text_selected: Self::color_to_hex(&theme.text_selected),

            background_primary: Self::color_to_hex(&theme.background_primary),
            background_secondary: Self::color_to_hex(&theme.background_secondary),
            background_selected: Self::color_to_hex(&theme.background_selected),
            background_hover: Self::color_to_hex(&theme.background_hover),

            browser_border: Self::color_to_hex(&theme.browser_border),
            browser_title: Self::color_to_hex(&theme.browser_title),
            browser_item_normal: Self::color_to_hex(&theme.browser_item_normal),
            browser_item_selected: Self::color_to_hex(&theme.browser_item_selected),
            browser_item_focused: Self::color_to_hex(&theme.browser_item_focused),
            browser_background: Self::color_to_hex(&theme.browser_background),
            browser_scrollbar: Self::color_to_hex(&theme.browser_scrollbar),

            form_border: Self::color_to_hex(&theme.form_border),
            form_label: Self::color_to_hex(&theme.form_label),
            form_label_focused: Self::color_to_hex(&theme.form_label_focused),
            form_field_background: Self::color_to_hex(&theme.form_field_background),
            form_field_text: Self::color_to_hex(&theme.form_field_text),
            form_checkbox_checked: Self::color_to_hex(&theme.form_checkbox_checked),
            form_checkbox_unchecked: Self::color_to_hex(&theme.form_checkbox_unchecked),
            form_error: Self::color_to_hex(&theme.form_error),

            editor_border: Self::color_to_hex(&theme.editor_border),
            editor_label: Self::color_to_hex(&theme.editor_label),
            editor_label_focused: Self::color_to_hex(&theme.editor_label_focused),
            editor_text: Self::color_to_hex(&theme.editor_text),
            editor_cursor: Self::color_to_hex(&theme.editor_cursor),
            editor_status: Self::color_to_hex(&theme.editor_status),
            editor_background: Self::color_to_hex(&theme.editor_background),

            menu_border: Self::color_to_hex(&theme.menu_border),
            menu_background: Self::color_to_hex(&theme.menu_background),
            menu_item_normal: Self::color_to_hex(&theme.menu_item_normal),
            menu_item_selected: Self::color_to_hex(&theme.menu_item_selected),
            menu_item_focused: Self::color_to_hex(&theme.menu_item_focused),
            menu_separator: Self::color_to_hex(&theme.menu_separator),

            status_info: Self::color_to_hex(&theme.status_info),
            status_success: Self::color_to_hex(&theme.status_success),
            status_warning: Self::color_to_hex(&theme.status_warning),
            status_error: Self::color_to_hex(&theme.status_error),
            status_background: Self::color_to_hex(&theme.status_background),

            button_normal: Self::color_to_hex(&theme.button_normal),
            button_hover: Self::color_to_hex(&theme.button_hover),
            button_active: Self::color_to_hex(&theme.button_active),
            button_disabled: Self::color_to_hex(&theme.button_disabled),

            command_echo: Self::color_to_hex(&theme.command_echo),
            selection_background: Self::color_to_hex(&theme.selection_background),
            link_color: Self::color_to_hex(&theme.link_color),
            speech_color: Self::color_to_hex(&theme.speech_color),
            whisper_color: Self::color_to_hex(&theme.whisper_color),
            thought_color: Self::color_to_hex(&theme.thought_color),

            injury_default_color: Self::color_to_hex(&theme.injury_default_color),

            // Legacy fields - map to editor equivalents
            border_color: Self::color_to_hex(&theme.editor_border),
            label_color: Self::color_to_hex(&theme.editor_label),
            focused_label_color: Self::color_to_hex(&theme.editor_label_focused),
            text_color: Self::color_to_hex(&theme.editor_text),
        }
    }

    /// Convert frontend-agnostic Color to hex string
    fn color_to_hex(color: &Color) -> String {
        format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
    }

    /// Convert to AppTheme
    pub fn to_app_theme(&self) -> Option<AppTheme> {
        // Parse all colors
        let window_border = Self::parse_color(&self.window_border)?;
        let window_border_focused = Self::parse_color(&self.window_border_focused)?;
        let window_background = Self::parse_color(&self.window_background)?;
        let window_title = Self::parse_color(&self.window_title)?;

        let text_primary = Self::parse_color(&self.text_primary)?;
        let text_secondary = Self::parse_color(&self.text_secondary)?;
        let text_disabled = Self::parse_color(&self.text_disabled)?;
        let text_selected = Self::parse_color(&self.text_selected)?;

        let background_primary = Self::parse_color(&self.background_primary)?;
        let background_secondary = Self::parse_color(&self.background_secondary)?;
        let background_selected = Self::parse_color(&self.background_selected)?;
        let background_hover = Self::parse_color(&self.background_hover)?;

        let browser_border = Self::parse_color(&self.browser_border)?;
        let browser_title = Self::parse_color(&self.browser_title)?;
        let browser_item_normal = Self::parse_color(&self.browser_item_normal)?;
        let browser_item_selected = Self::parse_color(&self.browser_item_selected)?;
        let browser_item_focused = Self::parse_color(&self.browser_item_focused)?;
        let browser_background = Self::parse_color(&self.browser_background)?;
        let browser_scrollbar = Self::parse_color(&self.browser_scrollbar)?;

        let form_border = Self::parse_color(&self.form_border)?;
        let form_label = Self::parse_color(&self.form_label)?;
        let form_label_focused = Self::parse_color(&self.form_label_focused)?;
        let form_field_background = Self::parse_color(&self.form_field_background)?;
        let form_field_text = Self::parse_color(&self.form_field_text)?;
        let form_checkbox_checked = Self::parse_color(&self.form_checkbox_checked)?;
        let form_checkbox_unchecked = Self::parse_color(&self.form_checkbox_unchecked)?;
        let form_error = Self::parse_color(&self.form_error)?;

        let editor_border = Self::parse_color(&self.editor_border)?;
        let editor_label = Self::parse_color(&self.editor_label)?;
        let editor_label_focused = Self::parse_color(&self.editor_label_focused)?;
        let editor_text = Self::parse_color(&self.editor_text)?;
        let editor_cursor = Self::parse_color(&self.editor_cursor)?;
        let editor_status = Self::parse_color(&self.editor_status)?;
        let editor_background = Self::parse_color(&self.editor_background)?;

        let menu_border = Self::parse_color(&self.menu_border)?;
        let menu_background = Self::parse_color(&self.menu_background)?;
        let menu_item_normal = Self::parse_color(&self.menu_item_normal)?;
        let menu_item_selected = Self::parse_color(&self.menu_item_selected)?;
        let menu_item_focused = Self::parse_color(&self.menu_item_focused)?;
        let menu_separator = Self::parse_color(&self.menu_separator)?;

        let status_info = Self::parse_color(&self.status_info)?;
        let status_success = Self::parse_color(&self.status_success)?;
        let status_warning = Self::parse_color(&self.status_warning)?;
        let status_error = Self::parse_color(&self.status_error)?;
        let status_background = Self::parse_color(&self.status_background)?;

        let button_normal = Self::parse_color(&self.button_normal)?;
        let button_hover = Self::parse_color(&self.button_hover)?;
        let button_active = Self::parse_color(&self.button_active)?;
        let button_disabled = Self::parse_color(&self.button_disabled)?;

        let command_echo = Self::parse_color(&self.command_echo)?;
        let selection_background = Self::parse_color(&self.selection_background)?;
        let link_color = Self::parse_color(&self.link_color)?;
        let speech_color = Self::parse_color(&self.speech_color)?;
        let whisper_color = Self::parse_color(&self.whisper_color)?;
        let thought_color = Self::parse_color(&self.thought_color)?;

        let injury_default_color = Self::parse_color(&self.injury_default_color)?;

        let border_color = Self::parse_color(&self.border_color)?;
        let label_color = Self::parse_color(&self.label_color)?;
        let focused_label_color = Self::parse_color(&self.focused_label_color)?;
        let text_color = Self::parse_color(&self.text_color)?;

        Some(AppTheme {
            name: self.name.clone(),
            description: self.description.clone(),
            window_border,
            window_border_focused,
            window_background,
            window_title,
            text_primary,
            text_secondary,
            text_disabled,
            text_selected,
            background_primary,
            background_secondary,
            background_selected,
            background_hover,
            browser_border,
            browser_title,
            browser_item_normal,
            browser_item_selected,
            browser_item_focused,
            browser_background,
            browser_scrollbar,
            form_border,
            form_label,
            form_label_focused,
            form_field_background,
            form_field_text,
            form_checkbox_checked,
            form_checkbox_unchecked,
            form_error,
            editor_border,
            editor_label,
            editor_label_focused,
            editor_text,
            editor_cursor,
            editor_status,
            editor_background,
            menu_border,
            menu_background,
            menu_item_normal,
            menu_item_selected,
            menu_item_focused,
            menu_separator,
            status_info,
            status_success,
            status_warning,
            status_error,
            status_background,
            button_normal,
            button_hover,
            button_active,
            button_disabled,
            command_echo,
            selection_background,
            link_color,
            speech_color,
            whisper_color,
            thought_color,
            injury_default_color,
            // Legacy fields removed - they don't exist in AppTheme
            // (ThemeData still has them for backward compatibility)
        })
    }

    /// Parse hex color string to Color
    pub fn parse_color(hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Color::rgb(r, g, b))
    }

    /// Resolve all palette color names to hex codes using the provided config
    ///
    /// This mutates all color fields in-place, converting any palette color names
    /// to their corresponding hex values (e.g., "Primary Blue" → "#0066cc")
    pub fn resolve_palette_colors(&mut self, config: &crate::config::Config) {
        // Window colors
        self.window_border = config.resolve_palette_color(&self.window_border);
        self.window_border_focused = config.resolve_palette_color(&self.window_border_focused);
        self.window_background = config.resolve_palette_color(&self.window_background);
        self.window_title = config.resolve_palette_color(&self.window_title);

        // Text colors
        self.text_primary = config.resolve_palette_color(&self.text_primary);
        self.text_secondary = config.resolve_palette_color(&self.text_secondary);
        self.text_disabled = config.resolve_palette_color(&self.text_disabled);
        self.text_selected = config.resolve_palette_color(&self.text_selected);

        // Background colors
        self.background_primary = config.resolve_palette_color(&self.background_primary);
        self.background_secondary = config.resolve_palette_color(&self.background_secondary);
        self.background_selected = config.resolve_palette_color(&self.background_selected);
        self.background_hover = config.resolve_palette_color(&self.background_hover);

        // Browser/List colors
        self.browser_border = config.resolve_palette_color(&self.browser_border);
        self.browser_title = config.resolve_palette_color(&self.browser_title);
        self.browser_item_normal = config.resolve_palette_color(&self.browser_item_normal);
        self.browser_item_selected = config.resolve_palette_color(&self.browser_item_selected);
        self.browser_item_focused = config.resolve_palette_color(&self.browser_item_focused);
        self.browser_background = config.resolve_palette_color(&self.browser_background);
        self.browser_scrollbar = config.resolve_palette_color(&self.browser_scrollbar);

        // Form colors
        self.form_border = config.resolve_palette_color(&self.form_border);
        self.form_label = config.resolve_palette_color(&self.form_label);
        self.form_label_focused = config.resolve_palette_color(&self.form_label_focused);
        self.form_field_background = config.resolve_palette_color(&self.form_field_background);
        self.form_field_text = config.resolve_palette_color(&self.form_field_text);
        self.form_checkbox_checked = config.resolve_palette_color(&self.form_checkbox_checked);
        self.form_checkbox_unchecked = config.resolve_palette_color(&self.form_checkbox_unchecked);
        self.form_error = config.resolve_palette_color(&self.form_error);

        // Editor colors
        self.editor_border = config.resolve_palette_color(&self.editor_border);
        self.editor_label = config.resolve_palette_color(&self.editor_label);
        self.editor_label_focused = config.resolve_palette_color(&self.editor_label_focused);
        self.editor_text = config.resolve_palette_color(&self.editor_text);
        self.editor_cursor = config.resolve_palette_color(&self.editor_cursor);
        self.editor_status = config.resolve_palette_color(&self.editor_status);
        self.editor_background = config.resolve_palette_color(&self.editor_background);

        // Menu colors
        self.menu_border = config.resolve_palette_color(&self.menu_border);
        self.menu_background = config.resolve_palette_color(&self.menu_background);
        self.menu_item_normal = config.resolve_palette_color(&self.menu_item_normal);
        self.menu_item_selected = config.resolve_palette_color(&self.menu_item_selected);
        self.menu_item_focused = config.resolve_palette_color(&self.menu_item_focused);
        self.menu_separator = config.resolve_palette_color(&self.menu_separator);

        // Status bar colors
        self.status_info = config.resolve_palette_color(&self.status_info);
        self.status_success = config.resolve_palette_color(&self.status_success);
        self.status_warning = config.resolve_palette_color(&self.status_warning);
        self.status_error = config.resolve_palette_color(&self.status_error);
        self.status_background = config.resolve_palette_color(&self.status_background);

        // Button colors
        self.button_normal = config.resolve_palette_color(&self.button_normal);
        self.button_hover = config.resolve_palette_color(&self.button_hover);
        self.button_active = config.resolve_palette_color(&self.button_active);
        self.button_disabled = config.resolve_palette_color(&self.button_disabled);

        // Command and messaging colors
        self.command_echo = config.resolve_palette_color(&self.command_echo);
        self.selection_background = config.resolve_palette_color(&self.selection_background);
        self.link_color = config.resolve_palette_color(&self.link_color);
        self.speech_color = config.resolve_palette_color(&self.speech_color);
        self.whisper_color = config.resolve_palette_color(&self.whisper_color);
        self.thought_color = config.resolve_palette_color(&self.thought_color);

        // Injury doll colors
        self.injury_default_color = config.resolve_palette_color(&self.injury_default_color);

        // Legacy/compatibility colors
        self.border_color = config.resolve_palette_color(&self.border_color);
        self.label_color = config.resolve_palette_color(&self.label_color);
        self.focused_label_color = config.resolve_palette_color(&self.focused_label_color);
        self.text_color = config.resolve_palette_color(&self.text_color);
    }

    /// Save this theme to a TOML file in ~/.two-face/themes/
    pub fn save_to_file(&self, config_base: Option<&str>) -> Result<PathBuf> {
        // Determine themes directory path
        let themes_dir = if let Some(base) = config_base {
            PathBuf::from(base).join("themes")
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
            home.join(".two-face").join("themes")
        };

        // Create themes directory if it doesn't exist
        fs::create_dir_all(&themes_dir)?;

        // Sanitize filename (remove invalid characters)
        let filename = self
            .name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        let filepath = themes_dir.join(format!("{}.toml", filename));

        // Serialize to TOML
        let toml_string = toml::to_string_pretty(self)?;

        // Write to file
        fs::write(&filepath, toml_string)?;

        Ok(filepath)
    }

    /// Load a theme from a TOML file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let theme_data: ThemeData = toml::from_str(&contents)?;
        Ok(theme_data)
    }
}
