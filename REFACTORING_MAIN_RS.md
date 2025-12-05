# Refactoring Plan for `main.rs`

This document outlines a plan of action to refactor `src/main.rs`. The guiding principles are **Separation of Concerns (SoC)**, making each part of the code responsible for one thing, and **Don't Repeat Yourself (DRY)**, avoiding duplicated logic.

The end goal is to make `main.rs` a thin orchestrator that simply initializes the application and delegates control to the appropriate frontend. All logic specific to the TUI (Terminal User Interface) or the planned GUI (Graphical User Interface) will be moved into their respective modules.

---

### **Phase 1: Relocate Frontend Runners**

*   **What:** Move the `run_tui` and `async_run_tui` functions from `main.rs` into the `src/frontend/tui/mod.rs` file. They will become part of the TUI module's public API, likely exposed via a single function like `pub fn run(app_core: AppCore)`. Similarly, the placeholder `run_gui` function should be moved to a new `src/frontend/gui/mod.rs` module.
*   **Why:** The logic for setting up and running a specific frontend (like initializing `ratatui`, entering the alternate screen, and managing the event loop) is an implementation detail of that frontend. `main.rs` should not be concerned with *how* a frontend runs, only *which* one to launch.
*   **Alignment:**
    *   **SoC:** This gives the TUI and GUI modules complete ownership of their own lifecycle. `main.rs` is responsible for *application startup*, while the frontend modules are responsible for *UI presentation and management*.

### **Phase 2: Deconstruct and Relocate Input Handling**

*   **What:** Move the entire `handle_frontend_event` function from `main.rs` into the `src/frontend/tui` module. This function, which is the heart of the TUI's interactivity, would likely become a private method on the `TuiFrontend` struct.
*   **Why:** This function processes raw `crossterm` events, which are specific to the terminal. It manages UI state like modal dialogs and input modes (normal vs. command). This is purely presentational logic and has no place in the main application binary. The core application logic should only receive high-level commands, not raw key presses.
*   **Alignment:**
    *   **SoC:** This is the most critical step. It cleanly separates the raw input processing (a View concern) from the application's business logic (a Model/ViewModel concern, i.e., `AppCore`). `TuiFrontend` becomes solely responsible for translating key presses into meaningful actions.

### **Phase 3: Extract Modal Dialog Logic**

*   **What:** The logic within the (relocated) `handle_frontend_event` function for managing different modal dialogs (Help, Keybinds, Layouts, etc.) should be extracted into a dedicated submodule, such as `src/frontend/tui/modals.rs`. A `ModalManager` struct or similar could be created within the TUI module to manage the state of the active modal.
*   **Why:** The `handle_frontend_event` function is large and complex. Breaking it down further by feature (like modal dialogs) improves readability and maintainability. Managing modal state is a classic UI responsibility.
*   **Alignment:**
    *   **SoC:** It further refines the TUI module's internal structure. The main TUI component delegates the specific concern of *modal presentation* to a specialized sub-component.
    *   **DRY:** If multiple modals share behavior (e.g., how they handle 'escape' or 'enter' keys), a base `Modal` trait or struct could be created to prevent duplicating that logic for each new modal.

### **Phase 4: Simplify `main.rs` to a Thin Orchestrator**

*   **What:** After the previous phases, `main.rs` should be reduced to its essential responsibilities:
    1.  Parsing command-line arguments to determine which frontend to run (e.g., `--tui` or `--gui`).
    2.  Loading the initial configuration.
    3.  Creating the central `AppCore` instance.
    4.  Calling `tui::run(app_core)` or `gui::run(app_core)` and passing ownership of the core state.
*   **Why:** This makes the application's entry point extremely simple and stable. It will rarely need to be modified, even as the frontends undergo significant changes. It provides a clear and concise "table of contents" for how the application starts.
*   **Alignment:**
    *   **SoC:** This achieves the ultimate goal. `main.rs` has a single responsibility: **launch the application**.
