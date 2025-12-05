# `main.rs` → modular TUI frontend refactor plan

## Goals
- Shrink `src/main.rs` to a bootstrap/composition file only (arg/config parsing, service construction, frontend selection, run).
- Move all TUI-specific rendering, input, and UI state to `frontend::tui::*` to enforce separation of duties.
- Remove duplication (DRY) across TUI screens by centralizing styles, widgets, and keymaps.
- Preserve behavior while improving navigability and testability; keep the app runnable after each phase.

## Guiding principles
- Separation of duties: core logic is UI-agnostic; TUI owns presentation and input; `main` wires dependencies.
- DRY: shared UI elements (styles, widgets, keymaps, error display) live in one place; no repeated view glue in `main.rs`.
- Incremental safety: small, runnable moves; tests/smoke runs per phase; keep public interfaces stable while relocating code.

## Target ownership map
- `src/main.rs`: bootstrap only (config/env parsing, logging setup, construct services/context, select frontend, call `run_*`).
- `src/frontend/tui/mod.rs`: public entry `run_tui(context, settings)`.
- `src/frontend/tui/runtime.rs`: event loop, lifecycle (input/read, tick, dispatch, render).
- `src/frontend/tui/state.rs`: UI state structs/enums and screen-local state.
- `src/frontend/tui/input.rs`: keymaps, raw input → intent mapping.
- `src/frontend/tui/commands.rs`: intents/commands and dispatch to core services.
- `src/frontend/tui/view/*.rs`: rendering/layout per screen plus shared view helpers.
- `src/frontend/tui/components/*.rs`: reusable widgets (tables, lists, prompts, spinners).
- `src/frontend/tui/theme.rs`: colors, styles, layout constants.
- `src/frontend/tui/errors.rs` (or helper): UI-facing error/notification helpers.
- Shared/core: domain types, services, and errors remain UI-agnostic.

## Work phases (incremental, runnable)
1) Extract low-risk helpers
   - Move pure render helpers, style constants, and small widgets into `view/`, `components/`, `theme.rs`.
   - Why: fastest DRY wins, reduces `main.rs` noise without behavior risk.
2) Extract input/keymaps
   - Centralize keybinding tables and parsing in `input.rs`; define intent/command enums.
   - Why: isolates “what keys mean” from orchestration; simplifies future binding changes.
3) Extract UI state + event loop
   - Move TUI state structs/enums and the event loop (read → dispatch → render) into `runtime.rs` and `state.rs`.
   - Why: `main` stops owning UI lifecycle; clean separation of duties.
4) Wire bootstrap-only `main`
   - `main` builds config/services/context, then calls `frontend::tui::run_tui(...)`.
   - Why: finishes separation; `main` no longer contains TUI logic.

## Detailed steps and rationale
- Inventory responsibilities in `main.rs`
  - Action: outline sections (bootstrap, services, state, rendering, input, commands, errors).
  - Why: a map prevents accidental omissions and guides extraction order.
- Define `AppContext`/`Services` struct
  - Action: package core service handles and settings passed into TUI runtime.
  - Why: clear dependency injection point; keeps TUI modular and DRY.
- Extract rendering/layout
  - Action: move widget/layout builders and screen renderers into `view/*`.
  - Why: keeps UI composition cohesive; reduces duplication; isolates visual changes.
- Extract shared UI components
  - Action: centralize reusable widgets (tables, list selectors, prompts, spinners) into `components/*`; add `theme.rs` for styles.
  - Why: DRY across screens; single place to tweak look/feel.
- Extract input handling
  - Action: move keymaps, parsing, and input → intent mapping into `input.rs`.
  - Why: separates gesture interpretation; easier rebinding and testing.
- Define command/intent layer
  - Action: create `commands.rs` to translate intents into calls to core services.
  - Why: decouples UI gestures from business logic; keeps `main` free of UI behavior.
- Extract TUI state and runtime
  - Action: move UI state structs/enums and the main loop into `state.rs` and `runtime.rs`.
  - Why: isolates lifecycle control; enables targeted tests of the loop without `main`.
- Error display helpers
  - Action: move UI-facing error/notification formatting into TUI module; keep error types in core.
  - Why: core stays UI-agnostic; DRY display patterns across screens.
- Config and feature flags
  - Action: `main` parses CLI/config once, passes structured settings into `run_tui`; TUI owns its toggles.
  - Why: `main` remains pure bootstrap; TUI behavior is configured at its boundary.

## DRY checkpoints
- Styles/colors/layout constants live in `theme.rs`.
- Keybinding tables unified in `input.rs`.
- Shared widgets centralized in `components/*`.
- Single command/intent mapping in `commands.rs`.
- No duplicate render helpers inside `main.rs`.

## Testing & verification
- Unit tests for input → intent mapping and command dispatch (fast feedback).
- Optional snapshot/render tests for key views if feasible.
- Smoke test via `cargo run` per phase to ensure behavior parity.

## Definition of done
- `src/main.rs` ~< 300 lines; contains only bootstrap and frontend selection.
- No TUI rendering/input/state logic in `main.rs`; all lives under `src/frontend/tui/`.
- TUI shares styles/widgets/keymaps via centralized modules (DRY).
- App runs with feature parity; tests updated/added around new boundaries.
