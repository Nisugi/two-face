# Two-Face: Refactoring Progress

**Project Start:** 2025-11-09
**Status:** Phase 1+2 - Foundation Complete (Milestones 1-5 Done)

---

## Completed Milestones

### ✅ Milestone 1: Project Setup & Bootstrap
- Created clean two-face project from VellumFE codebase
- Renamed to two-face, updated README
- Initialized git repository
- Verified compilation: **SUCCESS**

**Git Commit:** `23323ed` - "Milestone 1: Project Setup & Bootstrap"

---

### ✅ Milestone 2: Create Abstraction Layer
- Created `src/frontend/mod.rs` with Frontend trait
- Created `src/frontend/events.rs` with FrontendEvent enum
- Defined trait methods: `poll_events()`, `render()`, `cleanup()`, `size()`
- FrontendEvent supports: Key, Mouse, Resize, Paste, Quit
- Verified compilation: **SUCCESS**

**Git Commit:** `910e851` - "Milestone 2: Create Abstraction Layer"

**Key Files:**
- `src/frontend/mod.rs` - Frontend trait definition
- `src/frontend/events.rs` - Frontend-agnostic event types

---

### ✅ Milestone 3: Extract AppCore (Initial Structure)
- Created `src/core/mod.rs` and `src/core/app_core.rs`
- Defined AppCore struct with core business logic state:
  - Configuration (config, layout)
  - Window management (window_manager)
  - Game protocol (parser)
  - Stream routing (current_stream, stream_buffer)
  - Keybindings (keybind_map)
  - Sound (sound_player)
  - Commands (cmdlist)
  - Room tracking (nav_room_id, lich_room_id)
  - Performance stats (perf_stats)
- Added `from_existing()` constructor for incremental refactoring
- Added stub methods for future migration
- Verified compilation: **SUCCESS**

**Git Commit:** `d48c097` - "Milestone 3: Extract AppCore (Initial Structure)"

**Key Files:**
- `src/core/app_core.rs` - Core application state

**TODO:**
- Migrate business logic methods from App to AppCore
- Implement `handle_server_message()`
- Implement `handle_dot_command()`
- Implement `update_window_manager_config()`

---

### ✅ Milestone 4: Create Widget State Structs
- Created `src/widgets/` module for rendering-agnostic widget states
- Implemented **TextWindowState**:
  - Line buffer (VecDeque)
  - Scroll position tracking
  - Methods: `add_segment()`, `finish_line()`, `scroll_up()`, `scroll_down()`, `get_visible_lines()`
  - Unit tests
- Implemented **ProgressBarState**:
  - Current/max values
  - Custom text support
  - Color configuration
  - Methods: `set_progress()`, `percentage()`, `display_text()`
  - Unit tests
- Implemented **CountdownState**:
  - Unix timestamp tracking
  - CountdownType enum (Roundtime, Casttime, Stun)
  - Methods: `set_countdown()`, `remaining_seconds()`, `is_active()`
  - Server time offset support
  - Unit tests
- Verified compilation: **SUCCESS**

**Git Commit:** `83ff284` - "Milestone 4: Create Widget State Structs"

**Key Files:**
- `src/widgets/text_window.rs`
- `src/widgets/progress_bar.rs`
- `src/widgets/countdown.rs`

**TODO:**
- Create remaining widget states:
  - TabbedTextWindowState
  - CompassState
  - IndicatorState
  - InjuryDollState
  - HandsState
  - DashboardState
  - ActiveEffectsState
  - InventoryWindowState
  - RoomWindowState
  - MapWidgetState
  - SpellsWindowState
- Migrate full widget logic from `src/ui/` modules

---

### ✅ Milestone 5: Create TUI Frontend Module
- Created `src/frontend/tui/` module structure
- Implemented **TuiFrontend** struct implementing Frontend trait:
  - `new()` - Initialize terminal (raw mode, alternate screen, mouse capture)
  - `poll_events()` - Poll crossterm events and convert to FrontendEvent
  - `render()` - Placeholder (TODO: implement using AppCore)
  - `cleanup()` - Restore terminal state
  - `size()` - Get terminal dimensions
- Event conversion from crossterm → FrontendEvent
- Terminal setup/teardown in `new()` and Drop
- Created `src/frontend/tui/widgets/` for future rendering code
- Verified compilation: **SUCCESS**

**Git Commit:** `1251f72` - "Milestone 5: Create TUI Frontend Module"

**Key Files:**
- `src/frontend/tui/app.rs` - TuiFrontend implementation
- `src/frontend/tui/widgets/mod.rs` - Widget renderers (TODO)

**TODO:**
- Implement actual rendering in `render()` method
- Create widget renderers in `tui/widgets/`
- Migrate rendering logic from `src/ui/` modules

---

## Current Architecture

```
two-face/
├── src/
│   ├── core/                     # ✅ Business logic (frontend-agnostic)
│   │   ├── mod.rs
│   │   └── app_core.rs           # AppCore struct
│   ├── widgets/                  # ✅ Widget state (rendering-agnostic)
│   │   ├── mod.rs
│   │   ├── text_window.rs        # TextWindowState
│   │   ├── progress_bar.rs       # ProgressBarState
│   │   └── countdown.rs          # CountdownState
│   ├── frontend/                 # ✅ Frontend abstraction
│   │   ├── mod.rs                # Frontend trait
│   │   ├── events.rs             # FrontendEvent enum
│   │   └── tui/                  # ✅ TUI implementation
│   │       ├── mod.rs
│   │       ├── app.rs            # TuiFrontend
│   │       └── widgets/          # TODO: TUI renderers
│   │           └── mod.rs
│   ├── app.rs                    # ⚠️  Old App (still in use)
│   ├── ui/                       # ⚠️  Old UI code (still in use)
│   ├── config.rs                 # Config management
│   ├── network.rs                # Network I/O
│   ├── parser.rs                 # XML parsing
│   └── main.rs                   # Entry point (⚠️ uses old App)
```

**Legend:**
- ✅ = New architecture (complete for this phase)
- ⚠️  = Old code (still in use, needs migration)
- TODO = Structure exists but needs implementation

---

## Remaining Work (Milestones 6-7)

### Milestone 6: Wire Everything Together (NOT STARTED)

**Goal:** Update main.rs to use new architecture

**Tasks:**
1. Update `main.rs` to create AppCore instead of App
2. Update `main.rs` to create TuiFrontend
3. Create main event loop:
   ```rust
   let mut core = AppCore::from_existing(...);
   let mut frontend = TuiFrontend::new()?;

   while core.running {
       // Poll events
       let events = frontend.poll_events()?;
       for event in events {
           core.handle_event(event)?;
       }

       // Handle server messages
       while let Ok(msg) = server_rx.try_recv() {
           core.handle_server_message(msg)?;
       }

       // Render
       frontend.render(&core)?;
   }
   ```
4. Migrate business logic methods from App to AppCore
5. Implement rendering in TuiFrontend::render()

**Estimated Time:** 1-2 weeks

---

### Milestone 7: Testing & Verification (NOT STARTED)

**Goal:** Verify TUI works identically to VellumFE

**Tasks:**
1. Fix all compilation errors
2. Build release binary
3. Test all features:
   - [ ] Connects to Lich
   - [ ] Text displays in windows
   - [ ] Colors work (presets, highlights)
   - [ ] Progress bars update
   - [ ] Countdown timers work
   - [ ] Tabbed windows work
   - [ ] Mouse scrolling works
   - [ ] Window resize works
   - [ ] Clickable links work
   - [ ] Context menus work
   - [ ] Command input works
   - [ ] Dot commands work
   - [ ] Settings editor works
   - [ ] Highlight form works
   - [ ] Keybind form works
   - [ ] Window editor works
   - [ ] Sound playback works
   - [ ] Text selection works
   - [ ] Layout save/load works
4. Fix bugs
5. Performance testing

**Estimated Time:** 1-2 weeks

---

## Phase 3: GUI Frontend (FUTURE)

After Phase 1+2 complete and TUI is verified working:

1. Add egui dependencies to Cargo.toml
2. Create `src/frontend/gui/` module
3. Implement `GuiFrontend` struct
4. Create egui widget renderers in `gui/widgets/`
5. Add `--gui` flag to main.rs
6. Test and polish

**Estimated Time:** 6-8 weeks

---

## How to Continue Development

### Next Session Checklist

1. **Review current state:**
   ```bash
   cd /c/Gemstone/Projects/two-face
   git log --oneline
   cargo build  # Should succeed
   ```

2. **Start Milestone 6:**
   - Read `src/app.rs` to understand App::run() logic
   - Identify business logic to move to AppCore
   - Create minimal main event loop
   - Test compilation at each step

3. **Migration Strategy:**
   - Move one method at a time from App to AppCore
   - Test compilation after each move
   - Commit frequently
   - Don't try to do everything at once!

### Incremental Approach

**Week 1-2:** Milestone 6 Part 1
- Migrate handle_server_message() logic
- Migrate handle_dot_command() logic
- Get basic event loop working

**Week 3-4:** Milestone 6 Part 2
- Implement TuiFrontend::render()
- Migrate window rendering
- Get text display working

**Week 5-6:** Milestone 7
- Test all features
- Fix bugs
- Performance optimization

---

## Key Decisions Made

1. **Incremental Refactoring:** Used `AppCore::from_existing()` instead of reimplementing App::new()
2. **Minimal Widget States:** Created structure with core widgets, will add rest incrementally
3. **Placeholder Rendering:** TuiFrontend::render() is stubbed, will implement in Milestone 6
4. **Kept Old Code:** App and ui/ modules still exist and work, won't be deleted until new code is proven

---

## Success Metrics

- [x] Clean separation: core vs frontend vs widgets
- [x] Frontend trait defined and implemented (TUI)
- [x] Compiles without errors
- [ ] TUI works identically to VellumFE (Milestone 7)
- [ ] GUI frontend added (Phase 3)

---

## Lessons Learned

1. **Start Simple:** Don't try to refactor everything at once
2. **Incremental Wins:** Each milestone adds value independently
3. **Preserve Working Code:** Keep old code until new code is proven
4. **Test Often:** Compile after every small change
5. **Document Assumptions:** TODOs and comments are critical for future work

---

## Contact / Questions

For questions about this refactor:
- Review this document
- Check `GUI_IMPLEMENTATION_PLAN.md` for overall strategy
- Look at git commit messages for detailed changes
- Read TODO comments in code

---

**Next Step:** Start Milestone 6 - Wire everything together and implement the main event loop!
