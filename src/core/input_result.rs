//! Unified enum describing what should happen after a widget handles input.
//!
//! This keeps the widget manager agnostic of the specific dialog being shown;
//! it simply matches on `InputResult` and applies the requested side effects.

use crate::config::{HighlightPattern, KeyBindAction, PaletteColor, SpellColorRange};
use crate::data::ui_state::InputMode;

/// Result of handling input in a menu/widget
#[derive(Debug)]
pub enum InputResult {
    /// Continue - keep widget open, no state change
    Continue,

    /// Close the widget, return to normal mode
    Close,

    /// Save highlight and close
    SaveHighlight {
        name: String,
        pattern: HighlightPattern,
    },

    /// Delete highlight and refresh browser
    DeleteHighlight { name: String },

    /// Open highlight form for editing
    EditHighlight { name: String },

    /// Save keybind and close
    SaveKeybind {
        combo: String,
        action: KeyBindAction,
    },

    /// Delete keybind and refresh browser
    DeleteKeybind { combo: String },

    /// Edit keybind
    EditKeybind { combo: String },

    /// Save palette color and close
    SavePaletteColor { color: PaletteColor },

    /// Delete palette color and refresh browser
    DeletePaletteColor { name: String },

    /// Edit palette color
    EditPaletteColor { name: String },

    /// Save spell color and close
    SaveSpellColor { color: SpellColorRange },

    /// Delete spell color and refresh browser
    DeleteSpellColor { index: usize },

    /// Edit spell color
    EditSpellColor { index: usize },

    /// Save UI color and close
    SaveUIColor { key: String, value: String },

    /// Save setting and continue (settings editor)
    SaveSetting { key: String, value: String },

    /// Save window definition and close
    SaveWindow {
        window_def: crate::config::WindowDef,
    },

    /// Delete window and close editor
    DeleteWindow { name: String },

    /// Open a sub-widget (transition to different mode)
    OpenSubWidget {
        mode: InputMode,
        data: Option<String>, // Optional data to pass (e.g., item name to edit)
    },
}

impl InputResult {
    /// Helper to create a Close result
    pub fn close() -> Self {
        InputResult::Close
    }

    /// Helper to create a Continue result
    pub fn continue_editing() -> Self {
        InputResult::Continue
    }

    /// Check if this result closes the widget
    pub fn is_closing(&self) -> bool {
        matches!(
            self,
            InputResult::Close
                | InputResult::SaveHighlight { .. }
                | InputResult::SaveKeybind { .. }
                | InputResult::SavePaletteColor { .. }
                | InputResult::SaveSpellColor { .. }
                | InputResult::SaveUIColor { .. }
                | InputResult::SaveWindow { .. }
                | InputResult::DeleteWindow { .. }
        )
    }

    /// Check if this result opens another widget
    pub fn is_transition(&self) -> bool {
        matches!(
            self,
            InputResult::EditHighlight { .. }
                | InputResult::EditKeybind { .. }
                | InputResult::EditPaletteColor { .. }
                | InputResult::EditSpellColor { .. }
                | InputResult::OpenSubWidget { .. }
        )
    }
}
