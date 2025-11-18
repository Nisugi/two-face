# Troubleshooting & FAQ

Problems happen—here’s how to diagnose and fix the most common ones.

## Connection Issues

| Symptom | Fix |
|---------|-----|
| `Connected` never appears | Ensure Lich is running, port matches, firewall allows localhost traffic, and `config.connection.host` is correct. |
| Immediate disconnect | Check `tracing` output for `Failed to connect to Lich` or `Error reading from server`; the Lich port might be wrong or already in use. |
| Commands do nothing | Confirm `SET_FRONTEND_PID` succeeded (watch logs); some Lich setups require restarting the front-end after attaching. |

## Parser/Highlight Errors

- **Regex panic**: Highlight editor validates patterns, but manual edits to `highlights.toml` can still break things. Two-Face logs `Invalid highlight regex 'name': error`.
- **Malformed XML**: If you edit `cmdlist1.xml` or intercept XML logs, ensure your editor preserves UTF-8 and closing tags.
- **Unexpected Colors**: Conflicting highlight layers? Later rules overwrite earlier ones. Reorder rules or reduce “color entire line” usage.

## Layout Problems

- **Window missing**: Check the layout file to ensure `widget_type` matches a valid widget (`text`, `room`, `countdown`, etc.). Invalid widgets are ignored with a warning.
- **Overlapping windows**: Use the layout editor to adjust rows/cols; values are absolute, so two windows can overlap if configured so.
- **No streams**: Text windows need a `streams` array. Without it, the window never receives text.

## Keybinds & Menus

- **Keybind not firing**: Duplicate combos? Only the first match executes. Use the browser’s duplicate detection or inspect `keybinds.toml`.
- **Menus ignore input**: Menu focus might be elsewhere. Press `Esc` to close popups and re-open the desired menu.

## Sound

- **No audio**: Confirm you built with `--features sound`, `sound.enabled = true`, and your system has an audio device. Logs show `Sound playback disabled` otherwise.
- **Repeated spam**: Tune `sound.cooldown_ms` to a higher value so the same highlight isn’t triggered multiple times per second.

## Performance

- **Laggy UI**: Open the Performance Stats widget to see if parse or render times spike. Look for:
  - Huge buffer sizes (lower `ui.buffer_size` if necessary).
  - Extremely chatty highlights (simplify regex).
- **High CPU in idle**: Disable overlays you don’t need, or reduce the configuration update frequency (coming soon).

## Configuration Corruption

- If a `.toml` file becomes unreadable, Two-Face logs a parsing error and falls back to defaults where possible.
- Keep backups of `.two-face/`. Restoring a file usually resolves the issue.

## FAQ

**Q: Can I run multiple instances?**  
A: Yes. Each instance can point to a different character or even a different data directory with `--data-dir`.

**Q: How do I reset to defaults?**  
A: Move/rename `.two-face/` and restart Two-Face. It recreates the directory with fresh defaults. Keep a backup if you want to restore parts later.

**Q: Where are logs stored?**  
A: Logs go to stdout/stderr. Redirect them (`two-face ... 2>two-face.log`) or use `RUST_LOG` to tweak verbosity.

**Q: Is there scripting?**  
A: Two-Face relies on Lich for scripting. Use highlights, keybind macros, and cmdlist entries for light automation inside the client.

Need more help? Open an issue in your fork or drop into the community chat with your log snippets and configuration files.
