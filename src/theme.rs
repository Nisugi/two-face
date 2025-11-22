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
            section_header_color: self.editor_border, // Reuse border color for section headers
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

/// Color filter that can be applied to any theme for real-time color transformation
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ColorFilter {
    /// No filter applied
    None,
    /// Convert all colors to grayscale
    Grayscale,
    /// Simulate deuteranopia (red-green colorblindness)
    DeuteranopiaSimulation,
    /// Simulate protanopia (another form of red-green colorblindness)
    ProtanopiaSimulation,
    /// Simulate tritanopia (blue-yellow colorblindness)
    TritanopiaSimulation,
    /// Apply sepia tone filter
    Sepia,
    /// Reduce blue light with adjustable intensity (0.0 to 1.0)
    BlueLightFilter(f32),
}

impl Default for ColorFilter {
    fn default() -> Self {
        Self::None
    }
}

impl ColorFilter {
    /// Get all available color filters
    pub fn all() -> Vec<ColorFilter> {
        vec![
            ColorFilter::None,
            ColorFilter::Grayscale,
            ColorFilter::DeuteranopiaSimulation,
            ColorFilter::ProtanopiaSimulation,
            ColorFilter::TritanopiaSimulation,
            ColorFilter::Sepia,
            ColorFilter::BlueLightFilter(0.5),
        ]
    }

    /// Get a human-readable name for the filter
    pub fn name(&self) -> String {
        match self {
            ColorFilter::None => "None".to_string(),
            ColorFilter::Grayscale => "Grayscale".to_string(),
            ColorFilter::DeuteranopiaSimulation => "Deuteranopia Simulation".to_string(),
            ColorFilter::ProtanopiaSimulation => "Protanopia Simulation".to_string(),
            ColorFilter::TritanopiaSimulation => "Tritanopia Simulation".to_string(),
            ColorFilter::Sepia => "Sepia Tone".to_string(),
            ColorFilter::BlueLightFilter(intensity) => {
                format!("Blue Light Filter ({}%)", (intensity * 100.0) as i32)
            }
        }
    }

    /// Get a description of what the filter does
    pub fn description(&self) -> &'static str {
        match self {
            ColorFilter::None => "No color transformation applied",
            ColorFilter::Grayscale => "Convert all colors to grayscale (for achromatopsia or testing)",
            ColorFilter::DeuteranopiaSimulation => "Simulate how colors appear with deuteranopia",
            ColorFilter::ProtanopiaSimulation => "Simulate how colors appear with protanopia",
            ColorFilter::TritanopiaSimulation => "Simulate how colors appear with tritanopia",
            ColorFilter::Sepia => "Apply warm sepia tone filter for reduced eye strain",
            ColorFilter::BlueLightFilter(_) => "Reduce blue light wavelengths for evening use",
        }
    }

    /// Apply this filter to a color
    pub fn apply(&self, color: Color) -> Color {
        match self {
            ColorFilter::None => color,
            ColorFilter::Grayscale => Self::apply_grayscale(color),
            ColorFilter::DeuteranopiaSimulation => Self::apply_deuteranopia(color),
            ColorFilter::ProtanopiaSimulation => Self::apply_protanopia(color),
            ColorFilter::TritanopiaSimulation => Self::apply_tritanopia(color),
            ColorFilter::Sepia => Self::apply_sepia(color),
            ColorFilter::BlueLightFilter(intensity) => Self::apply_blue_light_filter(color, *intensity),
        }
    }

    /// Convert color to grayscale using luminance formula
    fn apply_grayscale(color: Color) -> Color {
        let (r, g, b) = color_to_rgb_components(color);

        // Use standard luminance formula (ITU-R BT.709)
        let gray = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) as u8;

        Color::Rgb(gray, gray, gray)
    }

    /// Simulate deuteranopia (red-green colorblindness - most common)
    /// Uses Brettel et al. (1997) transformation
    fn apply_deuteranopia(color: Color) -> Color {
        let (r, g, b) = color_to_rgb_components(color);

        // Simplified deuteranopia transformation matrix
        let new_r = (0.625 * r as f32 + 0.375 * g as f32).min(255.0) as u8;
        let new_g = (0.7 * g as f32 + 0.3 * r as f32).min(255.0) as u8;
        let new_b = b; // Blue channel unaffected

        Color::Rgb(new_r, new_g, new_b)
    }

    /// Simulate protanopia (another form of red-green colorblindness)
    fn apply_protanopia(color: Color) -> Color {
        let (r, g, b) = color_to_rgb_components(color);

        // Simplified protanopia transformation matrix
        let new_r = (0.567 * r as f32 + 0.433 * g as f32).min(255.0) as u8;
        let new_g = (0.558 * g as f32 + 0.442 * r as f32).min(255.0) as u8;
        let new_b = b; // Blue channel unaffected

        Color::Rgb(new_r, new_g, new_b)
    }

    /// Simulate tritanopia (blue-yellow colorblindness - rare)
    fn apply_tritanopia(color: Color) -> Color {
        let (r, g, b) = color_to_rgb_components(color);

        // Simplified tritanopia transformation matrix
        let new_r = r; // Red channel unaffected
        let new_g = (0.95 * g as f32 + 0.05 * b as f32).min(255.0) as u8;
        let new_b = (0.433 * g as f32 + 0.567 * b as f32).min(255.0) as u8;

        Color::Rgb(new_r, new_g, new_b)
    }

    /// Apply sepia tone filter
    fn apply_sepia(color: Color) -> Color {
        let (r, g, b) = color_to_rgb_components(color);

        // Standard sepia transformation
        let new_r = ((0.393 * r as f32 + 0.769 * g as f32 + 0.189 * b as f32).min(255.0)) as u8;
        let new_g = ((0.349 * r as f32 + 0.686 * g as f32 + 0.168 * b as f32).min(255.0)) as u8;
        let new_b = ((0.272 * r as f32 + 0.534 * g as f32 + 0.131 * b as f32).min(255.0)) as u8;

        Color::Rgb(new_r, new_g, new_b)
    }

    /// Reduce blue light wavelengths
    fn apply_blue_light_filter(color: Color, intensity: f32) -> Color {
        let (r, g, b) = color_to_rgb_components(color);
        let intensity = intensity.clamp(0.0, 1.0);

        // Reduce blue channel and slightly boost warm colors
        let new_r = ((r as f32 * (1.0 + intensity * 0.1)).min(255.0)) as u8;
        let new_g = ((g as f32 * (1.0 + intensity * 0.05)).min(255.0)) as u8;
        let new_b = ((b as f32 * (1.0 - intensity * 0.6)).max(0.0)) as u8;

        Color::Rgb(new_r, new_g, new_b)
    }
}

/// Theme variant modifiers that can be applied to any base theme
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeVariant {
    /// Standard theme (no modifications)
    Standard,
    /// High contrast variant - boosts contrast for low vision users
    HighContrast,
    /// Colorblind-friendly variant - transforms colors for deuteranopia/protanopia
    Colorblind,
    /// Low blue light variant - reduces blue wavelengths for evening use
    LowBlueLight,
}

impl Default for ThemeVariant {
    fn default() -> Self {
        Self::Standard
    }
}

impl ThemeVariant {
    /// Get all available variants
    pub fn all() -> Vec<ThemeVariant> {
        vec![
            ThemeVariant::Standard,
            ThemeVariant::HighContrast,
            ThemeVariant::Colorblind,
            ThemeVariant::LowBlueLight,
        ]
    }

    /// Get a human-readable name for the variant
    pub fn name(&self) -> &'static str {
        match self {
            ThemeVariant::Standard => "Standard",
            ThemeVariant::HighContrast => "High Contrast",
            ThemeVariant::Colorblind => "Colorblind Friendly",
            ThemeVariant::LowBlueLight => "Low Blue Light",
        }
    }

    /// Get a description of what the variant does
    pub fn description(&self) -> &'static str {
        match self {
            ThemeVariant::Standard => "Standard theme with no modifications",
            ThemeVariant::HighContrast => "Boosts contrast ratios for low vision users",
            ThemeVariant::Colorblind => "Transforms colors to be safe for red-green colorblindness",
            ThemeVariant::LowBlueLight => "Reduces blue light for comfortable evening use",
        }
    }

    /// Apply this variant to a color
    fn transform_color(&self, color: Color) -> Color {
        match self {
            ThemeVariant::Standard => color,
            ThemeVariant::HighContrast => Self::apply_high_contrast(color),
            ThemeVariant::Colorblind => Self::apply_colorblind_safe(color),
            ThemeVariant::LowBlueLight => Self::apply_low_blue_light(color),
        }
    }

    /// High contrast transformation - makes light colors lighter and dark colors darker
    fn apply_high_contrast(color: Color) -> Color {
        let (r, g, b) = color_to_rgb_components(color);

        // Calculate luminance
        let luminance = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) / 255.0;

        // If luminance > 0.5, make it lighter; otherwise make it darker
        let boost = if luminance > 0.5 {
            // Light color - boost towards white
            let factor = 1.5;
            Color::Rgb(
                ((r as f32 + (255 - r) as f32 * factor / 2.0).min(255.0)) as u8,
                ((g as f32 + (255 - g) as f32 * factor / 2.0).min(255.0)) as u8,
                ((b as f32 + (255 - b) as f32 * factor / 2.0).min(255.0)) as u8,
            )
        } else {
            // Dark color - reduce towards black
            let factor = 0.5;
            Color::Rgb(
                ((r as f32 * factor).max(0.0)) as u8,
                ((g as f32 * factor).max(0.0)) as u8,
                ((b as f32 * factor).max(0.0)) as u8,
            )
        };

        boost
    }

    /// Colorblind-safe transformation - converts to deuteranopia/protanopia safe palette
    fn apply_colorblind_safe(color: Color) -> Color {
        let (r, g, b) = color_to_rgb_components(color);

        // Calculate luminance to preserve brightness
        let luminance = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;

        // Determine the dominant color characteristic
        let is_reddish = r > g && r > b;
        let is_greenish = g > r && g > b;
        let is_bluish = b > r && b > g;

        // Map problematic colors to safe alternatives
        if is_reddish {
            // Red -> Magenta/Pink (distinguishable for colorblind users)
            let intensity = (luminance / 255.0).clamp(0.0, 1.0);
            Color::Rgb(
                (255.0 * intensity) as u8,
                (105.0 * intensity) as u8,
                (180.0 * intensity) as u8,
            )
        } else if is_greenish {
            // Green -> Blue (safe alternative)
            let intensity = (luminance / 255.0).clamp(0.0, 1.0);
            Color::Rgb(
                0,
                (191.0 * intensity) as u8,
                (255.0 * intensity) as u8,
            )
        } else if is_bluish {
            // Blue stays blue (safe)
            color
        } else {
            // Grayscale or mixed - preserve as-is
            color
        }
    }

    /// Low blue light transformation - reduces blue channel for evening use
    fn apply_low_blue_light(color: Color) -> Color {
        let (r, g, b) = color_to_rgb_components(color);

        // Reduce blue channel by 50% and shift towards warm colors
        let warm_r = ((r as f32 * 1.1).min(255.0)) as u8;
        let warm_g = ((g as f32 * 1.05).min(255.0)) as u8;
        let warm_b = ((b as f32 * 0.5).max(0.0)) as u8;

        Color::Rgb(warm_r, warm_g, warm_b)
    }
}

impl AppTheme {
    /// Apply a theme variant to this theme, creating a new transformed theme
    pub fn with_variant(&self, variant: ThemeVariant) -> AppTheme {
        if variant == ThemeVariant::Standard {
            return self.clone();
        }

        let mut theme = self.clone();

        // Update name and description to reflect variant
        theme.name = format!("{} ({})", self.name, variant.name());
        theme.description = format!("{} - {}", self.description, variant.description());

        // Apply variant transformation to all colors
        theme.window_border = variant.transform_color(theme.window_border);
        theme.window_border_focused = variant.transform_color(theme.window_border_focused);
        theme.window_background = variant.transform_color(theme.window_background);
        theme.window_title = variant.transform_color(theme.window_title);

        theme.text_primary = variant.transform_color(theme.text_primary);
        theme.text_secondary = variant.transform_color(theme.text_secondary);
        theme.text_disabled = variant.transform_color(theme.text_disabled);
        theme.text_selected = variant.transform_color(theme.text_selected);

        theme.background_primary = variant.transform_color(theme.background_primary);
        theme.background_secondary = variant.transform_color(theme.background_secondary);
        theme.background_selected = variant.transform_color(theme.background_selected);
        theme.background_hover = variant.transform_color(theme.background_hover);

        theme.editor_border = variant.transform_color(theme.editor_border);
        theme.editor_label = variant.transform_color(theme.editor_label);
        theme.editor_label_focused = variant.transform_color(theme.editor_label_focused);
        theme.editor_text = variant.transform_color(theme.editor_text);
        theme.editor_cursor = variant.transform_color(theme.editor_cursor);
        theme.editor_status = variant.transform_color(theme.editor_status);
        theme.editor_background = variant.transform_color(theme.editor_background);

        theme.browser_border = variant.transform_color(theme.browser_border);
        theme.browser_title = variant.transform_color(theme.browser_title);
        theme.browser_item_normal = variant.transform_color(theme.browser_item_normal);
        theme.browser_item_selected = variant.transform_color(theme.browser_item_selected);
        theme.browser_item_focused = variant.transform_color(theme.browser_item_focused);
        theme.browser_background = variant.transform_color(theme.browser_background);
        theme.browser_scrollbar = variant.transform_color(theme.browser_scrollbar);

        theme.form_border = variant.transform_color(theme.form_border);
        theme.form_label = variant.transform_color(theme.form_label);
        theme.form_label_focused = variant.transform_color(theme.form_label_focused);
        theme.form_field_background = variant.transform_color(theme.form_field_background);
        theme.form_field_text = variant.transform_color(theme.form_field_text);
        theme.form_checkbox_checked = variant.transform_color(theme.form_checkbox_checked);
        theme.form_checkbox_unchecked = variant.transform_color(theme.form_checkbox_unchecked);
        theme.form_error = variant.transform_color(theme.form_error);

        theme.menu_border = variant.transform_color(theme.menu_border);
        theme.menu_background = variant.transform_color(theme.menu_background);
        theme.menu_item_normal = variant.transform_color(theme.menu_item_normal);
        theme.menu_item_selected = variant.transform_color(theme.menu_item_selected);
        theme.menu_item_focused = variant.transform_color(theme.menu_item_focused);
        theme.menu_separator = variant.transform_color(theme.menu_separator);

        theme.status_info = variant.transform_color(theme.status_info);
        theme.status_success = variant.transform_color(theme.status_success);
        theme.status_warning = variant.transform_color(theme.status_warning);
        theme.status_error = variant.transform_color(theme.status_error);
        theme.status_background = variant.transform_color(theme.status_background);

        theme.button_normal = variant.transform_color(theme.button_normal);
        theme.button_hover = variant.transform_color(theme.button_hover);
        theme.button_active = variant.transform_color(theme.button_active);
        theme.button_disabled = variant.transform_color(theme.button_disabled);

        theme.command_echo = variant.transform_color(theme.command_echo);
        theme.selection_background = variant.transform_color(theme.selection_background);
        theme.link_color = variant.transform_color(theme.link_color);
        theme.speech_color = variant.transform_color(theme.speech_color);
        theme.whisper_color = variant.transform_color(theme.whisper_color);
        theme.thought_color = variant.transform_color(theme.thought_color);

        theme.injury_default_color = variant.transform_color(theme.injury_default_color);

        theme
    }

    /// Apply dynamic contrast adjustment to the theme
    ///
    /// # Arguments
    /// * `multiplier` - Contrast boost multiplier (1.0 = no change, 1.5 = 50% more contrast, etc.)
    ///
    /// # Examples
    /// ```
    /// let theme = ThemePresets::dark();
    /// let high_contrast = theme.with_contrast_boost(1.5); // 50% more contrast
    /// let low_contrast = theme.with_contrast_boost(0.7);  // 30% less contrast
    /// ```
    pub fn with_contrast_boost(&self, multiplier: f32) -> AppTheme {
        if (multiplier - 1.0).abs() < 0.01 {
            return self.clone();
        }

        let mut theme = self.clone();

        // Update description to reflect contrast adjustment
        if multiplier > 1.0 {
            theme.name = format!("{} (+{}% contrast)", self.name, ((multiplier - 1.0) * 100.0) as i32);
            theme.description = format!("{} - Boosted contrast by {}%", self.description, ((multiplier - 1.0) * 100.0) as i32);
        } else {
            theme.name = format!("{} ({}% contrast)", self.name, ((1.0 - multiplier) * 100.0) as i32);
            theme.description = format!("{} - Reduced contrast by {}%", self.description, ((1.0 - multiplier) * 100.0) as i32);
        }

        // Helper function to boost contrast between a color and a reference
        let boost_contrast = |color: Color, reference: Color| -> Color {
            let (r, g, b) = color_to_rgb_components(color);
            let (ref_r, ref_g, ref_b) = color_to_rgb_components(reference);

            // Calculate luminance of both colors
            let lum = 0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32;
            let ref_lum = 0.299 * ref_r as f32 + 0.587 * ref_g as f32 + 0.114 * ref_b as f32;

            // Determine if this is a light or dark color relative to reference
            let is_lighter = lum > ref_lum;

            // Apply contrast boost by pushing colors away from the reference
            let boost_component = |c: u8, ref_c: u8| -> u8 {
                let diff = c as f32 - ref_c as f32;
                let boosted = ref_c as f32 + (diff * multiplier);
                boosted.clamp(0.0, 255.0) as u8
            };

            Color::Rgb(
                boost_component(r, ref_r),
                boost_component(g, ref_g),
                boost_component(b, ref_b),
            )
        };

        // Use background as the reference point for contrast
        let bg_ref = theme.window_background;

        // Apply contrast boost to text colors (most important for readability)
        theme.text_primary = boost_contrast(theme.text_primary, bg_ref);
        theme.text_secondary = boost_contrast(theme.text_secondary, bg_ref);
        theme.text_disabled = boost_contrast(theme.text_disabled, bg_ref);
        theme.text_selected = boost_contrast(theme.text_selected, bg_ref);

        // Apply to browser items
        theme.browser_item_normal = boost_contrast(theme.browser_item_normal, theme.browser_background);
        theme.browser_item_focused = boost_contrast(theme.browser_item_focused, theme.browser_background);
        theme.browser_item_selected = boost_contrast(theme.browser_item_selected, theme.browser_background);

        // Apply to form elements
        theme.form_label = boost_contrast(theme.form_label, theme.browser_background);
        theme.form_label_focused = boost_contrast(theme.form_label_focused, theme.browser_background);
        theme.form_field_text = boost_contrast(theme.form_field_text, theme.form_field_background);

        // Apply to editor elements
        theme.editor_text = boost_contrast(theme.editor_text, theme.editor_background);
        theme.editor_label = boost_contrast(theme.editor_label, theme.editor_background);
        theme.editor_label_focused = boost_contrast(theme.editor_label_focused, theme.editor_background);

        // Apply to menu items
        theme.menu_item_normal = boost_contrast(theme.menu_item_normal, theme.menu_background);
        theme.menu_item_focused = boost_contrast(theme.menu_item_focused, theme.menu_background);
        theme.menu_item_selected = boost_contrast(theme.menu_item_selected, theme.menu_background);

        // Apply to borders for better definition
        theme.window_border = boost_contrast(theme.window_border, bg_ref);
        theme.window_border_focused = boost_contrast(theme.window_border_focused, bg_ref);
        theme.browser_border = boost_contrast(theme.browser_border, theme.browser_background);
        theme.form_border = boost_contrast(theme.form_border, theme.browser_background);
        theme.menu_border = boost_contrast(theme.menu_border, theme.menu_background);

        // Apply to status colors
        theme.status_info = boost_contrast(theme.status_info, bg_ref);
        theme.status_success = boost_contrast(theme.status_success, bg_ref);
        theme.status_warning = boost_contrast(theme.status_warning, bg_ref);
        theme.status_error = boost_contrast(theme.status_error, bg_ref);

        theme
    }

    /// Apply both a variant and contrast adjustment in one operation
    ///
    /// This is more efficient than calling `with_variant()` and `with_contrast_boost()` separately
    ///
    /// # Examples
    /// ```
    /// let theme = ThemePresets::ocean_depths();
    /// let adjusted = theme.with_variant_and_contrast(ThemeVariant::HighContrast, 1.3);
    /// ```
    pub fn with_variant_and_contrast(&self, variant: ThemeVariant, contrast_multiplier: f32) -> AppTheme {
        self.with_variant(variant).with_contrast_boost(contrast_multiplier)
    }

    /// Apply a color filter to the theme for real-time color transformation
    ///
    /// # Arguments
    /// * `filter` - The color filter to apply
    ///
    /// # Examples
    /// ```
    /// let theme = ThemePresets::dark();
    /// let grayscale = theme.with_color_filter(ColorFilter::Grayscale);
    /// let sepia = theme.with_color_filter(ColorFilter::Sepia);
    /// let blue_light = theme.with_color_filter(ColorFilter::BlueLightFilter(0.7));
    /// ```
    pub fn with_color_filter(&self, filter: ColorFilter) -> AppTheme {
        if matches!(filter, ColorFilter::None) {
            return self.clone();
        }

        let mut theme = self.clone();

        // Update name and description to reflect filter
        theme.name = format!("{} ({})", self.name, filter.name());
        theme.description = format!("{} - {}", self.description, filter.description());

        // Apply filter to all colors
        theme.window_border = filter.apply(theme.window_border);
        theme.window_border_focused = filter.apply(theme.window_border_focused);
        theme.window_background = filter.apply(theme.window_background);
        theme.window_title = filter.apply(theme.window_title);

        theme.text_primary = filter.apply(theme.text_primary);
        theme.text_secondary = filter.apply(theme.text_secondary);
        theme.text_disabled = filter.apply(theme.text_disabled);
        theme.text_selected = filter.apply(theme.text_selected);

        theme.background_primary = filter.apply(theme.background_primary);
        theme.background_secondary = filter.apply(theme.background_secondary);
        theme.background_selected = filter.apply(theme.background_selected);
        theme.background_hover = filter.apply(theme.background_hover);

        theme.editor_border = filter.apply(theme.editor_border);
        theme.editor_label = filter.apply(theme.editor_label);
        theme.editor_label_focused = filter.apply(theme.editor_label_focused);
        theme.editor_text = filter.apply(theme.editor_text);
        theme.editor_cursor = filter.apply(theme.editor_cursor);
        theme.editor_status = filter.apply(theme.editor_status);
        theme.editor_background = filter.apply(theme.editor_background);

        theme.browser_border = filter.apply(theme.browser_border);
        theme.browser_title = filter.apply(theme.browser_title);
        theme.browser_item_normal = filter.apply(theme.browser_item_normal);
        theme.browser_item_selected = filter.apply(theme.browser_item_selected);
        theme.browser_item_focused = filter.apply(theme.browser_item_focused);
        theme.browser_background = filter.apply(theme.browser_background);
        theme.browser_scrollbar = filter.apply(theme.browser_scrollbar);

        theme.form_border = filter.apply(theme.form_border);
        theme.form_label = filter.apply(theme.form_label);
        theme.form_label_focused = filter.apply(theme.form_label_focused);
        theme.form_field_background = filter.apply(theme.form_field_background);
        theme.form_field_text = filter.apply(theme.form_field_text);
        theme.form_checkbox_checked = filter.apply(theme.form_checkbox_checked);
        theme.form_checkbox_unchecked = filter.apply(theme.form_checkbox_unchecked);
        theme.form_error = filter.apply(theme.form_error);

        theme.menu_border = filter.apply(theme.menu_border);
        theme.menu_background = filter.apply(theme.menu_background);
        theme.menu_item_normal = filter.apply(theme.menu_item_normal);
        theme.menu_item_selected = filter.apply(theme.menu_item_selected);
        theme.menu_item_focused = filter.apply(theme.menu_item_focused);
        theme.menu_separator = filter.apply(theme.menu_separator);

        theme.status_info = filter.apply(theme.status_info);
        theme.status_success = filter.apply(theme.status_success);
        theme.status_warning = filter.apply(theme.status_warning);
        theme.status_error = filter.apply(theme.status_error);
        theme.status_background = filter.apply(theme.status_background);

        theme.button_normal = filter.apply(theme.button_normal);
        theme.button_hover = filter.apply(theme.button_hover);
        theme.button_active = filter.apply(theme.button_active);
        theme.button_disabled = filter.apply(theme.button_disabled);

        theme.command_echo = filter.apply(theme.command_echo);
        theme.selection_background = filter.apply(theme.selection_background);
        theme.link_color = filter.apply(theme.link_color);
        theme.speech_color = filter.apply(theme.speech_color);
        theme.whisper_color = filter.apply(theme.whisper_color);
        theme.thought_color = filter.apply(theme.thought_color);

        theme.injury_default_color = filter.apply(theme.injury_default_color);

        theme
    }

    /// Apply all transformations (variant, contrast, and filter) in one operation
    ///
    /// This is the most comprehensive theme transformation method
    ///
    /// # Examples
    /// ```
    /// let theme = ThemePresets::ocean_depths();
    /// let fully_adjusted = theme.with_all_transformations(
    ///     ThemeVariant::HighContrast,
    ///     1.5,
    ///     ColorFilter::BlueLightFilter(0.6)
    /// );
    /// ```
    pub fn with_all_transformations(
        &self,
        variant: ThemeVariant,
        contrast_multiplier: f32,
        filter: ColorFilter,
    ) -> AppTheme {
        self.with_variant(variant)
            .with_contrast_boost(contrast_multiplier)
            .with_color_filter(filter)
    }
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
    pub section_header_color: Color,
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

        // New general-purpose themes
        themes.insert("ocean-depths".to_string(), Self::ocean_depths());
        themes.insert("forest-canopy".to_string(), Self::forest_canopy());
        themes.insert("sunset-boulevard".to_string(), Self::sunset_boulevard());
        themes.insert("arctic-night".to_string(), Self::arctic_night());
        themes.insert("cyberpunk-neon".to_string(), Self::cyberpunk_neon());
        themes.insert("sepia-parchment".to_string(), Self::sepia_parchment());
        themes.insert("lavender-dreams".to_string(), Self::lavender_dreams());
        themes.insert("cherry-blossom".to_string(), Self::cherry_blossom());
        themes.insert("slate-professional".to_string(), Self::slate_professional());
        themes.insert("autumn-harvest".to_string(), Self::autumn_harvest());

        // Accessibility themes
        themes.insert("high-contrast-light".to_string(), Self::high_contrast_light());
        themes.insert("high-contrast-dark".to_string(), Self::high_contrast_dark());
        themes.insert("deuteranopia".to_string(), Self::deuteranopia_friendly());
        themes.insert("protanopia".to_string(), Self::protanopia_friendly());
        themes.insert("tritanopia".to_string(), Self::tritanopia_friendly());
        themes.insert("monochrome".to_string(), Self::monochrome());
        themes.insert("low-blue-light".to_string(), Self::low_blue_light());
        themes.insert("photophobia".to_string(), Self::photophobia_friendly());
        themes.insert("adhd-focus".to_string(), Self::adhd_focus());
        themes.insert("reduced-motion".to_string(), Self::reduced_motion());

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

    /// Ocean Depths - Deep ocean blues with teal and aqua accents
    pub fn ocean_depths() -> AppTheme {
        let mut theme = AppTheme {
            name: "Ocean Depths".to_string(),
            description: "Deep ocean blues with teal and aqua accents".to_string(),

            window_border: Color::Rgb(30, 77, 107),
            window_border_focused: Color::Rgb(0, 188, 212),
            window_background: Color::Rgb(10, 22, 40),
            window_title: Color::Rgb(224, 242, 247),

            text_primary: Color::Rgb(224, 242, 247),
            text_secondary: Color::Rgb(144, 202, 249),
            text_disabled: Color::Rgb(62, 109, 143),
            text_selected: Color::Rgb(0, 188, 212),

            background_primary: Color::Rgb(10, 22, 40),
            background_secondary: Color::Rgb(13, 31, 51),
            background_selected: Color::Rgb(30, 77, 107),
            background_hover: Color::Rgb(20, 45, 70),

            editor_border: Color::Rgb(30, 77, 107),
            editor_label: Color::Rgb(129, 212, 250),
            editor_label_focused: Color::Rgb(0, 188, 212),
            editor_text: Color::Rgb(179, 229, 252),
            editor_cursor: Color::Rgb(0, 188, 212),
            editor_status: Color::Rgb(77, 208, 225),
            editor_background: Color::Rgb(13, 31, 51),

            browser_border: Color::Rgb(30, 77, 107),
            browser_title: Color::Rgb(224, 242, 247),
            browser_item_normal: Color::Rgb(179, 229, 252),
            browser_item_selected: Color::Rgb(10, 22, 40),
            browser_item_focused: Color::Rgb(0, 188, 212),
            browser_background: Color::Rgb(13, 31, 51),
            browser_scrollbar: Color::Rgb(0, 188, 212),

            form_border: Color::Rgb(30, 77, 107),
            form_label: Color::Rgb(129, 212, 250),
            form_label_focused: Color::Rgb(0, 188, 212),
            form_field_background: Color::Rgb(13, 31, 51),
            form_field_text: Color::Rgb(179, 229, 252),
            form_checkbox_checked: Color::Rgb(77, 208, 225),
            form_checkbox_unchecked: Color::Rgb(62, 109, 143),
            form_error: Color::Rgb(239, 83, 80),

            menu_border: Color::Rgb(30, 77, 107),
            menu_background: Color::Rgb(10, 22, 40),
            menu_item_normal: Color::Rgb(179, 229, 252),
            menu_item_selected: Color::Rgb(13, 31, 51),
            menu_item_focused: Color::Rgb(0, 188, 212),
            menu_separator: Color::Rgb(30, 77, 107),

            status_info: Color::Rgb(0, 188, 212),
            status_success: Color::Rgb(77, 208, 225),
            status_warning: Color::Rgb(255, 167, 38),
            status_error: Color::Rgb(239, 83, 80),
            status_background: Color::Rgb(10, 22, 40),

            button_normal: Color::Rgb(0, 188, 212),
            button_hover: Color::Rgb(38, 198, 218),
            button_active: Color::Rgb(77, 208, 225),
            button_disabled: Color::Rgb(62, 109, 143),

            command_echo: Color::Rgb(224, 242, 247),
            selection_background: Color::Rgb(30, 77, 107),
            link_color: Color::Rgb(38, 198, 218),
            speech_color: Color::Rgb(77, 208, 225),
            whisper_color: Color::Rgb(144, 202, 249),
            thought_color: Color::Rgb(179, 229, 252),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Forest Canopy - Earthy greens with warm brown accents
    pub fn forest_canopy() -> AppTheme {
        let mut theme = AppTheme {
            name: "Forest Canopy".to_string(),
            description: "Earthy greens with warm brown accents".to_string(),

            window_border: Color::Rgb(46, 125, 50),
            window_border_focused: Color::Rgb(102, 187, 106),
            window_background: Color::Rgb(26, 38, 23),
            window_title: Color::Rgb(232, 245, 233),

            text_primary: Color::Rgb(232, 245, 233),
            text_secondary: Color::Rgb(200, 230, 201),
            text_disabled: Color::Rgb(76, 100, 78),
            text_selected: Color::Rgb(102, 187, 106),

            background_primary: Color::Rgb(26, 38, 23),
            background_secondary: Color::Rgb(27, 46, 26),
            background_selected: Color::Rgb(46, 125, 50),
            background_hover: Color::Rgb(35, 60, 35),

            editor_border: Color::Rgb(46, 125, 50),
            editor_label: Color::Rgb(174, 213, 129),
            editor_label_focused: Color::Rgb(102, 187, 106),
            editor_text: Color::Rgb(197, 225, 165),
            editor_cursor: Color::Rgb(102, 187, 106),
            editor_status: Color::Rgb(129, 199, 132),
            editor_background: Color::Rgb(27, 46, 26),

            browser_border: Color::Rgb(46, 125, 50),
            browser_title: Color::Rgb(232, 245, 233),
            browser_item_normal: Color::Rgb(197, 225, 165),
            browser_item_selected: Color::Rgb(26, 38, 23),
            browser_item_focused: Color::Rgb(102, 187, 106),
            browser_background: Color::Rgb(27, 46, 26),
            browser_scrollbar: Color::Rgb(102, 187, 106),

            form_border: Color::Rgb(46, 125, 50),
            form_label: Color::Rgb(174, 213, 129),
            form_label_focused: Color::Rgb(102, 187, 106),
            form_field_background: Color::Rgb(27, 46, 26),
            form_field_text: Color::Rgb(197, 225, 165),
            form_checkbox_checked: Color::Rgb(76, 175, 80),
            form_checkbox_unchecked: Color::Rgb(76, 100, 78),
            form_error: Color::Rgb(244, 67, 54),

            menu_border: Color::Rgb(46, 125, 50),
            menu_background: Color::Rgb(26, 38, 23),
            menu_item_normal: Color::Rgb(197, 225, 165),
            menu_item_selected: Color::Rgb(27, 46, 26),
            menu_item_focused: Color::Rgb(102, 187, 106),
            menu_separator: Color::Rgb(46, 125, 50),

            status_info: Color::Rgb(102, 187, 106),
            status_success: Color::Rgb(76, 175, 80),
            status_warning: Color::Rgb(255, 183, 77),
            status_error: Color::Rgb(244, 67, 54),
            status_background: Color::Rgb(26, 38, 23),

            button_normal: Color::Rgb(102, 187, 106),
            button_hover: Color::Rgb(129, 199, 132),
            button_active: Color::Rgb(76, 175, 80),
            button_disabled: Color::Rgb(76, 100, 78),

            command_echo: Color::Rgb(232, 245, 233),
            selection_background: Color::Rgb(46, 125, 50),
            link_color: Color::Rgb(129, 199, 132),
            speech_color: Color::Rgb(174, 213, 129),
            whisper_color: Color::Rgb(200, 230, 201),
            thought_color: Color::Rgb(197, 225, 165),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Sunset Boulevard - Warm sunset colors from dusk to twilight
    pub fn sunset_boulevard() -> AppTheme {
        let mut theme = AppTheme {
            name: "Sunset Boulevard".to_string(),
            description: "Warm sunset colors from dusk to twilight".to_string(),

            window_border: Color::Rgb(142, 36, 170),
            window_border_focused: Color::Rgb(255, 111, 0),
            window_background: Color::Rgb(45, 27, 46),
            window_title: Color::Rgb(255, 243, 224),

            text_primary: Color::Rgb(255, 243, 224),
            text_secondary: Color::Rgb(255, 204, 188),
            text_disabled: Color::Rgb(100, 70, 100),
            text_selected: Color::Rgb(255, 111, 0),

            background_primary: Color::Rgb(45, 27, 46),
            background_secondary: Color::Rgb(58, 36, 64),
            background_selected: Color::Rgb(142, 36, 170),
            background_hover: Color::Rgb(70, 45, 75),

            editor_border: Color::Rgb(142, 36, 170),
            editor_label: Color::Rgb(255, 204, 128),
            editor_label_focused: Color::Rgb(255, 111, 0),
            editor_text: Color::Rgb(255, 171, 145),
            editor_cursor: Color::Rgb(255, 111, 0),
            editor_status: Color::Rgb(255, 167, 38),
            editor_background: Color::Rgb(58, 36, 64),

            browser_border: Color::Rgb(142, 36, 170),
            browser_title: Color::Rgb(255, 243, 224),
            browser_item_normal: Color::Rgb(255, 171, 145),
            browser_item_selected: Color::Rgb(45, 27, 46),
            browser_item_focused: Color::Rgb(255, 111, 0),
            browser_background: Color::Rgb(58, 36, 64),
            browser_scrollbar: Color::Rgb(255, 111, 0),

            form_border: Color::Rgb(142, 36, 170),
            form_label: Color::Rgb(255, 204, 128),
            form_label_focused: Color::Rgb(255, 111, 0),
            form_field_background: Color::Rgb(58, 36, 64),
            form_field_text: Color::Rgb(255, 171, 145),
            form_checkbox_checked: Color::Rgb(255, 167, 38),
            form_checkbox_unchecked: Color::Rgb(100, 70, 100),
            form_error: Color::Rgb(240, 98, 146),

            menu_border: Color::Rgb(142, 36, 170),
            menu_background: Color::Rgb(45, 27, 46),
            menu_item_normal: Color::Rgb(255, 171, 145),
            menu_item_selected: Color::Rgb(58, 36, 64),
            menu_item_focused: Color::Rgb(255, 111, 0),
            menu_separator: Color::Rgb(142, 36, 170),

            status_info: Color::Rgb(255, 111, 0),
            status_success: Color::Rgb(255, 167, 38),
            status_warning: Color::Rgb(255, 213, 79),
            status_error: Color::Rgb(240, 98, 146),
            status_background: Color::Rgb(45, 27, 46),

            button_normal: Color::Rgb(255, 111, 0),
            button_hover: Color::Rgb(255, 145, 0),
            button_active: Color::Rgb(255, 167, 38),
            button_disabled: Color::Rgb(100, 70, 100),

            command_echo: Color::Rgb(255, 243, 224),
            selection_background: Color::Rgb(142, 36, 170),
            link_color: Color::Rgb(255, 145, 0),
            speech_color: Color::Rgb(255, 204, 128),
            whisper_color: Color::Rgb(255, 204, 188),
            thought_color: Color::Rgb(240, 98, 146),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Arctic Night - Crisp arctic colors with ice blue highlights
    pub fn arctic_night() -> AppTheme {
        let mut theme = AppTheme {
            name: "Arctic Night".to_string(),
            description: "Crisp arctic colors with ice blue highlights".to_string(),

            window_border: Color::Rgb(52, 73, 85),
            window_border_focused: Color::Rgb(100, 181, 246),
            window_background: Color::Rgb(13, 24, 33),
            window_title: Color::Rgb(240, 244, 248),

            text_primary: Color::Rgb(240, 244, 248),
            text_secondary: Color::Rgb(207, 216, 220),
            text_disabled: Color::Rgb(84, 110, 122),
            text_selected: Color::Rgb(100, 181, 246),

            background_primary: Color::Rgb(13, 24, 33),
            background_secondary: Color::Rgb(26, 37, 47),
            background_selected: Color::Rgb(52, 73, 85),
            background_hover: Color::Rgb(35, 50, 60),

            editor_border: Color::Rgb(52, 73, 85),
            editor_label: Color::Rgb(144, 164, 174),
            editor_label_focused: Color::Rgb(100, 181, 246),
            editor_text: Color::Rgb(176, 190, 197),
            editor_cursor: Color::Rgb(100, 181, 246),
            editor_status: Color::Rgb(77, 208, 225),
            editor_background: Color::Rgb(26, 37, 47),

            browser_border: Color::Rgb(52, 73, 85),
            browser_title: Color::Rgb(240, 244, 248),
            browser_item_normal: Color::Rgb(176, 190, 197),
            browser_item_selected: Color::Rgb(13, 24, 33),
            browser_item_focused: Color::Rgb(100, 181, 246),
            browser_background: Color::Rgb(26, 37, 47),
            browser_scrollbar: Color::Rgb(100, 181, 246),

            form_border: Color::Rgb(52, 73, 85),
            form_label: Color::Rgb(144, 164, 174),
            form_label_focused: Color::Rgb(100, 181, 246),
            form_field_background: Color::Rgb(26, 37, 47),
            form_field_text: Color::Rgb(176, 190, 197),
            form_checkbox_checked: Color::Rgb(77, 208, 225),
            form_checkbox_unchecked: Color::Rgb(84, 110, 122),
            form_error: Color::Rgb(255, 82, 82),

            menu_border: Color::Rgb(52, 73, 85),
            menu_background: Color::Rgb(13, 24, 33),
            menu_item_normal: Color::Rgb(176, 190, 197),
            menu_item_selected: Color::Rgb(26, 37, 47),
            menu_item_focused: Color::Rgb(100, 181, 246),
            menu_separator: Color::Rgb(52, 73, 85),

            status_info: Color::Rgb(100, 181, 246),
            status_success: Color::Rgb(77, 208, 225),
            status_warning: Color::Rgb(255, 171, 64),
            status_error: Color::Rgb(255, 82, 82),
            status_background: Color::Rgb(13, 24, 33),

            button_normal: Color::Rgb(100, 181, 246),
            button_hover: Color::Rgb(79, 195, 247),
            button_active: Color::Rgb(77, 208, 225),
            button_disabled: Color::Rgb(84, 110, 122),

            command_echo: Color::Rgb(240, 244, 248),
            selection_background: Color::Rgb(52, 73, 85),
            link_color: Color::Rgb(79, 195, 247),
            speech_color: Color::Rgb(144, 164, 174),
            whisper_color: Color::Rgb(207, 216, 220),
            thought_color: Color::Rgb(176, 190, 197),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Cyberpunk Neon - Vibrant neon colors on deep black background
    pub fn cyberpunk_neon() -> AppTheme {
        let mut theme = AppTheme {
            name: "Cyberpunk Neon".to_string(),
            description: "Vibrant neon colors on deep black background".to_string(),

            window_border: Color::Rgb(0, 255, 255),
            window_border_focused: Color::Rgb(255, 0, 110),
            window_background: Color::Rgb(10, 10, 10),
            window_title: Color::Rgb(0, 255, 159),

            text_primary: Color::Rgb(0, 255, 159),
            text_secondary: Color::Rgb(255, 0, 255),
            text_disabled: Color::Rgb(60, 60, 60),
            text_selected: Color::Rgb(255, 0, 110),

            background_primary: Color::Rgb(10, 10, 10),
            background_secondary: Color::Rgb(15, 15, 15),
            background_selected: Color::Rgb(40, 0, 40),
            background_hover: Color::Rgb(25, 25, 25),

            editor_border: Color::Rgb(0, 255, 255),
            editor_label: Color::Rgb(255, 0, 255),
            editor_label_focused: Color::Rgb(255, 0, 110),
            editor_text: Color::Rgb(0, 255, 159),
            editor_cursor: Color::Rgb(255, 0, 110),
            editor_status: Color::Rgb(0, 245, 255),
            editor_background: Color::Rgb(15, 15, 15),

            browser_border: Color::Rgb(0, 255, 255),
            browser_title: Color::Rgb(0, 255, 159),
            browser_item_normal: Color::Rgb(0, 255, 159),
            browser_item_selected: Color::Rgb(10, 10, 10),
            browser_item_focused: Color::Rgb(255, 0, 110),
            browser_background: Color::Rgb(15, 15, 15),
            browser_scrollbar: Color::Rgb(0, 255, 255),

            form_border: Color::Rgb(0, 255, 255),
            form_label: Color::Rgb(255, 0, 255),
            form_label_focused: Color::Rgb(255, 0, 110),
            form_field_background: Color::Rgb(15, 15, 15),
            form_field_text: Color::Rgb(0, 255, 159),
            form_checkbox_checked: Color::Rgb(57, 255, 20),
            form_checkbox_unchecked: Color::Rgb(60, 60, 60),
            form_error: Color::Rgb(255, 7, 58),

            menu_border: Color::Rgb(0, 255, 255),
            menu_background: Color::Rgb(10, 10, 10),
            menu_item_normal: Color::Rgb(0, 255, 159),
            menu_item_selected: Color::Rgb(15, 15, 15),
            menu_item_focused: Color::Rgb(255, 0, 110),
            menu_separator: Color::Rgb(0, 255, 255),

            status_info: Color::Rgb(0, 255, 255),
            status_success: Color::Rgb(57, 255, 20),
            status_warning: Color::Rgb(255, 255, 0),
            status_error: Color::Rgb(255, 7, 58),
            status_background: Color::Rgb(10, 10, 10),

            button_normal: Color::Rgb(0, 255, 255),
            button_hover: Color::Rgb(255, 0, 110),
            button_active: Color::Rgb(57, 255, 20),
            button_disabled: Color::Rgb(60, 60, 60),

            command_echo: Color::Rgb(0, 255, 159),
            selection_background: Color::Rgb(40, 0, 40),
            link_color: Color::Rgb(0, 245, 255),
            speech_color: Color::Rgb(255, 0, 255),
            whisper_color: Color::Rgb(0, 255, 255),
            thought_color: Color::Rgb(255, 0, 110),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Sepia Parchment - Warm vintage sepia tones for a classic look
    pub fn sepia_parchment() -> AppTheme {
        let mut theme = AppTheme {
            name: "Sepia Parchment".to_string(),
            description: "Warm vintage sepia tones for a classic look".to_string(),

            window_border: Color::Rgb(139, 115, 85),
            window_border_focused: Color::Rgb(218, 165, 32),
            window_background: Color::Rgb(43, 36, 25),
            window_title: Color::Rgb(244, 232, 208),

            text_primary: Color::Rgb(244, 232, 208),
            text_secondary: Color::Rgb(212, 197, 169),
            text_disabled: Color::Rgb(100, 85, 70),
            text_selected: Color::Rgb(218, 165, 32),

            background_primary: Color::Rgb(43, 36, 25),
            background_secondary: Color::Rgb(58, 47, 35),
            background_selected: Color::Rgb(139, 115, 85),
            background_hover: Color::Rgb(70, 60, 45),

            editor_border: Color::Rgb(139, 115, 85),
            editor_label: Color::Rgb(210, 180, 140),
            editor_label_focused: Color::Rgb(218, 165, 32),
            editor_text: Color::Rgb(232, 213, 176),
            editor_cursor: Color::Rgb(218, 165, 32),
            editor_status: Color::Rgb(184, 134, 11),
            editor_background: Color::Rgb(58, 47, 35),

            browser_border: Color::Rgb(139, 115, 85),
            browser_title: Color::Rgb(244, 232, 208),
            browser_item_normal: Color::Rgb(232, 213, 176),
            browser_item_selected: Color::Rgb(43, 36, 25),
            browser_item_focused: Color::Rgb(218, 165, 32),
            browser_background: Color::Rgb(58, 47, 35),
            browser_scrollbar: Color::Rgb(218, 165, 32),

            form_border: Color::Rgb(139, 115, 85),
            form_label: Color::Rgb(210, 180, 140),
            form_label_focused: Color::Rgb(218, 165, 32),
            form_field_background: Color::Rgb(58, 47, 35),
            form_field_text: Color::Rgb(232, 213, 176),
            form_checkbox_checked: Color::Rgb(184, 134, 11),
            form_checkbox_unchecked: Color::Rgb(100, 85, 70),
            form_error: Color::Rgb(220, 20, 60),

            menu_border: Color::Rgb(139, 115, 85),
            menu_background: Color::Rgb(43, 36, 25),
            menu_item_normal: Color::Rgb(232, 213, 176),
            menu_item_selected: Color::Rgb(58, 47, 35),
            menu_item_focused: Color::Rgb(218, 165, 32),
            menu_separator: Color::Rgb(139, 115, 85),

            status_info: Color::Rgb(218, 165, 32),
            status_success: Color::Rgb(184, 134, 11),
            status_warning: Color::Rgb(255, 140, 0),
            status_error: Color::Rgb(220, 20, 60),
            status_background: Color::Rgb(43, 36, 25),

            button_normal: Color::Rgb(218, 165, 32),
            button_hover: Color::Rgb(205, 133, 63),
            button_active: Color::Rgb(184, 134, 11),
            button_disabled: Color::Rgb(100, 85, 70),

            command_echo: Color::Rgb(244, 232, 208),
            selection_background: Color::Rgb(139, 115, 85),
            link_color: Color::Rgb(205, 133, 63),
            speech_color: Color::Rgb(210, 180, 140),
            whisper_color: Color::Rgb(212, 197, 169),
            thought_color: Color::Rgb(232, 213, 176),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Lavender Dreams - Soft lavender tones for a calming experience
    pub fn lavender_dreams() -> AppTheme {
        let mut theme = AppTheme {
            name: "Lavender Dreams".to_string(),
            description: "Soft lavender tones for a calming experience".to_string(),

            window_border: Color::Rgb(123, 31, 162),
            window_border_focused: Color::Rgb(186, 104, 200),
            window_background: Color::Rgb(26, 22, 37),
            window_title: Color::Rgb(243, 229, 245),

            text_primary: Color::Rgb(243, 229, 245),
            text_secondary: Color::Rgb(225, 190, 231),
            text_disabled: Color::Rgb(80, 60, 90),
            text_selected: Color::Rgb(186, 104, 200),

            background_primary: Color::Rgb(26, 22, 37),
            background_secondary: Color::Rgb(37, 26, 46),
            background_selected: Color::Rgb(123, 31, 162),
            background_hover: Color::Rgb(50, 35, 60),

            editor_border: Color::Rgb(123, 31, 162),
            editor_label: Color::Rgb(186, 104, 200),
            editor_label_focused: Color::Rgb(206, 147, 216),
            editor_text: Color::Rgb(225, 190, 231),
            editor_cursor: Color::Rgb(186, 104, 200),
            editor_status: Color::Rgb(156, 39, 176),
            editor_background: Color::Rgb(37, 26, 46),

            browser_border: Color::Rgb(123, 31, 162),
            browser_title: Color::Rgb(243, 229, 245),
            browser_item_normal: Color::Rgb(206, 147, 216),
            browser_item_selected: Color::Rgb(26, 22, 37),
            browser_item_focused: Color::Rgb(186, 104, 200),
            browser_background: Color::Rgb(37, 26, 46),
            browser_scrollbar: Color::Rgb(186, 104, 200),

            form_border: Color::Rgb(123, 31, 162),
            form_label: Color::Rgb(186, 104, 200),
            form_label_focused: Color::Rgb(206, 147, 216),
            form_field_background: Color::Rgb(37, 26, 46),
            form_field_text: Color::Rgb(225, 190, 231),
            form_checkbox_checked: Color::Rgb(156, 39, 176),
            form_checkbox_unchecked: Color::Rgb(80, 60, 90),
            form_error: Color::Rgb(240, 98, 146),

            menu_border: Color::Rgb(123, 31, 162),
            menu_background: Color::Rgb(26, 22, 37),
            menu_item_normal: Color::Rgb(206, 147, 216),
            menu_item_selected: Color::Rgb(37, 26, 46),
            menu_item_focused: Color::Rgb(186, 104, 200),
            menu_separator: Color::Rgb(123, 31, 162),

            status_info: Color::Rgb(186, 104, 200),
            status_success: Color::Rgb(156, 39, 176),
            status_warning: Color::Rgb(255, 167, 38),
            status_error: Color::Rgb(240, 98, 146),
            status_background: Color::Rgb(26, 22, 37),

            button_normal: Color::Rgb(186, 104, 200),
            button_hover: Color::Rgb(206, 147, 216),
            button_active: Color::Rgb(156, 39, 176),
            button_disabled: Color::Rgb(80, 60, 90),

            command_echo: Color::Rgb(243, 229, 245),
            selection_background: Color::Rgb(123, 31, 162),
            link_color: Color::Rgb(171, 71, 188),
            speech_color: Color::Rgb(186, 104, 200),
            whisper_color: Color::Rgb(225, 190, 231),
            thought_color: Color::Rgb(206, 147, 216),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Cherry Blossom - Delicate pink tones inspired by spring blooms
    pub fn cherry_blossom() -> AppTheme {
        let mut theme = AppTheme {
            name: "Cherry Blossom".to_string(),
            description: "Delicate pink tones inspired by spring blooms".to_string(),

            window_border: Color::Rgb(194, 24, 91),
            window_border_focused: Color::Rgb(236, 64, 122),
            window_background: Color::Rgb(45, 26, 31),
            window_title: Color::Rgb(252, 228, 236),

            text_primary: Color::Rgb(252, 228, 236),
            text_secondary: Color::Rgb(248, 187, 208),
            text_disabled: Color::Rgb(90, 60, 70),
            text_selected: Color::Rgb(236, 64, 122),

            background_primary: Color::Rgb(45, 26, 31),
            background_secondary: Color::Rgb(58, 37, 46),
            background_selected: Color::Rgb(194, 24, 91),
            background_hover: Color::Rgb(70, 50, 60),

            editor_border: Color::Rgb(194, 24, 91),
            editor_label: Color::Rgb(240, 98, 146),
            editor_label_focused: Color::Rgb(236, 64, 122),
            editor_text: Color::Rgb(244, 143, 177),
            editor_cursor: Color::Rgb(236, 64, 122),
            editor_status: Color::Rgb(233, 30, 99),
            editor_background: Color::Rgb(58, 37, 46),

            browser_border: Color::Rgb(194, 24, 91),
            browser_title: Color::Rgb(252, 228, 236),
            browser_item_normal: Color::Rgb(244, 143, 177),
            browser_item_selected: Color::Rgb(45, 26, 31),
            browser_item_focused: Color::Rgb(236, 64, 122),
            browser_background: Color::Rgb(58, 37, 46),
            browser_scrollbar: Color::Rgb(236, 64, 122),

            form_border: Color::Rgb(194, 24, 91),
            form_label: Color::Rgb(240, 98, 146),
            form_label_focused: Color::Rgb(236, 64, 122),
            form_field_background: Color::Rgb(58, 37, 46),
            form_field_text: Color::Rgb(244, 143, 177),
            form_checkbox_checked: Color::Rgb(102, 187, 106),
            form_checkbox_unchecked: Color::Rgb(90, 60, 70),
            form_error: Color::Rgb(233, 30, 99),

            menu_border: Color::Rgb(194, 24, 91),
            menu_background: Color::Rgb(45, 26, 31),
            menu_item_normal: Color::Rgb(244, 143, 177),
            menu_item_selected: Color::Rgb(58, 37, 46),
            menu_item_focused: Color::Rgb(236, 64, 122),
            menu_separator: Color::Rgb(194, 24, 91),

            status_info: Color::Rgb(236, 64, 122),
            status_success: Color::Rgb(102, 187, 106),
            status_warning: Color::Rgb(255, 183, 77),
            status_error: Color::Rgb(233, 30, 99),
            status_background: Color::Rgb(45, 26, 31),

            button_normal: Color::Rgb(236, 64, 122),
            button_hover: Color::Rgb(240, 98, 146),
            button_active: Color::Rgb(233, 30, 99),
            button_disabled: Color::Rgb(90, 60, 70),

            command_echo: Color::Rgb(252, 228, 236),
            selection_background: Color::Rgb(194, 24, 91),
            link_color: Color::Rgb(240, 98, 146),
            speech_color: Color::Rgb(244, 143, 177),
            whisper_color: Color::Rgb(248, 187, 208),
            thought_color: Color::Rgb(236, 64, 122),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Slate Professional - Professional gray tones with blue accents
    pub fn slate_professional() -> AppTheme {
        let mut theme = AppTheme {
            name: "Slate Professional".to_string(),
            description: "Professional gray tones with blue accents".to_string(),

            window_border: Color::Rgb(76, 86, 106),
            window_border_focused: Color::Rgb(94, 129, 172),
            window_background: Color::Rgb(30, 35, 39),
            window_title: Color::Rgb(236, 239, 244),

            text_primary: Color::Rgb(236, 239, 244),
            text_secondary: Color::Rgb(216, 222, 233),
            text_disabled: Color::Rgb(106, 120, 140),
            text_selected: Color::Rgb(94, 129, 172),

            background_primary: Color::Rgb(30, 35, 39),
            background_secondary: Color::Rgb(46, 52, 64),
            background_selected: Color::Rgb(76, 86, 106),
            background_hover: Color::Rgb(59, 66, 82),

            editor_border: Color::Rgb(76, 86, 106),
            editor_label: Color::Rgb(129, 161, 193),
            editor_label_focused: Color::Rgb(94, 129, 172),
            editor_text: Color::Rgb(216, 222, 233),
            editor_cursor: Color::Rgb(94, 129, 172),
            editor_status: Color::Rgb(136, 192, 208),
            editor_background: Color::Rgb(46, 52, 64),

            browser_border: Color::Rgb(76, 86, 106),
            browser_title: Color::Rgb(236, 239, 244),
            browser_item_normal: Color::Rgb(216, 222, 233),
            browser_item_selected: Color::Rgb(30, 35, 39),
            browser_item_focused: Color::Rgb(94, 129, 172),
            browser_background: Color::Rgb(46, 52, 64),
            browser_scrollbar: Color::Rgb(94, 129, 172),

            form_border: Color::Rgb(76, 86, 106),
            form_label: Color::Rgb(129, 161, 193),
            form_label_focused: Color::Rgb(94, 129, 172),
            form_field_background: Color::Rgb(46, 52, 64),
            form_field_text: Color::Rgb(216, 222, 233),
            form_checkbox_checked: Color::Rgb(163, 190, 140),
            form_checkbox_unchecked: Color::Rgb(106, 120, 140),
            form_error: Color::Rgb(191, 97, 106),

            menu_border: Color::Rgb(76, 86, 106),
            menu_background: Color::Rgb(30, 35, 39),
            menu_item_normal: Color::Rgb(216, 222, 233),
            menu_item_selected: Color::Rgb(46, 52, 64),
            menu_item_focused: Color::Rgb(94, 129, 172),
            menu_separator: Color::Rgb(76, 86, 106),

            status_info: Color::Rgb(94, 129, 172),
            status_success: Color::Rgb(163, 190, 140),
            status_warning: Color::Rgb(235, 203, 139),
            status_error: Color::Rgb(191, 97, 106),
            status_background: Color::Rgb(30, 35, 39),

            button_normal: Color::Rgb(94, 129, 172),
            button_hover: Color::Rgb(136, 192, 208),
            button_active: Color::Rgb(163, 190, 140),
            button_disabled: Color::Rgb(106, 120, 140),

            command_echo: Color::Rgb(236, 239, 244),
            selection_background: Color::Rgb(76, 86, 106),
            link_color: Color::Rgb(136, 192, 208),
            speech_color: Color::Rgb(129, 161, 193),
            whisper_color: Color::Rgb(216, 222, 233),
            thought_color: Color::Rgb(163, 190, 140),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Autumn Harvest - Warm autumn tones with golden highlights
    pub fn autumn_harvest() -> AppTheme {
        let mut theme = AppTheme {
            name: "Autumn Harvest".to_string(),
            description: "Warm autumn tones with golden highlights".to_string(),

            window_border: Color::Rgb(191, 54, 12),
            window_border_focused: Color::Rgb(255, 111, 0),
            window_background: Color::Rgb(42, 24, 16),
            window_title: Color::Rgb(255, 248, 225),

            text_primary: Color::Rgb(255, 248, 225),
            text_secondary: Color::Rgb(255, 224, 178),
            text_disabled: Color::Rgb(100, 70, 50),
            text_selected: Color::Rgb(255, 111, 0),

            background_primary: Color::Rgb(42, 24, 16),
            background_secondary: Color::Rgb(62, 39, 35),
            background_selected: Color::Rgb(191, 54, 12),
            background_hover: Color::Rgb(80, 55, 45),

            editor_border: Color::Rgb(191, 54, 12),
            editor_label: Color::Rgb(255, 183, 77),
            editor_label_focused: Color::Rgb(255, 111, 0),
            editor_text: Color::Rgb(255, 204, 128),
            editor_cursor: Color::Rgb(255, 111, 0),
            editor_status: Color::Rgb(255, 193, 7),
            editor_background: Color::Rgb(62, 39, 35),

            browser_border: Color::Rgb(191, 54, 12),
            browser_title: Color::Rgb(255, 248, 225),
            browser_item_normal: Color::Rgb(255, 204, 128),
            browser_item_selected: Color::Rgb(42, 24, 16),
            browser_item_focused: Color::Rgb(255, 111, 0),
            browser_background: Color::Rgb(62, 39, 35),
            browser_scrollbar: Color::Rgb(255, 111, 0),

            form_border: Color::Rgb(191, 54, 12),
            form_label: Color::Rgb(255, 183, 77),
            form_label_focused: Color::Rgb(255, 111, 0),
            form_field_background: Color::Rgb(62, 39, 35),
            form_field_text: Color::Rgb(255, 204, 128),
            form_checkbox_checked: Color::Rgb(139, 195, 74),
            form_checkbox_unchecked: Color::Rgb(100, 70, 50),
            form_error: Color::Rgb(211, 47, 47),

            menu_border: Color::Rgb(191, 54, 12),
            menu_background: Color::Rgb(42, 24, 16),
            menu_item_normal: Color::Rgb(255, 204, 128),
            menu_item_selected: Color::Rgb(62, 39, 35),
            menu_item_focused: Color::Rgb(255, 111, 0),
            menu_separator: Color::Rgb(191, 54, 12),

            status_info: Color::Rgb(255, 111, 0),
            status_success: Color::Rgb(139, 195, 74),
            status_warning: Color::Rgb(255, 193, 7),
            status_error: Color::Rgb(211, 47, 47),
            status_background: Color::Rgb(42, 24, 16),

            button_normal: Color::Rgb(255, 111, 0),
            button_hover: Color::Rgb(255, 152, 0),
            button_active: Color::Rgb(255, 193, 7),
            button_disabled: Color::Rgb(100, 70, 50),

            command_echo: Color::Rgb(255, 248, 225),
            selection_background: Color::Rgb(191, 54, 12),
            link_color: Color::Rgb(255, 152, 0),
            speech_color: Color::Rgb(255, 183, 77),
            whisper_color: Color::Rgb(255, 224, 178),
            thought_color: Color::Rgb(255, 204, 128),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    // ==================== ACCESSIBILITY THEMES ====================
    // These themes are designed for users with specific accessibility needs
    // following WCAG 2.1 guidelines.

    /// High Contrast Light - WCAG AAA compliant (21:1 contrast ratio)
    pub fn high_contrast_light() -> AppTheme {
        let mut theme = AppTheme {
            name: "High Contrast Light".to_string(),
            description: "Maximum contrast on white background for low vision (WCAG AAA)".to_string(),

            window_border: Color::Rgb(0, 0, 0),
            window_border_focused: Color::Rgb(0, 0, 255),
            window_background: Color::Rgb(255, 255, 255),
            window_title: Color::Rgb(0, 0, 0),

            text_primary: Color::Rgb(0, 0, 0),
            text_secondary: Color::Rgb(26, 26, 26),
            text_disabled: Color::Rgb(128, 128, 128),
            text_selected: Color::Rgb(0, 0, 255),

            background_primary: Color::Rgb(255, 255, 255),
            background_secondary: Color::Rgb(245, 245, 245),
            background_selected: Color::Rgb(200, 200, 255),
            background_hover: Color::Rgb(230, 230, 230),

            editor_border: Color::Rgb(0, 0, 0),
            editor_label: Color::Rgb(0, 0, 0),
            editor_label_focused: Color::Rgb(0, 0, 255),
            editor_text: Color::Rgb(0, 0, 0),
            editor_cursor: Color::Rgb(0, 0, 255),
            editor_status: Color::Rgb(0, 100, 0),
            editor_background: Color::Rgb(255, 255, 255),

            browser_border: Color::Rgb(0, 0, 0),
            browser_title: Color::Rgb(0, 0, 0),
            browser_item_normal: Color::Rgb(0, 0, 0),
            browser_item_selected: Color::Rgb(255, 255, 255),
            browser_item_focused: Color::Rgb(0, 0, 255),
            browser_background: Color::Rgb(255, 255, 255),
            browser_scrollbar: Color::Rgb(0, 0, 0),

            form_border: Color::Rgb(0, 0, 0),
            form_label: Color::Rgb(0, 0, 0),
            form_label_focused: Color::Rgb(0, 0, 255),
            form_field_background: Color::Rgb(255, 255, 255),
            form_field_text: Color::Rgb(0, 0, 0),
            form_checkbox_checked: Color::Rgb(0, 100, 0),
            form_checkbox_unchecked: Color::Rgb(128, 128, 128),
            form_error: Color::Rgb(139, 0, 0),

            menu_border: Color::Rgb(0, 0, 0),
            menu_background: Color::Rgb(245, 245, 245),
            menu_item_normal: Color::Rgb(0, 0, 0),
            menu_item_selected: Color::Rgb(255, 255, 255),
            menu_item_focused: Color::Rgb(0, 0, 255),
            menu_separator: Color::Rgb(0, 0, 0),

            status_info: Color::Rgb(0, 0, 255),
            status_success: Color::Rgb(0, 100, 0),
            status_warning: Color::Rgb(255, 140, 0),
            status_error: Color::Rgb(139, 0, 0),
            status_background: Color::Rgb(255, 255, 255),

            button_normal: Color::Rgb(0, 0, 0),
            button_hover: Color::Rgb(0, 0, 255),
            button_active: Color::Rgb(0, 100, 0),
            button_disabled: Color::Rgb(128, 128, 128),

            command_echo: Color::Rgb(0, 0, 0),
            selection_background: Color::Rgb(200, 200, 255),
            link_color: Color::Rgb(0, 0, 238),
            speech_color: Color::Rgb(0, 100, 0),
            whisper_color: Color::Rgb(0, 0, 139),
            thought_color: Color::Rgb(139, 0, 139),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// High Contrast Dark - WCAG AAA compliant (21:1 contrast ratio)
    pub fn high_contrast_dark() -> AppTheme {
        let mut theme = AppTheme {
            name: "High Contrast Dark".to_string(),
            description: "Maximum contrast on black background for low vision (WCAG AAA)".to_string(),

            window_border: Color::Rgb(255, 255, 255),
            window_border_focused: Color::Rgb(255, 255, 0),
            window_background: Color::Rgb(0, 0, 0),
            window_title: Color::Rgb(255, 255, 255),

            text_primary: Color::Rgb(255, 255, 255),
            text_secondary: Color::Rgb(240, 240, 240),
            text_disabled: Color::Rgb(128, 128, 128),
            text_selected: Color::Rgb(255, 255, 0),

            background_primary: Color::Rgb(0, 0, 0),
            background_secondary: Color::Rgb(10, 10, 10),
            background_selected: Color::Rgb(64, 64, 0),
            background_hover: Color::Rgb(30, 30, 30),

            editor_border: Color::Rgb(255, 255, 255),
            editor_label: Color::Rgb(255, 255, 255),
            editor_label_focused: Color::Rgb(255, 255, 0),
            editor_text: Color::Rgb(255, 255, 255),
            editor_cursor: Color::Rgb(255, 255, 0),
            editor_status: Color::Rgb(0, 255, 0),
            editor_background: Color::Rgb(0, 0, 0),

            browser_border: Color::Rgb(255, 255, 255),
            browser_title: Color::Rgb(255, 255, 255),
            browser_item_normal: Color::Rgb(255, 255, 255),
            browser_item_selected: Color::Rgb(0, 0, 0),
            browser_item_focused: Color::Rgb(255, 255, 0),
            browser_background: Color::Rgb(0, 0, 0),
            browser_scrollbar: Color::Rgb(255, 255, 255),

            form_border: Color::Rgb(255, 255, 255),
            form_label: Color::Rgb(255, 255, 255),
            form_label_focused: Color::Rgb(255, 255, 0),
            form_field_background: Color::Rgb(0, 0, 0),
            form_field_text: Color::Rgb(255, 255, 255),
            form_checkbox_checked: Color::Rgb(0, 255, 0),
            form_checkbox_unchecked: Color::Rgb(128, 128, 128),
            form_error: Color::Rgb(255, 0, 0),

            menu_border: Color::Rgb(255, 255, 255),
            menu_background: Color::Rgb(10, 10, 10),
            menu_item_normal: Color::Rgb(255, 255, 255),
            menu_item_selected: Color::Rgb(0, 0, 0),
            menu_item_focused: Color::Rgb(255, 255, 0),
            menu_separator: Color::Rgb(255, 255, 255),

            status_info: Color::Rgb(0, 255, 255),
            status_success: Color::Rgb(0, 255, 0),
            status_warning: Color::Rgb(255, 165, 0),
            status_error: Color::Rgb(255, 0, 0),
            status_background: Color::Rgb(0, 0, 0),

            button_normal: Color::Rgb(255, 255, 255),
            button_hover: Color::Rgb(255, 255, 0),
            button_active: Color::Rgb(0, 255, 0),
            button_disabled: Color::Rgb(128, 128, 128),

            command_echo: Color::Rgb(255, 255, 255),
            selection_background: Color::Rgb(64, 64, 0),
            link_color: Color::Rgb(0, 255, 255),
            speech_color: Color::Rgb(0, 255, 0),
            whisper_color: Color::Rgb(30, 144, 255),
            thought_color: Color::Rgb(218, 112, 214),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Load custom themes from ~/.two-face/themes/ directory
    /// Deuteranopia Friendly - Optimized for red-green colorblindness (most common form)
    pub fn deuteranopia_friendly() -> AppTheme {
        let mut theme = AppTheme {
            name: "Deuteranopia Friendly".to_string(),
            description: "Optimized for deuteranopia (red-green colorblindness)".to_string(),

            window_border: Color::Rgb(91, 143, 201),
            window_border_focused: Color::Rgb(255, 215, 0),
            window_background: Color::Rgb(26, 26, 46),
            window_title: Color::Rgb(234, 234, 234),

            text_primary: Color::Rgb(234, 234, 234),
            text_secondary: Color::Rgb(197, 197, 197),
            text_disabled: Color::Rgb(106, 120, 140),
            text_selected: Color::Rgb(255, 215, 0),

            background_primary: Color::Rgb(26, 26, 46),
            background_secondary: Color::Rgb(37, 37, 64),
            background_selected: Color::Rgb(91, 143, 201),
            background_hover: Color::Rgb(50, 50, 70),

            editor_border: Color::Rgb(91, 143, 201),
            editor_label: Color::Rgb(135, 206, 235),
            editor_label_focused: Color::Rgb(255, 215, 0),
            editor_text: Color::Rgb(168, 216, 255),
            editor_cursor: Color::Rgb(255, 215, 0),
            editor_status: Color::Rgb(0, 191, 255),
            editor_background: Color::Rgb(37, 37, 64),

            browser_border: Color::Rgb(91, 143, 201),
            browser_title: Color::Rgb(234, 234, 234),
            browser_item_normal: Color::Rgb(168, 216, 255),
            browser_item_selected: Color::Rgb(26, 26, 46),
            browser_item_focused: Color::Rgb(255, 215, 0),
            browser_background: Color::Rgb(37, 37, 64),
            browser_scrollbar: Color::Rgb(91, 143, 201),

            form_border: Color::Rgb(91, 143, 201),
            form_label: Color::Rgb(135, 206, 235),
            form_label_focused: Color::Rgb(255, 215, 0),
            form_field_background: Color::Rgb(37, 37, 64),
            form_field_text: Color::Rgb(168, 216, 255),
            form_checkbox_checked: Color::Rgb(0, 191, 255),  // Blue instead of green
            form_checkbox_unchecked: Color::Rgb(106, 120, 140),
            form_error: Color::Rgb(255, 20, 147),  // Pink instead of red

            menu_border: Color::Rgb(91, 143, 201),
            menu_background: Color::Rgb(26, 26, 46),
            menu_item_normal: Color::Rgb(168, 216, 255),
            menu_item_selected: Color::Rgb(37, 37, 64),
            menu_item_focused: Color::Rgb(255, 215, 0),
            menu_separator: Color::Rgb(91, 143, 201),

            status_info: Color::Rgb(0, 191, 255),
            status_success: Color::Rgb(0, 191, 255),  // Blue instead of green
            status_warning: Color::Rgb(255, 165, 0),
            status_error: Color::Rgb(255, 20, 147),  // Pink instead of red
            status_background: Color::Rgb(26, 26, 46),

            button_normal: Color::Rgb(91, 143, 201),
            button_hover: Color::Rgb(77, 166, 255),
            button_active: Color::Rgb(0, 191, 255),
            button_disabled: Color::Rgb(106, 120, 140),

            command_echo: Color::Rgb(234, 234, 234),
            selection_background: Color::Rgb(91, 143, 201),
            link_color: Color::Rgb(77, 166, 255),
            speech_color: Color::Rgb(135, 206, 235),
            whisper_color: Color::Rgb(168, 216, 255),
            thought_color: Color::Rgb(255, 215, 0),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Protanopia Friendly - Optimized for another form of red-green colorblindness
    pub fn protanopia_friendly() -> AppTheme {
        let mut theme = AppTheme {
            name: "Protanopia Friendly".to_string(),
            description: "Optimized for protanopia (red-green colorblindness variant)".to_string(),

            window_border: Color::Rgb(100, 149, 237),
            window_border_focused: Color::Rgb(255, 204, 0),
            window_background: Color::Rgb(31, 31, 31),
            window_title: Color::Rgb(224, 224, 224),

            text_primary: Color::Rgb(224, 224, 224),
            text_secondary: Color::Rgb(176, 176, 176),
            text_disabled: Color::Rgb(96, 96, 96),
            text_selected: Color::Rgb(255, 204, 0),

            background_primary: Color::Rgb(31, 31, 31),
            background_secondary: Color::Rgb(42, 42, 42),
            background_selected: Color::Rgb(100, 149, 237),
            background_hover: Color::Rgb(55, 55, 55),

            editor_border: Color::Rgb(100, 149, 237),
            editor_label: Color::Rgb(135, 206, 235),
            editor_label_focused: Color::Rgb(255, 204, 0),
            editor_text: Color::Rgb(173, 216, 230),
            editor_cursor: Color::Rgb(255, 204, 0),
            editor_status: Color::Rgb(0, 206, 209),
            editor_background: Color::Rgb(42, 42, 42),

            browser_border: Color::Rgb(100, 149, 237),
            browser_title: Color::Rgb(224, 224, 224),
            browser_item_normal: Color::Rgb(173, 216, 230),
            browser_item_selected: Color::Rgb(31, 31, 31),
            browser_item_focused: Color::Rgb(255, 204, 0),
            browser_background: Color::Rgb(42, 42, 42),
            browser_scrollbar: Color::Rgb(100, 149, 237),

            form_border: Color::Rgb(100, 149, 237),
            form_label: Color::Rgb(135, 206, 235),
            form_label_focused: Color::Rgb(255, 204, 0),
            form_field_background: Color::Rgb(42, 42, 42),
            form_field_text: Color::Rgb(173, 216, 230),
            form_checkbox_checked: Color::Rgb(0, 206, 209),  // Turquoise instead of green
            form_checkbox_unchecked: Color::Rgb(96, 96, 96),
            form_error: Color::Rgb(218, 112, 214),  // Orchid instead of red

            menu_border: Color::Rgb(100, 149, 237),
            menu_background: Color::Rgb(31, 31, 31),
            menu_item_normal: Color::Rgb(173, 216, 230),
            menu_item_selected: Color::Rgb(42, 42, 42),
            menu_item_focused: Color::Rgb(255, 204, 0),
            menu_separator: Color::Rgb(100, 149, 237),

            status_info: Color::Rgb(30, 144, 255),
            status_success: Color::Rgb(0, 206, 209),  // Turquoise instead of green
            status_warning: Color::Rgb(255, 140, 0),
            status_error: Color::Rgb(218, 112, 214),  // Orchid instead of red
            status_background: Color::Rgb(31, 31, 31),

            button_normal: Color::Rgb(100, 149, 237),
            button_hover: Color::Rgb(135, 206, 235),
            button_active: Color::Rgb(0, 206, 209),
            button_disabled: Color::Rgb(96, 96, 96),

            command_echo: Color::Rgb(224, 224, 224),
            selection_background: Color::Rgb(100, 149, 237),
            link_color: Color::Rgb(30, 144, 255),
            speech_color: Color::Rgb(135, 206, 235),
            whisper_color: Color::Rgb(173, 216, 230),
            thought_color: Color::Rgb(218, 112, 214),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Tritanopia Friendly - Optimized for blue-yellow colorblindness
    pub fn tritanopia_friendly() -> AppTheme {
        let mut theme = AppTheme {
            name: "Tritanopia Friendly".to_string(),
            description: "Optimized for tritanopia (blue-yellow colorblindness)".to_string(),

            window_border: Color::Rgb(255, 20, 147),
            window_border_focused: Color::Rgb(0, 255, 127),
            window_background: Color::Rgb(26, 26, 26),
            window_title: Color::Rgb(255, 255, 255),

            text_primary: Color::Rgb(255, 255, 255),
            text_secondary: Color::Rgb(204, 204, 204),
            text_disabled: Color::Rgb(100, 100, 100),
            text_selected: Color::Rgb(0, 255, 127),

            background_primary: Color::Rgb(26, 26, 26),
            background_secondary: Color::Rgb(37, 37, 37),
            background_selected: Color::Rgb(255, 20, 147),
            background_hover: Color::Rgb(50, 50, 50),

            editor_border: Color::Rgb(255, 20, 147),
            editor_label: Color::Rgb(255, 105, 180),
            editor_label_focused: Color::Rgb(0, 255, 127),
            editor_text: Color::Rgb(152, 251, 152),
            editor_cursor: Color::Rgb(0, 255, 127),
            editor_status: Color::Rgb(0, 250, 154),
            editor_background: Color::Rgb(37, 37, 37),

            browser_border: Color::Rgb(255, 20, 147),
            browser_title: Color::Rgb(255, 255, 255),
            browser_item_normal: Color::Rgb(152, 251, 152),
            browser_item_selected: Color::Rgb(26, 26, 26),
            browser_item_focused: Color::Rgb(0, 255, 127),
            browser_background: Color::Rgb(37, 37, 37),
            browser_scrollbar: Color::Rgb(255, 20, 147),

            form_border: Color::Rgb(255, 20, 147),
            form_label: Color::Rgb(255, 105, 180),
            form_label_focused: Color::Rgb(0, 255, 127),
            form_field_background: Color::Rgb(37, 37, 37),
            form_field_text: Color::Rgb(152, 251, 152),
            form_checkbox_checked: Color::Rgb(0, 250, 154),
            form_checkbox_unchecked: Color::Rgb(100, 100, 100),
            form_error: Color::Rgb(220, 20, 60),

            menu_border: Color::Rgb(255, 20, 147),
            menu_background: Color::Rgb(26, 26, 26),
            menu_item_normal: Color::Rgb(152, 251, 152),
            menu_item_selected: Color::Rgb(37, 37, 37),
            menu_item_focused: Color::Rgb(0, 255, 127),
            menu_separator: Color::Rgb(255, 20, 147),

            status_info: Color::Rgb(255, 105, 180),
            status_success: Color::Rgb(0, 250, 154),
            status_warning: Color::Rgb(255, 20, 147),  // Pink instead of yellow
            status_error: Color::Rgb(220, 20, 60),
            status_background: Color::Rgb(26, 26, 26),

            button_normal: Color::Rgb(255, 20, 147),
            button_hover: Color::Rgb(0, 255, 127),
            button_active: Color::Rgb(0, 250, 154),
            button_disabled: Color::Rgb(100, 100, 100),

            command_echo: Color::Rgb(255, 255, 255),
            selection_background: Color::Rgb(255, 20, 147),
            link_color: Color::Rgb(255, 105, 180),
            speech_color: Color::Rgb(152, 251, 152),
            whisper_color: Color::Rgb(144, 238, 144),
            thought_color: Color::Rgb(255, 182, 193),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Monochrome - Pure grayscale for complete colorblindness (achromatopsia)
    pub fn monochrome() -> AppTheme {
        let mut theme = AppTheme {
            name: "Monochrome".to_string(),
            description: "Pure grayscale for achromatopsia or preference".to_string(),

            window_border: Color::Rgb(128, 128, 128),
            window_border_focused: Color::Rgb(255, 255, 255),
            window_background: Color::Rgb(26, 26, 26),
            window_title: Color::Rgb(240, 240, 240),

            text_primary: Color::Rgb(240, 240, 240),
            text_secondary: Color::Rgb(192, 192, 192),
            text_disabled: Color::Rgb(96, 96, 96),
            text_selected: Color::Rgb(255, 255, 255),

            background_primary: Color::Rgb(26, 26, 26),
            background_secondary: Color::Rgb(37, 37, 37),
            background_selected: Color::Rgb(128, 128, 128),
            background_hover: Color::Rgb(50, 50, 50),

            editor_border: Color::Rgb(128, 128, 128),
            editor_label: Color::Rgb(176, 176, 176),
            editor_label_focused: Color::Rgb(255, 255, 255),
            editor_text: Color::Rgb(200, 200, 200),
            editor_cursor: Color::Rgb(255, 255, 255),
            editor_status: Color::Rgb(176, 176, 176),
            editor_background: Color::Rgb(37, 37, 37),

            browser_border: Color::Rgb(128, 128, 128),
            browser_title: Color::Rgb(240, 240, 240),
            browser_item_normal: Color::Rgb(200, 200, 200),
            browser_item_selected: Color::Rgb(26, 26, 26),
            browser_item_focused: Color::Rgb(255, 255, 255),
            browser_background: Color::Rgb(37, 37, 37),
            browser_scrollbar: Color::Rgb(128, 128, 128),

            form_border: Color::Rgb(128, 128, 128),
            form_label: Color::Rgb(176, 176, 176),
            form_label_focused: Color::Rgb(255, 255, 255),
            form_field_background: Color::Rgb(37, 37, 37),
            form_field_text: Color::Rgb(200, 200, 200),
            form_checkbox_checked: Color::Rgb(176, 176, 176),
            form_checkbox_unchecked: Color::Rgb(96, 96, 96),
            form_error: Color::Rgb(96, 96, 96),

            menu_border: Color::Rgb(128, 128, 128),
            menu_background: Color::Rgb(26, 26, 26),
            menu_item_normal: Color::Rgb(200, 200, 200),
            menu_item_selected: Color::Rgb(37, 37, 37),
            menu_item_focused: Color::Rgb(255, 255, 255),
            menu_separator: Color::Rgb(112, 112, 112),

            status_info: Color::Rgb(176, 176, 176),
            status_success: Color::Rgb(176, 176, 176),
            status_warning: Color::Rgb(144, 144, 144),
            status_error: Color::Rgb(96, 96, 96),
            status_background: Color::Rgb(26, 26, 26),

            button_normal: Color::Rgb(176, 176, 176),
            button_hover: Color::Rgb(208, 208, 208),
            button_active: Color::Rgb(255, 255, 255),
            button_disabled: Color::Rgb(96, 96, 96),

            command_echo: Color::Rgb(240, 240, 240),
            selection_background: Color::Rgb(128, 128, 128),
            link_color: Color::Rgb(208, 208, 208),
            speech_color: Color::Rgb(176, 176, 176),
            whisper_color: Color::Rgb(192, 192, 192),
            thought_color: Color::Rgb(160, 160, 160),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Low Blue Light - Reduces blue light for evening use and photosensitivity
    pub fn low_blue_light() -> AppTheme {
        let mut theme = AppTheme {
            name: "Low Blue Light".to_string(),
            description: "Warm colors with minimal blue light for evening use".to_string(),

            window_border: Color::Rgb(139, 90, 60),
            window_border_focused: Color::Rgb(255, 179, 71),
            window_background: Color::Rgb(42, 24, 16),
            window_title: Color::Rgb(255, 215, 181),

            text_primary: Color::Rgb(255, 215, 181),
            text_secondary: Color::Rgb(232, 196, 160),
            text_disabled: Color::Rgb(100, 70, 50),
            text_selected: Color::Rgb(255, 179, 71),

            background_primary: Color::Rgb(42, 24, 16),
            background_secondary: Color::Rgb(58, 35, 24),
            background_selected: Color::Rgb(139, 90, 60),
            background_hover: Color::Rgb(70, 45, 30),

            editor_border: Color::Rgb(139, 90, 60),
            editor_label: Color::Rgb(232, 184, 136),
            editor_label_focused: Color::Rgb(255, 179, 71),
            editor_text: Color::Rgb(244, 212, 176),
            editor_cursor: Color::Rgb(255, 179, 71),
            editor_status: Color::Rgb(199, 165, 99),
            editor_background: Color::Rgb(58, 35, 24),

            browser_border: Color::Rgb(139, 90, 60),
            browser_title: Color::Rgb(255, 215, 181),
            browser_item_normal: Color::Rgb(244, 212, 176),
            browser_item_selected: Color::Rgb(42, 24, 16),
            browser_item_focused: Color::Rgb(255, 179, 71),
            browser_background: Color::Rgb(58, 35, 24),
            browser_scrollbar: Color::Rgb(255, 179, 71),

            form_border: Color::Rgb(139, 90, 60),
            form_label: Color::Rgb(232, 184, 136),
            form_label_focused: Color::Rgb(255, 179, 71),
            form_field_background: Color::Rgb(58, 35, 24),
            form_field_text: Color::Rgb(244, 212, 176),
            form_checkbox_checked: Color::Rgb(199, 165, 99),
            form_checkbox_unchecked: Color::Rgb(100, 70, 50),
            form_error: Color::Rgb(205, 92, 92),

            menu_border: Color::Rgb(139, 90, 60),
            menu_background: Color::Rgb(42, 24, 16),
            menu_item_normal: Color::Rgb(244, 212, 176),
            menu_item_selected: Color::Rgb(58, 35, 24),
            menu_item_focused: Color::Rgb(255, 179, 71),
            menu_separator: Color::Rgb(139, 90, 60),

            status_info: Color::Rgb(255, 179, 71),
            status_success: Color::Rgb(199, 165, 99),
            status_warning: Color::Rgb(255, 140, 66),
            status_error: Color::Rgb(205, 92, 92),
            status_background: Color::Rgb(42, 24, 16),

            button_normal: Color::Rgb(255, 179, 71),
            button_hover: Color::Rgb(255, 153, 102),
            button_active: Color::Rgb(199, 165, 99),
            button_disabled: Color::Rgb(100, 70, 50),

            command_echo: Color::Rgb(255, 215, 181),
            selection_background: Color::Rgb(139, 90, 60),
            link_color: Color::Rgb(255, 153, 102),
            speech_color: Color::Rgb(232, 184, 136),
            whisper_color: Color::Rgb(232, 196, 160),
            thought_color: Color::Rgb(244, 212, 176),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Photophobia Friendly - Muted, low-brightness colors for light sensitivity
    pub fn photophobia_friendly() -> AppTheme {
        let mut theme = AppTheme {
            name: "Photophobia Friendly".to_string(),
            description: "Muted, low-brightness colors for light sensitivity".to_string(),

            window_border: Color::Rgb(58, 58, 58),
            window_border_focused: Color::Rgb(90, 122, 90),
            window_background: Color::Rgb(15, 15, 15),
            window_title: Color::Rgb(138, 138, 138),

            text_primary: Color::Rgb(138, 138, 138),
            text_secondary: Color::Rgb(106, 106, 106),
            text_disabled: Color::Rgb(64, 64, 64),
            text_selected: Color::Rgb(90, 122, 90),

            background_primary: Color::Rgb(15, 15, 15),
            background_secondary: Color::Rgb(18, 18, 18),
            background_selected: Color::Rgb(58, 58, 58),
            background_hover: Color::Rgb(25, 25, 25),

            editor_border: Color::Rgb(58, 58, 58),
            editor_label: Color::Rgb(106, 106, 106),
            editor_label_focused: Color::Rgb(90, 122, 90),
            editor_text: Color::Rgb(122, 122, 122),
            editor_cursor: Color::Rgb(90, 122, 90),
            editor_status: Color::Rgb(90, 106, 90),
            editor_background: Color::Rgb(18, 18, 18),

            browser_border: Color::Rgb(58, 58, 58),
            browser_title: Color::Rgb(138, 138, 138),
            browser_item_normal: Color::Rgb(122, 122, 122),
            browser_item_selected: Color::Rgb(15, 15, 15),
            browser_item_focused: Color::Rgb(90, 122, 90),
            browser_background: Color::Rgb(18, 18, 18),
            browser_scrollbar: Color::Rgb(58, 58, 58),

            form_border: Color::Rgb(58, 58, 58),
            form_label: Color::Rgb(106, 106, 106),
            form_label_focused: Color::Rgb(90, 122, 90),
            form_field_background: Color::Rgb(18, 18, 18),
            form_field_text: Color::Rgb(122, 122, 122),
            form_checkbox_checked: Color::Rgb(74, 106, 74),
            form_checkbox_unchecked: Color::Rgb(64, 64, 64),
            form_error: Color::Rgb(122, 74, 74),

            menu_border: Color::Rgb(58, 58, 58),
            menu_background: Color::Rgb(15, 15, 15),
            menu_item_normal: Color::Rgb(122, 122, 122),
            menu_item_selected: Color::Rgb(18, 18, 18),
            menu_item_focused: Color::Rgb(90, 122, 90),
            menu_separator: Color::Rgb(58, 58, 58),

            status_info: Color::Rgb(90, 106, 122),
            status_success: Color::Rgb(74, 106, 74),
            status_warning: Color::Rgb(122, 106, 74),
            status_error: Color::Rgb(122, 74, 74),
            status_background: Color::Rgb(15, 15, 15),

            button_normal: Color::Rgb(90, 122, 90),
            button_hover: Color::Rgb(106, 138, 106),
            button_active: Color::Rgb(74, 106, 74),
            button_disabled: Color::Rgb(64, 64, 64),

            command_echo: Color::Rgb(138, 138, 138),
            selection_background: Color::Rgb(58, 58, 58),
            link_color: Color::Rgb(90, 106, 122),
            speech_color: Color::Rgb(106, 106, 106),
            whisper_color: Color::Rgb(90, 106, 106),
            thought_color: Color::Rgb(122, 90, 122),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// ADHD Focus - Minimal color palette to reduce visual distractions
    pub fn adhd_focus() -> AppTheme {
        let mut theme = AppTheme {
            name: "ADHD Focus".to_string(),
            description: "Clean, minimal colors to reduce visual distractions".to_string(),

            window_border: Color::Rgb(64, 64, 64),
            window_border_focused: Color::Rgb(86, 156, 214),
            window_background: Color::Rgb(30, 30, 30),
            window_title: Color::Rgb(212, 212, 212),

            text_primary: Color::Rgb(212, 212, 212),
            text_secondary: Color::Rgb(128, 128, 128),
            text_disabled: Color::Rgb(96, 96, 96),
            text_selected: Color::Rgb(86, 156, 214),

            background_primary: Color::Rgb(30, 30, 30),
            background_secondary: Color::Rgb(37, 37, 38),
            background_selected: Color::Rgb(64, 64, 64),
            background_hover: Color::Rgb(45, 45, 45),

            editor_border: Color::Rgb(64, 64, 64),
            editor_label: Color::Rgb(156, 220, 254),
            editor_label_focused: Color::Rgb(86, 156, 214),
            editor_text: Color::Rgb(204, 204, 204),
            editor_cursor: Color::Rgb(86, 156, 214),
            editor_status: Color::Rgb(86, 156, 214),
            editor_background: Color::Rgb(37, 37, 38),

            browser_border: Color::Rgb(64, 64, 64),
            browser_title: Color::Rgb(212, 212, 212),
            browser_item_normal: Color::Rgb(204, 204, 204),
            browser_item_selected: Color::Rgb(30, 30, 30),
            browser_item_focused: Color::Rgb(86, 156, 214),
            browser_background: Color::Rgb(37, 37, 38),
            browser_scrollbar: Color::Rgb(64, 64, 64),

            form_border: Color::Rgb(64, 64, 64),
            form_label: Color::Rgb(156, 220, 254),
            form_label_focused: Color::Rgb(86, 156, 214),
            form_field_background: Color::Rgb(37, 37, 38),
            form_field_text: Color::Rgb(204, 204, 204),
            form_checkbox_checked: Color::Rgb(86, 156, 214),
            form_checkbox_unchecked: Color::Rgb(96, 96, 96),
            form_error: Color::Rgb(206, 145, 120),  // Only critical errors get different color

            menu_border: Color::Rgb(64, 64, 64),
            menu_background: Color::Rgb(30, 30, 30),
            menu_item_normal: Color::Rgb(204, 204, 204),
            menu_item_selected: Color::Rgb(37, 37, 38),
            menu_item_focused: Color::Rgb(86, 156, 214),
            menu_separator: Color::Rgb(64, 64, 64),

            status_info: Color::Rgb(86, 156, 214),
            status_success: Color::Rgb(86, 156, 214),  // Same color - minimal distraction
            status_warning: Color::Rgb(86, 156, 214),  // Same color - minimal distraction
            status_error: Color::Rgb(206, 145, 120),  // Only errors get different color
            status_background: Color::Rgb(30, 30, 30),

            button_normal: Color::Rgb(86, 156, 214),
            button_hover: Color::Rgb(86, 156, 214),
            button_active: Color::Rgb(86, 156, 214),
            button_disabled: Color::Rgb(96, 96, 96),

            command_echo: Color::Rgb(212, 212, 212),
            selection_background: Color::Rgb(64, 64, 64),
            link_color: Color::Rgb(86, 156, 214),
            speech_color: Color::Rgb(156, 220, 254),
            whisper_color: Color::Rgb(128, 128, 128),
            thought_color: Color::Rgb(204, 204, 204),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }

    /// Reduced Motion - Subtle colors to minimize visual stress
    pub fn reduced_motion() -> AppTheme {
        let mut theme = AppTheme {
            name: "Reduced Motion".to_string(),
            description: "Subtle colors to minimize visual stress and motion sensitivity".to_string(),

            window_border: Color::Rgb(90, 93, 97),
            window_border_focused: Color::Rgb(126, 163, 204),
            window_background: Color::Rgb(43, 45, 48),
            window_title: Color::Rgb(212, 212, 212),

            text_primary: Color::Rgb(212, 212, 212),
            text_secondary: Color::Rgb(157, 157, 157),
            text_disabled: Color::Rgb(100, 100, 100),
            text_selected: Color::Rgb(126, 163, 204),

            background_primary: Color::Rgb(43, 45, 48),
            background_secondary: Color::Rgb(51, 53, 56),
            background_selected: Color::Rgb(90, 93, 97),
            background_hover: Color::Rgb(60, 62, 65),

            editor_border: Color::Rgb(90, 93, 97),
            editor_label: Color::Rgb(136, 163, 196),
            editor_label_focused: Color::Rgb(126, 163, 204),
            editor_text: Color::Rgb(180, 180, 180),
            editor_cursor: Color::Rgb(126, 163, 204),
            editor_status: Color::Rgb(124, 182, 104),
            editor_background: Color::Rgb(51, 53, 56),

            browser_border: Color::Rgb(90, 93, 97),
            browser_title: Color::Rgb(212, 212, 212),
            browser_item_normal: Color::Rgb(180, 180, 180),
            browser_item_selected: Color::Rgb(43, 45, 48),
            browser_item_focused: Color::Rgb(126, 163, 204),
            browser_background: Color::Rgb(51, 53, 56),
            browser_scrollbar: Color::Rgb(90, 93, 97),

            form_border: Color::Rgb(90, 93, 97),
            form_label: Color::Rgb(157, 157, 157),
            form_label_focused: Color::Rgb(126, 163, 204),
            form_field_background: Color::Rgb(51, 53, 56),
            form_field_text: Color::Rgb(180, 180, 180),
            form_checkbox_checked: Color::Rgb(124, 182, 104),
            form_checkbox_unchecked: Color::Rgb(100, 100, 100),
            form_error: Color::Rgb(198, 99, 99),

            menu_border: Color::Rgb(90, 93, 97),
            menu_background: Color::Rgb(43, 45, 48),
            menu_item_normal: Color::Rgb(180, 180, 180),
            menu_item_selected: Color::Rgb(51, 53, 56),
            menu_item_focused: Color::Rgb(126, 163, 204),
            menu_separator: Color::Rgb(90, 93, 97),

            status_info: Color::Rgb(126, 163, 204),
            status_success: Color::Rgb(124, 182, 104),
            status_warning: Color::Rgb(212, 169, 89),
            status_error: Color::Rgb(198, 99, 99),
            status_background: Color::Rgb(43, 45, 48),

            button_normal: Color::Rgb(126, 163, 204),
            button_hover: Color::Rgb(136, 163, 196),
            button_active: Color::Rgb(124, 182, 104),
            button_disabled: Color::Rgb(100, 100, 100),

            command_echo: Color::Rgb(212, 212, 212),
            selection_background: Color::Rgb(90, 93, 97),
            link_color: Color::Rgb(136, 163, 196),
            speech_color: Color::Rgb(157, 157, 157),
            whisper_color: Color::Rgb(180, 180, 180),
            thought_color: Color::Rgb(160, 160, 160),
            injury_default_color: Color::Reset,
        };

        theme.injury_default_color =
            derive_injury_default_color(theme.window_background, theme.text_secondary);
        theme
    }
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
