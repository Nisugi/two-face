//! Compact dashboard widget for rendering stance/status indicator rows.
//!
//! Supports horizontal/vertical/grid layouts with configurable spacing,
//! alignment, and optional hiding of inactive icons.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Widget as RatatuiWidget},
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DashboardLayout {
    Horizontal,
    Vertical,
    Grid { rows: usize, cols: usize },
}

#[derive(Debug, Clone)]
pub struct DashboardIndicator {
    pub id: String,
    pub icon: String,
    pub colors: Vec<String>, // [off_color, on_color] or multi-level
    pub value: u8,           // 0 = off, 1+ = on (or multi-level)
}

pub struct Dashboard {
    label: String,
    indicators: Vec<DashboardIndicator>,
    indicator_map: HashMap<String, usize>,
    layout: DashboardLayout,
    spacing: u16,
    hide_inactive: bool,
    show_border: bool,
    border_style: Option<String>,
    border_color: Option<String>,
    border_sides: crate::config::BorderSides,
    background_color: Option<String>,
    content_align: Option<String>,
    transparent_background: bool,
}

impl Dashboard {
    pub fn new(label: &str, layout: DashboardLayout) -> Self {
        Self {
            label: label.to_string(),
            indicators: Vec::new(),
            indicator_map: HashMap::new(),
            layout,
            spacing: 1,
            hide_inactive: true,
            show_border: true,
            border_style: Some("single".to_string()),
            border_color: Some("#808080".to_string()),
            border_sides: crate::config::BorderSides::default(),
            background_color: None,
            content_align: None,
            transparent_background: true,
        }
    }

    pub fn add_indicator(&mut self, id: String, icon: String, colors: Vec<String>) {
        let indicator = DashboardIndicator {
            id: id.clone(),
            icon,
            colors,
            value: 0, // Default to off
        };

        self.indicator_map.insert(id.clone(), self.indicators.len());
        self.indicators.push(indicator);
    }

    pub fn set_indicator_value(&mut self, id: &str, value: u8) {
        if let Some(&idx) = self.indicator_map.get(id) {
            if let Some(indicator) = self.indicators.get_mut(idx) {
                indicator.value = value;
            }
        }
    }

    pub fn set_spacing(&mut self, spacing: u16) {
        self.spacing = spacing;
    }

    pub fn set_hide_inactive(&mut self, hide: bool) {
        self.hide_inactive = hide;
    }

    pub fn set_content_align(&mut self, align: Option<String>) {
        self.content_align = align;
    }

    pub fn set_border_config(&mut self, show: bool, style: Option<String>, color: Option<String>) {
        self.show_border = show;
        self.border_style = style;
        self.border_color = color;
    }

    pub fn set_border_sides(&mut self, sides: crate::config::BorderSides) {
        self.border_sides = sides;
    }

    pub fn set_background_color(&mut self, color: Option<String>) {
        self.background_color = color;
    }

    pub fn set_transparent_background(&mut self, transparent: bool) {
        self.transparent_background = transparent;
    }

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

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Clear area
        for y in area.top()..area.bottom() {
            for x in area.left()..area.right() {
                buf.get_mut(x, y).reset();
            }
        }

        // Create border block if enabled
        let inner_area = if self.show_border {
            let mut block = Block::default();

            let border_type = match self.border_style.as_deref() {
                Some("double") => BorderType::Double,
                Some("rounded") => BorderType::Rounded,
                Some("thick") => BorderType::Thick,
                _ => BorderType::Plain,
            };

            let borders = crate::config::parse_border_sides(&self.border_sides);

            block = block.borders(borders).border_type(border_type);

            if let Some(ref color_str) = self.border_color {
                let color = Self::parse_color(color_str);
                block = block.border_style(Style::default().fg(color));
            }

            if !self.label.is_empty() {
                block = block.title(self.label.clone());
            }

            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        // Set background if not transparent
        if !self.transparent_background {
            if let Some(ref bg_color_str) = self.background_color {
                let bg_color = Self::parse_color(bg_color_str);
                for y in inner_area.top()..inner_area.bottom() {
                    for x in inner_area.left()..inner_area.right() {
                        buf.get_mut(x, y).set_bg(bg_color);
                    }
                }
            }
        }

        // Filter visible indicators
        let visible_indicators: Vec<&DashboardIndicator> = self
            .indicators
            .iter()
            .filter(|ind| !self.hide_inactive || ind.value > 0)
            .collect();

        if visible_indicators.is_empty() {
            return;
        }

        // Render based on layout
        match self.layout {
            DashboardLayout::Horizontal => {
                self.render_horizontal(&visible_indicators, inner_area, buf)
            }
            DashboardLayout::Vertical => self.render_vertical(&visible_indicators, inner_area, buf),
            DashboardLayout::Grid { rows, cols } => {
                self.render_grid(&visible_indicators, rows, cols, inner_area, buf)
            }
        }
    }

    fn render_horizontal(&self, indicators: &[&DashboardIndicator], area: Rect, buf: &mut Buffer) {
        // Calculate total width needed
        let total_width: usize = indicators
            .iter()
            .map(|ind| ind.icon.chars().count())
            .sum::<usize>()
            + (indicators.len().saturating_sub(1)) * self.spacing as usize;

        // Calculate starting x position based on alignment
        let start_x = self.calculate_horizontal_offset(total_width, area.width as usize, area.x);

        let mut x = start_x;
        for indicator in indicators {
            let color_index =
                (indicator.value as usize).min(indicator.colors.len().saturating_sub(1));
            let color = Self::parse_color(&indicator.colors[color_index]);

            for ch in indicator.icon.chars() {
                if x >= area.right() {
                    break;
                }
                buf.get_mut(x, area.y).set_char(ch).set_fg(color);
                x += 1;
            }

            x += self.spacing;
        }
    }

    fn render_vertical(&self, indicators: &[&DashboardIndicator], area: Rect, buf: &mut Buffer) {
        // Calculate total height needed
        let total_height =
            indicators.len() + (indicators.len().saturating_sub(1)) * self.spacing as usize;

        // Calculate starting y position based on alignment
        let start_y = self.calculate_vertical_offset(total_height, area.height as usize, area.y);

        let mut y = start_y;
        for indicator in indicators {
            if y >= area.bottom() {
                break;
            }

            let color_index =
                (indicator.value as usize).min(indicator.colors.len().saturating_sub(1));
            let color = Self::parse_color(&indicator.colors[color_index]);

            let mut x = area.x;
            for ch in indicator.icon.chars() {
                if x >= area.right() {
                    break;
                }
                buf.get_mut(x, y).set_char(ch).set_fg(color);
                x += 1;
            }

            y += 1 + self.spacing;
        }
    }

    fn render_grid(
        &self,
        indicators: &[&DashboardIndicator],
        grid_rows: usize,
        grid_cols: usize,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let cell_width = area.width as usize / grid_cols;
        let cell_height = area.height as usize / grid_rows;

        for (idx, indicator) in indicators.iter().enumerate() {
            if idx >= grid_rows * grid_cols {
                break;
            }

            let grid_row = idx / grid_cols;
            let grid_col = idx % grid_cols;

            let x = area.x + (grid_col * cell_width) as u16;
            let y = area.y + (grid_row * cell_height) as u16;

            let color_index =
                (indicator.value as usize).min(indicator.colors.len().saturating_sub(1));
            let color = Self::parse_color(&indicator.colors[color_index]);

            let mut curr_x = x;
            for ch in indicator.icon.chars() {
                if curr_x >= area.right() || curr_x >= x + cell_width as u16 {
                    break;
                }
                buf.get_mut(curr_x, y).set_char(ch).set_fg(color);
                curr_x += 1;
            }
        }
    }

    fn calculate_horizontal_offset(
        &self,
        content_width: usize,
        available_width: usize,
        base_x: u16,
    ) -> u16 {
        let align = self.content_align.as_deref().unwrap_or("left");
        match align {
            "center" => {
                let offset = if available_width > content_width {
                    (available_width - content_width) / 2
                } else {
                    0
                };
                base_x + offset as u16
            }
            "right" => {
                let offset = if available_width > content_width {
                    available_width - content_width
                } else {
                    0
                };
                base_x + offset as u16
            }
            _ => base_x,
        }
    }

    fn calculate_vertical_offset(
        &self,
        content_height: usize,
        available_height: usize,
        base_y: u16,
    ) -> u16 {
        let align = self.content_align.as_deref().unwrap_or("top");
        match align {
            "center" => {
                let offset = if available_height > content_height {
                    (available_height - content_height) / 2
                } else {
                    0
                };
                base_y + offset as u16
            }
            "bottom" => {
                let offset = if available_height > content_height {
                    available_height - content_height
                } else {
                    0
                };
                base_y + offset as u16
            }
            _ => base_y,
        }
    }
}
