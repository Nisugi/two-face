# Recommendations: Prioritized SPEC Candidates

**Analysis Date**: 2025-11-21
**Basis**: Findings from feature parity, widget styling, and architecture analyses
**Focus**: Actionable SPECs to close gaps and prepare for GUI port

---

## Overview

This document proposes **8 prioritized SPEC candidates** organized into three categories:

1. **SPEC-TF-PARITY** (3 SPECs): Fix behavioral regressions vs VellumFE
2. **SPEC-TF-STYLE** (2 SPECs): Achieve visual consistency and theme integration
3. **SPEC-TF-CORE** (3 SPECs): Refactor for multi-frontend support

**Total Estimated Work**: Moderate-High (20-30 days for experienced team)

---

## Category 1: Feature Parity Fixes (SPEC-TF-PARITY)

### SPEC-TF-PARITY-RESIZE-001: Restore Resize Debouncing

**Priority**: HIGH
**Status**: Regression (VellumFE has this, Two-Face removed it)

**Rationale**:
VellumFE implements a 100ms resize debouncer to prevent excessive layout recalculations during terminal resize. Two-Face processes resize events immediately, causing performance degradation when users resize the terminal. This is a clear regression that impacts UX.

**Current State**:
- VellumFE: `src/app.rs` lines 30-77 define `ResizeDebouncer` struct with pending size tracking
- Two-Face: `src/frontend/tui/mod.rs` event loop processes `FrontendEvent::Resize` directly without debouncing

**Proposed Solution**:
```rust
// File: src/frontend/tui/resize_debouncer.rs (NEW)
pub struct ResizeDebouncer {
    last_resize_time: Instant,
    debounce_duration: Duration,
    pending_size: Option<(u16, u16)>,
}

impl ResizeDebouncer {
    pub fn check_resize(&mut self, width: u16, height: u16) -> Option<(u16, u16)> {
        // Return Some if enough time elapsed since last resize
        // Return None to debounce
    }

    pub fn check_pending(&mut self) -> Option<(u16, u16)> {
        // Check if pending resize should be processed
    }
}

// File: src/frontend/tui/mod.rs (MODIFY)
// Add to TuiFrontend struct:
resize_debouncer: ResizeDebouncer,

// In event loop:
FrontendEvent::Resize(w, h) => {
    if let Some((w, h)) = self.resize_debouncer.check_resize(w, h) {
        app_core.handle_resize(w, h)?;
    }
}

// Periodic check:
if let Some((w, h)) = self.resize_debouncer.check_pending() {
    app_core.handle_resize(w, h)?;
}
```

**Affected Files**:
- `src/frontend/tui/mod.rs` (event loop, add ResizeDebouncer)
- NEW: `src/frontend/tui/resize_debouncer.rs` (new file)

**Complexity**: Simple (copy pattern from VellumFE)
**Dependencies**: None (independent fix)
**GUI Port Impact**: Helpful (resize handling abstraction needed for GUI too)

**Acceptance Criteria**:
- Terminal resize events are debounced (configurable, default 100ms)
- No performance degradation during rapid resize
- Test: Resize terminal quickly, verify no flicker or excessive redraws

---

### SPEC-TF-PARITY-STREAM-HANDLING-001: Complete Multi-Stream XML Text Routing

**Priority**: HIGH
**Status**: Regression (text may not appear in correct windows)

**Rationale**:
Two-Face core supports XML stream routing (main, input, prompt, familiar, etc.) but TUI rendering doesn't consistently respect stream context. Text may be routed to wrong windows or missed entirely. This impacts gameplay for spells, combat, and other stream-aware content.

**Current State**:
- Core: `src/core/messages.rs` routes text to appropriate stream IDs
- UiState: `src/data/ui_state.rs` has window-to-stream mapping
- TUI: `src/frontend/tui/text_window.rs` doesn't always check stream context

**Proposed Solution**:
1. Add stream ID tracking to `TextSegment`:
```rust
pub struct TextSegment {
    text: String,
    stream_id: String,  // NEW: which stream this came from
    color: Option<Color>,
    bold: bool,
}
```

2. In text_window.rs, filter by stream:
```rust
impl TextWindow {
    fn get_visible_lines(&self) -> Vec<&TextSegment> {
        self.buffer.iter()
            .filter(|seg| {
                // Only show segments from this window's streams
                self.streams.contains(&seg.stream_id)
            })
            .collect()
    }
}
```

3. In TUI render loop, verify stream context:
```rust
// Ensure text is routed to correct window
for segment in &text_segments {
    if let Some(window_idx) = app_core.ui_state.stream_to_window(&segment.stream_id) {
        // Add segment to correct window
    }
}
```

**Affected Files**:
- `src/data/widget.rs` (add stream_id to TextSegment)
- `src/core/messages.rs` (verify stream ID assignment)
- `src/frontend/tui/text_window.rs` (filter by stream)
- `src/frontend/tui/mod.rs` (verify routing in render)
- NEW: `src/data/stream_routing.rs` (helper functions for stream→window mapping)

**Complexity**: Moderate (requires tracing text flow across multiple files)
**Dependencies**: Requires understanding of XML stream architecture
**GUI Port Impact**: Critical (stream awareness needed for GUI too)

**Acceptance Criteria**:
- Text in different streams (main, spells, combat) appears in correct windows
- No text loss during stream switching
- Test: Monitor spell casts, verify spell text appears in spell window
- Test: Monitor combat, verify combat text appears in combat window

---

### SPEC-TF-PARITY-WINDOW-PERSIST-001: Implement Window State Persistence

**Priority**: HIGH
**Status**: Regression (window positions may not survive restart)

**Rationale**:
Two-Face populates `UiState.window_config` at runtime but doesn't explicitly save it on shutdown. VellumFE saves window layouts to `~/.vellum-fe/layout/{character}`. Users expect their window arrangement to persist across restarts.

**Current State**:
- UiState: `src/data/ui_state.rs` contains `window_config: WindowConfig`
- Config: `src/config.rs` loads layout from TOML
- Missing: No save-on-exit logic for window state

**Proposed Solution**:
1. Create save function in Config:
```rust
// File: src/config.rs (ADD)
impl Config {
    pub fn save_window_state(&self, character: Option<&str>, window_config: &WindowConfig) -> Result<()> {
        // Create ~/.two-face/{character}/layouts/ if needed
        // Serialize WindowConfig to TOML
        // Save as layout_{timestamp}.toml or auto_{character}.toml
        Ok(())
    }
}
```

2. Call on shutdown in main.rs:
```rust
// File: src/main.rs (MODIFY event loop cleanup)
match signal {
    ControlFlow::Exit => {
        // Save window state before exit
        app_core.config.save_window_state(
            app_core.config.character.as_deref(),
            &app_core.ui_state.window_config
        )?;
        break;
    }
}
```

3. Load on startup:
```rust
// File: src/config.rs (MODIFY)
impl Layout {
    pub fn load_with_terminal_size(...) -> Result<(Self, Option<String>)> {
        // Priority: auto_{character}.toml → {character}.toml → ...
        // Already mostly implemented, just ensure window_config is loaded
    }
}
```

**Affected Files**:
- `src/config.rs` (add save_window_state method)
- `src/main.rs` (call save on exit)
- `src/data/window.rs` (ensure WindowConfig is serializable)

**Complexity**: Simple (mostly integration, pattern exists in VellumFE)
**Dependencies**: Requires proper shutdown handling in main event loop
**GUI Port Impact**: Neutral (both frontends need state persistence)

**Acceptance Criteria**:
- Window positions are saved on exit
- Window positions are restored on restart
- Test: Arrange windows, close app, restart, verify layout restored
- Test: Multiple characters have independent window layouts

---

## Category 2: Styling Consistency (SPEC-TF-STYLE)

### SPEC-TF-STYLE-UNIFICATION-001: Standardize Widget Styling via AppTheme

**Priority**: MEDIUM
**Status**: Many hardcoded colors, inconsistent theme usage

**Rationale**:
Two-Face has a comprehensive 77-field `AppTheme` but widgets use only ~15 fields and hardcode 50+ colors (cyan, yellow, white, etc.). This prevents consistent theme switching and makes GUI port harder. Standardizing on AppTheme improves visual consistency and enables dynamic theming.

**Current State**:
- Theme defined: `src/theme.rs` with 77 fields and 16 built-in themes
- Usage: PopupMenu uses 3-4 fields; HighlightForm, SettingsEditor, etc. hardcode colors
- Goal: 100% of widget colors come from AppTheme

**Proposed Solution**:
1. Create theme helper structs:
```rust
// File: src/theme.rs (ADD)
pub struct WidgetThemes {
    pub menu: MenuTheme,
    pub form: FormTheme,
    pub browser: BrowserTheme,
    pub table: TableTheme,
    pub editor: EditorTheme,
}

pub struct MenuTheme {
    pub border: Color,
    pub border_focused: Color,
    pub background: Color,
    pub item_normal: Color,
    pub item_selected: Color,
    pub item_focused: Color,
    pub separator: Color,
}

// Similar for FormTheme, BrowserTheme, TableTheme
```

2. Derive from AppTheme:
```rust
// File: src/theme.rs (MODIFY)
impl AppTheme {
    pub fn to_widget_themes(&self) -> WidgetThemes {
        WidgetThemes {
            menu: MenuTheme {
                border: self.menu_border,
                background: self.menu_background,
                item_selected: self.menu_item_selected,
                // ... etc
            },
            form: FormTheme {
                border: self.form_border,
                label: self.form_label,
                // ... etc
            },
        }
    }
}
```

3. Update each widget to accept theme:
```rust
// File: src/frontend/tui/popup_menu.rs (MODIFY)
pub struct PopupMenu {
    items: Vec<MenuItem>,
    selected: usize,
    position: (u16, u16),
    theme: MenuTheme,  // NEW
}

impl PopupMenu {
    pub fn new(items: Vec<MenuItem>, position: (u16, u16), theme: MenuTheme) -> Self {
        Self { items, selected: 0, position, theme }
    }

    pub fn render(&self) -> Vec<Span> {
        // Use self.theme.border, self.theme.item_selected, etc.
        // Instead of hardcoded Color::Cyan, Color::Yellow
    }
}
```

4. Pass theme through TUI frontend:
```rust
// File: src/frontend/tui/mod.rs (MODIFY)
impl TuiFrontend {
    fn render(&mut self, app: &mut dyn Any) -> Result<()> {
        let app_core = app.downcast_ref::<AppCore>()?;
        let themes = app_core.config.current_theme.to_widget_themes();

        // Pass themes to widgets:
        let popup = PopupMenu::new(items, pos, themes.menu);
        // ... render with themed widgets
    }
}
```

**Affected Files**:
- `src/theme.rs` (add WidgetThemes, MenuTheme, FormTheme, BrowserTheme, TableTheme, EditorTheme)
- `src/frontend/tui/popup_menu.rs` (accept MenuTheme)
- `src/frontend/tui/highlight_form.rs` (accept FormTheme)
- `src/frontend/tui/settings_editor.rs` (accept TableTheme)
- `src/frontend/tui/keybind_form.rs` (accept FormTheme)
- `src/frontend/tui/highlight_browser.rs` (accept BrowserTheme)
- `src/frontend/tui/color_palette_browser.rs` (accept BrowserTheme)
- `src/frontend/tui/window_editor.rs` (accept EditorTheme)
- `src/frontend/tui/theme_editor.rs` (accept EditorTheme)
- `src/frontend/tui/mod.rs` (pass themes to widgets during render)

**Complexity**: Moderate (systematic refactoring across 10+ widget files)
**Dependencies**: Requires understanding of theme system and all widgets
**GUI Port Impact**: Critical (GUI port needs same abstraction)

**Acceptance Criteria**:
- All hardcoded colors replaced with theme fields
- Theme switching (Ctrl+T) updates all widgets visually
- New custom themes are applied consistently
- Test: Create custom theme with distinctive colors, verify all widgets use it
- Test: Switch between built-in themes, verify consistent updates

---

### SPEC-TF-STYLE-FIELD-EXPANSION-001: Expand AppTheme for Complete Coverage

**Priority**: MEDIUM
**Status**: 77 fields insufficient, need ~16 additional fields

**Rationale**:
Gap analysis revealed missing theme fields:
- Menu: `menu_item_focused`, `menu_item_normal`, `menu_item_focused_bg`
- Form: `form_label_error`, `form_hint_color`
- Table: `table_header_bg`, `table_row_alternate_bg`, `table_row_hover_bg`
- Button: `button_focused`, `button_hover`
- Editor: `editor_selection_bg`, `editor_gutter_color`, `editor_line_number_color`

Adding these enables complete widget styling without hardcoding.

**Current State**:
- AppTheme: 77 fields in `src/theme.rs`
- Missing: 16 fields for complete widget coverage

**Proposed Solution**:
1. Add fields to AppTheme:
```rust
// File: src/theme.rs (MODIFY)
pub struct AppTheme {
    // ... existing 77 fields ...

    // NEW: Menu-specific
    pub menu_item_normal: Color,
    pub menu_item_focused: Color,
    pub menu_item_focused_bg: Color,

    // NEW: Form-specific
    pub form_label_error: Color,
    pub form_hint_color: Color,

    // NEW: Table/Browser
    pub table_header_bg: Color,
    pub table_row_alternate_bg: Color,
    pub table_row_hover_bg: Color,

    // NEW: Button
    pub button_focused: Color,
    pub button_hover: Color,

    // NEW: Editor
    pub editor_selection_bg: Color,
    pub editor_gutter_color: Color,
    pub editor_line_number_color: Color,

    // NEW: Status/Feedback
    pub success_color: Color,
    pub warning_color: Color,
    pub info_color: Color,
}
```

2. Update all 16 built-in themes with new field values:
```rust
// File: src/theme.rs (MODIFY ThemePresets)
fn dark() -> Self {
    AppTheme {
        // ... existing fields ...
        menu_item_normal: Color::White,
        menu_item_focused: Color::Cyan,
        menu_item_focused_bg: Color::DarkGray,
        form_label_error: Color::LightRed,
        form_hint_color: Color::Gray,
        table_header_bg: Color::DarkGray,
        table_row_alternate_bg: Color::Black,
        table_row_hover_bg: Color::DarkGray,
        button_focused: Color::Yellow,
        button_hover: Color::Cyan,
        editor_selection_bg: Color::DarkBlue,
        editor_gutter_color: Color::DarkGray,
        editor_line_number_color: Color::Gray,
        success_color: Color::Green,
        warning_color: Color::Yellow,
        info_color: Color::Cyan,
    }
}

// Repeat for all 15 other built-in themes (nord, dracula, etc.)
```

3. Update ThemeData struct for serialization:
```rust
// File: src/frontend/tui/theme_editor.rs (MODIFY)
pub struct ThemeData {
    // ... existing fields ...
    pub menu_item_normal: String,
    pub menu_item_focused: String,
    // ... etc for all 16 new fields
}
```

**Affected Files**:
- `src/theme.rs` (add 16 fields, update 16 preset themes)
- `src/frontend/tui/theme_editor.rs` (update ThemeData and theme editor UI)
- `src/theme.rs` get_color() method (add new field names)

**Complexity**: Simple (straightforward field additions, repetitive updates)
**Dependencies**: None (independent)
**GUI Port Impact**: Helpful (ensures theme system is comprehensive before GUI port)

**Acceptance Criteria**:
- All 16 built-in themes have values for new fields
- Custom themes serialize/deserialize all 93 fields
- Theme editor displays new fields in appropriate categories
- Test: Create custom theme, edit new fields, save, load, verify values

---

## Category 3: Architecture & GUI Readiness (SPEC-TF-CORE)

### SPEC-TF-CORE-INPUT-ABSTRACTION-001: Abstract TUI-Specific InputMode

**Priority**: HIGH (Blocks GUI Port)
**Status**: InputMode enum couples core to TUI concepts

**Rationale**:
`InputMode` enum in `src/data/ui_state.rs` contains TUI-specific concepts like `HighlightBrowser`, `SettingsEditor`, `WindowEditor`. GUI frontend needs different mode system. Abstracting this into a generic `EditorState` enables multi-frontend support.

**Current State**:
```rust
// File: src/data/ui_state.rs
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputMode {
    Normal,
    Navigation,
    Search,
    HighlightForm,      // TUI-specific
    HighlightBrowser,   // TUI-specific
    KeybindForm,        // TUI-specific
    SettingsEditor,     // TUI-specific
    WindowEditor,       // TUI-specific
    // ... 10+ TUI-specific modes
}
```

**Proposed Solution**:
1. Create frontend-agnostic EditorState:
```rust
// File: src/data/ui_state.rs (NEW enum)
pub enum EditorState {
    None,
    HighlightEditor { mode: EditorMode },  // Generic
    KeybindEditor { mode: EditorMode },
    SettingsEditor { mode: EditorMode },
    WindowEditor { mode: EditorMode },
    ThemeEditor { mode: EditorMode },
    ColorPaletteEditor { mode: EditorMode },
}

pub enum EditorMode {
    Browse,
    Edit,
    Create,
}
```

2. Keep InputMode for TUI internal use:
```rust
// File: src/frontend/tui/input_mode.rs (NEW - TUI-specific)
pub enum TuiInputMode {
    Normal,
    Search,
    HighlightForm,
    HighlightBrowser,
    // ... TUI-specific
}
```

3. Map between EditorState and TuiInputMode in TUI frontend:
```rust
// File: src/frontend/tui/mod.rs (NEW function)
fn editor_state_to_tui_mode(state: &EditorState) -> TuiInputMode {
    match state {
        EditorState::HighlightEditor { mode: EditorMode::Browse } => TuiInputMode::HighlightBrowser,
        EditorState::HighlightEditor { mode: EditorMode::Edit } => TuiInputMode::HighlightForm,
        EditorState::SettingsEditor { .. } => TuiInputMode::SettingsEditor,
        // ...
    }
}

fn tui_mode_to_editor_state(mode: &TuiInputMode) -> EditorState {
    match mode {
        TuiInputMode::HighlightBrowser => EditorState::HighlightEditor { mode: EditorMode::Browse },
        TuiInputMode::HighlightForm => EditorState::HighlightEditor { mode: EditorMode::Edit },
        // ...
    }
}
```

4. Update input routing:
```rust
// File: src/core/input_router.rs (MODIFY)
pub fn route_input(key: KeyEvent, editor_state: &EditorState, config: &Config) -> MenuAction {
    let context = match editor_state {
        EditorState::HighlightEditor { .. } => ActionContext::HighlightEditor,
        EditorState::KeybindEditor { .. } => ActionContext::KeybindEditor,
        EditorState::SettingsEditor { .. } => ActionContext::SettingsEditor,
        // ... map abstract states to contexts
    };
    config.menu_keybinds.resolve_action(key, context)
}
```

5. Update UiState:
```rust
// File: src/data/ui_state.rs (MODIFY)
pub struct UiState {
    pub editor_state: EditorState,  // NEW: frontend-agnostic
    pub input_mode: Option<TuiInputMode>,  // Optional: TUI-specific (will move to TUI frontend)
    // ... rest of fields
}
```

**Affected Files**:
- `src/data/ui_state.rs` (add EditorState enum, update UiState struct)
- NEW: `src/frontend/tui/input_mode.rs` (TUI-specific mode enum)
- `src/frontend/tui/mod.rs` (mapping functions, use EditorState)
- `src/core/input_router.rs` (accept EditorState instead of InputMode)
- `src/core/menu_actions.rs` (update to work with EditorState)

**Complexity**: Moderate (systematic refactoring of input handling)
**Dependencies**: Affects all input-related code paths
**GUI Port Impact**: Critical (enables GUI frontend to use own mode system)

**Acceptance Criteria**:
- EditorState used throughout core
- TUI frontend maps EditorState to TuiInputMode
- All input routing works with EditorState
- GUI stub can use own mode system independently
- Test: Verify all editor workflows work (highlight, keybind, settings, window, theme)
- Test: Verify GUI stub can override TuiInputMode without affecting core

---

### SPEC-TF-CORE-MODIFIER-ABSTRACTION-001: Abstract TUI Modifiers to Theme Layer

**Priority**: MEDIUM (Helpful for GUI Port)
**Status**: Widgets hardcode `Modifier::REVERSED`, `BOLD`, `UNDERLINE`

**Rationale**:
`Modifier` (REVERSED, BOLD, UNDERLINE) is ratatui-specific. GUI frontend would need different styling approach. Abstracting modifiers to theme layer enables GUI to interpret styling differently.

**Current State**:
```rust
// In widgets (popup_menu.rs, forms, etc.)
let selected_style = Style::new()
    .fg(Color::Yellow)
    .add_modifier(Modifier::REVERSED);  // TUI-specific

// In theme_editor.rs, highlight_form.rs, etc.
.add_modifier(Modifier::BOLD);  // Hardcoded
```

**Proposed Solution**:
1. Create abstract TextStyle:
```rust
// File: src/theme.rs (NEW enum)
pub enum TextStyle {
    Normal,
    Bold,
    Italic,
    Underline,
    Reversed,
    Dim,
    // Can add more as needed
}

pub struct StyledText {
    pub text: String,
    pub color: Option<Color>,
    pub background: Option<Color>,
    pub style: TextStyle,
}
```

2. Convert Modifier usage to TextStyle:
```rust
// File: src/frontend/tui/mod.rs (NEW function)
fn text_style_to_modifier(style: &TextStyle) -> Modifier {
    match style {
        TextStyle::Bold => Modifier::BOLD,
        TextStyle::Italic => Modifier::ITALIC,
        TextStyle::Underline => Modifier::UNDERLINED,
        TextStyle::Reversed => Modifier::REVERSED,
        TextStyle::Dim => Modifier::DIM,
        TextStyle::Normal => Modifier::empty(),
    }
}
```

3. Update widgets to use TextStyle:
```rust
// File: src/frontend/tui/popup_menu.rs (MODIFY)
pub struct PopupMenuTheme {
    pub border_color: Color,
    pub item_normal_color: Color,
    pub item_normal_style: TextStyle,  // NEW
    pub item_selected_color: Color,
    pub item_selected_style: TextStyle,  // NEW
}

// In render():
let modifier = text_style_to_modifier(&self.theme.item_selected_style);
let selected_style = Style::new()
    .fg(self.theme.item_selected_color)
    .add_modifier(modifier);
```

4. GUI frontend can interpret TextStyle differently:
```rust
// File: src/frontend/gui/mod.rs (Future implementation)
fn text_style_to_egui_richtext(style: &TextStyle, color: Color) -> egui::RichText {
    match style {
        TextStyle::Bold => egui::RichText::new(text).strong().color(color.to_egui()),
        TextStyle::Italic => egui::RichText::new(text).italics().color(color.to_egui()),
        // ... etc
    }
}
```

**Affected Files**:
- `src/theme.rs` (add TextStyle enum)
- `src/frontend/tui/mod.rs` (add conversion function)
- `src/frontend/tui/popup_menu.rs` (update PopupMenuTheme, render)
- `src/frontend/tui/highlight_form.rs` (use TextStyle)
- `src/frontend/tui/settings_editor.rs` (use TextStyle)
- `src/frontend/tui/keybind_form.rs` (use TextStyle)
- `src/frontend/tui/window_editor.rs` (use TextStyle)
- (and other widgets with modifiers)

**Complexity**: Moderate (systematic replacement of Modifier with TextStyle)
**Dependencies**: Requires SPEC-TF-STYLE-UNIFICATION-001 to be done first
**GUI Port Impact**: Critical (enables GUI styling without TUI dependencies)

**Acceptance Criteria**:
- All Modifier usage converted to TextStyle
- TUI frontend maps TextStyle to Modifier correctly
- GUI stub can implement own TextStyle mapping
- Visual appearance unchanged in TUI
- Test: Verify bold, underlined, reversed text appears correctly in TUI
- Test: Verify GUI stub can render styled text independently

---

### SPEC-TF-CORE-WIDGET-TRAIT-001: Create Widget Rendering Trait for Multi-Frontend

**Priority**: MEDIUM (Foundation for GUI Port)
**Status**: Widgets are ratatui-specific, no abstraction

**Rationale**:
Current widgets are tightly coupled to ratatui. To enable GUI port, we need a rendering trait that both TUI and GUI can implement. This creates the abstraction layer necessary for true multi-frontend support.

**Current State**:
```rust
// Widgets implement ratatui::widgets::Widget directly
impl Widget for PopupMenu { ... }
impl Widget for HighlightForm { ... }
impl Widget for SettingsEditor { ... }
// No abstraction for GUI
```

**Proposed Solution**:
1. Create abstract widget trait:
```rust
// File: src/frontend/widget_trait.rs (NEW)
pub trait RenderableWidget {
    /// Render to ratatui (TUI)
    fn render_tui(&self, area: Rect, buf: &mut Buffer);

    /// Render to egui (GUI) - future implementation
    fn render_gui(&self, ui: &mut egui::Ui) -> egui::Response;

    /// Get widget dimensions (width, height)
    fn get_size(&self) -> (u16, u16);

    /// Handle input event
    fn handle_event(&mut self, event: &crate::frontend::FrontendEvent) -> bool;

    /// Get focused element for accessibility
    fn get_focused_element(&self) -> Option<String>;
}
```

2. Implement for existing widgets:
```rust
// File: src/frontend/tui/popup_menu.rs (MODIFY)
impl Widget for PopupMenu { /* existing ratatui impl */ }

impl RenderableWidget for PopupMenu {
    fn render_tui(&self, area: Rect, buf: &mut Buffer) {
        // Call existing ratatui rendering
        Widget::render(self, area, buf);
    }

    fn render_gui(&self, ui: &mut egui::Ui) -> egui::Response {
        // Stub for GUI - will be implemented later
        unimplemented!("GUI implementation pending")
    }

    fn get_size(&self) -> (u16, u16) {
        // Return popup menu size
        (self.width, self.height)
    }

    fn handle_event(&mut self, event: &FrontendEvent) -> bool {
        match event {
            FrontendEvent::Key(k) => {
                self.handle_key(k);
                true
            }
            FrontendEvent::Mouse(_) => {
                // TODO: add mouse support
                false
            }
            _ => false,
        }
    }

    fn get_focused_element(&self) -> Option<String> {
        Some(format!("menu_item_{}", self.selected))
    }
}
```

3. Use in TUI frontend:
```rust
// File: src/frontend/tui/mod.rs (MODIFY)
impl TuiFrontend {
    fn render(&mut self, app: &mut dyn Any) -> Result<()> {
        let app_core = app.downcast_ref::<AppCore>()?;
        let theme = &app_core.config.current_theme;

        // Create renderable widgets
        let popup: Box<dyn RenderableWidget> = match app_core.ui_state.editor_state {
            EditorState::HighlightEditor { .. } => {
                Box::new(HighlightForm::new(/* ... */, theme))
            }
            EditorState::SettingsEditor { .. } => {
                Box::new(SettingsEditor::new(/* ... */, theme))
            }
            // ...
        };

        self.terminal.draw(|f| {
            popup.render_tui(f.size(), f.buffer_mut());
        })?;

        Ok(())
    }
}
```

4. Placeholder for GUI implementation:
```rust
// File: src/frontend/gui/mod.rs (MODIFY stub)
pub struct EguiApp {
    // Will implement RenderableWidget trait
}

impl EguiApp {
    pub fn new(app_core: AppCore) -> Self {
        Self { app_core }
    }

    pub fn render(&mut self, ctx: &egui::Context) {
        // For each widget in ui_state:
        match app_core.ui_state.editor_state {
            EditorState::HighlightEditor { .. } => {
                // Render as egui window
                let popup = Box::new(HighlightForm { /* ... */ });
                popup.render_gui(ui);
            }
        }
    }
}
```

**Affected Files**:
- NEW: `src/frontend/widget_trait.rs` (RenderableWidget trait)
- `src/frontend/mod.rs` (export RenderableWidget)
- `src/frontend/tui/popup_menu.rs` (implement RenderableWidget)
- `src/frontend/tui/highlight_form.rs` (implement RenderableWidget)
- `src/frontend/tui/settings_editor.rs` (implement RenderableWidget)
- `src/frontend/tui/keybind_form.rs` (implement RenderableWidget)
- `src/frontend/tui/window_editor.rs` (implement RenderableWidget)
- `src/frontend/tui/theme_editor.rs` (implement RenderableWidget)
- (All other widgets: ~30 files)
- `src/frontend/tui/mod.rs` (use RenderableWidget)
- `src/frontend/gui/mod.rs` (update stub with trait usage)

**Complexity**: High (requires implementing trait on 30+ widgets, but mechanical)
**Dependencies**: Should be done after SPEC-TF-STYLE-UNIFICATION-001 and SPEC-TF-CORE-INPUT-ABSTRACTION-001
**GUI Port Impact**: Critical (enables GUI stub to become real implementation)

**Acceptance Criteria**:
- RenderableWidget trait defined and documented
- All 30+ widgets implement RenderableWidget
- TUI frontend uses RenderableWidget trait
- GUI stub implements RenderableWidget (stubs returning unimplemented for GUI methods)
- Test: All widgets render correctly in TUI (no visual changes)
- Test: GUI stub builds without errors (unimplemented panics OK for now)
- Future: GUI implementation can fill in render_gui() methods incrementally

---

## Summary Table

| SPEC ID | Category | Title | Priority | Complexity | Est. Effort | GUI Impact |
|---------|----------|-------|----------|------------|-------------|-----------|
| SPEC-TF-PARITY-RESIZE-001 | Parity | Resize Debouncing | HIGH | Simple | 1-2 days | Helpful |
| SPEC-TF-PARITY-STREAM-HANDLING-001 | Parity | Stream Routing | HIGH | Moderate | 3-4 days | Critical |
| SPEC-TF-PARITY-WINDOW-PERSIST-001 | Parity | Window Persistence | HIGH | Simple | 1-2 days | Neutral |
| SPEC-TF-STYLE-UNIFICATION-001 | Style | Widget Theming | MEDIUM | Moderate | 4-5 days | Critical |
| SPEC-TF-STYLE-FIELD-EXPANSION-001 | Style | Expand AppTheme | MEDIUM | Simple | 2-3 days | Helpful |
| SPEC-TF-CORE-INPUT-ABSTRACTION-001 | Core | Abstract InputMode | HIGH | Moderate | 3-4 days | Critical |
| SPEC-TF-CORE-MODIFIER-ABSTRACTION-001 | Core | Abstract Modifiers | MEDIUM | Moderate | 4-5 days | Critical |
| SPEC-TF-CORE-WIDGET-TRAIT-001 | Core | Widget Rendering Trait | MEDIUM | High | 5-7 days | Critical |

**Total Estimated Effort**: 23-32 days (4-6 weeks for one developer, 2-3 weeks for team of 2-3)

---

## Implementation Priority Roadmap

**Phase 1: Parity Fixes (Week 1-2)** - Restore VellumFE equivalence
1. SPEC-TF-PARITY-RESIZE-001 (1-2 days)
2. SPEC-TF-PARITY-WINDOW-PERSIST-001 (1-2 days)
3. SPEC-TF-PARITY-STREAM-HANDLING-001 (3-4 days)

**Phase 2: Styling Foundation (Week 2-3)** - Prepare for GUI port
4. SPEC-TF-STYLE-FIELD-EXPANSION-001 (2-3 days)
5. SPEC-TF-STYLE-UNIFICATION-001 (4-5 days)

**Phase 3: Architecture Abstraction (Week 3-5)** - Enable GUI port
6. SPEC-TF-CORE-INPUT-ABSTRACTION-001 (3-4 days)
7. SPEC-TF-CORE-MODIFIER-ABSTRACTION-001 (4-5 days)
8. SPEC-TF-CORE-WIDGET-TRAIT-001 (5-7 days)

---

## Dependencies & Blocking Relationships

```
SPEC-TF-PARITY-RESIZE-001
SPEC-TF-PARITY-WINDOW-PERSIST-001
SPEC-TF-PARITY-STREAM-HANDLING-001 (independent)

SPEC-TF-STYLE-FIELD-EXPANSION-001 (independent)

SPEC-TF-STYLE-UNIFICATION-001
    ↑ Depends on: SPEC-TF-STYLE-FIELD-EXPANSION-001

SPEC-TF-CORE-INPUT-ABSTRACTION-001 (independent)

SPEC-TF-CORE-MODIFIER-ABSTRACTION-001
    ↑ Depends on: SPEC-TF-STYLE-UNIFICATION-001

SPEC-TF-CORE-WIDGET-TRAIT-001
    ↑ Depends on: SPEC-TF-STYLE-UNIFICATION-001, SPEC-TF-CORE-INPUT-ABSTRACTION-001
```

---

## Acceptance Criteria Summary

All SPECs must meet these criteria:

1. **Code Quality**
   - No regressions in existing functionality
   - All tests pass (existing + new)
   - Code follows project conventions

2. **Documentation**
   - Changes documented in code comments
   - Architectural changes documented
   - GUI port implications noted

3. **Testing**
   - Feature-specific tests pass
   - Integration tests verify workflows
   - GUI port stub still builds

4. **GUI Port Readiness**
   - TUI-specific assumptions removed or abstracted
   - GUI stub can implement independently
   - No new TUI-specific coupling introduced

---

**END OF RECOMMENDATIONS**
