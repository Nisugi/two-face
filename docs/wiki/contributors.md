# Contributor & Developer Notes

Thinking about hacking on Two-Face? Here’s a guide to the moving parts, conventions, and supporting documents.

## Repository Structure

- **Core logic**: `src/core/` (AppCore, input routing, menu actions, message types).
- **Data models**: `src/data/` (window definitions, widget state, UI state snapshots).
- **Frontends**: `src/frontend/tui/` (ratatui widgets) and `src/frontend/gui/` (future egui widgets).
- **Config & Themes**: `src/config.rs`, `src/theme.rs`, `src/sound.rs`, `src/selection.rs`.
- **Design Docs**:
  - `EGUI_IMPLEMENTATION_PLAN.md` – road map for the GUI.
  - `LAYOUT_SYSTEM_DOCUMENTATION.md` – DSL and layout engine rules.
  - `MENU_INPUT_TESTING_CHECKLIST.md` – manual QA steps for menu/keybind interactions.

## Coding Guidelines

- Prefer pure data structs in `data/`; keep rendering logic in frontends.
- When adding state, update both `AppCore` and the serialization code (if it needs to persist).
- Favor `tracing` for logs—no `println!` in production paths.
- Keep widget modules small and focused. A new widget usually means:
  1. Adding state to `data::widget`.
  2. Rendering code under `frontend/tui`.
  3. Optional config fields in `config::WindowDef`.

## Testing

- Unit tests exist for command substitution (`cmdlist.rs`), parser snippets, etc. Run `cargo test`.
- Manual QA is documented in `MENU_INPUT_TESTING_CHECKLIST.md`.
- Consider adding targeted tests when editing parser/event logic; XML parsing bugs are notoriously hard to track down.

## Adding Features

1. **Define config** in `config.rs` (with defaults and serialization).
2. **Update data models** (`data/`) so frontends get new fields.
3. **Teach AppCore** how to react (handle parser output, modify state, route events).
4. **Render** in each frontend (start with TUI, then port to GUI).
5. **Document** the change—update this wiki and any relevant design doc.

## Pull Requests / Patches

- Keep commits scoped and reference modules/files you touched.
- Run `cargo fmt` and `cargo clippy` before submitting.
- Include screenshots (TUI or GUI) when tweaking layout-sensitive widgets.

## Release Checklist

1. Bump versions in `Cargo.toml` (when public releases resume).
2. Update `README.md` and wiki pages with major changes.
3. Regenerate the defaults if you changed any files in `defaults/`.
4. Smoke-test both frontends (TUI mandatory, GUI once available).

## Community Contributions

- New themes, layouts, highlight sets, and layouts are welcome—drop them into the repo or host them in your fork.
- Document noteworthy configurations so others can learn from them.

Two-Face stays healthy when both players and developers share knowledge. Use this wiki as the living handbook, and add to it whenever you extend the client!
