# Customization Suite

Two-Face was built for tinkering. Themes, layouts, highlights, colors, sounds, and settings can all be edited in-app or by hand. Use this page as your playbook.

## Layouts & Window Editor

- **Access**: `menu:layouts` → *Add/Edit Window*.
- **Definition**: Each window entry includes name, widget type, position, size, optional min/max dimensions, streams, border sides/style, colors, and per-widget settings.
- **Responsive Layouts**: `config.layout_mappings` maps terminal sizes to layout files (e.g., `compact`, `wide`). Add entries so Two-Face automatically switches layouts when you resize.
- **Editor Features**:
  - Drag popup to reposition.
  - Field-specific validation for numeric rows/cols.
  - Checkboxes for border sides (top/bottom/left/right).
  - Preview for border colors and backgrounds.

Refer to `LAYOUT_SYSTEM_DOCUMENTATION.md` for anchors, padding rules, and example layouts.

## Themes & Colors

### App Themes

- **Files**: under `.two-face/themes/*.toml`.
- **Management**:
  - Open **Theme Browser** to preview built-in + custom themes.
  - Use **Theme Editor** to tweak or create new themes (`ThemeData`).
  - Save to disk; the file appears in `themes/` and can be shared.
- **Internals**: `theme.rs` defines `AppTheme` (window, text, background, menu, status, button colors) and `ThemePresets::all()` loads built-ins.

### Palettes & UI Colors

- **Palette Colors** (`colors.toml`): named swatches used by forms. Maintain them via the color palette browser and editor (`color_palette_browser.rs`, `color_form.rs`).
- **UI Colors**: fine-grained overrides for prompts, system messages, and UI chrome. The **UI Colors Browser** exposes editable FG/BG pairs grouped by category.

### Spell & Highlight Colors

- **Spell Colors**: define bar/text/background colors for ranges of spell IDs (e.g., warm tone for defensive spells). Use the spell color browser/form.
- **Highlight Colors**: set per-highlight colors along with bolding, background fill, and entire-line coloring.

## Highlights & Automation

- Editor supports:
  - Regex validation via `regex` crate.
  - Fast-parse toggle (Aho-Corasick) for simple substrings.
  - Sound trigger selection (scans `sounds/` directory).
  - Preview of applied colors.
- Attach actions: play sound, color entire line, toggle bold, mark as “fast parse.”
- Event patterns (configurable in `config.toml`) convert regex matches into `ParsedElement::Event` entries so timers/invocation counters update automatically.

## Sounds

- Configure master sound settings in `config.sound` (`enabled`, `volume`, `cooldown_ms`).
- `sound.rs` enforces cooldowns per `sound_id` to prevent spam (e.g., repeated “crit” highlight).
- Drop files into `sounds/` and point highlights or other features at them. Unknown extensions are skipped gracefully.

## Settings Editor

- Centralizes boolean, numeric, string, color, and enum settings from `config.toml`.
- Categories cover connection, UI behavior (buffer size, timestamps, selection), performance overlay position, drag modifiers, etc.
- Read-only settings are labeled; editable ones accept inline input or toggle/cycle actions using the shared widget traits.

## Profiles

- Highlights and keybinds can be saved/loaded as profiles (see [Configuration](configuration.md)).
- Consider storing theme variants or layout files in version control if you curate multiple setups.

## Tips

- **Test changes live**: Many editors apply immediately. No restart is required after saving highlights, keybinds, colors, or layouts.
- **Backup before experimentations**: Copy `.two-face/` or commit to git before big overhauls.
- **Share presets**: Theme `.toml` files and layout `.toml` files are transferable between installations; just drop them into the same directories.
