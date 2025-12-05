//! Frontend-agnostic input types for keyboard and mouse events.
//!
//! These types abstract over platform-specific input handling (crossterm for TUI,
//! native events for GUI) to enable shared input processing logic.

/// Represents a key press, independent of the underlying frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    /// A character key (a-z, 0-9, symbols, etc.)
    Char(char),
    /// Backspace key
    Backspace,
    /// Enter/Return key
    Enter,
    /// Left arrow
    Left,
    /// Right arrow
    Right,
    /// Up arrow
    Up,
    /// Down arrow
    Down,
    /// Home key
    Home,
    /// End key
    End,
    /// Page Up
    PageUp,
    /// Page Down
    PageDown,
    /// Tab key
    Tab,
    /// Shift+Tab (reverse tab)
    BackTab,
    /// Delete key
    Delete,
    /// Insert key
    Insert,
    /// Function keys (F1-F12)
    F(u8),
    /// Escape key
    Esc,
    /// Null (no-op)
    Null,
    /// Keypad 0
    Keypad0,
    /// Keypad 1
    Keypad1,
    /// Keypad 2
    Keypad2,
    /// Keypad 3
    Keypad3,
    /// Keypad 4
    Keypad4,
    /// Keypad 5
    Keypad5,
    /// Keypad 6
    Keypad6,
    /// Keypad 7
    Keypad7,
    /// Keypad 8
    Keypad8,
    /// Keypad 9
    Keypad9,
    /// Keypad period/decimal
    KeypadPeriod,
    /// Keypad plus
    KeypadPlus,
    /// Keypad minus
    KeypadMinus,
    /// Keypad multiply/asterisk
    KeypadMultiply,
    /// Keypad divide/slash
    KeypadDivide,
    /// Keypad Enter
    KeypadEnter,
}

/// A keyboard event combining a key code and modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    /// Create a new key event
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Create a key event with no modifiers
    pub fn from_code(code: KeyCode) -> Self {
        Self {
            code,
            modifiers: KeyModifiers::NONE,
        }
    }
}

/// Keyboard modifiers (Ctrl, Shift, Alt).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct KeyModifiers {
    pub ctrl: bool,
    pub shift: bool,
    pub alt: bool,
}

impl KeyModifiers {
    /// No modifiers pressed
    pub const NONE: Self = KeyModifiers {
        ctrl: false,
        shift: false,
        alt: false,
    };

    /// Only Ctrl pressed
    pub const CTRL: Self = KeyModifiers {
        ctrl: true,
        shift: false,
        alt: false,
    };

    /// Only Shift pressed
    pub const SHIFT: Self = KeyModifiers {
        ctrl: false,
        shift: true,
        alt: false,
    };

    /// Only Alt pressed
    pub const ALT: Self = KeyModifiers {
        ctrl: false,
        shift: false,
        alt: true,
    };

    /// Check if any modifiers are active
    pub fn is_empty(&self) -> bool {
        !self.ctrl && !self.shift && !self.alt
    }

    /// Check if Ctrl is pressed (regardless of other modifiers)
    pub fn contains_ctrl(&self) -> bool {
        self.ctrl
    }

    /// Check if Shift is pressed (regardless of other modifiers)
    pub fn contains_shift(&self) -> bool {
        self.shift
    }

    /// Check if Alt is pressed (regardless of other modifiers)
    pub fn contains_alt(&self) -> bool {
        self.alt
    }
}

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Mouse event kinds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventKind {
    /// Mouse button pressed
    Down(MouseButton),
    /// Mouse button released
    Up(MouseButton),
    /// Mouse dragged (button held while moving)
    Drag(MouseButton),
    /// Mouse moved without button pressed
    Moved,
    /// Scroll wheel up
    ScrollUp,
    /// Scroll wheel down
    ScrollDown,
}

/// A mouse event with position and modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseEvent {
    pub kind: MouseEventKind,
    pub column: u16,
    pub row: u16,
    pub modifiers: KeyModifiers,
}

impl MouseEvent {
    /// Create a new mouse event
    pub fn new(kind: MouseEventKind, column: u16, row: u16, modifiers: KeyModifiers) -> Self {
        Self {
            kind,
            column,
            row,
            modifiers,
        }
    }

    /// Get the button if this is a button event
    pub fn button(&self) -> Option<MouseButton> {
        match self.kind {
            MouseEventKind::Down(btn) | MouseEventKind::Up(btn) | MouseEventKind::Drag(btn) => {
                Some(btn)
            }
            _ => None,
        }
    }

    /// Check if this is a left click (down event)
    pub fn is_left_click(&self) -> bool {
        matches!(self.kind, MouseEventKind::Down(MouseButton::Left))
    }

    /// Check if this is a right click (down event)
    pub fn is_right_click(&self) -> bool {
        matches!(self.kind, MouseEventKind::Down(MouseButton::Right))
    }
}
