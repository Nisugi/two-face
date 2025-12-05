//! TUI renderer for ProgressBar widget data.

use crate::frontend::common::widget_data::ProgressBarData;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Widget},
};

/// Parse a hex color string to ratatui Color
fn parse_color(hex: &str) -> Option<Color> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(Color::Rgb(r, g, b))
}

/// Render a progress bar using TUI (ratatui) primitives
pub fn render_progress_bar(data: &ProgressBarData, area: Rect, buf: &mut Buffer) {
    if (data.border.show_border && area.width < 3) || area.height < 1 {
        return;
    }

    if !data.border.show_border && area.width == 0 {
        return;
    }

    Clear.render(area, buf);

    // Fill background if not transparent
    if !data.transparent_background {
        let bg_color = data
            .window_background_color
            .as_ref()
            .and_then(|c| parse_color(c))
            .or_else(|| {
                data.bar_background_color
                    .as_ref()
                    .and_then(|c| parse_color(c))
            })
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

    // Render border if configured
    let inner_area = if data.border.show_border {
        let mut block = Block::default().borders(Borders::ALL);

        if let Some(ref style) = data.border.border_style {
            let border_type = match style.as_str() {
                "double" => BorderType::Double,
                "rounded" => BorderType::Rounded,
                "thick" => BorderType::Thick,
                _ => BorderType::Plain,
            };
            block = block.border_type(border_type);
        }

        if let Some(ref color_hex) = data.border.border_color {
            if let Some(color) = parse_color(color_hex) {
                block = block.border_style(Style::default().fg(color));
            }
        }

        block = block.title(data.label.as_str());

        let inner = block.inner(area);
        block.render(area, buf);
        inner
    } else {
        area
    };

    if inner_area.width == 0 || inner_area.height == 0 {
        return;
    }

    let text_width = data.display_text.len() as u16;
    let available_width = inner_area.width;

    let bar_color = data
        .bar_fill_color
        .as_ref()
        .and_then(|c| parse_color(c))
        .unwrap_or(Color::Green);

    let bar_bg_color = data
        .bar_background_color
        .as_ref()
        .and_then(|c| parse_color(c))
        .or_else(|| {
            data.window_background_color
                .as_ref()
                .and_then(|c| parse_color(c))
        })
        .unwrap_or(Color::Reset);

    // Calculate split point based on percentage
    let split_position = ((data.percentage as f64 / 100.0) * available_width as f64) as u16;

    // Render the bar background
    let y = inner_area.y;
    if y < buf.area().height {
        for i in 0..available_width {
            let x = inner_area.x + i;
            if x < buf.area().width {
                buf[(x, y)].set_char(' ');
                if i < split_position {
                    buf[(x, y)].set_bg(bar_color);
                } else if !data.transparent_background {
                    buf[(x, y)].set_bg(bar_bg_color);
                }
            }
        }
    }

    // Render text centered on the bar
    if text_width > 0 && text_width <= available_width {
        let text_start_x = inner_area.x + (available_width.saturating_sub(text_width)) / 2;
        let text_fg = data
            .text_color
            .as_ref()
            .and_then(|c| parse_color(c))
            .unwrap_or(Color::White);

        for (i, c) in data.display_text.chars().enumerate() {
            let x = text_start_x + i as u16;
            if x < inner_area.x + inner_area.width && x < buf.area().width {
                let char_position = x - inner_area.x;

                buf[(x, y)].set_char(c);
                buf[(x, y)].set_fg(text_fg);

                if char_position < split_position {
                    buf[(x, y)].set_bg(bar_color);
                } else if !data.transparent_background {
                    buf[(x, y)].set_bg(bar_bg_color);
                }
            }
        }
    }
}
