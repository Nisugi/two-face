# Advanced Features

Once you are comfortable with the basics, explore these power-user tools baked directly into Two-Face.

## Clipboard & Selection

- **Modules**: `selection.rs`, `clipboard.rs`, `frontend/tui/text_window.rs`
- **Highlights**:
  - Drag with the mouse or use Shift+Arrow keys to mark text.
  - Selection respects window boundaries when `ui.selection_respect_window_boundaries` is true.
  - Copy to clipboard (system clipboard when possible; otherwise an internal buffer) for quick pasting into scripts or notes.

## Command History & Search

- Search bar overlays (per text window) accept regex patterns. The parser’s sanitized text ensures predictable matching.
- `history.txt` per character keeps commands even across sessions; use it as a mini log or to re-run complex instructions.

## Sound System

- **Module**: `sound.rs`
- Uses rodio when built with the `sound` feature; otherwise the API is a no-op, so you can keep sound settings enabled even on machines without audio output.
- `cooldown_map` prevents spam by tracking when each `sound_id` was last played.

## Performance Monitoring

- **Collector**: `performance.rs`
- **Widget**: `frontend/tui/performance_stats.rs`
- Tracks:
  - Frame times and FPS (rolling average + max).
  - Render, UI, text-wrap timing to pinpoint bottlenecks.
  - Parser chunk throughput and XML elements per second.
  - Network bytes/sec in/out.
  - Event processing costs and queue sizes.
  - Estimated memory usage based on buffered lines and window count.
- Useful when tuning layout complexity or testing highlight-heavy setups.

## Logging & Tracing

- Two-Face uses `tracing` for structured logs. Enable via environment variable:
  ```
  RUST_LOG=two_face=info two-face --character ...
  ```
- Log statements exist in the network stack, parser, configuration loader, and highlight evaluator to quickly spot malformed files.

## Event Patterns & Countdown Sync

- Define regex-driven timers in `config.event_patterns`.
- Supports:
  - `action`: `Set`, `Clear`, `Increment`.
  - `duration` and `duration_capture` for parsing `[XX]` style timers.
  - `duration_multiplier` (e.g., convert rounds to seconds).
- AppCore converts matches into `ParsedElement::Event`, which in turn updates countdown widgets and active effects.

## Menu System Shortcuts

- Menu keybinds let you navigate almost everything without touching the mouse.
- Use the Settings editor to tailor bindings for tabs, page up/down, delete, edit, and profile load/save operations.

## Soundless Environments

- Running without the `sound` feature? The API still logs attempts, so you can see whether a highlight would have played audio without causing errors.

## Dot-Command Autocomplete Catalog

- AppCore maintains a list of known dot-commands parsed from built-in scripts. Add your own by editing the command list or hooking into AppCore’s helper; both frontends pick up the changes automatically.

These features reward exploration, so don’t hesitate to toggle overlays, enable tracing, or script around the available hooks to craft the client you’ve always wanted.
