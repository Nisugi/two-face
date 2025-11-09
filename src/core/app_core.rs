use crate::cmdlist::CmdList;
use crate::config::{Config, KeyAction, Layout};
use crate::network::ServerMessage;
use crate::parser::{ParsedElement, XmlParser};
use crate::performance::PerformanceStats;
use crate::sound::SoundPlayer;
use crate::ui::WindowManager;
use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use std::collections::HashMap;
use tracing::info;

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
    /// TODO: Move implementation from App::handle_server_message()
    pub fn handle_server_message(&mut self, _msg: ServerMessage) -> Result<()> {
        // TODO: Implementation will be moved from App in next step
        Ok(())
    }

    /// Handle a dot command (local command not sent to server)
    ///
    /// Dot commands are local to the client (e.g., .quit, .addwindow, .settings)
    ///
    /// TODO: Move implementation from App::handle_dot_command()
    pub fn handle_dot_command(&mut self, _command: &str) -> Result<()> {
        // TODO: Implementation will be moved from App in next step
        Ok(())
    }

    /// Add a system message to the main window
    ///
    /// System messages are local notifications (e.g., "Layout saved", "Window created")
    ///
    /// TODO: Move implementation from App::add_system_message()
    pub fn add_system_message(&mut self, _message: &str) {
        // TODO: Implementation will be moved from App in next step
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
    /// TODO: Move implementation from App::update_window_manager_config()
    pub fn update_window_manager_config(&mut self) -> Result<()> {
        // TODO: Implementation will be moved from App in next step
        Ok(())
    }
}
