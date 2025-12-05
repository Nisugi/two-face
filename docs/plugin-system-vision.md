# Two-Face Plugin System Vision

## Overview

The main.rs refactoring creates the architectural foundation for a powerful plugin system that allows users and third-party developers to extend Two-Face with custom widgets, game-specific features, and integrations—all without modifying the core codebase.

This document explores the possibilities, design patterns, and ecosystem potential of a Two-Face plugin architecture.

---

## Why Plugins?

### Current Limitation
- All widgets are hardcoded into main.rs
- Adding new features requires modifying core codebase
- Game-specific functionality bloats the binary
- No way for community to contribute custom tools

### Post-Refactoring Opportunity
- Widgets become self-contained, autonomous components
- Clean interfaces (`InputHandler`, `Widget` traits)
- Factory pattern enables dynamic widget loading
- Clear separation: core = orchestration, plugins = features

---

## Plugin Categories

### 1. **Custom Game Windows**

Specialized windows for specific games or tasks that render alongside standard windows.

#### Examples

**Spell Tracker Widget**
```rust
pub struct SpellTrackerWidget {
    active_spells: Vec<ActiveSpell>,
    expiration_times: HashMap<String, Instant>,
    selected: usize,
}

struct ActiveSpell {
    name: String,
    duration: Duration,
    started_at: Instant,
}

impl Widget for SpellTrackerWidget {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Render spell list with countdown timers
        for (i, spell) in self.active_spells.iter().enumerate() {
            let remaining = spell.duration.saturating_sub(spell.started_at.elapsed());
            let line = format!("{}: {}s", spell.name, remaining.as_secs());
            // Render with color coding: green = >30s, yellow = 10-30s, red = <10s
        }
    }
}

impl MessageProcessor for SpellTrackerWidget {
    fn process_message(&mut self, msg: &str) -> ProcessResult {
        if let Some(spell) = parse_spell_activation(msg) {
            self.active_spells.push(spell);
        }
        if let Some(spell_name) = parse_spell_expiration(msg) {
            self.active_spells.retain(|s| s.name != spell_name);
        }
        ProcessResult::Passthrough
    }
}
```

**Use Cases**:
- **Spell management**: Track active spells, cooldowns, durations
- **Buff/debuff monitor**: Visual timers for active effects
- **Combat tracker**: Real-time combat state visualization
- **Quest tracker**: Track objectives and progress

**Inventory Manager Widget**
```rust
pub struct InventoryWidget {
    items: Vec<Item>,
    filter: String,
    sort_by: SortField,
    selected: usize,
}

impl InputHandler for InventoryWidget {
    fn handle_key(&mut self, event: KeyEvent) -> WidgetResult {
        match event.code {
            KeyCode::Char(c) => {
                self.filter.push(c);
                self.apply_filter();
                WidgetResult::Handled
            }
            KeyCode::Enter => {
                if let Some(item) = self.get_selected() {
                    self.send_command(&format!("get {}", item.name));
                }
                WidgetResult::Handled
            }
            _ => WidgetResult::NotHandled
        }
    }
}
```

**Use Cases**:
- **Search inventory**: Type-ahead filtering of items
- **Sort/organize**: By weight, value, type, location
- **Quick actions**: Drop, stow, sell with keyboard shortcuts
- **Capacity tracking**: Visual weight/count limits

**Character Stats Dashboard**
```rust
pub struct StatsDashboardWidget {
    stats: CharacterStats,
    combat_stats: CombatStatistics,
    session_start: Instant,
}

struct CharacterStats {
    health: (u32, u32),      // current, max
    mana: (u32, u32),
    stamina: (u32, u32),
    experience: u64,
    level: u32,
}

struct CombatStatistics {
    damage_dealt: u64,
    damage_taken: u64,
    kills: u32,
    deaths: u32,
    dps: f32,               // damage per second
}
```

**Use Cases**:
- **Real-time stats**: Health, mana, stamina bars
- **Combat analytics**: DPS meter, damage breakdown
- **Session tracking**: XP/hour, kills/hour
- **Goal progress**: Level progress, skill training

---

### 2. **Custom Editors & Browsers**

Extend the configuration system with specialized editors for advanced features.

#### Examples

**Macro Editor Widget**
```rust
pub struct MacroEditorWidget {
    macros: HashMap<String, Macro>,
    selected_macro: Option<String>,
    recording: bool,
    current_recording: Vec<String>,
}

struct Macro {
    name: String,
    commands: Vec<String>,
    keybind: Option<String>,
    description: String,
}

impl FormWidget for MacroEditorWidget {
    fn save(&mut self) -> Result<FormData> {
        // Save macros to ~/.two-face/macros.toml
        Config::save_macros(&self.macros)?;
        Ok(FormData::Macros(self.macros.clone()))
    }
}
```

**Use Cases**:
- **Command macros**: Record and replay command sequences
- **Quick actions**: One-key complex operations
- **Conditional macros**: Execute based on game state
- **Import/export**: Share macros with community

**Trigger Builder Widget**
```rust
pub struct TriggerBuilderWidget {
    triggers: Vec<Trigger>,
    editor_state: TriggerEditorState,
}

struct Trigger {
    name: String,
    pattern: Regex,
    actions: Vec<TriggerAction>,
    enabled: bool,
    cooldown: Option<Duration>,
}

enum TriggerAction {
    SendCommand(String),
    PlaySound(PathBuf),
    ShowNotification(String),
    RunPlugin(String),
}
```

**Use Cases**:
- **Visual trigger editor**: No regex knowledge required
- **Action chains**: Multiple actions per trigger
- **Trigger testing**: Test patterns against sample text
- **Import from Lich**: Convert Lich triggers

**Layout Designer Widget**
```rust
pub struct LayoutDesignerWidget {
    layout: Layout,
    selected_window: Option<String>,
    drag_mode: bool,
    preview_mode: bool,
}

impl InputHandler for LayoutDesignerWidget {
    fn handle_mouse(&mut self, event: MouseEvent) -> WidgetResult {
        match event.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.select_window_at(event.column, event.row);
                self.drag_mode = true;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                if self.drag_mode {
                    self.move_selected_window(event.column, event.row);
                }
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.drag_mode = false;
            }
            _ => {}
        }
        WidgetResult::Handled
    }
}
```

**Use Cases**:
- **Visual layout editor**: Drag-and-drop window positioning
- **Live preview**: See changes in real-time
- **Template library**: Pre-built layouts for common setups
- **Responsive layouts**: Define layouts for different terminal sizes

---

### 3. **Game Analysis & Visualization**

Data collection, analytics, and visualization tools.

#### Examples

**Combat Logger Widget**
```rust
pub struct CombatLoggerWidget {
    encounters: Vec<Encounter>,
    current_encounter: Option<Encounter>,
    statistics: CombatStatistics,
    chart_type: ChartType,
}

struct Encounter {
    start_time: Instant,
    duration: Duration,
    enemies: Vec<Enemy>,
    damage_dealt: Vec<DamageEvent>,
    damage_taken: Vec<DamageEvent>,
    outcome: EncounterOutcome,
}

enum ChartType {
    DamageOverTime,
    DamageByType,
    DamageByTarget,
    EncounterTimeline,
}

impl Widget for CombatLoggerWidget {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        match self.chart_type {
            ChartType::DamageOverTime => self.render_line_chart(area, buf),
            ChartType::DamageByType => self.render_pie_chart(area, buf),
            ChartType::DamageByTarget => self.render_bar_chart(area, buf),
            ChartType::EncounterTimeline => self.render_timeline(area, buf),
        }
    }
}
```

**Use Cases**:
- **DPS meter**: Real-time damage per second tracking
- **Combat analysis**: Breakdown by damage type, target, time
- **Performance trends**: Compare encounters, identify improvements
- **Export reports**: CSV/JSON for external analysis

**Experience Tracker Widget**
```rust
pub struct ExperienceTrackerWidget {
    session_start: Instant,
    initial_exp: u64,
    current_exp: u64,
    exp_events: Vec<ExpEvent>,
    predictions: ExpPredictions,
}

struct ExpEvent {
    timestamp: Instant,
    amount: u64,
    source: String,  // "combat", "quest", "task", etc.
}

struct ExpPredictions {
    exp_per_hour: f64,
    time_to_next_level: Duration,
    estimated_level_at: HashMap<Duration, u32>,  // "In 1 hour: level 45"
}

impl Widget for ExperienceTrackerWidget {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let total_exp = self.current_exp - self.initial_exp;
        let session_duration = self.session_start.elapsed();
        let exp_per_hour = (total_exp as f64 / session_duration.as_secs_f64()) * 3600.0;

        // Render graph showing exp over time
        // Show predictions: "Level 50 in 3h 27m at current rate"
    }
}
```

**Use Cases**:
- **XP tracking**: Session XP, XP/hour, source breakdown
- **Level predictions**: Time to next level at current rate
- **Goal tracking**: "How long to level 100?"
- **Efficiency analysis**: Compare different grinding methods

**Map Visualization Widget**
```rust
pub struct MapWidget {
    rooms: HashMap<String, Room>,
    current_room: String,
    path_history: VecDeque<String>,
    minimap_mode: bool,
}

struct Room {
    id: String,
    name: String,
    description: String,
    exits: HashMap<Direction, String>,
    position: (i32, i32),  // For visualization
    visited_count: u32,
}

impl MessageProcessor for MapWidget {
    fn process_message(&mut self, msg: &str) -> ProcessResult {
        if let Some(room) = parse_room_description(msg) {
            self.current_room = room.id.clone();
            self.rooms.insert(room.id.clone(), room);
            self.path_history.push_back(room.id);
        }
        ProcessResult::Passthrough
    }
}
```

**Use Cases**:
- **Auto-mapping**: Build map as you explore
- **Minimap**: Visual representation of surroundings
- **Path finding**: "How do I get to town?"
- **Map sharing**: Export/import maps with community

---

### 4. **External Integrations**

Connect Two-Face to external services and tools.

#### Examples

**Discord Notifier Widget**
```rust
pub struct DiscordNotifierWidget {
    webhook_url: String,
    triggers: Vec<NotificationTrigger>,
    rate_limiter: RateLimiter,
}

struct NotificationTrigger {
    pattern: Regex,
    message_template: String,
    enabled: bool,
    priority: NotificationPriority,
}

impl MessageProcessor for DiscordNotifierWidget {
    fn process_message(&mut self, msg: &str) -> ProcessResult {
        for trigger in &self.triggers {
            if trigger.enabled && trigger.pattern.is_match(msg) {
                if self.rate_limiter.check_rate_limit() {
                    let notification = trigger.format_message(msg);
                    self.send_webhook(notification);
                }
            }
        }
        ProcessResult::Passthrough
    }
}
```

**Use Cases**:
- **Death notifications**: Alert Discord when you die
- **Rare item drops**: Notify on valuable finds
- **Quest completion**: Share achievements
- **AFK monitoring**: Get pinged when action needed

**Web Dashboard Widget**
```rust
pub struct WebDashboardWidget {
    server: WebServer,
    port: u16,
    enabled: bool,
}

impl WebDashboardWidget {
    fn serve_dashboard(&self) {
        // Serve web interface at http://localhost:8080
        // Show: current stats, combat log, inventory, map
        // Allow: remote monitoring from phone/tablet
    }
}
```

**Use Cases**:
- **Remote monitoring**: Check game status from phone
- **Multi-screen setup**: Stats on second monitor
- **Streaming overlays**: OBS integration
- **Data sharing**: Publish stats to personal website

**Lich Script Integration Widget**
```rust
pub struct LichIntegrationWidget {
    script_runner: ScriptRunner,
    active_scripts: Vec<String>,
    script_output: HashMap<String, Vec<String>>,
}

impl InputHandler for LichIntegrationWidget {
    fn handle_action(&mut self, action: MenuAction) -> WidgetResult {
        match action {
            MenuAction::Activate => {
                if let Some(script) = self.get_selected_script() {
                    self.script_runner.run_script(script)?;
                }
                WidgetResult::Handled
            }
            MenuAction::Stop => {
                self.script_runner.stop_all_scripts()?;
                WidgetResult::Handled
            }
            _ => WidgetResult::NotHandled
        }
    }
}
```

**Use Cases**:
- **Script browser**: List and run Lich scripts
- **Script output**: Capture script messages separately
- **Script management**: Start, stop, monitor scripts
- **Hybrid mode**: Use Two-Face UI with Lich backend

---

### 5. **Developer & Debug Tools**

Tools for script developers and power users.

#### Examples

**Trigger Debugger Widget**
```rust
pub struct TriggerDebuggerWidget {
    test_input: String,
    triggers: Vec<Trigger>,
    match_results: Vec<MatchResult>,
    regex_tester: RegexTester,
}

struct MatchResult {
    trigger_name: String,
    matched: bool,
    captures: Vec<String>,
    execution_time: Duration,
}

impl InputHandler for TriggerDebuggerWidget {
    fn handle_action(&mut self, action: MenuAction) -> WidgetResult {
        match action {
            MenuAction::Test => {
                self.match_results = self.test_triggers(&self.test_input);
                WidgetResult::Handled
            }
            _ => WidgetResult::NotHandled
        }
    }
}
```

**Use Cases**:
- **Test triggers**: See which triggers match test input
- **Regex debugging**: Visual regex testing and capture groups
- **Performance profiling**: Identify slow triggers
- **Trigger conflicts**: Find overlapping patterns

**Network Monitor Widget**
```rust
pub struct NetworkMonitorWidget {
    packets: VecDeque<Packet>,
    filter: PacketFilter,
    statistics: NetworkStatistics,
}

struct Packet {
    timestamp: Instant,
    direction: Direction,
    content: String,
    size: usize,
}

struct NetworkStatistics {
    bytes_received: u64,
    bytes_sent: u64,
    packets_per_second: f64,
    average_latency: Duration,
}
```

**Use Cases**:
- **Traffic analysis**: Monitor bandwidth usage
- **Latency tracking**: Measure server response times
- **Protocol debugging**: Inspect raw game messages
- **Trigger testing**: See raw messages before processing

**Performance Profiler Widget**
```rust
pub struct ProfilerWidget {
    frame_times: VecDeque<Duration>,
    render_times: VecDeque<Duration>,
    event_processing_times: VecDeque<Duration>,
    memory_usage: VecDeque<usize>,
}

impl Widget for ProfilerWidget {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // Render performance graphs
        // Show: FPS, frame time, memory usage, CPU usage
        // Identify: performance bottlenecks, memory leaks
    }
}
```

**Use Cases**:
- **Performance monitoring**: FPS, frame times, lag spikes
- **Memory profiling**: Track memory usage over time
- **Bottleneck identification**: Find slow operations
- **Optimization testing**: Measure improvement impact

---

## Plugin Architecture

### Core Traits

```rust
/// Core widget interface
pub trait Widget: Send + Sync {
    /// Render the widget in the given area
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Get widget title (for window decoration)
    fn title(&self) -> &str;

    /// Called when widget is opened
    fn on_open(&mut self) {}

    /// Called when widget is closed
    fn on_close(&mut self) {}
}

/// Input handling interface
pub trait InputHandler {
    /// Handle a menu action (navigation, activation, etc.)
    fn handle_action(&mut self, action: MenuAction) -> WidgetResult;

    /// Handle raw keyboard input
    fn handle_key(&mut self, event: KeyEvent) -> WidgetResult;
}

/// Message processing interface
pub trait MessageProcessor {
    /// Process a server message
    fn process_message(&mut self, msg: &str) -> ProcessResult;
}

/// Results from widget operations
pub enum WidgetResult {
    Handled,                    // Event was handled
    NotHandled,                 // Pass to next handler
    Close,                      // Close this widget
    SaveAndClose(Box<dyn Any>), // Save data and close
}

pub enum ProcessResult {
    Passthrough,           // Let message continue normally
    Consumed,              // Suppress message from other windows
    Modified(String),      // Replace message with modified version
}
```

### Plugin Descriptor

```rust
/// Plugin metadata and factory
pub trait WidgetPlugin: Send + Sync {
    /// Plugin name (unique identifier)
    fn name(&self) -> &str;

    /// Plugin version (semver)
    fn version(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// Create a new widget instance
    fn create(&self) -> Box<dyn Widget>;

    /// Menu action that opens this widget
    fn menu_action(&self) -> MenuAction;

    /// Required permissions
    fn permissions(&self) -> PluginPermissions;
}

pub struct PluginPermissions {
    pub network: bool,           // Can make network requests
    pub filesystem: FileAccess,  // File system access level
    pub commands: bool,          // Can send game commands
    pub message_intercept: bool, // Can intercept/modify messages
}

pub enum FileAccess {
    None,
    ReadOnly,
    ReadWrite,
    PluginDataOnly,  // Only ~/.two-face/plugins/<name>/data/
}
```

### Plugin Loading

```rust
pub struct PluginRegistry {
    plugins: HashMap<String, Box<dyn WidgetPlugin>>,
    plugin_widgets: HashMap<String, Box<dyn Widget>>,
}

impl PluginRegistry {
    /// Load plugins from directory
    pub fn load_from_directory(&mut self, path: &Path) -> Result<Vec<PluginLoadResult>> {
        let mut results = Vec::new();

        for entry in fs::read_dir(path)? {
            let path = entry?.path();

            if path.extension() == Some(OsStr::new("so"))
                || path.extension() == Some(OsStr::new("dll"))
                || path.extension() == Some(OsStr::new("dylib")) {

                match self.load_plugin(&path) {
                    Ok(plugin) => {
                        self.plugins.insert(plugin.name().to_string(), plugin);
                        results.push(PluginLoadResult::Success(path));
                    }
                    Err(e) => {
                        results.push(PluginLoadResult::Failed(path, e));
                    }
                }
            }
        }

        Ok(results)
    }

    /// Dynamically load a plugin library
    unsafe fn load_plugin(&mut self, path: &Path) -> Result<Box<dyn WidgetPlugin>> {
        let lib = Library::new(path)?;

        // Look for exported symbol: create_plugin
        let create_plugin: Symbol<fn() -> Box<dyn WidgetPlugin>> =
            lib.get(b"create_plugin")?;

        let plugin = create_plugin();

        // Validate plugin
        self.validate_plugin(&plugin)?;

        Ok(plugin)
    }

    fn validate_plugin(&self, plugin: &dyn WidgetPlugin) -> Result<()> {
        // Check name uniqueness
        if self.plugins.contains_key(plugin.name()) {
            return Err(anyhow!("Plugin '{}' already loaded", plugin.name()));
        }

        // Validate version format
        semver::Version::parse(plugin.version())?;

        // Check permissions against security policy
        self.check_permissions(plugin.permissions())?;

        Ok(())
    }
}
```

---

## Plugin Configuration

### Plugin Manifest

Each plugin includes a manifest file describing its metadata and configuration.

```toml
# ~/.two-face/plugins/spell-tracker/plugin.toml

[plugin]
name = "spell-tracker"
version = "1.2.0"
author = "community-developer"
description = "Track active spells with countdown timers"
homepage = "https://github.com/user/spell-tracker"
license = "MIT"

[permissions]
network = false
filesystem = "plugin-data-only"
commands = false
message_intercept = true

[dependencies]
two-face-api = "0.1.0"

[config]
default_window_size = { width = 30, height = 15 }
update_interval_ms = 100
color_scheme = "default"

[[keybinds]]
action = "open"
default = "F5"

[[keybinds]]
action = "toggle_visibility"
default = "Shift+F5"

[data]
# Plugin-specific configuration
spell_database = "spells.json"
known_durations = { "Spirit Warding I" = 1200, "Elemental Blade" = 600 }
```

### User Configuration

Users can override plugin settings in their config.

```toml
# ~/.two-face/config.toml

[plugins.spell-tracker]
enabled = true
keybind = "F5"
window_position = { x = 50, y = 10, width = 30, height = 15 }
auto_start = false

[plugins.spell-tracker.config]
update_interval_ms = 50  # Override default
color_scheme = "custom"

[plugins.combat-logger]
enabled = true
keybind = "F6"
auto_start = true  # Start logging on combat start

[plugins.combat-logger.config]
max_encounters = 100
export_format = "json"
```

---

## Plugin Development

### Minimal Plugin Example

```rust
// spell-tracker/src/lib.rs

use two_face_plugin_api::*;

pub struct SpellTracker {
    spells: Vec<ActiveSpell>,
    selected: usize,
}

struct ActiveSpell {
    name: String,
    expires_at: Instant,
}

impl Widget for SpellTracker {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        for (i, spell) in self.spells.iter().enumerate() {
            let remaining = spell.expires_at.saturating_duration_since(Instant::now());
            let style = if i == self.selected {
                Style::default().bg(Color::Blue)
            } else {
                Style::default()
            };

            buf.set_string(
                area.x,
                area.y + i as u16,
                format!("{}: {}s", spell.name, remaining.as_secs()),
                style
            );
        }
    }

    fn title(&self) -> &str {
        "Active Spells"
    }
}

impl InputHandler for SpellTracker {
    fn handle_action(&mut self, action: MenuAction) -> WidgetResult {
        match action {
            MenuAction::NavigateUp => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                WidgetResult::Handled
            }
            MenuAction::NavigateDown => {
                if self.selected < self.spells.len().saturating_sub(1) {
                    self.selected += 1;
                }
                WidgetResult::Handled
            }
            MenuAction::Cancel => WidgetResult::Close,
            _ => WidgetResult::NotHandled
        }
    }

    fn handle_key(&mut self, _event: KeyEvent) -> WidgetResult {
        WidgetResult::NotHandled
    }
}

impl MessageProcessor for SpellTracker {
    fn process_message(&mut self, msg: &str) -> ProcessResult {
        // Simple pattern matching (in real plugin, use regex)
        if msg.contains("You gesture") {
            // Extract spell name and duration from message
            // Add to active spells list
        }

        ProcessResult::Passthrough
    }
}

pub struct SpellTrackerPlugin;

impl WidgetPlugin for SpellTrackerPlugin {
    fn name(&self) -> &str {
        "spell-tracker"
    }

    fn version(&self) -> &str {
        env!("CARGO_PKG_VERSION")
    }

    fn description(&self) -> &str {
        "Track active spells with countdown timers"
    }

    fn create(&self) -> Box<dyn Widget> {
        Box::new(SpellTracker {
            spells: Vec::new(),
            selected: 0,
        })
    }

    fn menu_action(&self) -> MenuAction {
        MenuAction::Plugin("spell-tracker".to_string())
    }

    fn permissions(&self) -> PluginPermissions {
        PluginPermissions {
            network: false,
            filesystem: FileAccess::PluginDataOnly,
            commands: false,
            message_intercept: true,
        }
    }
}

#[no_mangle]
pub extern "C" fn create_plugin() -> Box<dyn WidgetPlugin> {
    Box::new(SpellTrackerPlugin)
}
```

### Plugin SDK Tools

```bash
# Create new plugin from template
$ two-face plugin new spell-tracker

# Build plugin
$ cd spell-tracker
$ cargo build --release

# Install plugin locally
$ two-face plugin install target/release/libspell_tracker.so

# Test plugin
$ two-face plugin test spell-tracker

# Package plugin for distribution
$ two-face plugin package
# Creates: spell-tracker-1.2.0.tpkg

# Publish to plugin registry
$ two-face plugin publish spell-tracker-1.2.0.tpkg
```

---

## Security & Sandboxing

### Permission System

Plugins declare required permissions, and users must approve them.

```rust
pub struct PluginPermissions {
    /// Can make outbound network requests
    pub network: bool,

    /// File system access level
    pub filesystem: FileAccess,

    /// Can send commands to game server
    pub commands: bool,

    /// Can intercept and modify server messages
    pub message_intercept: bool,

    /// Can access clipboard
    pub clipboard: bool,

    /// Can spawn external processes
    pub spawn_process: bool,
}
```

### Approval Flow

```
User: two-face plugin install spell-tracker.so

Two-Face: Plugin 'spell-tracker' requests the following permissions:
  ✓ Read/write plugin data directory
  ✓ Intercept server messages
  ✗ Network access
  ✗ Send game commands

  Allow these permissions? [y/N]
```

### Sandboxing Options

#### 1. Capability-Based Security
```rust
// Plugin can only do what it's explicitly given capability to do
pub struct PluginContext {
    data_dir: PathBuf,
    command_sender: Option<CommandSender>,
    network: Option<NetworkClient>,
}

impl PluginContext {
    pub fn send_command(&self, cmd: &str) -> Result<()> {
        self.command_sender
            .as_ref()
            .ok_or(anyhow!("Plugin doesn't have command permission"))?
            .send(cmd)
    }
}
```

#### 2. WASM Plugins (Future)
Instead of native `.so/.dll`, compile plugins to WebAssembly:
- Fully sandboxed execution
- Cross-platform (same plugin works everywhere)
- Can't access host system unless explicitly granted
- Performance overhead acceptable for most plugins

```rust
// Load WASM plugin
let wasm_bytes = fs::read("spell-tracker.wasm")?;
let plugin = WasmPlugin::new(wasm_bytes)?;
```

---

## Plugin Discovery & Distribution

### Plugin Registry

Central repository of curated plugins.

**Website**: `plugins.two-face.dev`

**Features**:
- Browse plugins by category
- Search and filter
- User ratings and reviews
- Security audit status
- Compatibility information
- Download statistics

### Installation Methods

```bash
# Install from registry
$ two-face plugin install spell-tracker

# Install from URL
$ two-face plugin install https://example.com/plugins/spell-tracker-1.2.0.tpkg

# Install from local file
$ two-face plugin install ./spell-tracker.so

# Install from Git repo
$ two-face plugin install git://github.com/user/spell-tracker

# List installed plugins
$ two-face plugin list

# Update all plugins
$ two-face plugin update

# Remove plugin
$ two-face plugin remove spell-tracker
```

### Plugin Marketplace

**Categories**:
- Combat & Analysis
- Character Management
- Automation & Scripting
- Integrations & Notifications
- Developer Tools
- UI Enhancements
- Game-Specific (GS4, DR, etc.)

**Curation**:
- Security review for featured plugins
- Community moderation
- Verified developer badges
- Automated malware scanning

---

## Community Ecosystem

### Example Plugin Ecosystem

**Official Plugins** (bundled with Two-Face):
- `spell-tracker` - Track active spells
- `basic-stats` - Health/mana/stamina display
- `experience-tracker` - XP monitoring

**Community Plugins**:
- `gsiv-wiki` - In-game wiki browser
- `lich-integration` - Lich script runner
- `discord-bridge` - Discord notifications
- `damage-meter` - Combat DPS tracker
- `inventory-manager` - Advanced inventory tools
- `map-builder` - Auto-mapping system
- `macro-system` - Command macro recorder
- `layout-designer` - Visual layout editor
- `theme-gallery` - Community theme browser
- `trigger-marketplace` - Share triggers
- `script-debugger` - Debug Lich scripts
- `network-analyzer` - Packet inspector
- `performance-monitor` - FPS/latency tracking

### Developer Resources

**Documentation**:
- Plugin API reference
- Tutorial: "Your First Plugin"
- Example plugins repository
- Best practices guide
- Security guidelines

**Support**:
- Plugin developer Discord channel
- GitHub discussions
- Stack Overflow tag: `two-face-plugin`

**Tools**:
- Plugin template generator
- Hot-reload development mode
- Plugin testing framework
- Performance profiler
- Security linter

---

## Implementation Roadmap

### Phase 1: Foundation (Main.rs Refactoring)
**Status**: Current refactoring plan
- ✅ Extract widget input handling to traits
- ✅ Create clean widget lifecycle management
- ✅ Establish widget autonomy
- ✅ Factory pattern for widget creation

**Result**: Clean interfaces that naturally support plugins

### Phase 2: Plugin API Design
**Estimated**: 2-3 months after refactoring
- Define stable `Widget`, `InputHandler`, `MessageProcessor` traits
- Create plugin ABI (stable across Rust versions)
- Design permission system
- Build plugin context API

**Result**: Stable plugin API contract

### Phase 3: Dynamic Loading
**Estimated**: 1-2 months
- Implement `.so/.dll/.dylib` loading
- Plugin discovery from `~/.two-face/plugins/`
- Plugin registry and lifecycle management
- Error handling for malformed plugins

**Result**: Plugins can be loaded at runtime

### Phase 4: Plugin SDK
**Estimated**: 1-2 months
- Create `two-face-plugin-api` crate
- Documentation and tutorials
- Example plugins repository
- Plugin template generator: `two-face plugin new <name>`

**Result**: Easy plugin development experience

### Phase 5: Advanced Features
**Estimated**: Ongoing
- Message interception hooks
- Persistent storage API
- Custom keybind registration
- Inter-plugin communication
- Plugin configuration UI

**Result**: Full-featured plugin system

### Phase 6: Distribution & Ecosystem
**Estimated**: 3-6 months
- Plugin registry website
- Plugin packaging format (`.tpkg`)
- Plugin manager CLI
- Security review process
- Community moderation tools

**Result**: Thriving plugin ecosystem

---

## Success Metrics

### Technical Goals
- Plugin API stability: No breaking changes across minor versions
- Performance: Plugin overhead < 5% in typical usage
- Security: Zero critical vulnerabilities in plugin system
- Compatibility: Plugins work across platforms (Win/Mac/Linux)

### Ecosystem Goals
- 10+ community plugins in first 6 months
- 50+ plugins within 1 year
- At least one plugin with 100+ users
- Active plugin developer community

### User Experience Goals
- Plugin installation in < 30 seconds
- Plugin discovery through built-in browser
- Clear permission system users understand
- Plugins feel like native features

---

## Long-Term Vision

### Two-Face as a Platform

Transform Two-Face from a **game client** into a **platform for game interaction**:

- **Core**: Minimal, stable, well-tested foundation
- **Official Plugins**: Common features maintained by core team
- **Community Plugins**: Niche features, game-specific tools, experimental ideas
- **Plugin Marketplace**: Discover, install, share plugins easily

### Benefits

**For Users**:
- Customize Two-Face for their exact needs
- Access community-created tools
- Don't wait for official features
- Share configurations and plugins

**For Developers**:
- Extend Two-Face without forking
- Rapid prototyping of new ideas
- Share work with community
- Build reputation through quality plugins

**For Project**:
- Core stays small and maintainable
- Community drives feature development
- Users become contributors
- Ecosystem grows organically

---

## Conclusion

The main.rs refactoring doesn't just clean up code—it **unlocks a platform architecture** that enables:

1. **Extensibility**: Users can add features without touching core
2. **Community Growth**: Plugin ecosystem creates vibrant community
3. **Maintainability**: Core stays small, features distributed to plugins
4. **Innovation**: Experimental ideas can be plugins, not core commits
5. **Customization**: Every user can build their perfect client

The plugin system transforms Two-Face from "a game client" to "the platform for game clients"—where the community drives evolution through shared plugins, and the core remains a stable, reliable foundation.

This is the **long-term strategic value** of the architectural refactoring: not just cleaner code today, but an extensible platform for tomorrow.
