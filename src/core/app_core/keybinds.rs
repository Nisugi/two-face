use std::collections::HashMap;

use anyhow::Result;

use crate::config::{Config, KeyAction, KeyBindAction};
use crate::frontend::common::KeyEvent;

use super::AppCore;

impl AppCore {
    /// Build runtime keybind map from config for fast O(1) lookups
    /// Converts string-based keybinds (e.g., "num_0", "Ctrl+s") to KeyEvent structs
    pub(super) fn build_keybind_map(config: &Config) -> HashMap<KeyEvent, KeyBindAction> {
        let mut map = HashMap::new();

        for (key_string, action) in &config.keybinds {
            // Parse the key string into a (KeyCode, KeyModifiers) tuple
            if let Some((code, modifiers)) = crate::config::parse_key_string(key_string) {
                // Create a KeyEvent from the parsed code and modifiers
                let key_event = KeyEvent { code, modifiers };
                map.insert(key_event, action.clone());
            } else {
                tracing::warn!("Failed to parse keybind string: '{}'", key_string);
            }
        }

        tracing::debug!("Built keybind map with {} entries", map.len());
        map
    }

    /// Rebuild the keybind map (call after config changes)
    pub fn rebuild_keybind_map(&mut self) {
        self.keybind_map = Self::build_keybind_map(&self.config);
    }

    /// Execute a keybind action (called when a bound key is pressed)
    /// Returns a list of commands to send to the server (for macros)
    pub fn execute_keybind_action(&mut self, action: &KeyBindAction) -> Result<Vec<String>> {
        match action {
            KeyBindAction::Action(action_str) => {
                // Parse the action string to a KeyAction
                if let Some(key_action) = KeyAction::from_str(action_str) {
                    self.execute_key_action(key_action)?;
                } else {
                    tracing::warn!("Unknown keybind action: '{}'", action_str);
                }
                Ok(vec![]) // Actions don't send commands to server
            }
            KeyBindAction::Macro(macro_action) => {
                // Strip any trailing \r or \n from macro text (legacy from wrayth-style macros)
                // These control characters corrupt the StyledLine and cause rendering artifacts
                let clean_text =
                    macro_action.macro_text.trim_end_matches(&['\r', '\n'][..]).to_string();

                tracing::info!(
                    "[MACRO] Executing macro: '{}' (raw: '{}')",
                    clean_text,
                    macro_action.macro_text
                );

                // Send the macro text as a command (posts prompt+echo, returns command for server)
                let command = self.send_command(clean_text)?;
                tracing::info!("[MACRO] send_command returned: '{}'", command);
                Ok(vec![command]) // Return command for network layer to send
            }
        }
    }

    /// Execute a KeyAction (dispatch to the appropriate method)
    fn execute_key_action(&mut self, action: KeyAction) -> Result<()> {
        match action {
            // Command input actions - now handled by CommandInput widget
            KeyAction::SendCommand
            | KeyAction::CursorLeft
            | KeyAction::CursorRight
            | KeyAction::CursorWordLeft
            | KeyAction::CursorWordRight
            | KeyAction::CursorHome
            | KeyAction::CursorEnd
            | KeyAction::CursorBackspace
            | KeyAction::CursorDelete
            | KeyAction::CursorDeleteWord
            | KeyAction::CursorClearLine
            | KeyAction::PreviousCommand
            | KeyAction::NextCommand
            | KeyAction::SendLastCommand
            | KeyAction::SendSecondLastCommand
            | KeyAction::Copy
            | KeyAction::Paste
            | KeyAction::SelectAll => {
                // These actions are now handled by the CommandInput widget
                // via frontend.command_input_key() in main.rs
                // If we get here, it means the routing logic in main.rs missed something
                tracing::warn!(
                    "Command input action {:?} reached execute_key_action - should be routed to widget",
                    action
                );
            }

            // Window actions
            KeyAction::SwitchCurrentWindow => {
                // TODO: Implement window switching logic
                tracing::debug!("SwitchCurrentWindow not yet implemented");
            }
            KeyAction::ScrollCurrentWindowUpOne => self.scroll_current_window_up_one(),
            KeyAction::ScrollCurrentWindowDownOne => self.scroll_current_window_down_one(),
            KeyAction::ScrollCurrentWindowUpPage => self.scroll_current_window_up_page(),
            KeyAction::ScrollCurrentWindowDownPage => self.scroll_current_window_down_page(),
            KeyAction::ScrollCurrentWindowHome => self.scroll_current_window_home(),
            KeyAction::ScrollCurrentWindowEnd => self.scroll_current_window_end(),

            // Search actions (already implemented elsewhere)
            KeyAction::StartSearch => {
                // TODO: Set input mode to Search
                tracing::debug!("StartSearch should be handled by input mode change");
            }
            KeyAction::NextSearchMatch => {
                // TODO: Implement search navigation
                tracing::debug!("NextSearchMatch not yet implemented");
            }
            KeyAction::PrevSearchMatch => {
                // TODO: Implement search navigation
                tracing::debug!("PrevSearchMatch not yet implemented");
            }
            KeyAction::ClearSearch => {
                // TODO: Implement search clearing
                tracing::debug!("ClearSearch not yet implemented");
            }

            // Tab navigation actions - need to be handled in main.rs (require frontend access)
            KeyAction::NextTab | KeyAction::PrevTab | KeyAction::NextUnreadTab => {
                // These actions must be routed to frontend in main.rs
                // execute_key_action doesn't have frontend access
                tracing::warn!(
                    "Tab navigation action {:?} reached execute_key_action - should be routed to frontend",
                    action
                );
            }

            // System toggles
            KeyAction::TogglePerformanceStats => {
                self.config.ui.performance_stats_enabled = !self.config.ui.performance_stats_enabled;
                let status = if self.config.ui.performance_stats_enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                self.add_system_message(&format!("Performance overlay {}", status));
                tracing::info!("Performance stats overlay toggled: {}", status);
            }
            KeyAction::ToggleIgnores => {
                self.config.ui.ignores_enabled = !self.config.ui.ignores_enabled;
                let status = if self.config.ui.ignores_enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                self.add_system_message(&format!("Squelch patterns {}", status));
                tracing::info!("Squelch patterns toggled: {}", status);
            }
            KeyAction::ToggleSounds => {
                self.config.sound.enabled = !self.config.sound.enabled;
                let status = if self.config.sound.enabled {
                    "enabled"
                } else {
                    "disabled"
                };
                self.add_system_message(&format!("Sound system {}", status));
                tracing::info!("Sound system toggled: {}", status);
            }

            // TTS (Text-to-Speech) actions - Accessibility
            KeyAction::TtsNext => {
                if let Err(e) = self.tts_manager.speak_next() {
                    tracing::warn!("TTS speak_next failed: {}", e);
                }
            }
            KeyAction::TtsPrevious => {
                if let Err(e) = self.tts_manager.speak_previous() {
                    tracing::warn!("TTS speak_previous failed: {}", e);
                }
            }
            KeyAction::TtsNextUnread => {
                if let Err(e) = self.tts_manager.speak_next_unread() {
                    tracing::warn!("TTS speak_next_unread failed: {}", e);
                }
            }
            KeyAction::TtsStop => {
                if let Err(e) = self.tts_manager.stop() {
                    tracing::warn!("TTS stop failed: {}", e);
                }
            }
            KeyAction::TtsMuteToggle => {
                self.tts_manager.toggle_mute();
                let status = if self.tts_manager.is_muted() { "muted" } else { "unmuted" };
                self.add_system_message(&format!("TTS {}", status));
            }
            KeyAction::TtsIncreaseRate => {
                if let Err(e) = self.tts_manager.increase_rate() {
                    tracing::warn!("TTS increase_rate failed: {}", e);
                } else {
                    self.add_system_message("TTS rate increased");
                }
            }
            KeyAction::TtsDecreaseRate => {
                if let Err(e) = self.tts_manager.decrease_rate() {
                    tracing::warn!("TTS decrease_rate failed: {}", e);
                } else {
                    self.add_system_message("TTS rate decreased");
                }
            }
            KeyAction::TtsIncreaseVolume => {
                if let Err(e) = self.tts_manager.increase_volume() {
                    tracing::warn!("TTS increase_volume failed: {}", e);
                } else {
                    self.add_system_message("TTS volume increased");
                }
            }
            KeyAction::TtsDecreaseVolume => {
                if let Err(e) = self.tts_manager.decrease_volume() {
                    tracing::warn!("TTS decrease_volume failed: {}", e);
                } else {
                    self.add_system_message("TTS volume decreased");
                }
            }

            // Macro actions (should not reach here - handled by execute_keybind_action)
            KeyAction::SendMacro(text) => {
                self.send_command(text)?;
            }
        }

        Ok(())
    }
}
