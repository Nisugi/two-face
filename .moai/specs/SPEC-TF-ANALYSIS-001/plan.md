# SPEC-TF-ANALYSIS-001: Implementation Plan

**TAG BLOCK**:
```yaml
spec_id: SPEC-TF-ANALYSIS-001
version: 1.0.0
status: draft
category: analysis
domain: TF-ANALYSIS
```

---

## Overview

This plan outlines the step-by-step approach to conducting a comprehensive comparative analysis between VellumFE (reference implementation) and Two-Face (refactored target).

**Goal**: Produce 4 deliverables that identify feature parity gaps, widget styling inconsistencies, architectural differences, and actionable next steps for Two-Face development.

---

## Implementation Strategy

### Phase 1: Project Familiarization
**Objective**: Understand both codebases and establish analysis baseline

**Tasks**:
1. **VellumFE Code Review**
   - Read `src/app.rs` to understand main event loop and state management
   - Survey `src/ui/` widgets for styling patterns and rendering logic
   - Review `src/config.rs`, `src/parser.rs`, `src/network.rs` for shared infrastructure
   - Document VellumFE's monolithic architecture pattern

2. **Two-Face Code Review**
   - Read `src/core/app_core.rs` to understand frontend-agnostic core
   - Review `src/data/ui_state.rs` and `src/data/widget.rs` for state separation
   - Survey `src/frontend/tui/` widgets for TUI-specific rendering
   - Analyze `src/core/input_router.rs` and `src/core/event_bridge.rs` for abstraction layer
   - Document Two-Face's multi-frontend architecture pattern

3. **Documentation Cross-Reference**
   - Read `docs/THEME_ARCHITECTURE.md` for theme system design
   - Review `docs/wiki/project_overview.md`, `docs/wiki/widgets.md`, etc.
   - Check README.md roadmap and PROGRESS.md (if exists) for known gaps
   - Identify documented vs implemented features

**Deliverable**: Internal notes on architecture patterns and feature inventory

---

### Phase 2: Feature Parity Analysis
**Objective**: Create `feature_parity.md` with comprehensive feature-by-feature comparison

**Tasks**:
1. **Identify Feature Areas**
   - Launch flow (CLI args, connection establishment, authentication)
   - Configuration system (loading, saving, validation)
   - Menu system (popup menus, context menus, keybinds)
   - Editor widgets (theme editor, settings editor, highlight form, keybind form, window editor)
   - Logging and error handling (error messages, logging verbosity)
   - Terminal resize behavior (debouncing, layout recalculation)
   - Performance instrumentation (stats display, performance tracking)
   - Accessibility (TTS support, sound playback)
   - Highlight and automation (highlight triggers, command lists, macros)
   - Window management (dynamic layout, window editing, stream routing)

2. **Behavioral Comparison** (for each feature area):
   - **Static Analysis**: Compare implementation in both codebases
   - **Dynamic Testing** (if feasible): Run both apps and observe behavior
   - **Documentation**: Check for documented vs actual behavior differences
   - **Categorize Parity**:
     - **Same**: Identical behavior and implementation
     - **Intentional Difference**: Two-Face deliberately changed for improvement
     - **Regression**: Two-Face lost VellumFE functionality
     - **Missing**: Two-Face hasn't implemented yet

3. **Create Feature Matrix Table**
   - One row per feature area (minimum 15 rows)
   - Columns: VellumFE Behavior, Two-Face Behavior, Parity Status, Notes
   - Use concrete examples (e.g., "VellumFE prompts on quit, Two-Face auto-saves")
   - Flag high-impact regressions and missing features

4. **Write Feature Parity Narrative**
   - Summary of overall parity status
   - Highlight critical regressions that block feature parity
   - Identify intentional improvements in Two-Face
   - Call out missing features with high user impact

**Deliverable**: `feature_parity.md`

---

### Phase 3: Widget Style Analysis
**Objective**: Create `widget_style.md` with visual consistency and theme integration analysis

**Tasks**:
1. **Identify Widget Types**
   - Popup menus (`popup_menu.rs`)
   - Context menus (in-game right-click menus)
   - Editor widgets (theme editor, settings editor, highlight form, keybind form, window editor)
   - Dialogs and prompts (confirmation dialogs, input prompts)
   - Status bars and indicators (status line, vitals, indicators)
   - Error panels and notifications (error display, toast notifications)
   - Focus states (focused vs unfocused borders, titles, colors)

2. **VellumFE Style Extraction** (for each widget):
   - Analyze `src/ui/<widget>.rs` for hardcoded colors and styles
   - Document border colors, background colors, text colors, modifiers
   - Identify focus/unfocus state differences
   - Note any theme-aware vs hardcoded styling

3. **Two-Face Style Extraction** (for each widget):
   - Analyze `src/frontend/tui/<widget>.rs` for styling patterns
   - Document AppTheme field usage (which of 77 fields are used)
   - Identify inconsistencies between widgets (different border colors, text styles)
   - Check for TUI-specific hardcoded styles vs theme-driven styles

4. **Define Desired Standard** (for each widget):
   - Propose unified styling that leverages AppTheme system
   - Identify missing AppTheme fields needed for consistency
   - Consider GUI port implications (e.g., avoid TUI-specific assumptions)
   - Balance backwards compatibility with VellumFE and future GUI needs

5. **Theme System Integration Assessment**
   - How well do current widgets use AppTheme's 77 fields?
   - Are there gaps in AppTheme coverage (missing fields for certain widgets)?
   - Are widgets consistent in their theme field usage?
   - Recommendations for AppTheme extension or refactoring

**Deliverable**: `widget_style.md`

---

### Phase 4: Architecture Comparison
**Objective**: Create `architecture_summary.md` with high-level structural analysis

**Tasks**:
1. **Module Layout Comparison**
   - Create side-by-side directory tree comparison
   - VellumFE: `src/app.rs`, `src/ui/`, `src/config.rs`, etc.
   - Two-Face: `src/core/`, `src/data/`, `src/frontend/tui/`, `src/frontend/gui/`, etc.
   - Highlight structural differences (monolithic vs layered)
   - Note shared modules (config, parser, network) that exist in both

2. **State Management Analysis**
   - **VellumFE**: Document `App` struct and direct state mutation pattern
   - **Two-Face**: Document `AppCore` + `UiState` separation
   - Trace state flow: event → mutation → render
   - Identify coupling points (where state and UI are tightly bound)
   - Assess frontend-agnostic design success

3. **Input/Event Handling Analysis**
   - **VellumFE**: Event loop in `app.rs`, direct widget input handlers
   - **Two-Face**: `input_router.rs`, `event_bridge.rs`, abstraction layer
   - Trace input flow: keyboard event → router → action → state update
   - Identify abstraction benefits and overhead costs

4. **Rendering Pipeline Analysis**
   - **VellumFE**: `App::render()` directly uses ratatui to draw widgets
   - **Two-Face**: `AppCore` prepares state → Frontend reads state → Renders with ratatui
   - Assess decoupling success and performance implications
   - Identify TUI-specific assumptions that leak into core

5. **Wins and Losses Assessment**
   - **Wins**: Where Two-Face is cleaner, more modular, or better designed
   - **Losses**: Where Two-Face is messier, more coupled, or regressed
   - **GUI Port Blockers**: TUI-specific assumptions in core that will hurt egui port
   - **GUI Port Enablers**: Abstraction points that make egui port easier

**Deliverable**: `architecture_summary.md`

---

### Phase 5: Recommendations Synthesis
**Objective**: Create `recommendations.md` with prioritized follow-up SPECs

**Tasks**:
1. **Synthesize Findings**
   - Review all findings from feature parity, widget style, and architecture analyses
   - Identify common themes (e.g., style inconsistency, missing features, coupling issues)
   - Group related issues into logical SPEC candidates

2. **Generate SPEC Candidates**
   - **SPEC-TF-STYLE-XXX**: Style and theme unification tasks
     - Example: SPEC-TF-STYLE-MENU-001 (Unify popup menu styling)
     - Example: SPEC-TF-STYLE-EDITOR-001 (Standardize editor widget borders)
   - **SPEC-TF-PARITY-XXX**: Behavioral regression fixes and missing features
     - Example: SPEC-TF-PARITY-RESIZE-001 (Add resize debouncing)
     - Example: SPEC-TF-PARITY-TTS-001 (Port TTS prompting from VellumFE)
   - **SPEC-TF-CORE-XXX**: Core API cleanup for multi-frontend support
     - Example: SPEC-TF-CORE-STATE-001 (Remove TUI assumptions from UiState)
     - Example: SPEC-TF-CORE-RENDERER-001 (Abstract rendering interface for egui)

3. **Prioritize Recommendations**
   - **High Priority**: Critical regressions, GUI port blockers
   - **Medium Priority**: Style inconsistencies, moderate feature gaps
   - **Low Priority**: Nice-to-have improvements, long-term refactors

4. **Document Each Recommendation**
   - **Rationale**: Why this SPEC matters
   - **Affected Files/Modules**: Where changes will occur
   - **Complexity Estimate**: Simple, Moderate, Complex
   - **Dependencies**: Blocking or blocked-by relationships
   - **GUI Port Impact**: Critical, Helpful, Neutral

5. **Create Dependency Graph** (optional)
   - Visualize SPEC dependencies (e.g., SPEC-TF-CORE-STATE-001 blocks SPEC-TF-GUI-001)
   - Identify parallelizable vs sequential work

**Deliverable**: `recommendations.md`

---

## Technical Approach

### Analysis Tools and Techniques

1. **Static Code Analysis**
   - File and directory structure comparison
   - Module dependency tracing (use `use` statements)
   - Widget implementation pattern analysis (hardcoded vs theme-driven)
   - State mutation tracing (where and how state changes)

2. **Dynamic Behavioral Testing** (if feasible)
   - Build and run VellumFE: `cd vellumfe && cargo run --release`
   - Build and run Two-Face: `cd two-face && cargo run --release`
   - Test scenarios:
     - Launch and connection flow
     - Open theme editor, settings editor, highlight browser
     - Trigger resize events (expand/contract terminal)
     - Test TTS and sound playback
     - Modify highlights, keybinds, window layouts
   - Document observed differences in UX

3. **Documentation Cross-Reference**
   - Compare documented behavior (wiki, THEME_ARCHITECTURE.md) with actual code
   - Identify gaps between intended design and implementation

4. **Code Search and Grep**
   - Search for widget rendering code: `grep -r "fn render" src/`
   - Search for theme field usage: `grep -r "theme\." src/`
   - Search for hardcoded colors: `grep -r "Color::" src/`
   - Find state mutation points: `grep -r "pub fn " src/core/`

---

## Milestones and Checkpoints

### Milestone 1: Familiarization Complete
**Checkpoint**: Internal notes on both architectures documented
**Criteria**: Clear understanding of VellumFE monolithic vs Two-Face layered design

### Milestone 2: Feature Parity Analysis Complete
**Checkpoint**: `feature_parity.md` delivered
**Criteria**: Minimum 15 feature areas analyzed with concrete examples

### Milestone 3: Widget Style Analysis Complete
**Checkpoint**: `widget_style.md` delivered
**Criteria**: Minimum 8 widget types analyzed with theme integration assessment

### Milestone 4: Architecture Analysis Complete
**Checkpoint**: `architecture_summary.md` delivered
**Criteria**: Module layout, state management, input handling, rendering pipeline documented

### Milestone 5: Recommendations Synthesized
**Checkpoint**: `recommendations.md` delivered
**Criteria**: Minimum 5 prioritized SPEC candidates with rationale and dependencies

### Milestone 6: SPEC-TF-ANALYSIS-001 Complete
**Checkpoint**: All 4 deliverables reviewed and finalized
**Criteria**: Acceptance criteria from `spec.md` satisfied

---

## Risks and Mitigations

### Risk 1: VellumFE Build Failures
**Impact**: Cannot run dynamic behavioral tests
**Mitigation**: Rely on static code analysis and documentation; use issue history for behavior evidence

### Risk 2: Overwhelming Scope
**Impact**: Analysis takes too long, recommendations become unactionable
**Mitigation**: Timebox each phase; focus on high-impact findings; defer deep-dive details to follow-up SPECs

### Risk 3: Subjective Styling Judgments
**Impact**: "Desired Standard" recommendations lack clear rationale
**Mitigation**: Anchor all style recommendations to:
  - AppTheme system integration
  - GUI port compatibility
  - Consistency across widgets

---

## Dependencies

### Required Resources
- **VellumFE Source Code**: `C:\gemstone\projects\vellumfe`
- **Two-Face Source Code**: `C:\gemstone\projects\two-face`
- **Documentation**: `docs/THEME_ARCHITECTURE.md`, `docs/wiki/*.md`
- **Build Environment**: Rust toolchain (cargo)

### External Dependencies
- None (analysis is self-contained within two codebases)

---

## Success Criteria

### Analysis Quality
- ✅ All findings are evidence-based (code references or test observations)
- ✅ No unsupported generalizations or assumptions
- ✅ Concrete examples for every behavioral difference

### Deliverable Completeness
- ✅ `feature_parity.md`: Minimum 15 feature areas, parity status for each
- ✅ `widget_style.md`: Minimum 8 widget types, VellumFE/Two-Face/Desired Standard for each
- ✅ `architecture_summary.md`: Module layout, state management, input handling, rendering pipeline documented
- ✅ `recommendations.md`: Minimum 5 SPEC candidates with priority, rationale, complexity, dependencies

### Actionability
- ✅ Each recommended SPEC is implementable with clear scope
- ✅ Dependencies between SPECs are documented
- ✅ GUI port impact is explicitly assessed for all recommendations

---

**END OF PLAN**
