# Two-Face Keybind Actions Reference

Complete reference for all keybind actions available in Two-Face.

## Overview

Two-Face supports **44 built-in actions** that can be bound to any key combination. Actions are organized into categories based on their function.

### Action Types

- **Action Keybinds**: Trigger specific Two-Face functions (e.g., `cursor_left`, `scroll_current_window_up_one`)
- **Macro Keybinds**: Send literal text to the game (e.g., `"go north"`, `"cast 506"`)

---

## Command Input Actions (11)

Actions that manipulate the command input line.

| Action | Description | Implementation Status |
|--------|-------------|----------------------|
| `send_command` | Submit the current command line | ✅ Implemented |
| `cursor_left` | Move cursor left one character | ✅ Implemented |
| `cursor_right` | Move cursor right one character | ✅ Implemented |
| `cursor_word_left` | Jump cursor left by word | ✅ Implemented |
| `cursor_word_right` | Jump cursor right by word | ✅ Implemented |
| `cursor_home` | Jump cursor to start of line | ✅ Implemented |
| `cursor_end` | Jump cursor to end of line | ✅ Implemented |
| `cursor_backspace` | Delete character before cursor | ✅ Implemented |
| `cursor_delete` | Delete character at cursor | ✅ Implemented |
| `cursor_delete_word` | Delete from cursor to end of word | ✅ Implemented |
| `cursor_clear_line` | Clear entire command line | ✅ Implemented |

**Implementation**: Handled by CommandInput widget via frontend routing.

---

## Command History Actions (4)

Navigate and reuse previous commands.

| Action | Description | Implementation Status |
|--------|-------------|----------------------|
| `previous_command` | Navigate to previous command in history | ✅ Implemented |
| `next_command` | Navigate to next command in history | ✅ Implemented |
| `send_last_command` | Resend the last command | ✅ Implemented |
| `send_second_last_command` | Resend the second-to-last command | ✅ Implemented |

**Implementation**: Handled by CommandInput widget via frontend routing.

---

## Window Scrolling Actions (7)

Control scrolling for the currently focused text window.

| Action | Description | Implementation Status |
|--------|-------------|----------------------|
| `switch_current_window` | Cycle through windows | 🚧 TODO |
| `scroll_current_window_up_one` | Scroll active window up by 1 line | ✅ Implemented |
| `scroll_current_window_down_one` | Scroll active window down by 1 line | ✅ Implemented |
| `scroll_current_window_up_page` | Scroll active window up by 1 page (20 lines) | ✅ Implemented |
| `scroll_current_window_down_page` | Scroll active window down by 1 page (20 lines) | ✅ Implemented |
| `scroll_current_window_home` | Scroll to top of window (oldest content) | ✅ Implemented |
| `scroll_current_window_end` | Scroll to bottom of window (newest content) | ✅ Implemented |

**Implementation**: Methods in `AppCore` ([app_core.rs:302-367](../src/core/app_core.rs#L302-367))

**Note**: "Home" scrolls to the *top* (oldest messages), "End" scrolls to *bottom* (newest messages).

---

## Tab Navigation Actions (3)

Navigate between tabs in TabbedText windows (e.g., chat window).

| Action | Description | Implementation Status |
|--------|-------------|----------------------|
| `next_tab` | Switch to next tab | ✅ Implemented |
| `prev_tab` | Switch to previous tab | ✅ Implemented |
| `next_unread_tab` | Jump to next tab with unread messages | 🚧 TODO |

**Implementation**: Routed to frontend in [main.rs:3377-3404](../src/main.rs#L3377-3404), using frontend methods `next_tab_all()` and `prev_tab_all()`.

**Note**: `next_unread_tab` is pending frontend implementation.

---

## Search Actions (4)

Search functionality for text windows.

| Action | Description | Implementation Status |
|--------|-------------|----------------------|
| `start_search` | Open search mode | 🚧 TODO |
| `next_search_match` | Jump to next search result | 🚧 TODO |
| `prev_search_match` | Jump to previous search result | 🚧 TODO |
| `clear_search` | Clear search and exit search mode | 🚧 TODO |

**Implementation**: Placeholders exist, needs input mode integration.

---

## Clipboard Actions (3)

Clipboard integration for command input.

| Action | Description | Implementation Status |
|--------|-------------|----------------------|
| `copy` | Copy selected text to clipboard | ✅ Widget method exists |
| `paste` | Paste from clipboard | ✅ Widget method exists |
| `select_all` | Select all text in command input | ✅ Widget method exists |

**Implementation**: CommandInput widget has clipboard support, needs keybind routing.

**Current Behavior**:
- Mouse selection automatically copies to clipboard
- `Ctrl+V` pastes in command line
- `Ctrl+A` selects all (configurable via these actions)

---

## Text-to-Speech (TTS) Actions (9)

Accessibility features for screen readers.

| Action | Description | Implementation Status |
|--------|-------------|----------------------|
| `tts_next` | Read next message | ✅ Implemented |
| `tts_previous` | Read previous message | ✅ Implemented |
| `tts_next_unread` | Skip to next unread message | ✅ Implemented |
| `tts_stop` | Stop current speech | ✅ Implemented |
| `tts_mute_toggle` | Toggle TTS on/off | ✅ Implemented |
| `tts_increase_rate` | Speed up speech rate (+0.1) | ✅ Implemented |
| `tts_decrease_rate` | Slow down speech rate (-0.1) | ✅ Implemented |
| `tts_increase_volume` | Increase volume (+0.1) | ✅ Implemented |
| `tts_decrease_volume` | Decrease volume (-0.1) | ✅ Implemented |

**Implementation**: Full TTS integration via `tts_manager` ([app_core.rs:487-517](../src/core/app_core.rs#L487-517))

---

## System Toggle Actions (3)

Toggle global system features on/off.

| Action | Description | Implementation Status |
|--------|-------------|----------------------|
| `toggle_performance_stats` | Show/hide performance overlay | ✅ Implemented |
| `toggle_ignores` | Enable/disable squelch patterns globally | ✅ Implemented |
| `toggle_sounds` | Enable/disable sound system | ✅ Implemented |

**Implementation**:
- `toggle_performance_stats` - Toggles `config.ui.performance_stats_enabled` ([app_core.rs:478-483](../src/core/app_core.rs#L478-483))
- `toggle_ignores` - Toggles `config.ui.ignores_enabled` ([app_core.rs:484-489](../src/core/app_core.rs#L484-489))
- `toggle_sounds` - Toggles `config.sound.enabled` ([app_core.rs:490-495](../src/core/app_core.rs#L490-495))
- All show system messages to confirm the toggle state

---

## Configuration Examples

### Action Keybind Format

In `keybinds.toml`:

```toml
[ctrl+e]
action = "cursor_end"

[ctrl+a]
action = "cursor_home"

[ctrl+w]
action = "cursor_delete_word"

[ctrl+u]
action = "cursor_clear_line"

[pageup]
action = "scroll_current_window_up_page"

[pagedown]
action = "scroll_current_window_down_page"

[home]
action = "scroll_current_window_home"

[end]
action = "scroll_current_window_end"

[ctrl+tab]
action = "next_tab"

[ctrl+shift+tab]
action = "prev_tab"

[ctrl+n]
action = "next_unread_tab"

[ctrl+i]
action = "toggle_ignores"

[ctrl+m]
action = "toggle_sounds"

[f12]
action = "toggle_performance_stats"
```

### Macro Keybind Format

```toml
[num_1]
macro_text = "go southwest"

[num_2]
macro_text = "go south"

[f1]
macro_text = "cast 506"

[f2]
macro_text = "get all"
```

**Note**: Legacy `\r` and `\n` escape sequences are automatically stripped. Multi-command macros will be revisited with a better system in the future.

---

## Key Combination Syntax

### Modifiers
- `ctrl+` - Control key
- `alt+` - Alt key
- `shift+` - Shift key
- Combine multiple: `ctrl+shift+a`

### Special Keys
- Function keys: `f1`, `f2`, ..., `f12`
- Navigation: `pageup`, `pagedown`, `home`, `end`
- Arrows: `up`, `down`, `left`, `right`
- Numpad: `num_0` through `num_9`, `num_+`, `num_-`, `num_*`, `num_/`
- Other: `tab`, `space`, `backspace`, `delete`, `enter`

### Regular Keys
- Letters: `a`, `b`, `c`, ... (case-insensitive)
- Numbers: `0`, `1`, `2`, ...
- Symbols: `-`, `=`, `[`, `]`, etc.

---

## Default Keybinds

Two-Face comes with sensible defaults. View them with:
```bash
cat ~/.two-face/keybinds.toml
```

Or from the embedded defaults:
```bash
# Located in: defaults/keybinds/default.toml
```

To reset to defaults, simply delete your `keybinds.toml` file.

---

## Managing Keybinds

### Via Dot Commands

```
.keybinds              # Open keybinds browser
.addkeybind            # Create new keybind
```

### Via Files

**keybinds.toml structure**:
```toml
[global]  # Global keybinds (quit, search, close)
quit = "ctrl+c"
start_search = "ctrl+f"
...

[ctrl+1]  # User keybinds (game mode)
action = "scroll_current_window_up_page"
...
```

**File locations**:
- **Global keybinds**: `~/.two-face/keybinds.toml` - `[global]` section
- **User keybinds**: `~/.two-face/keybinds.toml` - all other sections
- **Character-specific**: `~/.two-face/characters/<name>/keybinds.toml`

Character-specific keybinds override global keybinds for that character.

---

## Implementation Status Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Fully implemented and working |
| 🚧 | Defined but needs implementation |
| ⚠️ | Partially implemented |

---

## Action Routing

Actions are routed differently based on their category:

1. **Command Input Actions** → Routed to CommandInput widget via frontend
2. **Window Actions** → Handled by AppCore methods
3. **Tab Navigation** → Will route to focused TabbedText window
4. **TTS Actions** → Handled by TTS manager
5. **Macro Actions** → Processed by AppCore, sent to game server

---

## Adding Custom Actions

To add a new action:

1. Add enum variant to `KeyAction` in [config.rs:1474](../src/config.rs#L1474)
2. Add to `from_str()` parser in [config.rs:1815](../src/config.rs#L1815)
3. Add handler in `execute_key_action()` in [app_core.rs:405](../src/core/app_core.rs#L405)
4. Update this documentation
5. Add to AVAILABLE_ACTIONS in [keybind_form.rs:61](../src/frontend/tui/keybind_form.rs#L61)

---

## See Also

- [Input and Menus Documentation](wiki/input_and_menus.md)
- [Configuration Guide](wiki/configuration.md)
- [Keybind Form Widget](../src/frontend/tui/keybind_form.rs)
- [Keybind Browser Widget](../src/frontend/tui/keybind_browser.rs)

---

**Last Updated**: 2025-12-04
**Version**: Two-Face v0.1.0
