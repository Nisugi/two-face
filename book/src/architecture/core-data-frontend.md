# Core-Data-Frontend Architecture

Two-Face strictly separates code into three layers. This design enables frontend independence and clean testing.

## Layer Overview

```
┌─────────────────────────────────────┐
│           FRONTEND                  │
│   • Renders widgets                 │
│   • Captures user input             │
│   • Converts events                 │
│   • NO business logic               │
├─────────────────────────────────────┤
│             CORE                    │
│   • Processes messages              │
│   • Handles commands                │
│   • Dispatches keybinds             │
│   • Routes data to state            │
├─────────────────────────────────────┤
│             DATA                    │
│   • Pure data structures            │
│   • No behavior                     │
│   • Owned by Core                   │
└─────────────────────────────────────┘
```

## Data Layer

Location: `src/data/`

The data layer contains pure data structures with no logic or rendering.

### UiState

```rust
pub struct UiState {
    pub windows: HashMap<String, WindowState>,
    pub focused_window: Option<String>,
    pub input_mode: InputMode,
    pub popup_menu: Option<PopupMenu>,
    pub selection_state: Option<SelectionState>,
    pub search_input: String,
}
```

### GameState

```rust
pub struct GameState {
    pub connected: bool,
    pub character_name: Option<String>,
    pub room_id: Option<String>,
    pub room_name: Option<String>,
    pub exits: Vec<String>,
    pub vitals: Vitals,
    pub left_hand: Option<String>,
    pub right_hand: Option<String>,
    pub spell: Option<String>,
    pub status: StatusInfo,
}
```

### WindowState

```rust
pub struct WindowState {
    pub name: String,
    pub widget_type: WidgetType,
    pub streams: Vec<String>,
    pub visible: bool,
    pub rect: Rect,

    // Content (varies by widget type)
    pub text_content: Option<TextContent>,
    pub progress_data: Option<ProgressData>,
    pub countdown_data: Option<CountdownData>,
    pub compass_data: Option<CompassData>,
}
```

### TextContent

```rust
pub struct TextContent {
    pub lines: VecDeque<StyledLine>,
    pub scroll_offset: usize,
    pub max_lines: usize,
    pub title: String,
    pub generation: u64,  // Change counter
}
```

### Input Modes

```rust
pub enum InputMode {
    // Standard modes
    Normal,              // Command input active
    Navigation,          // Vi-style navigation
    History,             // Scrolling command history
    Search,              // Text search mode
    Menu,                // Context menu open

    // Editor modes (priority windows)
    HighlightBrowser,
    HighlightForm,
    KeybindBrowser,
    KeybindForm,
    ColorPaletteBrowser,
    WindowEditor,
    // ...
}
```

## Core Layer

Location: `src/core/`

The core layer contains all business logic but no rendering code.

### AppCore

The central orchestrator:

```rust
pub struct AppCore {
    pub config: Config,
    pub layout: Layout,
    pub game_state: GameState,
    pub ui_state: UiState,
    pub message_processor: MessageProcessor,
    pub parser: XmlParser,
    pub performance_stats: PerformanceStats,
}
```

### AppCore Methods

```rust
impl AppCore {
    // Message processing
    pub fn process_server_message(&mut self, msg: &str) { ... }

    // User input
    pub fn handle_command(&mut self, cmd: &str) { ... }
    pub fn dispatch_keybind(&mut self, key: &KeyEvent) { ... }

    // State management
    pub fn get_window_state(&self, name: &str) -> Option<&WindowState> { ... }
    pub fn get_window_state_mut(&mut self, name: &str) -> Option<&mut WindowState> { ... }

    // Layout
    pub fn apply_layout(&mut self, layout: Layout) { ... }
}
```

### MessageProcessor

Routes parsed XML to appropriate state:

```rust
impl MessageProcessor {
    pub fn process(
        &mut self,
        element: ParsedElement,
        game_state: &mut GameState,
        ui_state: &mut UiState,
    ) {
        match element {
            ParsedElement::Text { content, stream, .. } => {
                // Route to appropriate window's text_content
            }
            ParsedElement::ProgressBar { id, value, max, .. } => {
                // Update vitals or progress widget
            }
            ParsedElement::Compass { directions } => {
                // Update game_state.exits
            }
            // ... handle all element types
        }
    }
}
```

## Frontend Layer

Location: `src/frontend/`

The frontend layer renders UI and captures input. It has no business logic.

### Frontend Trait

```rust
pub trait Frontend {
    fn poll_events(&mut self) -> Result<Vec<FrontendEvent>>;
    fn render(&mut self, app: &mut dyn Any) -> Result<()>;
    fn cleanup(&mut self) -> Result<()>;
    fn size(&self) -> (u16, u16);
}
```

### FrontendEvent

```rust
pub enum FrontendEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Paste(String),
    FocusGained,
    FocusLost,
}
```

### TUI Frontend

The terminal UI implementation:

```rust
pub struct TuiFrontend {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    widget_manager: WidgetManager,
}

impl Frontend for TuiFrontend {
    fn poll_events(&mut self) -> Result<Vec<FrontendEvent>> {
        // Convert crossterm events to FrontendEvent
    }

    fn render(&mut self, app: &mut dyn Any) -> Result<()> {
        // Sync data to widgets
        // Render all visible widgets
    }
}
```

### Widget Manager

Caches and manages frontend widgets:

```rust
pub struct WidgetManager {
    pub text_windows: HashMap<String, TextWindow>,
    pub tabbed_text_windows: HashMap<String, TabbedTextWindow>,
    pub command_inputs: HashMap<String, CommandInput>,
    pub progress_bars: HashMap<String, ProgressBar>,
    pub countdowns: HashMap<String, Countdown>,
    pub compass_widgets: HashMap<String, Compass>,
    // ... more widget types

    // Generation tracking
    pub last_synced_generation: HashMap<String, u64>,
}
```

## Layer Communication

### Data Flow Pattern

```
Server Message
      ↓
[Network Layer] ──────→ Raw XML string
      ↓
[Parser]        ──────→ Vec<ParsedElement>
      ↓
[MessageProcessor] ───→ State updates
      ↓
[Data Layer]    ──────→ GameState, UiState, WindowState
      ↓
[Sync Functions] ─────→ Widget updates
      ↓
[Frontend]      ──────→ Rendered frame
```

### User Input Flow

```
User Input
      ↓
[Frontend] ───────────→ FrontendEvent
      ↓
[Input Router] ───────→ Keybind match?
      ↓
[AppCore] ────────────→ Execute action
      ↓
Either:
  • UI Action (state change)
  • Game Command (send to server)
```

## Strict Separation Benefits

### 1. Frontend Independence

Core and Data know nothing about rendering:

```rust
// In core/
// ❌ FORBIDDEN:
use ratatui::*;
use crossterm::*;

// ✅ ALLOWED:
use crate::data::*;
```

This enables adding a GUI frontend without modifying core logic.

### 2. Testability

Core logic can be tested without a terminal:

```rust
#[test]
fn test_message_processing() {
    let mut app = AppCore::new(test_config());

    // Simulate server message
    app.process_server_message("<pushBold/>A goblin<popBold/>");

    // Verify state without rendering
    let main_window = app.ui_state.windows.get("main").unwrap();
    assert!(main_window.text_content.as_ref().unwrap()
        .lines.back().unwrap().segments[0].bold);
}
```

### 3. Clear Boundaries

Each layer has explicit responsibilities:

| Question | Answer |
|----------|--------|
| Where do I add a new data field? | Data layer |
| Where do I process server XML? | Core layer (parser/message processor) |
| Where do I render a new widget? | Frontend layer |
| Where do I handle a keybind? | Core layer (commands/keybinds) |

### 4. Maintainability

Changes are localized:

- Change rendering style → Frontend only
- Change data format → Data + consumers
- Change business logic → Core only
- Add new widget type → All layers, but well-defined boundaries

## State Hierarchy

```
AppCore
├── config: Config                    # Configuration (immutable)
│   ├── connection: ConnectionConfig
│   ├── ui: UiConfig
│   ├── highlights: HashMap<String, HighlightPattern>
│   ├── keybinds: HashMap<String, KeyBindAction>
│   └── colors: ColorConfig
│
├── layout: Layout                    # Current window layout
│   └── windows: Vec<WindowDef>
│
├── game_state: GameState             # Game session state
│   ├── connected: bool
│   ├── character_name: Option<String>
│   ├── room_id, room_name: Option<String>
│   ├── exits: Vec<String>
│   ├── vitals: Vitals
│   └── status: StatusInfo
│
├── ui_state: UiState                 # UI interaction state
│   ├── windows: HashMap<String, WindowState>
│   ├── focused_window: Option<String>
│   ├── input_mode: InputMode
│   └── popup_menu: Option<PopupMenu>
│
└── message_processor: MessageProcessor
    ├── current_stream: String
    └── squelch_matcher: Option<AhoCorasick>
```

## Adding a New Feature

### Example: Add "Stance" Display

**1. Data Layer** - Add data structure:

```rust
// src/data/game_state.rs
pub struct GameState {
    // ... existing fields
    pub stance: Option<Stance>,
}

pub struct Stance {
    pub name: String,
    pub offensive: u8,
    pub defensive: u8,
}
```

**2. Core Layer** - Handle the XML:

```rust
// src/core/messages.rs
ParsedElement::Stance { name, offensive, defensive } => {
    game_state.stance = Some(Stance { name, offensive, defensive });
}
```

**3. Frontend Layer** - Render the widget:

```rust
// src/frontend/tui/stance_widget.rs
pub struct StanceWidget {
    stance: Option<Stance>,
}

impl Widget for &StanceWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render stance display
    }
}
```

**4. Sync Function** - Connect data to widget:

```rust
// src/frontend/tui/sync.rs
pub fn sync_stance_widgets(
    game_state: &GameState,
    widget_manager: &mut WidgetManager,
) {
    if let Some(stance) = &game_state.stance {
        let widget = widget_manager.stance_widgets
            .entry("stance".to_string())
            .or_insert_with(StanceWidget::new);
        widget.update(stance);
    }
}
```

## See Also

- [Message Flow](./message-flow.md) - Detailed data flow
- [Widget Sync](./widget-sync.md) - Synchronization patterns
- [Performance](./performance.md) - Optimization strategies

