//! Input routing for menu widgets
//!
//! Routes keyboard input to the appropriate MenuAction based on:
//! - Current InputMode (which widget has focus)
//! - Menu keybinds configuration
//! - Widget context (browser vs form vs editor)

use crate::config::{Config, MenuKeybinds};
use crate::core::menu_actions::{key_event_to_string, ActionContext, MenuAction};
use crate::data::ui_state::InputMode;
use crossterm::event::KeyEvent;

/// Route a key event to a MenuAction based on current context
pub fn route_input(key: KeyEvent, mode: &InputMode, config: &Config) -> MenuAction {
    // Determine the action context based on InputMode
    let context = get_action_context(mode);

    // Resolve the key to an action using menu keybinds
    config.menu_keybinds.resolve_action(key, context)
}

/// Map InputMode to ActionContext for keybind resolution
fn get_action_context(mode: &InputMode) -> ActionContext {
    match mode {
        // Browser widgets
        InputMode::HighlightBrowser
        | InputMode::KeybindBrowser
        | InputMode::ColorPaletteBrowser
        | InputMode::SpellColorsBrowser
        | InputMode::UIColorsBrowser
        | InputMode::ThemeBrowser => ActionContext::Browser,

        // Form widgets
        InputMode::HighlightForm
        | InputMode::KeybindForm
        | InputMode::ColorForm
        | InputMode::SpellColorForm
        | InputMode::ThemeEditor => ActionContext::Form,

        // Settings editor (hybrid - has both navigation and inline editing)
        InputMode::SettingsEditor => ActionContext::SettingsEditor,

        // Window editor (most complex - has navigation, inline editing, reordering)
        InputMode::WindowEditor => ActionContext::WindowEditor,

        // Normal modes - should not route through menu system
        InputMode::Normal
        | InputMode::Navigation
        | InputMode::History
        | InputMode::Search
        | InputMode::Menu => ActionContext::Browser, // Fallback (shouldn't be called)
    }
}

/// Check if the current InputMode should use menu keybinds
pub fn should_use_menu_keybinds(mode: &InputMode) -> bool {
    !matches!(mode, InputMode::Normal | InputMode::Navigation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyModifiers};

    #[test]
    fn test_context_mapping() {
        assert!(matches!(
            get_action_context(&InputMode::HighlightBrowser),
            ActionContext::Browser
        ));

        assert!(matches!(
            get_action_context(&InputMode::HighlightForm),
            ActionContext::Form
        ));

        assert!(matches!(
            get_action_context(&InputMode::SettingsEditor),
            ActionContext::SettingsEditor
        ));
    }

    #[test]
    fn test_menu_keybind_filtering() {
        assert!(!should_use_menu_keybinds(&InputMode::Normal));
        assert!(!should_use_menu_keybinds(&InputMode::Navigation));
        assert!(should_use_menu_keybinds(&InputMode::HighlightBrowser));
        assert!(should_use_menu_keybinds(&InputMode::WindowEditor));
    }
}
