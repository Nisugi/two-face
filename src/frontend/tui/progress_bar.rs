//! Vital/stat style progress bar used throughout the HUD.
//!
//! Provides configurable borders, text, and fill colors so it matches the theme
//! chosen by the user.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Widget},
};

/// A progress bar widget for displaying vitals (health, mana, stamina, spirit)
pub struct ProgressBar {
    label: String,
    current: u32,
    max: u32,
    custom_text: Option<String>,
    show_border: bool,
    border_style: Option<String>,
    border_color: Option<Color>,
    bar_fill: Option<Color>,
    bar_background: Option<Color>,
    window_background: Option<Color>,
    transparent_background: bool,
    text_color: Option<Color>,
}

impl ProgressBar {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            current: 0,
            max: 100,
            custom_text: None,
            show_border: false,
            border_style: None,
            border_color: None,
            bar_fill: Some(Color::Rgb(0, 255, 0)), // Green by default
            bar_background: None,
            window_background: None,
            transparent_background: true,
            text_color: Some(Color::White),
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
        self.border_color = border_color.and_then(|c| Self::parse_color(&c));
    }

    pub fn set_title(&mut self, title: String) {
        self.label = title;
    }

    pub fn set_colors(&mut self, bar_fill: Option<String>, bar_background: Option<String>) {
        if let Some(fill) = bar_fill.and_then(|c| Self::parse_color(&c)) {
            self.bar_fill = Some(fill);
        }
        if let Some(bg) = bar_background.and_then(|c| Self::parse_color(&c)) {
            self.bar_background = Some(bg);
        }
    }

    pub fn set_value(&mut self, current: u32, max: u32) {
        self.current = current;
        self.max = max;
        self.custom_text = None;
    }

    pub fn set_value_with_text(&mut self, current: u32, max: u32, custom_text: Option<String>) {
        self.current = current;
        self.max = max;
        self.custom_text = custom_text;
    }

    pub fn set_transparent_background(&mut self, transparent: bool) {
        self.transparent_background = transparent;
    }

    pub fn set_text_color(&mut self, color: Option<String>) {
        self.text_color = color.and_then(|c| Self::parse_color(&c));
    }

    pub fn set_background_color(&mut self, color: Option<String>) {
        self.window_background = color.and_then(|c| Self::parse_color(&c));
    }

    fn parse_color(hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

        Some(Color::Rgb(r, g, b))
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        if (self.show_border && area.width < 3) || area.height < 1 {
            return;
        }

        if !self.show_border && area.width == 0 {
            return;
        }

        Clear.render(area, buf);

        if !self.transparent_background {
            let bg_color = self
                .window_background
                .or(self.bar_background)
                .unwrap_or(Color::Reset);
            for row in 0..area.height {
                for col in 0..area.width {
                    let x = area.x + col;
                    let y = area.y + row;
                    if x < buf.area().width && y < buf.area().height {
                        buf[(x, y)].set_bg(bg_color);
                    }
                }
            }
        }

        let inner_area = if self.show_border {
            let mut block = Block::default().borders(Borders::ALL);

            if let Some(ref style) = self.border_style {
                let border_type = match style.as_str() {
                    "double" => BorderType::Double,
                    "rounded" => BorderType::Rounded,
                    "thick" => BorderType::Thick,
                    _ => BorderType::Plain,
                };
                block = block.border_type(border_type);
            }

            if let Some(color) = self.border_color {
                block = block.border_style(Style::default().fg(color));
            }

            block = block.title(self.label.as_str());

            let inner = block.inner(area);
            use ratatui::widgets::Widget;
            block.render(area, buf);
            inner
        } else {
            area
        };

        if inner_area.width == 0 || inner_area.height == 0 {
            return;
        }

        // Calculate percentage
        let percentage = if self.max > 0 {
            (self.current as f64 / self.max as f64 * 100.0) as u32
        } else {
            0
        };

        // Build display text
        let display_text = if let Some(ref custom) = self.custom_text {
            custom.clone()
        } else {
            format!("{}/{}", self.current, self.max)
        };

        let text_width = display_text.len() as u16;
        let available_width = inner_area.width;

        let bar_color = self.bar_fill.unwrap_or(Color::Green);
        let bar_bg_color = self
            .bar_background
            .or(self.window_background)
            .unwrap_or(Color::Reset);

        // Calculate split point based on percentage
        let split_position = ((percentage as f64 / 100.0) * available_width as f64) as u16;

        // Render the bar background
        let y = inner_area.y;
        if y < buf.area().height {
            for i in 0..available_width {
                let x = inner_area.x + i;
                if x < buf.area().width {
                    buf[(x, y)].set_char(' ');
                    if i < split_position {
                        buf[(x, y)].set_bg(bar_color);
                    } else if !self.transparent_background {
                        buf[(x, y)].set_bg(bar_bg_color);
                    }
                }
            }
        }

        // Render text centered on the bar
        if text_width > 0 && text_width <= available_width {
            let text_start_x = inner_area.x + (available_width.saturating_sub(text_width)) / 2;
            let text_fg = self.text_color.unwrap_or(Color::White);

            for (i, c) in display_text.chars().enumerate() {
                let x = text_start_x + i as u16;
                if x < inner_area.x + inner_area.width && x < buf.area().width {
                    let char_position = x - inner_area.x;

                    buf[(x, y)].set_char(c);
                    buf[(x, y)].set_fg(text_fg);

                    if char_position < split_position {
                        buf[(x, y)].set_bg(bar_color);
                    } else if !self.transparent_background {
                        buf[(x, y)].set_bg(bar_bg_color);
                    }
                }
            }
        }
    }

    pub fn render_with_focus(&self, area: Rect, buf: &mut Buffer, _focused: bool) {
        self.render(area, buf);
    }

    pub fn render_themed(&self, area: Rect, buf: &mut Buffer, _theme: &crate::theme::AppTheme) {
        self.render(area, buf);
    }
}
