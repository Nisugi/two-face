//! QuickBar widget - scrollable container with clickable links
//!
//! Displays one or more rows of clickable links that wrap intelligently,
//! preserving complete link boundaries. Supports multiple bar variations
//! (quick, quick-combat, quick-simu) and custom script-controlled content.

use crate::config::QuickBarWidgetData;
use crate::frontend::tui::text_window::{LinkData, SpanType, StyledText};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Widget},
};
use std::collections::HashMap;

/// Processed QuickBar state ready for rendering
pub struct QuickBar {
    /// Raw widget data from config
    pub data: QuickBarWidgetData,

    /// Wrapped lines (each line contains styled spans)
    pub wrapped_lines: Vec<Vec<StyledText>>,

    /// Link map for click handling: (row, col) -> LinkData
    pub link_map: HashMap<(u16, u16), LinkData>,

    /// Total number of wrapped lines (for scroll bounds)
    pub total_lines: usize,
}

impl QuickBar {
    /// Create a new QuickBar from widget data
    pub fn new(data: QuickBarWidgetData) -> Self {
        Self {
            data,
            wrapped_lines: vec![],
            link_map: HashMap::new(),
            total_lines: 0,
        }
    }

    /// Update QuickBar content from raw text
    /// This parses links and performs word wrap
    pub fn set_content(&mut self, content: &str, available_width: usize) {
        // Parse content into StyledText spans
        let spans = Self::parse_content(content);

        // Perform link-aware word wrap
        self.wrapped_lines = Self::wrap_content(&spans, available_width);
        self.total_lines = self.wrapped_lines.len();

        // Reset scroll to top
        self.data.scroll_offset = 0;

        // Rebuild link map
        self.rebuild_link_map();
    }

    /// Parse raw QuickBar content into StyledText spans
    /// Format: "[Link Text]" or plain text
    fn parse_content(content: &str) -> Vec<StyledText> {
        let mut spans = Vec::new();
        let mut current_pos = 0;

        // Simple regex-free parser for [Link] format
        while current_pos < content.len() {
            if let Some(start_bracket) = content[current_pos..].find('[') {
                let abs_start = current_pos + start_bracket;

                // Add any plain text before the bracket
                if start_bracket > 0 {
                    let plain_text = &content[current_pos..abs_start];
                    if !plain_text.is_empty() {
                        spans.push(StyledText {
                            content: plain_text.to_string(),
                            fg: None,
                            bg: None,
                            bold: false,
                            span_type: SpanType::Normal,
                            link_data: None,
                        });
                    }
                }

                // Find closing bracket
                if let Some(end_bracket) = content[abs_start + 1..].find(']') {
                    let abs_end = abs_start + 1 + end_bracket;
                    let link_text = &content[abs_start + 1..abs_end];

                    // Create link span
                    spans.push(StyledText {
                        content: format!("[{}]", link_text),
                        fg: Some(Color::Cyan), // Links are cyan
                        bg: None,
                        bold: false,
                        span_type: SpanType::Link,
                        link_data: Some(LinkData {
                            exist_id: String::new(), // QuickBar links don't use exist_id
                            noun: link_text.to_string(),
                            text: link_text.to_string(),
                            coord: None,
                        }),
                    });

                    current_pos = abs_end + 1;
                } else {
                    // No closing bracket, treat as plain text
                    spans.push(StyledText {
                        content: content[abs_start..].to_string(),
                        fg: None,
                        bg: None,
                        bold: false,
                        span_type: SpanType::Normal,
                        link_data: None,
                    });
                    break;
                }
            } else {
                // No more brackets, add remaining text
                let remaining = &content[current_pos..];
                if !remaining.is_empty() {
                    spans.push(StyledText {
                        content: remaining.to_string(),
                        fg: None,
                        bg: None,
                        bold: false,
                        span_type: SpanType::Normal,
                        link_data: None,
                    });
                }
                break;
            }
        }

        spans
    }

    /// Perform link-aware word wrap
    /// Links are treated as atomic units and never split across lines
    fn wrap_content(spans: &[StyledText], available_width: usize) -> Vec<Vec<StyledText>> {
        let mut wrapped_lines: Vec<Vec<StyledText>> = vec![];
        let mut current_line: Vec<StyledText> = vec![];
        let mut current_line_width = 0;

        for span in spans {
            let span_width = span.content.len();

            // Check if span fits on current line
            if current_line_width + span_width <= available_width {
                // Fits! Add to current line
                current_line.push(span.clone());
                current_line_width += span_width;
            } else {
                // Doesn't fit
                if span.span_type == SpanType::Link {
                    // Link: never split, move entire link to next line
                    if !current_line.is_empty() {
                        wrapped_lines.push(current_line);
                        current_line = vec![];
                        current_line_width = 0;
                    }

                    // If link is too wide for even an empty line, it will overflow
                    // For now, just add it anyway (will be truncated during render)
                    current_line.push(span.clone());
                    current_line_width = span_width;
                } else {
                    // Plain text: can split on word boundaries
                    let words: Vec<&str> = span.content.split_whitespace().collect();

                    for word in words {
                        let word_width = word.len() + 1; // +1 for space

                        if current_line_width + word_width <= available_width {
                            // Word fits
                            current_line.push(StyledText {
                                content: format!("{} ", word),
                                ..span.clone()
                            });
                            current_line_width += word_width;
                        } else {
                            // Word doesn't fit, wrap to next line
                            if !current_line.is_empty() {
                                wrapped_lines.push(current_line);
                                current_line = vec![];
                                current_line_width = 0;
                            }

                            current_line.push(StyledText {
                                content: format!("{} ", word),
                                ..span.clone()
                            });
                            current_line_width = word_width;
                        }
                    }
                }
            }
        }

        // Add last line if not empty
        if !current_line.is_empty() {
            wrapped_lines.push(current_line);
        }

        // Return at least one empty line if no content
        if wrapped_lines.is_empty() {
            wrapped_lines.push(vec![]);
        }

        wrapped_lines
    }

    /// Rebuild link map with screen coordinates
    fn rebuild_link_map(&mut self) {
        self.link_map.clear();

        for (row_idx, line) in self.wrapped_lines.iter().enumerate() {
            let mut col_offset = 0;

            for span in line {
                if span.span_type == SpanType::Link {
                    if let Some(ref link_data) = span.link_data {
                        // Map every column position within the link
                        for i in 0..span.content.len() {
                            self.link_map.insert(
                                (row_idx as u16, (col_offset + i) as u16),
                                link_data.clone(),
                            );
                        }
                    }
                }
                col_offset += span.content.len();
            }
        }
    }

    /// Scroll up by one row
    pub fn scroll_up(&mut self) {
        self.data.scroll_offset = self.data.scroll_offset.saturating_sub(1);
    }

    /// Scroll down by one row
    pub fn scroll_down(&mut self, visible_rows: usize) {
        let max_scroll = self.total_lines.saturating_sub(visible_rows);
        if self.data.scroll_offset < max_scroll {
            self.data.scroll_offset += 1;
        }
    }

    /// Get link at screen coordinates (relative to widget, not scroll)
    pub fn get_link_at(&self, row: u16, col: u16) -> Option<&LinkData> {
        // Adjust for scroll offset
        let actual_row = row as usize + self.data.scroll_offset;
        self.link_map.get(&(actual_row as u16, col))
    }

    /// Render the QuickBar widget
    pub fn render(&self, area: Rect, buf: &mut Buffer, border_style: Option<Style>) {
        // Calculate visible area (accounting for border if present)
        let inner_area = if border_style.is_some() {
            Rect {
                x: area.x + 1,
                y: area.y + 1,
                width: area.width.saturating_sub(2),
                height: area.height.saturating_sub(2),
            }
        } else {
            area
        };

        // Draw border if provided
        if let Some(style) = border_style {
            let block = Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .border_type(BorderType::Plain)
                .border_style(style);
            block.render(area, buf);
        }

        // Render visible lines
        let visible_rows = inner_area.height as usize;
        let start_line = self.data.scroll_offset;
        let end_line = (start_line + visible_rows).min(self.total_lines);

        for (viewport_row, line_idx) in (start_line..end_line).enumerate() {
            if let Some(line) = self.wrapped_lines.get(line_idx) {
                let mut col_offset = 0;

                for span in line {
                    let style = Style::default()
                        .fg(span.fg.unwrap_or(Color::White))
                        .bg(span.bg.unwrap_or(Color::Reset))
                        .add_modifier(if span.bold {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        });

                    // Render each character of the span
                    for (i, ch) in span.content.chars().enumerate() {
                        let x = inner_area.x + col_offset + i as u16;
                        let y = inner_area.y + viewport_row as u16;

                        if x < inner_area.x + inner_area.width {
                            buf.get_mut(x, y).set_char(ch).set_style(style);
                        }
                    }

                    col_offset += span.content.len() as u16;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_links() {
        let content = "[Go North] [Attack Orc]";
        let spans = QuickBar::parse_content(content);

        assert_eq!(spans.len(), 3); // 2 links + 1 space
        assert_eq!(spans[0].content, "[Go North]");
        assert_eq!(spans[0].span_type, SpanType::Link);
        assert_eq!(spans[2].content, "[Attack Orc]");
        assert_eq!(spans[2].span_type, SpanType::Link);
    }

    #[test]
    fn test_wrap_preserves_links() {
        let spans = vec![
            StyledText {
                content: "[Very Long Link Name]".to_string(),
                fg: Some(Color::Cyan),
                bg: None,
                bold: false,
                span_type: SpanType::Link,
                link_data: None,
            },
            StyledText {
                content: " ".to_string(),
                fg: None,
                bg: None,
                bold: false,
                span_type: SpanType::Normal,
                link_data: None,
            },
            StyledText {
                content: "[Short]".to_string(),
                fg: Some(Color::Cyan),
                bg: None,
                bold: false,
                span_type: SpanType::Link,
                link_data: None,
            },
        ];

        let wrapped = QuickBar::wrap_content(&spans, 20);

        // First link should be on first line
        assert_eq!(wrapped[0][0].content, "[Very Long Link Name]");
        // Second link should be on second line (doesn't fit)
        assert_eq!(wrapped[1][0].content, "[Short]");
    }

    #[test]
    fn test_scroll_bounds() {
        let mut data = QuickBarWidgetData {
            active_bar: "quick".to_string(),
            bars: std::collections::HashMap::new(),
            default_bar: "quick".to_string(),
            scroll_offset: 0,
        };

        let mut quickbar = QuickBar::new(data.clone());
        quickbar.total_lines = 5;

        // Scroll down with 2 visible rows (max scroll = 5 - 2 = 3)
        quickbar.scroll_down(2);
        assert_eq!(quickbar.data.scroll_offset, 1);

        quickbar.scroll_down(2);
        quickbar.scroll_down(2);
        quickbar.scroll_down(2);
        // Should cap at 3
        assert_eq!(quickbar.data.scroll_offset, 3);

        // Scroll up
        quickbar.scroll_up();
        assert_eq!(quickbar.data.scroll_offset, 2);

        // Scroll up past 0 should saturate
        quickbar.data.scroll_offset = 0;
        quickbar.scroll_up();
        assert_eq!(quickbar.data.scroll_offset, 0);
    }
}
