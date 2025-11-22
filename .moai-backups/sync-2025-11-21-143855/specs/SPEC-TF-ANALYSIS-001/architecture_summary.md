# Architecture Comparison: Two-Face vs VellumFE

**Analysis Date**: 2025-11-21
**Scope**: Module structure, state management, input handling, and rendering pipeline
**Focus**: Multi-frontend readiness and architectural quality

---

## Executive Summary

**VellumFE** uses a **monolithic architecture** with all state and rendering logic in a single `App` struct. **Two-Face** refactored this into a **layered architecture** with separate `core/` (business logic), `data/` (state), and `frontend/` (rendering) modules.

**Two-Face Improvements**:
- ✓ Clear separation of concerns (core logic decoupled from rendering)
- ✓ Frontend abstraction trait enables multi-frontend support
- ✓ Testable business logic without UI dependencies

**Two-Face Issues**:
- ✗ Still tightly coupled in TUI-specific areas (no true GUI abstraction yet)
- ✗ UiState mixes UI-agnostic data with TUI-specific rendering state
- ✗ Input routing abstracted but still InputMode-aware (TUI concept)

**GUI Port Verdict**: Two-Face's architecture is a **solid foundation** but requires additional refactoring to fully decouple GUI-specific concerns.

---

## Module Layout Comparison

### VellumFE (Monolithic)

```
src/
├── main.rs              # Entry point, terminal setup
├── app.rs               # SINGLE SOURCE OF TRUTH for all state/logic (3000+ lines)
│   ├── App struct       # Contains all game state, UI state, windows, modes
│   ├── Event loop       # Direct event handling with inline state mutations
│   ├── Resize debouncer # Terminal size change handling
│   ├── Input processing # Menu keybinds, form handling
│   └── Rendering        # Direct ratatui rendering
├── ui/                  # Rendering widgets (no state management)
│   ├── popup_menu.rs
│   ├── highlight_form.rs
│   ├── keybind_form.rs
│   ├── theme_editor.rs   # Not in VellumFE
│   └── ... (40+ widget files)
├── config.rs            # Configuration loading (includes EditorTheme, ColorConfig)
├── parser.rs            # XML parsing
├── network.rs           # Lich connection handling
├── cmdlist.rs           # Context menu population
├── performance.rs       # Performance metrics
└── selection.rs         # Text selection state

Lines of Code (LOC):
- app.rs: ~3000 LOC
- Total src: ~25,000 LOC
```

**Characteristics**:
- Single `App` struct owns **all state** (game, UI, windows, forms, browsers)
- Direct mutation: event handler modifies `App` fields directly
- Widgets read state via `&App` references
- Tight coupling between domain logic and TUI rendering
- Difficult to test business logic without spinning up terminal

---

### Two-Face (Layered)

```
src/
├── main.rs                    # Entry point
├── core/                      # Business logic layer (frontend-agnostic)
│   ├── mod.rs
│   ├── app_core.rs            # AppCore struct (game state, XML processing, command routing)
│   ├── state.rs               # GameState struct (room, vitals, inventory, etc.)
│   ├── messages.rs            # MessageProcessor (XML → state updates)
│   ├── input_router.rs        # Routes keyboard input to menu actions
│   ├── menu_actions.rs        # Menu action handlers
│   ├── event_bridge.rs        # Translates frontend events to core actions
│   └── input_result.rs        # Result types from input processing
├── data/                      # State structures (no rendering code)
│   ├── mod.rs
│   ├── ui_state.rs            # UiState struct (windows, focus, input modes, popups)
│   ├── window.rs              # WindowConfig, WidgetConfig structures
│   └── widget.rs              # Widget state definitions
├── frontend/                  # Frontend abstraction
│   ├── mod.rs                 # Frontend trait definition
│   ├── events.rs              # FrontendEvent enum (keyboard, mouse, resize)
│   ├── tui/                   # Ratatui implementation (2700+ LOC)
│   │   ├── mod.rs             # TuiFrontend implementation
│   │   ├── popup_menu.rs
│   │   ├── highlight_form.rs
│   │   ├── keybind_form.rs
│   │   ├── theme_editor.rs
│   │   ├── window_editor.rs
│   │   ├── settings_editor.rs
│   │   └── ... (40+ widget files)
│   └── gui/                   # egui implementation (stub)
│       └── mod.rs             # Placeholder for future GUI
├── config.rs                  # Configuration loading (includes AppTheme with 77 fields)
├── parser.rs                  # XML parsing
├── network.rs                 # eAccess + Lich connection handling
├── cmdlist.rs                 # Context menu population
├── performance.rs             # Performance metrics
├── selection.rs               # Text selection state
├── sound.rs                   # Sound player
├── clipboard.rs               # Clipboard integration
├── theme.rs                   # AppTheme definition + 16 built-in themes
└── tts/                       # Text-to-speech (accessibility feature)

Lines of Code (LOC):
- app_core.rs: ~400 LOC
- ui_state.rs: ~500 LOC
- frontend/tui/mod.rs: ~2700 LOC
- Total src: ~28,000 LOC (slightly larger due to additional features)
```

**Characteristics**:
- **Separation of Concerns**:
  - `core/` = Domain logic (no rendering)
  - `data/` = State structures (no business logic)
  - `frontend/` = UI implementation
- **AppCore** manages game state and message processing
- **UiState** tracks UI state (windows, focus, input mode)
- **TUI Frontend** reads immutable snapshots and renders
- Better testability (core logic decoupled from rendering)
- Foundation for multi-frontend support (GUI stub exists)

---

## State Management Comparison

### VellumFE: Monolithic State

```rust
pub struct App {
    // Configuration
    pub config: Config,
    pub layout: Layout,

    // Game state (mixed)
    pub character_name: String,
    pub character_code: String,
    pub room_name: String,
    pub room_id: String,
    pub vitals: Vitals,
    pub inventory: Inventory,
    pub spells: SpellList,
    pub active_effects: Vec<ActiveEffect>,
    // ... 20+ game state fields

    // UI State (mixed)
    pub input_mode: InputMode,
    pub windows: Vec<WindowManager>,
    pub focused_window_index: usize,
    pub text_input: String,
    pub search_query: String,
    pub popup_menu: Option<PopupMenu>,
    pub highlight_browser: Option<HighlightBrowser>,
    pub keybind_browser: Option<KeybindBrowser>,
    // ... 15+ UI state fields

    // Rendering state (TUI-specific)
    pub resize_debouncer: ResizeDebouncer,
    pub last_render_time: Instant,
    pub drag_state: Option<DragState>,
    // ... TUI-specific fields

    // Misc
    pub parser: XmlParser,
    pub sound_player: Option<SoundPlayer>,
    pub perf_stats: PerformanceStats,
}
```

**Mutation Pattern**:
```rust
impl App {
    pub fn update(&mut self, event: Event) {
        match event {
            Event::Key(k) => {
                // Direct mutation
                self.input_mode = InputMode::Normal;
                self.text_input.push_str(&key_str);
                self.windows[idx].add_line(line);
                // ...
            }
        }
    }
}
```

**Data Flow**:
```
Network
  ↓ (Raw bytes)
Parser
  ↓ (Parsed elements)
App::process_message() (Direct mutation)
  ↓ (State changed inline)
App::render() (Reads mutated state)
  ↓ (ratatui rendering)
Terminal
```

---

### Two-Face: Layered State

```rust
// CORE LAYER (No rendering)
pub struct AppCore {
    pub config: Config,
    pub game_state: GameState,           // Game data only
    pub ui_state: UiState,               // UI state
    pub parser: XmlParser,
    pub message_processor: MessageProcessor,
    pub cmdlist: Option<CmdList>,
    pub current_stream: String,
    pub stream_buffer: String,
    // ... (No rendering state)
}

pub struct GameState {
    pub character_name: String,
    pub room_name: String,
    pub vitals: Vitals,
    pub inventory: Inventory,
    pub spells: SpellList,
    pub active_effects: Vec<ActiveEffect>,
    pub combat_log: Vec<CombatEntry>,
    pub room_components: HashMap<String, Vec<Vec<TextSegment>>>,
    // ... Game data only
}

// DATA LAYER
pub struct UiState {
    pub input_mode: InputMode,          // Current input context
    pub focused_window: Option<usize>,  // Which window has focus
    pub window_config: WindowConfig,    // Window geometry + ordering
    pub last_link_click: Option<(u16, u16)>,
    pub popup_menu: Option<PopupMenuData>,
    pub pending_editors: Vec<EditorState>,
    pub search_query: String,
    pub selected_text: Option<String>,
    pub perf_stats: PerformanceStats,
    // ... UI state (no rendering)
}

// FRONTEND LAYER
pub trait Frontend {
    fn poll_events(&mut self) -> Result<Vec<FrontendEvent>>;
    fn render(&mut self, app: &mut dyn Any) -> Result<()>;
    fn cleanup(&mut self) -> Result<()>;
}

pub struct TuiFrontend {
    terminal: Terminal<CrosstermBackend>,
    // Local rendering state only
}
```

**Mutation Pattern**:
```rust
// Core processes message
impl MessageProcessor {
    pub fn process_element(&mut self, elem: ParsedElement, app_core: &mut AppCore) {
        match elem {
            ParsedElement::Room(r) => {
                // Update game state
                app_core.game_state.room_name = r.name;
                // No rendering
            }
        }
    }
}

// Frontend reads state
impl TuiFrontend {
    pub fn render(&mut self, app: &mut dyn Any) -> Result<()> {
        let app_core = app.downcast_ref::<AppCore>()?;
        // Read-only access to app_core.game_state and app_core.ui_state
        // Render based on state snapshot
    }
}
```

**Data Flow**:
```
Network
  ↓ (Raw bytes)
Parser
  ↓ (Parsed elements)
MessageProcessor (Immutable state update)
  ↓ (AppCore fields changed)
TuiFrontend::render() (Read-only)
  ↓ (Reads AppCore state, generates UI commands)
ratatui rendering
  ↓
Terminal
```

**Key Differences**:
| Aspect | VellumFE | Two-Face |
|--------|----------|----------|
| State Owner | Single App struct | AppCore (core) + separate modules |
| Game/UI Separation | Mixed | Separated (GameState vs UiState) |
| Rendering State | In App | Local to TuiFrontend only |
| Mutation Points | Many (direct field access) | Centralized (MessageProcessor, input handlers) |
| Testability | Difficult (requires terminal) | Easy (test core logic without UI) |
| Multi-frontend | Impossible | Possible (Frontend trait) |

---

## Input and Event Handling

### VellumFE: Direct Event Loop

```rust
// In app.rs
pub fn run(&mut self) -> Result<()> {
    loop {
        // Poll events
        if crossterm::event::poll(Duration::from_millis(16))? {
            let event = crossterm::event::read()?;

            match event {
                Event::Key(key) => {
                    // Direct state mutation based on mode
                    match self.input_mode {
                        InputMode::Normal => {
                            // Update text_input
                            self.text_input.push(key.c);
                        }
                        InputMode::HighlightForm => {
                            // Update highlight_form fields directly
                            self.highlight_form.as_mut().unwrap().handle_key(key);
                        }
                        // ... dozens of mode-specific handlers
                    }
                }
                Event::Resize(w, h) => {
                    // Check resize debouncer
                    if let Some((w, h)) = self.resize_debouncer.check_resize(w, h) {
                        // Recalculate layout
                        self.recalculate_windows(w, h);
                    }
                }
            }
        }

        // Render
        self.render()?;
    }
}
```

**Characteristics**:
- Crossterm events handled directly in event loop
- Mode-specific logic scattered across match statements
- Direct mutation of App fields
- Resize debouncing built-in
- No clear abstraction boundary between event handling and rendering

---

### Two-Face: Abstracted Event Pipeline

```rust
// File: frontend/events.rs
pub enum FrontendEvent {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
    Paste(String),
}

// File: frontend/tui/mod.rs
impl TuiFrontend {
    pub fn poll_events(&mut self) -> Result<Vec<FrontendEvent>> {
        // Poll crossterm, translate to FrontendEvent
        if crossterm::event::poll(Duration::from_millis(16))? {
            let event = crossterm::event::read()?;
            match event {
                crossterm::event::Event::Key(k) => {
                    Ok(vec![FrontendEvent::Key(k)])
                }
                crossterm::event::Event::Resize(w, h) => {
                    Ok(vec![FrontendEvent::Resize(w, h)])
                }
                // ...
            }
        } else {
            Ok(vec![])
        }
    }
}

// File: core/input_router.rs
pub fn route_input(key: KeyEvent, mode: &InputMode, config: &Config) -> MenuAction {
    let context = get_action_context(mode);
    config.menu_keybinds.resolve_action(key, context)
}

// File: main event loop (driver)
loop {
    let events = frontend.poll_events()?;
    for event in events {
        match event {
            FrontendEvent::Key(k) => {
                let action = input_router::route_input(k, &app_core.ui_state.input_mode, &app_core.config);
                // Execute menu action (in core)
            }
            FrontendEvent::Resize(w, h) => {
                // Delegate to core resize handler
                app_core.handle_resize(w, h)?;
            }
        }
    }
    frontend.render(&mut app_core)?;
}
```

**Characteristics**:
- Events abstracted into `FrontendEvent` enum (frontend-agnostic)
- `TuiFrontend::poll_events()` translates crossterm → FrontendEvent
- `input_router.rs` provides centralized input routing
- **Issue**: No resize debouncing (regression compared to VellumFE)
- **Issue**: InputMode is UI concept, couples core to TUI assumptions

---

## Rendering Pipeline

### VellumFE: Direct Ratatui Rendering

```rust
pub fn render(&mut self) -> Result<()> {
    self.terminal.draw(|f| {
        // Direct rendering from App state
        for (idx, window) in self.windows.iter().enumerate() {
            let is_focused = Some(idx) == self.focused_window_index;

            // Render window border
            let border_style = if is_focused {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Cyan)
            };

            let block = Block::default()
                .borders(Borders::ALL)
                .style(border_style)
                .title(window.name.clone());

            f.render_widget(&block, window.rect);

            // Render window content (delegates to widget)
            match window.widget_type.as_str() {
                "text" => {
                    let widget = TextWindowWidget::new(&window);
                    f.render_widget(widget, window.content_rect);
                }
                "dashboard" => {
                    // ...
                }
            }
        }

        // Render popup/editor on top if present
        if let Some(ref popup) = self.popup_menu {
            f.render_widget(popup, popup.rect);
        }

        if let Some(ref form) = self.highlight_form {
            f.render_widget(form, form.rect);
        }
    })?;

    Ok(())
}
```

**Characteristics**:
- Direct `terminal.draw()` call with frame manipulation
- Window geometry read from `self.windows`
- Widgets rendered based on type
- Popups/forms rendered on top with z-ordering

---

### Two-Face: State-First Rendering

```rust
// File: frontend/tui/mod.rs
impl TuiFrontend {
    pub fn render(&mut self, app: &mut dyn Any) -> Result<()> {
        let app_core = app.downcast_mut::<AppCore>()?;

        self.terminal.draw(|f| {
            // 1. Read state
            let windows = &app_core.ui_state.window_config.windows;
            let focused_window = app_core.ui_state.focused_window;
            let input_mode = &app_core.ui_state.input_mode;

            // 2. Render windows from layout
            let mut layout_calculator = LayoutCalculator::new(f.size());
            for (idx, window_cfg) in windows.iter().enumerate() {
                let is_focused = Some(idx) == focused_window;

                // Calculate rect from config
                let rect = layout_calculator.calculate_window_rect(window_cfg);

                // Render based on widget type
                match window_cfg.widget_type.as_str() {
                    "text" => {
                        let text_window = TextWindowWidget::new(window_cfg, &app_core.game_state);
                        f.render_widget(text_window, rect);
                    }
                    "dashboard" => {
                        let dashboard = DashboardWidget::new(window_cfg, &app_core.game_state);
                        f.render_widget(dashboard, rect);
                    }
                }
            }

            // 3. Render modal/popup on top
            if let Some(ref popup) = app_core.ui_state.popup_menu {
                let popup_widget = PopupMenu::from_state(popup);
                f.render_widget(popup_widget, popup_rect);
            }

            // Render active editor if present
            match input_mode {
                InputMode::HighlightForm => {
                    if let Some(editor) = app_core.ui_state.pending_editors.first() {
                        // Render editor
                    }
                }
                // ...
            }
        })?;

        Ok(())
    }
}
```

**Characteristics**:
- Reads from `UiState` to determine layout
- Renders windows based on `WindowConfig`
- Decoupled from direct state mutation
- **Issue**: Still uses `InputMode` which is TUI-specific concept

---

## Architectural Comparison Matrix

| Aspect | VellumFE | Two-Face | Winner |
|--------|----------|----------|--------|
| **Separation of Concerns** | Monolithic | Layered (core/data/frontend) | Two-Face |
| **State Organization** | Single App struct | AppCore + UiState + GameState | Two-Face |
| **Testability** | Difficult (needs terminal) | Easy (core testable in isolation) | Two-Face |
| **UI Abstraction** | None (ratatui directly) | Frontend trait (basic) | Two-Face |
| **Multi-frontend Ready** | No | Partial (GUI stub, needs work) | Two-Face |
| **Input Handling** | Direct mutation in loop | Routed through input_router | Two-Face |
| **Resize Handling** | Debounced 100ms | **No debouncing** | VellumFE |
| **Lines in Core Logic** | 3000 in app.rs | 400 in app_core.rs | Two-Face |
| **Code Organization** | 8 top-level modules | 8 top-level modules | Tie |
| **Feature Completeness** | Reference baseline | 95% parity + new features | Two-Face |

---

## GUI Port Readiness Analysis

### Blockers (Critical Issues for GUI Port)

1. **InputMode enum couples UI to TUI**
   - Location: `src/data/ui_state.rs`
   - Problem: `InputMode::HighlightBrowser`, `InputMode::SettingsEditor` are TUI-specific concepts
   - Impact: GUI frontend would need different mode system
   - Fix: Define abstract `EditorState` enum, map to GUI-specific states separately

2. **Widget-specific rendering in TuiFrontend**
   - Location: `src/frontend/tui/mod.rs` (2700 LOC)
   - Problem: 40+ TUI widget files with hardcoded colors and modifiers
   - Impact: Each widget needs GUI equivalent (e.g., PopupMenu → egui Window)
   - Fix: Define rendering trait for each widget type

3. **Resize debouncing removed**
   - Location: Not in `frontend/tui/mod.rs` event loop
   - Problem: TUI resize handling is immediate (performance regression)
   - Impact: GUI port would need different resize strategy anyway
   - Fix: Add configurable debounce, abstract from TUI specifics

4. **TUI-specific modifiers (REVERSED, BOLD, UNDERLINE)**
   - Location: Widget files (popup_menu.rs, forms, etc.)
   - Problem: `Modifier` enum is ratatui-specific; egui has different styling
   - Impact: Styling system needs abstraction layer
   - Fix: Define `TextStyle` enum mapping to platform-specific modifiers

### Enablers (Features Supporting GUI Port)

1. **Frontend abstraction trait** ✓
   - Location: `src/frontend/mod.rs`
   - Benefit: Provides interface for both TUI and GUI
   - Status: Basic but functional

2. **Core decoupled from rendering** ✓
   - Location: `src/core/` has no rendering imports
   - Benefit: Game logic testable, reusable across frontends
   - Status: Well implemented

3. **UiState separate from GameState** ✓
   - Location: `src/data/ui_state.rs` and `src/data/state.rs`
   - Benefit: Game state is frontend-agnostic
   - Status: Good separation (but UiState is still TUI-aware)

4. **Configuration abstraction** ✓
   - Location: `src/config.rs` with `AppTheme` (77 fields)
   - Benefit: Theme system can work for both TUI and GUI
   - Status: Excellent foundation for GUI theming

5. **GUI placeholder structure** ✓
   - Location: `src/frontend/gui/mod.rs`
   - Benefit: Scaffolding exists for egui implementation
   - Status: Stub only, needs implementation

---

## Recommendations for Architecture

### High Priority (GUI Port Enablers)

1. **Abstract InputMode into generic EditorState**
   - Define: `pub enum EditorState { None, HighlightEditor, SettingsEditor, ... }`
   - Decouple from `InputMode` TUI naming
   - Allows GUI frontend to use different state machine

2. **Create Widget Trait for rendering abstraction**
   ```rust
   pub trait Widget {
       fn render_tui(&self, area: Rect, buf: &mut Buffer);
       fn render_gui(&self, ui: &mut egui::Ui);
   }
   ```

3. **Add resize debouncing back to TUI frontend**
   - Implement `ResizeDebouncer` similar to VellumFE
   - Restore performance to expected level
   - Use configurable debounce value

4. **Expand GUI stub to include basic widgets**
   - Start with PopupMenu in egui
   - Test Frontend trait implementation
   - Validate multi-frontend approach

### Medium Priority (Code Quality)

5. **Extract common widget styling to ThemeProvider**
   - Create centralized style application
   - Remove hardcoded colors from widgets
   - Enable theme switching

6. **Move TUI-specific modifiers to theme layer**
   - Abstract `Modifier` usage
   - Allow GUI to interpret styling differently

7. **Document Frontend trait implementation requirements**
   - Clarify what each frontend must implement
   - Provide examples for future ports

---

## Summary

**Two-Face's Architecture**: Solid foundation for multi-frontend development with good separation of concerns, but requires additional abstraction work for true GUI portability.

**Key Improvement Over VellumFE**:
- Modular structure enables easier maintenance and testing
- Clear boundaries between domain logic and UI
- Foundation for multi-frontend approach

**Remaining Work for GUI Port**:
- Abstract TUI-specific concepts (InputMode, Modifiers, coordinate systems)
- Expand Widget trait system
- Implement egui equivalents of 40+ TUI widgets
- Test integration of multiple frontends

**Estimated GUI Port Effort**: Medium (requires framework design + widget implementation, 2-3 months with dedicated team)

---

**END OF ARCHITECTURE SUMMARY**
