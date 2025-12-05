# Dual-Frontend Architecture Blockers

## Executive Summary

This document identifies ALL remaining blockers that prevent a clean dual-frontend architecture (TUI + GUI) in Two-Face. While we've made significant progress with the frontend abstraction layer, **core modules still have direct dependencies on TUI libraries** that must be resolved before the main.rs refactor.

**Key Finding**: The main blocker is not just main.rs, but the fact that **core business logic (AppCore, Config) directly imports and uses crossterm types**, making it impossible to instantiate the application in a GUI context.

---

## Critical Blockers (Must Fix Before main.rs Refactor)

### Blocker #1: Core Module Has Direct crossterm Dependencies

**Severity**: 🔴 CRITICAL
**Location**: `src/core/`
**Estimated Effort**: 4-6 hours

#### Problem

Core business logic modules directly import and use crossterm types for input routing and keybind mapping.

#### Files Affected

```rust
// src/core/input_router.rs:11
use crossterm::event::KeyEvent;

// src/core/menu_actions.rs:6
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

// src/core/app_core.rs:136, 268-269
pub keybind_map: HashMap<crossterm::event::KeyEvent, KeyBindAction>,
```

#### Impact

- Cannot instantiate AppCore without crossterm dependency
- GUI frontend would need to bundle crossterm just for type compatibility
- Violates separation of concerns: core should not know about TUI libraries
- Blocks main.rs refactor (can't route frontend-agnostic events to core)

#### Fix Required

1. Create frontend-agnostic key event representation:
   ```rust
   // src/frontend/common/input.rs
   #[derive(Debug, Clone, PartialEq, Eq, Hash)]
   pub struct KeyEvent {
       pub code: KeyCode,
       pub modifiers: KeyModifiers,
   }
   ```

2. Update AppCore to use agnostic type:
   ```rust
   // src/core/app_core.rs
   use crate::frontend::common::input::KeyEvent;

   pub keybind_map: HashMap<frontend::common::KeyEvent, KeyBindAction>,
   ```

3. Update input_router to accept agnostic events:
   ```rust
   // src/core/input_router.rs
   pub fn route_input(
       event: &frontend::common::KeyEvent,  // Not crossterm::event::KeyEvent
       app_core: &mut AppCore
   ) -> InputRouteResult
   ```

4. Add conversion in TUI bridge:
   ```rust
   // src/frontend/tui/crossterm_bridge.rs
   pub fn from_crossterm_event(event: crossterm::event::KeyEvent) -> frontend::common::KeyEvent {
       frontend::common::KeyEvent {
           code: from_crossterm_keycode(event.code),
           modifiers: from_crossterm_modifiers(event.modifiers),
       }
   }
   ```

#### Dependencies

This blocker depends on **Blocker #2** (config.rs must be fixed first) and **Blocker #3** (complete KeyCode enum).

---

### Blocker #2: Config Module Has crossterm::event Dependencies

**Severity**: 🔴 CRITICAL
**Location**: `src/config.rs`
**Estimated Effort**: 3-5 hours

#### Problem

Configuration system uses crossterm types for keybind resolution and parsing.

#### Files Affected

```rust
// Line 8
use crossterm::event::{KeyCode, KeyModifiers};

// Line 1836
pub fn resolve_action(&self, key: crossterm::event::KeyEvent, ...) -> Option<MenuAction>

// Line 1844
if matches!(key.code, crossterm::event::KeyCode::BackTab)

// Lines 2001-2100
fn parse_key_string(s: &str) -> Result<(KeyCode, KeyModifiers)>
// Returns crossterm types
```

#### Impact

- Config module cannot be loaded in GUI without crossterm
- Keybind configuration tied to TUI library
- Blocks AppCore refactor (AppCore uses config types)

#### Fix Required

1. Update `parse_key_string()` to return frontend-agnostic types:
   ```rust
   // src/config.rs
   fn parse_key_string(s: &str) -> Result<(frontend::common::KeyCode, frontend::common::KeyModifiers)> {
       // Parse string -> agnostic KeyCode enum
   }
   ```

2. Update `MenuKeybinds::resolve_action()`:
   ```rust
   pub fn resolve_action(
       &self,
       key: &frontend::common::KeyEvent,  // Not crossterm::event::KeyEvent
       mode: InputMode
   ) -> Option<MenuAction>
   ```

3. Update keybind matching logic to use agnostic types throughout.

#### Dependencies

This blocker depends on **Blocker #3** (KeyCode enum must include all variants first).

---

### Blocker #3: Missing Keypad Keys in Frontend Abstraction

**Severity**: 🟡 HIGH
**Location**: `src/frontend/common/input.rs`
**Estimated Effort**: 1-2 hours

#### Problem

The frontend-agnostic `KeyCode` enum lacks keypad variants that exist in crossterm and are supported by config.rs ("num_0", "num_1", etc.).

#### Missing Variants

```rust
// These exist in crossterm but NOT in frontend::common::KeyCode:
Keypad0, Keypad1, Keypad2, Keypad3, Keypad4,
Keypad5, Keypad6, Keypad7, Keypad8, Keypad9,
KeypadPeriod, KeypadPlus, KeypadMinus,
KeypadMultiply, KeypadDivide, KeypadEnter
```

#### Impact

- Users cannot configure numpad keybinds in GUI frontend
- Config parsing supports "num_0" but frontend abstraction doesn't
- Missing variants cause conversion failures

#### Fix Required

1. Add keypad variants to `src/frontend/common/input.rs`:
   ```rust
   pub enum KeyCode {
       // ... existing variants
       Keypad0,
       Keypad1,
       Keypad2,
       Keypad3,
       Keypad4,
       Keypad5,
       Keypad6,
       Keypad7,
       Keypad8,
       Keypad9,
       KeypadPeriod,
       KeypadPlus,
       KeypadMinus,
       KeypadMultiply,
       KeypadDivide,
       KeypadEnter,
   }
   ```

2. Update crossterm bridge to convert keypad keys:
   ```rust
   // src/frontend/tui/crossterm_bridge.rs
   pub fn from_crossterm_keycode(code: crossterm::event::KeyCode) -> frontend::common::KeyCode {
       match code {
           // ... existing conversions
           crossterm::event::KeyCode::KpDigit0 => KeyCode::Keypad0,
           crossterm::event::KeyCode::KpDigit1 => KeyCode::Keypad1,
           // ... etc
       }
   }
   ```

3. Update `config::parse_key_string()` to map "num_0" -> `KeyCode::Keypad0`.

#### Dependencies

This is a **prerequisite** for Blockers #1 and #2.

---

### Blocker #4: Theme System Uses ratatui::style::Color

**Severity**: 🟡 HIGH
**Location**: `src/theme.rs`
**Estimated Effort**: 4-6 hours

#### Problem

The entire theme system uses ratatui's Color type directly, making it impossible to use themes in GUI without ratatui dependency.

#### Files Affected

```rust
// Line 6
use ratatui::style::Color;

// 60+ fields in AppTheme struct:
pub window_border: Color,
pub text_primary: Color,
pub background_primary: Color,
// ... 57 more fields

// Line 150+
pub fn get_color(&self, name: &str) -> Option<Color>

// Theme presets use ratatui Color enum:
Color::Rgb(45, 45, 45)
Color::Yellow
Color::Reset
```

#### Impact

- Theme system cannot be used in GUI without ratatui
- All 60+ color fields tied to TUI library
- Theme presets are TUI-specific

#### Fix Required

**Option A: Frontend-Agnostic Color Type (Recommended)**

1. Create agnostic color representation:
   ```rust
   // src/theme.rs or src/frontend/common/color.rs
   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   pub enum Color {
       Rgb(u8, u8, u8),
       Named(NamedColor),
       Reset,
   }

   #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
   pub enum NamedColor {
       Black, Red, Green, Yellow, Blue, Magenta, Cyan, White,
       // ... 16 color variants
   }
   ```

2. Update all AppTheme fields to use agnostic Color.

3. Add conversion utilities:
   ```rust
   // src/frontend/tui/color_bridge.rs
   pub fn to_ratatui_color(color: theme::Color) -> ratatui::style::Color {
       match color {
           theme::Color::Rgb(r, g, b) => ratatui::style::Color::Rgb(r, g, b),
           theme::Color::Named(NamedColor::Red) => ratatui::style::Color::Red,
           // ...
       }
   }

   // Future: src/frontend/gui/color_bridge.rs
   pub fn to_egui_color(color: theme::Color) -> egui::Color32 {
       match color {
           theme::Color::Rgb(r, g, b) => egui::Color32::from_rgb(r, g, b),
           // ...
       }
   }
   ```

4. Update theme presets to use agnostic colors:
   ```rust
   monokai_bg: Color::Rgb(39, 40, 34),
   monokai_text: Color::Named(NamedColor::White),
   ```

**Option B: Keep ratatui Color + Conversion (Less Clean)**

- Keep ratatui Color in theme
- Add conversion layer when loading themes in GUI
- Maintains coupling but defers breaking change

**Recommendation**: Option A - Clean separation is worth the refactor effort.

#### Dependencies

Independent of other blockers - can be done in parallel.

---

### Blocker #5: Selection System Uses ratatui::layout::Rect

**Severity**: 🟠 MEDIUM
**Location**: `src/selection.rs`
**Effort**: 2-3 hours

#### Problem

Text selection tracking uses TUI-specific rectangle type.

#### Files Affected

```rust
// Line 7
use ratatui::layout::Rect;

// Selection utilities use Rect:
pub fn get_visible_content(
    content_area: Rect,  // ratatui type
    ...
)
```

#### Impact

- Text selection cannot work in GUI without ratatui
- Spatial calculations tied to TUI library

#### Fix Required

1. Create frontend-agnostic rectangle type:
   ```rust
   // src/frontend/common/geometry.rs
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   pub struct Rect {
       pub x: u16,
       pub y: u16,
       pub width: u16,
       pub height: u16,
   }
   ```

2. Update SelectionState to use agnostic Rect.

3. Add conversion utilities in TUI bridge:
   ```rust
   pub fn to_ratatui_rect(rect: frontend::common::Rect) -> ratatui::layout::Rect {
       ratatui::layout::Rect {
           x: rect.x,
           y: rect.y,
           width: rect.width,
           height: rect.height,
       }
   }
   ```

**Alternative**: Use simple `(u16, u16, u16, u16)` tuple instead of custom type.

#### Dependencies

Independent - can be done anytime before GUI implementation.

---

## High Priority Blockers

### Blocker #6: Config Border Parsing Uses ratatui::widgets::Borders

**Severity**: 🟠 MEDIUM
**Location**: `src/config.rs:1181`
**Effort**: 1-2 hours

#### Problem

Border configuration parsing returns TUI-specific type.

```rust
pub fn parse_border_sides(sides: &BorderSides) -> ratatui::widgets::Borders
```

#### Impact

Border configuration in layouts is TUI-specific.

#### Fix Required

**Option A: Move to Frontend (Recommended)**

Move `parse_border_sides()` to TUI frontend - it's rendering logic, not config.

```rust
// Remove from config.rs
// Add to src/frontend/tui/layout_renderer.rs
fn parse_border_sides(sides: &config::BorderSides) -> ratatui::widgets::Borders {
    // Same logic, but lives in TUI module
}
```

**Option B: Store as Enum in Config**

```rust
// src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BorderConfig {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool,
}

// Let each frontend interpret borders appropriately
```

**Recommendation**: Option A - rendering logic belongs in frontend.

---

### Blocker #7: UI State Contains TUI-Specific Concepts

**Severity**: 🟠 MEDIUM
**Location**: `src/data/ui_state.rs`
**Effort**: 3-4 hours

#### Problem

While ui_state.rs has no direct TUI imports, it contains TUI-specific concepts that assume character-based grid coordinates.

#### TUI-Coupled State

```rust
// Line 61-66: Character grid coordinates
pub struct MouseDragState {
    pub start_x: u16,
    pub start_y: u16,
    pub current_x: u16,
    pub current_y: u16,
}

// Line 70-75: Terminal-specific resize
pub enum DragOperation {
    WindowMove { window_name: String },
    WindowResize { window_name: String, edge: ResizeEdge },
}

// Line 341-364: Character grid positioning
impl PopupMenu {
    pub fn check_click(&self, x: u16, y: u16) -> Option<usize> {
        // Assumes character coordinates
    }
}
```

#### Impact

GUI would need pixel coordinates, not character grid coordinates.

#### Fix Required

**Option A: Generic Coordinates**

```rust
pub struct MouseDragState<Coord = u16> {
    pub start_x: Coord,
    pub start_y: Coord,
    pub current_x: Coord,
    pub current_y: Coord,
}

// TUI uses MouseDragState<u16> (character grid)
// GUI uses MouseDragState<f32> (pixel coordinates)
```

**Option B: Keep Character Grid (Simpler)**

Accept that GUI will use "virtual character grid" (like terminal emulators do). Convert pixel coordinates to character units in GUI frontend.

```rust
// GUI frontend converts mouse events:
let char_x = pixel_x / char_width;
let char_y = pixel_y / line_height;
```

**Recommendation**: Option B - Many GUI apps use character grids for consistency. Less refactoring needed.

---

### Blocker #8: Data Module Stores Coordinates in Character Grid Units

**Severity**: 🟠 MEDIUM
**Location**: `src/data/window.rs`
**Effort**: 3-4 hours

#### Problem

WindowPosition struct uses `u16` coordinates assuming terminal character grid.

```rust
// Line 85-91
pub struct WindowPosition {
    pub x: u16,      // Character column
    pub y: u16,      // Character row
    pub width: u16,  // Characters wide
    pub height: u16, // Characters tall
}
```

#### Impact

GUI would need separate position tracking (pixels vs characters).

#### Fix Required

**Option A: Generic Position Type**

```rust
pub struct WindowPosition<Coord = u16> {
    pub x: Coord,
    pub y: Coord,
    pub width: Coord,
    pub height: Coord,
}
```

**Option B: Normalized Positions**

```rust
pub struct WindowPosition {
    pub x: f32,      // 0.0 - 1.0
    pub y: f32,      // 0.0 - 1.0
    pub width: f32,  // 0.0 - 1.0
    pub height: f32, // 0.0 - 1.0
}

// Frontends convert to their native units
```

**Option C: Virtual Character Grid (Simplest)**

GUI uses character-based virtual grid (like VS Code, Sublime Text). Maintains consistency with TUI.

**Recommendation**: Option C - Simplest approach, works well in practice.

---

## Medium Priority (Can Work Around Initially)

### Blocker #9: Missing Frontend Trait Methods

**Severity**: 🟢 LOW
**Location**: `src/frontend/mod.rs`
**Effort**: 1-2 hours

#### Problem

Frontend trait has incomplete interface for dual-frontend needs.

#### Missing Methods

```rust
pub trait Frontend {
    // ... existing methods

    // MISSING:
    fn set_title(&mut self, title: &str);
    fn request_redraw(&mut self);
    fn is_focused(&self) -> bool;
    fn set_cursor_position(&mut self, x: u16, y: u16);
    fn get_clipboard(&self) -> Result<String>;
    fn set_clipboard(&mut self, text: &str) -> Result<()>;
}
```

#### Impact

GUI implementation may need additional methods for window management.

#### Fix Required

Expand Frontend trait as needed during GUI implementation.

---

### Blocker #10: AppCore Doesn't Store Terminal Size

**Severity**: 🟢 LOW
**Location**: `src/core/app_core.rs`
**Effort**: 1 hour

#### Problem

AppCore has no terminal_size field, relying on frontend to provide it.

#### Impact

Layout calculations cannot be done in AppCore without passing size everywhere.

#### Fix Required

**Option A: Store in AppCore**

```rust
pub struct AppCore {
    // ... existing fields
    pub terminal_size: (u16, u16),
}

// Update on resize events
pub fn handle_resize(&mut self, width: u16, height: u16) {
    self.terminal_size = (width, height);
    self.recalculate_layouts();
}
```

**Option B: Keep Current Pattern**

Current pattern works fine - frontend passes size as needed.

**Recommendation**: Option B - current pattern is acceptable.

---

## Documentation & Architecture Issues

### Blocker #11: Inconsistent Module Documentation

**Severity**: 🟢 LOW
**Location**: Multiple files
**Effort**: 2-3 hours

#### Problem

Some modules claim to be frontend-agnostic but aren't.

```rust
// src/core/mod.rs:4
//! Core application logic.
//! NO imports from frontend/

// But core modules DO import crossterm!
```

#### Fix Required

Update module documentation to accurately reflect current state and planned refactors.

---

## Dependency Graph

```
main.rs refactor
    ↓ (depends on)
AppCore using frontend-agnostic events (Blocker #1)
    ↓ (depends on)
Config using frontend-agnostic events (Blocker #2)
    ↓ (depends on)
Frontend abstraction has complete KeyCode enum (Blocker #3)

Theme system refactor (Blocker #4)
    ↓ (independent)
Can be done in parallel with core refactor

Selection system refactor (Blocker #5)
Border parsing cleanup (Blocker #6)
UI state coordinate review (Blocker #7)
WindowPosition strategy (Blocker #8)
    ↓ (all low priority)
Can be done after core refactor
```

---

## Recommended Refactoring Order

### Phase 1: Input Abstraction (CRITICAL - Do First)

**Goal**: Make core modules frontend-agnostic

**Tasks**:
1. ✅ Add keypad keys to `frontend::common::KeyCode` (Blocker #3)
2. ✅ Update `config.rs` to use frontend-agnostic types (Blocker #2)
3. ✅ Update core modules to use frontend-agnostic events (Blocker #1)
4. ✅ Move border parsing to TUI frontend (Blocker #6)

**Estimated Time**: 9-15 hours

**Result**: Core can be instantiated without TUI dependencies

### Phase 2: Theme Abstraction (HIGH - Do Second)

**Goal**: Enable cross-frontend theme support

**Tasks**:
5. ✅ Create frontend-agnostic color representation (Blocker #4)
6. ✅ Update theme system to use agnostic colors
7. ✅ Add TUI color conversion utilities

**Estimated Time**: 6-9 hours

**Result**: Themes work in both TUI and GUI

### Phase 3: Spatial Abstractions (MEDIUM - Can Defer)

**Goal**: Handle coordinate system differences

**Tasks**:
8. ✅ Update selection system to use agnostic rectangles (Blocker #5)
9. ✅ Review UI state coordinate types (Blocker #7)
10. ✅ Decide on WindowPosition coordinate strategy (Blocker #8)

**Estimated Time**: 8-11 hours

**Result**: Pixel-perfect GUI support (or decision to use virtual character grid)

### Phase 4: Main.rs Refactor

**Goal**: Extract TUI logic from orchestration

**Tasks**:
- Proceed with main-refactoring-plan.md
- Extract mouse handling
- Extract keyboard routing
- Extract widget factory

**Estimated Time**: Per main-refactoring-plan.md

**Result**: Clean orchestrator + true dual-frontend architecture

### Phase 5: Polish (LOW - After GUI Works)

**Goal**: Complete the architecture

**Tasks**:
11. ✅ Expand Frontend trait interface (Blocker #9)
12. ✅ Update module documentation (Blocker #11)

**Estimated Time**: 3-5 hours

**Result**: Production-ready dual-frontend architecture

---

## Total Estimated Effort

| Phase | Effort | Priority |
|-------|--------|----------|
| Phase 1: Input Abstraction | 9-15 hours | Critical |
| Phase 2: Theme Abstraction | 6-9 hours | High |
| Phase 3: Spatial Abstractions | 8-11 hours | Medium |
| Phase 4: Main.rs Refactor | Per separate plan | High |
| Phase 5: Polish | 3-5 hours | Low |
| **Critical Path (Phase 1+2)** | **15-24 hours** | **Required** |
| **Full Cleanup (All Phases)** | **26-40 hours** | **Ideal** |
| **Minimum for main.rs** | **9-15 hours** | **Blocker** |

---

## Risk Assessment

### High Risk Issues

1. **Blocker #1 (Core crossterm deps)** - 🔴 HIGH RISK
   - Touches 4,288-line app_core.rs
   - Risk of breaking existing TUI functionality
   - Requires comprehensive testing

2. **Blocker #4 (Theme system)** - 🟡 MEDIUM RISK
   - Affects 60+ color fields across entire app
   - All theme presets must be updated
   - Risk of color rendering bugs

### Medium Risk Issues

3. **Blocker #2 (Config crossterm deps)** - 🟡 MEDIUM RISK
   - Large config.rs file (2000+ lines)
   - Many keybind parsing edge cases

4. **Blockers #7 & #8 (Coordinate systems)** - 🟡 MEDIUM RISK
   - Conceptual mismatch between TUI/GUI
   - Decision impacts entire architecture

### Low Risk Issues

5. **Blockers #3, #5, #6, #9, #10, #11** - 🟢 LOW RISK
   - Isolated changes with clear boundaries
   - Well-defined scope
   - Easy to test and verify

---

## Positive Discoveries

### Clean Modules (No Blockers Found)

✅ **data/ module is CLEAN** - No TUI dependencies found
✅ **network.rs is CLEAN** - No frontend coupling
✅ **Parser is CLEAN** - Pure business logic
✅ **Frontend abstraction exists** - Good foundation in `frontend/common/`

### Good Architectural Decisions

1. ✅ UiState and WindowState are pure data structures
2. ✅ Message processing is decoupled from rendering
3. ✅ Widget data structures use hex color strings (mostly frontend-agnostic)
4. ✅ Clear module boundaries between core, data, and frontend

---

## Immediate Next Steps

### Before Main.rs Refactor

1. **Create feature branch**: `feature/dual-frontend-core-cleanup`

2. **Fix Critical Path (Phase 1)**:
   - [ ] Blocker #3: Add keypad keys (1-2 hours) - Quick win
   - [ ] Blocker #2: Config.rs frontend-agnostic (3-5 hours) - Largest blocker
   - [ ] Blocker #1: Core input routing (4-6 hours) - Unblocks main.rs
   - [ ] Blocker #6: Move border parsing (1-2 hours) - Cleanup

3. **Test Thoroughly**:
   - [ ] Run existing TUI to ensure no regressions
   - [ ] Test all keybind configurations
   - [ ] Verify theme loading works
   - [ ] Check menu navigation

4. **Optional Phase 2 (Theme)**:
   - [ ] Blocker #4: Frontend-agnostic colors (4-6 hours)
   - [ ] Test theme rendering

5. **Proceed to Main.rs**:
   - With clean foundation, proceed with main-refactoring-plan.md

---

## Testing Strategy

### Per-Blocker Testing

After fixing each blocker:
1. Run Two-Face with existing config
2. Test keybind resolution
3. Test theme application
4. Test window interactions
5. Verify no visual regressions

### Integration Testing

After Phase 1 complete:
1. Full gameplay session test
2. Test all configuration editors
3. Test all menu actions
4. Performance benchmarking (ensure no slowdown)

### Pre-Main.rs Refactor Testing

Before starting main.rs refactor:
1. Comprehensive regression test suite
2. Document current behavior
3. Create test cases for all input scenarios
4. Establish performance baseline

---

## Success Criteria

### Phase 1 Success (Critical Path)

- ✅ AppCore can be instantiated without crossterm
- ✅ Config module has no TUI dependencies
- ✅ All keybinds work (including numpad)
- ✅ Existing TUI functionality unchanged
- ✅ No performance degradation

### Phase 2 Success (Theme Abstraction)

- ✅ Themes load without ratatui dependency
- ✅ All colors render correctly in TUI
- ✅ Theme presets work as before
- ✅ Color conversion utilities tested

### Final Success (All Phases)

- ✅ Core modules are 100% frontend-agnostic
- ✅ GUI implementation requires zero core changes
- ✅ Plugin system becomes feasible
- ✅ Documentation accurately reflects architecture
- ✅ All tests pass

---

## Conclusion

While we've made excellent progress with the frontend abstraction layer, **the core modules still have critical dependencies on TUI libraries**. The main blockers are:

1. **Core modules import crossterm** (Blocker #1) - CRITICAL
2. **Config module imports crossterm** (Blocker #2) - CRITICAL
3. **Missing keypad keys in abstraction** (Blocker #3) - HIGH
4. **Theme system uses ratatui colors** (Blocker #4) - HIGH

**Phase 1 (Input Abstraction)** must be completed before the main.rs refactor to ensure core business logic is truly frontend-agnostic. This is **9-15 hours of work** but essential for a clean dual-frontend architecture.

Once Phase 1 is complete, the main.rs refactor can proceed with confidence that the foundation is solid.
