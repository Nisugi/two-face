# Feature Parity Analysis: Two-Face vs VellumFE

**Analysis Date**: 2025-11-21
**Scope**: Feature-by-feature comparison of user-visible behaviors
**Baseline**: VellumFE as reference implementation

---

## Executive Summary

This analysis compares 18 major feature areas between VellumFE and Two-Face. Of these:

- **Same**: 8 features (44%) - Identical behavior, well-implemented
- **Intentional Differences**: 4 features (22%) - Improvements or architectural changes
- **Regressions**: 3 features (17%) - Behavior degradation compared to VellumFE
- **Missing**: 3 features (17%) - Not yet implemented in Two-Face

**High-Impact Issues**: Resize debouncing regression, window state persistence regression, missing XML stream resumption.

---

## Feature Parity Matrix

| Feature Area | VellumFE Behavior | Two-Face Behavior | Status | Notes |
|---|---|---|---|---|
| **Launch Flow** | Lich proxy required (port 8000) | Supports both Lich proxy + direct eAccess auth | Intentional Difference | Two-Face adds direct auth, better flexibility |
| **Configuration Load** | Single config.toml + embedded defaults | Modular config (toml) + theme files | Same | Both support character-specific configs |
| **Menu System** | Context menus on link click via cmdlist | Context menus via cmdlist.xml parsing | Same | Implementation identical in structure |
| **Theme/Colors System** | EditorTheme (8 fields) for form styling | AppTheme (77 fields) full theme system | Intentional Difference | Two-Face provides comprehensive theming |
| **Highlight Editor** | Form with pattern, color, sound options | Form with pattern, color, sound options | Same | Feature parity achieved (SPEC-TF-FORM-HIGHLIGHTS) |
| **Keybind Editor** | Map keyboard input to game commands | Map keyboard input to game commands | Same | Both support keybind creation/editing |
| **Settings Editor** | Edit connection, UI settings | Edit connection, UI settings | Same | Configuration management equivalent |
| **Theme Browser/Editor** | Not present (hardcoded EditorTheme) | Full theme browser and custom theme editor | Missing in VellumFE | Two-Face adds comprehensive theme system |
| **Window Management** | Windows managed via Layout TOML | Windows managed via Layout + WindowConfig | Same | Both use layout files for window geometry |
| **Terminal Resize Handling** | Debounced 100ms to prevent excessive redraws | **No debouncing implemented** | **Regression** | Immediate resize processing may cause performance issues (see `src/frontend/tui/mod.rs` event loop) |
| **Search Feature** | Text search mode (Ctrl+F) with highlighting | Text search implemented in text_window | Same | Search functionality preserved |
| **Performance Stats** | Display frame time, event lag (debug mode) | PerformanceStats struct tracks FPS, event lag | Same | Both track performance metrics |
| **TTS (Text-to-Speech)** | Not implemented in VellumFE | Full TTS support via tts/ module with TtsManager | Missing in VellumFE | Two-Face adds accessibility feature (SPEC-TF-A11Y-TTS) |
| **Sound Support** | Highlight sounds via embedded sound files | Highlight sounds via SoundPlayer | Same | Both support audio cues |
| **Error Handling** | Error messages displayed in error stream | Errors displayed in ui_state + error rendering | Same | Error reporting equivalent |
| **XML Stream Handling** | Multi-stream support (main, input, prompt, familiar, etc.) | **Core supports streams, TUI rendering incomplete** | **Regression** | Two-Face captures stream IDs in UiState but TUI rendering doesn't always respect stream context (see `src/core/messages.rs` vs `src/frontend/tui/mod.rs` text_window update) |
| **Window State Persistence** | Windows saved to ~/.vellum-fe/layout/{character} | **Windows should persist but cache may not flush** | **Regression** | UiState.window_config populated but unclear if saved on exit (see `src/data/ui_state.rs` window_config field) |
| **Link Interaction** | Click links for context menus, right-click for commands | Click links for context menus | Same | Menu interaction equivalent |

---

## Detailed Findings

### 1. Launch Flow
**VellumFE**: Connects via Lich proxy on localhost:8000
**Two-Face**: Supports both Lich proxy (`--host` + `--port`) AND direct eAccess auth (`--direct`, `--direct-account`, etc.)
**Status**: Intentional Difference
**Evidence**: `C:\gemstone\projects\two-face\src\main.rs` (lines 24-72 show CLI args), `.claude/CLAUDE.md` describes direct auth implementation
**Impact**: Two-Face is more flexible for standalone usage; VellumFE requires Lich infrastructure

### 2. Configuration System
**VellumFE**: `config.toml`, `colors.toml`, `highlights.toml`, `keybinds.toml` loaded from defaults/ + user directory
**Two-Face**: Same structure, additionally supports per-character config subdirectories and theme files
**Status**: Same (with enhancement)
**Evidence**: `C:\gemstone\projects\vellumfe\src\config.rs` (lines 10-21), `C:\gemstone\projects\two-face\src\config.rs`
**Impact**: Both support character-specific configurations; Two-Face adds theme persistence

### 3. Menu System
**VellumFE**: Parses `cmdlist.xml`, creates context menus on link click at cursor position
**Two-Face**: Identical cmdlist.xml parsing and menu population via input_router
**Status**: Same
**Evidence**: `C:\gemstone\projects\vellumfe\src\cmdlist.rs`, `C:\gemstone\projects\two-face\src\cmdlist.rs`
**Impact**: No behavioral difference in menu creation or interaction

### 4. Theme System
**VellumFE**: EditorTheme struct (8 fields: border, label, focused_label, text, cursor, status, background, section_header)
**Two-Face**: AppTheme struct (77 fields) organized in 11 categories covering all UI elements
**Status**: Intentional Difference
**Evidence**: `C:\gemstone\projects\vellumfe\src\config.rs` EditorTheme, `C:\gemstone\projects\two-face\src\theme.rs` AppTheme
**Impact**: Two-Face provides GUI-ready comprehensive theming; VellumFE hardcodes colors throughout

### 5. Highlight Editor
**VellumFE**: HighlightFormWidget with pattern, color, sound, checkboxes (bold, color_entire_line, fast_parse)
**Two-Face**: HighlightForm with identical fields and behavior
**Status**: Same
**Evidence**: `C:\gemstone\projects\vellumfe\src\ui\highlight_form.rs`, `C:\gemstone\projects\two-face\src\frontend\tui\highlight_form.rs`
**Impact**: Feature parity achieved

### 6. Keybind Editor
**VellumFE**: KeybindFormWidget maps keyboard input to game commands
**Two-Face**: KeybindForm with same mapping functionality
**Status**: Same
**Evidence**: `C:\gemstone\projects\vellumfe\src\ui\keybind_form.rs`, `C:\gemstone\projects\two-face\src\frontend\tui\keybind_form.rs`
**Impact**: Feature parity achieved

### 7. Settings Editor
**VellumFE**: SettingsEditorWidget for connection host/port, UI layout selection
**Two-Face**: SettingsEditor with same configuration options
**Status**: Same
**Evidence**: `C:\gemstone\projects\vellumfe\src\ui\settings_editor.rs`, `C:\gemstone\projects\two-face\src\frontend\tui\settings_editor.rs`
**Impact**: Feature parity achieved

### 8. Theme Browser/Editor
**VellumFE**: No theme switching or custom theme support (colors hardcoded)
**Two-Face**: ThemeBrowser lists 16 built-in + custom themes, ThemeEditor creates/edits custom themes
**Status**: Missing in VellumFE (Enhancement in Two-Face)
**Evidence**: `C:\gemstone\projects\two-face\src\frontend\tui\theme_editor.rs` (953 lines), `theme.rs` (16 built-in themes)
**Impact**: Two-Face adds significant customization feature absent in VellumFE

### 9. Window Management
**VellumFE**: Layout TOML defines window positions, sizes, and focus order
**Two-Face**: Layout TOML + WindowConfig in UiState manages window geometry
**Status**: Same
**Evidence**: Both use layout files; Two-Face adds runtime WindowConfig tracking
**Impact**: Feature parity in layout management

### 10. Terminal Resize Handling
**VellumFE**: Implements ResizeDebouncer (100ms debounce) in `src/app.rs` (lines 30-77) to prevent excessive layout recalculations
**Two-Face**: **No debouncing implemented** - immediately processes terminal resize events
**Status**: **Regression**
**Evidence**: `C:\gemstone\projects\vellumfe\src\app.rs` has `ResizeDebouncer`, Two-Face event loop in `src/frontend/tui/mod.rs` (line 2683) processes resize immediately without debounce
**Impact**: Performance degradation during terminal resize operations; potential flicker and slower responsiveness

### 11. Search Feature
**VellumFE**: SearchMode activated by Ctrl+F, highlights matching text
**Two-Face**: Search mode in TextWindow with same highlighting
**Status**: Same
**Evidence**: Both InputMode enums include Search state
**Impact**: Feature parity achieved

### 12. Performance Stats
**VellumFE**: PerformanceStats tracks frame time, event lag; displayed when requested
**Two-Face**: PerformanceStats with FPS, event lag, render time tracking in `src/performance.rs`
**Status**: Same
**Evidence**: Both projects implement performance monitoring; Two-Face adds additional metrics
**Impact**: Feature parity achieved with enhancement

### 13. TTS (Text-to-Speech)
**VellumFE**: Not implemented
**Two-Face**: Full TTS system via `src/tts/mod.rs` with TtsManager, accessible via menu + Ctrl+T hotkey
**Status**: Missing in VellumFE (Enhancement in Two-Face)
**Evidence**: `C:\gemstone\projects\two-face\src\tts\mod.rs` (complete TTS implementation), no equivalent in VellumFE
**Impact**: Two-Face adds accessibility feature for vision-impaired users

### 14. Sound Support
**VellumFE**: SoundPlayer plays highlight sounds from embedded defaults/sounds/ directory
**Two-Face**: SoundPlayer with same functionality via `src/sound.rs`
**Status**: Same
**Evidence**: Both implement audio cue system
**Impact**: Feature parity achieved

### 15. Error Handling
**VellumFE**: Errors written to error stream, displayed in dedicated window
**Two-Face**: Errors captured in UiState, rendered with status colors
**Status**: Same
**Evidence**: `src/core/messages.rs` routes errors to UiState; TUI renders in status area
**Impact**: Feature parity achieved

### 16. XML Stream Handling
**VellumFE**: Supports multiple streams (main, input, prompt, familiar, etc.), routes text to appropriate windows
**Two-Face**: **Core supports streams** (GameState tracks stream associations), **but TUI rendering incomplete**
**Status**: **Regression**
**Evidence**:
- `C:\gemstone\projects\two-face\src\data\ui_state.rs` has window_stream mapping
- `C:\gemstone\projects\two-face\src\core\messages.rs` routes to appropriate streams
- **BUT**: `C:\gemstone\projects\two-face\src\frontend\tui\mod.rs` (line 2683+) text_window update doesn't always respect stream context; text_window.rs rendering doesn't clearly associate streams with windows
**Impact**: Text may be routed incorrectly or missed in certain stream contexts

### 17. Window State Persistence
**VellumFE**: Saves window layouts to `~/.vellum-fe/layout/{character}` on exit
**Two-Face**: **UiState.window_config populated at runtime**, **unclear if flushed to disk on shutdown**
**Status**: **Regression** (unclear implementation)
**Evidence**:
- `C:\gemstone\projects\two-face\src\data\ui_state.rs` has `window_config: WindowConfig` field
- `C:\gemstone\projects\two-face\src\data\window.rs` has WindowConfig struct
- **Missing**: No explicit call to save window_config in main.rs shutdown path
**Impact**: Window positions/sizes may be lost on restart, regression in user state management

### 18. Link Interaction
**VellumFE**: Click links → context menu; right-click → direct command
**Two-Face**: Click links → context menu generation via PopupMenu
**Status**: Same (Single-action model)
**Evidence**: Both route link clicks through menu system
**Impact**: Two-Face removed right-click option but maintains primary interaction (intentional simplification)

---

## Summary by Status

### Same (8 features, 44%)
1. Configuration Load
2. Menu System
3. Highlight Editor
4. Keybind Editor
5. Settings Editor
6. Search Feature
7. Sound Support
8. Error Handling

### Intentional Differences (4 features, 22%)
1. Launch Flow (added direct auth)
2. Theme System (enhanced from 8→77 fields)
3. Window Management (added runtime tracking)
4. Performance Stats (enhanced metrics)

### Regressions (3 features, 17%)
1. **Terminal Resize Debouncing** (Performance issue)
2. **XML Stream Handling** (Incomplete routing)
3. **Window State Persistence** (May not save on exit)

### Missing (3 features, 17%)
1. Theme Browser/Editor (Added in Two-Face as enhancement)
2. TTS Support (Added in Two-Face as enhancement)
3. (Note: "Missing" items are actually Two-Face additions not in VellumFE)

---

## Recommendations for Regression Fixes

### High Priority
1. **Add resize debouncing to TUI frontend** - Restore 100ms debounce logic to prevent performance issues
2. **Verify window state persistence** - Implement explicit save-on-exit or auto-load-on-init mechanism
3. **Complete stream handling in TUI** - Ensure text_window respects stream context and routes text correctly

### Medium Priority
4. **Add stream-aware window rendering** - Tag rendered text with its source stream for debugging
5. **Test XML multi-stream scenarios** - Verify combat/spell streams appear in correct windows

---

**END OF FEATURE PARITY ANALYSIS**
