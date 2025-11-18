//! Multi-tab wrapper around `TextWindow` for stream multiplexing.
//!
//! Handles unread counts, tab bar placement, and themed chrome while delegating
//! actual text rendering to the existing `TextWindow`.

use super::text_window::TextWindow;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Widget as RatatuiWidget},
};

#[derive(Clone, Debug, PartialEq)]
pub enum TabBarPosition {
    Top,
    Bottom,
}

struct TabInfo {
    name: String,
    stream: String,
    window: TextWindow,
    has_unread: bool,
    unread_count: usize,
}

pub struct TabbedTextWindow {
    tabs: Vec<TabInfo>,
    active_tab_index: usize,
    tab_bar_position: TabBarPosition,
    show_border: bool,
    border_style: Option<String>,
    border_color: Option<String>,
    border_sides: crate::config::BorderSides,
    title: String,
    transparent_background: bool,
    background_color: Option<String>,
    content_text_color: Option<String>,
    tab_active_color: Option<String>,
    tab_inactive_color: Option<String>,
    tab_unread_color: Option<String>,
    tab_unread_prefix: String,
}

impl TabbedTextWindow {
    pub fn new(title: &str, tab_bar_position: TabBarPosition) -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_index: 0,
            tab_bar_position,
            show_border: true,
            border_style: Some("single".to_string()),
            border_color: Some("#808080".to_string()),
            border_sides: crate::config::BorderSides::default(),
            title: title.to_string(),
            transparent_background: true,
            background_color: None,
            content_text_color: None,
            tab_active_color: Some("#FFFF00".to_string()), // Yellow
            tab_inactive_color: Some("#808080".to_string()), // Gray
            tab_unread_color: Some("#FFFFFF".to_string()), // White
            tab_unread_prefix: "* ".to_string(),
        }
    }

    pub fn with_tabs(title: &str, tabs: Vec<(String, String)>, max_lines_per_tab: usize) -> Self {
        let mut window = Self::new(title, TabBarPosition::Top);
        for (name, stream) in tabs {
            window.add_tab(name, stream, max_lines_per_tab, false);
        }
        window
    }

    pub fn with_border_config(
        mut self,
        show: bool,
        style: Option<String>,
        color: Option<String>,
    ) -> Self {
        self.show_border = show;
        self.border_style = style;
        self.border_color = color;
        self
    }

    pub fn with_tab_bar_position(mut self, position: TabBarPosition) -> Self {
        self.tab_bar_position = position;
        self
    }

    pub fn with_tab_colors(
        mut self,
        active: Option<String>,
        inactive: Option<String>,
        unread: Option<String>,
    ) -> Self {
        self.tab_active_color = active;
        self.tab_inactive_color = inactive;
        self.tab_unread_color = unread;
        self
    }

    pub fn with_unread_prefix(mut self, prefix: String) -> Self {
        self.tab_unread_prefix = prefix;
        self
    }

    pub fn add_tab(
        &mut self,
        name: String,
        stream: String,
        max_lines: usize,
        show_timestamps: bool,
    ) {
        let mut window = TextWindow::new(&name, max_lines);
        window.set_show_timestamps(show_timestamps);
        window.set_border_config(false, None, None); // Tabs don't have their own borders
        window.set_background_color(self.background_color.clone());
        window.set_text_color(self.content_text_color.clone());

        self.tabs.push(TabInfo {
            name,
            stream,
            window,
            has_unread: false,
            unread_count: 0,
        });
    }

    pub fn remove_tab(&mut self, name: &str) -> bool {
        // Can't remove the last tab
        if self.tabs.len() <= 1 {
            return false;
        }

        if let Some(idx) = self.tabs.iter().position(|t| t.name == name) {
            self.tabs.remove(idx);

            // Adjust active index if needed
            if self.active_tab_index >= self.tabs.len() {
                self.active_tab_index = self.tabs.len().saturating_sub(1);
            }
            true
        } else {
            false
        }
    }

    pub fn rename_tab(&mut self, old_name: &str, new_name: String) -> bool {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.name == old_name) {
            tab.name = new_name;
            true
        } else {
            false
        }
    }

    pub fn switch_to_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab_index = index;
            // Clear unread status
            if let Some(tab) = self.tabs.get_mut(index) {
                tab.has_unread = false;
                tab.unread_count = 0;
            }
        }
    }

    pub fn switch_to_tab_by_name(&mut self, name: &str) {
        if let Some(idx) = self.tabs.iter().position(|t| t.name == name) {
            self.switch_to_tab(idx);
        }
    }

    pub fn get_tab_names(&self) -> Vec<String> {
        self.tabs.iter().map(|t| t.name.clone()).collect()
    }

    /// Switch to the next tab (wraps around to first tab)
    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            let next_index = (self.active_tab_index + 1) % self.tabs.len();
            self.switch_to_tab(next_index);
        }
    }

    /// Switch to the previous tab (wraps around to last tab)
    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            let prev_index = if self.active_tab_index == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab_index - 1
            };
            self.switch_to_tab(prev_index);
        }
    }

    /// Switch to the next tab with unread messages
    /// Returns true if found and switched, false if no unread tabs
    pub fn next_tab_with_unread(&mut self) -> bool {
        if self.tabs.is_empty() {
            return false;
        }

        // Search for next unread tab starting from current position
        let start_index = (self.active_tab_index + 1) % self.tabs.len();

        // First pass: from current+1 to end
        for i in start_index..self.tabs.len() {
            if self.tabs[i].has_unread {
                self.switch_to_tab(i);
                return true;
            }
        }

        // Second pass: from beginning to current (wrap around)
        for i in 0..start_index {
            if self.tabs[i].has_unread {
                self.switch_to_tab(i);
                return true;
            }
        }

        false
    }

    /// Get current active tab index
    pub fn get_active_tab_index(&self) -> usize {
        self.active_tab_index
    }

    /// Check if any tabs have unread messages
    pub fn has_unread_tabs(&self) -> bool {
        self.tabs.iter().any(|t| t.has_unread)
    }

    pub fn reorder_tabs(&mut self, new_order: &[String]) {
        let mut new_tabs = Vec::new();
        for name in new_order {
            if let Some(idx) = self.tabs.iter().position(|t| &t.name == name) {
                new_tabs.push(self.tabs.remove(idx));
            }
        }
        // Add any tabs that weren't in the new order
        new_tabs.append(&mut self.tabs);
        self.tabs = new_tabs;

        // Reset active index
        self.active_tab_index = 0;
    }

    pub fn add_text_to_stream(&mut self, stream: &str, styled: super::text_window::StyledText) {
        for (idx, tab) in self.tabs.iter_mut().enumerate() {
            if tab.stream == stream {
                tab.window.add_text(styled.clone());

                // Mark as unread if not active tab
                if idx != self.active_tab_index {
                    tab.has_unread = true;
                    tab.unread_count += 1;
                }
            }
        }
    }

    pub fn finish_line_for_stream(&mut self, stream: &str, width: u16) {
        for tab in &mut self.tabs {
            if tab.stream == stream {
                tab.window.finish_line(width);
            }
        }
    }

    pub fn add_text_to_tab(&mut self, tab_name: &str, styled: super::text_window::StyledText) {
        if let Some((idx, tab)) = self
            .tabs
            .iter_mut()
            .enumerate()
            .find(|(_, t)| t.name == tab_name)
        {
            tab.window.add_text(styled);

            // Mark as unread if not active tab
            if idx != self.active_tab_index {
                tab.has_unread = true;
                tab.unread_count += 1;
            }
        }
    }

    pub fn finish_line_for_tab(&mut self, tab_name: &str, width: u16) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.name == tab_name) {
            tab.window.finish_line(width);
        }
    }

    pub fn get_all_streams(&self) -> Vec<String> {
        self.tabs.iter().map(|t| t.stream.clone()).collect()
    }

    pub fn clear_stream(&mut self, stream: &str) {
        for tab in &mut self.tabs {
            if tab.stream == stream {
                tab.window.clear();
            }
        }
    }

    pub fn scroll_up(&mut self, amount: usize) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.window.scroll_up(amount);
        }
    }

    pub fn scroll_down(&mut self, amount: usize) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.window.scroll_down(amount);
        }
    }

    pub fn start_search(&mut self, pattern: &str) -> Result<usize, regex::Error> {
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.window.start_search(pattern)
        } else {
            Ok(0)
        }
    }

    pub fn clear_search(&mut self) {
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.window.clear_search();
        }
    }

    pub fn next_match(&mut self) -> bool {
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.window.next_match()
        } else {
            false
        }
    }

    pub fn prev_match(&mut self) -> bool {
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.window.prev_match()
        } else {
            false
        }
    }

    pub fn search_info(&self) -> Option<(usize, usize)> {
        self.tabs
            .get(self.active_tab_index)
            .and_then(|tab| tab.window.search_info())
    }

    pub fn set_border_config(&mut self, show: bool, style: Option<String>, color: Option<String>) {
        self.show_border = show;
        self.border_style = style;
        self.border_color = color;
    }

    pub fn set_border_sides(&mut self, sides: crate::config::BorderSides) {
        self.border_sides = sides;
    }

    pub fn set_transparent_background(&mut self, transparent: bool) {
        self.transparent_background = transparent;
    }

    pub fn set_background_color(&mut self, color: Option<String>) {
        self.background_color = color;
    }

    pub fn apply_window_colors(
        &mut self,
        text_color: Option<String>,
        background_color: Option<String>,
    ) {
        self.content_text_color = text_color.clone();
        self.background_color = background_color.clone();

        for tab in &mut self.tabs {
            tab.window.set_text_color(text_color.clone());
            tab.window.set_background_color(background_color.clone());
        }
    }

    pub fn get_tab_at_position(&self, x: u16, tab_bar_rect: Rect) -> Option<usize> {
        let mut curr_x = tab_bar_rect.x;

        for (idx, tab) in self.tabs.iter().enumerate() {
            let tab_text = if idx == self.active_tab_index {
                tab.name.clone()
            } else if tab.has_unread {
                format!("{}{}", self.tab_unread_prefix, tab.name)
            } else {
                tab.name.clone()
            };

            let tab_width = tab_text.chars().count() as u16;
            let divider_width = if idx < self.tabs.len() - 1 { 3 } else { 0 }; // " | "

            if x >= curr_x && x < curr_x + tab_width + divider_width {
                return Some(idx);
            }

            curr_x += tab_width + divider_width;
        }

        None
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

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        if self.tabs.is_empty() {
            return;
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

            if !self.title.is_empty() {
                block = block.title(self.title.clone());
            }

            let inner = block.inner(area);
            block.render(area, buf);
            inner
        } else {
            area
        };

        // Split inner area for tab bar and content
        let (tab_bar_area, content_area) = match self.tab_bar_position {
            TabBarPosition::Top => {
                let tab_bar = Rect {
                    x: inner_area.x,
                    y: inner_area.y,
                    width: inner_area.width,
                    height: 1,
                };
                let content = Rect {
                    x: inner_area.x,
                    y: inner_area.y + 1,
                    width: inner_area.width,
                    height: inner_area.height.saturating_sub(1),
                };
                (tab_bar, content)
            }
            TabBarPosition::Bottom => {
                let content = Rect {
                    x: inner_area.x,
                    y: inner_area.y,
                    width: inner_area.width,
                    height: inner_area.height.saturating_sub(1),
                };
                let tab_bar = Rect {
                    x: inner_area.x,
                    y: inner_area.y + content.height,
                    width: inner_area.width,
                    height: 1,
                };
                (tab_bar, content)
            }
        };

        // Render tab bar
        self.render_tab_bar(tab_bar_area, buf);

        // Render active tab content
        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
            tab.window.render(content_area, buf);
        }
    }

    fn render_tab_bar(&self, area: Rect, buf: &mut Buffer) {
        let active_color = self
            .tab_active_color
            .as_ref()
            .map(|c| Self::parse_color(c))
            .unwrap_or(Color::Yellow);
        let inactive_color = self
            .tab_inactive_color
            .as_ref()
            .map(|c| Self::parse_color(c))
            .unwrap_or(Color::DarkGray);
        let unread_color = self
            .tab_unread_color
            .as_ref()
            .map(|c| Self::parse_color(c))
            .unwrap_or(Color::White);

        let mut x = area.x;

        for (idx, tab) in self.tabs.iter().enumerate() {
            if x >= area.right() {
                break;
            }

            // Determine tab text and style
            let (tab_text, style) = if idx == self.active_tab_index {
                (
                    tab.name.clone(),
                    Style::default()
                        .fg(active_color)
                        .add_modifier(Modifier::BOLD),
                )
            } else if tab.has_unread {
                (
                    format!("{}{}", self.tab_unread_prefix, tab.name),
                    Style::default().fg(unread_color),
                )
            } else {
                (tab.name.clone(), Style::default().fg(inactive_color))
            };

            // Render tab text
            for ch in tab_text.chars() {
                if x >= area.right() {
                    break;
                }
                buf.get_mut(x, area.y).set_char(ch).set_style(style);
                x += 1;
            }

            // Render divider if not last tab
            if idx < self.tabs.len() - 1 {
                let divider = " | ";
                for ch in divider.chars() {
                    if x >= area.right() {
                        break;
                    }
                    buf.get_mut(x, area.y)
                        .set_char(ch)
                        .set_style(Style::default().fg(inactive_color));
                    x += 1;
                }
            }
        }
    }
}
