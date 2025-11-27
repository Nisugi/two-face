# Input, Keybinds & Menus

Two-Face’s interaction model balances familiar VellumFE shortcuts with modern niceties. This page documents keyboard/mouse handling, the command line, menus, and how to edit keybinds.

## Event Flow

1. **Frontend** captures events via crossterm (`frontend/events.rs`) and turns them into `FrontendEvent`.
2. **Core Input Router** (`core/input_router.rs`) uses the active mode (`data::ui_state::InputMode`) to decide whether the event goes to the command line, a popup, or a global handler.
3. **Menu Actions** (`core/menu_actions.rs`) map `KeyEvent`s to semantic actions (NavigateUp, Save, Toggle, etc.) so all widgets behave consistently.

## Command Input Widget

- **Insert & Navigate**: Supports UTF-8, Home/End, word jumps (Ctrl+Left/Right), delete/backspace, selection, cut/copy/paste (`Ctrl+C/X/V`).
- **History**: Up/Down arrow cycles recent commands; persistent history is stored per-character under `.two-face/<char>/history.txt`.
- **Autocomplete**:
  - Dot-commands: type `.s` and press `Tab` to cycle known commands, driven by `available_commands` from AppCore.
  - Window/template names: type part of a name when editing layout commands (`menu:window`) and cycle completions.
- **Selection**: Shift+Arrow begins a selection; `Ctrl+A` selects all; `Ctrl+C` copies into the clipboard helper (`clipboard.rs`).

## Keybinds

- **File Format** (`keybinds.toml`):
  ```toml
  "ctrl+k" = { Action = "send_command" }
  "alt+1"  = { Macro = { macro_text = ".loot\r" } }
  ```
- **Actions** map directly to AppCore commands (scrolling windows, toggling overlays, switching focus).
- **Macros** send literal text (including `\r` for Enter). Use the UI form for escaping convenience.

### Editing Keybinds In-App

1. Open **Keybind Browser** (`menu:keybinds`).
2. Navigate to an entry, press `Enter` to edit, or `Del` to remove.
3. The **Keybind Form** (`keybind_form.rs`) lets you pick “Action” vs “Macro,” assign the combo, and validate duplicates.
4. Save to `keybinds.toml`. Use “Save Profile” to snapshot the current set into `profiles/keybinds/`.

### Menu Keybinds

Separate from gameplay keybinds, menu navigation shortcuts live in `menu_keybinds.toml`. The Settings menu exposes them; typical combinations:

- `Tab` / `Shift+Tab`: cycle form fields.
- `Enter`: select/confirm.
- `Esc`: cancel/close.
- `Ctrl+s`: save in most forms.

## Menus & Browsers

- **PopupMenu** renders context menus for windows, layout tools, and profile pickers.
- Navigation is uniform: `Arrow Keys` move selection, `Enter` executes, `Esc` closes. Some menus also listen for accelerators (press the highlighted letter).
- Browser popups (highlights, colors, spells, themes, settings) show help text at the bottom listing the active shortcuts.

## Mouse Support

- **Selection**: Click and drag inside a text window to highlight; release to copy (if clipboard integration is enabled).
- **Menus**: Left click to select entries; the widget handles hit-testing.
- **Popups**: Drag by clicking the border or header when the popup supports it (most configuration dialogs do).

## Search & Focus

- `Ctrl+F` (default) toggles search mode in the active text window.
- `Enter` commits the regex; `n` / `Shift+n` move between matches.
- Focus changes obey layout z-order; use `menu:windows` to switch explicitly.

## Input Modes

Two-Face tracks whether you are:

1. **In Command Mode** – global hotkeys active (e.g., switch windows, toggle overlays).
2. **Typing in CLI** – text editing shortcuts active.
3. **Inside a Popup/Form** – only form navigation actions active; game commands paused until you close the popup.

This separation prevents accidental commands while configuring highlights or layouts.
