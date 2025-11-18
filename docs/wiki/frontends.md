# Frontends

Two-Face ships with a full-featured TUI frontend today and reserves space for a GUI built with egui. Both consume the same `AppCore`, so switching frontends does not change your configuration, highlights, or layouts.

## Choosing a Frontend

```
two-face --frontend tui      # default
two-face --frontend gui      # future egui build
```

If you omit `--frontend`, the CLI defaults to `tui`. When the GUI lands it will live under `src/frontend/gui`, but the CLI flag already exists so documentation doesn’t have to change later.

---

## TUI (ratatui) Frontend

Located in `src/frontend/tui`, the terminal UI mimics VellumFE’s look and feel while adding quality-of-life tools.

### Layout & Rendering

- **Window Definitions** (`config::WindowDef`): each layout entry defines location, size, borders, colors, and stream bindings.
- **Widget Modules**: Every on-screen component has a dedicated module (`text_window.rs`, `room_window.rs`, `inventory_window.rs`, `dashboard.rs`, etc.). See the [Widgets Reference](widgets.md) for a full tour.
- **Scrolling & Selection**: Long-running windows support scrollback, selection (`selection.rs`), and optional search bars.
- **Drag Handles**: Popup editors (window editor, highlight/keybind forms) can be dragged by their title bars; see `window_editor.rs` for implementation details.

### Input & Navigation

- **Keyboard**: `frontend/events.rs` normalizes crossterm key/mouse events which the core translates via `core/menu_actions`.
- **Mouse** (optional): Click-to-focus, context menus, and selection behave just like ProfanityFE. Events are fed into `SelectionState` so text windows know which lines to highlight.
- **Menu System**: Menus are just another widget (`popup_menu.rs`) with arrow/tab navigation and command dispatch.

### Performance Overlay

`frontend/tui/performance_stats.rs` renders the metrics from `performance.rs` so you can verify FPS, parse speed, network throughput, allocation estimates, etc. Toggle it via the default hotkey (see [Input & Menus](input_and_menus.md)).

### Theme Awareness

Ratatui widgets read colors from `theme.rs` + the selected layout definitions. Switching themes updates borders, backgrounds, hover states, and text colors without reloading the client.

---

## GUI (egui) Frontend (Planned)

The GUI frontend is scoped out in `EGUI_IMPLEMENTATION_PLAN.md` and will live under `src/frontend/gui`. Goals:

- Native window with proportional fonts and resizable panes.
- Shared widget logic where possible (reuse `data::window` state, highlight/command editors).
- System clipboard integration and OS-native file pickers.

Until the GUI is complete the `gui` module contains placeholders, but the rest of the wiki already documents cross-cutting features so the new frontend slides into the same workflow.

---

## Frontend Switching Tips

- Layout files can be shared, but the GUI may eventually support additional options (e.g., proportional font metrics). Any GUI-only settings will live next to the TUI ones under `Config::ui`.
- The CLI flag is read once at startup; switch by restarting with a different `--frontend` value.
- Because themes are frontend-agnostic, designers can create one set of `.toml` files that look good in both environments.
