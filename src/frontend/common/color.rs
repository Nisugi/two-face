//! Frontend-agnostic color representation.
//!
//! This module provides UI-agnostic color types that can be converted to
//! TUI-specific (ratatui) or GUI-specific (egui/iced) color representations.

/// Represents a color in RGB format with optional alpha channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    /// Create a new RGB color
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Create a color from hex string (e.g., "#FF5733")
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Self::rgb(r, g, b))
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }

    // ANSI 256-color palette (standard terminal colors)
    pub const BLACK: Self = Self::rgb(0, 0, 0);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const YELLOW: Self = Self::rgb(255, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);
    pub const MAGENTA: Self = Self::rgb(255, 0, 255);
    pub const CYAN: Self = Self::rgb(0, 255, 255);
    pub const GRAY: Self = Self::rgb(128, 128, 128);
    pub const DARK_GRAY: Self = Self::rgb(64, 64, 64);
    pub const LIGHT_RED: Self = Self::rgb(255, 128, 128);
    pub const LIGHT_GREEN: Self = Self::rgb(128, 255, 128);
    pub const LIGHT_YELLOW: Self = Self::rgb(255, 255, 128);
    pub const LIGHT_BLUE: Self = Self::rgb(128, 128, 255);
    pub const LIGHT_MAGENTA: Self = Self::rgb(255, 128, 255);
    pub const LIGHT_CYAN: Self = Self::rgb(128, 255, 255);
    pub const WHITE: Self = Self::rgb(255, 255, 255);

    // Common UI colors
    pub const TRANSPARENT: Self = Self::rgb(0, 0, 0); // Will be handled specially in rendering
}

/// Named color variants for ease of use
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NamedColor {
    /// Standard ANSI colors
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    LightRed,
    LightGreen,
    LightYellow,
    LightBlue,
    LightMagenta,
    LightCyan,
    White,

    /// RGB color
    Rgb(u8, u8, u8),

    /// Indexed ANSI 256-color palette (0-255)
    Indexed(u8),

    /// Reset to default terminal color
    Reset,
}

impl NamedColor {
    /// Convert to RGB color
    pub fn to_rgb(&self) -> Color {
        match self {
            NamedColor::Black => Color::BLACK,
            NamedColor::Red => Color::RED,
            NamedColor::Green => Color::GREEN,
            NamedColor::Yellow => Color::YELLOW,
            NamedColor::Blue => Color::BLUE,
            NamedColor::Magenta => Color::MAGENTA,
            NamedColor::Cyan => Color::CYAN,
            NamedColor::Gray => Color::GRAY,
            NamedColor::DarkGray => Color::DARK_GRAY,
            NamedColor::LightRed => Color::LIGHT_RED,
            NamedColor::LightGreen => Color::LIGHT_GREEN,
            NamedColor::LightYellow => Color::LIGHT_YELLOW,
            NamedColor::LightBlue => Color::LIGHT_BLUE,
            NamedColor::LightMagenta => Color::LIGHT_MAGENTA,
            NamedColor::LightCyan => Color::LIGHT_CYAN,
            NamedColor::White => Color::WHITE,
            NamedColor::Rgb(r, g, b) => Color::rgb(*r, *g, *b),
            NamedColor::Indexed(idx) => {
                // ANSI 256-color to RGB approximation
                // This is a simplified conversion - real terminals may vary
                match idx {
                    0..=15 => {
                        // Basic 16 colors
                        match idx {
                            0 => Color::BLACK,
                            1 => Color::RED,
                            2 => Color::GREEN,
                            3 => Color::YELLOW,
                            4 => Color::BLUE,
                            5 => Color::MAGENTA,
                            6 => Color::CYAN,
                            7 => Color::GRAY,
                            8 => Color::DARK_GRAY,
                            9 => Color::LIGHT_RED,
                            10 => Color::LIGHT_GREEN,
                            11 => Color::LIGHT_YELLOW,
                            12 => Color::LIGHT_BLUE,
                            13 => Color::LIGHT_MAGENTA,
                            14 => Color::LIGHT_CYAN,
                            15 => Color::WHITE,
                            _ => Color::BLACK,
                        }
                    }
                    16..=231 => {
                        // 216-color cube (6x6x6)
                        let idx = idx - 16;
                        let r = ((idx / 36) * 51) as u8;
                        let g = (((idx % 36) / 6) * 51) as u8;
                        let b = ((idx % 6) * 51) as u8;
                        Color::rgb(r, g, b)
                    }
                    232..=255 => {
                        // Grayscale ramp
                        let gray = ((idx - 232) * 10 + 8) as u8;
                        Color::rgb(gray, gray, gray)
                    }
                    _ => Color::BLACK,
                }
            }
            NamedColor::Reset => Color::WHITE, // Default to white
        }
    }
}

impl From<Color> for NamedColor {
    fn from(color: Color) -> Self {
        NamedColor::Rgb(color.r, color.g, color.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_constants() {
        assert_eq!(Color::RED, Color::rgb(255, 0, 0));
        assert_eq!(Color::GREEN, Color::rgb(0, 255, 0));
        assert_eq!(Color::BLUE, Color::rgb(0, 0, 255));
    }

    #[test]
    fn test_hex_conversion() {
        let color = Color::from_hex("#FF5733").unwrap();
        assert_eq!(color.r, 255);
        assert_eq!(color.g, 87);
        assert_eq!(color.b, 51);
        assert_eq!(color.to_hex(), "#FF5733");
    }

    #[test]
    fn test_named_color_conversion() {
        assert_eq!(NamedColor::Red.to_rgb(), Color::RED);
        assert_eq!(NamedColor::Rgb(255, 128, 64).to_rgb(), Color::rgb(255, 128, 64));
    }
}
