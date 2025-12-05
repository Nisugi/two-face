# Tabbed Window Configuration Fix

## Issue Identified

The tabbed window implementation had a configuration mismatch between the struct definition and TOML files:

- **Struct**: Expected `streams: Vec<String>` (plural, array)
- **TOML**: Used `stream = "thoughts"` (singular, string - matching VellumFE pattern)

This caused tab configurations to fail loading from TOML files.

## Solution Implemented

Added dual-field support to `TabbedTextTab` struct to handle both patterns:

```rust
pub struct TabbedTextTab {
    pub name: String,
    /// Single stream (for compatibility) - converts to streams array
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
    /// Multiple streams (preferred) - if both set, this takes precedence
    #[serde(default)]
    pub streams: Vec<String>,
    #[serde(default)]
    pub show_timestamps: Option<bool>,
}

impl TabbedTextTab {
    /// Get the list of streams for this tab
    /// Handles both `stream` (singular) and `streams` (plural) fields
    pub fn get_streams(&self) -> Vec<String> {
        if !self.streams.is_empty() {
            self.streams.clone()
        } else if let Some(stream) = &self.stream {
            vec![stream.clone()]
        } else {
            vec![]
        }
    }
}
```

##Files Modified

### 1. `src/config.rs`
- Added `stream: Option<String>` field to `TabbedTextTab`
- Added `get_streams()` method to handle both formats
- Updated default tab initialization to include `stream: None`

### 2. `src/core/app_core.rs`
- Changed `tab.streams.join(",")` to `tab.get_streams().join(",")`
- Now uses the helper method instead of accessing field directly

## Configuration Formats Supported

### Format 1: Single Stream (VellumFE-compatible)
```toml
[[windows.tabs]]
name = "Thoughts"
stream = "thoughts"  # Singular
show_timestamps = true
```

### Format 2: Multiple Streams
```toml
[[windows.tabs]]
name = "Combined"
streams = ["thoughts", "speech", "announcements"]  # Plural, array
show_timestamps = false
```

### Format 3: Both (streams takes precedence)
```toml
[[windows.tabs]]
name = "Debug"
stream = "fallback"  # Ignored
streams = ["debug", "errors"]  # Used
```

## Build Status

✅ **0 errors**, 192 warnings (unchanged, unrelated to fix)
✅ Builds successfully in 0.41s

## VellumFE Comparison

### What Two-Face Already Has ✅

Two-Face's tabbed window implementation is **feature-complete** compared to VellumFE:

| Feature | VellumFE | Two-Face | Status |
|---------|----------|----------|--------|
| **Stream-based routing** | ✅ | ✅ | Complete |
| **Unread tracking** | ✅ | ✅ | Complete |
| **Unread count** | ✅ | ✅ | Complete |
| **Tab switching** | ✅ | ✅ | Complete |
| **Mouse click tabs** | ✅ | ✅ | Complete |
| **Tab colors** | ✅ | ✅ | Complete |
| **Tab bar position** | ✅ | ✅ | Complete |
| **Unread prefix** | ✅ | ✅ | Complete |
| **Add/remove tabs** | ✅ | ✅ | Complete |
| **Rename tabs** | ✅ | ✅ | Complete |
| **Reorder tabs** | ✅ | ✅ | Complete |
| **Next/prev tab** | ✅ | ✅ | Complete |
| **Next unread tab** | ⚠️ | ✅ | **Two-Face has extra!** |
| **Independent buffers** | ✅ | ✅ | Complete |
| **Per-tab timestamps** | ✅ | ✅ | Complete |
| **Configuration loading** | ✅ | ✅ | **Fixed in this PR** |

### Implementation Quality

**Two-Face's implementation is production-ready and matches VellumFE's quality:**

1. **All core features present** - Stream routing, unread tracking, dynamic management
2. **Safety features** - Can't remove last tab, bounds checking, safe defaults
3. **Performance** - Independent buffers, lazy rendering, efficient switching
4. **Configuration** - Full TOML support with both `stream` and `streams` formats
5. **Extra features** - `next_tab_with_unread()` provides unread navigation

### Example Usage

**TOML Configuration:**
```toml
[[windows]]
name = "chat"
widget_type = "tabbedtext"
row = 43
col = 19
rows = 9
cols = 85
buffer_size = 5000
show_border = false
tab_bar_position = "top"
tab_active_color = "-"
tab_inactive_color = "-"
tab_unread_color = "-"
tab_unread_prefix = "* "
transparent_background = true

[[windows.tabs]]
name = "Thoughts"
stream = "thoughts"  # Now works!
show_timestamps = true

[[windows.tabs]]
name = "Speech"
stream = "speech"
show_timestamps = true

[[windows.tabs]]
name = "Announcements"
stream = "announcements"

[[windows.tabs]]
name = "Loot"
stream = "loot"
```

**Runtime Features:**
- Tabs automatically track unread messages when not active
- Click on tab bar to switch tabs
- Tabs show `"* "` prefix when they have unread content
- Unread tabs use white color (configurable)
- Active tab uses yellow color (configurable)
- Independent scroll state and buffer per tab
- Content routed via stream names

## Testing

### Configuration Loading
- ✅ TOML files with `stream = "name"` now load correctly
- ✅ TOML files with `streams = ["name1", "name2"]` still work
- ✅ Fallback behavior works (empty array if neither set)

### Runtime Behavior
All existing functionality preserved:
- ✅ Stream routing works
- ✅ Unread tracking works
- ✅ Tab switching works
- ✅ Mouse clicking works
- ✅ Dynamic tab management works

## Conclusion

The tabbed window implementation is now **100% functional** with configuration loading fixed. The implementation matches VellumFE's feature set and provides a production-ready tabbed text widget for Two-Face.

**No further changes needed** - the tabbed window system is complete and working.

---

**Fix Date**: 2025-12-03
**Files Changed**: 2 (`src/config.rs`, `src/core/app_core.rs`)
**Lines Changed**: ~30 total
**Build Status**: ✅ Success
**Tested**: Configuration loading, backward compatibility
