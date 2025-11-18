//! Renders the profanity-style injury doll showing wounds/scars per body part.
//!
//! The widget maps injury levels to configurable colors and can be embedded in
//! any window with optional borders/background alignment tweaks.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Clear, Widget as RatatuiWidget},
};
use std::collections::HashMap;

/// Injury doll widget showing body part injuries/scars
/// Layout:
///  👁   👁
///     0    ns
///    /|\
///   o | o  nk
///    / \
///   o   o  bk
pub struct InjuryDoll {
    label: String,
    // Map body part name to injury level (0=none, 1-3=injury, 4-6=scar)
    injuries: HashMap<String, u8>,
    show_border: bool,
    border_style: Option<String>,
    border_color: Option<Color>,
    border_sides: crate::config::BorderSides,
    // ProfanityFE injury colors: none, injury1-3, scar1-3
    colors: Vec<String>,
    background_color: Option<Color>,
    content_align: Option<String>,
    transparent_background: bool,
}

impl InjuryDoll {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            injuries: HashMap::new(),
            show_border: false,
            border_style: None,
            border_color: None,
            border_sides: crate::config::BorderSides::default(),
            colors: vec![
                "#333333".to_string(), // 0: none
                "#aa5500".to_string(), // 1: injury 1 (brown)
                "#ff8800".to_string(), // 2: injury 2 (orange)
                "#ff0000".to_string(), // 3: injury 3 (bright red)
                "#999999".to_string(), // 4: scar 1 (light gray)
                "#777777".to_string(), // 5: scar 2 (medium gray)
                "#555555".to_string(), // 6: scar 3 (darker gray)
            ],
            background_color: None,
            content_align: None,
            transparent_background: true, // Default to transparent
        }
    }

    pub fn set_border_config(
        &mut self,
        show_border: bool,
        border_style: Option<String>,
        border_color: Option<String>,
    ) {
        self.show_border = show_border;
        self.border_style = border_style;
        self.border_color = border_color.and_then(|value| Self::parse_color(&value));
    }

    pub fn set_border_sides(&mut self, border_sides: crate::config::BorderSides) {
        self.border_sides = border_sides;
    }

    pub fn set_title(&mut self, title: String) {
        self.label = title;
    }

    pub fn set_injury(&mut self, body_part: String, level: u8) {
        self.injuries.insert(body_part, level.min(6));
    }

    pub fn set_colors(&mut self, colors: Vec<String>) {
        if colors.len() == 7 {
            self.colors = colors;
        }
    }

    pub fn set_background_color(&mut self, color: Option<String>) {
        self.background_color = match color {
            Some(ref s) if s.trim() == "-" => None,
            Some(value) => Self::parse_color(&value),
            None => None,
        };
    }

    pub fn set_content_align(&mut self, align: Option<String>) {
        self.content_align = align;
    }

    pub fn set_transparent_background(&mut self, transparent: bool) {
        self.transparent_background = transparent;
    }

    fn parse_color(hex: &str) -> Option<Color> {
        if !hex.starts_with('#') || hex.len() != 7 {
            return None;
        }

        let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }

    fn fill_background(area: Rect, buf: &mut Buffer, color: Color) {
        for row in 0..area.height {
            for col in 0..area.width {
                let x = area.x + col;
                let y = area.y + row;
                if x < buf.area().width && y < buf.area().height {
                    buf[(x, y)].set_bg(color);
                }
            }
        }
    }

    fn get_injury_color(&self, body_part: &str) -> Color {
        let level = self.injuries.get(body_part).copied().unwrap_or(0);
        let color_hex = &self.colors[level as usize];
        Self::parse_color(color_hex).unwrap_or(Color::White)
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if !self.transparent_background {
            Clear.render(area, buf);
            if let Some(bg_color) = self.background_color {
                Self::fill_background(area, buf, bg_color);
            }
        }

        let mut block = Block::default();
        if !self.transparent_background {
            if let Some(bg_color) = self.background_color {
                block = block.style(Style::default().bg(bg_color));
            }
        }

        if self.show_border {
            let borders = crate::config::parse_border_sides(&self.border_sides);
            block = block.borders(borders);

            if let Some(ref style) = self.border_style {
                let border_type = match style.as_str() {
                    "double" => BorderType::Double,
                    "rounded" => BorderType::Rounded,
                    "thick" => BorderType::Thick,
                    "quadrant_inside" => BorderType::QuadrantInside,
                    "quadrant_outside" => BorderType::QuadrantOutside,
                    _ => BorderType::Plain,
                };
                block = block.border_type(border_type);
            }

            if let Some(color) = self.border_color {
                block = block.border_style(Style::default().fg(color));
            }

            block = block.title(self.label.as_str());
        }

        let inner_area = if self.show_border {
            block.inner(area)
        } else {
            area
        };

        if self.show_border {
            block.render(area, buf);
        }

        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        let bg_color = self.background_color;

        // Calculate content alignment offset
        // Injury doll content is 5 cols x 6 rows
        const CONTENT_WIDTH: u16 = 5;
        const CONTENT_HEIGHT: u16 = 6;

        let (row_offset, col_offset) = if let Some(ref align_str) = self.content_align {
            let align = crate::config::ContentAlign::from_str(align_str);
            align.calculate_offset(
                CONTENT_WIDTH,
                CONTENT_HEIGHT,
                inner_area.width,
                inner_area.height,
            )
        } else {
            (0, 0) // Default to top-left
        };

        // Define all body part positions (col, row, char, body_part_name)
        let positions = [
            // Row 0: Eyes
            (0, 0, '\u{f06e}', "leftEye"), // Nerd Font eye icon
            (4, 0, '\u{f06e}', "rightEye"),
            // Row 1: Head
            (2, 1, '0', "head"),
            // Row 2: Arms/Chest
            (1, 2, '/', "leftArm"),
            (2, 2, '|', "chest"),
            (3, 2, '\\', "rightArm"),
            // Row 3: Hands/Abdomen
            (0, 3, 'o', "leftHand"),
            (2, 3, '|', "abdomen"),
            (4, 3, 'o', "rightHand"),
            // Row 4: Leg tops
            (1, 4, '/', "leftLeg"),
            (3, 4, '\\', "rightLeg"),
            // Row 5: Leg bottoms (same body parts, just visual continuation)
            (0, 5, 'o', "leftLeg"),
            (4, 5, 'o', "rightLeg"),
        ];

        // Render body parts
        for (col, row, ch, body_part) in positions.iter() {
            let x = inner_area.x + col + col_offset;
            let y = inner_area.y + row + row_offset;

            // Bounds check
            if x < buf.area().width && y < buf.area().height {
                let color = self.get_injury_color(body_part);
                buf[(x, y)].set_char(*ch);
                buf[(x, y)].set_fg(color);
                if !self.transparent_background {
                    if let Some(bg) = bg_color {
                        buf[(x, y)].set_bg(bg);
                    }
                }
            }
        }

        // Render special indicators on the right with text labels: nk, bk, ns
        let text_indicators = [
            (6, 1, "nk", "neck"), // neck - row 1
            (6, 3, "bk", "back"), // back - row 3
            (6, 5, "ns", "nsys"), // nerves - row 5
        ];

        for (start_col, row, text, body_part) in text_indicators.iter() {
            let color = self.get_injury_color(body_part);

            for (i, ch) in text.chars().enumerate() {
                let x = inner_area.x + start_col + i as u16 + col_offset;
                let y = inner_area.y + row + row_offset;

                if x < buf.area().width && y < buf.area().height {
                    buf[(x, y)].set_char(ch);
                    buf[(x, y)].set_fg(color);
                    if !self.transparent_background {
                        if let Some(bg) = bg_color {
                            buf[(x, y)].set_bg(bg);
                        }
                    }
                }
            }
        }
    }
}
