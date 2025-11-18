# Layout System Documentation

## Overview

This document provides a comprehensive analysis of the layout system in both VellumFE (the original) and two-face (the new implementation), covering every command, every operation, and every design decision.

## Core Architecture

### The Dual-Layout Pattern

Both VellumFE and two-face use a dual-layout architecture:

1. **baseline_layout** - The pristine, unscaled layout loaded from disk
   - Never modified during runtime (except when loading a new layout)
   - Used as the source of truth for `.resize` operations
   - Prevents cumulative scaling errors

2. **layout** - The working copy that may be scaled
   - Modified during window editing, resizing, adding/removing windows
   - This is what gets serialized during `.savelayout`
   - Contains current terminal size metadata

### WindowDef Enum Structure

```rust
pub enum WindowDef {
    Text {
        #[serde(flatten)]
        base: WindowBase,
        #[serde(flatten)]
        data: TextWidgetData,
    },
    Room {
        #[serde(flatten)]
        base: WindowBase,
        #[serde(flatten)]
        data: RoomWidgetData,
    },
    CommandInput {
        #[serde(flatten)]
        base: WindowBase,
        #[serde(flatten)]
        data: CommandInputWidgetData,
    },
}
```

Key points:
- Composition pattern: shared `WindowBase` + widget-specific data
- `#[serde(flatten)]` allows seamless TOML serialization
- Type-safe: impossible to access wrong fields for widget type
- Memory efficient: no wasted space on unused fields

### WindowPosition vs WindowDef

- **WindowDef**: The configuration (TOML-serializable)
  - Contains: name, widget_type, row, col, rows, cols, borders, colors, etc.
  - Stored in: `app_core.layout.windows`
  - Accessed via: `.base()` and `.base_mut()`

- **WindowPosition**: The runtime rendering coordinates
  - Contains: x, y, width, height (pure coordinates)
  - Stored in: `app_core.ui_state.windows[name].position`
  - Used by: TUI/GUI rendering code

**Critical invariant**: These must stay in sync. Most bugs come from violating this.

---

## Command: `.addwindow`

### VellumFE Implementation

**Location**: `src/app.rs:2212-2236`

**How it works**:
1. User enters `.addwindow <template_name>`
2. Gets window template from `available_window_templates()`
3. Creates new window with EXACT positions from template
4. Adds to `self.layout.windows`
5. Calls `update_window_manager_config()` which:
   - Converts each window to WindowConfig
   - Uses EXACT positions: `row: w.row, col: w.col, rows: w.rows, cols: w.cols`
   - **NO SCALING** happens here

**Key code**:
```rust
if let Some(template) = available_window_templates().iter().find(|t| t.name == template_name) {
    let mut new_window = template.window_def.clone();
    new_window.name = generate_unique_name(&template.name, &self.layout.windows);
    self.layout.windows.push(new_window);
    self.update_window_manager_config();
    self.add_system_message(&format!("Window '{}' added", new_window.name));
}
```

### two-face Implementation

**Location**: `src/core/app_core.rs:493-557`

**How it works**:
1. User enters `.addwindow <template_name>`
2. Gets window template from `Config::get_window_template()`
3. Generates unique name
4. Adds to `self.layout.windows`
5. Creates UI state with EXACT positions from template
6. **NO SCALING** happens

**Key code**:
```rust
let mut window_def = template.clone();
let unique_name = self.generate_unique_window_name(window_def.name());

// Update the name in the enum variant
match &mut window_def {
    WindowDef::Text { base, .. } => base.name = unique_name.clone(),
    WindowDef::Room { base, .. } => base.name = unique_name.clone(),
    WindowDef::CommandInput { base, .. } => base.name = unique_name.clone(),
}

let position = WindowPosition {
    x: window_def.base().col,
    y: window_def.base().row,
    width: window_def.base().cols,
    height: window_def.base().rows,
};

self.layout.windows.push(window_def.clone());
```

### Design Decision: Templates vs Dynamic Creation

**Why templates?**
- Ensures windows start with sane defaults
- Prevents users from creating windows with invalid configurations
- Makes `.addwindow` predictable and fast

**Available templates**:
- `main` - Text window for main story text
- `thoughts` - Text window for character thoughts
- `speech` - Text window for speech/dialogue
- `loot` - Text window for loot notifications
- `ambients` - Text window for ambient descriptions
- `announcements` - Text window for game announcements
- `room` - Room window for room descriptions
- `command_input` - Command input window (not in template list, auto-created)

### Known Flaws

1. **No validation of window overlap** - Users can create overlapping windows
2. **No validation of screen bounds** - Windows can be placed off-screen
3. **Template positions may not fit current terminal** - No automatic adjustment

---

## Command: `.savelayout`

### VellumFE Implementation

**Location**: `src/app.rs:2238-2246`

**How it works**:
1. User enters `.savelayout <name>`
2. Gets current terminal size via `crossterm::terminal::size()`
3. Calls `self.layout.save(name, terminal_size, force=false)`
4. Saves to `~/.vellum-fe/layouts/<name>.toml`

**Key code**:
```rust
"savelayout" => {
    let name = parts.get(1).unwrap_or(&"default");
    let terminal_size = crossterm::terminal::size().ok();
    // Don't force terminal size for manual saves (preserve baseline)
    match self.layout.save(name, terminal_size, false) {
        Ok(_) => self.add_system_message(&format!("Layout saved as '{}'", name)),
        Err(e) => self.add_system_message(&format!("Failed to save layout: {}", e)),
    }
}
```

**The `force` parameter**:
- `force=false` for manual `.savelayout` - preserves baseline size if layout hasn't been resized
- `force=true` for autosave on `.quit` - forces current terminal size

### two-face Implementation

**Location**: `src/main.rs:1054-1068`

**How it works**:
1. User enters `.savelayout <name>`
2. Gets current terminal size from frontend
3. Updates `layout.terminal_width` and `layout.terminal_height`
4. Calls `Layout::save_to_file()`
5. Saves to `~/.vellum-fe/layouts/<name>.toml`

**Key code**:
```rust
let name = command_parts.get(1).map(|s| s.as_str()).unwrap_or("default");
let (width, height) = frontend.size();

// Update terminal size in layout before saving
app_core.layout.terminal_width = Some(width);
app_core.layout.terminal_height = Some(height);

let layout_path = Config::get_layout_path(name);
if let Err(e) = crate::config::Layout::save_to_file(&app_core.layout, &layout_path) {
    app_core.add_system_message(&format!("Failed to save layout: {}", e));
} else {
    app_core.add_system_message(&format!("Layout '{}' saved ({}x{})", name, width, height));
}
```

### TOML Format

**Example saved layout**:
```toml
terminal_width = 123
terminal_height = 86

[[windows]]
widget_type = "text"
name = "main"
row = 11
col = 0
rows = 72
cols = 122
show_border = true
border_style = "single"
border_color = "-"
show_title = true
background_color = "-"
text_color = "-"
transparent_background = true
locked = false
streams = ["main"]
buffer_size = 10000

[windows.border_sides]
top = true
bottom = true
left = true
right = true

[[windows]]
widget_type = "room"
name = "room"
row = 0
col = 0
rows = 11
cols = 123
show_border = true
border_style = "single"
# ... etc
```

**Key observations**:
- Terminal size stored at top level
- Each window is a `[[windows]]` array entry
- `widget_type` discriminates the enum variant
- Border sides are nested tables
- Exact positions saved (no scaling during save)

### Known Flaws

1. **No backup of previous layout** - Overwriting a layout is destructive
2. **No validation during save** - Can save broken layouts
3. **Terminal size mismatch not warned** - Saving 123x86 layout on 80x24 terminal is allowed

---

## Command: `.loadlayout`

### VellumFE Implementation

**Location**: `src/app.rs:2247-2268`

**How it works**:
1. User enters `.loadlayout <name>`
2. Constructs path: `~/.vellum-fe/layouts/<name>.toml`
3. Calls `Layout::load_from_file()`
4. Sets both `self.layout` AND `self.baseline_layout` to loaded layout
5. Stores layout name in `self.base_layout_name`
6. Calls `update_window_manager_config()` with EXACT positions
7. **NO SCALING** happens

**Key code**:
```rust
"loadlayout" => {
    let name = parts.get(1).unwrap_or(&"default");
    let layout_path = Config::get_layout_path(name);

    match Layout::load_from_file(&layout_path) {
        Ok(new_layout) => {
            self.layout = new_layout.clone();
            self.baseline_layout = Some(new_layout);  // Store as baseline
            self.base_layout_name = Some(name.to_string());
            self.add_system_message(&format!("Layout '{}' loaded", name));
            self.update_window_manager_config();  // EXACT positions
        }
        Err(e) => {
            self.add_system_message(&format!("Failed to load layout: {}", e));
        }
    }
}
```

**Critical**: This uses EXACT positions from the TOML file. If the layout was saved at 123x86 but you're loading on 80x24, windows will be positioned incorrectly (possibly off-screen).

### two-face Implementation

**Location**: `src/main.rs:1015-1052`

**How it works**:
1. User enters `.loadlayout <name>`
2. Constructs path: `~/.vellum-fe/layouts/<name>.toml`
3. Calls `Layout::load_from_file()`
4. Sets both `layout` AND `baseline_layout`
5. Stores layout name in `base_layout_name`
6. Syncs to UI state with EXACT positions
7. **NO SCALING** happens (unless explicitly requested by user in future)

**Key code**:
```rust
let name = command_parts.get(1).map(|s| s.as_str()).unwrap_or("default");
let layout_path = Config::get_layout_path(name);

match Layout::load_from_file(&layout_path) {
    Ok(loaded_layout) => {
        app_core.layout = loaded_layout.clone();
        app_core.baseline_layout = Some(loaded_layout.clone());
        app_core.base_layout_name = Some(name.to_string());

        // Sync windows to UI state with EXACT positions
        app_core.sync_layout_to_ui_state();
        app_core.needs_render = true;

        app_core.add_system_message(&format!("Layout '{}' loaded", name));
    }
    Err(e) => {
        app_core.add_system_message(&format!("Failed to load layout '{}': {}", name, e));
    }
}
```

**The sync_layout_to_ui_state() function**:
```rust
pub fn sync_layout_to_ui_state(&mut self) {
    for window_def in &self.layout.windows {
        let position = WindowPosition {
            x: window_def.base().col,
            y: window_def.base().row,
            width: window_def.base().cols,
            height: window_def.base().rows,
        };

        if let Some(window_state) = self.ui_state.windows.get_mut(window_def.name()) {
            window_state.position = position;
        }
    }
}
```

### Design Decision: No Auto-Scaling on Load

**Why not auto-scale?**
- Preserves user's exact window positions
- Allows users to have different layouts for different terminal sizes
- `.resize` provides explicit scaling when needed

**Consequence**: Loading a 123x86 layout on 80x24 terminal shows windows off-screen. User must run `.resize` manually.

### Known Flaws

1. **No terminal size check** - Can load 123x86 layout on 80x24 terminal
2. **No warning about size mismatch** - User discovers off-screen windows by accident
3. **No auto-resize option** - Some users may expect automatic scaling

---

## Command: `.quit` and Autosave

### VellumFE Implementation

**Location**: `src/app.rs:2194-2208` (quit handler) + `src/app.rs:122-136` (autosave)

**How it works**:
1. User enters `.quit`
2. Sets `self.should_quit = true`
3. On next loop iteration, checks `should_quit`
4. If autosave enabled, calls `self.layout.save("autolayout", terminal_size, force=true)`
5. Exits application

**Key code**:
```rust
// In command handler
"quit" | "exit" | "q" => {
    self.should_quit = true;
    self.add_system_message("Goodbye!");
}

// In main loop before exit
if self.should_quit {
    if self.config.autosave_layout {
        let terminal_size = crossterm::terminal::size().ok();
        // Force=true: use current terminal size for autosave
        if let Err(e) = self.layout.save("autolayout", terminal_size, true) {
            tracing::error!("Failed to autosave layout: {}", e);
        }
    }
    return Ok(());
}
```

**The `force=true` parameter**: Forces the current terminal size to be saved, even if the baseline had a different size. This ensures autolayout reflects the user's current session.

### two-face Implementation

**Location**: `src/main.rs:1070-1090` + autosave in main loop (not yet implemented in new version)

**Current status**: Basic `.quit` exists, but autosave needs to be re-implemented.

**TODO**: Add autosave logic in main loop before exit:
```rust
// Should be added before exiting main loop
if app_core.should_quit {
    if app_core.config.autosave_layout {
        let (width, height) = frontend.size();
        app_core.layout.terminal_width = Some(width);
        app_core.layout.terminal_height = Some(height);

        let autolayout_path = Config::get_layout_path("autolayout");
        if let Err(e) = Layout::save_to_file(&app_core.layout, &autolayout_path) {
            tracing::error!("Failed to autosave layout: {}", e);
        }
    }
}
```

### Design Decision: Separate autolayout vs named layouts

**Why "autolayout"?**
- Preserves named layouts from accidental overwrite
- Users can exit without worrying about losing their custom layouts
- Can always `.loadlayout autolayout` to restore last session

**Consequence**: Users accumulate many named layouts. No automatic cleanup.

### Known Flaws

1. **Autosave not implemented in two-face yet**
2. **No autosave confirmation** - Silent on success, only logs on error
3. **No autosave disable per-session** - Must edit config file

---

## Command: `.resize`

### VellumFE Implementation

**Location**: `src/app.rs:3609-3669`

**How it works**:
1. User enters `.resize`
2. Gets current terminal size via `crossterm::terminal::size()`
3. Gets baseline layout from `self.baseline_layout`
4. Resets `self.layout` to `baseline_layout` (critical - prevents cumulative errors!)
5. Calls `apply_proportional_resize2()` to scale all windows
6. Updates window manager config with scaled positions
7. Autosaves to "autolayout" if enabled

**Key code**:
```rust
"resize" => {
    if let Some(baseline) = &self.baseline_layout {
        let (term_width, term_height) = crossterm::terminal::size()?;

        // Reset to baseline BEFORE scaling
        self.layout = baseline.clone();

        // Apply proportional scaling
        self.layout.apply_proportional_resize2(term_width, term_height);

        // Update window manager with scaled positions
        self.update_window_manager_config();

        // Autosave the resized layout
        if self.config.autosave_layout {
            let terminal_size = Some((term_width, term_height));
            let _ = self.layout.save("autolayout", terminal_size, true);
        }

        self.add_system_message(&format!("Layout resized to {}x{}", term_width, term_height));
    } else {
        self.add_system_message("No baseline layout loaded");
    }
}
```

**The apply_proportional_resize2() function**:
```rust
pub fn apply_proportional_resize2(&mut self, new_width: u16, new_height: u16) {
    let baseline_width = self.terminal_width.unwrap_or(new_width) as f32;
    let baseline_height = self.terminal_height.unwrap_or(new_height) as f32;

    let width_scale = new_width as f32 / baseline_width;
    let height_scale = new_height as f32 / baseline_height;

    for window in &mut self.windows {
        window.col = (window.col as f32 * width_scale) as u16;
        window.row = (window.row as f32 * height_scale) as u16;
        window.cols = (window.cols as f32 * width_scale).max(1.0) as u16;
        window.rows = (window.rows as f32 * height_scale).max(1.0) as u16;

        // Respect min/max constraints
        if let Some(min_cols) = window.min_cols {
            window.cols = window.cols.max(min_cols);
        }
        if let Some(max_cols) = window.max_cols {
            window.cols = window.cols.min(max_cols);
        }
        if let Some(min_rows) = window.min_rows {
            window.rows = window.rows.max(min_rows);
        }
        if let Some(max_rows) = window.max_rows {
            window.rows = window.rows.min(max_rows);
        }
    }

    self.terminal_width = Some(new_width);
    self.terminal_height = Some(new_height);
}
```

### two-face Implementation

**Location**: `src/core/app_core.rs:770-833` + `src/main.rs:1092-1099`

**How it works**:
1. User enters `.resize`
2. Gets current terminal size from frontend
3. Gets baseline layout from `self.baseline_layout`
4. Resets `self.layout` to `baseline_layout` (prevents cumulative errors!)
5. Calls `calculate_window_positions()` for proportional scaling
6. Applies scaled positions back to layout definitions
7. Updates UI state with scaled positions
8. Suggests user run `.savelayout` to persist

**Key code in main.rs**:
```rust
else if command == ".resize" {
    tracing::info!("[MAIN.RS] User entered .resize command");
    let (width, height) = frontend.size();
    tracing::info!("[MAIN.RS] Terminal size from frontend: {}x{}", width, height);
    app_core.resize_windows(width, height);
    app_core.needs_render = true;
}
```

**Key code in app_core.rs**:
```rust
pub fn resize_windows(&mut self, terminal_width: u16, terminal_height: u16) {
    tracing::info!("========== RESIZE WINDOWS START ==========");

    let baseline_layout = if let Some(ref bl) = self.baseline_layout {
        bl.clone()
    } else {
        self.add_system_message("Error: No baseline layout - cannot resize");
        return;
    };

    let baseline_width = baseline_layout.terminal_width.unwrap_or(terminal_width);
    let baseline_height = baseline_layout.terminal_height.unwrap_or(terminal_height);

    // Reset to baseline (prevents cumulative errors)
    self.layout = baseline_layout;

    // Calculate scaled positions
    let positions = self.calculate_window_positions(terminal_width, terminal_height);

    // Apply scaled positions back to layout definitions
    for window_def in &mut self.layout.windows {
        if let Some(position) = positions.get(window_def.name()) {
            let base = window_def.base_mut();
            base.col = position.x;
            base.row = position.y;
            base.cols = position.width;
            base.rows = position.height;
        }
    }

    // Update layout terminal size
    self.layout.terminal_width = Some(terminal_width);
    self.layout.terminal_height = Some(terminal_height);

    // Apply to UI state
    for window_def in &self.layout.windows {
        if let Some(window_state) = self.ui_state.windows.get_mut(window_def.name()) {
            let base = window_def.base();
            window_state.position = WindowPosition {
                x: base.col,
                y: base.row,
                width: base.cols,
                height: base.rows,
            };
        }
    }

    self.add_system_message(&format!("Resized to {}x{} - use .savelayout to save", terminal_width, terminal_height));
    tracing::info!("========== RESIZE WINDOWS COMPLETE ==========");
}
```

**The calculate_window_positions() function**:
Uses VellumFE's proportional scaling algorithm with min/max constraints.

### Design Decision: Manual .resize Only

**Why not auto-resize on terminal size change?**
- Prevents unwanted scaling when user adjusts terminal temporarily
- Gives user explicit control over when scaling happens
- Avoids cumulative scaling errors from multiple resize events
- Allows users to manually position windows off the standard grid

**Consequence**: Users must remember to run `.resize` after changing terminal size.

### Known Flaws

1. **No terminal resize event handler** - Must manually type `.resize`
2. **Proportional scaling may create gaps** - Windows don't always tile perfectly
3. **Min/max constraints can cause overlaps** - If window can't shrink enough, may overlap with neighbors
4. **Rounding errors accumulate** - Float to integer conversion loses precision

---

## Command: `.reload`

### VellumFE Implementation

**Location**: `src/app.rs:2270-2282`

**How it works**:
1. User enters `.reload`
2. If `base_layout_name` exists, calls equivalent of `.loadlayout <base_layout_name>`
3. Otherwise, shows error message

**Key code**:
```rust
"reload" => {
    if let Some(name) = &self.base_layout_name.clone() {
        let layout_path = Config::get_layout_path(name);
        match Layout::load_from_file(&layout_path) {
            Ok(new_layout) => {
                self.layout = new_layout.clone();
                self.baseline_layout = Some(new_layout);
                self.update_window_manager_config();
                self.add_system_message(&format!("Layout '{}' reloaded", name));
            }
            Err(e) => {
                self.add_system_message(&format!("Failed to reload layout: {}", e));
            }
        }
    } else {
        self.add_system_message("No layout loaded to reload");
    }
}
```

### two-face Implementation

**Location**: Not yet implemented in new version

**TODO**: Add `.reload` command handler in main.rs:
```rust
else if command == ".reload" {
    if let Some(name) = &app_core.base_layout_name.clone() {
        let layout_path = Config::get_layout_path(name);
        match Layout::load_from_file(&layout_path) {
            Ok(loaded_layout) => {
                app_core.layout = loaded_layout.clone();
                app_core.baseline_layout = Some(loaded_layout.clone());
                app_core.sync_layout_to_ui_state();
                app_core.needs_render = true;
                app_core.add_system_message(&format!("Layout '{}' reloaded", name));
            }
            Err(e) => {
                app_core.add_system_message(&format!("Failed to reload layout '{}': {}", name, e));
            }
        }
    } else {
        app_core.add_system_message("No layout loaded to reload");
    }
}
```

### Design Decision: Why .reload?

**Use cases**:
- Testing layout changes: edit TOML in external editor, then `.reload` in app
- Reverting window edits: discard in-memory changes, reload from disk
- Recovering from bad window positioning: reload baseline

**Consequence**: Another command to document and maintain. Could be replaced with `.loadlayout <last_name>`.

---

## Terminal Resize Event Handling

### VellumFE Implementation

**Location**: `src/app.rs:3555-3587`

**How it works**:
1. Crossterm event loop detects `Event::Resize` events
2. VellumFE **ignores** automatic resize events
3. User must manually run `.resize` command
4. No automatic scaling happens

**Key code**:
```rust
Event::Resize(width, height) => {
    // VellumFE does NOT auto-resize
    // User must run .resize command manually
    tracing::debug!("Terminal resized to {}x{} (ignored, use .resize command)", width, height);
}
```

**Why this design?**
- Prevents unwanted scaling during temporary terminal size changes
- Users often resize terminal to read something, then resize back
- Auto-scaling would destroy carefully positioned windows
- Explicit `.resize` command gives user control

### two-face Implementation

**Current status**: Same as VellumFE - no automatic handling

**Future option**: Could add auto-resize setting in config:
```toml
[app]
auto_resize_on_terminal_change = false  # default
```

### Known Flaws

1. **User may not know about .resize** - Windows appear broken after terminal resize
2. **No visual indicator** - Terminal size change is silent
3. **No "resize needed" prompt** - Could detect mismatch and suggest `.resize`

---

## Window Editing

### The Window Editor

**Location (VellumFE)**: `src/ui/window_editor.rs`
**Location (two-face)**: `src/frontend/tui/window_editor.rs`

**How it works**:
1. User enters `.menu` -> selects "Edit Window" -> selects window name
2. Window editor modal opens with text fields for all configurable properties
3. User edits fields (row, col, rows, cols, title, border style, etc.)
4. User presses Ctrl+S to save
5. Editor calls `update_window_position()` with edited window definition
6. **EXACT positions applied** - no scaling

### The Critical Bug (Now Fixed)

**The problem**: `update_window_position()` was calling `calculate_window_positions()`, which scaled the window instead of using exact positions.

**Symptom**: User edits "rows" from 72 to 50, presses Ctrl+S, nothing visible happens (because scaling algorithm re-scaled it back to ~72).

**The fix**: Remove `calculate_window_positions()` call, use exact positions from window definition:
```rust
pub fn update_window_position(&mut self, window_def: &WindowDef, _terminal_width: u16, _terminal_height: u16) {
    let base = window_def.base();
    let position = WindowPosition {
        x: base.col,
        y: base.row,
        width: base.cols,
        height: base.rows,
    };

    if let Some(window_state) = self.ui_state.windows.get_mut(window_def.name()) {
        window_state.position = position.clone();
        self.needs_render = true;
    }
}
```

Now when user edits rows to 50, it immediately applies rows=50, no scaling.

---

## Scaling Algorithm Deep Dive

### The calculate_window_positions() Function

**Location**: `src/core/app_core.rs:672-768`

**Purpose**: The ONLY function that should perform proportional scaling of windows.

**How it works**:
1. Gets baseline terminal size from layout
2. Calculates scale factors: `width_scale = new_width / baseline_width`
3. For each window:
   - Scale position: `new_col = old_col * width_scale`
   - Scale size: `new_cols = old_cols * width_scale`
   - Apply min/max constraints
   - Clamp to terminal bounds
4. Returns HashMap of `window_name -> WindowPosition`

**Key code**:
```rust
pub fn calculate_window_positions(&self, terminal_width: u16, terminal_height: u16) -> HashMap<String, WindowPosition> {
    let baseline_width = self.layout.terminal_width.unwrap_or(terminal_width) as f32;
    let baseline_height = self.layout.terminal_height.unwrap_or(terminal_height) as f32;

    let width_scale = terminal_width as f32 / baseline_width;
    let height_scale = terminal_height as f32 / baseline_height;

    let mut positions = HashMap::new();

    for window_def in &self.layout.windows {
        let base = window_def.base();

        // Scale position and size
        let scaled_col = (base.col as f32 * width_scale) as u16;
        let scaled_row = (base.row as f32 * height_scale) as u16;
        let scaled_cols = ((base.cols as f32 * width_scale).max(1.0)) as u16;
        let scaled_rows = ((base.rows as f32 * height_scale).max(1.0)) as u16;

        // Apply min/max constraints
        let final_cols = if let Some(min) = base.min_cols {
            scaled_cols.max(min)
        } else { scaled_cols };

        let final_cols = if let Some(max) = base.max_cols {
            final_cols.min(max)
        } else { final_cols };

        // Similar for rows...

        // Clamp to terminal bounds
        let clamped_col = scaled_col.min(terminal_width.saturating_sub(1));
        let clamped_row = scaled_row.min(terminal_height.saturating_sub(1));

        positions.insert(
            window_def.name().to_string(),
            WindowPosition {
                x: clamped_col,
                y: clamped_row,
                width: final_cols.min(terminal_width - clamped_col),
                height: final_rows.min(terminal_height - clamped_row),
            }
        );
    }

    positions
}
```

### Where calculate_window_positions() Should Be Called

**ONLY these places**:
1. ✅ `resize_windows()` - Manual `.resize` command
2. ✅ Initial layout load (if auto-resize enabled in future)

**NEVER these places**:
1. ❌ `update_window_position()` - Window editor (FIXED)
2. ❌ `add_new_window()` - Adding windows (FIXED)
3. ❌ `sync_layout_to_ui_state()` - Loading layouts (FIXED)
4. ❌ Any UI rendering code - UI should use exact positions from state

### Known Flaws in Scaling Algorithm

1. **Rounding errors**: Float to integer conversion loses precision
   - Example: 123 * 0.65 = 79.95 → 79, but 123 * 0.7 = 86.1 → 86
   - Over multiple resize operations, windows drift from intended positions

2. **Min/max constraint conflicts**:
   - If `min_cols=50` but scaled width is 30, window becomes 50
   - This can cause window to overlap with neighbors or exceed terminal bounds

3. **No gap filling**:
   - Proportional scaling can leave gaps between windows
   - Example: Two 50-wide windows side-by-side, scaled to 80-wide terminal → 32 + 32 = 64, leaving 16 columns empty

4. **No layout topology awareness**:
   - Doesn't know which windows are meant to be adjacent
   - Gaps between windows may appear after scaling

5. **No aspect ratio preservation**:
   - Width and height scale independently
   - A square window may become rectangular after resize

---

## Complete Command Flow Diagrams

### .addwindow Flow

```
User types: .addwindow main
    ↓
main.rs: Parse command, extract template name
    ↓
app_core.rs: add_new_window("main")
    ↓
config.rs: Config::get_window_template("main")
    ↓
Returns: WindowDef::Text with exact positions from template
    ↓
app_core.rs: Generate unique name (e.g. "main_2" if "main" exists)
    ↓
app_core.rs: Add to layout.windows Vec
    ↓
app_core.rs: Create UI state with EXACT position
    ↓
app_core.rs: Set needs_render = true
    ↓
Render loop: Draw window at exact position
```

**Key point**: NO SCALING at any step.

### .savelayout Flow

```
User types: .savelayout myconfig
    ↓
main.rs: Parse command, extract name
    ↓
frontend: Get terminal size (width, height)
    ↓
app_core.layout.terminal_width = Some(width)
app_core.layout.terminal_height = Some(height)
    ↓
config.rs: Layout::save_to_file(layout, path)
    ↓
serde: Serialize Layout to TOML
    ↓
fs::write: Write to ~/.vellum-fe/layouts/myconfig.toml
    ↓
Success message displayed
```

**Key point**: Terminal size saved with layout for future scaling reference.

### .loadlayout Flow

```
User types: .loadlayout myconfig
    ↓
main.rs: Parse command, extract name
    ↓
config.rs: Layout::load_from_file("~/.vellum-fe/layouts/myconfig.toml")
    ↓
serde: Deserialize TOML to Layout struct
    ↓
app_core.layout = loaded_layout.clone()
app_core.baseline_layout = Some(loaded_layout.clone())
app_core.base_layout_name = Some("myconfig")
    ↓
app_core.rs: sync_layout_to_ui_state()
    ↓
For each window in layout.windows:
    Get EXACT position (col, row, cols, rows)
    Update ui_state.windows[name].position
    ↓
app_core.needs_render = true
    ↓
Render loop: Draw windows at exact positions
```

**Key point**: NO SCALING, uses exact positions from TOML.

### .resize Flow

```
User types: .resize
    ↓
main.rs: Parse command
    ↓
frontend: Get terminal size (width, height)
    ↓
app_core.rs: resize_windows(width, height)
    ↓
Check baseline_layout exists (error if not)
    ↓
baseline_layout = self.baseline_layout.clone()
    ↓
self.layout = baseline_layout  ← RESET TO BASELINE
    ↓
calculate_window_positions(width, height)
    ↓
For each window:
    Calculate scale factors
    Apply proportional scaling
    Apply min/max constraints
    Clamp to terminal bounds
    ↓
Returns: HashMap<name, WindowPosition>
    ↓
For each window in layout.windows:
    window.base_mut().col = position.x
    window.base_mut().row = position.y
    window.base_mut().cols = position.width
    window.base_mut().rows = position.height
    ↓
layout.terminal_width = Some(width)
layout.terminal_height = Some(height)
    ↓
Sync scaled positions to ui_state
    ↓
app_core.needs_render = true
    ↓
Render loop: Draw windows at scaled positions
    ↓
Suggest user run .savelayout to persist
```

**Key point**: ONLY place calculate_window_positions() is called. Resets to baseline first!

### Window Edit Flow

```
User: .menu → Edit Window → main → Edit fields → Ctrl+S
    ↓
window_editor.rs: Build WindowDef from edited fields
    ↓
window_editor.rs: Call app_core.update_window_position(window_def)
    ↓
app_core.rs: update_window_position()
    ↓
Get EXACT position from window_def.base()
    col, row, cols, rows
    ↓
Create WindowPosition from exact values
    ↓
Update ui_state.windows[name].position = WindowPosition
    ↓
Update layout.windows (find and replace WindowDef)
    ↓
needs_render = true
    ↓
Render loop: Draw window at exact new position
```

**Key point**: NO SCALING. This was the critical bug - now fixed!

---

## Comprehensive Flaw Summary

### 1. Layout Loading Flaws

**Flaw**: No terminal size validation on load
- **Symptom**: Loading 123x86 layout on 80x24 terminal shows windows off-screen
- **Impact**: Confusing user experience, appears broken
- **Mitigation**: User must manually run `.resize`
- **Fix option**: Add warning message if terminal size differs from layout size

**Flaw**: No validation of window positions
- **Symptom**: Can load layouts with overlapping windows or off-screen windows
- **Impact**: Broken visuals, windows not clickable/editable
- **Mitigation**: None currently
- **Fix option**: Validate on load, clamp to bounds, warn user

### 2. Layout Saving Flaws

**Flaw**: Overwrites without backup
- **Symptom**: `.savelayout myconfig` overwrites existing myconfig.toml
- **Impact**: Lost work if user saves broken layout
- **Mitigation**: User must manually backup TOML files
- **Fix option**: Create .bak files before overwriting

**Flaw**: No validation before save
- **Symptom**: Can save layouts with invalid configurations
- **Impact**: Broken layouts persist on disk
- **Mitigation**: None currently
- **Fix option**: Validate layout before serializing

### 3. Scaling Algorithm Flaws

**Flaw**: Rounding errors accumulate
- **Symptom**: Windows drift from intended positions over multiple resizes
- **Impact**: Manual repositioning required
- **Mitigation**: Reset to baseline before each resize (IMPLEMENTED)
- **Fix option**: Use higher precision arithmetic, or snap to grid

**Flaw**: No gap filling
- **Symptom**: Empty spaces between windows after proportional scaling
- **Impact**: Wasted screen space
- **Mitigation**: Manual window positioning
- **Fix option**: Implement smart gap-filling algorithm

**Flaw**: Min/max constraints cause overlaps
- **Symptom**: Windows with min_cols=50 overlap neighbors when terminal shrinks to 60 cols
- **Impact**: Overlapping content, unreadable
- **Mitigation**: User must manually reposition
- **Fix option**: Detect overlaps, auto-adjust, or warn user

**Flaw**: No aspect ratio preservation
- **Symptom**: Square windows become rectangular after resize
- **Impact**: Aesthetic issue, may break assumptions in widget code
- **Mitigation**: None currently
- **Fix option**: Add aspect_ratio field, preserve during resize

### 4. Terminal Resize Event Flaws

**Flaw**: No automatic resize on terminal size change
- **Symptom**: Terminal resize → windows appear broken → user confused
- **Impact**: Poor UX, requires manual `.resize`
- **Mitigation**: User must learn `.resize` command
- **Fix option**: Add auto_resize_on_terminal_change config option

**Flaw**: No visual indicator of size mismatch
- **Symptom**: User doesn't know terminal size != layout size
- **Impact**: Confusion why windows look wrong
- **Mitigation**: None currently
- **Fix option**: Show terminal size vs layout size in status bar

### 5. Window Editor Flaws

**Flaw**: No real-time preview
- **Symptom**: Must press Ctrl+S to see changes
- **Impact**: Slow iteration on positioning
- **Mitigation**: Save frequently
- **Fix option**: Live preview mode

**Flaw**: No undo/redo
- **Symptom**: Bad edit requires manual reversal or `.reload`
- **Impact**: Frustrating editing experience
- **Mitigation**: `.reload` to discard changes
- **Fix option**: Implement edit history

**Flaw**: No validation of edits
- **Symptom**: Can set row=9999, cols=0, etc.
- **Impact**: Broken window positions
- **Mitigation**: User must manually fix
- **Fix option**: Validate fields before accepting

### 6. Command Completeness Flaws

**Flaw**: No `.removewindow` command
- **Symptom**: Can't delete windows without editing TOML
- **Impact**: Cumbersome window management
- **Mitigation**: Edit layout file manually
- **Fix option**: Implement `.removewindow <name>`

**Flaw**: No `.listwindows` command
- **Symptom**: Can't see window names without `.menu`
- **Impact**: Must remember or guess names
- **Mitigation**: Check layout TOML file
- **Fix option**: Implement `.listwindows`

**Flaw**: No `.editwindow <name>` direct command
- **Symptom**: Must go through `.menu` → Edit Window → select name
- **Impact**: Slow workflow for frequent edits
- **Mitigation**: Use `.menu`
- **Fix option**: Implement `.editwindow <name>` shortcut

### 7. Autosave Flaws

**Flaw**: Not implemented in two-face yet
- **Symptom**: Layout not saved on `.quit`
- **Impact**: Lost changes on exit
- **Mitigation**: Manual `.savelayout` before `.quit`
- **Fix option**: Implement autosave (high priority)

**Flaw**: No autosave confirmation or logging
- **Symptom**: Silent on success in VellumFE
- **Impact**: User doesn't know if autosave worked
- **Mitigation**: Check autolayout.toml timestamp
- **Fix option**: Add success message or log entry

### 8. Error Handling Flaws

**Flaw**: Generic error messages
- **Symptom**: "Failed to load layout: ..." with technical serde error
- **Impact**: User can't understand what went wrong
- **Mitigation**: Check logs
- **Fix option**: Parse error, show user-friendly message

**Flaw**: No recovery suggestions
- **Symptom**: Error shown, no hint on how to fix
- **Impact**: User stuck
- **Mitigation**: Manual debugging
- **Fix option**: Include suggestions in error messages

---

## Recommended Improvements (Priority Order)

### Critical (Should fix ASAP)

1. **Implement autosave in two-face** - Data loss risk
2. **Add terminal size mismatch warning on load** - Poor UX currently
3. **Validate window positions on load** - Prevent broken layouts

### High Priority

4. **Add `.removewindow` command** - Basic functionality gap
5. **Add `.listwindows` command** - Usability improvement
6. **Implement `.reload` in two-face** - Missing from original
7. **Add backup creation before overwriting layouts** - Data safety

### Medium Priority

8. **Add real-time preview in window editor** - UX improvement
9. **Implement undo/redo in window editor** - UX improvement
10. **Add auto_resize config option** - User preference
11. **Better error messages** - UX improvement

### Low Priority

12. **Smart gap-filling in resize** - Nice to have
13. **Aspect ratio preservation** - Edge case
14. **Overlap detection and warning** - Edge case

---

## Testing Checklist

### Basic Operations
- [ ] `.addwindow main` creates window at template position
- [ ] `.addwindow main` again creates "main_2" with unique name
- [ ] `.savelayout test` saves to ~/.vellum-fe/layouts/test.toml
- [ ] `.loadlayout test` loads exact positions from test.toml
- [ ] `.quit` exits application
- [ ] `.resize` scales all windows proportionally

### Edge Cases
- [ ] `.loadlayout` on non-existent file shows error
- [ ] `.loadlayout` with 123x86 layout on 80x24 terminal (windows off-screen)
- [ ] `.resize` after above brings windows into view
- [ ] `.savelayout` after `.resize` persists scaled positions
- [ ] Window editor: change rows from 72 to 50, Ctrl+S applies immediately
- [ ] Window editor: set invalid value (e.g. cols=0), check behavior
- [ ] Multiple `.resize` calls don't cause cumulative drift
- [ ] `.addwindow` after `.resize` uses template position (not scaled)

### Layout Persistence
- [ ] Save layout, exit, restart, load layout - exact positions preserved
- [ ] Autosave on exit (once implemented) saves current state
- [ ] `.reload` discards in-memory edits and restores from disk

### Stress Tests
- [ ] Load layout with 20+ windows
- [ ] Resize from 200x60 to 80x24 and back
- [ ] Edit window while layout is scaled (non-baseline)
- [ ] Save and load layout with all widget types (Text, Room, CommandInput)

---

## Conclusion

The layout system in both VellumFE and two-face follows a clear architecture:

**Baseline → Working → Scaled → Saved**

1. **Baseline layout** - Pristine, unscaled, loaded from disk
2. **Working layout** - May be modified, edited, or scaled
3. **Scaled positions** - Calculated dynamically, not stored
4. **Saved layout** - Working layout serialized with terminal size metadata

**Key principles**:
- **Exact positions by default** - No automatic scaling
- **Manual .resize only** - User controls scaling
- **Reset to baseline before scaling** - Prevents cumulative errors
- **Dual layout pattern** - baseline_layout + layout

**Critical bug fixed**:
- `update_window_position()` no longer calls `calculate_window_positions()`
- Window editor now applies exact positions immediately

**Remaining work**:
- Implement autosave in two-face
- Implement `.reload` in two-face
- Add validation and warnings
- Improve error messages

This system provides precise control over window positioning while supporting proportional scaling when needed. The explicit `.resize` command gives users full control over when scaling happens, preventing unwanted layout changes during temporary terminal adjustments.


## Known Issues and TODOs

### Widget Creation During Layout Load

**Issue**: Not all widget types are created in the sync loop during layout load.

**Current Status**:
- ✅ **Text windows**: Created via `.entry().or_insert_with()` in sync loop
- ✅ **Command input**: Created during initialization  
- ✅ **Room windows**: Fixed - now created in sync loop (line 166-173 of tui/mod.rs)
- ❌ **Other widgets**: Progress, Countdown, Compass, Indicator, Hands, Dashboard currently render directly without persistent state

**Why This Matters**:
If a widget type needs persistent state between renders (like TextWindow for scrolling/history or RoomWindow for component layout), it MUST be created during the sync loop. Otherwise, the widget won't exist when the render loop tries to use it.

**Pattern to Follow**:
```rust
} else if let crate::data::WindowContent::YourWidget(_) = &window.content {
    if !self.your_widgets.contains_key(name) {
        let widget = YourWidget::new(/* params */);
        self.your_widgets.insert(name.clone(), widget);
        tracing::debug!("Created YourWidget for '{}' during sync", name);
    }
}
```

**Location**: `src/frontend/tui/mod.rs` lines 89-185 in `sync_text_windows()` function

**TODO**: As more complex widget implementations are added, remember to add their creation logic to this sync loop.

