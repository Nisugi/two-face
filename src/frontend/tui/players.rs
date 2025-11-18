//! Scrollable player list derived from the room stream text.
//!
//! Parses the XML-ish player line emitted by Wizard FE and stores each entry in
//! a `ScrollableContainer`, keeping stance/status suffixes intact.

use super::scrollable_container::ScrollableContainer;
use ratatui::{buffer::Buffer, layout::Rect};

pub struct Players {
    container: ScrollableContainer,
    count: u32,
}

impl Players {
    pub fn new(title: &str) -> Self {
        let mut container = ScrollableContainer::new(title);
        // Players widget hides values and percentages by default
        container.set_display_options(false, false);

        Self {
            container,
            count: 0,
        }
    }

    /// Parse players from formatted game text
    /// Format: "<b>[sit] Player1</b>, <b>Player2</b>, <b>[kne] Deddalus</b>"
    pub fn set_players_from_text(&mut self, text: &str) {
        self.container.clear();
        self.count = 0;

        if text.is_empty() {
            self.update_title();
            return;
        }

        // Split by comma to get individual players
        let players: Vec<&str> = text.split(',').map(|s| s.trim()).collect();

        for (idx, player_str) in players.iter().enumerate() {
            if player_str.is_empty() {
                continue;
            }

            // Extract status prefix [xxx] if present
            let status = if let Some(start) = player_str.find('[') {
                if let Some(end) = player_str.find(']') {
                    if end > start {
                        Some(player_str[start..=end].to_string())
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            // Strip all XML tags to get clean player name
            let clean_name = Self::strip_xml_tags(player_str);

            // Remove status prefix from the name if it was at the start
            let clean_name = if let Some(ref s) = status {
                clean_name.trim_start_matches(s).trim().to_string()
            } else {
                clean_name
            };

            if clean_name.is_empty() {
                continue;
            }

            // Create unique ID
            let id = format!("player_{}", idx);

            // Add to container with status as suffix
            self.container.add_or_update_item_full(
                id, clean_name, None,   // no alternate text
                0,      // value (hidden)
                1,      // max (hidden)
                status, // suffix (status like "[sit]")
                None,   // no color override
                None,
            );

            self.count += 1;
        }

        self.update_title();
    }

    /// Strip all XML tags from a string
    fn strip_xml_tags(input: &str) -> String {
        let mut result = String::new();
        let mut in_tag = false;

        for ch in input.chars() {
            match ch {
                '<' => in_tag = true,
                '>' => in_tag = false,
                _ => {
                    if !in_tag {
                        result.push(ch);
                    }
                }
            }
        }

        result.trim().to_string()
    }

    fn update_title(&mut self) {
        let title = format!("Players [{:02}]", self.count);
        self.container.set_title(title);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        self.container.scroll_up(amount);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.container.scroll_down(amount);
    }

    pub fn set_border_config(&mut self, show: bool, style: Option<String>, color: Option<String>) {
        self.container.set_border_config(show, style, color);
    }

    pub fn set_border_sides(&mut self, sides: crate::config::BorderSides) {
        self.container.set_border_sides(sides);
    }

    pub fn set_bar_color(&mut self, color: String) {
        self.container.set_bar_color(color);
    }

    pub fn set_transparent_background(&mut self, transparent: bool) {
        self.container.set_transparent_background(transparent);
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer) {
        self.container.render(area, buf);
    }

    pub fn render_with_focus(&mut self, area: Rect, buf: &mut Buffer, focused: bool) {
        self.container.render_with_focus(area, buf, focused);
    }
}
