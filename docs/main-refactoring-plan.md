# Main.rs Refactoring Plan: From God File to Orchestrator

## Executive Summary

**Current State**: main.rs is a 3,469-line monolith containing CLI parsing, TUI-specific menu building, mouse handling, keyboard routing, widget instantiation, and direct widget manipulation.

**Target State**: main.rs reduced to ~200 lines serving as a clean orchestrator that delegates to TuiFrontend for all presentation logic.

**Expected Benefits**:
- 94% reduction in main.rs size (3,469 → 200 lines)
- True dual-frontend architecture enabling GUI implementation
- Testable input handling isolated from event loop infrastructure
- Elimination of 1,500+ lines of duplicated widget handling logic
- Clear separation: main.rs = orchestration, TuiFrontend = presentation

---

## Phase 1: Extract Mouse Event Handling (Priority 1)

### What to Move
Lines 903-1560 (~700 lines) containing:
- Window hit-testing and coordinate detection
- Drag operation state management (window dragging, link dragging)
- Click handling (links, tabs, menu items)
- Text selection and clipboard operations
- Scroll event routing to windows

### Why This Matters
1. **Separation of Concerns**: Mouse interaction is purely TUI presentation logic, not application orchestration
2. **Testability**: Can't currently unit test mouse handling without spawning full event loop
3. **Reusability**: GUI frontend will need different mouse handling - keeping it in main.rs forces duplication
4. **Encapsulation**: Main.rs shouldn't know about window coordinates, drag states, or link positions

### How to Refactor

**Create**: `TuiFrontend::handle_mouse_event(&mut self, event: MouseEvent, app_core: &mut AppCore) -> Result<bool>`

**Move These Blocks**:
```rust
// FROM main.rs lines 903-1560
// TO TuiFrontend method

// Window hit-testing logic (repeated 10+ times)
for (name, window) in &app_core.ui_state.windows {
    if x >= pos.x && x < pos.x + pos.width && ... { }
}

// Drag detection and state management
if let Some(drag) = &mut app_core.ui_state.mouse_drag { }

// Link clicking and dragging
if let Some(link_drag) = &mut app_core.ui_state.link_drag_state { }

// Text selection
if let Some(selection) = &mut app_core.ui_state.selection_state { }

// Tab switching on mouse click
if mouse_y == pos.y && window.tabs.len() > 1 { }
```

**Replace in main.rs** with single line:
```rust
Event::Mouse(mouse_event) => {
    let handled = frontend.handle_mouse_event(mouse_event, &mut app_core)?;
    if handled { app_core.needs_render = true; }
}
```

### Alignment to Principles
- ✅ **Separation of Duties**: TUI interaction logic moves to TUI module
- ✅ **DRY**: Window hit-testing logic (repeated 10+ times) becomes helper method
- ✅ **Single Responsibility**: main.rs no longer responsible for coordinate math

**Estimated Reduction**: -700 lines from main.rs

---

## Phase 2: Extract Keyboard Event Routing (Priority 2)

### What to Move
Lines 1645-3469 (~1,800 lines) containing:
- 3-layer keybind routing system
- 15+ widget-specific input handlers
- Crossterm bridge conversions
- Action mapping and execution

### Why This Matters
1. **Massive Code Duplication**: Each of 15+ widgets has navigation boilerplate copied in main.rs
2. **Violation of Encapsulation**: Widgets should handle their own input, not have main.rs reach into them
3. **Impossible to Extend**: Adding new widget requires modifying main.rs input handler
4. **No Widget Autonomy**: Widgets are passive data holders instead of self-contained components

### How to Refactor

**Step 2.1: Create Widget Input Trait**

```rust
// src/frontend/tui/widget_trait.rs (NEW FILE)
pub trait InputHandler {
    fn handle_action(&mut self, action: MenuAction) -> WidgetResult;
    fn handle_key(&mut self, event: KeyEvent) -> WidgetResult;
}

pub enum WidgetResult {
    Handled,
    NotHandled,
    Close,
    SaveAndClose(Box<dyn Any>),
}
```

**Step 2.2: Implement Trait on Each Widget**

Instead of:
```rust
// CURRENT: In main.rs lines 1895-1945 (50 lines)
InputMode::HighlightBrowser => {
    if let Some(ref mut browser) = frontend.highlight_browser {
        match action {
            MenuAction::NavigateUp => browser.navigate_up(),
            MenuAction::NavigateDown => browser.navigate_down(),
            // ... 40 more lines
        }
    }
}
```

Do this:
```rust
// NEW: In src/frontend/tui/highlight_browser.rs
impl InputHandler for HighlightBrowser {
    fn handle_action(&mut self, action: MenuAction) -> WidgetResult {
        match action {
            MenuAction::NavigateUp => { self.navigate_up(); WidgetResult::Handled }
            MenuAction::NavigateDown => { self.navigate_down(); WidgetResult::Handled }
            MenuAction::Cancel => WidgetResult::Close,
            // ... widget owns its input logic
        }
    }
}
```

**Step 2.3: Create TuiFrontend Router**

```rust
// src/frontend/tui/mod.rs
impl TuiFrontend {
    pub fn handle_keyboard_event(
        &mut self,
        event: KeyEvent,
        app_core: &mut AppCore
    ) -> Result<bool> {
        // Layer 1: Global keybinds (Ctrl+C, Esc)
        if let Some(action) = self.check_global_keybinds(event) {
            return self.handle_global_action(action, app_core);
        }

        // Layer 2: Priority windows (editors, browsers, forms)
        if let Some(result) = self.route_to_active_widget(event, app_core)? {
            return Ok(self.handle_widget_result(result, app_core));
        }

        // Layer 3: User keybinds (keybinds.toml)
        if let Some(action) = app_core.keybind_map.get(&event) {
            app_core.send_command(action);
            return Ok(true);
        }

        // Layer 4: Command input
        self.handle_command_input(event, app_core)
    }

    fn route_to_active_widget(&mut self, event: KeyEvent, app_core: &AppCore) -> Result<Option<WidgetResult>> {
        match app_core.ui_state.input_mode {
            InputMode::HighlightBrowser => {
                self.highlight_browser.as_mut()
                    .map(|w| w.handle_key(event))
            }
            InputMode::KeybindBrowser => {
                self.keybind_browser.as_mut()
                    .map(|w| w.handle_key(event))
            }
            // ... delegates to widgets instead of handling inline
        }
    }
}
```

**Replace in main.rs** with:
```rust
Event::Key(key) => {
    let handled = frontend.handle_keyboard_event(key, &mut app_core)?;
    if handled { app_core.needs_render = true; }
}
```

### Alignment to Principles
- ✅ **Separation of Duties**: Widget input handling belongs to widgets, not main.rs
- ✅ **DRY**: Eliminates 1,500+ lines of duplicated navigation/form handling boilerplate
- ✅ **Open/Closed Principle**: Can add new widgets without modifying main.rs router

**Estimated Reduction**: -1,800 lines from main.rs

---

## Phase 3: Extract Menu Building Logic (Priority 3)

### What to Move
Lines 157-417 (~260 lines) containing:
- `build_config_submenu()` - Creates TUI-specific menu structure
- `build_settings_items()` - Converts Config to SettingItem widgets
- `build_hidewindow_picker()` - Generates window visibility menu

### Why This Matters
1. **Wrong Abstraction Level**: Main.rs shouldn't know about TUI-specific types like `PopupMenuItem` and `SettingItem`
2. **Not Frontend-Agnostic**: These functions return TUI-specific types, blocking GUI implementation
3. **Poor Cohesion**: Menu building logic is scattered across top of main.rs instead of with TUI module

### How to Refactor

**Create**: `src/frontend/tui/menu_builder.rs` (NEW FILE)

```rust
// Move functions verbatim, make them TuiFrontend methods
impl TuiFrontend {
    pub fn build_config_submenu(&self) -> Vec<PopupMenuItem> {
        // Lines 157-266 → here
    }

    pub fn build_settings_items(&self, config: &Config) -> Vec<SettingItem> {
        // Lines 269-384 → here
    }

    pub fn build_hidewindow_picker(&self, app_core: &AppCore) -> Vec<PopupMenuItem> {
        // Lines 387-414 → here
    }
}
```

**Update main.rs usage** from:
```rust
// CURRENT: lines 549-554
let items = build_config_submenu();
app_core.ui_state.popup_menu = Some(...);
```

To:
```rust
// NEW: Call via frontend
let items = frontend.build_config_submenu();
app_core.ui_state.popup_menu = Some(...);
```

### Alignment to Principles
- ✅ **Separation of Duties**: TUI menu structure belongs in TUI module
- ✅ **DRY**: Menu builders become reusable TuiFrontend methods
- ✅ **Encapsulation**: Main.rs no longer imports TUI-specific types

**Estimated Reduction**: -260 lines from main.rs

---

## Phase 4: Extract Widget Factory Pattern (Priority 4)

### What to Move
Lines 420-664 (~240 lines) - `handle_menu_action()` function containing:
- 45 match arms that instantiate TUI widgets
- Direct manipulation of `frontend.highlight_browser = Some(...)`
- Theme/layout coordination
- Settings editor opening

### Why This Matters
1. **Tight Coupling**: Main.rs directly creates TUI widget instances (violates dependency inversion)
2. **Knowledge Duplication**: Main.rs must know constructor signatures for 15+ widget types
3. **Fragile**: Adding widget field requires changes in main.rs, not just TuiFrontend
4. **No Abstraction**: Can't swap widget implementations without modifying main.rs

### How to Refactor

**Create**: `TuiFrontend::open_editor()` factory method

```rust
// src/frontend/tui/mod.rs
impl TuiFrontend {
    pub fn open_editor(&mut self, editor_type: EditorType, app_core: &mut AppCore) -> Result<()> {
        match editor_type {
            EditorType::HighlightBrowser => {
                self.highlight_browser = Some(HighlightBrowser::new(
                    &app_core.config.highlights,
                    app_core.config.common_highlights.keys()
                ));
                app_core.ui_state.input_mode = InputMode::HighlightBrowser;
            }
            EditorType::KeybindBrowser => {
                self.keybind_browser = Some(KeybindBrowser::new(
                    &app_core.config.keybinds,
                    app_core.config.common_keybinds.keys()
                ));
                app_core.ui_state.input_mode = InputMode::KeybindBrowser;
            }
            // ... all 15+ widget types
        }
        Ok(())
    }

    pub fn close_current_editor(&mut self, app_core: &mut AppCore) {
        // Clear the editor corresponding to current input_mode
        match app_core.ui_state.input_mode {
            InputMode::HighlightBrowser => self.highlight_browser = None,
            InputMode::KeybindBrowser => self.keybind_browser = None,
            // ... etc
        }
        app_core.ui_state.input_mode = InputMode::Normal;
    }

    pub fn close_all_editors(&mut self, app_core: &mut AppCore) {
        self.highlight_browser = None;
        self.highlight_form = None;
        self.keybind_browser = None;
        // ... all widgets
        app_core.ui_state.input_mode = InputMode::Normal;
    }
}

pub enum EditorType {
    HighlightBrowser,
    HighlightForm { name: Option<String> },
    KeybindBrowser,
    // ... all editor types
}
```

**Replace in main.rs** from:
```rust
// CURRENT: lines 522-527
MenuAction::OpenHighlightBrowser => {
    frontend.highlight_browser = Some(HighlightBrowser::new(...));
    app_core.ui_state.input_mode = InputMode::HighlightBrowser;
}
```

To:
```rust
// NEW: Single method call
MenuAction::OpenHighlightBrowser => {
    frontend.open_editor(EditorType::HighlightBrowser, &mut app_core)?;
}
```

**Also Fixes**: Lines 1778-1792 (Escape key handler) from:
```rust
// CURRENT: Manual widget cleanup
frontend.highlight_browser = None;
frontend.highlight_form = None;
frontend.keybind_browser = None;
// ... 10 more lines
app_core.ui_state.input_mode = InputMode::Normal;
```

To:
```rust
// NEW: Single method call
frontend.close_all_editors(&mut app_core);
```

### Alignment to Principles
- ✅ **Separation of Duties**: TuiFrontend owns widget lifecycle, not main.rs
- ✅ **DRY**: Widget instantiation logic centralized in one place
- ✅ **Dependency Inversion**: Main.rs depends on abstract `open_editor()`, not concrete widget types

**Estimated Reduction**: -240 lines from main.rs

---

## Phase 5: Create Helper Methods in AppCore (Priority 5)

### What to Move
Repeated logic scattered throughout main.rs:

**Window Hit-Testing** (repeated 10+ times):
```rust
for (name, window) in &app_core.ui_state.windows {
    let pos = &window.position;
    if x >= pos.x && x < pos.x + pos.width && y >= pos.y && y < pos.y + pos.height {
        // ... handle click
    }
}
```

**Window Management** (lines 665-670, used in multiple places):
```rust
if let Some(selected) = app_core.config.windows.iter().find(|w| w.name == name) {
    if let Some(layout) = app_core.config.layouts.get(&app_core.config.default_layout) {
        for window in &mut layout.windows {
            if window.name == selected.name {
                window.hidden = !window.hidden;
            }
        }
    }
}
```

### Why This Matters
1. **Code Duplication**: Window coordinate checking copy-pasted throughout mouse handler
2. **Business Logic in Presentation**: Window visibility toggling should be AppCore responsibility
3. **Hard to Test**: Can't unit test window management without full UI state

### How to Refactor

**Add to AppCore**:
```rust
// src/core/app_core.rs
impl AppCore {
    pub fn window_at_position(&self, x: u16, y: u16) -> Option<&str> {
        self.ui_state.windows.iter()
            .find(|(_, window)| {
                let pos = &window.position;
                x >= pos.x && x < pos.x + pos.width &&
                y >= pos.y && y < pos.y + pos.height
            })
            .map(|(name, _)| name.as_str())
    }

    pub fn tab_at_position(&self, window_name: &str, x: u16, y: u16) -> Option<usize> {
        // Lines 1003-1025 → extract tab hit-testing
    }

    pub fn toggle_window_visibility(&mut self, window_name: &str) -> Result<()> {
        // Lines 665-670 → extract window visibility toggling
    }

    pub fn link_at_position(&self, window_name: &str, x: u16, y: u16) -> Option<&str> {
        // Lines 1050-1089 → extract link detection
    }
}
```

**Replace in TuiFrontend::handle_mouse_event()**:
```rust
// OLD: Nested loops everywhere
for (name, window) in &app_core.ui_state.windows { ... }

// NEW: Clean helper calls
if let Some(window_name) = app_core.window_at_position(x, y) {
    if let Some(tab_index) = app_core.tab_at_position(window_name, x, y) {
        // Handle tab click
    }
}
```

### Alignment to Principles
- ✅ **DRY**: Window coordinate logic written once, used everywhere
- ✅ **Separation of Duties**: Business logic (window visibility) in AppCore, presentation in TuiFrontend
- ✅ **Testability**: Can unit test `window_at_position()` without spawning UI

**Estimated Reduction**: -200 lines from main.rs (via deduplication)

---

## Phase 6: Simplify Main Event Loop (Final Cleanup)

### What Remains
After phases 1-5, main.rs should contain only:
- CLI argument parsing (~80 lines)
- Logging initialization (~20 lines)
- `run_tui()` orchestration (~100 lines):
  - Network connection spawning
  - Event polling: `event::poll()` → `frontend.handle_event()`
  - Server message processing: `app_core.process_server_data()`
  - Render coordination: `frontend.render(&mut app_core)`
  - Countdown timer management

### Final Structure

```rust
// main.rs (~200 lines total)

mod config;
mod core;
mod data;
mod frontend;
mod network;

use clap::Parser;

#[derive(Parser)]
struct Cli { /* ~50 lines */ }

fn main() -> Result<()> {
    setup_logging()?;
    let cli = Cli::parse();

    match cli.frontend_type {
        FrontendType::Tui => run_tui(cli),
        FrontendType::Gui => run_gui(cli),
    }
}

async fn run_tui(cli: Cli) -> Result<()> {
    // Network connection (~20 lines)
    let network_handle = spawn_network_connection(cli)?;

    // Initialize frontend and core (~10 lines)
    let mut frontend = TuiFrontend::new()?;
    let mut app_core = AppCore::new(config)?;

    // Event loop (~50 lines)
    loop {
        tokio::select! {
            Some(event) = frontend.poll_event() => {
                frontend.handle_event(event, &mut app_core)?;
            }
            Some(msg) = rx.recv() => {
                app_core.process_server_data(&msg)?;
            }
            _ = countdown_ticker.tick() => {
                app_core.update_countdowns()?;
            }
        }

        if app_core.needs_render {
            frontend.render(&mut app_core)?;
            app_core.needs_render = false;
        }

        if app_core.should_exit { break; }
    }

    frontend.cleanup()?;
    Ok(())
}
```

### Alignment to Principles
- ✅ **Single Responsibility**: Main.rs only orchestrates, doesn't implement
- ✅ **Open/Closed**: Adding features doesn't require changing main.rs
- ✅ **Separation of Duties**: Clear boundaries between orchestration, presentation, and business logic

**Final Size**: ~200 lines (94% reduction from 3,469)

---

## Implementation Order & Dependencies

```
Phase 3 (Menu Builders)
    ↓ (independent)
Phase 1 (Mouse Handling) + Phase 5 (AppCore Helpers)
    ↓ (uses helpers)
Phase 4 (Widget Factory)
    ↓ (depends on factory)
Phase 2 (Keyboard Routing)
    ↓ (cleanup)
Phase 6 (Final Simplification)
```

**Recommended Order**:
1. **Phase 3** first (simple move, no dependencies)
2. **Phase 5** second (helpers needed by Phase 1)
3. **Phase 1** third (mouse handling uses AppCore helpers)
4. **Phase 4** fourth (factory pattern needed by keyboard routing)
5. **Phase 2** fifth (largest refactor, benefits from all prior work)
6. **Phase 6** last (final cleanup and documentation)

---

## Success Metrics

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| main.rs lines | 3,469 | ~200 | -94% |
| TUI-specific code in main.rs | 3,000+ | 0 | -100% |
| Duplicated input handling | 1,500+ lines | 0 | -100% |
| Testable components | 0 | 15+ widgets | ∞ |
| GUI implementation effort | Fork entire main.rs | Implement Frontend trait | -90% |
| Widget autonomy | None (passive) | Full (self-contained) | +100% |

---

## Risk Mitigation

1. **Phase-by-phase commits**: Each phase can be committed independently
2. **Behavioral equivalence testing**: Run before/after each phase to verify no functionality loss
3. **Rollback points**: Each phase is independently reversible
4. **Incremental testing**: Can test mouse handling in isolation after Phase 1
5. **No API changes**: External behavior unchanged, only internal structure

---

## Long-Term Benefits

### Enables Future Work
- ✅ GUI frontend implementation (no duplication needed)
- ✅ Widget unit testing (isolated from event loop)
- ✅ Plugin system (widgets as loadable modules)
- ✅ Alternative input methods (touch, gamepad)
- ✅ Accessibility features (screen readers, keyboard-only navigation)

### Improves Developer Experience
- ✅ New developers can understand main.rs in 5 minutes
- ✅ Widget changes don't require touching main.rs
- ✅ Clear mental model: "main orchestrates, frontend presents, widgets handle"
- ✅ Easier debugging (widget input handling isolated)

### Maintains Code Quality
- ✅ SOLID principles compliance
- ✅ DRY: No duplicated navigation/form handling
- ✅ Testability: Can test widgets in isolation
- ✅ Maintainability: Clear responsibility boundaries

---

## Summary

This refactoring transforms main.rs from a **3,469-line god file** into a **200-line orchestrator** by:

1. **Moving TUI logic to TuiFrontend** (mouse handling, keyboard routing, menu building)
2. **Giving widgets autonomy** (self-contained input handling via traits)
3. **Extracting business logic to AppCore** (window management, hit-testing helpers)
4. **Creating clean abstractions** (widget factory, editor lifecycle management)
5. **Eliminating duplication** (1,500+ lines of copy-pasted navigation code)

The result is a **true dual-frontend architecture** where GUI implementation requires implementing the Frontend trait, not forking 3,000 lines of TUI-specific logic from main.rs.

**Every change aligns with**:
- ✅ **Separation of Duties**: Right code in right module
- ✅ **DRY**: Write once, use everywhere
- ✅ **SOLID**: Single responsibility, open/closed, dependency inversion
- ✅ **Testability**: Isolated, unit-testable components
