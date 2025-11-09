use crate::cmdlist::CmdList;
use crate::config::{Config, KeyAction, Layout};
use crate::network::ServerMessage;
use crate::parser::XmlParser;
use crate::performance::PerformanceStats;
use crate::sound::SoundPlayer;
use crate::ui::{WindowManager, StyledText, SpanType};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::style::Color;
use std::collections::HashMap;

/// Core application state (frontend-agnostic)
///
/// AppCore contains all business logic that is independent of the rendering layer.
/// It handles configuration, game state, XML parsing, stream routing, and window management logic.
/// Both TUI and GUI frontends will use AppCore for shared state and logic.
pub struct AppCore {
    /// Application configuration
    pub config: Config,

    /// Current window layout
    pub layout: Layout,

    /// Window manager (handles widget state and stream routing)
    pub window_manager: WindowManager,

    /// XML parser for GemStone IV protocol
    pub parser: XmlParser,

    /// Application running flag
    pub running: bool,

    /// Current active stream (where incoming text is routed)
    pub current_stream: String,

    /// If true, discard text because no window exists for current stream
    pub discard_current_stream: bool,

    /// Track if current chunk (since last prompt) has main stream text
    pub chunk_has_main_text: bool,

    /// Track if current chunk has silent updates (buffs, vitals, etc.)
    pub chunk_has_silent_updates: bool,

    /// Server time offset (server_time - local_time) for countdown calculations
    pub server_time_offset: i64,

    /// Buffer for accumulating stream text (used for combat/playerlist)
    pub stream_buffer: String,

    /// Parsed keybindings map (key combo -> action)
    pub keybind_map: HashMap<(KeyCode, KeyModifiers), KeyAction>,

    /// Performance statistics
    pub perf_stats: PerformanceStats,

    /// Whether to show performance stats
    pub show_perf_stats: bool,

    /// Sound player (None if initialization failed)
    pub sound_player: Option<SoundPlayer>,

    /// Command list for context menus (None if failed to load)
    pub cmdlist: Option<CmdList>,

    /// Counter for menu request correlation IDs
    pub menu_request_counter: u32,

    /// Navigation room ID (e.g., "2022628" from <nav rm='2022628'/>)
    pub nav_room_id: Option<String>,

    /// Lich room ID (e.g., "33711" extracted from room name display)
    pub lich_room_id: Option<String>,

    /// Room subtitle (e.g., " - Emberthorn Refuge, Bowery")
    pub room_subtitle: Option<String>,
}

impl AppCore {
    /// Create AppCore by extracting state from an initialized App
    ///
    /// This is a bridge method during the refactoring process. It allows us to use
    /// App::new() for initialization while testing the new architecture.
    ///
    /// TODO: Eventually replace this with a direct AppCore::new()
    pub fn from_app(app: &crate::app::App) -> Self {
        Self {
            config: app.config.clone(),
            layout: app.layout.clone(),
            window_manager: app.window_manager.clone(),
            parser: app.parser.clone(),
            running: app.running,
            current_stream: app.current_stream.clone(),
            discard_current_stream: app.discard_current_stream,
            chunk_has_main_text: app.chunk_has_main_text,
            chunk_has_silent_updates: app.chunk_has_silent_updates,
            server_time_offset: app.server_time_offset,
            stream_buffer: app.stream_buffer.clone(),
            keybind_map: app.keybind_map.clone(),
            perf_stats: app.perf_stats.clone(),
            show_perf_stats: app.show_perf_stats,
            // SoundPlayer can't be cloned (has OutputStream), so we'll skip it for now
            // TODO: Share sound player or recreate it
            sound_player: None,
            cmdlist: app.cmdlist.clone(),
            menu_request_counter: app.menu_request_counter,
            nav_room_id: app.nav_room_id.clone(),
            lich_room_id: app.lich_room_id.clone(),
            room_subtitle: app.room_subtitle.clone(),
        }
    }

    /// Create a new AppCore instance from existing components
    ///
    /// This is a temporary constructor used during the refactoring process.
    /// Eventually this will be the primary constructor that initializes everything.
    ///
    /// TODO: Refactor App::new() logic into this method
    pub fn from_existing(
        config: Config,
        layout: Layout,
        window_manager: WindowManager,
        parser: XmlParser,
        keybind_map: HashMap<(KeyCode, KeyModifiers), KeyAction>,
        sound_player: Option<SoundPlayer>,
        cmdlist: Option<CmdList>,
    ) -> Self {
        Self {
            config,
            layout,
            window_manager,
            parser,
            running: true,
            current_stream: "main".to_string(),
            discard_current_stream: false,
            chunk_has_main_text: false,
            chunk_has_silent_updates: false,
            server_time_offset: 0,
            stream_buffer: String::new(),
            keybind_map,
            perf_stats: PerformanceStats::new(),
            show_perf_stats: false,
            sound_player,
            cmdlist,
            menu_request_counter: 0,
            nav_room_id: None,
            lich_room_id: None,
            room_subtitle: None,
        }
    }

    /// Handle a server message (process game data)
    ///
    /// This method processes XML data from the game server, parses it,
    /// and routes text to appropriate windows.
    ///
    /// Simplified implementation for experimental mode (Milestone 6.4)
    /// TODO: Add remaining functionality from App::handle_server_message()
    pub fn handle_server_message(&mut self, msg: ServerMessage) -> Result<()> {
        use crate::parser::ParsedElement;
        use crate::ui::StyledText;
        use crate::ui::SpanType;
        use ratatui::style::Color;

        match msg {
            ServerMessage::Connected => {
                tracing::info!("Connected to server");
                self.add_system_message("Connected to Lich");

                // Play startup music if enabled
                if self.config.ui.startup_music {
                    if let Some(ref player) = self.sound_player {
                        if let Err(e) = player.play_from_sounds_dir(&self.config.ui.startup_music_file, Some(0.5)) {
                            tracing::warn!("Failed to play startup_music: {}", e);
                        }
                    }
                }
            }
            ServerMessage::Disconnected => {
                tracing::info!("Disconnected from server");
                self.add_system_message("Disconnected from Lich");
                self.running = false;
            }
            ServerMessage::Text(line) => {
                // Track network bytes received
                self.perf_stats.record_bytes_received(line.len() as u64);

                // Handle empty lines
                if line.is_empty() {
                    self.add_text_to_current_stream(StyledText {
                        content: String::new(),
                        fg: None,
                        bg: None,
                        bold: false,
                        span_type: SpanType::Normal,
                        link_data: None,
                    });
                    self.finish_current_line();
                    return Ok(());
                }

                // Parse XML
                let parse_start = std::time::Instant::now();
                let elements = self.parser.parse_line(&line);
                let parse_duration = parse_start.elapsed();
                self.perf_stats.record_parse(parse_duration);
                self.perf_stats.record_elements_parsed(elements.len() as u64);

                // Process elements
                for element in elements {
                    match element {
                        ParsedElement::Text { content, fg_color, bg_color, bold, span_type, link_data, .. } => {
                            if self.discard_current_stream {
                                continue;
                            }

                            // Add text even if empty (for proper line breaks)
                            self.add_text_to_current_stream(StyledText {
                                content,
                                fg: fg_color.and_then(|c| Self::parse_hex_color(&c)),
                                bg: bg_color.and_then(|c| Self::parse_hex_color(&c)),
                                bold,
                                span_type,
                                link_data,
                            });
                        }
                        ParsedElement::Prompt { text, .. } => {
                            // Reset to main stream
                            self.current_stream = "main".to_string();
                            self.discard_current_stream = false;

                            // Show prompt
                            if !text.trim().is_empty() {
                                for ch in text.chars() {
                                    let char_str = ch.to_string();
                                    let color = self.config.colors.prompt_colors
                                        .iter()
                                        .find(|pc| pc.character == char_str)
                                        .and_then(|pc| {
                                            pc.fg.as_ref().or(pc.color.as_ref())
                                                .and_then(|color_str| Self::parse_hex_color(color_str))
                                        })
                                        .unwrap_or(Color::DarkGray);

                                    self.add_text_to_current_stream(StyledText {
                                        content: char_str,
                                        fg: Some(color),
                                        bg: None,
                                        bold: false,
                                        span_type: SpanType::Normal,
                                        link_data: None,
                                    });
                                }
                            }
                        }
                        ParsedElement::StreamPush { id } => {
                            self.current_stream = id.clone();
                            if !self.window_manager.has_window_for_stream(&id) {
                                self.discard_current_stream = true;
                            } else {
                                self.discard_current_stream = false;
                            }
                        }
                        ParsedElement::StreamPop => {
                            self.current_stream = "main".to_string();
                            self.discard_current_stream = false;
                        }
                        ParsedElement::ProgressBar { id, value, max, text } => {
                            // Handle progress bar updates
                            let window_id = match id.as_str() {
                                "pbarStance" => "stance",
                                "mindState" => "mindstate",
                                "encumlevel" => "encumbrance",
                                _ => &id,
                            };

                            if let Some(window) = self.window_manager.get_window(window_id) {
                                if !text.is_empty() {
                                    window.set_progress_with_text(value, max, Some(text));
                                } else {
                                    window.set_progress(value, max);
                                }
                            }
                        }
                        ParsedElement::RoundTime { value } => {
                            if let Some(window) = self.window_manager.get_window("roundtime") {
                                window.set_countdown(value as u64);
                            }
                        }
                        ParsedElement::CastTime { value } => {
                            if let Some(window) = self.window_manager.get_window("casttime") {
                                window.set_countdown(value as u64);
                            }
                        }
                        // TODO: Add more element handlers as needed
                        _ => {}
                    }
                }

                // Finish the line after processing all elements from this server line
                // Each line from the server represents one logical line that needs wrapping
                self.finish_current_line();
            }
        }

        Ok(())
    }

    /// Add text to the current stream's window
    fn add_text_to_current_stream(&mut self, text: StyledText) {
        if let Some(window_name) = self.window_manager.stream_map.get(&self.current_stream).cloned() {
            if let Some(window) = self.window_manager.get_window(&window_name) {
                window.add_text(text);
            }
        }
    }

    /// Finish the current line in all windows
    fn finish_current_line(&mut self) {
        // Get terminal width for wrapping (default to 120 if we can't get it)
        let inner_width = 120u16.saturating_sub(2);

        if let Some(window_name) = self.window_manager.stream_map.get(&self.current_stream).cloned() {
            if let Some(window) = self.window_manager.get_window(&window_name) {
                window.finish_line(inner_width);
            }
        }
    }

    /// Parse hex color string to ratatui Color
    fn parse_hex_color(hex: &str) -> Option<Color> {
        if !hex.starts_with('#') || hex.len() != 7 {
            return None;
        }

        let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
        let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
        let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }

    /// Handle a dot command (local command not sent to server)
    ///
    /// Dot commands are local to the client (e.g., .quit, .addwindow, .settings)
    ///
    /// Simplified implementation for experimental mode (Milestone 6.5)
    /// TODO: Add remaining commands from App::handle_dot_command()
    pub fn handle_dot_command(&mut self, command: &str) -> Result<()> {
        let parts: Vec<&str> = command[1..].split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "quit" | "q" => {
                self.running = false;
                self.add_system_message("Exiting...");
            }
            "savelayout" => {
                let name = parts.get(1).unwrap_or(&"default");
                let terminal_size = crossterm::terminal::size().ok();
                match self.layout.save(name, terminal_size, false) {
                    Ok(_) => self.add_system_message(&format!("Layout saved as '{}'", name)),
                    Err(e) => self.add_system_message(&format!("Failed to save layout: {}", e)),
                }
            }
            "loadlayout" => {
                let name = parts.get(1).unwrap_or(&"default");
                let layout_path = crate::config::Config::layout_path(name)?;
                match crate::config::Layout::load_from_file(&layout_path) {
                    Ok(new_layout) => {
                        self.layout = new_layout;
                        self.add_system_message(&format!("Layout '{}' loaded", name));
                        self.update_window_manager_config()?;
                    }
                    Err(e) => self.add_system_message(&format!("Failed to load layout: {}", e)),
                }
            }
            "layouts" => {
                match crate::config::Config::list_layouts() {
                    Ok(layouts) => {
                        if layouts.is_empty() {
                            self.add_system_message("No saved layouts");
                        } else {
                            self.add_system_message(&format!("Saved layouts: {}", layouts.join(", ")));
                        }
                    }
                    Err(e) => self.add_system_message(&format!("Failed to list layouts: {}", e)),
                }
            }
            "windows" | "listwindows" => {
                let window_names = self.window_manager.get_window_names();
                if window_names.is_empty() {
                    self.add_system_message("No windows");
                } else {
                    self.add_system_message(&format!("Windows ({}): {}", window_names.len(), window_names.join(", ")));
                }
            }
            "help" => {
                self.add_system_message("Available commands (experimental mode):");
                self.add_system_message("  .quit / .q - Exit application");
                self.add_system_message("  .savelayout [name] - Save current layout");
                self.add_system_message("  .loadlayout [name] - Load saved layout");
                self.add_system_message("  .layouts - List saved layouts");
                self.add_system_message("  .windows - List active windows");
                self.add_system_message("  .help - Show this help");
            }
            _ => {
                self.add_system_message(&format!("Unknown command: .{} (try .help)", parts[0]));
            }
        }

        Ok(())
    }

    /// Add a system message to the main window
    ///
    /// System messages are local notifications (e.g., "Layout saved", "Window created")
    pub fn add_system_message(&mut self, message: &str) {
        use crate::ui::StyledText;
        use crate::ui::SpanType;
        use ratatui::style::Color;

        // Add message to main window
        if let Some(window) = self.window_manager.get_window("main") {
            window.add_text(StyledText {
                content: format!("[VellumFE] {}", message),
                fg: Some(Color::Yellow),
                bg: None,
                bold: false,
                span_type: SpanType::Normal,
                link_data: None,
            });
            window.finish_line(120);
        }

        tracing::info!("System message: {}", message);
    }

    /// Check and auto-resize layout if terminal is smaller than designed size
    ///
    /// TODO: Move implementation from App::check_and_auto_resize()
    pub fn check_and_auto_resize(&mut self) -> Result<()> {
        // TODO: Implementation will be moved from App in next step
        Ok(())
    }

    /// Update window manager configuration after layout changes
    ///
    /// Simplified implementation for experimental mode
    /// TODO: Add full implementation from App::update_window_manager_config()
    pub fn update_window_manager_config(&mut self) -> Result<()> {
        use crate::ui::WindowConfig;

        // Convert layout window definitions to window manager configs
        let window_configs: Vec<WindowConfig> = self.layout
            .windows
            .iter()
            .map(|w| WindowConfig {
                name: w.name.clone(),
                widget_type: w.widget_type.clone(),
                streams: w.streams.clone(),
                row: w.row,
                col: w.col,
                rows: w.rows,
                cols: w.cols,
                buffer_size: w.buffer_size,
                show_border: w.show_border,
                border_style: w.border_style.clone(),
                border_color: w.border_color.clone(),
                border_sides: w.border_sides.clone(),
                title: w.title.clone(),
                content_align: w.content_align.clone(),
                background_color: w.background_color.clone(),
                bar_fill: w.bar_fill.clone(),
                bar_background: w.bar_background.clone(),
                text_color: w.text_color.clone(),
                transparent_background: w.transparent_background,
                countdown_icon: Some(self.config.ui.countdown_icon.clone()),
                indicator_colors: w.indicator_colors.clone(),
                dashboard_layout: w.dashboard_layout.clone(),
                dashboard_indicators: w.dashboard_indicators.clone(),
                dashboard_spacing: w.dashboard_spacing,
                dashboard_hide_inactive: w.dashboard_hide_inactive,
                visible_count: w.visible_count,
                effect_category: w.effect_category.clone(),
                tabs: w.tabs.clone(),
                tab_bar_position: w.tab_bar_position.clone(),
                tab_active_color: w.tab_active_color.clone(),
                tab_inactive_color: w.tab_inactive_color.clone(),
                tab_unread_color: w.tab_unread_color.clone(),
                tab_unread_prefix: w.tab_unread_prefix.clone(),
                hand_icon: w.hand_icon.clone(),
                compass_active_color: w.compass_active_color.clone(),
                compass_inactive_color: w.compass_inactive_color.clone(),
                show_timestamps: w.show_timestamps,
                numbers_only: Some(w.numbers_only),
                injury_default_color: w.injury_default_color.clone(),
                injury1_color: w.injury1_color.clone(),
                injury2_color: w.injury2_color.clone(),
                injury3_color: w.injury3_color.clone(),
                scar1_color: w.scar1_color.clone(),
                scar2_color: w.scar2_color.clone(),
                scar3_color: w.scar3_color.clone(),
            })
            .collect();

        // Update window manager with new configs
        self.window_manager.update_config(window_configs);

        Ok(())
    }
}
