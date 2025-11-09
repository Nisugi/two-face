use crate::ui::{TextSegment, StyledText};
use std::collections::VecDeque;

/// Text window state (rendering-agnostic)
///
/// Holds text content, scroll position, and other state for a text window.
/// Can be rendered by TUI (ratatui) or GUI (egui) frontends.
///
/// TODO: Migrate full implementation from src/ui/text_window.rs
pub struct TextWindowState {
    /// Logical lines (before wrapping)
    pub lines: VecDeque<Vec<TextSegment>>,

    /// Maximum number of lines to keep in buffer
    pub max_lines: usize,

    /// Scroll offset from bottom (0 = live view, >0 = scrolled back)
    pub scroll_offset: usize,

    /// Absolute scroll position (None = following live)
    pub scroll_position: Option<usize>,

    /// Accumulator for current line being built
    pub(crate) current_line: Vec<TextSegment>,

    /// Whether timestamps should be shown
    pub show_timestamps: bool,
}

impl TextWindowState {
    /// Create a new text window state
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            max_lines,
            scroll_offset: 0,
            scroll_position: None,
            current_line: Vec::new(),
            show_timestamps: false,
        }
    }

    /// Add styled text to the current line
    ///
    /// Note: For now, this is a placeholder. Real implementation will come from
    /// migrating the full TextWindow logic.
    pub fn add_text_segment(&mut self, segment: TextSegment) {
        self.current_line.push(segment);
    }

    /// Add a complete styled text segment
    pub fn add_segment(&mut self, segment: TextSegment) {
        self.current_line.push(segment);
    }

    /// Finish the current line and add it to the buffer
    pub fn finish_line(&mut self) {
        let line = std::mem::take(&mut self.current_line);
        self.lines.push_back(line);

        // Trim to max_lines
        while self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
    }

    /// Scroll up by N lines
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_add(n);
        self.scroll_position = Some(self.lines.len().saturating_sub(self.scroll_offset));
    }

    /// Scroll down by N lines
    pub fn scroll_down(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
        if self.scroll_offset == 0 {
            self.scroll_position = None; // Return to live view
        } else {
            self.scroll_position = Some(self.lines.len().saturating_sub(self.scroll_offset));
        }
    }

    /// Scroll to bottom (live view)
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
        self.scroll_position = None;
    }

    /// Get visible lines for rendering
    pub fn get_visible_lines(&self, height: usize) -> &[Vec<TextSegment>] {
        let total = self.lines.len();
        if total == 0 {
            return &[];
        }

        let start = if let Some(pos) = self.scroll_position {
            pos.saturating_sub(height).min(total.saturating_sub(height))
        } else {
            total.saturating_sub(height)
        };

        let end = start + height.min(total);
        let start = start.min(total);
        let end = end.min(total);

        if start >= end {
            &[]
        } else {
            &self.lines.as_slices().0[start..end]
        }
    }

    /// Clear all lines
    pub fn clear(&mut self) {
        self.lines.clear();
        self.current_line.clear();
        self.scroll_to_bottom();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_window_basic() {
        let mut tw = TextWindowState::new(100);
        assert_eq!(tw.lines.len(), 0);

        tw.add_text("Hello", None, None, false);
        tw.finish_line();
        assert_eq!(tw.lines.len(), 1);

        tw.scroll_up(1);
        assert_eq!(tw.scroll_offset, 1);

        tw.scroll_to_bottom();
        assert_eq!(tw.scroll_offset, 0);
        assert_eq!(tw.scroll_position, None);
    }
}
