# Windows & Widgets Reference

Use this page as a catalog of everything you can place in a layout. Each widget originates from `src/frontend/tui/*` and is controlled by data in `config::WindowDef` plus the layout editor popups.

## Text Windows

- **Module**: `frontend/tui/text_window.rs`
- **Streams**: `main`, `thoughts`, `inv`, `logons`, etc.
- **Features**:
  - Unlimited scrollback (bounded by `ui.buffer_size`).
  - Timestamp injection, search (regex), highlight layering, and clickable links.
  - Selection + copy via mouse drag or keyboard selection shortcuts.
  - Text alignment/centering when content is shorter than the viewport.
- **Configuration**: Border style (single/double/rounded), background color, show timestamps, linked streams.

## Room Window

- **Module**: `frontend/tui/room_window.rs`
- **Purpose**: Shows the current room description, objects, players, and exits in separate “components.”
- **Features**:
  - Toggle component visibility (desc/objs/players/exits).
  - Preserves per-component buffers and wraps text based on window width.
  - Clickable link data for room entities (uses the same link cache as text windows).

## Inventory & Spells

- **Modules**: `inventory_window.rs`, `spells_window.rs`
- **Purpose**: Display inventory and known spells using non-scrollable (per update) buffers.
- **Features**:
  - Automatic deduplication/aggregation of `<a>` links so long item names are clickable.
  - Optional borders/colors inherited from the parent window definition.

## Hands & Dashboard

- **Modules**: `hand.rs`, `dashboard.rs`
- **Purpose**: Show left/right/spell hands, stances, or other status icons.
- **Features**:
  - Configurable icons (text) plus truncated content strings.
  - Dashboard supports horizontal, vertical, or grid layouts with spacing and hide-when-inactive options.

## Countdown & Progress Bars

- **Modules**: `countdown.rs`, `progress_bar.rs`
- **Purpose**: Visualize RT/CT timers, vitals, and other numeric data.
- **Features**:
  - Countdown uses block glyphs plus numeric display; respects server vs local time offset.
  - Progress bars support custom fill/background colors, transparent backgrounds, and optional text overlays.

## Indicators & Status Widgets

- **Modules**: `indicator.rs`, `active_effects.rs`, `targets.rs`, `players.rs`, `injury_doll.rs`, `compass.rs`, `command_input.rs`
- **Highlights**:
  - **Indicator**: simple “on/off” states for prone/kneel/etc.
  - **Active Effects**: scrollable list with durations formatted `[HH:MM]` or `[MM:SS]`.
  - **Players/Targets**: parse comma-delimited lines from the parser and display them with stance prefixes.
  - **Injury Doll**: matches Profanity’s ASCII art with colorized body parts.
  - **Compass**: 4x3 layout with up/down/out and diagonals.
  - **Command Input**: includes history, selection, cut/copy, autocomplete, and per-character history storage.

## Popups & Forms

- **Modules**: `popup_menu.rs`, `highlight_form.rs`, `highlight_browser.rs`, `keybind_form.rs`, `keybind_browser.rs`, `color_palette_browser.rs`, `color_form.rs`, `spell_color_browser.rs`, `spell_color_form.rs`, `theme_browser.rs`, `theme_editor.rs`, `uicolors_browser.rs`, `settings_editor.rs`, `window_editor.rs`
- **Features**:
  - All popups support dragging via the border/title bar and obey the shared widget trait set (`Navigable`, `Selectable`, `TextEditable`, etc.).
  - Menus show keyboard hints and dispatch commands through `core/menu_actions`.
  - Editors (highlights, colors, spells, themes) integrate with profiles and validation logic to catch typos before saving.

## Spacer & Layout Helpers

- **Module**: `spacer.rs`
- **Purpose**: Reserve space or create background bands inside a layout. No borders, no focus—just fill the area with a color if desired.

## Performance Stats

- **Module**: `performance_stats.rs` (under `frontend/tui`) with backing data in `performance.rs`.
- **Metrics**:
  - FPS, average frame/render/UI/text-wrap times.
  - Network IO (bytes/sec), parser chunk counts, XML elements/sec.
  - Event processing time and queue depth.
  - Estimated memory usage (based on buffered lines/windows).
- **Usage**: Toggle the widget (`menu:performance`) and move/resize via your layout file.

## Adding Widgets to a Layout

1. Open the **Layouts** menu (`menu:layouts`).
2. Use **Add Window** to define a new window or **Edit Window** to change an existing entry.
3. Set the `widget_type` (`text`, `room`, `countdown`, etc.) and configure borders, colors, streams.
4. Save; the layout file under `.two-face/layouts/` updates immediately.

Refer to `LAYOUT_SYSTEM_DOCUMENTATION.md` for deeper notes on the layout DSL, padding rules, and responsive mappings.
