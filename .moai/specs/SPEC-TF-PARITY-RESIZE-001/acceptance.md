# Acceptance Criteria: SPEC-TF-PARITY-RESIZE-001

**TAG**: `SPEC-TF-PARITY-RESIZE-001`
**Feature**: Terminal Resize Debouncing
**Test Framework**: Rust #[test], Manual Testing

---

## Test Scenarios (Given-When-Then Format)

### Scenario 1: First Resize Event Processed Immediately

**Given**:
- TuiFrontend is initialized with ResizeDebouncer (100ms debounce)
- No resize events have been processed yet
- ResizeDebouncer.last_resize_time is at initialization time

**When**:
- A terminal resize event arrives (width=120, height=40)

**Then**:
- ResizeDebouncer.check_resize(120, 40) returns Some((120, 40))
- FrontendEvent::Resize { width: 120, height: 40 } is emitted
- ResizeDebouncer.last_resize_time is updated to current time
- ResizeDebouncer.pending_size remains None

**Test Type**: Unit Test
**Test Location**: `src/frontend/tui/mod.rs` tests module
**Test Name**: `test_first_resize_processed_immediately`

---

### Scenario 2: Rapid Resize Events Are Debounced

**Given**:
- A resize event was processed at t=0ms (dimensions: 120x40)
- ResizeDebouncer.last_resize_time = t=0ms

**When**:
- Resize event arrives at t=20ms (dimensions: 125x40)
- Resize event arrives at t=40ms (dimensions: 130x40)
- Resize event arrives at t=60ms (dimensions: 135x40)
- Resize event arrives at t=80ms (dimensions: 140x40)

**Then**:
- All check_resize() calls return None (within 100ms window)
- ResizeDebouncer.pending_size = Some((140, 40)) (latest dimensions)
- No FrontendEvent::Resize is emitted during this phase
- ResizeDebouncer.last_resize_time remains at t=0ms

**Test Type**: Unit Test
**Test Location**: `src/frontend/tui/mod.rs` tests module
**Test Name**: `test_rapid_resizes_debounced`

---

### Scenario 3: Pending Resize Processed After Debounce Period

**Given**:
- A resize was stored as pending at t=0ms (dimensions: 140x40)
- ResizeDebouncer.pending_size = Some((140, 40))
- ResizeDebouncer.last_resize_time = t=0ms (from previous processed resize)

**When**:
- Event loop iteration occurs at t=110ms
- check_pending() is called

**Then**:
- check_pending() returns Some((140, 40))
- FrontendEvent::Resize { width: 140, height: 40 } is emitted
- ResizeDebouncer.pending_size is cleared (becomes None)
- ResizeDebouncer.last_resize_time is updated to t=110ms

**Test Type**: Unit Test
**Test Location**: `src/frontend/tui/mod.rs` tests module
**Test Name**: `test_pending_resize_processed_after_debounce`

---

### Scenario 4: Resize After Debounce Period Processed Immediately

**Given**:
- Last resize was processed at t=0ms
- ResizeDebouncer.last_resize_time = t=0ms
- ResizeDebouncer.pending_size = None

**When**:
- Resize event arrives at t=150ms (dimensions: 150x50)

**Then**:
- check_resize(150, 50) returns Some((150, 50))
- FrontendEvent::Resize { width: 150, height: 50 } is emitted immediately
- ResizeDebouncer.last_resize_time = t=150ms
- ResizeDebouncer.pending_size remains None

**Test Type**: Unit Test
**Test Location**: `src/frontend/tui/mod.rs` tests module
**Test Name**: `test_resize_after_debounce_period_immediate`

---

### Scenario 5: Multiple Pending Resizes Store Latest Dimensions

**Given**:
- Last resize processed at t=0ms
- ResizeDebouncer.pending_size = None

**When**:
- Resize event arrives at t=20ms (dimensions: 120x40) → stored as pending
- Resize event arrives at t=40ms (dimensions: 130x40) → updates pending
- Resize event arrives at t=60ms (dimensions: 140x40) → updates pending

**Then**:
- ResizeDebouncer.pending_size = Some((140, 40)) (only latest stored)
- Intermediate sizes (120x40, 130x40) are discarded
- check_pending() at t=110ms returns Some((140, 40))

**Test Type**: Unit Test
**Test Location**: `src/frontend/tui/mod.rs` tests module
**Test Name**: `test_multiple_pending_resizes_store_latest`

---

### Scenario 6: No Pending Resize Returns None

**Given**:
- ResizeDebouncer.pending_size = None
- No pending resize exists

**When**:
- check_pending() is called

**Then**:
- check_pending() returns None
- No FrontendEvent::Resize is emitted

**Test Type**: Unit Test
**Test Location**: `src/frontend/tui/mod.rs` tests module
**Test Name**: `test_no_pending_resize_returns_none`

---

### Scenario 7: Integration - Resize Event Reduction During Rapid Resizing

**Given**:
- Two-Face TUI is running in release mode
- Terminal emulator sends 15 resize events over 500ms (simulated rapid drag)

**When**:
- User drags terminal window edge continuously
- Resize events flow through poll_events()

**Then**:
- Approximately 5-6 FrontendEvent::Resize events are emitted (70%+ reduction)
- First event is processed immediately
- Subsequent events within 100ms are debounced
- Final resize is processed within 100ms of last event

**Test Type**: Integration Test (Manual)
**Test Procedure**:
1. Build Two-Face in release mode
2. Launch and connect to game
3. Drag terminal window edge continuously for 2-3 seconds
4. Observe rendering behavior (should be smooth, no flicker)
5. Check logs (if tracing enabled) for resize event processing frequency

**Expected Metrics**:
- Event reduction: 70-80%
- Render smoothness: 60 FPS maintained
- Final resize latency: <100ms

---

### Scenario 8: Performance - Frame Time During Resize

**Given**:
- Two-Face TUI is running with performance stats enabled
- Terminal is being actively resized (continuous drag operation)

**When**:
- User drags terminal window for 2 seconds
- Performance stats track frame time

**Then**:
- Frame time remains under 16ms (60 FPS threshold)
- No frame time spikes above 20ms
- Rendering is smooth and responsive

**Test Type**: Performance Test (Manual)
**Test Procedure**:
1. Enable performance stats (if available in Two-Face)
2. Launch Two-Face in release mode
3. Resize terminal continuously for 2-3 seconds
4. Check performance stats output for frame time metrics

**Expected Results**:
- Average frame time: <10ms
- Max frame time: <16ms
- No perceived lag or stuttering

---

### Scenario 9: Parity - Match VellumFE Resize Behavior

**Given**:
- VellumFE is available for comparison testing
- Both VellumFE and Two-Face are running in release mode
- Same terminal emulator and environment

**When**:
- Resize terminal window for VellumFE
- Resize terminal window for Two-Face
- Compare rendering smoothness and responsiveness

**Then**:
- Two-Face resize smoothness matches or exceeds VellumFE
- No visual difference in rendering during resize
- Final resize processed within same latency as VellumFE

**Test Type**: Comparative Manual Test
**Test Procedure**:
1. Launch VellumFE, resize window, observe behavior
2. Launch Two-Face, resize window, observe behavior
3. Compare smoothness and final layout update latency

**Expected Results**:
- Two-Face: Equal or better smoothness than VellumFE
- Two-Face: Equal or better responsiveness (100ms vs. 300ms debounce)

---

### Scenario 10: No Breaking Changes - Existing Resize Handlers Work

**Given**:
- Two-Face has existing resize handling logic in app_core.rs and other components
- Debouncing is implemented in TuiFrontend

**When**:
- A resize event is processed (either immediate or pending)
- FrontendEvent::Resize is emitted

**Then**:
- All downstream resize handlers receive the event unchanged
- Layout recalculations execute as before
- Window dimensions update correctly
- No errors or panics occur

**Test Type**: Integration Test (Existing Test Suite)
**Test Procedure**:
1. Run existing Two-Face test suite
2. Verify all tests pass without modification
3. Manual testing: Resize terminal, verify UI reflects new dimensions

**Expected Results**:
- All existing tests pass
- No regressions in resize handling behavior
- UI correctly reflects terminal dimensions after resize

---

## Quality Gate Criteria

### Code Quality

**CQ-1: Implementation Matches Reference Pattern**
- ✅ ResizeDebouncer struct matches VellumFE design
- ✅ Two-phase debouncing logic (check_resize + check_pending) implemented
- ✅ Timing logic uses std::time::Instant and Duration
- ✅ Code is readable with clear inline comments

**CQ-2: Rust Best Practices**
- ✅ No unwrap() or expect() in production code (use Option properly)
- ✅ No unsafe code
- ✅ Proper ownership and borrowing (no unnecessary clones)
- ✅ Idiomatic Rust patterns (Option::take(), duration_since(), etc.)

**CQ-3: Documentation**
- ✅ ResizeDebouncer struct has doc comments
- ✅ check_resize() and check_pending() methods documented
- ✅ Inline comments explain debouncing strategy
- ✅ Integration points in poll_events() clearly commented

---

### Test Coverage

**TC-1: Unit Test Coverage**
- ✅ 6+ unit tests for ResizeDebouncer behavior
- ✅ All test scenarios (1-6) implemented and passing
- ✅ Edge cases covered (no pending, multiple pending, timing boundaries)

**TC-2: Integration Test Coverage**
- ✅ Existing test suite passes without modification
- ✅ No regressions in resize handling
- ✅ Manual integration testing completed (scenarios 7-10)

**TC-3: Performance Validation**
- ✅ Event reduction measured (target: 70%+)
- ✅ Frame time measured during resize (target: <16ms)
- ✅ Comparison with VellumFE (parity or improvement)

---

### Performance Metrics

**PM-1: Event Reduction**
- **Baseline**: 15-30 resize events/second during continuous drag
- **Target**: 3-6 resize events/second (70-80% reduction)
- **Measurement**: Log resize event processing frequency
- **Pass Criteria**: ≥70% reduction in processed events

**PM-2: Frame Time**
- **Baseline**: 30-50ms during resize (without debouncing)
- **Target**: <16ms during resize (60 FPS)
- **Measurement**: Performance stats frame time tracking
- **Pass Criteria**: Frame time ≤16ms during resize operations

**PM-3: Responsiveness**
- **Target**: Final resize processed within 100ms of last event
- **Measurement**: Manual observation + timing logs
- **Pass Criteria**: UI reflects final dimensions ≤100ms after user stops drag

---

### Regression Prevention

**RP-1: No Breaking Changes**
- ✅ FrontendEvent::Resize API unchanged
- ✅ No modifications to app_core.rs resize handling
- ✅ No modifications to input_router.rs
- ✅ Existing tests pass without changes

**RP-2: No New Dependencies**
- ✅ Uses only std::time (no external crates)
- ✅ Cargo.toml unchanged
- ✅ Build process unchanged

**RP-3: Backward Compatibility**
- ✅ Debouncing can be disabled by setting duration to 0ms (if needed)
- ✅ No configuration changes required for users
- ✅ Behavior transparent to downstream components

---

## Definition of Done

**All Criteria Must Be Met**:

1. **Implementation Complete**:
   - ✅ ResizeDebouncer struct added to `src/frontend/tui/mod.rs`
   - ✅ TuiFrontend integrates ResizeDebouncer (100ms timing)
   - ✅ poll_events() modified with debouncing logic

2. **Tests Pass**:
   - ✅ All unit tests pass (scenarios 1-6)
   - ✅ All integration tests pass (existing suite)
   - ✅ Manual testing confirms smooth resize (scenarios 7-9)

3. **Performance Validated**:
   - ✅ Event reduction ≥70% measured
   - ✅ Frame time ≤16ms during resize
   - ✅ Parity with VellumFE confirmed

4. **Quality Gates Pass**:
   - ✅ Code quality criteria met (CQ-1, CQ-2, CQ-3)
   - ✅ Test coverage criteria met (TC-1, TC-2, TC-3)
   - ✅ Performance metrics met (PM-1, PM-2, PM-3)
   - ✅ Regression prevention verified (RP-1, RP-2, RP-3)

5. **Documentation Updated**:
   - ✅ Inline code comments explain debouncing logic
   - ✅ ResizeDebouncer methods have doc comments
   - ✅ SPEC-TF-ANALYSIS-001 updated (regression resolved)

6. **Ready for Merge**:
   - ✅ All tests green
   - ✅ Code review completed (if team workflow)
   - ✅ Branch ready to merge to main/master

---

## Test Execution Checklist

**Before Implementation**:
- ✅ Review VellumFE reference implementation (src/app.rs lines 30-77)
- ✅ Verify Two-Face event loop structure (src/frontend/tui/mod.rs)
- ✅ Confirm SPEC-TF-PARITY-RESIZE-001 ID is unique (no conflicts)

**During Implementation**:
- ✅ Implement ResizeDebouncer struct
- ✅ Write unit tests incrementally (TDD approach)
- ✅ Integrate into TuiFrontend
- ✅ Modify poll_events() with debouncing logic

**After Implementation**:
- ✅ Run unit tests (cargo test)
- ✅ Run full test suite (cargo test --all)
- ✅ Build release binary (cargo build --release)
- ✅ Manual testing: Resize terminal, verify smoothness
- ✅ Performance testing: Measure event reduction and frame time
- ✅ Comparative testing: Compare with VellumFE (if available)

**Before Merge**:
- ✅ All tests pass
- ✅ Performance criteria met
- ✅ No regressions detected
- ✅ Code review approved (if applicable)
- ✅ Documentation updated

---

## Success Metrics Summary

| Metric                        | Baseline (Current) | Target (Post-Implementation) | Pass Criteria       |
| ----------------------------- | ------------------ | ---------------------------- | ------------------- |
| Resize Events Processed/sec   | 15-30              | 3-6                          | ≥70% reduction      |
| Frame Time During Resize (ms) | 30-50              | <16                          | ≤16ms               |
| Final Resize Latency (ms)     | Immediate          | <100                         | ≤100ms              |
| Unit Test Coverage            | N/A                | 6+ tests                     | All tests pass      |
| Integration Test Regressions  | N/A                | 0                            | All existing pass   |
| User-Perceived Smoothness     | Laggy/Flicker      | Smooth                       | Matches VellumFE    |

---

## Notes

**Testing Philosophy**:
- Unit tests verify debouncer timing correctness
- Integration tests verify event reduction and no regressions
- Manual tests verify user-perceived smoothness and parity with VellumFE
- Performance tests quantify improvement

**Known Limitations**:
- Integration tests may require mocking crossterm event polling (future enhancement)
- Performance metrics depend on availability of Two-Face performance tracking infrastructure
- Comparative testing requires VellumFE installation (optional but recommended)

**Future Test Enhancements**:
- Automated performance regression tests in CI
- Parameterized tests for different debounce durations (50ms, 100ms, 150ms, 200ms)
- Stress testing with extreme resize frequencies (50+ events/second)
