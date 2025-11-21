# SPEC-TF-ANALYSIS-001: Two-Face vs VellumFE Comparative Analysis

**TAG BLOCK**:
```yaml
spec_id: SPEC-TF-ANALYSIS-001
version: 1.1.0
status: complete
category: analysis
domain: TF-ANALYSIS
priority: high
created_at: 2025-11-21
updated_at: 2025-11-21
author: @user
parent_spec: null
derived_from: null
supersedes: null
```

**Related Tags**: None (Initial analysis SPEC)

---

## 1. Environment

**WHEN** analyzing the Two-Face refactored codebase **AND** comparing it against the VellumFE reference implementation:

### Technical Context
- **Two-Face Location**: `C:\gemstone\projects\two-face`
- **VellumFE Location**: `C:\gemstone\projects\vellumfe`
- **Two-Face Architecture**:
  - Frontend-agnostic core (`src/core/`, `src/data/`)
  - Ratatui TUI implementation (`src/frontend/tui/`)
  - Placeholder GUI module (`src/frontend/gui/`)
- **VellumFE Architecture**:
  - Monolithic TUI application (`src/app.rs`)
  - Tightly coupled UI widgets (`src/ui/`)
  - Direct ratatui rendering throughout

### Analysis Scope
- **Feature Parity**: All user-facing features, UX behaviors, and workflows
- **Widget Styling**: Visual appearance, consistency, and theme compatibility
- **Architecture**: Module structure, state management, rendering pipeline
- **GUI Readiness**: Architectural decisions that enable/hinder egui port

### Constraints
- **Read-Only Analysis**: No code modifications during SPEC execution
- **Concrete Examples**: Prefer specific behavioral comparisons over generalizations
- **Future GUI Context**: Analyze with ratatui (TUI) and egui (future GUI) in mind
- **VellumFE as Reference**: Treat VellumFE as the stable reference implementation

---

## 2. Assumptions

### Project Status Assumptions
- **ASSUMPTION 1**: Two-Face Milestone 6 ("Wire Everything Together") is in progress but not complete
- **ASSUMPTION 2**: VellumFE represents the complete, stable feature set and UX baseline
- **ASSUMPTION 3**: Two-Face intentionally refactored architecture for multi-frontend support
- **ASSUMPTION 4**: Some Two-Face differences are intentional improvements, others are regressions

### Analysis Methodology Assumptions
- **ASSUMPTION 5**: Both projects can be built and run for behavioral testing
- **ASSUMPTION 6**: Static code analysis provides sufficient architectural insight
- **ASSUMPTION 7**: Existing documentation (THEME_ARCHITECTURE.md, wiki docs) is accurate
- **ASSUMPTION 8**: Missing features in Two-Face are accidental omissions, not intentional cuts

### Deliverable Assumptions
- **ASSUMPTION 9**: Recommendations will inform follow-up SPEC-TF-STYLE, SPEC-TF-PARITY, SPEC-TF-CORE specs
- **ASSUMPTION 10**: Analysis depth should balance comprehensiveness with actionability

---

## 3. Requirements

### R1: Feature/UX Parity Analysis
**GIVEN** VellumFE as the reference implementation
**WHEN** comparing feature-by-feature against Two-Face
**THEN** produce `feature_parity.md` with:
- **Feature Matrix Table** containing:
  - **Rows**: Major features (launch flow, configuration, menus, editors, keybindings, logging, error handling, resize behavior, performance instrumentation, TTS, sound, highlights, command lists, window management, etc.)
  - **Columns**: VellumFE behavior, Two-Face behavior, Parity Status (same/intentional difference/regression/missing), Notes
- **Concrete Examples**: Specific behavioral differences (e.g., "Editor save: VellumFE prompts on quit, Two-Face exits immediately")
- **UX Impact Assessment**: User-visible consequences of each difference
- **Testing Evidence**: Results from running both apps with similar scenarios

### R2: Widget/Style Consistency Analysis
**GIVEN** inconsistent widget styling across Two-Face widgets
**WHEN** analyzing visual appearance and behavior
**THEN** produce `widget_style.md` with:
- **Widget-by-Widget Comparison**:
  - Menus (popup, context menus)
  - Editor widgets (theme editor, settings editor, highlight form, keybind form, window editor)
  - Dialogs and status messages
  - Status bars and indicators
  - Error panels and notifications
  - Focused vs unfocused states
- **For Each Widget Type**:
  - VellumFE look/behavior (colors, borders, layout)
  - Two-Face look/behavior (current implementation)
  - Suggested "Desired Standard" (unified style for theme compatibility)
- **Theme System Integration**: How current widget styles interact with AppTheme (77 fields)
- **GUI Port Implications**: Widget design decisions that help/hurt egui migration

### R3: Architecture Comparison
**GIVEN** Two-Face's refactored multi-frontend architecture
**WHEN** comparing against VellumFE's monolithic design
**THEN** produce `architecture_summary.md` with:
- **Module Layout Comparison**:
  - VellumFE: `src/app.rs`, `src/ui/`, `src/config.rs`, `src/parser.rs`, `src/network.rs`
  - Two-Face: `src/core/`, `src/data/`, `src/frontend/tui/`, `src/frontend/gui/`, `src/config.rs`, `src/parser.rs`, `src/network.rs`
- **State Management Analysis**:
  - VellumFE: Direct state mutation in `App` struct
  - Two-Face: `AppCore` + `UiState` separation
- **Input/Event Handling**:
  - VellumFE: Event loop in `app.rs` with direct widget updates
  - Two-Face: `input_router.rs` + `event_bridge.rs` abstraction
- **Rendering Pipeline**:
  - VellumFE: Direct ratatui rendering from `App::render()`
  - Two-Face: `AppCore` state → Frontend reads and renders
- **Wins and Losses**:
  - Where Two-Face is cleaner/more modular
  - Where Two-Face is still tightly coupled or messy
  - Areas that will hurt during egui port (e.g., TUI-specific assumptions in core)

### R4: Prioritized Recommendations
**GIVEN** findings from feature parity, widget style, and architecture analyses
**WHEN** identifying actionable next steps
**THEN** produce `recommendations.md` with:
- **Prioritized SPEC Candidates** (High/Medium/Low priority):
  - SPEC-TF-STYLE-XXX: Style/theme unification tasks
  - SPEC-TF-PARITY-XXX: Behavioral regression fixes
  - SPEC-TF-CORE-XXX: Core API cleanup for multi-frontend support
- **For Each Recommendation**:
  - Short rationale (why it matters)
  - Affected modules/files
  - Estimated complexity (simple/moderate/complex)
  - Dependencies (blocking/blocked-by relationships)
  - GUI port impact (critical/helpful/neutral)

---

## 4. Specifications

### S1: Analysis Methodology
**Analysis Process**:
1. **Static Code Analysis**:
   - Compare module structures using file lists and directory trees
   - Analyze widget implementations for styling patterns
   - Review `AppCore`, `UiState`, `App` structs for state management differences
2. **Dynamic Behavioral Testing** (if feasible):
   - Run VellumFE and Two-Face with identical scenarios
   - Document observable UX differences (prompts, error messages, resize behavior)
   - Test feature workflows (theme switching, editor usage, highlight management)
3. **Documentation Review**:
   - Cross-reference THEME_ARCHITECTURE.md, wiki docs, and code
   - Identify documented vs actual behavior gaps
4. **Architectural Tracing**:
   - Trace data flow from network → parser → state → rendering
   - Identify coupling points and abstraction boundaries

### S2: Deliverable Format Standards
**Feature Parity Matrix** (`feature_parity.md`):
```markdown
| Feature Area | VellumFE Behavior | Two-Face Behavior | Parity Status | Notes |
|--------------|-------------------|-------------------|---------------|-------|
| Editor Save  | Prompts on quit   | Auto-saves        | Intentional   | Two-Face improves UX |
| Resize       | Debounced 100ms   | Immediate         | Regression    | Performance issue     |
```

**Widget Style Comparison** (`widget_style.md`):
```markdown
### Popup Menu
**VellumFE**:
- Border: `Color::Cyan`
- Background: `Color::Black`
- Selected: `Color::Yellow` with `Modifier::REVERSED`

**Two-Face**:
- Border: `theme.menu_border`
- Background: `theme.menu_background`
- Selected: `theme.menu_selected_text` with `Modifier::BOLD`

**Desired Standard**:
- Use AppTheme fields consistently
- Ensure all widgets respect `theme.menu_*` fields
- Add missing theme fields for gaps (e.g., `menu_border_focused`)
```

**Architecture Summary** (`architecture_summary.md`):
```markdown
## State Management
### VellumFE
- Single `App` struct owns all state
- Direct mutation during event handling
- UI widgets read state via `&App` references

### Two-Face
- `AppCore` owns game + UI state
- `UiState` separated from `GameState`
- Frontends read immutable state snapshots

**Analysis**: Two-Face separation enables multi-frontend but introduces indirection overhead.
```

**Recommendations** (`recommendations.md`):
```markdown
### SPEC-TF-PARITY-RESIZE-001: Fix Resize Debouncing Regression
**Priority**: High
**Rationale**: Performance regression impacts UX during terminal resize
**Files**: `src/frontend/tui/mod.rs`, event loop
**Complexity**: Simple
**Dependencies**: None
**GUI Impact**: Neutral (TUI-specific)
```

### S3: Quality Criteria
- **Concrete Examples**: Every parity status must include specific behavior example
- **Actionable Recommendations**: Each recommended SPEC must be implementable
- **Traceability**: All findings traced to specific files/modules
- **GUI Port Focus**: Explicit analysis of egui migration impact

---

## 5. Constraints

### Technical Constraints
- **Read-Only**: No code modifications during analysis
- **No Time Estimates**: Avoid predicting implementation duration
- **Evidence-Based**: All claims supported by code references or test results

### Scope Constraints
- **Focus Areas**: Feature parity, widget styling, architecture, GUI readiness
- **Out of Scope**: Performance benchmarking (except observable UX impact), external dependencies, deployment

### Deliverable Constraints
- **Markdown Format**: All deliverables in Markdown
- **Location**: `.moai/specs/SPEC-TF-ANALYSIS-001/` directory
- **File Names**: `feature_parity.md`, `widget_style.md`, `architecture_summary.md`, `recommendations.md`

---

## 6. Acceptance Criteria

### AC1: Feature Parity Matrix Completeness
**GIVEN** `feature_parity.md` is delivered
**WHEN** reviewing the feature matrix
**THEN**:
- All major feature areas are covered (minimum 15 rows)
- Each row has concrete VellumFE and Two-Face behavior examples
- Parity status is one of: same, intentional difference, regression, missing
- High-impact regressions/missing features are flagged

### AC2: Widget Style Analysis Depth
**GIVEN** `widget_style.md` is delivered
**WHEN** reviewing widget comparisons
**THEN**:
- Minimum 8 widget types analyzed (menus, editors, dialogs, status bars, etc.)
- Each widget has VellumFE, Two-Face, and Desired Standard sections
- Theme field usage is documented (references to AppTheme's 77 fields)
- GUI port implications are explicitly stated

### AC3: Architecture Clarity
**GIVEN** `architecture_summary.md` is delivered
**WHEN** reviewing architectural analysis
**THEN**:
- Module layout comparison shows clear structural differences
- State management section explains data flow in both projects
- Rendering pipeline differences are traced with file references
- "Wins" and "Losses" sections provide balanced assessment
- GUI port blockers/enablers are identified

### AC4: Actionable Recommendations
**GIVEN** `recommendations.md` is delivered
**WHEN** reviewing recommended SPECs
**THEN**:
- Minimum 5 SPEC candidates with priority labels
- Each recommendation has rationale, affected files, complexity estimate
- Dependencies between recommendations are documented
- GUI port impact is assessed (critical/helpful/neutral)

### AC5: Traceability and Evidence
**GIVEN** all deliverables are complete
**WHEN** spot-checking claims and findings
**THEN**:
- Behavioral claims reference specific code files or test observations
- Architectural statements trace to module/function level
- No unsupported generalizations or assumptions

---

## 7. Risks and Mitigations

### Risk 1: Incomplete Dynamic Testing
**Risk**: Cannot run both apps for behavioral comparison
**Impact**: Feature parity analysis relies only on static code analysis
**Mitigation**: Use existing documentation, code comments, and issue history as behavioral evidence

### Risk 2: Analysis Scope Creep
**Risk**: Analysis expands beyond 4 deliverables and becomes unactionable
**Impact**: Recommendations become overwhelming, delaying actual implementation
**Mitigation**: Strictly scope to 4 deliverables; defer deep-dive topics to follow-up SPECs

### Risk 3: Subjective Styling Recommendations
**Risk**: "Desired Standard" widget styles are subjective preferences
**Impact**: Recommendations lack clear rationale
**Mitigation**: Anchor styling recommendations to AppTheme system integration and GUI port needs

---

## 8. Traceability

### References
- **VellumFE Repository**: `C:\gemstone\projects\vellumfe`
- **Two-Face Repository**: `C:\gemstone\projects\two-face`
- **Architecture Docs**: `docs/THEME_ARCHITECTURE.md`, `docs/wiki/project_overview.md`
- **Configuration**: `.claude/CLAUDE.md` (Direct eAccess authentication notes)

### Related Work
- **Future SPECs**: SPEC-TF-STYLE-XXX, SPEC-TF-PARITY-XXX, SPEC-TF-CORE-XXX (to be derived from recommendations)
- **Milestone Context**: Two-Face Milestone 6 ("Wire Everything Together")

---

**END OF SPEC**
