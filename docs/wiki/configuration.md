# Runtime Directory & Configuration Files

Two-Face mirrors VellumFE’s file layout so migrating is painless. Everything lives under the “Two-Face directory,” which defaults to:

- **Linux/macOS**: `~/.two-face/`
- **Windows**: `%USERPROFILE%\.two-face\`

You can override the location with `--data-dir` or the `TWO_FACE_DIR` environment variable.

## Directory Layout

```
.two-face/
├── config.toml             # Core settings (connection, UI defaults, sound, etc.)
├── colors.toml             # Theme palette, prompt colors, UI colors, spell colors
├── highlights.toml         # All highlight patterns
├── keybinds.toml           # User keybinds and macros
├── menu_keybinds.toml      # Menu/form/browser shortcuts (auto-managed)
├── layouts/
│   ├── layout.toml         # Default layout
│   └── ...                 # Additional saved layouts
├── profiles/
│   ├── highlights/*.toml   # Alternate highlight sets
│   └── keybinds/*.toml     # Alternate keybind sets
├── sounds/                 # User-loaded audio files
├── cmdlist1.xml            # Command radial data (copied from defaults)
└── <character>/
    ├── config.toml         # Character overrides
    ├── colors.toml         # Character colors
    └── history.txt         # Command history for the CLI
```

> **Tip:** Two-Face never overwrites customized files. When a default changes upstream, the embedded copy in `defaults/` is only used if your file is missing.

## Config File Responsibilities

| File | Description | Loader |
|------|-------------|--------|
| `config.toml` | Connection info (`connection`), UI settings (`ui`), sound, layout mappings, event patterns, menu keybinds, active theme. | `Config::load` |
| `colors.toml` | Palette entries, prompt colors, UI color overrides, spell color ranges, theme metadata. | `Config::load_colors` |
| `highlights.toml` | Named regex highlights with colors, sounds, and parse hints. | `Config::load_highlights` |
| `keybinds.toml` | Simple map of `combo = { Action | Macro }`. | `Config::load_keybinds` |
| `layouts/*.toml` | Complete window definitions keyed by name; selected layout referenced in `config.toml`. | `Config::load_layouts` |
| `cmdlist1.xml` | Raw command radial definitions (coords + menu text + commands). | `cmdlist::CmdList::load` |

All loaders are centralized in `src/config.rs` and expose helper methods for saving profiles (`save_highlights_as`, `save_keybinds_as`, etc.).

## Character Overrides

When you launch with `--character <Name>`, Two-Face looks for per-character files under `.two-face/<Name>/`. Any file present there shadows the global version, letting you keep per-character layouts, highlights, and even command histories.

- Use the built-in UI (window editor, highlight editor, etc.) while a character is active to save directly into that folder.
- Remove a per-character file to fall back to the global copy on next startup.

## Defaults Extraction

On first run (or when a file is missing), embedded defaults are written to disk. The embedded content lives under `defaults/` in the repo and is referenced via `include_str!` or `include_dir!` macros, so you can customize them before building if you maintain your own fork.

## Profiles & Hot-Swapping

Both highlights and keybinds support named profiles stored in `profiles/highlights/*.toml` and `profiles/keybinds/*.toml`. Use the in-app browser menus to save/load profiles quickly:

1. Open the browser (`menu:highlights`, `menu:keybinds`).
2. Press the UI hint for “Save As…” or “Load Profile.”
3. The editor calls `Config::save_highlights_as` or `Config::load_keybinds_from`, so files appear under the relevant directory automatically.

## Command History

Each character gets a rolling `history.txt`. The command input widget appends commands longer than `ui.min_command_length` automatically, and the file is truncated to your `max_history` capacity to keep things tidy.

## Sound Assets

Drop `.ogg`, `.mp3`, `.wav`, or `.flac` files into `sounds/`. The highlight editor and other features scan for files on startup (`sound::ensure_sounds_directory`). If you create a subfolder, include the folder name in the path (`alerts/crit.ogg`).

## Backups

Because everything is plain text, simply include `.two-face/` in your backup solution. Consider version-controlling the directory if you heavily customize layouts or highlights.
