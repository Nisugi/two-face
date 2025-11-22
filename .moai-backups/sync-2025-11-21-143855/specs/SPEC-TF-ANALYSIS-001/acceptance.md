# SPEC-TF-ANALYSIS-001: Acceptance Criteria

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

This document defines the detailed acceptance criteria for SPEC-TF-ANALYSIS-001. All criteria must be satisfied for the analysis to be considered complete and ready for follow-up implementation SPECs.

---

## AC1: Feature Parity Matrix Completeness

### Acceptance Criteria
**GIVEN** `feature_parity.md` is delivered
**WHEN** reviewing the feature matrix for completeness
**THEN** the following conditions are met:

### Test Scenarios

#### Scenario 1.1: Feature Coverage Breadth
**GIVEN** the feature parity matrix table
**WHEN** counting the number of feature area rows
**THEN**:
- ✅ Minimum 15 feature areas are analyzed
- ✅ Feature areas include:
  - Launch flow (CLI args, connection, authentication)
  - Configuration system (loading, saving, validation)
  - Menu system (popup menus, context menus)
  - Editor widgets (theme, settings, highlight, keybind, window)
  - Logging and error handling
  - Terminal resize behavior
  - Performance instrumentation
  - Accessibility (TTS, sound)
  - Highlight and automation (triggers, command lists, macros)
  - Window management (layout, editing, stream routing)
  - Additional domain-specific features (inventory, compass, injury doll, etc.)

#### Scenario 1.2: Behavioral Specificity
**GIVEN** each feature area row in the matrix
**WHEN** reviewing the VellumFE Behavior and Two-Face Behavior columns
**THEN**:
- ✅ VellumFE behavior is described with concrete examples (not vague statements)
- ✅ Two-Face behavior is described with concrete examples
- ✅ Differences are observable and testable (e.g., "VellumFE prompts on quit with 'Save changes? (y/n)', Two-Face auto-saves on quit without prompt")
- ❌ No rows contain vague statements like "VellumFE has better error handling"

#### Scenario 1.3: Parity Status Accuracy
**GIVEN** each feature area row in the matrix
**WHEN** checking the Parity Status column
**THEN**:
- ✅ Status is one of: **Same**, **Intentional Difference**, **Regression**, **Missing**
- ✅ **Same**: VellumFE and Two-Face have identical behavior
- ✅ **Intentional Difference**: Two-Face deliberately changed for improvement (rationale in Notes)
- ✅ **Regression**: Two-Face lost VellumFE functionality (flagged as high-impact if user-facing)
- ✅ **Missing**: Two-Face hasn't implemented yet (flagged if blocking milestone)

#### Scenario 1.4: High-Impact Flagging
**GIVEN** all feature areas marked as Regression or Missing
**WHEN** reviewing the Notes column
**THEN**:
- ✅ High-impact regressions are explicitly flagged (e.g., "⚠️ HIGH IMPACT: Blocks UX parity")
- ✅ Critical missing features are flagged (e.g., "🚨 CRITICAL: Required for Milestone 6 completion")
- ✅ Notes explain user-visible impact (e.g., "Users will experience lag during resize")

#### Scenario 1.5: Evidence and Traceability
**GIVEN** each behavioral claim in the matrix
**WHEN** spot-checking for evidence
**THEN**:
- ✅ Static analysis claims reference specific files (e.g., "VellumFE: src/app.rs:123, Two-Face: src/core/input_router.rs:456")
- ✅ Dynamic testing claims describe observable behavior (e.g., "Tested: VellumFE shows prompt, Two-Face does not")
- ✅ No unsupported generalizations

---

## AC2: Widget Style Analysis Depth

### Acceptance Criteria
**GIVEN** `widget_style.md` is delivered
**WHEN** reviewing widget comparisons for depth and completeness
**THEN** the following conditions are met:

### Test Scenarios

#### Scenario 2.1: Widget Type Coverage
**GIVEN** the list of analyzed widget types
**WHEN** counting the number of widget sections
**THEN**:
- ✅ Minimum 8 widget types are analyzed
- ✅ Widget types include:
  - Popup menus (`popup_menu.rs`)
  - Context menus (right-click menus)
  - Editor widgets (theme editor, settings editor, highlight form, keybind form, window editor)
  - Dialogs and prompts
  - Status bars and indicators
  - Error panels and notifications
  - Focus states (focused vs unfocused borders, titles, colors)

#### Scenario 2.2: VellumFE Style Documentation
**GIVEN** each widget type section
**WHEN** reviewing the VellumFE subsection
**THEN**:
- ✅ Border colors are documented (e.g., "Border: `Color::Cyan`")
- ✅ Background colors are documented (e.g., "Background: `Color::Black`")
- ✅ Text colors and modifiers are documented (e.g., "Selected: `Color::Yellow` with `Modifier::REVERSED`")
- ✅ Focus vs unfocus states are differentiated (if applicable)
- ✅ File references are provided (e.g., "VellumFE: src/ui/popup_menu.rs:234")

#### Scenario 2.3: Two-Face Style Documentation
**GIVEN** each widget type section
**WHEN** reviewing the Two-Face subsection
**THEN**:
- ✅ AppTheme field usage is documented (e.g., "Border: `theme.menu_border`")
- ✅ Hardcoded colors (if any) are flagged (e.g., "⚠️ Hardcoded: `Color::Red` instead of `theme.error_color`")
- ✅ Inconsistencies with other widgets are noted (e.g., "Uses `theme.window_border` instead of `theme.menu_border` like popup_menu.rs")
- ✅ File references are provided (e.g., "Two-Face: src/frontend/tui/theme_editor.rs:567")

#### Scenario 2.4: Desired Standard Recommendations
**GIVEN** each widget type section
**WHEN** reviewing the Desired Standard subsection
**THEN**:
- ✅ Proposed unified styling leverages AppTheme fields
- ✅ Missing AppTheme fields are identified (e.g., "Add `menu_border_focused` field to AppTheme")
- ✅ GUI port implications are stated (e.g., "Avoid TUI-specific `Modifier::REVERSED`; use semantic colors for GUI compatibility")
- ✅ Backwards compatibility considerations are noted (if breaking VellumFE user expectations)

#### Scenario 2.5: Theme System Integration Assessment
**GIVEN** the Theme System Integration section in `widget_style.md`
**WHEN** reviewing the assessment
**THEN**:
- ✅ AppTheme coverage analysis shows which of 77 fields are used by widgets
- ✅ Gaps in AppTheme are identified (e.g., "No theme field for dialog border color")
- ✅ Inconsistencies in theme field usage are documented (e.g., "Popup menu uses `menu_border`, but context menu uses `window_border`")
- ✅ Recommendations for AppTheme extension or refactoring are provided

#### Scenario 2.6: GUI Port Implications
**GIVEN** each widget's Desired Standard subsection
**WHEN** checking for GUI port analysis
**THEN**:
- ✅ TUI-specific assumptions are flagged (e.g., "TUI uses `Modifier::BOLD`; egui needs bold font variant")
- ✅ GUI-friendly recommendations are provided (e.g., "Use semantic colors instead of terminal-specific modifiers")
- ✅ Shared styling patterns for TUI and GUI are suggested (e.g., "Define border thickness as AppTheme field for both ratatui and egui")

---

## AC3: Architecture Clarity

### Acceptance Criteria
**GIVEN** `architecture_summary.md` is delivered
**WHEN** reviewing architectural analysis for clarity and depth
**THEN** the following conditions are met:

### Test Scenarios

#### Scenario 3.1: Module Layout Comparison
**GIVEN** the Module Layout Comparison section
**WHEN** reviewing the side-by-side comparison
**THEN**:
- ✅ VellumFE directory structure is documented (e.g., `src/app.rs`, `src/ui/`, `src/config.rs`)
- ✅ Two-Face directory structure is documented (e.g., `src/core/`, `src/data/`, `src/frontend/tui/`, `src/frontend/gui/`)
- ✅ Structural differences are highlighted (e.g., "VellumFE: Monolithic `app.rs`; Two-Face: Layered `core/`, `data/`, `frontend/`")
- ✅ Shared modules are noted (e.g., "Both have `config.rs`, `parser.rs`, `network.rs`")

#### Scenario 3.2: State Management Analysis
**GIVEN** the State Management section
**WHEN** reviewing VellumFE and Two-Face state flow
**THEN**:
- ✅ VellumFE state management is documented (e.g., "Single `App` struct owns all state, direct mutation during event handling")
- ✅ Two-Face state management is documented (e.g., "`AppCore` owns game + UI state, `UiState` separated from `GameState`")
- ✅ Data flow is traced (e.g., "Event → Mutation → Render" for VellumFE; "Event → Router → Action → State Update → Frontend Read" for Two-Face)
- ✅ Coupling points are identified (e.g., "VellumFE: UI widgets read `&App` directly; Two-Face: Frontends read immutable state snapshots")

#### Scenario 3.3: Input/Event Handling Analysis
**GIVEN** the Input/Event Handling section
**WHEN** reviewing input flow comparison
**THEN**:
- ✅ VellumFE input handling is documented (e.g., "Event loop in `app.rs`, direct widget input handlers")
- ✅ Two-Face input handling is documented (e.g., "`input_router.rs`, `event_bridge.rs` abstraction layer")
- ✅ Input flow is traced (e.g., "Keyboard event → `match` in `app.rs` → Direct state mutation" for VellumFE; "Keyboard event → `input_router` → `InputAction` → `AppCore::handle_action()`" for Two-Face)
- ✅ Abstraction benefits and overhead costs are assessed

#### Scenario 3.4: Rendering Pipeline Analysis
**GIVEN** the Rendering Pipeline section
**WHEN** reviewing rendering comparison
**THEN**:
- ✅ VellumFE rendering is documented (e.g., "`App::render()` directly uses ratatui to draw widgets")
- ✅ Two-Face rendering is documented (e.g., "`AppCore` prepares state → Frontend reads state → Renders with ratatui")
- ✅ Rendering flow is traced with file references
- ✅ Decoupling success and performance implications are assessed (e.g., "Two-Face decoupling enables multi-frontend but adds indirection overhead")
- ✅ TUI-specific assumptions that leak into core are identified

#### Scenario 3.5: Wins and Losses Assessment
**GIVEN** the Wins and Losses sections
**WHEN** reviewing the balanced assessment
**THEN**:
- ✅ **Wins**: Specific examples of where Two-Face is cleaner/more modular (e.g., "AppCore separates game state from UI state, enabling testability")
- ✅ **Losses**: Specific examples of where Two-Face is messier/more coupled (e.g., "TUI-specific `Rect` types in `UiState` block GUI port")
- ✅ **GUI Port Blockers**: TUI assumptions in core that will hurt egui migration (e.g., "Hardcoded ratatui `Rect` in `data/window.rs`")
- ✅ **GUI Port Enablers**: Abstraction points that make egui port easier (e.g., "`input_router` can dispatch to TUI or GUI frontend")

#### Scenario 3.6: Traceability
**GIVEN** all architectural claims
**WHEN** spot-checking for evidence
**THEN**:
- ✅ All claims reference specific files and modules
- ✅ Data flow traces include function names or line number ranges
- ✅ No vague statements like "Two-Face is better architected"

---

## AC4: Actionable Recommendations

### Acceptance Criteria
**GIVEN** `recommendations.md` is delivered
**WHEN** reviewing recommended SPECs for actionability
**THEN** the following conditions are met:

### Test Scenarios

#### Scenario 4.1: Recommendation Count and Prioritization
**GIVEN** the list of recommended SPECs
**WHEN** counting and categorizing recommendations
**THEN**:
- ✅ Minimum 5 SPEC candidates are documented
- ✅ Each SPEC has a priority label: **High**, **Medium**, or **Low**
- ✅ High priority SPECs address:
  - Critical regressions (blocking feature parity)
  - GUI port blockers (blocking egui migration)
- ✅ Medium priority SPECs address:
  - Style inconsistencies
  - Moderate feature gaps
- ✅ Low priority SPECs address:
  - Nice-to-have improvements
  - Long-term refactors

#### Scenario 4.2: SPEC Candidate Structure
**GIVEN** each recommended SPEC
**WHEN** reviewing the documentation
**THEN**:
- ✅ **Rationale**: Short explanation of why this SPEC matters (1-3 sentences)
- ✅ **Affected Files/Modules**: List of files or modules that will change
- ✅ **Complexity Estimate**: Labeled as **Simple**, **Moderate**, or **Complex**
- ✅ **Dependencies**: Blocking or blocked-by relationships documented (e.g., "Blocked by: SPEC-TF-CORE-STATE-001")
- ✅ **GUI Port Impact**: Labeled as **Critical**, **Helpful**, or **Neutral**

#### Scenario 4.3: Rationale Quality
**GIVEN** each SPEC's rationale
**WHEN** evaluating the justification
**THEN**:
- ✅ Rationale is specific and evidence-based (e.g., "Fixes resize debouncing regression identified in feature_parity.md Section 2.3")
- ✅ Rationale explains user or developer impact (e.g., "Users experience lag during resize; developers cannot port to GUI without core refactor")
- ❌ No vague rationales like "Improves code quality"

#### Scenario 4.4: Affected Files Specificity
**GIVEN** each SPEC's affected files list
**WHEN** checking for specificity
**THEN**:
- ✅ Files are listed with paths (e.g., `src/frontend/tui/mod.rs`, `src/core/input_router.rs`)
- ✅ Modules are described with scope (e.g., "Event loop in `tui/mod.rs`, resize debouncer")
- ❌ No vague statements like "Various files" or "Core modules"

#### Scenario 4.5: Complexity Estimation Consistency
**GIVEN** all SPEC complexity estimates
**WHEN** reviewing for consistency
**THEN**:
- ✅ **Simple**: Single-file changes, no architectural impact, < 100 lines of code
- ✅ **Moderate**: Multi-file changes, localized refactor, 100-500 lines of code
- ✅ **Complex**: Cross-module refactor, architectural changes, > 500 lines of code

#### Scenario 4.6: Dependency Documentation
**GIVEN** all SPEC dependencies
**WHEN** reviewing blocking relationships
**THEN**:
- ✅ Dependencies are explicit (e.g., "SPEC-TF-CORE-STATE-001 must complete before SPEC-TF-GUI-RENDERER-001")
- ✅ Circular dependencies are avoided
- ✅ Parallelizable work is identified (e.g., "SPEC-TF-STYLE-MENU-001 and SPEC-TF-STYLE-EDITOR-001 can run in parallel")

#### Scenario 4.7: GUI Port Impact Assessment
**GIVEN** each SPEC's GUI port impact label
**WHEN** verifying the categorization
**THEN**:
- ✅ **Critical**: Blocking egui port (e.g., "Removes TUI-specific `Rect` from `UiState`")
- ✅ **Helpful**: Improves GUI compatibility (e.g., "Unifies widget styling for easier egui mapping")
- ✅ **Neutral**: No impact on GUI port (e.g., "TUI-only performance optimization")

---

## AC5: Traceability and Evidence

### Acceptance Criteria
**GIVEN** all deliverables are complete
**WHEN** spot-checking claims and findings for evidence
**THEN** the following conditions are met:

### Test Scenarios

#### Scenario 5.1: Code Reference Accuracy
**GIVEN** a random sample of 10 code references across all deliverables
**WHEN** verifying file paths and line numbers
**THEN**:
- ✅ File paths exist in the respective repositories
- ✅ Line numbers (if provided) point to relevant code
- ✅ Code snippets (if provided) match actual file contents

#### Scenario 5.2: Behavioral Claims Evidence
**GIVEN** a random sample of 5 behavioral claims from `feature_parity.md`
**WHEN** checking for supporting evidence
**THEN**:
- ✅ Claims are supported by:
  - Static code analysis (file references), OR
  - Dynamic testing observations (described behavior), OR
  - Documentation cross-references (wiki, README, THEME_ARCHITECTURE.md)
- ❌ No unsupported claims like "VellumFE is faster" without evidence

#### Scenario 5.3: Architectural Claims Traceability
**GIVEN** a random sample of 5 architectural statements from `architecture_summary.md`
**WHEN** tracing to source code
**THEN**:
- ✅ Statements reference specific modules, structs, or functions
- ✅ Data flow traces include function call chains or state update sequences
- ✅ No vague statements without file/module references

#### Scenario 5.4: Widget Style Claims Evidence
**GIVEN** a random sample of 5 widget style claims from `widget_style.md`
**WHEN** verifying color and styling details
**THEN**:
- ✅ Color values match source code (e.g., "`Color::Cyan`" found in `src/ui/popup_menu.rs`)
- ✅ AppTheme field usage matches code (e.g., "`theme.menu_border`" found in `src/frontend/tui/popup_menu.rs`)
- ✅ Hardcoded colors are accurately identified and flagged

#### Scenario 5.5: Recommendation Traceability
**GIVEN** a random sample of 3 SPEC recommendations from `recommendations.md`
**WHEN** tracing rationale to findings
**THEN**:
- ✅ Rationale references specific findings from `feature_parity.md`, `widget_style.md`, or `architecture_summary.md`
- ✅ Affected files match modules discussed in earlier deliverables
- ✅ Dependencies align with architectural constraints identified in `architecture_summary.md`

---

## Definition of Done

### Deliverable Completeness
- ✅ `feature_parity.md` delivered with minimum 15 feature areas
- ✅ `widget_style.md` delivered with minimum 8 widget types
- ✅ `architecture_summary.md` delivered with module layout, state management, input handling, rendering pipeline sections
- ✅ `recommendations.md` delivered with minimum 5 prioritized SPEC candidates

### Quality Gates
- ✅ All acceptance criteria (AC1-AC5) are satisfied
- ✅ All test scenarios pass verification
- ✅ No unsupported claims or vague statements
- ✅ All file references are accurate and traceable

### Review and Approval
- ✅ User reviews all deliverables and confirms alignment with intent
- ✅ Critical findings (high-impact regressions, GUI port blockers) are acknowledged
- ✅ Recommended SPECs are prioritized for implementation roadmap

---

**END OF ACCEPTANCE CRITERIA**
