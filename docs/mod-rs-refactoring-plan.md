# Refactoring Plan: src/frontend/tui/mod.rs

**Status**: PLANNING
**Target**: Break up 5,500-line monolithic file into focused modules
**Risk Level**: HIGH (touches event loop, rendering, input handling)
**Approach**: Incremental extraction with continuous testing

---

## Current State Analysis

### File Statistics
- **Lines**: 5,500
- **Methods**: 72 in TuiFrontend impl block
- **Widget Caches**: 18 HashMap fields
- **Responsibilities**: Everything (event polling, rendering, sync, input routing, theme management)

### Pain Points
1. **Navigation**: Finding specific functionality requires extensive scrolling
2. **Testing**: Can't unit test individual concerns in isolation
3. **Maintenance**: Every change touches the same massive file
4. **Merge Conflicts**: High risk in team environments
5. **Cognitive Load**: Understanding the full system requires loading 5,500 lines into working memory

---

## Proposed Module Structure

```
src/frontend/tui/
├── mod.rs                    # Slim coordinator (200-300 lines)
│   └── TuiFrontend struct
│   └── Frontend trait impl (delegates to modules)
│
├── event_loop.rs             # Event polling and dispatch
│   └── poll_event() → FrontendEvent
│   └── handle_key_event()
│   └── handle_mouse_event()
│   └── handle_resize_event()
│
├── widget_manager.rs         # Widget cache management
│   └── All the HashMap<String, Widget> fields
│   └── All the sync_* methods
│   └── Widget creation and updates
│
├── renderer.rs               # Drawing and layout
│   └── render() method
│   └── Layout calculations
│   └── Theme application
│
├── input_handler.rs          # Input routing by mode
│   └── handle_normal_mode_keys()
│   └── handle_search_mode_keys()
│   └── Route to appropriate widget
│
└── theme_cache.rs            # Theme management
    └── Theme cache HashMap
    └── update_theme_cache()
    └── get_cached_theme()
```

---

## Extraction Strategy (Incremental, Safe)

### Phase 1: Extract Helper Modules (Low Risk)
**Goal**: Move pure functions and data structures out first

1. **theme_cache.rs** - Theme management (self-contained, low coupling)
   - Move theme cache HashMap
   - Move update_theme_cache() method
   - Test: Verify themes still load correctly

2. **resize_debouncer.rs** - Resize handling (already well-encapsulated)
   - Move ResizeDebouncer struct and impl
   - Move resize handling logic
   - Test: Verify resize still works smoothly

### Phase 2: Extract Widget Manager (Medium Risk)
**Goal**: Separate widget lifecycle management from event/render loop

1. **widget_manager.rs** - All widget caches and sync
   - Move all 18 HashMap<String, Widget> fields to WidgetManager struct
   - Move all sync_* methods (sync_text_windows, sync_command_inputs, etc.)
   - TuiFrontend contains `widget_manager: WidgetManager`
   - Test: Verify all widgets render and update correctly

### Phase 3: Extract Input Handler (Medium Risk)
**Goal**: Separate input routing logic from main event loop

1. **input_handler.rs** - Input mode routing
   - Extract handle_normal_mode_keys() (lines 5160-5500)
   - Extract handle_search_mode_keys() (lines 5000-5160)
   - Create InputHandler struct that holds keybind maps
   - Test: Verify all keybinds work (use existing keybinds.toml test cases)

### Phase 4: Extract Event Loop (High Risk)
**Goal**: Separate crossterm event polling from application logic

1. **event_loop.rs** - Event polling and conversion
   - Extract poll() method and event handling
   - Convert crossterm events → FrontendEvent
   - Keep main event loop simple and testable
   - Test: Comprehensive smoke test (all features work)

### Phase 5: Extract Renderer (High Risk)
**Goal**: Separate drawing logic from coordination logic

1. **renderer.rs** - Layout and drawing
   - Extract render() method
   - Extract layout calculation helpers
   - Renderer operates on WidgetManager state
   - Test: Visual regression testing (screenshots?)

---

## Testing Strategy

### After Each Phase
1. **Compile**: Must build without errors
2. **Smoke Test**: Launch app, verify basic functionality
3. **Feature Test**: Test specific functionality moved in that phase
4. **Regression Test**: Run through common workflows

### Critical Test Cases
- [ ] App launches without crash
- [ ] All windows render correctly
- [ ] Keybinds work (user, menu, global)
- [ ] Tab switching works
- [ ] Command input and history work
- [ ] Search mode works
- [ ] Resize handling works
- [ ] Themes switch correctly
- [ ] Performance stats toggle works

---

## Implementation Notes

### Import Management
After extraction, mod.rs will need to re-export types:
```rust
mod event_loop;
mod widget_manager;
mod renderer;
mod input_handler;
mod theme_cache;

pub use widget_manager::WidgetManager;
pub use renderer::Renderer;
// etc.
```

### Struct Ownership
Current: TuiFrontend owns everything
After: TuiFrontend delegates to specialized managers

```rust
pub struct TuiFrontend {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    widget_manager: WidgetManager,
    theme_cache: ThemeCache,
    input_handler: InputHandler,
    renderer: Renderer,
    resize_debouncer: ResizeDebouncer,
}
```

### Method Delegation Pattern
```rust
impl Frontend for TuiFrontend {
    fn render(&mut self, app_core: &AppCore) -> Result<()> {
        // Delegate to specialized modules
        self.renderer.render(
            &mut self.terminal,
            &self.widget_manager,
            &self.theme_cache,
            app_core,
        )
    }
}
```

---

## Risk Mitigation

1. **One Phase at a Time**: Complete and test each phase before moving to next
2. **Git Commits**: Commit after each successful extraction
3. **Rollback Plan**: If extraction breaks functionality, revert that commit
4. **Parallel Branch**: Work in feature branch, merge only when fully tested
5. **Preserve Tests**: Don't remove existing tests during refactoring

---

## Success Criteria

### Code Quality
- [ ] No single file > 1,000 lines
- [ ] Each module has clear, single responsibility
- [ ] Reduced coupling between modules
- [ ] Improved testability (can unit test individual modules)

### Functionality
- [ ] All existing features work identically
- [ ] No performance regression
- [ ] No new bugs introduced

### Maintainability
- [ ] Easier to find specific functionality
- [ ] Easier to add new features
- [ ] Easier to onboard new contributors
- [ ] Lower risk of merge conflicts

---

**Next Step**: Start with Phase 1 - Extract theme_cache.rs (lowest risk, self-contained)
