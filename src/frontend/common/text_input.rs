//! Frontend-agnostic text input handling.
//!
//! Provides a trait-based abstraction over text editing backends (tui-textarea for TUI,
//! native widgets for GUI) to enable shared text editing logic across frontends.

use super::input::{KeyCode, KeyModifiers};

/// Cursor movement operations for text editing
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorMove {
    /// Move to beginning of current line
    Head,
    /// Move to end of current line
    End,
    /// Move to beginning of document
    Top,
    /// Move to end of document
    Bottom,
    /// Move forward one character
    Forward,
    /// Move backward one character
    Back,
    /// Move up one line
    Up,
    /// Move down one line
    Down,
    /// Move forward one word
    WordForward,
    /// Move backward one word
    WordBack,
    /// Move to previous paragraph
    ParagraphBack,
    /// Move to next paragraph
    ParagraphForward,
    /// Jump to specific (row, col) position
    Jump(u16, u16),
}

/// Frontend-agnostic text input field interface.
///
/// This trait abstracts over different text editing backends:
/// - TUI: `tui-textarea::TextArea`
/// - GUI: Native text widgets (egui::TextEdit, iced::TextInput, etc.)
pub trait TextInput {
    /// Get the current text content
    fn text(&self) -> String;

    /// Get lines of text (for multi-line editors)
    fn lines(&self) -> Vec<String>;

    /// Set the entire text content
    fn set_text(&mut self, text: String);

    /// Insert a character at the current cursor position
    fn insert_char(&mut self, c: char);

    /// Insert a string at the current cursor position
    fn insert_str(&mut self, s: &str);

    /// Delete the character before the cursor (backspace)
    fn delete_char(&mut self);

    /// Delete the character at the cursor (delete key)
    fn delete_forward_char(&mut self);

    /// Delete from cursor to end of line
    fn delete_line_by_end(&mut self);

    /// Delete from cursor to beginning of line
    fn delete_line_by_head(&mut self);

    /// Delete the entire current line
    fn delete_current_line(&mut self);

    /// Delete the next word
    fn delete_next_word(&mut self);

    /// Delete the previous word
    fn delete_prev_word(&mut self);

    /// Move the cursor
    fn move_cursor(&mut self, mv: CursorMove);

    /// Get current cursor position (row, col)
    fn cursor_position(&self) -> (u16, u16);

    /// Start text selection at current cursor
    fn start_selection(&mut self);

    /// Cancel current selection
    fn cancel_selection(&mut self);

    /// Check if text is currently selected
    fn has_selection(&self) -> bool;

    /// Get selected text (empty string if no selection)
    fn selected_text(&self) -> String;

    /// Get yanked text (clipboard-like internal buffer)
    fn yank_text(&self) -> String;

    /// Set yanked text (clipboard-like internal buffer)
    fn set_yank_text(&mut self, text: String);

    /// Clear all text
    fn clear(&mut self);

    /// Check if the field is empty
    fn is_empty(&self) -> bool {
        self.text().is_empty()
    }

    /// Get the number of lines
    fn line_count(&self) -> usize {
        self.lines().len()
    }
}

/// Frontend-agnostic text editing operations (high-level actions).
///
/// These operations build on top of TextInput to provide common editing workflows
/// like select-all, copy, cut, paste that work across any frontend.
pub trait TextEditor: TextInput {
    /// Select all text in the field (Ctrl+A)
    fn select_all(&mut self) {
        self.move_cursor(CursorMove::End);
        self.start_selection();
        self.move_cursor(CursorMove::Head);
    }

    /// Copy selected text to internal buffer (like clipboard)
    fn copy(&mut self) -> String {
        let text = self.selected_text();
        if !text.is_empty() {
            self.set_yank_text(text.clone());
        }
        text
    }

    /// Cut selected text to internal buffer
    fn cut(&mut self) -> String {
        let text = self.copy();
        if !text.is_empty() {
            self.insert_str(""); // Replace selection with empty string
        }
        text
    }

    /// Paste text from internal buffer
    fn paste(&mut self, text: &str) {
        self.insert_str(text);
    }

    /// Undo last operation (if supported by backend)
    fn undo(&mut self) {
        // Default: no-op (backends can override)
    }

    /// Redo last undone operation (if supported by backend)
    fn redo(&mut self) {
        // Default: no-op (backends can override)
    }
}

/// Auto-implement TextEditor for any type that implements TextInput
impl<T: TextInput> TextEditor for T {}

/// Helper to convert frontend-agnostic KeyCode to text input operations
pub fn handle_text_input_key(
    input: &mut dyn TextInput,
    key: KeyCode,
    modifiers: KeyModifiers,
) -> bool {
    match (key, modifiers.ctrl, modifiers.shift, modifiers.alt) {
        // Character input
        (KeyCode::Char(c), false, _, false) => {
            input.insert_char(c);
            true
        }

        // Ctrl+A: Select all
        (KeyCode::Char('a'), true, _, _) => {
            input.move_cursor(CursorMove::End);
            input.start_selection();
            input.move_cursor(CursorMove::Head);
            true
        }

        // Movement
        (KeyCode::Left, false, false, false) => {
            input.move_cursor(CursorMove::Back);
            true
        }
        (KeyCode::Right, false, false, false) => {
            input.move_cursor(CursorMove::Forward);
            true
        }
        (KeyCode::Up, false, false, false) => {
            input.move_cursor(CursorMove::Up);
            true
        }
        (KeyCode::Down, false, false, false) => {
            input.move_cursor(CursorMove::Down);
            true
        }
        (KeyCode::Home, false, false, false) => {
            input.move_cursor(CursorMove::Head);
            true
        }
        (KeyCode::End, false, false, false) => {
            input.move_cursor(CursorMove::End);
            true
        }

        // Word movement (Ctrl+Arrow)
        (KeyCode::Left, true, false, false) => {
            input.move_cursor(CursorMove::WordBack);
            true
        }
        (KeyCode::Right, true, false, false) => {
            input.move_cursor(CursorMove::WordForward);
            true
        }

        // Deletion
        (KeyCode::Backspace, false, false, false) => {
            input.delete_char();
            true
        }
        (KeyCode::Delete, false, false, false) => {
            input.delete_forward_char();
            true
        }
        (KeyCode::Backspace, true, false, false) => {
            input.delete_prev_word();
            true
        }
        (KeyCode::Delete, true, false, false) => {
            input.delete_next_word();
            true
        }

        // Ctrl+U: Delete to line start
        (KeyCode::Char('u'), true, false, false) => {
            input.delete_line_by_head();
            true
        }

        // Ctrl+K: Delete to line end
        (KeyCode::Char('k'), true, false, false) => {
            input.delete_line_by_end();
            true
        }

        _ => false,
    }
}
