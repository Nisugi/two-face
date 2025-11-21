# Implementation Plan: SPEC-TF-PARITY-RESIZE-001

**TAG**: `SPEC-TF-PARITY-RESIZE-001`
**Feature**: Terminal Resize Debouncing
**Estimated Complexity**: Simple (focused scope, clear reference implementation)

---

## Implementation Milestones

### Primary Goal: Implement Resize Debouncer

**Objective**: Add ResizeDebouncer struct to Two-Face TUI frontend

**Tasks**:
1. Add ResizeDebouncer struct to `src/frontend/tui/mod.rs` (before TuiFrontend definition)
   - Copy VellumFE implementation as baseline
   - Adjust debounce_duration to 100ms
   - Preserve two-phase approach (check_resize + check_pending)

2. Integrate ResizeDebouncer into TuiFrontend struct
   - Add `resize_debouncer: ResizeDebouncer` field
   - Initialize in `TuiFrontend::new()` with 100ms duration
   - Ensure proper ownership and lifetime management

**Dependencies**: None (standalone implementation)

---

### Secondary Goal: Modify Event Loop

**Objective**: Apply debouncing logic in poll_events() method

**Tasks**:
1. Modify Event::Resize handling in `poll_events()` (line ~2184)
   - Call `self.resize_debouncer.check_resize(width, height)`
   - Only push FrontendEvent::Resize if Some() is returned
   - Remove direct event push for resize events

2. Add pending resize check after event polling
   - Call `self.resize_debouncer.check_pending()` after event::poll() block
   - Push FrontendEvent::Resize if pending resize is ready
   - Ensure this runs on every poll_events() call

**Dependencies**: Primary Goal must be complete

---

### Final Goal: Testing and Validation

**Objective**: Verify debounce timing and performance improvement

**Tasks**:
1. Add unit tests for ResizeDebouncer
   - Test check_resize() returns None within 100ms window
   - Test check_resize() returns Some() after 100ms elapsed
   - Test pending_size storage and retrieval
   - Test check_pending() behavior before/after debounce period

2. Create integration test for event loop
   - Simulate rapid resize events (10+ events in 200ms)
   - Verify event reduction (expect 1-2 emitted vs. 10+ input)
   - Verify final resize is processed within 100ms

3. Performance validation
   - Manual testing: Resize terminal window continuously for 2-3 seconds
   - Observe render smoothness (should be noticeably smoother than current behavior)
   - Check performance stats (if available) for render call frequency
   - Compare against VellumFE resize behavior (should match or exceed smoothness)

**Dependencies**: Primary and Secondary Goals must be complete

---

## Technical Approach

### Architecture Design Direction

**Pattern**: Two-Phase Debouncing (Proven Pattern from VellumFE)

**Phase 1 - Immediate Check**:
- On resize event arrival, check if 100ms has elapsed since last processed resize
- If yes: Process immediately, reset timer
- If no: Store as pending resize

**Phase 2 - Pending Processing**:
- On every event loop iteration, check if pending resize exists
- If pending exists and 100ms elapsed: Process pending resize, clear pending state
- This ensures final resize is processed even if no new events arrive

**Timing Architecture**:
```
User drags window edge:
  t=0ms:   Resize event 1 arrives → Process immediately (first event)
  t=20ms:  Resize event 2 arrives → Store as pending (within 100ms window)
  t=40ms:  Resize event 3 arrives → Update pending size
  t=60ms:  Resize event 4 arrives → Update pending size
  t=80ms:  Resize event 5 arrives → Update pending size
  t=110ms: Event loop checks pending → Process pending resize (100ms elapsed)
  t=130ms: Resize event 6 arrives → Process immediately (110ms since last)
```

### Integration Strategy

**File Modifications**:
- `src/frontend/tui/mod.rs`: Single file modification
  - Add ResizeDebouncer struct (before TuiFrontend)
  - Add field to TuiFrontend struct
  - Modify poll_events() method (2 changes: resize event handling + pending check)

**No Modifications Required**:
- `src/frontend/events.rs` - FrontendEvent::Resize unchanged
- `src/core/app_core.rs` - Resize handling logic unchanged
- `src/core/input_router.rs` - Routing logic unchanged
- Any downstream components - API compatibility maintained

**Risk Assessment**: LOW
- Isolated change (single file, single component)
- No breaking changes to public APIs
- Reference implementation proven in production (VellumFE)
- Minimal performance overhead (2 Instant comparisons per event loop iteration)

---

## Implementation Notes

### VellumFE Reference Behavior

**VellumFE Debouncer Characteristics**:
- 300ms debounce duration (conservative for stability)
- Stores last_resize_time as Instant (monotonic clock)
- Uses Option<(u16, u16)> for pending size (memory efficient)
- Two-phase processing prevents "lost" final resize events

**Event Loop Integration** (VellumFE app.rs lines 4812-4835):
```rust
Event::Resize(width, height) => {
    if let Some((w, h)) = self.resize_debouncer.check_resize(width, height) {
        tracing::debug!("Terminal resized to {}x{} (debounced)", w, h);
        self.auto_scale_layout(w, h);
    } else {
        tracing::trace!("Resize debounced (waiting)", width, height);
    }
}

// Later in event loop:
if let Some((width, height)) = self.resize_debouncer.check_pending() {
    tracing::debug!("Processing pending resize to {}x{}", width, height);
    self.auto_scale_layout(width, height);
}
```

**Two-Face Adaptation**:
- Replace `self.auto_scale_layout()` with `events.push(FrontendEvent::Resize)`
- Use 100ms instead of 300ms for better responsiveness
- Maintain two-phase pattern (critical for correctness)

### Performance Expectations

**Current Behavior** (No Debouncing):
- Terminal resize generates 15-30 events/second during continuous drag
- Each event triggers full layout recalculation + render
- Frame time may spike to 30-50ms (below 60 FPS)
- Potential flicker or lag perception

**Expected Behavior** (With 100ms Debouncing):
- ~10 events/second processed (70% reduction)
- Frame time stays under 16ms (60 FPS maintained)
- Smooth rendering during resize operations
- Final resize processed within 100ms of user stopping drag

**Measurable Improvement**:
- Resize event reduction: 70-80% (baseline: 15-30/sec → 3-6/sec)
- Render call reduction: 70-80% (proportional to event reduction)
- Frame time improvement: 30-50ms → <16ms during resize
- User-perceived smoothness: Matches VellumFE reference behavior

---

## Testing Strategy

### Unit Test Coverage

**Test Module**: `src/frontend/tui/mod.rs` (add #[cfg(test)] mod tests)

**Test Cases**:
1. `test_resize_debouncer_immediate_first_resize()`
   - First resize always processes immediately
   - Verify last_resize_time is updated

2. `test_resize_debouncer_debounce_rapid_resizes()`
   - Send 5 resizes within 50ms
   - Verify only first returns Some(), rest return None
   - Verify pending_size stores latest dimensions

3. `test_resize_debouncer_pending_processing()`
   - Send resize, verify it's pending
   - Wait 101ms, call check_pending()
   - Verify pending resize is returned and cleared

4. `test_resize_debouncer_no_pending_when_none_stored()`
   - Call check_pending() with no pending resize
   - Verify returns None

### Integration Test Coverage

**Test Module**: Integration tests (if Two-Face has integration test directory)

**Test Scenario**: Rapid Resize Event Stream
```rust
#[test]
fn test_resize_event_debouncing() {
    let mut frontend = TuiFrontend::new(...);

    // Simulate rapid resize events
    // (This would require mocking crossterm event::poll/read)
    // Expect: Only 1-2 resize events emitted from 10 input events
}
```

**Note**: Integration test may require refactoring poll_events() for testability (dependency injection of event source)

### Manual Testing Checklist

**Test Procedure**:
1. Build Two-Face in release mode (cargo build --release)
2. Launch Two-Face and connect to game
3. Continuously resize terminal window for 2-3 seconds
4. Observe rendering behavior:
   - ✅ Smooth rendering during resize
   - ✅ No flicker or lag
   - ✅ Final layout reflects terminal dimensions within 100ms
5. Compare against VellumFE (if available):
   - Two-Face resize smoothness should match or exceed VellumFE

**Performance Metrics** (if available):
- Check `src/performance.rs` PerformanceStats
- Log render call frequency during resize
- Verify frame time stays under 16ms

---

## Risk Mitigation

### Potential Issues

**Issue 1: Debounce Duration Too Short**
- Symptom: Still experiencing performance issues during resize
- Mitigation: Increase debounce to 150ms or 200ms
- Verification: Test with various debounce values (50ms, 100ms, 150ms, 200ms)

**Issue 2: Pending Resize Not Processed**
- Symptom: Final resize dimensions not reflected in UI
- Mitigation: Ensure check_pending() is called on every poll_events() iteration
- Verification: Add tracing logs to check_pending() to verify execution

**Issue 3: Event Loop Overhead**
- Symptom: Increased CPU usage from check_pending() calls
- Mitigation: check_pending() is lightweight (2 Instant comparisons, O(1))
- Verification: Profile CPU usage during resize operations

**Issue 4: Race Condition with Pending State**
- Symptom: Incorrect resize dimensions processed
- Mitigation: ResizeDebouncer stores latest pending_size (correct by design)
- Verification: Unit test verifying latest size overwrites previous pending size

---

## Definition of Done

- ✅ ResizeDebouncer struct implemented in `src/frontend/tui/mod.rs`
- ✅ TuiFrontend integrates ResizeDebouncer with 100ms timing
- ✅ poll_events() modified to apply debouncing logic
- ✅ Unit tests pass (debouncer timing verification)
- ✅ Integration tests pass (event reduction verification)
- ✅ Manual testing confirms smooth resize behavior
- ✅ Performance metrics show 70%+ event reduction (if measurable)
- ✅ Code follows VellumFE reference pattern for maintainability
- ✅ No breaking changes to existing resize handling
- ✅ Documentation updated (inline comments explain debouncing logic)

---

## Next Steps After Implementation

1. **Merge to Main Branch**
   - Ensure all tests pass
   - Code review (if team workflow)
   - Merge feature branch

2. **Monitor Production Performance**
   - Gather user feedback on resize smoothness
   - Check for any unexpected behavior
   - Validate performance improvement in real-world usage

3. **Consider Follow-up Enhancements**
   - Make debounce duration configurable via config.toml
   - Add performance metrics tracking (resize event reduction ratio)
   - Explore adaptive debouncing based on terminal size delta

4. **Update SPEC-TF-ANALYSIS-001**
   - Mark "Terminal Resize Debouncing" regression as RESOLVED
   - Document performance improvement metrics
   - Update parity status: REGRESSION → SAME (or ENHANCED if 100ms proves superior)
