//! Core application logic - Pure business logic without UI coupling
//!
//! AppCore manages game state, configuration, and message processing.
//! It has NO knowledge of rendering - all state is stored in data structures
//! that frontends read from.

use crate::cmdlist::CmdList;
use crate::config::{Config, Layout};
use crate::core::{GameState, MessageProcessor};
use crate::data::*;
use crate::parser::{ParsedElement, XmlParser};
use crate::performance::PerformanceStats;
use anyhow::Result;
use std::collections::HashMap;

/// Pending menu request for correlation
#[derive(Clone, Debug)]
pub struct PendingMenuRequest {
    pub exist_id: String,
    pub noun: String,
}

/// Core application state - frontend-agnostic
pub struct AppCore {
    // === Configuration ===
    /// Application configuration (presets, highlights, keybinds, etc.)
    pub config: Config,

    /// Current window layout definition
    pub layout: Layout,

    /// Baseline layout for proportional resizing
    pub baseline_layout: Option<Layout>,

    // === State ===
    /// Game session state (connection, character, room, vitals, etc.)
    pub game_state: GameState,

    /// UI state (windows, focus, input, popups, etc.)
    pub ui_state: UiState,

    // === Message Processing ===
    /// XML parser for GemStone IV protocol
    pub parser: XmlParser,

    /// Message processor (routes parsed elements to state updates)
    pub message_processor: MessageProcessor,

    // === Stream Management ===
    /// Current active stream ID (where text is being routed)
    pub current_stream: String,

    /// If true, discard text because no window exists for stream
    pub discard_current_stream: bool,

    /// Buffer for accumulating multi-line stream content
    pub stream_buffer: String,

    // === Timing ===
    /// Server time offset (server_time - local_time) for countdown calculations
    pub server_time_offset: i64,

    // === Optional Features ===
    /// Command list for context menus (None if failed to load)
    pub cmdlist: Option<CmdList>,

    /// Menu request counter for correlating menu responses
    pub menu_request_counter: u32,

    /// Pending menu requests (counter -> PendingMenuRequest)
    pub pending_menu_requests: HashMap<String, PendingMenuRequest>,

    /// Cached menu categories for submenus (category_name -> items)
    pub menu_categories: HashMap<String, Vec<crate::data::ui_state::PopupMenuItem>>,

    /// Position of last link click (for menu positioning)
    pub last_link_click_pos: Option<(u16, u16)>,

    /// Performance statistics tracking
    pub perf_stats: PerformanceStats,

    /// Whether to show performance stats
    pub show_perf_stats: bool,

    /// Sound player for highlight sounds
    pub sound_player: Option<crate::sound::SoundPlayer>,

    /// Text-to-Speech manager for accessibility
    pub tts_manager: crate::tts::TtsManager,

    // === Navigation State ===
    /// Navigation room ID from <nav rm='...'/>
    pub nav_room_id: Option<String>,

    /// Lich room ID extracted from room display
    pub lich_room_id: Option<String>,

    /// Room subtitle (e.g., " - Emberthorn Refuge, Bowery")
    pub room_subtitle: Option<String>,

    /// Room component buffers (id -> lines of segments)
    /// Components: "room desc", "room objs", "room players", "room exits"
    pub room_components: HashMap<String, Vec<Vec<TextSegment>>>,

    /// Current room component being built
    pub current_room_component: Option<String>,

    /// Flag indicating room window needs sync
    pub room_window_dirty: bool,

    // === Runtime Flags ===
    /// Application running flag
    pub running: bool,

    /// Dirty flag - true if state changed and needs re-render
    pub needs_render: bool,

    /// Track if current chunk has main stream text
    pub chunk_has_main_text: bool,

    /// Track if current chunk has silent updates (vitals, buffs, etc.)
    pub chunk_has_silent_updates: bool,

    /// Track if layout has been modified since last .savelayout
    pub layout_modified_since_save: bool,

    /// Track if save reminder has been shown this session
    pub save_reminder_shown: bool,

    /// Base layout name for autosave reference
    pub base_layout_name: Option<String>,

    // === Keybind Runtime Cache ===
    /// Runtime keybind map for fast O(1) lookups (KeyEvent -> KeyBindAction)
    /// Built from config.keybinds at startup and on config reload
    pub keybind_map: HashMap<crossterm::event::KeyEvent, crate::config::KeyBindAction>,
}

impl AppCore {
    fn available_themes_message(theme_presets: &HashMap<String, crate::theme::AppTheme>) -> String {
        let mut names: Vec<_> = theme_presets.keys().cloned().collect();
        names.sort();
        format!("Available themes: {}", names.join(", "))
    }

    fn apply_layout_theme(
        &mut self,
        theme_name: Option<&str>,
    ) -> Option<(String, crate::theme::AppTheme)> {
        let theme_id = theme_name?;
        if theme_id == self.config.active_theme {
            return None;
        }

        let theme_presets =
            crate::theme::ThemePresets::all_with_custom(self.config.character.as_deref());

        if let Some(theme) = theme_presets.get(theme_id) {
            self.config.active_theme = theme_id.to_string();
            if let Err(e) = self.config.save(self.config.character.as_deref()) {
                tracing::warn!("Failed to save config after applying layout theme: {}", e);
            }
            Some((theme_id.to_string(), theme.clone()))
        } else {
            tracing::warn!(
                "Layout requested unknown theme '{}', keeping current theme '{}'",
                theme_id,
                self.config.active_theme
            );
            None
        }
    }

    /// Create a new AppCore instance
    pub fn new(config: Config) -> Result<Self> {
        // Load layout from file system
        let layout = Layout::load(config.character.as_deref())?;

        // Load command list
        let cmdlist = CmdList::load().ok();

        // Create message processor
        let message_processor = MessageProcessor::new(config.clone());

        // Convert presets from config to parser format
        let preset_list: Vec<(String, Option<String>, Option<String>)> = config
            .colors
            .presets
            .iter()
            .map(|(id, preset)| (id.clone(), preset.fg.clone(), preset.bg.clone()))
            .collect();

        // Create parser with presets and event patterns
        let parser = XmlParser::with_presets(preset_list, config.event_patterns.clone());

        // Initialize sound player (if sound feature is enabled)
        let sound_player = crate::sound::SoundPlayer::new(true, 0.8, 500).ok();
        if sound_player.is_some() {
            tracing::debug!("Sound player initialized");
            // Ensure sounds directory exists
            if let Err(e) = crate::sound::ensure_sounds_directory() {
                tracing::warn!("Failed to create sounds directory: {}", e);
            }
        }

        // Initialize TTS manager (respects config.tts.enabled)
        let tts_manager = crate::tts::TtsManager::new(
            config.tts.enabled,
            config.tts.rate,
            config.tts.volume
        );
        if config.tts.enabled {
            tracing::info!("TTS enabled - accessibility features active");
        }

        // Build the runtime keybind map from config
        let keybind_map = Self::build_keybind_map(&config);

        let layout_theme = layout.theme.clone();
        let mut app = Self {
            config,
            layout: layout.clone(),
            baseline_layout: Some(layout),
            game_state: GameState::new(),
            ui_state: UiState::new(),
            parser,
            message_processor,
            current_stream: String::from("main"),
            discard_current_stream: false,
            stream_buffer: String::new(),
            server_time_offset: 0,
            cmdlist,
            menu_request_counter: 0,
            pending_menu_requests: HashMap::new(),
            menu_categories: HashMap::new(),
            last_link_click_pos: None,
            perf_stats: PerformanceStats::new(),
            show_perf_stats: false,
            sound_player,
            tts_manager,
            nav_room_id: None,
            lich_room_id: None,
            room_subtitle: None,
            room_components: HashMap::new(),
            current_room_component: None,
            room_window_dirty: false,
            running: true,
            needs_render: true,
            chunk_has_main_text: false,
            chunk_has_silent_updates: false,
            layout_modified_since_save: false,
            save_reminder_shown: false,
            base_layout_name: None,
            keybind_map,
        };

        if let Some((theme_id, _)) = app.apply_layout_theme(layout_theme.as_deref()) {
            app.add_system_message(&format!("Theme switched to: {}", theme_id));
            // Update frontend cache later; AppCore just updates config here.
            // The frontend will refresh during initialization from config.
        }

        Ok(app)
    }

    /// Build runtime keybind map from config for fast O(1) lookups
    /// Converts string-based keybinds (e.g., "num_0", "Ctrl+s") to KeyEvent structs
    fn build_keybind_map(config: &Config) -> HashMap<crossterm::event::KeyEvent, crate::config::KeyBindAction> {
        use crossterm::event::{KeyEvent, KeyEventKind, KeyEventState};
        let mut map = HashMap::new();

        for (key_string, action) in &config.keybinds {
            // Parse the key string into a (KeyCode, KeyModifiers) tuple
            if let Some((code, modifiers)) = crate::config::parse_key_string(key_string) {
                // Create a KeyEvent from the parsed code and modifiers
                let key_event = KeyEvent {
                    code,
                    modifiers,
                    kind: KeyEventKind::Press,
                    state: KeyEventState::empty(),
                };
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

    // ===========================================================================================
    // Window Scrolling Methods
    // ===========================================================================================

    /// Scroll the currently focused window up by one line
    pub fn scroll_current_window_up_one(&mut self) {
        if let Some(window_name) = &self.ui_state.focused_window.clone() {
            if let Some(window) = self.ui_state.windows.get_mut(window_name) {
                if let crate::data::WindowContent::Text(ref mut content) = window.content {
                    content.scroll_up(1);
                }
            }
        }
    }

    /// Scroll the currently focused window down by one line
    pub fn scroll_current_window_down_one(&mut self) {
        if let Some(window_name) = &self.ui_state.focused_window.clone() {
            if let Some(window) = self.ui_state.windows.get_mut(window_name) {
                if let crate::data::WindowContent::Text(ref mut content) = window.content {
                    content.scroll_down(1);
                }
            }
        }
    }

    /// Scroll the currently focused window up by one page
    pub fn scroll_current_window_up_page(&mut self) {
        if let Some(window_name) = &self.ui_state.focused_window.clone() {
            if let Some(window) = self.ui_state.windows.get_mut(window_name) {
                if let crate::data::WindowContent::Text(ref mut content) = window.content {
                    // Use a reasonable page size (20 lines)
                    content.scroll_up(20);
                }
            }
        }
    }

    /// Scroll the currently focused window down by one page
    pub fn scroll_current_window_down_page(&mut self) {
        if let Some(window_name) = &self.ui_state.focused_window.clone() {
            if let Some(window) = self.ui_state.windows.get_mut(window_name) {
                if let crate::data::WindowContent::Text(ref mut content) = window.content {
                    // Use a reasonable page size (20 lines)
                    content.scroll_down(20);
                }
            }
        }
    }

    // ===========================================================================================
    // Keybind Action Execution
    // ===========================================================================================

    /// Execute a keybind action (called when a bound key is pressed)
    /// Returns a list of commands to send to the server (for macros)
    pub fn execute_keybind_action(&mut self, action: &crate::config::KeyBindAction) -> Result<Vec<String>> {
        use crate::config::{KeyAction, KeyBindAction};

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
                let clean_text = macro_action.macro_text.trim_end_matches(&['\r', '\n'][..]).to_string();

                tracing::info!("[MACRO] Executing macro: '{}' (raw: '{}')",
                    clean_text, macro_action.macro_text);

                // Send the macro text as a command (posts prompt+echo, returns command for server)
                let command = self.send_command(clean_text)?;
                tracing::info!("[MACRO] send_command returned: '{}'", command);
                Ok(vec![command]) // Return command for network layer to send
            }
        }
    }

    /// Execute a KeyAction (dispatch to the appropriate method)
    fn execute_key_action(&mut self, action: crate::config::KeyAction) -> Result<()> {
        use crate::config::KeyAction;

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
            | KeyAction::PreviousCommand
            | KeyAction::NextCommand
            | KeyAction::SendLastCommand
            | KeyAction::SendSecondLastCommand => {
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

            // Debug/Performance actions
            KeyAction::TogglePerformanceStats => {
                // TODO: Toggle performance stats overlay
                tracing::debug!("TogglePerformanceStats not yet implemented");
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

    /// Poll TTS events from callback channel and handle them
    /// Should be called in the main event loop to enable auto-play
    pub fn poll_tts_events(&mut self) {
        use std::sync::mpsc::TryRecvError;

        loop {
            match self.tts_manager.try_recv_event() {
                Ok(event) => {
                    match event {
                        crate::tts::TtsEvent::UtteranceEnded(id) => {
                            // Check if this was the current utterance
                            if self.tts_manager.is_current_utterance(id) {
                                tracing::debug!("Utterance {:?} ended (manual control - no auto-play)", id);
                                // Auto-play disabled - user has full manual control with Ctrl+Alt+Left/Right/Up
                            }
                        }
                        crate::tts::TtsEvent::UtteranceStarted(id) => {
                            tracing::debug!("Utterance {:?} started", id);
                        }
                        crate::tts::TtsEvent::UtteranceStopped(id) => {
                            tracing::debug!("Utterance {:?} stopped", id);
                        }
                    }
                }
                Err(TryRecvError::Empty) => {
                    // No more events to process
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    tracing::error!("TTS event channel disconnected");
                    break;
                }
            }
        }
    }

    /// Initialize windows based on current layout
    pub fn init_windows(&mut self, terminal_width: u16, terminal_height: u16) {
        // Calculate window positions from layout
        let positions = self.calculate_window_positions(terminal_width, terminal_height);

        // Create windows based on layout (only visible ones)
        for window_def in &self.layout.windows {
            // Skip hidden windows
            if !window_def.base().visible {
                tracing::debug!("Skipping hidden window '{}' during init", window_def.name());
                continue;
            }

            let position = positions
                .get(window_def.name())
                .cloned()
                .unwrap_or(WindowPosition {
                    x: 0,
                    y: 0,
                    width: 80,
                    height: 24,
                });

            let mut widget_type = match window_def.widget_type() {
                "text" => WidgetType::Text,
                "tabbedtext" => WidgetType::TabbedText,
                "progress" => WidgetType::Progress,
                "countdown" => WidgetType::Countdown,
                "compass" => WidgetType::Compass,
                "injury_doll" | "injuries" => WidgetType::InjuryDoll,
                "indicator" => WidgetType::Indicator,
                "room" => WidgetType::Room,
                "inventory" => WidgetType::Inventory,
                "command_input" | "commandinput" => WidgetType::CommandInput, // Support both for backward compatibility
                "dashboard" => WidgetType::Dashboard,
                "hand" => WidgetType::Hand,
                "active_effects" => WidgetType::ActiveEffects,
                "targets" => WidgetType::Targets,
                "players" => WidgetType::Players,
                "spells" => WidgetType::Spells,
                _ => WidgetType::Text,
            };

            let title = window_def
                .base()
                .title
                .as_deref()
                .unwrap_or(window_def.name());

            let content = match widget_type {
                WidgetType::Text => {
                    let buffer_size =
                        if let crate::config::WindowDef::Text { data, .. } = window_def {
                            data.buffer_size
                        } else {
                            1000 // fallback
                        };
                    WindowContent::Text(TextContent::new(title, buffer_size))
                }
                WidgetType::CommandInput => WindowContent::CommandInput {
                    text: String::new(),
                    cursor: 0,
                    history: Vec::new(),
                    history_index: None,
                },
                WidgetType::Progress => WindowContent::Progress(ProgressData {
                    value: 100,
                    max: 100,
                    label: title.to_string(),
                    color: None,
                }),
                WidgetType::Countdown => WindowContent::Countdown(CountdownData {
                    end_time: 0,
                    label: title.to_string(),
                }),
                WidgetType::Compass => WindowContent::Compass(CompassData {
                    directions: Vec::new(),
                }),
                WidgetType::InjuryDoll => WindowContent::InjuryDoll(InjuryDollData::new()),
                WidgetType::Indicator => WindowContent::Indicator(IndicatorData {
                    status: String::from("standing"),
                    color: None,
                }),
                WidgetType::Hand => WindowContent::Hand {
                    item: None,
                    link: None,
                },
                WidgetType::Room => WindowContent::Room(RoomContent {
                    name: String::new(),
                    description: Vec::new(),
                    exits: Vec::new(),
                    players: Vec::new(),
                    objects: Vec::new(),
                }),
                WidgetType::Inventory => WindowContent::Inventory(TextContent::new(title, 10000)),
                WidgetType::Spells => WindowContent::Spells(TextContent::new(title, 10000)),
                WidgetType::ActiveEffects => {
                    // Extract category from window def
                    let category =
                        if let crate::config::WindowDef::ActiveEffects { data, .. } = window_def {
                            data.category.clone()
                        } else {
                            "Unknown".to_string()
                        };
                    WindowContent::ActiveEffects(crate::data::ActiveEffectsContent {
                        category,
                        effects: Vec::new(),
                    })
                }
                WidgetType::Targets => WindowContent::Targets {
                    targets_text: String::new(),
                },
                WidgetType::Players => WindowContent::Players {
                    players_text: String::new(),
                },
                WidgetType::Dashboard => WindowContent::Dashboard {
                    indicators: Vec::new(),
                },
                _ => WindowContent::Empty,
            };

            let window = WindowState {
                name: window_def.name().to_string(),
                widget_type,
                content,
                position,
                visible: true,
                focused: false,
            };

            self.ui_state
                .set_window(window_def.name().to_string(), window);
        }

        self.needs_render = true;
    }

    /// Add a single new window without destroying existing ones
    ///
    /// Uses absolute positioning from window definition with optional delta-based scaling.
    pub fn add_new_window(
        &mut self,
        window_def: &crate::config::WindowDef,
        terminal_width: u16,
        terminal_height: u16,
    ) {
        tracing::info!(
            "add_new_window: '{}' ({})",
            window_def.name(),
            window_def.widget_type()
        );

        // Use exact position from window definition
        let base = window_def.base();
        let position = WindowPosition {
            x: base.col,
            y: base.row,
            width: base.cols,
            height: base.rows,
        };

        tracing::debug!(
            "Window '{}' will be created at exact pos=({},{}) size={}x{}",
            window_def.name(),
            position.x,
            position.y,
            position.width,
            position.height
        );

        let is_room_window = window_def.widget_type() == "room";

        let widget_type = match window_def.widget_type() {
            "text" => WidgetType::Text,
            "tabbedtext" => WidgetType::TabbedText,
            "progress" => WidgetType::Progress,
            "countdown" => WidgetType::Countdown,
            "compass" => WidgetType::Compass,
            "injury_doll" | "injuries" => WidgetType::InjuryDoll,
            "indicator" => WidgetType::Indicator,
            "room" => WidgetType::Room,
            "inventory" => WidgetType::Inventory,
            "command_input" | "commandinput" => WidgetType::CommandInput,
            "dashboard" => WidgetType::Dashboard,
            "hand" => WidgetType::Hand,
            "active_effects" => WidgetType::ActiveEffects,
            "targets" => WidgetType::Targets,
            "players" => WidgetType::Players,
            "spells" => WidgetType::Spells,
            _ => WidgetType::Text,
        };

        let title = window_def
            .base()
            .title
            .as_deref()
            .unwrap_or(window_def.name());

        let content = match widget_type {
            WidgetType::Text => {
                let buffer_size = if let crate::config::WindowDef::Text { data, .. } = window_def {
                    data.buffer_size
                } else {
                    1000 // fallback
                };
                WindowContent::Text(TextContent::new(title, buffer_size))
            }
            WidgetType::CommandInput => WindowContent::CommandInput {
                text: String::new(),
                cursor: 0,
                history: Vec::new(),
                history_index: None,
            },
            WidgetType::Progress => WindowContent::Progress(ProgressData {
                value: 100,
                max: 100,
                label: title.to_string(),
                color: None,
            }),
            WidgetType::Countdown => WindowContent::Countdown(CountdownData {
                end_time: 0,
                label: title.to_string(),
            }),
            WidgetType::Compass => WindowContent::Compass(CompassData {
                directions: Vec::new(),
            }),
            WidgetType::InjuryDoll => WindowContent::InjuryDoll(InjuryDollData::new()),
            WidgetType::Indicator => WindowContent::Indicator(IndicatorData {
                status: String::from("standing"),
                color: None,
            }),
            WidgetType::Hand => WindowContent::Hand {
                item: None,
                link: None,
            },
            WidgetType::Room => WindowContent::Room(RoomContent {
                name: String::new(),
                description: Vec::new(),
                exits: Vec::new(),
                players: Vec::new(),
                objects: Vec::new(),
            }),
            WidgetType::Inventory => WindowContent::Inventory(TextContent::new(title, 0)),
            WidgetType::Spells => WindowContent::Spells(TextContent::new(title, 0)),
            WidgetType::ActiveEffects => {
                // Extract category from window def
                let category =
                    if let crate::config::WindowDef::ActiveEffects { data, .. } = window_def {
                        data.category.clone()
                    } else {
                        "Unknown".to_string()
                    };
                WindowContent::ActiveEffects(crate::data::ActiveEffectsContent {
                    category,
                    effects: Vec::new(),
                })
            }
            WidgetType::Targets => WindowContent::Targets {
                targets_text: String::new(),
            },
            WidgetType::Players => WindowContent::Players {
                players_text: String::new(),
            },
            WidgetType::Dashboard => WindowContent::Dashboard {
                indicators: Vec::new(),
            },
            _ => WindowContent::Empty,
        };

        let window = WindowState {
            name: window_def.name().to_string(),
            widget_type,
            content,
            position: position.clone(),
            visible: true,
            focused: false,
        };

        self.ui_state
            .set_window(window_def.name().to_string(), window);
        self.needs_render = true;

        // Clear inventory cache if this is an inventory window to force initial render
        if window_def.widget_type() == "inventory" {
            self.message_processor.clear_inventory_cache();
        }

        // Set dirty flag for room windows to trigger sync in TUI frontend
        if is_room_window {
            self.room_window_dirty = true;
        }

        tracing::info!(
            "Created new window '{}' at ({}, {}) size {}x{}",
            window_def.name(),
            position.x,
            position.y,
            position.width,
            position.height
        );
    }

    /// Update an existing window's position without destroying content
    /// Update an existing window's position from window definition (uses exact positions, no scaling)
    ///
    /// This is called when editing a window via the window editor. It applies the exact
    /// position from the window definition to the UI state without any scaling.
    pub fn update_window_position(
        &mut self,
        window_def: &crate::config::WindowDef,
        _terminal_width: u16,
        _terminal_height: u16,
    ) {
        let base = window_def.base();
        let position = WindowPosition {
            x: base.col,
            y: base.row,
            width: base.cols,
            height: base.rows,
        };

        if let Some(window_state) = self.ui_state.windows.get_mut(window_def.name()) {
            window_state.position = position.clone();
            self.needs_render = true;
            tracing::info!(
                "Updated window '{}' to EXACT position ({}, {}) size {}x{}",
                window_def.name(),
                position.x,
                position.y,
                position.width,
                position.height
            );
        }
    }

    /// Remove a window from UI state
    pub fn remove_window(&mut self, name: &str) {
        self.ui_state.remove_window(name);
        self.needs_render = true;
        tracing::info!("Removed window '{}'", name);
    }

    /// Process incoming XML data from server
    pub fn process_server_data(&mut self, data: &str) -> Result<()> {
        // Parse XML line by line
        for line in data.lines() {
            let elements = self.parser.parse_line(line);

            // Process each element
            for element in elements {
                self.process_element(&element)?;
            }

            // Finish the current line after processing all elements from this network line
            // This ensures newlines from the game are preserved (like VellumFE does)
            self.message_processor
                .flush_current_stream_with_tts(&mut self.ui_state, Some(&mut self.tts_manager));
        }

        Ok(())
    }

    /// Process a single parsed XML element
    fn process_element(&mut self, element: &ParsedElement) -> Result<()> {
        // Handle MenuResponse specially (needs access to cmdlist and menu state)
        if let ParsedElement::MenuResponse { id, coords } = element {
            self.message_processor.chunk_has_silent_updates = true; // Mark as silent update
            self.handle_menu_response(id, coords);
            self.needs_render = true;
            return Ok(());
        }

        // Update game state and UI state via message processor
        self.message_processor.process_element(
            element,
            &mut self.game_state,
            &mut self.ui_state,
            &mut self.room_components,
            &mut self.current_room_component,
            &mut self.room_window_dirty,
            &mut self.nav_room_id,
            &mut self.lich_room_id,
            &mut self.room_subtitle,
            Some(&mut self.tts_manager),
        );

        // Mark that we need to render
        self.needs_render = true;

        Ok(())
    }

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
                    tracing::debug!("[SEND_COMMAND] Building styled line with prompt: '{}'", self.game_state.last_prompt);
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
                    tracing::info!("[SEND_COMMAND] Added StyledLine with {} segments to main window", segments.len());
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
            "hidewindow" => {
                if let Some(name) = parts.get(1) {
                    // Hide specific window
                    self.hide_window(name);
                } else {
                    // No arguments - open window picker for hiding
                    return Ok("action:hidewindow".to_string());
                }
            }
            "rename" => {
                if parts.len() >= 3 {
                    let window_name = parts[1];
                    let new_title = parts[2..].join(" ");
                    self.rename_window(window_name, &new_title);
                } else {
                    self.add_system_message("Usage: .rename <window> <new title>");
                }
            }
            "border" => {
                if parts.len() >= 3 {
                    let window_name = parts[1];
                    let border_style = parts[2];
                    let border_color = parts.get(3).map(|s| s.to_string());
                    self.set_window_border(window_name, border_style, border_color);
                } else {
                    self.add_system_message("Usage: .border <window> <style> [color]");
                    self.add_system_message("Styles: all, none, top, bottom, left, right");
                }
            }

            // Highlight commands
            "highlights" | "hl" => {
                // Open highlight browser instead of just listing
                return Ok("action:highlights".to_string());
            }
            "addhighlight" | "addhl" => {
                return Ok("action:addhighlight".to_string());
            }
            "edithighlight" | "edithl" => {
                if let Some(name) = parts.get(1) {
                    return Ok(format!("action:edithighlight:{}", name));
                } else {
                    self.add_system_message("Usage: .edithighlight <name>");
                }
            }

            // Keybind commands
            "keybinds" | "kb" => {
                return Ok("action:keybinds".to_string());
            }
            "addkeybind" | "addkey" => {
                return Ok("action:addkeybind".to_string());
            }

            // Color commands
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

            // Theme commands
            "themes" => {
                return Ok("action:themes".to_string());
            }
            "settheme" | "theme" => {
                if let Some(theme_name) = parts.get(1) {
                    // Validate theme exists (includes built-in and custom)
                    let theme_presets = crate::theme::ThemePresets::all_with_custom(
                        self.config.character.as_deref(),
                    );
                    let theme_list_message = Self::available_themes_message(&theme_presets);
                    if theme_presets.contains_key(*theme_name) {
                        self.config.active_theme = theme_name.to_string();
                        self.add_system_message(&format!("Theme switched to: {}", theme_name));

                        // Save config
                        if let Err(e) = self.config.save(self.config.character.as_deref()) {
                            tracing::error!("Failed to save config after theme change: {}", e);
                            self.add_system_message(&format!(
                                "Warning: Failed to save theme preference: {}",
                                e
                            ));
                        }

                        // Return action so main.rs can update frontend cache
                        return Ok(format!("action:settheme:{}", theme_name));
                    } else {
                        self.add_system_message(&format!("Unknown theme: {}", theme_name));
                        self.add_system_message(&theme_list_message);
                    }
                } else {
                    self.add_system_message("Usage: .settheme <name>");
                    let theme_presets = crate::theme::ThemePresets::all_with_custom(
                        self.config.character.as_deref(),
                    );
                    self.add_system_message(&Self::available_themes_message(&theme_presets));
                }
            }
            "edittheme" => {
                return Ok("action:edittheme".to_string());
            }

            // Tab navigation commands
            "nexttab" => {
                return Ok("action:nexttab".to_string());
            }
            "prevtab" => {
                return Ok("action:prevtab".to_string());
            }
            "gonew" | "nextunread" => {
                return Ok("action:gonew".to_string());
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

    /// Get list of available dot commands for tab completion
    pub fn get_available_commands(&self) -> Vec<String> {
        vec![
            // Application commands
            ".quit".to_string(),
            ".q".to_string(),
            ".help".to_string(),
            ".h".to_string(),
            ".?".to_string(),
            // Layout commands
            ".savelayout".to_string(),
            ".loadlayout".to_string(),
            ".layouts".to_string(),
            ".resize".to_string(),
            // Window management
            ".windows".to_string(),
            ".deletewindow".to_string(),
            ".delwindow".to_string(),
            ".addwindow".to_string(),
            ".rename".to_string(),
            ".border".to_string(),
            ".editwindow".to_string(),
            ".editwin".to_string(),
            // Highlight commands
            ".highlights".to_string(),
            ".hl".to_string(),
            ".addhighlight".to_string(),
            ".addhl".to_string(),
            ".edithighlight".to_string(),
            ".edithl".to_string(),
            // Keybind commands
            ".keybinds".to_string(),
            ".kb".to_string(),
            ".addkeybind".to_string(),
            ".addkey".to_string(),
            // Color commands
            ".colors".to_string(),
            ".colorpalette".to_string(),
            ".addcolor".to_string(),
            ".createcolor".to_string(),
            ".uicolors".to_string(),
            ".spellcolors".to_string(),
            ".addspellcolor".to_string(),
            ".newspellcolor".to_string(),
            // Theme commands
            ".themes".to_string(),
            ".settheme".to_string(),
            ".theme".to_string(),
            ".edittheme".to_string(),
            // Tab navigation
            ".nexttab".to_string(),
            ".prevtab".to_string(),
            ".gonew".to_string(),
            ".nextunread".to_string(),
            // Settings
            ".settings".to_string(),
            // Menu system
            ".menu".to_string(),
        ]
    }

    /// Get list of window names for tab completion
    pub fn get_window_names(&self) -> Vec<String> {
        self.layout
            .windows
            .iter()
            .map(|w| w.name().to_string())
            .collect()
    }

    /// Generate a unique spacer widget name based on existing spacers in layout
    /// Uses max number + 1 algorithm, checking ALL widgets including hidden ones
    /// Pattern: spacer_1, spacer_2, spacer_3, etc.
    pub fn generate_spacer_name(layout: &Layout) -> String {
        let max_number = layout
            .windows
            .iter()
            .filter_map(|w| {
                // Only consider spacer widgets
                match w {
                    crate::config::WindowDef::Spacer { base, .. } => {
                        // Extract number from name like "spacer_5"
                        if let Some(num_str) = base.name.strip_prefix("spacer_") {
                            num_str.parse::<u32>().ok()
                        } else {
                            None
                        }
                    }
                    _ => None,
                }
            })
            .max()
            .unwrap_or(0);

        format!("spacer_{}", max_number + 1)
    }

    /// List all loaded highlights
    fn list_highlights(&mut self) {
        let count = self.config.highlights.len();

        // Collect all highlight info first to avoid borrow checker issues
        let mut lines = vec![format!("=== Highlights ({}) ===", count)];

        for (name, pattern) in &self.config.highlights {
            let mut info = format!("  {} - pattern: '{}'", name, pattern.pattern);
            if let Some(ref fg) = pattern.fg {
                info.push_str(&format!(" fg:{}", fg));
            }
            if let Some(ref bg) = pattern.bg {
                info.push_str(&format!(" bg:{}", bg));
            }
            if pattern.bold {
                info.push_str(" bold");
            }
            lines.push(info);
        }

        // Add all messages
        for line in lines {
            self.add_system_message(&line);
        }
    }

    /// Add a system message to the main window
    pub fn add_system_message(&mut self, message: &str) {
        use crate::data::{SpanType, StyledLine, TextSegment, WindowContent};

        if let Some(main_window) = self.ui_state.get_window_mut("main") {
            if let WindowContent::Text(ref mut content) = main_window.content {
                let line = StyledLine {
                    segments: vec![TextSegment {
                        text: message.to_string(),
                        fg: Some("#00ff00".to_string()),
                        bg: None,
                        bold: true,
                        span_type: SpanType::Normal,
                        link_data: None,
                    }],
                };
                content.add_line(line);
                self.needs_render = true;
            }
        }
    }

    /// Show help for dot commands
    fn show_help(&mut self) {
        self.add_system_message("=== Two-Face Dot Commands ===");
        self.add_system_message("Application: .quit/.q, .help/.h/.?, .menu, .settings");
        self.add_system_message(
            "Layouts: .savelayout [name], .loadlayout [name], .layouts, .resize",
        );
        self.add_system_message("Windows: .windows, .addwindow <name> <type> <x> <y> <w> [h]");
        self.add_system_message(
            "         .deletewindow <name>, .rename <win> <title>, .editwindow [name]",
        );
        self.add_system_message("         .border <win> <style> [color]");
        self.add_system_message("Highlights: .highlights, .addhighlight, .edithighlight <name>");
        self.add_system_message("Keybinds: .keybinds, .addkeybind");
        self.add_system_message(
            "Colors: .colors, .addcolor, .uicolors, .spellcolors, .addspellcolor",
        );
        self.add_system_message("Themes: .themes, .settheme <name>");
    }

    /// Save current layout
    pub fn save_layout(&mut self, name: &str, terminal_width: u16, terminal_height: u16) {
        tracing::info!("========== SAVE LAYOUT: '{}' START ==========", name);
        tracing::info!(
            "Current terminal size: {}x{}",
            terminal_width,
            terminal_height
        );
        tracing::info!("Layout has {} windows defined", self.layout.windows.len());
        tracing::info!(
            "UI state has {} windows rendered",
            self.ui_state.windows.len()
        );

        // IMPORTANT: Capture actual window positions from UI state before saving
        // (user may have moved/resized windows with mouse)
        for window_def in &mut self.layout.windows {
            let window_name = window_def.name().to_string();
            let base = window_def.base();

            tracing::debug!(
                "Window '{}' BEFORE capture: pos=({},{}) size={}x{}",
                window_name,
                base.col,
                base.row,
                base.cols,
                base.rows
            );

            if let Some(window_state) = self.ui_state.windows.get(&window_name) {
                let ui_pos = &window_state.position;
                tracing::info!(
                    "Window '{}' - Capturing from UI state: pos=({},{}) size={}x{}",
                    window_name,
                    ui_pos.x,
                    ui_pos.y,
                    ui_pos.width,
                    ui_pos.height
                );

                // Clamp window position and size to terminal boundaries before saving
                let clamped_x = ui_pos.x.min(terminal_width.saturating_sub(1));
                let clamped_y = ui_pos.y.min(terminal_height.saturating_sub(1));

                // Ensure width doesn't exceed available space
                let max_width = terminal_width.saturating_sub(clamped_x);
                let clamped_width = ui_pos.width.min(max_width).max(10);

                // Ensure height doesn't exceed available space
                let max_height = terminal_height.saturating_sub(clamped_y);
                let clamped_height = ui_pos.height.min(max_height).max(3);

                if clamped_x != ui_pos.x
                    || clamped_y != ui_pos.y
                    || clamped_width != ui_pos.width
                    || clamped_height != ui_pos.height
                {
                    tracing::warn!(
                        "Window '{}' clamped: ({},{} {}x{}) -> ({},{} {}x{}) to fit terminal {}x{}",
                        window_name,
                        ui_pos.x,
                        ui_pos.y,
                        ui_pos.width,
                        ui_pos.height,
                        clamped_x,
                        clamped_y,
                        clamped_width,
                        clamped_height,
                        terminal_width,
                        terminal_height
                    );
                }

                let base = window_def.base_mut();
                base.row = clamped_y;
                base.col = clamped_x;
                base.rows = clamped_height;
                base.cols = clamped_width;

                tracing::debug!(
                    "Window '{}' AFTER capture: pos=({},{}) size={}x{}",
                    window_name,
                    base.col,
                    base.row,
                    base.cols,
                    base.rows
                );
            } else {
                tracing::warn!(
                    "Window '{}' is in layout but NOT in ui_state! Cannot capture position.",
                    window_name
                );
            }
        }

        let layout_path = match Config::layout_path(name) {
            Ok(path) => path,
            Err(e) => {
                tracing::error!("Failed to get layout path for '{}': {}", name, e);
                self.add_system_message(&format!("Failed to get layout path: {}", e));
                return;
            }
        };

        tracing::info!("Saving layout to: {}", layout_path.display());

        // Pass actual terminal size with force=true so it always updates to current terminal size
        self.layout.theme = Some(self.config.active_theme.clone());
        match self
            .layout
            .save(name, Some((terminal_width, terminal_height)), true)
        {
            Ok(_) => {
                tracing::info!(
                    "Layout '{}' saved successfully to {}",
                    name,
                    layout_path.display()
                );
                tracing::info!("========== SAVE LAYOUT: '{}' SUCCESS ==========", name);
                self.add_system_message(&format!("Layout saved as '{}'", name));
                // Clear modified flag and update base layout name
                self.layout_modified_since_save = false;
                self.base_layout_name = Some(name.to_string());
            }
            Err(e) => {
                tracing::error!("Failed to save layout '{}': {}", name, e);
                tracing::info!("========== SAVE LAYOUT: '{}' FAILED ==========", name);
                self.add_system_message(&format!("Failed to save layout: {}", e));
            }
        }
    }

    /// Load a saved layout and update window positions/configs
    ///
    /// Loads layout at exact positions specified in file.
    /// Use .resize command for delta-based proportional scaling after loading.
    pub fn load_layout(
        &mut self,
        name: &str,
        terminal_width: u16,
        terminal_height: u16,
    ) -> Option<(String, crate::theme::AppTheme)> {
        tracing::info!("========== LOAD LAYOUT: '{}' START ==========", name);
        tracing::info!(
            "Current terminal size: {}x{}",
            terminal_width,
            terminal_height
        );
        tracing::info!("Current layout has {} windows", self.layout.windows.len());
        tracing::info!(
            "Current UI state has {} windows",
            self.ui_state.windows.len()
        );

        let layout_path = match Config::layout_path(name) {
            Ok(path) => path,
            Err(e) => {
                tracing::error!("Failed to get layout path for '{}': {}", name, e);
                self.add_system_message(&format!("Failed to get layout path: {}", e));
                return None;
            }
        };

        tracing::info!("Loading layout from: {}", layout_path.display());

        match Layout::load_from_file(&layout_path) {
            Ok(new_layout) => {
                let theme_update = self.apply_layout_theme(new_layout.theme.as_deref());
                tracing::info!("Layout file loaded successfully");
                tracing::info!("Loaded layout has {} windows", new_layout.windows.len());
                tracing::info!(
                    "Loaded layout terminal size: {}x{}",
                    new_layout.terminal_width.unwrap_or(0),
                    new_layout.terminal_height.unwrap_or(0)
                );

                // Log all windows in the loaded layout
                for (idx, window_def) in new_layout.windows.iter().enumerate() {
                    let base = window_def.base();
                    tracing::info!(
                        "  [{}] Window '{}' ({}): pos=({},{}) size={}x{}",
                        idx,
                        window_def.name(),
                        window_def.widget_type(),
                        base.col,
                        base.row,
                        base.cols,
                        base.rows
                    );
                }

                // Check if terminal is too small for any window
                let mut terminal_too_small = false;
                for window_def in &new_layout.windows {
                    let base = window_def.base();
                    let required_width = base.col + base.cols;
                    let required_height = base.row + base.rows;

                    if required_width > terminal_width || required_height > terminal_height {
                        tracing::warn!(
                            "Window '{}' requires {}x{} but terminal is only {}x{}",
                            window_def.name(),
                            required_width,
                            required_height,
                            terminal_width,
                            terminal_height
                        );
                        terminal_too_small = true;
                    }
                }

                if terminal_too_small {
                    tracing::error!("Terminal too small to load layout '{}'", name);
                    self.add_system_message(&format!(
                        "Cannot load layout '{}': terminal too small",
                        name
                    ));
                    self.add_system_message("Increase terminal size or use a different layout");
                    return None;
                }

                // Store new layout
                let old_layout = std::mem::replace(&mut self.layout, new_layout.clone());
                self.baseline_layout = Some(new_layout);

                tracing::info!("Calling sync_layout_to_ui_state to apply changes...");

                // Update positions for existing windows, create new ones, remove old ones
                self.sync_layout_to_ui_state(terminal_width, terminal_height, &old_layout);

                tracing::info!(
                    "After sync: UI state now has {} windows",
                    self.ui_state.windows.len()
                );
                tracing::info!("========== LOAD LAYOUT: '{}' SUCCESS ==========", name);

                self.add_system_message(&format!("Layout '{}' loaded", name));

                // Clear modified flag and update base layout name
                self.layout_modified_since_save = false;
                self.base_layout_name = Some(name.to_string());
                self.needs_render = true;
                return theme_update;
            }
            Err(e) => {
                tracing::error!("Failed to load layout file '{}': {}", name, e);
                tracing::info!("========== LOAD LAYOUT: '{}' FAILED ==========", name);
                self.add_system_message(&format!("Failed to load layout: {}", e));
            }
        }

        None
    }

    /// Resize all windows proportionally based on current terminal size (VellumFE algorithm)
    ///
    /// This command resets to the baseline layout and applies delta-based proportional distribution.
    /// This is the ONLY place (besides initial load) that should perform scaling operations.
    pub fn resize_windows(&mut self, terminal_width: u16, terminal_height: u16) {
        use std::collections::HashSet;

        tracing::info!("========== RESIZE WINDOWS START (VellumFE algorithm) ==========");
        tracing::info!(
            "Target terminal size: {}x{}",
            terminal_width,
            terminal_height
        );

        // Get baseline layout (the original, unscaled layout)
        let baseline_layout = if let Some(ref bl) = self.baseline_layout {
            bl.clone()
        } else {
            tracing::error!("No baseline layout available");
            self.add_system_message("Error: No baseline layout - cannot resize");
            self.add_system_message("Load a layout first with .loadlayout");
            return;
        };

        let baseline_width = baseline_layout.terminal_width.unwrap_or(terminal_width);
        let baseline_height = baseline_layout.terminal_height.unwrap_or(terminal_height);

        tracing::info!(
            "Baseline terminal size: {}x{}",
            baseline_width,
            baseline_height
        );

        // Calculate deltas (not scale factors!)
        let width_delta = terminal_width as i32 - baseline_width as i32;
        let height_delta = terminal_height as i32 - baseline_height as i32;

        tracing::info!("Delta: width={:+}, height={:+}", width_delta, height_delta);

        if width_delta == 0 && height_delta == 0 {
            tracing::info!("No resize needed - terminal size matches baseline");
            self.add_system_message("Already at baseline size - no resize needed");
            return;
        }

        // Reset layout to baseline (critical - prevents cumulative scaling errors)
        self.layout = baseline_layout;

        tracing::info!("Reset to baseline layout - now applying proportional distribution...");

        // Categorize widgets by scaling behavior
        let mut static_both = HashSet::new();
        let mut static_height = HashSet::new();
        for window_def in &self.layout.windows {
            let base = window_def.base();
            match window_def.widget_type() {
                "compass" | "injury_doll" | "dashboard" | "indicator" => {
                    static_both.insert(base.name.clone());
                }
                "progress" | "countdown" | "hands" | "hand" | "lefthand" | "righthand"
                | "spellhand" | "command_input" => {
                    static_height.insert(base.name.clone());
                }
                _ => {}
            }
        }

        // Snapshot baseline row positions BEFORE any resizing
        // This ensures width distribution uses original row groupings
        let baseline_rows: Vec<(String, u16, u16)> = if let Some(ref baseline) = self.baseline_layout {
            baseline
                .windows
                .iter()
                .map(|w| (w.name().to_string(), w.base().row, w.base().rows))
                .collect()
        } else {
            self.layout
                .windows
                .iter()
                .map(|w| (w.base().name.clone(), w.base().row, w.base().rows))
                .collect()
        };

        // Apply VellumFE's proportional distribution algorithm
        self.apply_height_resize(height_delta, &static_both, &static_height);
        self.apply_width_resize(width_delta, &static_both, &baseline_rows);

        // Update layout terminal size to current
        self.layout.terminal_width = Some(terminal_width);
        self.layout.terminal_height = Some(terminal_height);

        // Apply resized positions to UI state
        for window_def in &self.layout.windows {
            if let Some(window_state) = self.ui_state.windows.get_mut(window_def.name()) {
                let base = window_def.base();
                window_state.position = WindowPosition {
                    x: base.col,
                    y: base.row,
                    width: base.cols,
                    height: base.rows,
                };
                tracing::debug!(
                    "Applied to UI: '{}' @ ({},{}) size {}x{}",
                    base.name,
                    base.col,
                    base.row,
                    base.cols,
                    base.rows
                );
            }
        }

        self.needs_render = true;
        self.add_system_message(&format!(
            "Resized to {}x{} - use .savelayout to save",
            terminal_width, terminal_height
        ));
        tracing::info!("========== RESIZE WINDOWS COMPLETE ==========");
    }

    /// Helper to get minimum widget size based on widget type (from VellumFE)
    fn widget_min_size(&self, widget_type: &str) -> (u16, u16) {
        match widget_type {
            "progress" | "countdown" | "indicator" | "hands" | "hand" => (10, 1),
            "compass" => (13, 5),
            "injury_doll" => (20, 10),
            "dashboard" => (15, 3),
            "command_input" => (20, 1),
            _ => (5, 3), // text, room, tabbed, etc.
        }
    }

    pub fn window_min_size(&self, window_name: &str) -> (u16, u16) {
        if let Some(window_def) = self.layout.windows.iter().find(|w| w.name() == window_name) {
            let (default_min_cols, default_min_rows) =
                self.widget_min_size(window_def.widget_type());
            let base = window_def.base();
            let min_cols = base.min_cols.unwrap_or(default_min_cols);
            let min_rows = base.min_rows.unwrap_or(default_min_rows);
            (min_cols, min_rows)
        } else {
            self.widget_min_size("text")
        }
    }

    /// Apply proportional height resize (from VellumFE apply_height_resize)
    /// Adapted for WindowDef enum structure
    fn apply_height_resize(
        &mut self,
        height_delta: i32,
        static_both: &std::collections::HashSet<String>,
        static_height: &std::collections::HashSet<String>,
    ) {
        use std::collections::{HashMap, HashSet};

        if height_delta == 0 {
            return;
        }

        tracing::debug!("--- HEIGHT SCALING (VellumFE COLUMN-BY-COLUMN) ---");
        tracing::debug!("height_delta={}", height_delta);

        // PHASE 1: Calculate height deltas column-by-column
        let mut height_deltas: HashMap<String, i32> = HashMap::new();

        // Find max column
        let max_col = self.layout.windows.iter()
            .map(|w| {
                let base = w.base();
                base.col + base.cols
            })
            .max()
            .unwrap_or(0);

        tracing::debug!("Processing columns 0..{}", max_col);

        // Column-by-column: Calculate height deltas
        for current_col in 0..max_col {
            // Find UNPROCESSED windows that occupy this column
            let mut windows_at_col: Vec<String> = Vec::new();
            for window_def in &self.layout.windows {
                let base = window_def.base();
                // Skip if already has delta (VellumFE-compatible: only process each window once)
                if height_deltas.contains_key(&base.name) {
                    continue;
                }
                if base.col <= current_col && base.col + base.cols > current_col {
                    windows_at_col.push(base.name.clone());
                }
            }

            if windows_at_col.is_empty() {
                continue;
            }

            tracing::debug!("Column {}: {} windows present", current_col, windows_at_col.len());

            // Calculate total scalable height (only UNPROCESSED non-static windows)
            let mut total_scalable_height = 0u16;
            for window_name in &windows_at_col {
                // Skip if static
                if static_both.contains(window_name.as_str()) || static_height.contains(window_name.as_str()) {
                    continue;
                }

                // Get window height (include ALL non-static windows)
                let window_def = self.layout.windows.iter()
                    .find(|w| w.name() == window_name)
                    .unwrap();
                let base = window_def.base();
                total_scalable_height += base.rows;
            }

            if total_scalable_height == 0 {
                continue;
            }

            tracing::debug!("  Total scalable height at column {}: {}", current_col, total_scalable_height);

            // Distribute height_delta proportionally
            for window_name in &windows_at_col {
                // Handle static windows
                if static_both.contains(window_name.as_str()) || static_height.contains(window_name.as_str()) {
                    if !height_deltas.contains_key(window_name) {
                        tracing::debug!("    {} is static, recording 0 delta", window_name);
                        height_deltas.insert(window_name.clone(), 0);
                    }
                    continue;
                }

                // Calculate proportional delta for this window at this column
                let window_def = self.layout.windows.iter()
                    .find(|w| w.name() == window_name)
                    .unwrap();
                let base = window_def.base();
                let proportion = base.rows as f64 / total_scalable_height as f64;
                let delta = (proportion * height_delta as f64).floor() as i32;

                // Record delta (all windows in list are unprocessed)
                height_deltas.insert(window_name.clone(), delta);
                tracing::debug!(
                    "    {} (rows={}): proportion={:.4}, delta={}",
                    window_name,
                    base.rows,
                    proportion,
                    delta
                );
            }
        }

        tracing::debug!("Height deltas calculated for {} windows", height_deltas.len());

        // Distribute leftover rows (VellumFE-compatible)
        // Calculate how much was actually distributed
        let total_distributed: i32 = height_deltas.values().sum();
        let mut leftover = height_delta - total_distributed;

        if leftover != 0 {
            tracing::debug!("Height leftover after proportional distribution: {} rows", leftover);

            // Sort windows by row for consistent leftover distribution (top to bottom)
            let mut window_names: Vec<String> = height_deltas.keys()
                .filter(|name| {
                    // Only distribute to non-static windows
                    !static_both.contains(name.as_str()) && !static_height.contains(name.as_str())
                })
                .cloned()
                .collect();

            window_names.sort_by_key(|name| {
                self.layout.windows.iter()
                    .find(|w| w.name() == name)
                    .map(|w| w.base().row)
                    .unwrap_or(0)
            });

            // Distribute leftover one row at a time to first windows
            let mut idx = 0;
            while leftover > 0 && idx < window_names.len() {
                let name = &window_names[idx];
                if let Some(delta) = height_deltas.get_mut(name) {
                    *delta += 1;
                    tracing::debug!("  Distributing +1 leftover row to {}", name);
                    leftover -= 1;
                }
                idx += 1;
            }
            while leftover < 0 && idx < window_names.len() {
                let name = &window_names[idx];
                if let Some(delta) = height_deltas.get_mut(name) {
                    *delta -= 1;
                    tracing::debug!("  Distributing -1 leftover row to {}", name);
                    leftover += 1;
                }
                idx += 1;
            }
        }

        // PHASE 2: Apply deltas with column-by-column cascading
        tracing::debug!("Applying height deltas with column-by-column cascading...");

        let mut height_applied = HashSet::new();
        let max_col = self.layout.windows.iter()
            .map(|w| {
                let base = w.base();
                base.col + base.cols
            })
            .max()
            .unwrap_or(0);

        // Process each column independently
        for current_col in 0..max_col {
            // Find windows that occupy this column and haven't been applied yet
            let mut windows_at_col: Vec<(String, u16, u16)> = self.layout.windows.iter()
                .filter(|w| {
                    let base = w.base();
                    let col_start = base.col;
                    let col_end = base.col + base.cols;
                    current_col >= col_start && current_col < col_end && !height_applied.contains(&base.name)
                })
                .map(|w| {
                    let base = w.base();
                    (base.name.clone(), base.row, base.rows)
                })
                .collect();

            if windows_at_col.is_empty() {
                continue;
            }

            // Sort by row (top to bottom)
            windows_at_col.sort_by_key(|(_, row, _)| *row);

            if current_col == 0 || current_col == 20 {  // Only log for interesting columns
                tracing::debug!("Column {}: {} unapplied windows, cascading top-to-bottom", current_col, windows_at_col.len());
            }

            // Cascade within this column
            let mut current_row = windows_at_col[0].1;  // Start at first window's row

            for (window_name, original_row, original_rows) in windows_at_col {
                let delta = height_deltas.get(&window_name).copied().unwrap_or(0);

                // Get window for constraints
                let window_def = self.layout.windows.iter()
                    .find(|w| w.name() == &window_name)
                    .unwrap();
                let base = window_def.base();
                let widget_type = window_def.widget_type();
                let (_, min_rows) = self.widget_min_size(&widget_type);
                let min_constraint = base.min_rows.unwrap_or(min_rows);
                let max_constraint = base.max_rows;

                // Calculate new rows with constraints
                let mut new_rows = (original_rows as i32 + delta).max(min_constraint as i32) as u16;
                if let Some(max) = max_constraint {
                    new_rows = new_rows.min(max);
                }

                // Apply changes
                if let Some(w) = self.layout.windows.iter_mut().find(|w| w.name() == &window_name) {
                    let base = w.base_mut();
                    base.row = current_row;
                    base.rows = new_rows;
                    height_applied.insert(window_name.clone());

                    tracing::debug!(
                        "  Col {}: {} row {} -> {}, rows {} -> {} (delta={})",
                        current_col, window_name, original_row, current_row, original_rows, new_rows, delta
                    );
                }

                // Next window starts where this one ends (cascading)
                current_row += new_rows;
            }
        }

        tracing::debug!("Height resize complete");
    }

    /// Apply proportional width resize (from VellumFE apply_width_resize)
    /// Adapted for WindowDef enum structure
    /// baseline_rows: Vec of (name, baseline_row, baseline_rows) for grouping windows by original row
    fn apply_width_resize(
        &mut self,
        width_delta: i32,
        static_both: &std::collections::HashSet<String>,
        _baseline_rows: &[(String, u16, u16)],
    ) {
        use std::collections::{HashMap, HashSet};

        if width_delta == 0 {
            return;
        }

        tracing::debug!("--- WIDTH SCALING (VellumFE ROW-BY-ROW) ---");
        tracing::debug!("width_delta={}", width_delta);

        // PHASE 1: Calculate width deltas row-by-row
        let mut width_deltas: HashMap<String, i32> = HashMap::new();

        // Find max row
        let max_row = self.layout.windows.iter()
            .map(|w| {
                let base = w.base();
                base.row + base.rows
            })
            .max()
            .unwrap_or(0);

        tracing::debug!("Processing rows 0..{}", max_row);

        // Row-by-row: Calculate width deltas
        for current_row in 0..max_row {
            // Find UNPROCESSED windows that occupy this row
            let mut windows_at_row: Vec<String> = Vec::new();
            for window_def in &self.layout.windows {
                let base = window_def.base();
                // Skip if already has delta (VellumFE-compatible: only process each window once)
                if width_deltas.contains_key(&base.name) {
                    continue;
                }
                if base.row <= current_row && base.row + base.rows > current_row {
                    windows_at_row.push(base.name.clone());
                }
            }

            if windows_at_row.is_empty() {
                continue;
            }

            tracing::debug!("Row {}: {} windows present", current_row, windows_at_row.len());

            // Calculate total scalable width (only UNPROCESSED non-static windows)
            let mut total_scalable_width = 0u16;
            for window_name in &windows_at_row {
                // Skip if static
                if static_both.contains(window_name.as_str()) {
                    continue;
                }

                // Get window width (include ALL non-static windows)
                let window_def = self.layout.windows.iter()
                    .find(|w| w.name() == window_name)
                    .unwrap();
                let base = window_def.base();
                total_scalable_width += base.cols;
            }

            if total_scalable_width == 0 {
                continue;
            }

            tracing::debug!("  Total scalable width at row {}: {}", current_row, total_scalable_width);

            // Distribute width_delta proportionally
            for window_name in &windows_at_row {
                // Handle static windows
                if static_both.contains(window_name.as_str()) {
                    if !width_deltas.contains_key(window_name) {
                        tracing::debug!("    {} is static, recording 0 delta", window_name);
                        width_deltas.insert(window_name.clone(), 0);
                    }
                    continue;
                }

                // Calculate proportional delta for this window at this row
                let window_def = self.layout.windows.iter()
                    .find(|w| w.name() == window_name)
                    .unwrap();
                let base = window_def.base();
                let proportion = base.cols as f64 / total_scalable_width as f64;
                let delta = (proportion * width_delta as f64).floor() as i32;

                // Record delta (all windows in list are unprocessed)
                width_deltas.insert(window_name.clone(), delta);
                tracing::debug!(
                    "    {} (cols={}): proportion={:.4}, delta={}",
                    window_name,
                    base.cols,
                    proportion,
                    delta
                );
            }
        }

        tracing::debug!("Width deltas calculated for {} windows", width_deltas.len());

        // Distribute leftover columns (VellumFE-compatible)
        // Calculate how much was actually distributed
        let total_distributed: i32 = width_deltas.values().sum();
        let mut leftover = width_delta - total_distributed;

        if leftover != 0 {
            tracing::debug!("Width leftover after proportional distribution: {} columns", leftover);

            // Sort windows by column for consistent leftover distribution (left to right)
            let mut window_names: Vec<String> = width_deltas.keys()
                .filter(|name| {
                    // Only distribute to non-static windows
                    !static_both.contains(name.as_str())
                })
                .cloned()
                .collect();

            window_names.sort_by_key(|name| {
                self.layout.windows.iter()
                    .find(|w| w.name() == name)
                    .map(|w| w.base().col)
                    .unwrap_or(0)
            });

            // Distribute leftover one column at a time to first windows
            let mut idx = 0;
            while leftover > 0 && idx < window_names.len() {
                let name = &window_names[idx];
                if let Some(delta) = width_deltas.get_mut(name) {
                    *delta += 1;
                    tracing::debug!("  Distributing +1 leftover column to {}", name);
                    leftover -= 1;
                }
                idx += 1;
            }
            while leftover < 0 && idx < window_names.len() {
                let name = &window_names[idx];
                if let Some(delta) = width_deltas.get_mut(name) {
                    *delta -= 1;
                    tracing::debug!("  Distributing -1 leftover column to {}", name);
                    leftover += 1;
                }
                idx += 1;
            }
        }

        // PHASE 2: Apply deltas with cascading based on BASELINE row positions
        tracing::debug!("Applying width deltas using baseline row groupings...");

        let mut width_applied = HashSet::new();

        // Group windows by their BASELINE row positions
        // This ensures windows that were originally in the same row cascade together,
        // even though height cascading has moved them to different rows
        let mut baseline_row_groups: std::collections::HashMap<u16, Vec<String>> = std::collections::HashMap::new();
        for (name, baseline_row, _baseline_rows) in _baseline_rows {
            baseline_row_groups.entry(*baseline_row)
                .or_insert_with(Vec::new)
                .push(name.clone());
        }

        // Sort baseline rows to process top to bottom
        let mut sorted_baseline_rows: Vec<u16> = baseline_row_groups.keys().copied().collect();
        sorted_baseline_rows.sort();

        // Process each baseline row group independently
        for baseline_row in sorted_baseline_rows {
            let window_names = baseline_row_groups.get(&baseline_row).unwrap();

            // Find unapplied windows from this baseline row group
            let mut windows_at_row: Vec<(String, u16, u16)> = window_names.iter()
                .filter(|name| !width_applied.contains(*name))
                .filter_map(|name| {
                    self.layout.windows.iter()
                        .find(|w| w.name() == name)
                        .map(|w| {
                            let base = w.base();
                            (base.name.clone(), base.col, base.cols)
                        })
                })
                .collect();

            if windows_at_row.is_empty() {
                continue;
            }

            // Sort by column (left to right)
            windows_at_row.sort_by_key(|(_, col, _)| *col);

            tracing::debug!("Baseline row {}: {} unapplied windows, cascading left-to-right", baseline_row, windows_at_row.len());

            // Cascade within this baseline row group
            let mut current_col = windows_at_row[0].1;  // Start at first window's column

            for (window_name, original_col, original_cols) in windows_at_row {
                let delta = width_deltas.get(&window_name).copied().unwrap_or(0);

                // Get window for constraints
                let window_def = self.layout.windows.iter()
                    .find(|w| w.name() == &window_name)
                    .unwrap();
                let base = window_def.base();
                let widget_type = window_def.widget_type();
                let (min_cols, _) = self.widget_min_size(&widget_type);
                let min_constraint = base.min_cols.unwrap_or(min_cols);
                let max_constraint = base.max_cols;

                // Calculate new cols with constraints
                let mut new_cols = (original_cols as i32 + delta).max(min_constraint as i32) as u16;
                if let Some(max) = max_constraint {
                    new_cols = new_cols.min(max);
                }

                // Apply changes
                if let Some(w) = self.layout.windows.iter_mut().find(|w| w.name() == &window_name) {
                    let base = w.base_mut();
                    base.col = current_col;
                    base.cols = new_cols;
                    width_applied.insert(window_name.clone());

                    tracing::debug!(
                        "  Baseline row {}: {} col {} -> {}, cols {} -> {} (delta={})",
                        baseline_row, window_name, original_col, current_col, original_cols, new_cols, delta
                    );
                }

                // Next window starts where this one ends (cascading)
                current_col += new_cols;
            }
        }

        tracing::debug!("Width resize complete");
    }

    /// Sync layout WindowDefs to ui_state WindowStates without destroying content
    ///
    /// Uses exact positions from layout file.
    /// Use .resize command for delta-based proportional scaling.
    pub fn sync_layout_to_ui_state(
        &mut self,
        terminal_width: u16,
        terminal_height: u16,
        old_layout: &Layout,
    ) {
        tracing::info!("--- sync_layout_to_ui_state START ---");
        tracing::info!("Terminal size: {}x{}", terminal_width, terminal_height);
        tracing::info!("New layout has {} windows", self.layout.windows.len());

        // Use exact positions from layout file
        tracing::debug!("Using exact positions from layout file");

        // Track which windows are in the new layout AND visible
        let new_window_names: std::collections::HashSet<String> = self
            .layout
            .windows
            .iter()
            .filter(|w| w.base().visible)
            .map(|w| w.name().to_string())
            .collect();

        tracing::info!("Visible windows in new layout: {:?}", new_window_names);

        let current_window_names: std::collections::HashSet<String> =
            self.ui_state.windows.keys().cloned().collect();
        tracing::info!("Windows currently in UI state: {:?}", current_window_names);

        // Collect windows to create (can't create while iterating due to borrow checker)
        let mut windows_to_create: Vec<crate::config::WindowDef> = Vec::new();
        let mut windows_to_update = 0;

        // Update existing windows' positions
        for window_def in &self.layout.windows {
            let window_name = window_def.name().to_string();
            let base = window_def.base();

            // Skip hidden windows
            if !base.visible {
                tracing::debug!("Skipping hidden window '{}'", window_name);
                continue;
            }

            // Use exact position from layout file
            let position = WindowPosition {
                x: base.col,
                y: base.row,
                width: base.cols,
                height: base.rows,
            };

            tracing::debug!(
                "Processing window '{}': exact pos=({},{}) size={}x{}",
                window_name,
                position.x,
                position.y,
                position.width,
                position.height
            );

            if let Some(window_state) = self.ui_state.windows.get_mut(&window_name) {
                // Window exists - just update position (preserve content!)
                let old_pos = window_state.position.clone();
                window_state.position = position.clone();
                windows_to_update += 1;
                tracing::info!(
                    "UPDATING window '{}': pos ({},{})→({},{}) size {}x{}→{}x{}",
                    window_name,
                    old_pos.x,
                    old_pos.y,
                    position.x,
                    position.y,
                    old_pos.width,
                    old_pos.height,
                    position.width,
                    position.height
                );
            } else {
                // Window doesn't exist - queue for creation
                tracing::info!(
                    "Window '{}' not in UI state - queuing for creation",
                    window_name
                );
                windows_to_create.push(window_def.clone());
            }
        }

        tracing::info!(
            "Summary: {} windows to update, {} windows to create",
            windows_to_update,
            windows_to_create.len()
        );

        // Create new windows
        if !windows_to_create.is_empty() {
            tracing::info!("Creating {} new windows...", windows_to_create.len());
            for window_def in windows_to_create {
                let window_name = window_def.name().to_string();
                tracing::info!(
                    "CREATING window '{}' ({})",
                    window_name,
                    window_def.widget_type()
                );
                self.add_new_window(&window_def, terminal_width, terminal_height);
            }
        }

        // Remove windows that are no longer in the layout
        let windows_to_remove: Vec<String> = self
            .ui_state
            .windows
            .keys()
            .filter(|name| !new_window_names.contains(*name))
            .cloned()
            .collect();

        if !windows_to_remove.is_empty() {
            tracing::info!(
                "Removing {} windows not in new layout: {:?}",
                windows_to_remove.len(),
                windows_to_remove
            );
            for window_name in windows_to_remove {
                self.ui_state.remove_window(&window_name);
                tracing::info!("REMOVED window '{}'", window_name);
            }
        } else {
            tracing::info!("No windows to remove");
        }

        tracing::info!("--- sync_layout_to_ui_state COMPLETE ---");
    }

    /// Load a saved layout with terminal size for immediate reinitialization
    pub fn load_layout_with_size(&mut self, name: &str, width: u16, height: u16) {
        let layout_path = match Config::layout_path(name) {
            Ok(path) => path,
            Err(e) => {
                self.add_system_message(&format!("Failed to get layout path: {}", e));
                return;
            }
        };

        match Layout::load_from_file(&layout_path) {
            Ok(new_layout) => {
                self.apply_layout_theme(new_layout.theme.as_deref());
                self.layout = new_layout.clone();
                self.baseline_layout = Some(new_layout);
                self.add_system_message(&format!("Layout '{}' loaded", name));

                // Clear modified flag and update base layout name
                self.layout_modified_since_save = false;
                self.base_layout_name = Some(name.to_string());

                // Reinitialize windows from new layout with actual terminal size
                self.init_windows(width, height);
                self.needs_render = true;
            }
            Err(e) => self.add_system_message(&format!("Failed to load layout: {}", e)),
        }
    }

    /// List all saved layouts
    fn list_layouts(&mut self) {
        match Config::list_layouts() {
            Ok(layouts) => {
                if layouts.is_empty() {
                    self.add_system_message("No saved layouts");
                } else {
                    self.add_system_message(&format!("=== Saved Layouts ({}) ===", layouts.len()));
                    for layout in layouts {
                        self.add_system_message(&format!("  {}", layout));
                    }
                }
            }
            Err(e) => self.add_system_message(&format!("Failed to list layouts: {}", e)),
        }
    }

    /// Resize layout using delta-based proportional distribution
    /// This method is called by the .resize command and requires manual invocation
    pub fn resize_to_terminal(&mut self, terminal_width: u16, terminal_height: u16) {
        // Need a baseline layout to calculate delta from
        let baseline = match &self.baseline_layout {
            Some(baseline) => baseline,
            None => {
                self.add_system_message(
                    "No baseline layout - save current layout first with .savelayout",
                );
                return;
            }
        };

        // Get baseline terminal size
        let baseline_width = baseline.terminal_width.unwrap_or(80);
        let baseline_height = baseline.terminal_height.unwrap_or(24);

        // Calculate delta
        let width_delta = terminal_width as i32 - baseline_width as i32;
        let height_delta = terminal_height as i32 - baseline_height as i32;

        if width_delta == 0 && height_delta == 0 {
            self.add_system_message(&format!(
                "Terminal size unchanged ({}x{})",
                terminal_width, terminal_height
            ));
            return;
        }

        tracing::info!(
            "Resizing layout: baseline {}x{} -> current {}x{} (delta: {}x{})",
            baseline_width,
            baseline_height,
            terminal_width,
            terminal_height,
            width_delta,
            height_delta
        );

        // Simple delta-based proportional distribution
        // For each window: calculate its proportion of total size, then distribute delta proportionally

        // Calculate total scalable width and height from baseline
        let total_baseline_width: u16 = baseline.windows.iter().map(|w| w.base().cols).sum();
        let total_baseline_height: u16 = baseline.windows.iter().map(|w| w.base().rows).sum();

        let mut width_remainder = width_delta;
        let mut height_remainder = height_delta;

        // Apply proportional resize to each window in the layout
        for window_def in &mut self.layout.windows {
            let window_name = window_def.name().to_string();
            let baseline_window = baseline.windows.iter().find(|w| w.name() == window_name);

            if let Some(baseline_win) = baseline_window {
                let baseline_base = baseline_win.base();
                let base = window_def.base_mut();

                // Calculate width adjustment
                if total_baseline_width > 0 && width_delta != 0 {
                    let proportion = baseline_base.cols as f64 / total_baseline_width as f64;
                    let width_share = (proportion * width_delta as f64).floor() as i32;
                    let new_width = (baseline_base.cols as i32 + width_share).max(1) as u16;
                    base.cols = new_width;
                    width_remainder -= width_share;
                }

                // Calculate height adjustment
                if total_baseline_height > 0 && height_delta != 0 {
                    let proportion = baseline_base.rows as f64 / total_baseline_height as f64;
                    let height_share = (proportion * height_delta as f64).floor() as i32;
                    let new_height = (baseline_base.rows as i32 + height_share).max(1) as u16;
                    base.rows = new_height;
                    height_remainder -= height_share;
                }
            }
        }

        // Distribute remainders to first windows (simple round-robin)
        if width_remainder != 0 {
            for window_def in &mut self.layout.windows {
                if width_remainder == 0 {
                    break;
                }
                let base = window_def.base_mut();
                if width_remainder > 0 {
                    base.cols += 1;
                    width_remainder -= 1;
                } else if base.cols > 1 {
                    base.cols -= 1;
                    width_remainder += 1;
                }
            }
        }

        if height_remainder != 0 {
            for window_def in &mut self.layout.windows {
                if height_remainder == 0 {
                    break;
                }
                let base = window_def.base_mut();
                if height_remainder > 0 {
                    base.rows += 1;
                    height_remainder -= 1;
                } else if base.rows > 1 {
                    base.rows -= 1;
                    height_remainder += 1;
                }
            }
        }

        // Recalculate positions for vertically stacked windows
        // Sort windows by baseline Y position to maintain stacking order
        let mut window_positions: Vec<(String, u16, u16, u16, u16)> = baseline
            .windows
            .iter()
            .map(|w| {
                (
                    w.name().to_string(),
                    w.base().col,
                    w.base().row,
                    w.base().cols,
                    w.base().rows,
                )
            })
            .collect();
        window_positions.sort_by_key(|(_, _, row, _, _)| *row);

        // Track the bottom edge of the previous window for each column group
        let mut col_groups: std::collections::HashMap<u16, u16> = std::collections::HashMap::new();

        // Recalculate Y positions for stacked windows
        for (name, baseline_col, baseline_row, _baseline_cols, baseline_rows) in window_positions {
            if let Some(window_def) = self.layout.windows.iter_mut().find(|w| w.name() == name) {
                let base = window_def.base_mut();

                // Check if this window should be stacked under a previous window
                if let Some(&prev_bottom) = col_groups.get(&baseline_col) {
                    // If baseline Y matches the previous window's bottom, they're stacked
                    if baseline_row == prev_bottom {
                        base.row = col_groups[&baseline_col];
                    }
                }

                // Update the bottom edge for this column group
                col_groups.insert(baseline_col, base.row + base.rows);
            }
        }

        // Update layout terminal size
        self.layout.terminal_width = Some(terminal_width);
        self.layout.terminal_height = Some(terminal_height);

        // Mark as modified and trigger re-init
        self.layout_modified_since_save = true;
        self.init_windows(terminal_width, terminal_height);
        self.needs_render = true;

        self.add_system_message(&format!(
            "Layout resized to {}x{} (delta: {:+}x{:+})",
            terminal_width, terminal_height, width_delta, height_delta
        ));
    }

    /// Wrapper for resize command - gets terminal size from layout
    fn resize_to_current_terminal(&mut self) {
        let width = self.layout.terminal_width.unwrap_or(80);
        let height = self.layout.terminal_height.unwrap_or(24);
        self.resize_to_terminal(width, height);
    }

    /// List all windows
    fn list_windows(&mut self) {
        let window_count = self.ui_state.windows.len();

        // Collect window info first to avoid borrow checker issues
        let mut window_info = Vec::new();
        for (name, window) in &self.ui_state.windows {
            let pos = &window.position;
            let visible = if window.visible { "visible" } else { "hidden" };
            window_info.push(format!(
                "  {} - {}x{} at ({},{}) - {} - {}",
                name,
                pos.width,
                pos.height,
                pos.x,
                pos.y,
                visible,
                format!("{:?}", window.widget_type)
            ));
        }

        // Now add all messages
        self.add_system_message(&format!("=== Windows ({}) ===", window_count));
        for info in window_info {
            self.add_system_message(&info);
        }
    }

    /// Hide a window (keep in layout for persistence, remove from UI)
    pub fn hide_window(&mut self, name: &str) {
        if name == "main" {
            self.add_system_message("Cannot hide main window");
            return;
        }

        // Find ALL windows with this name and mark as hidden (handles duplicates)
        let mut found_count = 0;
        for window_def in self.layout.windows.iter_mut() {
            if window_def.name() == name && window_def.base().visible {
                window_def.base_mut().visible = false;
                found_count += 1;
            }
        }

        if found_count > 0 {
            // Remove from UI state (but keep in layout!)
            self.ui_state.remove_window(name);

            let msg = if found_count > 1 {
                format!(
                    "Window '{}' hidden ({} duplicates removed)",
                    name, found_count
                )
            } else {
                format!("Window '{}' hidden", name)
            };
            self.add_system_message(&msg);
            self.mark_layout_modified();
            self.needs_render = true;
            tracing::info!(
                "Hid {} instance(s) of window '{}' - template(s) preserved in layout",
                found_count,
                name
            );
        } else {
            self.add_system_message(&format!("Window '{}' not found or already hidden", name));
        }
    }

    /// Show a window (unhide it - restore from layout template)
    pub fn show_window(&mut self, name: &str, terminal_width: u16, terminal_height: u16) {
        // Find window in layout and mark as visible
        let window_def_clone =
            if let Some(window_def) = self.layout.windows.iter_mut().find(|w| w.name() == name) {
                // Mark as visible
                window_def.base_mut().visible = true;
                // Clone for use after the mutable borrow ends
                window_def.clone()
            } else {
                self.add_system_message(&format!("Window template '{}' not found in layout", name));
                return;
            };

        // Create in UI state from layout template (borrow checker happy now)
        self.add_new_window(&window_def_clone, terminal_width, terminal_height);

        self.add_system_message(&format!("Window '{}' shown", name));
        self.mark_layout_modified();
        self.needs_render = true;
        tracing::info!("Showed window '{}' - restored from layout template", name);
    }

    /// Delete a window (legacy - use hide_window instead)
    fn delete_window(&mut self, name: &str) {
        // For backwards compatibility, redirect to hide
        self.hide_window(name);
    }

    /// Add a new window
    fn add_window(
        &mut self,
        name: &str,
        widget_type_str: &str,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
    ) {
        use crate::config::WindowDef;
        use crate::data::{
            CompassData, CountdownData, IndicatorData, ProgressData, RoomContent, TextContent,
            WidgetType, WindowContent, WindowPosition, WindowState,
        };

        // Check if window already exists
        if self.ui_state.windows.contains_key(name) {
            self.add_system_message(&format!("Window '{}' already exists", name));
            return;
        }

        // Parse widget type
        let widget_type = match widget_type_str.to_lowercase().as_str() {
            "text" => WidgetType::Text,
            "progress" => WidgetType::Progress,
            "countdown" => WidgetType::Countdown,
            "compass" => WidgetType::Compass,
            "injury_doll" | "injuries" => WidgetType::InjuryDoll,
            "hand" => WidgetType::Hand,
            "room" => WidgetType::Room,
            "indicator" => WidgetType::Indicator,
            "command_input" | "commandinput" => WidgetType::CommandInput,
            _ => {
                self.add_system_message(&format!("Unknown widget type: {}", widget_type_str));
                self.add_system_message("Types: text, progress, countdown, compass, injury_doll, hand, room, indicator, command_input");
                return;
            }
        };

        // Create window content based on type
        let content = match widget_type {
            WidgetType::Text => WindowContent::Text(TextContent::new(name, 1000)),
            WidgetType::Progress => WindowContent::Progress(ProgressData {
                value: 100,
                max: 100,
                label: name.to_string(),
                color: None,
            }),
            WidgetType::Countdown => WindowContent::Countdown(CountdownData {
                end_time: 0,
                label: name.to_string(),
            }),
            WidgetType::Compass => WindowContent::Compass(CompassData {
                directions: Vec::new(),
            }),
            WidgetType::InjuryDoll => WindowContent::InjuryDoll(InjuryDollData::new()),
            WidgetType::Hand => WindowContent::Hand {
                item: None,
                link: None,
            },
            WidgetType::Room => WindowContent::Room(RoomContent {
                name: String::new(),
                description: Vec::new(),
                exits: Vec::new(),
                players: Vec::new(),
                objects: Vec::new(),
            }),
            WidgetType::Indicator => WindowContent::Indicator(IndicatorData {
                status: String::from("standing"),
                color: None,
            }),
            WidgetType::CommandInput => WindowContent::CommandInput {
                text: String::new(),
                cursor: 0,
                history: Vec::new(),
                history_index: None,
            },
            _ => WindowContent::Empty,
        };

        // Create window state
        let window = WindowState {
            name: name.to_string(),
            widget_type: widget_type.clone(),
            content,
            position: WindowPosition {
                x,
                y,
                width,
                height,
            },
            visible: true,
            focused: false,
        };

        // Add to UI state
        self.ui_state.set_window(name.to_string(), window);

        // Create window definition for layout
        use crate::config::{
            BorderSides, CommandInputWidgetData, RoomWidgetData, TextWidgetData, WindowBase,
        };

        let base = WindowBase {
            name: name.to_string(),
            row: y,
            col: x,
            rows: height,
            cols: width,
            show_border: true,
            border_style: "single".to_string(),
            border_sides: BorderSides::default(),
            border_color: None,
            show_title: true,
            title: Some(name.to_string()),
            background_color: None,
            text_color: None,
            transparent_background: true,
            locked: false,
            min_rows: None,
            max_rows: None,
            min_cols: None,
            max_cols: None,
            visible: true,
        };

        let window_def = match widget_type_str.to_lowercase().as_str() {
            "text" => WindowDef::Text {
                base,
                data: TextWidgetData {
                    streams: vec![],
                    buffer_size: 1000,
                },
            },
            "room" => WindowDef::Room {
                base,
                data: RoomWidgetData {
                    buffer_size: 0,
                    show_desc: true,
                    show_objs: true,
                    show_players: true,
                    show_exits: true,
                    show_name: false,
                },
            },
            "command_input" | "commandinput" => WindowDef::CommandInput {
                base,
                data: CommandInputWidgetData::default(),
            },
            _ => {
                // Default to text window for unknown types
                WindowDef::Text {
                    base,
                    data: TextWidgetData {
                        streams: vec![],
                        buffer_size: 1000,
                    },
                }
            }
        };

        // Add to layout at the front (so new windows appear on top)
        self.layout.windows.insert(0, window_def);

        self.add_system_message(&format!(
            "Window '{}' added ({}x{} at {},{}) - type: {}",
            name, width, height, x, y, widget_type_str
        ));
        self.needs_render = true;
    }

    /// Rename a window's title
    fn rename_window(&mut self, window_name: &str, new_title: &str) {
        // Update in layout definition
        if let Some(window_def) = self
            .layout
            .windows
            .iter_mut()
            .find(|w| w.name() == window_name)
        {
            window_def.base_mut().title = Some(new_title.to_string());
            self.add_system_message(&format!(
                "Window '{}' renamed to '{}'",
                window_name, new_title
            ));
            self.needs_render = true;
        } else {
            self.add_system_message(&format!("Window '{}' not found", window_name));
        }
    }

    /// Set window border style and color
    fn set_window_border(&mut self, window_name: &str, style: &str, color: Option<String>) {
        if let Some(window_def) = self
            .layout
            .windows
            .iter_mut()
            .find(|w| w.name() == window_name)
        {
            use crate::config::BorderSides;

            let style_lower = style.to_lowercase();
            let (new_show, new_sides) = match style_lower.as_str() {
                "none" => (false, window_def.base().border_sides.clone()),
                "all" => (true, BorderSides::default()),
                "top" => (
                    true,
                    BorderSides {
                        top: true,
                        bottom: false,
                        left: false,
                        right: false,
                    },
                ),
                "bottom" => (
                    true,
                    BorderSides {
                        top: false,
                        bottom: true,
                        left: false,
                        right: false,
                    },
                ),
                "left" => (
                    true,
                    BorderSides {
                        top: false,
                        bottom: false,
                        left: true,
                        right: false,
                    },
                ),
                "right" => (
                    true,
                    BorderSides {
                        top: false,
                        bottom: false,
                        left: false,
                        right: true,
                    },
                ),
                _ => {
                    self.add_system_message(&format!("Unknown border style: {}", style));
                    return;
                }
            };

            window_def
                .base_mut()
                .apply_border_configuration(new_show, new_sides);

            // Set border color if provided
            if let Some(c) = color {
                window_def.base_mut().border_color = Some(c);
            }

            self.add_system_message(&format!("Border updated for window '{}'", window_name));
            self.needs_render = true;
        } else {
            self.add_system_message(&format!("Window '{}' not found", window_name));
        }
    }

    /// Handle terminal resize
    pub fn resize(&mut self, width: u16, height: u16) {
        // Recalculate all window positions
        let positions = self.calculate_window_positions(width, height);

        // Update all window positions
        for (name, position) in positions {
            if let Some(window) = self.ui_state.get_window_mut(&name) {
                window.position = position;
            }
        }

        self.needs_render = true;
    }

    /// Calculate window positions based on layout and terminal size
    fn calculate_window_positions(
        &self,
        width: u16,
        height: u16,
    ) -> HashMap<String, WindowPosition> {
        let mut positions = HashMap::new();

        // Use layout file values directly (row, col, rows, cols from layout)
        // Scale if terminal size differs from layout's expected terminal size
        let layout_width = self.layout.terminal_width.unwrap_or(width) as f32;
        let layout_height = self.layout.terminal_height.unwrap_or(height) as f32;
        let actual_width = width as f32;
        let actual_height = height as f32;

        // Calculate scale factors (don't scale if layout size is 0 or terminal size matches)
        let scale_x = if layout_width > 0.0 && (layout_width - actual_width).abs() > 1.0 {
            actual_width / layout_width
        } else {
            1.0
        };
        let scale_y = if layout_height > 0.0 && (layout_height - actual_height).abs() > 1.0 {
            actual_height / layout_height
        } else {
            1.0
        };

        tracing::debug!(
            "Layout terminal size: {}x{}, actual: {}x{}, scale: {:.2}x{:.2}",
            layout_width,
            layout_height,
            actual_width,
            actual_height,
            scale_x,
            scale_y
        );

        for window_def in &self.layout.windows {
            // Scale window position and size
            let scaled_x = (window_def.base().col as f32 * scale_x) as u16;
            let scaled_y = (window_def.base().row as f32 * scale_y) as u16;
            let mut scaled_width = (window_def.base().cols as f32 * scale_x).max(1.0) as u16;
            let mut scaled_height = (window_def.base().rows as f32 * scale_y).max(1.0) as u16;

            // Apply min/max constraints from window settings
            if let Some(min_cols) = window_def.base().min_cols {
                if scaled_width < min_cols {
                    tracing::debug!(
                        "Window '{}': enforcing min_cols={} (was {})",
                        window_def.name(),
                        min_cols,
                        scaled_width
                    );
                    scaled_width = min_cols;
                }
            }
            if let Some(max_cols) = window_def.base().max_cols {
                if scaled_width > max_cols {
                    tracing::debug!(
                        "Window '{}': enforcing max_cols={} (was {})",
                        window_def.name(),
                        max_cols,
                        scaled_width
                    );
                    scaled_width = max_cols;
                }
            }
            if let Some(min_rows) = window_def.base().min_rows {
                if scaled_height < min_rows {
                    tracing::debug!(
                        "Window '{}': enforcing min_rows={} (was {})",
                        window_def.name(),
                        min_rows,
                        scaled_height
                    );
                    scaled_height = min_rows;
                }
            }
            if let Some(max_rows) = window_def.base().max_rows {
                if scaled_height > max_rows {
                    tracing::debug!(
                        "Window '{}': enforcing max_rows={} (was {})",
                        window_def.name(),
                        max_rows,
                        scaled_height
                    );
                    scaled_height = max_rows;
                }
            }

            tracing::debug!(
                "Window '{}': layout pos=({},{}) size={}x{}, scaled pos=({},{}) size={}x{}",
                window_def.name(),
                window_def.base().col,
                window_def.base().row,
                window_def.base().cols,
                window_def.base().rows,
                scaled_x,
                scaled_y,
                scaled_width,
                scaled_height
            );

            positions.insert(
                window_def.name().to_string(),
                WindowPosition {
                    x: scaled_x,
                    y: scaled_y,
                    width: scaled_width,
                    height: scaled_height,
                },
            );
        }

        positions
    }

    /// Build main menu for .menu command
    fn build_main_menu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        vec![
            crate::data::ui_state::PopupMenuItem {
                text: "Colors >".to_string(),
                command: "__SUBMENU__colors".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Highlights >".to_string(),
                command: "__SUBMENU__highlights".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Keybinds >".to_string(),
                command: "__SUBMENU__keybinds".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Layouts >".to_string(),
                command: "__SUBMENU__layouts".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Themes >".to_string(),
                command: "__SUBMENU__themes".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Settings".to_string(),
                command: ".settings".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Windows >".to_string(),
                command: "__SUBMENU__windows".to_string(),
                disabled: false,
            },
        ]
    }

    /// Build colors submenu
    fn build_colors_submenu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        vec![
            crate::data::ui_state::PopupMenuItem {
                text: "Add color".to_string(),
                command: ".addcolor".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Browse colors".to_string(),
                command: ".colors".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Add spellcolor".to_string(),
                command: ".addspellcolor".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Browse spellcolors".to_string(),
                command: ".spellcolors".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Browse UI colors".to_string(),
                command: ".uicolors".to_string(),
                disabled: false,
            },
        ]
    }

    /// Build highlights submenu
    fn build_highlights_submenu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        vec![
            crate::data::ui_state::PopupMenuItem {
                text: "Add highlight".to_string(),
                command: ".addhighlight".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Browse highlights".to_string(),
                command: ".highlights".to_string(),
                disabled: false,
            },
        ]
    }

    /// Build keybinds submenu
    fn build_keybinds_submenu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        vec![
            crate::data::ui_state::PopupMenuItem {
                text: "Add keybind".to_string(),
                command: ".addkeybind".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Browse keybinds".to_string(),
                command: ".keybinds".to_string(),
                disabled: false,
            },
        ]
    }

    /// Build themes submenu
    fn build_themes_submenu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        vec![
            crate::data::ui_state::PopupMenuItem {
                text: "Browse themes".to_string(),
                command: ".themes".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Edit theme".to_string(),
                command: ".edittheme".to_string(),
                disabled: false,
            },
        ]
    }

    /// Build windows submenu
    fn build_windows_submenu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        vec![
            crate::data::ui_state::PopupMenuItem {
                text: "Add window".to_string(),
                command: ".addwindow".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Edit window".to_string(),
                command: ".editwindow".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "Hide window".to_string(),
                command: ".hidewindow".to_string(),
                disabled: false,
            },
            crate::data::ui_state::PopupMenuItem {
                text: "List windows".to_string(),
                command: ".windows".to_string(),
                disabled: false,
            },
        ]
    }

    /// Build layouts submenu
    fn build_layouts_submenu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        let mut items = Vec::new();

        // Get list of saved layouts
        match Config::list_layouts() {
            Ok(layouts) => {
                for layout_name in layouts {
                    items.push(crate::data::ui_state::PopupMenuItem {
                        text: layout_name.clone(),
                        command: format!("loadlayout:{}", layout_name),
                        disabled: false,
                    });
                }
            }
            Err(_) => {
                // If we can't load layouts, just show a disabled message
                items.push(crate::data::ui_state::PopupMenuItem {
                    text: "No layouts found".to_string(),
                    command: String::new(),
                    disabled: true,
                });
            }
        }

        items
    }

    /// Build submenu based on category name
    pub fn build_submenu(&self, category: &str) -> Vec<crate::data::ui_state::PopupMenuItem> {
        match category {
            "colors" => self.build_colors_submenu(),
            "highlights" => self.build_highlights_submenu(),
            "keybinds" => self.build_keybinds_submenu(),
            "layouts" => self.build_layouts_submenu(),
            "themes" => self.build_themes_submenu(),
            "windows" => self.build_windows_submenu(),
            _ => Vec::new(),
        }
    }

    /// Handle menu response from server
    fn handle_menu_response(&mut self, counter: &str, coords: &[(String, Option<String>)]) {
        // Look up the pending request
        let pending = match self.pending_menu_requests.remove(counter) {
            Some(p) => p,
            None => {
                tracing::warn!("Received menu response for unknown counter: {}", counter);
                return;
            }
        };

        tracing::info!(
            "Menu response for exist_id {} (noun: {}): {} coords",
            pending.exist_id,
            pending.noun,
            coords.len()
        );

        // Check if cmdlist is loaded
        let cmdlist = match &self.cmdlist {
            Some(list) => list,
            None => {
                tracing::warn!("Context menu received but cmdlist not loaded");
                return;
            }
        };

        // Group menu items by category
        let mut categories: HashMap<String, Vec<crate::data::ui_state::PopupMenuItem>> =
            HashMap::new();

        for (coord, secondary_noun) in coords {
            if let Some(entry) = cmdlist.get(coord) {
                // Skip _dialog commands
                if entry.command.starts_with("_dialog") {
                    continue;
                }

                // Build menu text (remove @ and # placeholders, substitute %)
                let menu_text = Self::format_menu_text(&entry.menu, secondary_noun.as_deref());

                // Build command with placeholders substituted
                let command = CmdList::substitute_command(
                    &entry.command,
                    &pending.noun,
                    &pending.exist_id,
                    secondary_noun.as_deref(),
                );

                let category = if entry.menu_cat.is_empty() {
                    "0".to_string()
                } else {
                    entry.menu_cat.clone()
                };

                categories.entry(category).or_insert_with(Vec::new).push(
                    crate::data::ui_state::PopupMenuItem {
                        text: menu_text,
                        command,
                        disabled: false,
                    },
                );
            }
        }

        if categories.is_empty() {
            tracing::warn!("No menu items available for this object");
            return;
        }

        // Build final menu with categories
        let mut menu_items = Vec::new();
        let mut sorted_cats: Vec<_> = categories.keys().cloned().collect();

        // Sort categories, but keep "0" at the end
        sorted_cats.sort_by(|a, b| {
            if a == "0" {
                std::cmp::Ordering::Greater
            } else if b == "0" {
                std::cmp::Ordering::Less
            } else {
                a.cmp(b)
            }
        });

        // Add items to menu
        for cat in &sorted_cats {
            let items = categories.get(cat).unwrap();

            // Categories with _ become submenus (except "0")
            if cat.contains('_') && cat != "0" {
                // Cache submenu items
                self.menu_categories.insert(cat.clone(), items.clone());

                // Add submenu entry to main menu
                let cat_name = cat.split('_').nth(1).unwrap_or(cat).replace('-', " ");
                let cat_name = cat_name
                    .chars()
                    .next()
                    .map(|c| c.to_uppercase().to_string())
                    .unwrap_or_default()
                    + &cat_name[1..];
                menu_items.push(crate::data::ui_state::PopupMenuItem {
                    text: format!("{} >", cat_name),
                    command: format!("__SUBMENU__{}", cat),
                    disabled: false,
                });
            } else {
                // Add items directly to main menu
                menu_items.extend(items.clone());
            }
        }

        // Create popup menu at last click position (or centered)
        let position = self.last_link_click_pos.unwrap_or((40, 12));

        self.ui_state.popup_menu =
            Some(crate::data::ui_state::PopupMenu::new(menu_items, position));
        self.ui_state.input_mode = crate::data::ui_state::InputMode::Menu;

        tracing::info!(
            "Created context menu with {} items",
            self.ui_state.popup_menu.as_ref().unwrap().get_items().len()
        );
    }

    /// Format menu text by removing @ and # placeholders and substituting %
    fn format_menu_text(menu: &str, secondary_noun: Option<&str>) -> String {
        let mut text = menu.to_string();

        // Substitute % with secondary noun
        if let Some(sec_noun) = secondary_noun {
            text = text.replace('%', sec_noun);
        }

        // Find first @ or #
        if let Some(pos) = text.find(|c| c == '@' || c == '#') {
            let remaining = text[pos + 1..].trim();
            if remaining.is_empty() {
                // Placeholder at end - truncate
                text[..pos].trim_end().to_string()
            } else {
                // Placeholder in middle - remove it but keep rest
                let before = text[..pos].trim_end();
                let after = text[pos + 1..].trim_start();
                if before.is_empty() {
                    after.to_string()
                } else {
                    format!("{} {}", before, after)
                }
            }
        } else {
            text
        }
    }

    /// Request context menu for a link
    /// Returns the _menu command to send to the server
    pub fn request_menu(
        &mut self,
        exist_id: String,
        noun: String,
        click_pos: (u16, u16),
    ) -> String {
        // Increment counter
        self.menu_request_counter += 1;
        let counter = self.menu_request_counter;

        // Store pending request
        self.pending_menu_requests.insert(
            counter.to_string(),
            PendingMenuRequest {
                exist_id: exist_id.clone(),
                noun,
            },
        );

        // Store click position for menu placement
        self.last_link_click_pos = Some(click_pos);

        // Return command to send to server
        format!("_menu #{} {}\n", exist_id, counter)
    }

    /// Mark layout as modified and show reminder (once per session)
    pub fn mark_layout_modified(&mut self) {
        self.layout_modified_since_save = true;

        // Show reminder once per session
        if !self.save_reminder_shown {
            self.add_system_message(
                "Tip: Use .savelayout <name> to preserve changes as a reusable template",
            );
            self.save_reminder_shown = true;
        }
    }

    /// Quit the application
    pub fn quit(&mut self) {
        // Show reminder if layout was modified
        if self.layout_modified_since_save {
            self.add_system_message(
                "Layout modified - use .savelayout <name> to create reusable template",
            );
        }

        // Autosave to character-specific layout.toml (if character is set)
        if let Some(ref character) = self.config.character {
            let terminal_size = self
                .layout
                .terminal_width
                .and_then(|w| self.layout.terminal_height.map(|h| (w, h)));

            let base_layout_name = self
                .base_layout_name
                .clone()
                .or_else(|| self.layout.base_layout.clone())
                .unwrap_or_else(|| "default".to_string());

            self.layout.theme = Some(self.config.active_theme.clone());
            if let Err(e) = self
                .layout
                .save_auto(character, &base_layout_name, terminal_size)
            {
                tracing::warn!("Failed to autosave layout on quit: {}", e);
            } else {
                tracing::info!(
                    "Layout autosaved to character profile '{}' (base: {}, terminal: {:?})",
                    character,
                    base_layout_name,
                    terminal_size
                );
            }
        } else {
            // No character set - save to default profile: ~/.two-face/default/layout.toml
            let terminal_size = self
                .layout
                .terminal_width
                .and_then(|w| self.layout.terminal_height.map(|h| (w, h)));

            let base_layout_name = self
                .base_layout_name
                .clone()
                .or_else(|| self.layout.base_layout.clone())
                .unwrap_or_else(|| "default".to_string());

            self.layout.theme = Some(self.config.active_theme.clone());
            if let Err(e) = self
                .layout
                .save_auto("default", &base_layout_name, terminal_size)
            {
                tracing::warn!("Failed to autosave layout on quit: {}", e);
            } else {
                tracing::info!(
                    "Layout autosaved to default profile (base: {}, terminal: {:?})",
                    base_layout_name,
                    terminal_size
                );
            }
        }

        self.running = false;
    }

    /// Save configuration to disk
    pub fn save_config(&mut self) -> Result<()> {
        self.config.save(self.config.character.as_deref())
    }

    /// Start search mode (Ctrl+F)
    pub fn start_search_mode(&mut self) {
        self.ui_state.input_mode = crate::data::ui_state::InputMode::Search;
        self.ui_state.search_input.clear();
        self.ui_state.search_cursor = 0;
        self.needs_render = true;
    }

    /// Get the focused window name (or "main" as default)
    pub fn get_focused_window_name(&self) -> String {
        self.ui_state
            .focused_window
            .clone()
            .unwrap_or_else(|| "main".to_string())
    }

    /// Clear search mode
    pub fn clear_search_mode(&mut self) {
        // Exit search mode
        if self.ui_state.input_mode == crate::data::ui_state::InputMode::Search {
            self.ui_state.input_mode = crate::data::ui_state::InputMode::Normal;
        }

        self.ui_state.search_input.clear();
        self.ui_state.search_cursor = 0;
        self.needs_render = true;
    }

    // ========== Menu Building Methods ==========

    /// Build the top-level "Add Window" menu showing widget categories
    pub fn build_add_window_menu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        let categories_map = crate::config::Config::get_addable_templates_by_category(&self.layout);

        // Sort categories for consistent display
        let mut categories: Vec<_> = categories_map.into_iter().collect();
        categories.sort_by_key(|(cat, _)| cat.clone());

        categories
            .into_iter()
            .map(
                |(category, _templates)| crate::data::ui_state::PopupMenuItem {
                    text: category.display_name().to_string(),
                    command: format!("__SUBMENU_ADD__{:?}", category),
                    disabled: false,
                },
            )
            .collect()
    }

    /// Build category submenu showing available windows of that type
    pub fn build_add_window_category_menu(
        &self,
        category: &crate::config::WidgetCategory,
    ) -> Vec<crate::data::ui_state::PopupMenuItem> {
        let categories_map = crate::config::Config::get_addable_templates_by_category(&self.layout);

        if let Some(templates) = categories_map.get(category) {
            templates
                .iter()
                .map(|name| crate::data::ui_state::PopupMenuItem {
                    text: self.get_window_display_name(name),
                    command: format!("__ADD__{}", name),
                    disabled: false,
                })
                .collect()
        } else {
            vec![]
        }
    }

    /// Build "Hide Window" menu showing currently visible windows
    pub fn build_hide_window_menu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        let visible = crate::config::Config::list_visible_windows(&self.layout);

        visible
            .into_iter()
            // Filter out essential windows that should never be hidden
            .filter(|name| name != "story" && name != "command_input")
            .map(|name| crate::data::ui_state::PopupMenuItem {
                text: self.get_window_display_name(&name),
                command: format!("__HIDE__{}", name),
                disabled: false,
            })
            .collect()
    }

    /// Build "Edit Window" menu showing currently visible windows
    pub fn build_edit_window_menu(&self) -> Vec<crate::data::ui_state::PopupMenuItem> {
        let visible = crate::config::Config::list_visible_windows(&self.layout);

        visible
            .into_iter()
            .map(|name| crate::data::ui_state::PopupMenuItem {
                text: self.get_window_display_name(&name),
                command: format!("__EDIT__{}", name),
                disabled: false,
            })
            .collect()
    }

    /// Get display name for a window (uses title from template, or falls back to name)
    fn get_window_display_name(&self, name: &str) -> String {
        crate::config::Config::get_window_template(name)
            .and_then(|t| t.base().title.clone())
            .unwrap_or_else(|| name.to_string())
    }

    /// Check if text matches any highlight patterns with sounds and play them
    pub fn check_sound_triggers(&self, text: &str) {
        if let Some(ref sound_player) = self.sound_player {
            for (_name, pattern) in &self.config.highlights {
                // Skip if no sound configured for this pattern
                if pattern.sound.is_none() {
                    continue;
                }

                let matches = if pattern.fast_parse {
                    // Fast parse: check if any of the pipe-separated patterns are in the text
                    pattern.pattern.split('|').any(|p| text.contains(p.trim()))
                } else {
                    // Regex parse
                    if let Ok(regex) = regex::Regex::new(&pattern.pattern) {
                        regex.is_match(text)
                    } else {
                        false
                    }
                };

                if matches {
                    if let Some(ref sound_file) = pattern.sound {
                        // Play the sound
                        if let Err(e) =
                            sound_player.play_from_sounds_dir(sound_file, pattern.sound_volume)
                        {
                            tracing::warn!("Failed to play sound '{}': {}", sound_file, e);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Layout, WindowBase, WindowDef, SpacerWidgetData, BorderSides};

    // Test helper to create a minimal WindowBase
    fn test_window_base(name: &str) -> WindowBase {
        WindowBase {
            name: name.to_string(),
            row: 0,
            col: 0,
            rows: 2,
            cols: 5,
            show_border: false,
            border_style: "single".to_string(),
            border_sides: BorderSides::default(),
            border_color: None,
            show_title: false,
            title: None,
            background_color: None,
            text_color: None,
            transparent_background: true,
            locked: false,
            min_rows: None,
            max_rows: None,
            min_cols: None,
            max_cols: None,
            visible: true,
        }
    }

    #[test]
    fn test_generate_spacer_name_empty_layout() {
        // RED: With no spacers, should return spacer_1
        let layout = Layout {
            windows: vec![],
            terminal_width: None,
            terminal_height: None,
            base_layout: None,
            theme: None,
        };

        let name = AppCore::generate_spacer_name(&layout);
        assert_eq!(name, "spacer_1");
    }

    #[test]
    fn test_generate_spacer_name_single_spacer() {
        // RED: With one spacer_1, should return spacer_2
        let spacer1 = WindowDef::Spacer {
            base: test_window_base("spacer_1"),
            data: SpacerWidgetData {},
        };
        let layout = Layout {
            windows: vec![spacer1],
            terminal_width: None,
            terminal_height: None,
            base_layout: None,
            theme: None,
        };

        let name = AppCore::generate_spacer_name(&layout);
        assert_eq!(name, "spacer_2");
    }

    #[test]
    fn test_generate_spacer_name_multiple_spacers() {
        // RED: With spacer_1, spacer_2, spacer_3, should return spacer_4
        let spacer1 = WindowDef::Spacer {
            base: test_window_base("spacer_1"),
            data: SpacerWidgetData {},
        };
        let spacer2 = WindowDef::Spacer {
            base: test_window_base("spacer_2"),
            data: SpacerWidgetData {},
        };
        let spacer3 = WindowDef::Spacer {
            base: test_window_base("spacer_3"),
            data: SpacerWidgetData {},
        };
        let layout = Layout {
            windows: vec![spacer1, spacer2, spacer3],
            terminal_width: None,
            terminal_height: None,
            base_layout: None,
            theme: None,
        };

        let name = AppCore::generate_spacer_name(&layout);
        assert_eq!(name, "spacer_4");
    }

    #[test]
    fn test_generate_spacer_name_with_gaps() {
        // RED: With spacer_1 and spacer_3 (gap at 2), should return spacer_4 (max + 1)
        let spacer1 = WindowDef::Spacer {
            base: test_window_base("spacer_1"),
            data: SpacerWidgetData {},
        };
        let spacer3 = WindowDef::Spacer {
            base: test_window_base("spacer_3"),
            data: SpacerWidgetData {},
        };
        let layout = Layout {
            windows: vec![spacer1, spacer3],
            terminal_width: None,
            terminal_height: None,
            base_layout: None,
            theme: None,
        };

        let name = AppCore::generate_spacer_name(&layout);
        assert_eq!(name, "spacer_4");
    }

    #[test]
    fn test_generate_spacer_name_ignores_non_spacers() {
        // RED: Non-spacer widgets should be ignored
        let text_widget = WindowDef::Text {
            base: test_window_base("main"),
            data: crate::config::TextWidgetData {
                streams: vec!["main".to_string()],
                buffer_size: 1000,
            },
        };
        let spacer1 = WindowDef::Spacer {
            base: test_window_base("spacer_1"),
            data: SpacerWidgetData {},
        };
        let layout = Layout {
            windows: vec![text_widget, spacer1],
            terminal_width: None,
            terminal_height: None,
            base_layout: None,
            theme: None,
        };

        let name = AppCore::generate_spacer_name(&layout);
        assert_eq!(name, "spacer_2");
    }

    #[test]
    fn test_generate_spacer_name_with_hidden_spacers() {
        // RED: Hidden spacers should be considered (widgets can be hidden, not deleted)
        let mut visible_base = test_window_base("spacer_1");
        visible_base.visible = true;

        let mut hidden_base = test_window_base("spacer_2");
        hidden_base.visible = false;

        let visible_spacer = WindowDef::Spacer {
            base: visible_base,
            data: SpacerWidgetData {},
        };
        let hidden_spacer = WindowDef::Spacer {
            base: hidden_base,
            data: SpacerWidgetData {},
        };
        let layout = Layout {
            windows: vec![visible_spacer, hidden_spacer],
            terminal_width: None,
            terminal_height: None,
            base_layout: None,
            theme: None,
        };

        let name = AppCore::generate_spacer_name(&layout);
        assert_eq!(name, "spacer_3");
    }

    #[test]
    fn test_generate_spacer_name_non_sequential() {
        // RED: With spacer_2, spacer_5 (max is 5), should return spacer_6
        let spacer2 = WindowDef::Spacer {
            base: test_window_base("spacer_2"),
            data: SpacerWidgetData {},
        };
        let spacer5 = WindowDef::Spacer {
            base: test_window_base("spacer_5"),
            data: SpacerWidgetData {},
        };
        let layout = Layout {
            windows: vec![spacer2, spacer5],
            terminal_width: None,
            terminal_height: None,
            base_layout: None,
            theme: None,
        };

        let name = AppCore::generate_spacer_name(&layout);
        assert_eq!(name, "spacer_6");
    }

    #[test]
    fn test_generate_spacer_name_large_numbers() {
        // RED: Should handle large numbers correctly
        let spacer99 = WindowDef::Spacer {
            base: test_window_base("spacer_99"),
            data: SpacerWidgetData {},
        };
        let layout = Layout {
            windows: vec![spacer99],
            terminal_width: None,
            terminal_height: None,
            base_layout: None,
            theme: None,
        };

        let name = AppCore::generate_spacer_name(&layout);
        assert_eq!(name, "spacer_100");
    }
}
