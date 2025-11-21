# SPEC-TF-PARITY-RESIZE-001: Terminal Resize Debouncing

**TAG**: `SPEC-TF-PARITY-RESIZE-001`
**Status**: Draft
**Created**: 2025-11-21
**Owner**: @user
**Priority**: HIGH
**Category**: Performance / Parity Fix
**Complexity**: Simple
**Related SPECs**: SPEC-TF-ANALYSIS-001

---

## Environment

**WHEN** the terminal window is resized:
- User drags terminal window edges to change dimensions
- Terminal emulator reports resize events to the application
- Multiple resize events fire rapidly during continuous window dragging
- Each event triggers layout recalculation and re-render

**Current System Behavior** (Two-Face):
- Immediate processing of every resize event
- No debouncing mechanism in `src/frontend/tui/mod.rs` event loop (line 2184-2186)
- Direct pass-through from crossterm Event::Resize to FrontendEvent::Resize
- Performance degradation during rapid resize sequences

**Reference System Behavior** (VellumFE):
- ResizeDebouncer struct implements 300ms debounce logic (`vellumfe/src/app.rs` lines 30-77)
- Two-phase debouncing:
  1. `check_resize()`: Immediate processing if 300ms elapsed since last resize, else store pending size
  2. `check_pending()`: Check on every event loop iteration for pending resizes that exceeded debounce period
- Smooth performance during window resizing

---

## Assumptions

**Technical Assumptions**:
- Two-Face uses crossterm for terminal event handling (confirmed: `src/frontend/tui/mod.rs`)
- Event loop architecture allows inserting debounce logic before FrontendEvent emission
- Rust std::time::Instant provides sufficient precision for 100ms debounce timing
- No external dependencies required (use standard library Duration/Instant)

**Performance Assumptions**:
- 100ms debounce duration provides optimal balance between responsiveness and performance
- VellumFE uses 300ms; Two-Face can be more aggressive with 100ms for better UX
- Debouncing reduces layout recalculations by 70-90% during active resizing

**UX Assumptions**:
- Users expect smooth rendering during terminal resize operations
- Final resize should be processed within 100ms of user stopping window drag
- Intermediate resize events can be safely dropped without UX degradation

---

## Requirements

### Functional Requirements

**FR-1: Implement Resize Debouncer**
- **WHEN** a terminal resize event is received
- **THEN** the system **SHALL** apply 100ms debouncing logic
- **AND** only process resize if 100ms elapsed since last processed resize
- **OR** store pending size if within debounce window

**FR-2: Pending Resize Processing**
- **WHEN** the event loop iterates
- **AND** a pending resize exists
- **AND** 100ms has elapsed since the pending resize was stored
- **THEN** the system **SHALL** process the pending resize immediately

**FR-3: Maintain Resize Event Semantics**
- **WHEN** a resize is processed (either immediate or pending)
- **THEN** the system **SHALL** emit FrontendEvent::Resize with current terminal dimensions
- **AND** maintain existing resize handling behavior in downstream components

### Non-Functional Requirements

**NFR-1: Performance**
- **GIVEN** continuous terminal resize operations (10+ events/second)
- **WHEN** debouncing is active
- **THEN** render calls **SHALL** be reduced by at least 70%
- **AND** frame time **SHALL** remain under 16ms (60 FPS target)

**NFR-2: Responsiveness**
- **GIVEN** user stops resizing terminal window
- **WHEN** the final resize event is processed
- **THEN** the UI **SHALL** reflect final dimensions within 100ms

**NFR-3: No Breaking Changes**
- **GIVEN** existing resize infrastructure in Two-Face
- **WHEN** debouncing is implemented
- **THEN** all existing resize handlers **SHALL** function without modification
- **AND** no API changes to FrontendEvent::Resize

---

## Specifications

### Design Constraints

**DC-1: Follow VellumFE Reference Pattern**
- ResizeDebouncer struct from `vellumfe/src/app.rs` lines 30-77 provides reference implementation
- Two-phase approach (check_resize + check_pending) is proven and battle-tested
- Adaptation: Use 100ms debounce (vs. VellumFE's 300ms) for better responsiveness

**DC-2: Integration Points**
- Primary integration: `src/frontend/tui/mod.rs` TuiFrontend struct
- Event polling location: `poll_events()` method (lines 2169-2203)
- Resize event handling: Lines 2184-2186 (Event::Resize branch)
- No changes required to `src/frontend/events.rs` (FrontendEvent remains unchanged)

**DC-3: Rust Standard Library Only**
- Use `std::time::Instant` for timing
- Use `std::time::Duration` for debounce interval
- No external crate dependencies

### Implementation Specification

**Component 1: ResizeDebouncer Struct**

Location: `src/frontend/tui/mod.rs` (add before TuiFrontend struct)

```rust
/// Debouncer for terminal resize events to prevent excessive layout recalculations
struct ResizeDebouncer {
    last_resize_time: std::time::Instant,
    debounce_duration: std::time::Duration,
    pending_size: Option<(u16, u16)>, // (width, height)
}

impl ResizeDebouncer {
    fn new(debounce_ms: u64) -> Self {
        Self {
            last_resize_time: std::time::Instant::now(),
            debounce_duration: std::time::Duration::from_millis(debounce_ms),
            pending_size: None,
        }
    }

    /// Check if a resize event should be processed or debounced
    /// Returns Some(size) if resize should be processed now, None if debounced
    fn check_resize(&mut self, width: u16, height: u16) -> Option<(u16, u16)> {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_resize_time);

        if elapsed >= self.debounce_duration {
            // Process this resize immediately
            self.last_resize_time = now;
            self.pending_size = None;
            Some((width, height))
        } else {
            // Store for later processing
            self.pending_size = Some((width, height));
            None
        }
    }

    /// Check if there's a pending resize that should be processed
    fn check_pending(&mut self) -> Option<(u16, u16)> {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_resize_time);

        if elapsed >= self.debounce_duration {
            if let Some(size) = self.pending_size.take() {
                self.last_resize_time = now;
                return Some(size);
            }
        }
        None
    }
}
```

**Component 2: TuiFrontend Integration**

Add field to TuiFrontend struct:
```rust
pub struct TuiFrontend {
    // ... existing fields ...
    resize_debouncer: ResizeDebouncer,
}
```

Initialize in TuiFrontend::new():
```rust
resize_debouncer: ResizeDebouncer::new(100), // 100ms debounce
```

**Component 3: Event Loop Modification**

Modify `poll_events()` method in `src/frontend/tui/mod.rs`:

```rust
fn poll_events(&mut self) -> Result<Vec<FrontendEvent>> {
    let mut events = Vec::new();

    // Poll for events (non-blocking)
    if event::poll(std::time::Duration::from_millis(16))? {
        match event::read()? {
            Event::Key(key) => {
                // ... existing key handling ...
            }
            Event::Resize(width, height) => {
                // Apply debouncing
                if let Some((w, h)) = self.resize_debouncer.check_resize(width, height) {
                    events.push(FrontendEvent::Resize { width: w, height: h });
                }
                // If None, resize is pending and will be checked below
            }
            Event::Mouse(mouse) => {
                // ... existing mouse handling ...
            }
            Event::Paste(text) => {
                // ... existing paste handling ...
            }
            _ => {}
        }
    }

    // Check for pending resize (if debounce period has passed)
    if let Some((width, height)) = self.resize_debouncer.check_pending() {
        events.push(FrontendEvent::Resize { width, height });
    }

    Ok(events)
}
```

### Testing Specification

**Test 1: Debounce Timer Accuracy**
- Unit test: Verify check_resize() returns None within 100ms window
- Unit test: Verify check_resize() returns Some() after 100ms elapsed
- Unit test: Verify pending_size is stored correctly

**Test 2: Pending Resize Processing**
- Unit test: Verify check_pending() returns None before debounce period
- Unit test: Verify check_pending() returns Some() after debounce period
- Unit test: Verify pending_size is cleared after processing

**Test 3: Integration Test**
- Simulate rapid resize events (10 events within 200ms)
- Verify only 1-2 resize events are emitted
- Verify final resize is processed within 100ms of last event

**Test 4: Performance Validation**
- Benchmark: Measure render calls during 1-second continuous resize
- Expected: 70%+ reduction compared to no-debounce baseline
- Measure frame time stays under 16ms

---

## Traceability

**Parent SPEC**: SPEC-TF-ANALYSIS-001 (Feature Parity Analysis)
**Regression Type**: Performance degradation (immediate resize handling)
**Reference Implementation**: VellumFE `src/app.rs` lines 30-77, 4814-4835

**Related Components**:
- `src/frontend/tui/mod.rs` - Primary modification target
- `src/frontend/events.rs` - FrontendEvent::Resize definition (no changes)
- `src/core/app_core.rs` - Downstream resize handling (no changes)

**Affected Files**:
- `src/frontend/tui/mod.rs` - Add ResizeDebouncer struct and integration

**Test Coverage**:
- Unit tests: `src/frontend/tui/mod.rs` tests module
- Integration tests: Resize event simulation
- Performance benchmarks: Render call frequency, frame time

---

## Success Criteria

1. **Debounce Implementation**: ResizeDebouncer struct correctly implements 100ms timing
2. **Event Reduction**: 70%+ reduction in resize events during rapid terminal resizing
3. **Responsiveness**: Final resize processed within 100ms of user stopping drag
4. **No Regressions**: All existing tests pass without modification
5. **Performance**: Frame time remains under 16ms during resize operations
6. **Code Quality**: Implementation matches VellumFE pattern for maintainability

---

## Notes

**Design Decision: 100ms vs. 300ms**
- VellumFE uses 300ms debounce (conservative approach)
- Two-Face uses 100ms for better responsiveness while maintaining performance benefits
- Justification: Modern terminals and faster event loops allow shorter debounce without flicker

**Future Enhancement Opportunities**:
- Adaptive debounce duration based on terminal size changes
- Configurable debounce timing via config.toml
- Performance metrics tracking (resize event reduction ratio)
