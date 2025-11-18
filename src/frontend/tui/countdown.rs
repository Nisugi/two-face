//! Simple countdown timer widget that mirrors Profanity's RT/CT bars.
//!
//! Displays a numeric timer plus up to ten block glyphs so the user can gauge
//! duration at a glance.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Widget},
};
use std::time::{SystemTime, UNIX_EPOCH};

/// A countdown widget for displaying roundtime, casttime, stuntime, etc.
pub struct Countdown {
    label: String,
    end_time: i64, // Unix timestamp when countdown ends
    show_border: bool,
    border_style: Option<String>,
    border_color: Option<String>,
    text_color: Option<String>,
    transparent_background: bool,
    icon: char, // Character to use for countdown blocks
}

impl Countdown {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            end_time: 0,
            show_border: true,
            border_style: None,
            border_color: None,
            text_color: None,
            transparent_background: true,
            icon: '█', // Default to filled block
        }
    }

    pub fn set_icon(&mut self, icon: char) {
        self.icon = icon;
    }

    pub fn set_border_config(
        &mut self,
        show_border: bool,
        border_style: Option<String>,
        border_color: Option<String>,
    ) {
        self.show_border = show_border;
        self.border_style = border_style;
        self.border_color = border_color;
    }

    pub fn set_title(&mut self, title: String) {
        self.label = title;
    }

    pub fn set_text_color(&mut self, color: Option<String>) {
        self.text_color = color;
    }

    pub fn set_transparent_background(&mut self, transparent: bool) {
        self.transparent_background = transparent;
    }

    pub fn set_end_time(&mut self, end_time: i64) {
        self.end_time = end_time;
    }

    /// Get remaining seconds
    /// Applies server_time_offset to local time to account for clock drift
    fn remaining_seconds(&self, server_time_offset: i64) -> i64 {
        let local_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let adjusted_time = local_time + server_time_offset;
        self.end_time - adjusted_time
    }

    /// Parse a hex color string to ratatui Color
    fn parse_color(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Color::White;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

        Color::Rgb(r, g, b)
    }

    pub fn render(
        &self,
        area: Rect,
        buf: &mut Buffer,
        server_time_offset: i64,
        theme: &crate::theme::AppTheme,
    ) {
        if area.width < 3 || area.height < 1 {
            return;
        }

        let border_color = self
            .border_color
            .as_ref()
            .map(|c| Self::parse_color(c))
            .unwrap_or(Color::White);

        let inner_area: Rect;

        if self.show_border {
            // Use Block widget for borders
            let mut block = Block::default().borders(ratatui::widgets::Borders::ALL);

            if let Some(ref style) = self.border_style {
                use ratatui::widgets::BorderType;
                let border_type = match style.as_str() {
                    "double" => BorderType::Double,
                    "rounded" => BorderType::Rounded,
                    "thick" => BorderType::Thick,
                    _ => BorderType::Plain,
                };
                block = block.border_type(border_type);
            }

            block = block.border_style(Style::default().fg(border_color));
            block = block.title(self.label.as_str());

            inner_area = block.inner(area);
            block.render(area, buf);
        } else {
            inner_area = area;
        }

        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        let remaining = self.remaining_seconds(server_time_offset).max(0) as u32;

        let text_color = self
            .text_color
            .as_ref()
            .map(|c| Self::parse_color(c))
            .unwrap_or(Color::White);

        // Clear the bar area
        let y = inner_area.y;
        if y < buf.area().height {
            for i in 0..inner_area.width {
                let x = inner_area.x + i;
                if x < buf.area().width {
                    buf[(x, y)].set_char(' ');
                }
            }
        }

        // If countdown is 0, leave it blank (invisible)
        if remaining == 0 {
            return;
        }

        // Simple block-based countdown:
        // - Max 10 blocks
        // - Show N blocks where N = min(remaining_seconds, 10)
        const MAX_BLOCKS: u32 = 10;
        let blocks_to_show = remaining.min(MAX_BLOCKS);

        // Right-align the number so it doesn't shift when going from 10->9
        // Reserve 2 chars for the number + 1 for space = 3 total
        // Format: " 9 ████████" or "10 ████████"
        let remaining_text = format!("{:>2} ", remaining);
        let text_width = remaining_text.len() as u16; // Always 3 chars

        // Render countdown number on the left (right-aligned within 3 chars)
        let y = inner_area.y;
        if y < buf.area().height {
            for (i, c) in remaining_text.chars().enumerate() {
                let x = inner_area.x + i as u16;
                if x < inner_area.x + inner_area.width && x < buf.area().width {
                    buf[(x, y)].set_char(c);
                    buf[(x, y)].set_fg(text_color);
                }
            }

            // Render blocks after the number
            for i in 0..blocks_to_show {
                let pos = text_width + i as u16;
                if pos < inner_area.width {
                    let x = inner_area.x + pos;
                    if x < buf.area().width {
                        buf[(x, y)].set_char(self.icon);
                        buf[(x, y)].set_fg(text_color);
                    }
                }
            }
        }
    }
}
