use anyhow::Result;

use super::AppCore;

impl AppCore {
    /// Send command to server
    pub fn send_command(&mut self, command: String) -> Result<String> {
        use crate::data::{SpanType, StyledLine, TextSegment, WindowContent};

        // Check for dot commands (local client commands)
        if command.starts_with('.') {
            return self.handle_dot_command(&command);
        }

        // Echo command to main window (prompt + command)
        if !command.is_empty() {
            tracing::info!("[SEND_COMMAND] Echoing command to main window: '{}'", command);
            if let Some(main_window) = self.ui_state.windows.get_mut("main") {
                if let WindowContent::Text(ref mut content) = main_window.content {
                    let mut segments = Vec::new();

                    // Add prompt with per-character coloring (same as prompt rendering)
                    tracing::debug!(
                        "[SEND_COMMAND] Building styled line with prompt: '{}'",
                        self.game_state.last_prompt
                    );
                    for ch in self.game_state.last_prompt.chars() {
                        let char_str = ch.to_string();

                        // Find color for this character in prompt_colors config
                        let color = self
                            .config
                            .colors
                            .prompt_colors
                            .iter()
                            .find(|pc| pc.character == char_str)
                            .and_then(|pc| {
                                // Prefer fg, fallback to color (legacy)
                                pc.fg.as_ref().or(pc.color.as_ref()).cloned()
                            })
                            .unwrap_or_else(|| "#808080".to_string()); // Default dark gray

                        segments.push(TextSegment {
                            text: char_str,
                            fg: Some(color),
                            bg: None,
                            bold: false,
                            span_type: SpanType::Normal,
                            link_data: None,
                        });
                    }

                    // Add the command text (in default color)
                    segments.push(TextSegment {
                        text: command.clone(),
                        fg: Some("#ffffff".to_string()), // White text for command
                        bg: None,
                        bold: false,
                        span_type: SpanType::Normal,
                        link_data: None,
                    });

                    // Add the styled line to the main window
                    content.add_line(StyledLine { segments: segments.clone() });
                    tracing::info!(
                        "[SEND_COMMAND] Added StyledLine with {} segments to main window",
                        segments.len()
                    );
                }
            }
        }

        // Command history is now managed by the CommandInput widget

        // Return formatted command for network layer to send
        Ok(format!("{}\n", command))
    }

    /// Handle dot commands (local client commands)
    fn handle_dot_command(&mut self, command: &str) -> Result<String> {
        let parts: Vec<&str> = command[1..].split_whitespace().collect();
        let cmd = parts.first().map(|s| s.to_lowercase()).unwrap_or_default();

        match cmd.as_str() {
            // Application commands
            "quit" | "q" => {
                self.quit();
            }
            "help" | "h" | "?" => {
                self.show_help();
            }

            // Layout commands
            "savelayout" => {
                let name = parts.get(1).unwrap_or(&"default");
                tracing::info!("[APP_CORE] User entered .savelayout command: '{}'", name);
                // Note: This is a placeholder - actual handling should be in main.rs with terminal size
                // For now, we'll use the layout's terminal size or fallback
                let width = self.layout.terminal_width.unwrap_or(80);
                let height = self.layout.terminal_height.unwrap_or(24);
                tracing::warn!(
                    "savelayout called without terminal size - using layout size {}x{}",
                    width,
                    height
                );
                self.save_layout(name, width, height);
            }
            "loadlayout" => {
                // This is just a placeholder - actual handling is in main.rs with terminal size
                self.add_system_message(
                    "Layout loading requires terminal size - handled by main event loop",
                );
            }
            "layouts" => {
                self.list_layouts();
            }
            "resize" => {
                self.resize_to_current_terminal();
            }

            // Window management commands
            "windows" => {
                self.list_windows();
            }
            "deletewindow" | "delwindow" => {
                if let Some(name) = parts.get(1) {
                    self.delete_window(name);
                } else {
                    self.add_system_message("Usage: .deletewindow <name>");
                }
            }
            "addwindow" => {
                if parts.len() >= 6 {
                    let name = parts[1];
                    let widget_type = parts[2];
                    let x = parts[3].parse::<u16>().unwrap_or(0);
                    let y = parts[4].parse::<u16>().unwrap_or(0);
                    let width = parts[5].parse::<u16>().unwrap_or(40);
                    let height = parts
                        .get(6)
                        .and_then(|h| h.parse::<u16>().ok())
                        .unwrap_or(10);
                    self.add_window(name, widget_type, x, y, width, height);
                } else if parts.len() == 1 {
                    // No arguments - open widget picker
                    return Ok("action:addwindow".to_string());
                } else {
                    self.add_system_message(
                        "Usage: .addwindow <name> <type> <x> <y> <width> [height]",
                    );
                    self.add_system_message(
                        "Types: text, progress, countdown, compass, hands, room, indicator",
                    );
                }
            }
            "rename" => {
                if parts.len() >= 3 {
                    let name = parts[1];
                    let new_title = parts[2..].join(" ");
                    self.rename_window(name, &new_title);
                } else {
                    self.add_system_message("Usage: .rename <window> <new title>");
                }
            }
            "border" => {
                if parts.len() >= 3 {
                    let name = parts[1];
                    let style = parts[2];
                    let color = parts.get(3).map(|c| c.to_string());
                    self.set_window_border(name, style, color);
                } else {
                    self.add_system_message("Usage: .border <window> <style> [color]");
                }
            }

            // Highlights
            "highlights" | "hl" => {
                return Ok("action:highlights".to_string());
            }
            "addhighlight" | "addhl" => {
                return Ok("action:addhighlight".to_string());
            }
            "edithighlight" | "edithl" => {
                if let Some(name) = parts.get(1) {
                    return Ok(format!("action:edithighlight:{}", name));
                } else {
                    return Ok("action:edithighlight".to_string());
                }
            }
            "testline" => {
                if let Some(text) = parts.get(1) {
                    let rest_of_line = command[command.find(text).unwrap_or(0)..].to_string();
                    self.inject_test_line(&rest_of_line);
                } else {
                    self.add_system_message("Usage: .testline <text>");
                }
            }

            // Keybinds
            "keybinds" | "kb" => {
                return Ok("action:keybinds".to_string());
            }
            "addkeybind" | "addkey" => {
                return Ok("action:addkeybind".to_string());
            }

            // Colors
            "colors" | "colorpalette" => {
                return Ok("action:colors".to_string());
            }
            "addcolor" | "createcolor" => {
                return Ok("action:addcolor".to_string());
            }
            "uicolors" => {
                return Ok("action:uicolors".to_string());
            }
            "spellcolors" => {
                return Ok("action:spellcolors".to_string());
            }
            "addspellcolor" | "newspellcolor" => {
                return Ok("action:addspellcolor".to_string());
            }

            // Themes
            "themes" => {
                return Ok("action:themes".to_string());
            }
            "settheme" | "theme" => {
                if let Some(name) = parts.get(1) {
                    return Ok(format!("action:settheme:{}", name));
                } else {
                    self.add_system_message("Usage: .settheme <name>");
                }
            }
            "edittheme" => {
                return Ok("action:edittheme".to_string());
            }

            // Tab navigation
            "nexttab" => {
                return Ok("action:nexttab".to_string());
            }
            "prevtab" => {
                return Ok("action:prevtab".to_string());
            }
            "gonew" | "nextunread" => {
                return Ok("action:nextunread".to_string());
            }

            // Settings
            "settings" => {
                return Ok("action:settings".to_string());
            }

            // Window editor
            "editwindow" | "editwin" => {
                if let Some(name) = parts.get(1) {
                    return Ok(format!("action:editwindow:{}", name));
                } else {
                    // Open window picker
                    return Ok("action:editwindow".to_string());
                }
            }

            // Menu system
            "menu" => {
                // Build main menu
                let items = self.build_main_menu();

                tracing::debug!("Creating menu with {} items", items.len());

                // Create popup menu at center of screen
                // Position will be adjusted by frontend based on actual terminal size
                self.ui_state.popup_menu = Some(crate::data::ui_state::PopupMenu::new(
                    items,
                    (40, 12), // Default center position
                ));

                // Switch to Menu input mode
                self.ui_state.input_mode = crate::data::ui_state::InputMode::Menu;
                tracing::debug!("Input mode set to Menu: {:?}", self.ui_state.input_mode);
                self.needs_render = true;
            }

            _ => {
                self.add_system_message(&format!("Unknown command: {}", command));
                self.add_system_message("Type .help for list of commands");
            }
        }

        // Command input is now managed by the CommandInput widget

        // Don't send anything to server
        Ok(String::new())
    }
}
