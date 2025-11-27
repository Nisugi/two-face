//! Shared action vocabulary for the configuration popups and browsers.
//!
//! Translates raw `KeyEvent`s and textual keybinds into semantic `MenuAction`s
//! so every widget can react consistently regardless of current context.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// All possible menu/widget actions
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    // Navigation
    NavigateUp,
    NavigateDown,
    NavigateLeft,
    NavigateRight,
    PageUp,
    PageDown,
    Home,
    End,

    // Field Navigation (forms)
    NextField,
    PreviousField,

    // Selection/Confirmation
    Select, // Enter - select item or accept dropdown
    Cancel, // Esc - close widget

    // Editing
    Save,   // Ctrl+s
    Delete, // Delete key or Ctrl+D

    // Text Editing (always available in TextAreas)
    SelectAll, // Ctrl+A
    Copy,      // Ctrl+C
    Cut,       // Ctrl+X
    Paste,     // Ctrl+V

    // Toggles/Cycling
    Toggle, // Space - toggle boolean

    // Reordering (WindowEditor)
    MoveUp,   // Shift+Up
    MoveDown, // Shift+Down

    // List Management (WindowEditor)
    Add,  // 'A'
    Edit, // 'E'

    // No action (key not bound or not applicable in this context)
    None,
}

/// Context for action resolution - determines which actions are valid
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActionContext {
    Browser,        // In a browser widget (navigate + select/delete)
    Form,           // In a form widget (field nav + save/cancel)
    TextInput,      // Focused on a TextArea field (clipboard ops)
    Dropdown,       // Focused on dropdown field (up/down cycles)
    SettingsEditor, // In settings editor (hybrid navigation/editing)
    WindowEditor,   // In window editor (most complex - all actions)
}

/// Convert KeyEvent to string representation for matching against keybinds
pub fn key_event_to_string(key: KeyEvent) -> String {
    let mut parts = Vec::new();

    // Add modifiers
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if key.modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("Shift");
    }
    if key.modifiers.contains(KeyModifiers::ALT) {
        parts.push("Alt");
    }

    // Add key code
    let key_str = match key.code {
        KeyCode::Char(c) => {
            // For letter keys with Shift+Ctrl/Alt, use uppercase
            if key.modifiers.contains(KeyModifiers::SHIFT) && c.is_ascii_lowercase() {
                c.to_ascii_uppercase().to_string()
            } else {
                c.to_string()
            }
        }
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => {
            // BackTab is usually Shift+Tab, so remove Shift from parts if present
            parts.retain(|p| *p != "Shift");
            "Tab".to_string()
        }
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => return String::new(), // Unrecognized key
    };

    parts.push(&key_str);
    parts.join("+")
}

/// Parse a keybind string into components for comparison
pub fn normalize_keybind(s: &str) -> String {
    // Normalize the keybind string (handle case, order modifiers)
    let parts: Vec<&str> = s.split('+').collect();
    let mut modifiers = Vec::new();
    let mut key = "";

    for part in parts {
        match part.trim() {
            "Ctrl" | "Control" => modifiers.push("Ctrl"),
            "Shift" => modifiers.push("Shift"),
            "Alt" => modifiers.push("Alt"),
            k => key = k,
        }
    }

    // Sort modifiers for consistent comparison
    modifiers.sort();
    modifiers.push(key);
    modifiers.join("+")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_event_to_string() {
        let key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::CONTROL);
        assert_eq!(key_event_to_string(key), "Ctrl+s");

        let key = KeyEvent::new(KeyCode::Up, KeyModifiers::SHIFT);
        assert_eq!(key_event_to_string(key), "Shift+Up");

        let key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        assert_eq!(key_event_to_string(key), "Enter");
    }

    #[test]
    fn test_normalize_keybind() {
        assert_eq!(normalize_keybind("Ctrl+s"), "Ctrl+s");
        assert_eq!(normalize_keybind("Shift+Up"), "Shift+Up");
        assert_eq!(normalize_keybind("Enter"), "Enter");
        assert_eq!(normalize_keybind("Control+A"), "Ctrl+A"); // Normalize Control -> Ctrl
    }
}
