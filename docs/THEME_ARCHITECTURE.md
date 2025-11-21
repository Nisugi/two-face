# Theme Architecture

**Last Updated**: 2025-01-18

This document describes the design and implementation of the theme system in Two-Face.

---

## Overview

The theme system provides comprehensive visual customization for the entire TUI application. It consists of:

- **77 theme fields** organized into 11 categories
- **16 built-in themes** with diverse color schemes
- **Custom theme support** via TOML files
- **Runtime theme switching** without restart
- **Theme editor** for creating custom themes

---

## Core Components

### 1. Theme Definition

**File**: `src/theme.rs` (1594 lines)

**Primary Structures**:

```rust
/// Complete application theme
pub struct AppTheme {
    // Meta
    pub name: String,
    pub description: String,

    // Window colors (4 fields)
    pub window_border: Color,
    pub window_border_focused: Color,
    pub window_background: Color,
    pub window_title: Color,

    // Text colors (4 fields)
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,
    pub text_selected: Color,

    // ... 67 more fields organized in 9 categories ...
}
```

**EditorTheme** (subset for backwards compatibility):
```rust
pub struct EditorTheme {
    pub border_color: Color,
    pub label_color: Color,
    pub focused_label_color: Color,
    pub text_color: Color,
    pub cursor_color: Color,
    pub status_message_color: Color,
    pub background_color: Color,
    pub section_header_color: Color,
}
```

**ThemeData** (serializable format):
```rust
// In src/frontend/tui/theme_editor.rs
pub struct ThemeData {
    pub name: String,
    pub description: String,
    // All theme fields as Option<String> for TOML serialization
    pub window_border: Option<String>,
    // ... etc
}
```

### 2. Theme Presets

**Location**: `src/theme.rs` - `ThemePresets` impl

**Built-in Themes** (16 total):
1. **dark** (default) - Classic dark theme with cyan accents
2. **light** - Bright theme for daytime use
3. **nord** - Arctic blue palette
4. **dracula** - Purple and pink accents
5. **solarized-dark** - Precision colors (dark mode)
6. **solarized-light** - Precision colors (light mode)
7. **monokai** - Vibrant warm coding theme
8. **gruvbox-dark** - Retro earthy colors
9. **night-owl** - Deep ocean blues
10. **catppuccin** - Mocha pastels
11. **cyberpunk** - Neon on black
12. **retro-terminal** - Amber/green monochrome
13. **apex** - Space station grays
14. **minimalist-warm** - Beige paper tones
15. **forest-creek** - Deep greens
16. **synthwave** - Neon magenta/cyan

**Implementation**:
```rust
impl ThemePresets {
    pub fn all() -> HashMap<String, AppTheme> {
        // Returns HashMap<theme_id, AppTheme>
        // Each theme defined as static function (e.g., Self::dark())
    }

    pub fn default_theme_name() -> &'static str {
        "dark"
    }
}
```

### 3. Custom Theme System

**Storage Location**: `~/.two-face/{character}/themes/*.toml`

**Format**: TOML serialization
```toml
name = "My Custom Theme"
description = "A personalized color scheme"

# All colors as hex strings
window_border = "#00ffff"
window_border_focused = "#ffff00"
text_primary = "#ffffff"
# ... 74 more fields
```

**Loading**:
```rust
impl ThemePresets {
    fn load_custom_themes(character: Option<&str>) -> HashMap<String, AppTheme> {
        // Scans ~/.two-face/{character}/themes/*.toml
        // Deserializes ThemeData → converts to AppTheme
        // Returns HashMap keyed by filename (without .toml)
    }

    pub fn all_with_custom(character: Option<&str>) -> HashMap<String, AppTheme> {
        let mut themes = Self::all(); // Built-in themes
        themes.extend(Self::load_custom_themes(character)); // Custom themes
        themes
    }
}
```

### 4. Theme Management

**Configuration**: `src/config.rs`
```rust
pub struct Config {
    pub active_theme: String, // Theme ID (e.g., "dark", "nord", "my-custom")
    // ... other config fields
}

impl Config {
    pub fn get_theme(&self) -> AppTheme {
        ThemePresets::all_with_custom(self.character.as_deref())
            .get(&self.active_theme)
            .cloned()
            .unwrap_or_else(ThemePresets::dark) // Fallback to dark theme
    }
}
```

**Active Theme Tracking**:
- Stored in `config.active_theme` (String)
- Persisted in `~/.two-face/{character}/config.toml`
- Default: "dark"

---

## Application Flow

### Startup Sequence

1. **Config Loading** (`main.rs`):
```rust
let config = Config::load(&args.character);
// config.active_theme loaded from config.toml
```

2. **AppCore Initialization** (`src/core/app_core.rs`):
```rust
let theme = config.get_theme(); // Loads active theme
// Theme stored in AppCore, passed to frontend
```

3. **Frontend Setup** (`src/frontend/tui/mod.rs`):
```rust
pub fn new(app_core: &AppCore) -> Self {
    let theme = app_core.config.get_theme();
    // Theme used to render initial UI
}
```

### Runtime Theme Switching

1. **User Opens Theme Browser**:
```bash
.themes  # Command to open theme browser
```

2. **Theme Selection** (`src/frontend/tui/theme_browser.rs`):
```rust
// User navigates list and presses Enter
// Returns selected theme ID
pub fn get_selected_theme(&self) -> Option<String>
```

3. **Theme Application** (`src/main.rs`):
```rust
// Update config
app_core.config.active_theme = selected_theme_id;

// Save to disk
app_core.config.save();

// Trigger re-render (automatic with needs_render = true)
app_core.needs_render = true;
```

4. **Re-render**:
```rust
// Next frame, theme is loaded fresh
let theme = app_core.config.get_theme();
// All widgets re-render with new theme
```

### Custom Theme Creation

1. **User Opens Theme Editor**:
```bash
.edittheme              # Create new theme
.edittheme mytheme      # Edit existing custom theme
```

2. **Theme Editing** (`src/frontend/tui/theme_editor.rs`):
```rust
pub struct ThemeEditor {
    theme_data: ThemeData,  // Editable theme data
    // ... editing state
}

// User edits colors via color picker
// Changes stored in theme_data
```

3. **Theme Saving**:
```rust
impl ThemeData {
    pub fn save_to_file(&self, character: Option<&str>) -> Result<()> {
        let path = format!("~/.two-face/{}/themes/{}.toml",
                          character.unwrap_or("default"),
                          self.name);

        let toml = toml::to_string_pretty(self)?;
        std::fs::write(path, toml)?;
        Ok(())
    }
}
```

4. **Theme Availability**:
- Custom themes automatically available in `.themes` browser
- Loaded on-demand via `ThemePresets::load_custom_themes()`

---

## Theme Application Patterns

### Pattern 1: Direct Field Access (Most Common)

**Used by**: 90% of widgets

```rust
pub fn render(&self, area: Rect, buf: &mut Buffer, theme: &AppTheme) {
    // Direct access to theme fields
    let border_style = Style::default().fg(theme.window_border);
    let text_style = Style::default().fg(theme.text_primary);

    // Apply to widgets
    buf.set_string(x, y, "Text", text_style);
}
```

**Files**: All browser widgets, forms, settings editor, theme editor

### Pattern 2: Fallback Chain (Window Borders)

**Used by**: Window rendering in `mod.rs`

```rust
// Widget-specific border_color from layout config (optional)
// Falls back to theme.window_border
let border_color = normalize_color(&window_def.base().border_color)
    .or_else(|| color_to_hex_string(&theme.window_border));

// Focused windows use theme.window_border_focused
if window.focused {
    border_style = Style::default().fg(theme.window_border_focused);
} else {
    border_style = Style::default().fg(theme.window_border);
}
```

**Fallback Hierarchy**:
1. Window-specific `border_color` (from layout file)
2. `theme.window_border_focused` (if focused)
3. `theme.window_border` (default)

### Pattern 3: EditorTheme Conversion

**Used by**: Window editor

```rust
// In theme.rs
impl AppTheme {
    pub fn to_editor_theme(&self) -> EditorTheme {
        EditorTheme {
            border_color: self.editor_border,
            label_color: self.editor_label,
            focused_label_color: self.editor_label_focused,
            text_color: self.editor_text,
            cursor_color: self.editor_cursor,
            status_message_color: self.editor_status,
            background_color: self.editor_background,
            section_header_color: self.menu_item_focused,
        }
    }
}

// In window_editor.rs
pub fn render(&mut self, area: Rect, buf: &mut Buffer, theme: &EditorTheme) {
    // Uses EditorTheme instead of AppTheme
    let border = Style::default().fg(theme.border_color);
}

// In mod.rs
let editor_theme = theme.to_editor_theme();
window_editor.render(area, buf, &editor_theme);
```

**Why?**: Backwards compatibility with original editor implementation

### Pattern 4: Dynamic Color Lookup (Rarely Used)

**Used by**: Color palette browser, dynamic color resolution

```rust
impl AppTheme {
    pub fn get_color(&self, name: &str) -> Option<Color> {
        match name {
            "window_border" => Some(self.window_border),
            "window_border_focused" => Some(self.window_border_focused),
            "text_primary" => Some(self.text_primary),
            // ... all 75 color fields
            _ => None,
        }
    }
}

// Usage
if let Some(color) = theme.get_color("link_color") {
    style = Style::default().fg(color);
}
```

### Pattern 5: Color Derivation (Special Cases)

**Used by**: Injury doll widget

```rust
// Derive injury_default_color from theme
let injury_color = blend_colors(
    theme.window_background,
    theme.text_secondary,
    0.3  // 30% blend
);

pub fn blend_colors(bg: Color, fg: Color, alpha: f32) -> Color {
    // Extract RGB components
    // Blend: result = bg * (1 - alpha) + fg * alpha
    // Return new Color
}
```

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                        Startup                              │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │  config.toml     │
                    │  active_theme:   │
                    │    "dark"        │
                    └──────────────────┘
                              │
                              ▼
                   Config::get_theme()
                              │
          ┌───────────────────┴───────────────────┐
          ▼                                       ▼
    ┌──────────────────┐                ┌─────────────────────┐
    │ Built-in Themes  │                │  Custom Themes      │
    │ ThemePresets::   │                │  ~/.two-face/       │
    │   all()          │                │    themes/*.toml    │
    └──────────────────┘                └─────────────────────┘
          │                                       │
          └───────────────────┬───────────────────┘
                              ▼
                       ┌─────────────┐
                       │  AppTheme   │
                       │  (active)   │
                       └─────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                     Rendering Loop                          │
└─────────────────────────────────────────────────────────────┘
          │                               │                │
          ▼                               ▼                ▼
    ┌──────────┐                   ┌──────────┐     ┌──────────┐
    │ Widget A │                   │ Widget B │     │ Widget C │
    │ .render( │                   │ .render( │     │ .render( │
    │  &theme) │                   │  &theme) │     │  &theme) │
    └──────────┘                   └──────────┘     └──────────┘
          │                               │                │
          └───────────────────┬───────────────────────────┘
                              ▼
                    ┌──────────────────┐
                    │   Terminal       │
                    │   Display        │
                    └──────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                   Theme Switching                           │
└─────────────────────────────────────────────────────────────┘
          │
          ▼
    .themes command
          │
          ▼
    ┌────────────────┐
    │ Theme Browser  │
    │  - dark        │
    │  - light       │
    │  → nord        │  (user selects)
    │  - dracula     │
    └────────────────┘
          │
          ▼
    config.active_theme = "nord"
    config.save()
          │
          ▼
    app_core.needs_render = true
          │
          ▼
    (re-render with new theme)

┌─────────────────────────────────────────────────────────────┐
│                 Custom Theme Creation                       │
└─────────────────────────────────────────────────────────────┘
          │
          ▼
    .edittheme command
          │
          ▼
    ┌────────────────┐
    │  Theme Editor  │
    │                │
    │  Name: ────    │  (user edits)
    │  Colors:       │
    │   window_─     │
    │   text_pr─     │
    └────────────────┘
          │
          ▼
    ThemeData::save_to_file()
          │
          ▼
    ~/.two-face/themes/my-theme.toml
          │
          ▼
    (auto-available in .themes browser)
```

---

## File Structure

```
two-face/
├── src/
│   ├── theme.rs                          # Core theme definitions
│   │   ├── struct AppTheme               # 77 theme fields
│   │   ├── struct EditorTheme            # Subset for editors
│   │   ├── impl ThemePresets             # Built-in themes
│   │   └── Color conversion utils        # hex ↔ Color
│   │
│   ├── config.rs                         # Configuration
│   │   ├── struct Config                 # active_theme field
│   │   └── fn get_theme()                # Theme loading
│   │
│   └── frontend/tui/
│       ├── theme_browser.rs              # Theme selection UI
│       ├── theme_editor.rs               # Theme creation UI
│       │   └── struct ThemeData          # Serializable theme
│       │
│       └── */                            # All widgets
│           └── *.rs                      # Use &AppTheme in render()
│
└── ~/.two-face/                          # User data
    └── {character}/
        ├── config.toml                   # active_theme = "dark"
        └── themes/                       # Custom themes
            ├── my-theme.toml
            ├── work-theme.toml
            └── night-theme.toml
```

---

## Color Format Handling

### Internal Representation
All colors stored as `ratatui::style::Color` enum:
```rust
pub enum Color {
    Reset,
    Black,
    Red,
    // ... named colors
    Rgb(u8, u8, u8),        // Most common for themes
    Indexed(u8),            // 256-color palette
}
```

### Theme Definition
Built-in themes use `Color::Rgb()`:
```rust
AppTheme {
    window_border: Color::Rgb(0, 255, 255),  // Cyan
    text_primary: Color::Rgb(255, 255, 255), // White
    // ...
}
```

### Custom Theme Serialization
TOML files use hex strings:
```toml
window_border = "#00ffff"
text_primary = "#ffffff"
```

### Conversion Functions
```rust
// Hex string → Color
pub fn parse_hex_color(hex: &str) -> Option<Color> {
    // "#RRGGBB" → Color::Rgb(r, g, b)
}

// Color → Hex string
pub fn color_to_hex_string(color: &Color) -> Option<String> {
    // Color::Rgb(r, g, b) → "#RRGGBB"
}
```

### Fallback Strategy
```rust
// Named colors fallback to hex
let color = config.ui.some_color  // May be "red" or "#ff0000"
    .as_deref()
    .and_then(parse_hex_color)
    .unwrap_or(Color::White);     // Ultimate fallback
```

---

## Performance Considerations

### Theme Loading
- **Cold Start**: Loads on application startup (~1-2ms for all 16 built-in themes)
- **Custom Themes**: TOML parsing adds ~5-10ms per theme file
- **Optimization**: Themes loaded once, then cached in `Config`

### Theme Switching
- **No Restart Required**: Immediate re-render
- **Memory**: Single `AppTheme` instance (~1KB)
- **Speed**: O(1) HashMap lookup by theme ID

### Render Performance
- **Zero Overhead**: Direct field access (`theme.text_primary`)
- **No Dynamic Dispatch**: All colors resolved at compile time
- **Branch Prediction**: Focused/unfocused states predictable

---

## Extension Points

### Adding New Built-in Themes
1. Add theme function to `ThemePresets` impl in `src/theme.rs`
2. Register in `ThemePresets::all()` HashMap
3. Done - auto-available in `.themes` browser

Example:
```rust
impl ThemePresets {
    fn my_new_theme() -> AppTheme {
        AppTheme {
            name: "My New Theme".to_string(),
            description: "A beautiful new color scheme".to_string(),
            window_border: Color::Rgb(100, 150, 200),
            // ... all 75 color fields
        }
    }

    pub fn all() -> HashMap<String, AppTheme> {
        let mut themes = HashMap::new();
        // ... existing themes
        themes.insert("my-new-theme".to_string(), Self::my_new_theme());
        themes
    }
}
```

### Adding New Theme Fields
1. Add field to `AppTheme` struct in `src/theme.rs`
2. Add to `ThemeData` struct in `src/frontend/tui/theme_editor.rs`
3. Add to theme editor UI rendering
4. Add to all built-in theme definitions
5. Use in widget rendering

**Warning**: This breaks custom theme compatibility unless you provide defaults.

### Custom Theme Locations
Modify `ThemePresets::load_custom_themes()` to scan additional directories:
```rust
fn load_custom_themes(character: Option<&str>) -> HashMap<String, AppTheme> {
    let mut themes = HashMap::new();

    // User themes
    scan_theme_dir(&format!("~/.two-face/{}/themes", character), &mut themes);

    // System themes
    scan_theme_dir("/usr/share/two-face/themes", &mut themes);

    themes
}
```

---

## Known Limitations

### 1. Unused Fields
- 25 out of 75 color fields are completely unused
- Creates maintenance burden
- Confuses theme creators

**Impact**: Low (fields are ignored)
**Fix**: Remove unused fields (breaking change)

### 2. Hardcoded Colors
- ~80+ hardcoded `Color::White`, `Color::Cyan`, etc. in widgets
- Performance stats widget, color picker, etc.
- Doesn't respect theme

**Impact**: Medium (some widgets ignore theme)
**Fix**: Replace hardcoded colors with theme fields

### 3. Config vs Theme Duplication
- `command_echo` in both `AppTheme` and `config.colors.ui`
- `selection_background` duplicated
- Actual code uses `config.colors.ui` version

**Impact**: Low (theme fields ignored)
**Fix**: Consolidate to single source

### 4. No Live Preview
- Theme editor doesn't show live preview
- Must save and switch to see changes

**Impact**: Medium (poor UX)
**Fix**: Add preview panel to theme editor

### 5. No Theme Export
- Can't export effective theme (with all fallbacks resolved)
- Can't duplicate built-in theme to customize

**Impact**: Low (workaround: manually copy all fields)
**Fix**: Add theme export function

---

## Best Practices

### For Theme Creators

1. **Start from Existing Theme**: Copy a built-in theme via theme editor
2. **Test All Widgets**: Open all browsers, forms, editors to test
3. **Check Contrast**: Ensure text readable on backgrounds
4. **Name Clearly**: Use descriptive names (e.g., "ocean-blue" not "theme1")
5. **Document**: Add description explaining theme purpose

### For Widget Developers

1. **Always Use Theme**: Never hardcode colors
2. **Use Semantic Fields**: `text_primary` not `window_border` for text
3. **Respect Focused State**: Use `_focused` variants when applicable
4. **Provide Fallbacks**: Use `unwrap_or(theme.text_primary)` for optional colors
5. **Test Multiple Themes**: Verify widget in light and dark themes

### For Theme System Maintainers

1. **Document Usage**: Keep this file updated with new patterns
2. **Deprecate Carefully**: Unused fields should be marked deprecated before removal
3. **Version Custom Themes**: Add version field to detect old custom themes
4. **Validate on Load**: Check for required fields, provide defaults for missing
5. **Performance Profile**: Monitor theme loading/switching performance

---

## Future Enhancements

### Short Term
1. Remove unused theme fields (break custom theme compat once)
2. Fix hardcoded colors in widgets
3. Add theme export function
4. Add live preview to theme editor

### Medium Term
1. Theme inheritance (extend existing theme)
2. Per-window theme overrides
3. Dynamic theme switching based on time of day
4. Theme validation tools

### Long Term
1. Theme gallery/sharing
2. Theme import from popular formats (iTerm2, VS Code, etc.)
3. Automatic contrast checking
4. Theme generation from base colors
5. Animation/transition support
