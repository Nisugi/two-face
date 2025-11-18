# Egui Implementation Plan for Two-Face

**Version:** 1.0
**Date:** 2025-11-10
**Status:** Planning Phase

---

## Executive Summary

This document outlines the comprehensive plan for migrating two-face from a ratatui terminal UI to an egui-based GUI with full theming/skinning support. The migration will happen in phases, maintaining backward compatibility with the TUI during transition.

**Key Goals:**
1. Provide a modern GUI frontend using egui
2. Support comprehensive theming with shareable theme files
3. Recreate Wrayth client aesthetics (injury doll, custom chrome, textured backgrounds)
4. Enable runtime theme switching with hot-reload
5. Maintain feature parity with current TUI implementation

---

## 1. Architecture Overview

### 1.1 Application Structure

```
two-face/
├── src/
│   ├── main.rs                  # Entry point, frontend selection
│   ├── core/
│   │   ├── app_core.rs         # Shared game state (frontend-agnostic)
│   │   ├── game_state.rs       # Room, vitals, inventory, etc.
│   │   └── command_handler.rs  # Dot commands, game commands
│   ├── frontend/
│   │   ├── mod.rs              # Frontend trait
│   │   ├── tui/                # Ratatui TUI (current)
│   │   │   ├── mod.rs
│   │   │   └── app.rs
│   │   └── gui/                # Egui GUI (new)
│   │       ├── mod.rs
│   │       ├── app.rs
│   │       ├── widgets/
│   │       ├── theme.rs
│   │       └── assets.rs
│   ├── parser.rs               # XML parser (shared)
│   ├── network.rs              # Lich connection (shared)
│   └── config.rs               # Config management (shared)
└── assets/
    └── themes/
        ├── default/
        │   ├── theme.toml
        │   ├── textures/
        │   ├── fonts/
        │   └── sounds/
        └── wrayth-classic/
            ├── theme.toml
            ├── textures/
            │   ├── window_bg.png
            │   ├── wire_grate.png
            │   ├── injury_doll_base.png
            │   └── wound_overlays/
            └── fonts/
```

### 1.2 Frontend Abstraction

Define a `Frontend` trait to allow runtime selection between TUI and GUI:

```rust
pub trait Frontend {
    fn init(&mut self, core: Arc<Mutex<AppCore>>) -> Result<()>;
    fn run(&mut self) -> Result<()>;
    fn handle_game_event(&mut self, event: GameEvent);
    fn shutdown(&mut self) -> Result<()>;
}
```

**Command-line selection:**
```bash
two-face --frontend tui   # Current ratatui (default during transition)
two-face --frontend gui   # New egui GUI
```

### 1.3 Shared State Architecture

```rust
// Core game state (frontend-agnostic)
pub struct AppCore {
    pub vitals: Vitals,
    pub room: RoomState,
    pub inventory: Inventory,
    pub active_effects: Vec<Effect>,
    pub text_buffers: HashMap<String, TextBuffer>,
    pub menu_state: MenuState,
    pub link_cache: LinkCache,
    // ... etc
}

// GUI-specific state
pub struct EguiApp {
    core: Arc<Mutex<AppCore>>,
    theme: Theme,
    theme_loader: ThemeLoader,
    window_manager: GuiWindowManager,
    asset_cache: AssetCache,
    // ... GUI-specific state
}
```

---

## 2. Layout System Design

### 2.1 Layout Structure (Inspired by Wrayth)

**Wrayth's Layout Approach:**
- Central main window area (primary game text)
- Collapsible left/right sidebars
- Dockable widgets to sidebars (can hide/show)
- Free positioning and resizing in main area with snap-together behavior
- Detachable windows (can pop outside main window)

**Our Egui Layout:**

```
┌──────────────────────────────────────────────┐
│  Left Sidebar    │  Central Area  │  Right   │
│  (egui_dock)     │  (egui_dock)   │  Sidebar │
│  ┌──────────┐    │  ┌──────────┐  │ ┌──────┐│
│  │Tab│Tab│  │    │  │Main Win  │  │ │Vitals││
│  │───┴───┤  │    │  │          │  │ └──────┘│
│  │Content│  │    │  │  [tabs]  │  │ ┌──────┐│
│  └────────┘  │    │  └──────────┘  │ │Timer ││
│  [collapsible]    │  [splits/tabs] │ └──────┘│
└──────────────────────────────────────────────┘
         ↓ (can detach any window)
    ┌─────────────────┐
    │ OS Window       │  ← Detached windows
    │ (multi-monitor) │
    └─────────────────┘
```

### 2.2 Docking System

Using **egui_dock** crate for mature docking support:

```rust
use egui_dock::{DockArea, DockState, NodeIndex, Style};

pub struct LayoutManager {
    left_sidebar: SidebarState,
    right_sidebar: SidebarState,
    central_area: DockState<WindowWidget>,
    detached_windows: Vec<DetachedWindow>,
}

pub struct SidebarState {
    visible: bool,
    width: f32,
    dock_tree: DockState<WindowWidget>,
}

impl LayoutManager {
    pub fn render(&mut self, ctx: &egui::Context) {
        // Left sidebar (collapsible)
        egui::SidePanel::left("left_sidebar")
            .resizable(true)
            .show_animated(ctx, self.left_sidebar.visible, |ui| {
                DockArea::new(&mut self.left_sidebar.dock_tree).show_inside(ui, &mut TabViewer);
            });

        // Right sidebar (collapsible)
        egui::SidePanel::right("right_sidebar")
            .resizable(true)
            .show_animated(ctx, self.right_sidebar.visible, |ui| {
                DockArea::new(&mut self.right_sidebar.dock_tree).show_inside(ui, &mut TabViewer);
            });

        // Central panel (always visible)
        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.central_area).show_inside(ui, &mut TabViewer);
        });

        // Detached windows
        for window in &mut self.detached_windows {
            window.render(ctx);
        }
    }
}
```

**Key Features:**
- **Tabs everywhere** - Any window can be tabbed with any other
- **Splits** - Split any area horizontally or vertically (infinite nesting)
- **Drag between areas** - Drag from sidebar to central or vice versa
- **Visual feedback** - Shows where tab will dock when dragging

### 2.3 Detached Windows

```rust
pub struct DetachedWindow {
    id: String,
    widget: Box<dyn Widget>,
    viewport_id: ViewportId,
    position: Option<Pos2>,
    size: Option<Vec2>,
}

impl DetachedWindow {
    pub fn render(&mut self, ctx: &egui::Context) {
        ctx.show_viewport_immediate(
            self.viewport_id,
            egui::ViewportBuilder::default()
                .with_title(&self.id)
                .with_inner_size(self.size.unwrap_or(vec2(800.0, 600.0))),
            |ctx, _class| {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.widget.render(ui);

                    // Right-click context menu
                    ui.interact(ui.max_rect(), ui.id(), Sense::click())
                        .context_menu(|ui| {
                            if ui.button("Re-dock").clicked() {
                                // Signal to re-dock this window
                            }
                        });
                });
            },
        );
    }
}
```

### 2.4 Improvements Over Wrayth

#### Smart Layouts / Perspectives
```rust
pub struct LayoutPerspective {
    name: String,
    description: String,
    dock_state_central: DockState<WindowWidget>,
    dock_state_left: Option<DockState<WindowWidget>>,
    dock_state_right: Option<DockState<WindowWidget>>,
    left_visible: bool,
    right_visible: bool,
}

// Predefined perspectives:
// - "Combat" - Vitals prominent, countdown timers large, minimal chat
// - "Social" - Chat tabs prominent, story window large
// - "Crafting" - Inventory, hands, room window focused
// - "Hunting" - Loot, experience, active effects prominent
// - "Minimal" - Just main window and vitals
```

#### Window Templates
- Quick "Add Window" menu with categories
- Text Windows (Main, Thoughts, Speech, Deaths, etc.)
- Vitals (Health, Mana, Stamina bars)
- Status (Compass, Hands, Active Effects)
- Advanced (Injury Doll, Map, Dashboard)
- Drag from template palette directly to docking area

#### Layout Sharing
```toml
# layouts/combat_optimized.toml
[meta]
name = "Combat Optimized"
author = "PlayerName"
version = "1.0"
description = "Maximized for hunting efficiency"

[central]
split = "horizontal"
ratio = 0.7
# ... full dock tree serialization
```

#### Better Sidebar Behavior
- **Pin/unpin tabs** - Pinned tabs always visible, unpinned auto-hide
- **Accordion-style collapse** - Multiple widgets, expand one at a time
- **Quick-peek on hover** - Sidebar collapsed, hover shows temporary overlay
- **Orientation control** - Horizontal or vertical tab labels
- **Mini-mode** - Icon-only buttons when collapsed

#### Advanced Docking Features
- **Snap zones** - Visual indicators showing where window will dock
- **Tab groups** - Save groups of tabs as named sets
- **Locked layouts** - Lock to prevent accidental changes
- **Floating mode** - Windows float within app (not OS-level detached)
- **Picture-in-picture** - Small overlay windows always on top

### 2.5 Layout Persistence

```rust
#[derive(Serialize, Deserialize)]
pub struct SavedLayout {
    pub name: String,
    pub meta: LayoutMeta,
    pub sidebars: SidebarLayout,
    pub central: DockTreeLayout,
    pub detached: Vec<DetachedWindowLayout>,
}

// Serialize egui_dock's DockState to TOML
// Save per-character: ~/.two-face/layouts/<character>_<name>.toml
// Auto-save on exit to auto_<character>.toml
// Quick-save/quick-load keybinds
```

### 2.6 Context Menus on Window Tabs

```rust
// Context menu on any window tab:
// - "Detach Window" (pop out to OS window)
// - "Move to Left Sidebar"
// - "Move to Right Sidebar"
// - "Move to Center"
// - "Split Horizontally"
// - "Split Vertically"
// - "Close Tab"
// - "Close All Other Tabs"
// - "Lock Position"
```

---

## 3. Theme System Design

### 3.1 Theme Structure

```rust
/// Complete theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub meta: ThemeMeta,
    pub palette: ThemePalette,
    pub typography: ThemeTypography,
    pub textures: ThemeTextures,
    pub window_chrome: WindowChrome,
    pub widgets: WidgetPalette,
    pub injury_doll: Option<InjuryDollTheme>,
    pub hud: HudTheme,
    pub audio_visual: AudioVisualExtras,
}

/// Theme metadata and inheritance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeMeta {
    pub name: String,
    pub author: String,
    pub version: String,
    pub description: String,
    pub inherits_from: Option<String>, // e.g., "default"
}

/// Color palette
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemePalette {
    // Base colors
    pub background: Color32,
    pub foreground: Color32,
    pub primary: Color32,
    pub secondary: Color32,
    pub accent: Color32,

    // Semantic colors
    pub success: Color32,
    pub warning: Color32,
    pub error: Color32,
    pub info: Color32,

    // UI element colors
    pub window_background: Color32,
    pub window_border: Color32,
    pub window_title_bar: Color32,
    pub window_title_text: Color32,

    pub button_default: Color32,
    pub button_hovered: Color32,
    pub button_pressed: Color32,
    pub button_text: Color32,

    pub input_background: Color32,
    pub input_border: Color32,
    pub input_text: Color32,
    pub input_cursor: Color32,

    pub selection_background: Color32,
    pub selection_foreground: Color32,

    pub link_color: Color32,
    pub link_hovered: Color32,

    // Game-specific colors
    pub health_full: Color32,
    pub health_low: Color32,
    pub mana_full: Color32,
    pub mana_low: Color32,
    pub stamina_full: Color32,
    pub stamina_low: Color32,

    pub command_echo: Color32,
    pub system_message: Color32,
}

/// Typography settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTypography {
    pub main_font: FontConfig,
    pub monospace_font: FontConfig,
    pub title_font: FontConfig,
    pub small_font: FontConfig,

    pub default_size: f32,
    pub title_size: f32,
    pub small_size: f32,

    pub line_spacing: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub path: Option<String>, // Relative to theme directory
    pub fallbacks: Vec<String>,
}

/// Texture/image assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeTextures {
    pub window_background: Option<TextureConfig>,
    pub window_border: Option<TextureConfig>,
    pub panel_background: Option<TextureConfig>,
    pub button_normal: Option<TextureConfig>,
    pub button_hovered: Option<TextureConfig>,
    pub button_pressed: Option<TextureConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureConfig {
    pub path: String, // Relative to theme directory
    pub tint: Option<Color32>,
    pub opacity: f32,
    pub scale_mode: ScaleMode, // Stretch, Tile, NineSlice
    pub nine_slice: Option<NineSliceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScaleMode {
    Stretch,
    Tile,
    NineSlice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NineSliceConfig {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

/// Window chrome styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowChrome {
    pub border_width: f32,
    pub border_radius: f32,
    pub title_bar_height: f32,
    pub title_bar_padding: f32,
    pub shadow_enabled: bool,
    pub shadow_offset: (f32, f32),
    pub shadow_blur: f32,
    pub shadow_color: Color32,
    pub resize_handle_size: f32,
}

/// Widget-specific styling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPalette {
    pub progress_bar: ProgressBarStyle,
    pub countdown: CountdownStyle,
    pub compass: CompassStyle,
    pub hands: HandsStyle,
    pub text_window: TextWindowStyle,
    pub tabbed_window: TabbedWindowStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressBarStyle {
    pub height: f32,
    pub border_width: f32,
    pub border_color: Color32,
    pub background_color: Color32,
    pub fill_color: Color32,
    pub text_color: Color32,
    pub show_text: bool,
    pub show_percentage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountdownStyle {
    pub height: f32,
    pub border_width: f32,
    pub border_color: Color32,
    pub background_color: Color32,
    pub fill_color: Color32,
    pub text_color: Color32,
    pub roundtime_color: Color32,
    pub casttime_color: Color32,
    pub stun_color: Color32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompassStyle {
    pub active_color: Color32,
    pub inactive_color: Color32,
    pub background_color: Color32,
    pub border_color: Color32,
    pub text_color: Color32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandsStyle {
    pub background_color: Color32,
    pub border_color: Color32,
    pub text_color: Color32,
    pub left_label_color: Color32,
    pub right_label_color: Color32,
    pub empty_color: Color32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextWindowStyle {
    pub background_color: Color32,
    pub background_texture: Option<String>, // Path to texture
    pub border_width: f32,
    pub border_color: Color32,
    pub text_color: Color32,
    pub padding: f32,
    pub scrollbar_width: f32,
    pub scrollbar_color: Color32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabbedWindowStyle {
    pub tab_bar_height: f32,
    pub tab_padding: f32,
    pub tab_active_color: Color32,
    pub tab_inactive_color: Color32,
    pub tab_hovered_color: Color32,
    pub tab_text_active: Color32,
    pub tab_text_inactive: Color32,
    pub tab_unread_indicator: String,
    pub tab_unread_color: Color32,
}

/// Injury doll theming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjuryDollTheme {
    pub base_image: String, // Path to base doll image
    pub wound_overlays: WoundOverlays,
    pub scar_overlays: ScarOverlays,
    pub body_part_regions: HashMap<String, BodyPartRegion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WoundOverlays {
    pub rank1: HashMap<String, String>, // body_part -> image path
    pub rank2: HashMap<String, String>,
    pub rank3: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScarOverlays {
    pub rank1: HashMap<String, String>,
    pub rank2: HashMap<String, String>,
    pub rank3: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyPartRegion {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

/// HUD/Dashboard theming
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudTheme {
    pub vitals_bar_spacing: f32,
    pub vitals_bar_height: f32,
    pub timer_height: f32,
    pub show_icons: bool,
    pub icon_size: f32,
}

/// Audio and visual extras
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioVisualExtras {
    pub enable_animations: bool,
    pub animation_speed: f32,
    pub enable_particles: bool,
    pub enable_shadows: bool,
    pub enable_blur: bool,
}
```

### 3.2 Theme File Format (TOML)

Example `theme.toml`:

```toml
[meta]
name = "Wrayth Classic"
author = "Two-Face Team"
version = "1.0.0"
description = "Recreation of classic Wrayth visual style"
inherits_from = "default"

[palette]
background = "#1a1a1a"
foreground = "#e0e0e0"
primary = "#4a90e2"
accent = "#f39c12"

window_background = "#2d2d2d"
window_border = "#4a4a4a"
window_title_bar = "#3a3a3a"
window_title_text = "#ffffff"

link_color = "#5dade2"
link_hovered = "#85c1e9"

health_full = "#27ae60"
health_low = "#e74c3c"
mana_full = "#3498db"
mana_low = "#9b59b6"

[typography]
default_size = 14.0
title_size = 16.0
line_spacing = 1.2

[typography.main_font]
family = "Consolas"
fallbacks = ["Courier New", "monospace"]

[typography.title_font]
family = "Arial"
fallbacks = ["Helvetica", "sans-serif"]

[textures.window_background]
path = "textures/wire_grate.png"
opacity = 0.15
scale_mode = "Tile"

[textures.panel_background]
path = "textures/dark_metal.png"
opacity = 0.3
scale_mode = "NineSlice"

[textures.panel_background.nine_slice]
left = 16.0
right = 16.0
top = 16.0
bottom = 16.0

[window_chrome]
border_width = 2.0
border_radius = 4.0
title_bar_height = 28.0
shadow_enabled = true
shadow_offset = [0.0, 2.0]
shadow_blur = 8.0
shadow_color = "#00000080"

[widgets.progress_bar]
height = 24.0
border_width = 1.0
border_color = "#4a4a4a"
show_percentage = true

[widgets.compass]
active_color = "#00ff00"
inactive_color = "#333333"

[widgets.text_window]
background_texture = "textures/wire_grate.png"
padding = 8.0
scrollbar_width = 12.0

[widgets.tabbed_window]
tab_bar_height = 30.0
tab_padding = 12.0
tab_unread_indicator = "* "

[injury_doll]
base_image = "injury_doll/base.png"

[injury_doll.wound_overlays.rank1]
head = "injury_doll/wounds/head_1.png"
chest = "injury_doll/wounds/chest_1.png"
abdomen = "injury_doll/wounds/abdomen_1.png"
# ... etc

[injury_doll.body_part_regions.head]
x = 60.0
y = 10.0
width = 40.0
height = 40.0

[hud]
vitals_bar_spacing = 4.0
vitals_bar_height = 24.0
show_icons = true
icon_size = 20.0

[audio_visual]
enable_animations = true
animation_speed = 1.0
enable_shadows = true
```

### 3.3 Theme Loader Implementation

```rust
pub struct ThemeLoader {
    themes_dir: PathBuf,
    cache: HashMap<String, Theme>,
    current_theme: String,
}

impl ThemeLoader {
    pub fn new(themes_dir: PathBuf) -> Self {
        Self {
            themes_dir,
            cache: HashMap::new(),
            current_theme: "default".to_string(),
        }
    }

    /// Load a theme by name (with inheritance support)
    pub fn load_theme(&mut self, name: &str) -> Result<Theme> {
        // Check cache first
        if let Some(theme) = self.cache.get(name) {
            return Ok(theme.clone());
        }

        let theme_path = self.themes_dir.join(name).join("theme.toml");
        let toml_str = std::fs::read_to_string(&theme_path)?;
        let mut theme: Theme = toml::from_str(&toml_str)?;

        // Handle inheritance
        if let Some(parent_name) = &theme.meta.inherits_from {
            let parent = self.load_theme(parent_name)?;
            theme = merge_themes(parent, theme);
        }

        // Resolve relative paths
        self.resolve_asset_paths(&mut theme, name)?;

        // Cache and return
        self.cache.insert(name.to_string(), theme.clone());
        Ok(theme)
    }

    /// Hot-reload current theme
    pub fn reload_current(&mut self) -> Result<Theme> {
        self.cache.remove(&self.current_theme);
        self.load_theme(&self.current_theme)
    }

    /// List available themes
    pub fn list_themes(&self) -> Result<Vec<String>> {
        let mut themes = Vec::new();
        for entry in std::fs::read_dir(&self.themes_dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                themes.push(name);
            }
        }
        Ok(themes)
    }

    fn resolve_asset_paths(&self, theme: &mut Theme, theme_name: &str) -> Result<()> {
        let theme_dir = self.themes_dir.join(theme_name);

        // Resolve texture paths
        if let Some(ref mut tex) = theme.textures.window_background {
            tex.path = theme_dir.join(&tex.path).to_string_lossy().to_string();
        }
        // ... resolve all other texture paths

        // Resolve font paths
        if let Some(ref mut path) = theme.typography.main_font.path {
            *path = theme_dir.join(&path).to_string_lossy().to_string();
        }
        // ... resolve all other font paths

        // Resolve injury doll paths
        if let Some(ref mut doll) = theme.injury_doll {
            doll.base_image = theme_dir.join(&doll.base_image).to_string_lossy().to_string();
            // ... resolve all overlay paths
        }

        Ok(())
    }
}

/// Merge child theme over parent theme (child values override parent)
fn merge_themes(parent: Theme, child: Theme) -> Theme {
    // Deep merge logic - child overrides parent for each field
    // Use derive_more or manual implementation
    Theme {
        meta: child.meta,
        palette: merge_palette(parent.palette, child.palette),
        typography: merge_typography(parent.typography, child.typography),
        // ... etc
    }
}
```

### 3.4 Applying Theme to Egui

```rust
impl EguiApp {
    pub fn apply_theme(&mut self, ctx: &egui::Context) {
        let theme = &self.theme;

        // Apply to egui Style
        let mut style = (*ctx.style()).clone();
        let mut visuals = style.visuals.clone();

        // Colors
        visuals.override_text_color = Some(theme.palette.foreground);
        visuals.window_fill = theme.palette.window_background;
        visuals.window_stroke = egui::Stroke::new(
            theme.window_chrome.border_width,
            theme.palette.window_border,
        );

        // Widgets
        visuals.widgets.noninteractive.bg_fill = theme.palette.button_default;
        visuals.widgets.inactive.bg_fill = theme.palette.button_default;
        visuals.widgets.hovered.bg_fill = theme.palette.button_hovered;
        visuals.widgets.active.bg_fill = theme.palette.button_pressed;

        // Selection
        visuals.selection.bg_fill = theme.palette.selection_background;
        visuals.selection.stroke = egui::Stroke::new(1.0, theme.palette.selection_foreground);

        // Window rounding
        visuals.window_rounding = egui::Rounding::same(theme.window_chrome.border_radius);

        style.visuals = visuals;
        ctx.set_style(style);

        // Load fonts
        let mut fonts = egui::FontDefinitions::default();

        // Add custom fonts from theme
        if let Some(ref font_path) = theme.typography.main_font.path {
            if let Ok(font_data) = std::fs::read(font_path) {
                fonts.font_data.insert(
                    "main_font".to_owned(),
                    egui::FontData::from_owned(font_data),
                );
                fonts.families.insert(
                    egui::FontFamily::Proportional,
                    vec!["main_font".to_owned()],
                );
            }
        }

        ctx.set_fonts(fonts);

        // Set default text styles
        let mut text_styles = std::collections::BTreeMap::new();
        text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::proportional(theme.typography.default_size),
        );
        text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::proportional(theme.typography.title_size),
        );
        text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::proportional(theme.typography.small_size),
        );

        ctx.set_style(egui::Style {
            text_styles,
            ..(*ctx.style()).clone()
        });
    }
}
```

---

## 4. Widget Migration Strategy

### 4.1 Widget Priority Order

**Phase 1: Core Widgets (MVP)**
1. TextWindow - Main game text display
2. CommandInput - Command entry
3. ProgressBar - Vitals (health, mana, stamina, etc.)
4. Countdown - Roundtime/casttime

**Phase 2: Enhanced Widgets**
5. TabbedWindow - Chat tabs (speech, thoughts, etc.)
6. Compass - Direction navigation
7. Hands - Left/right hand display
8. RoomWindow - Room description with exits

**Phase 3: Advanced Widgets**
9. InjuryDoll - Layered wound/scar visualization
10. Dashboard - Comprehensive status display
11. ActiveEffects - Buffs/debuffs
12. Map - Local area navigation

### 4.2 TextWindow Implementation

```rust
pub struct GuiTextWindow {
    name: String,
    buffer: Vec<StyledLine>,
    scroll_offset: usize,
    max_lines: usize,
    theme_style: TextWindowStyle,
    background_texture: Option<egui::TextureHandle>,
}

impl GuiTextWindow {
    pub fn render(&mut self, ui: &mut egui::Ui, theme: &Theme) {
        // Apply background texture if configured
        if let Some(ref texture) = self.background_texture {
            self.render_background(ui, texture, &theme.widgets.text_window);
        }

        // Create scrollable area
        egui::ScrollArea::vertical()
            .max_height(ui.available_height())
            .show(ui, |ui| {
                ui.style_mut().spacing.item_spacing.y = theme.typography.line_spacing;

                for line in self.buffer.iter().skip(self.scroll_offset) {
                    self.render_line(ui, line, theme);
                }
            });
    }

    fn render_background(&self, ui: &mut egui::Ui, texture: &egui::TextureHandle, style: &TextWindowStyle) {
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter();

        match style.background_texture.as_ref().map(|t| &t.scale_mode) {
            Some(ScaleMode::Tile) => {
                // Tile texture across background
                let tex_size = texture.size_vec2();
                for y in (rect.min.y as i32..rect.max.y as i32).step_by(tex_size.y as usize) {
                    for x in (rect.min.x as i32..rect.max.x as i32).step_by(tex_size.x as usize) {
                        let tile_rect = egui::Rect::from_min_size(
                            egui::pos2(x as f32, y as f32),
                            tex_size,
                        );
                        painter.image(
                            texture.id(),
                            tile_rect,
                            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                            egui::Color32::WHITE.linear_multiply(style.background_texture.as_ref().unwrap().opacity),
                        );
                    }
                }
            },
            Some(ScaleMode::Stretch) => {
                // Stretch texture to fill
                painter.image(
                    texture.id(),
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE.linear_multiply(style.background_texture.as_ref().unwrap().opacity),
                );
            },
            Some(ScaleMode::NineSlice) => {
                // Nine-slice scaling for borders
                // (more complex implementation)
            },
            None => {
                // Solid color background
                painter.rect_filled(rect, 0.0, style.background_color);
            }
        }
    }

    fn render_line(&self, ui: &mut egui::Ui, line: &StyledLine, theme: &Theme) {
        ui.horizontal(|ui| {
            for segment in &line.segments {
                let mut job = egui::text::LayoutJob::default();

                // Build rich text with colors and styles
                let color = segment.fg_color.unwrap_or(theme.palette.foreground);
                let font_id = if segment.bold {
                    egui::FontId::proportional(theme.typography.default_size * 1.1)
                } else {
                    egui::FontId::proportional(theme.typography.default_size)
                };

                job.append(
                    &segment.text,
                    0.0,
                    egui::text::TextFormat {
                        font_id,
                        color,
                        ..Default::default()
                    },
                );

                // Handle links
                if let Some(ref link_data) = segment.link_data {
                    ui.hyperlink_to(&segment.text, "");
                    if ui.response().clicked() {
                        // Handle link click -> show context menu
                    }
                } else {
                    ui.label(job);
                }
            }
        });
    }
}
```

### 4.3 InjuryDoll Implementation (Layered Rendering)

```rust
pub struct GuiInjuryDoll {
    theme: InjuryDollTheme,
    base_texture: egui::TextureHandle,
    wound_textures: HashMap<String, HashMap<u8, egui::TextureHandle>>, // body_part -> rank -> texture
    scar_textures: HashMap<String, HashMap<u8, egui::TextureHandle>>,
    current_injuries: HashMap<String, (u8, bool)>, // body_part -> (rank, is_scar)
}

impl GuiInjuryDoll {
    pub fn render(&mut self, ui: &mut egui::Ui) {
        let (rect, response) = ui.allocate_exact_size(
            egui::vec2(200.0, 300.0),
            egui::Sense::hover(),
        );

        let painter = ui.painter();

        // Layer 1: Base doll image
        painter.image(
            self.base_texture.id(),
            rect,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            egui::Color32::WHITE,
        );

        // Layer 2: Wound overlays (ordered by severity)
        for (body_part, (rank, is_scar)) in &self.current_injuries {
            if *is_scar {
                if let Some(texture) = self.scar_textures.get(body_part).and_then(|m| m.get(rank)) {
                    let region = &self.theme.body_part_regions[body_part];
                    let overlay_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + region.x, rect.min.y + region.y),
                        egui::vec2(region.width, region.height),
                    );
                    painter.image(
                        texture.id(),
                        overlay_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            } else {
                if let Some(texture) = self.wound_textures.get(body_part).and_then(|m| m.get(rank)) {
                    let region = &self.theme.body_part_regions[body_part];
                    let overlay_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + region.x, rect.min.y + region.y),
                        egui::vec2(region.width, region.height),
                    );
                    painter.image(
                        texture.id(),
                        overlay_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
            }
        }

        // Tooltip on hover
        if response.hovered() {
            if let Some(body_part) = self.get_body_part_at_position(response.hover_pos().unwrap(), rect) {
                if let Some((rank, is_scar)) = self.current_injuries.get(&body_part) {
                    let severity = match rank {
                        1 => "Minor",
                        2 => "Moderate",
                        3 => "Severe",
                        _ => "Unknown",
                    };
                    let injury_type = if *is_scar { "Scar" } else { "Wound" };
                    egui::show_tooltip_text(
                        ui.ctx(),
                        egui::Id::new("injury_tooltip"),
                        format!("{}: {} {}", body_part, severity, injury_type),
                    );
                }
            }
        }
    }

    fn get_body_part_at_position(&self, pos: egui::Pos2, base_rect: egui::Rect) -> Option<String> {
        let rel_x = pos.x - base_rect.min.x;
        let rel_y = pos.y - base_rect.min.y;

        for (body_part, region) in &self.theme.body_part_regions {
            if rel_x >= region.x && rel_x <= region.x + region.width
                && rel_y >= region.y && rel_y <= region.y + region.height
            {
                return Some(body_part.clone());
            }
        }
        None
    }
}
```

---

## 5. Asset Management

### 5.1 Asset Cache

```rust
pub struct AssetCache {
    textures: HashMap<String, egui::TextureHandle>,
    fonts: HashMap<String, egui::FontData>,
    sounds: HashMap<String, Vec<u8>>,
}

impl AssetCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
            fonts: HashMap::new(),
            sounds: HashMap::new(),
        }
    }

    /// Load texture from path (cached)
    pub fn load_texture(
        &mut self,
        ctx: &egui::Context,
        path: &str,
    ) -> Result<egui::TextureHandle> {
        if let Some(texture) = self.textures.get(path) {
            return Ok(texture.clone());
        }

        let image = image::open(path)?;
        let size = [image.width() as usize, image.height() as usize];
        let pixels = image.to_rgba8().into_raw();

        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
        let texture = ctx.load_texture(path, color_image, Default::default());

        self.textures.insert(path.to_string(), texture.clone());
        Ok(texture)
    }

    /// Preload all theme assets
    pub fn preload_theme(&mut self, ctx: &egui::Context, theme: &Theme) -> Result<()> {
        // Preload all textures referenced in theme
        if let Some(ref tex_cfg) = theme.textures.window_background {
            self.load_texture(ctx, &tex_cfg.path)?;
        }

        if let Some(ref doll) = theme.injury_doll {
            self.load_texture(ctx, &doll.base_image)?;

            // Load all wound overlays
            for overlays in [&doll.wound_overlays.rank1, &doll.wound_overlays.rank2, &doll.wound_overlays.rank3] {
                for path in overlays.values() {
                    self.load_texture(ctx, path)?;
                }
            }

            // Load all scar overlays
            for overlays in [&doll.scar_overlays.rank1, &doll.scar_overlays.rank2, &doll.scar_overlays.rank3] {
                for path in overlays.values() {
                    self.load_texture(ctx, path)?;
                }
            }
        }

        Ok(())
    }

    /// Clear all cached assets (for theme reload)
    pub fn clear(&mut self) {
        self.textures.clear();
        self.fonts.clear();
        self.sounds.clear();
    }
}
```

### 5.2 Resource Bundling

For distribution, embed default theme in binary:

```rust
// In theme.rs
pub fn get_embedded_default_theme() -> Theme {
    const DEFAULT_THEME_TOML: &str = include_str!("../../assets/themes/default/theme.toml");
    toml::from_str(DEFAULT_THEME_TOML).expect("Failed to parse embedded default theme")
}

pub fn get_embedded_texture(name: &str) -> Option<&'static [u8]> {
    match name {
        "wire_grate.png" => Some(include_bytes!("../../assets/themes/default/textures/wire_grate.png")),
        "injury_doll_base.png" => Some(include_bytes!("../../assets/themes/default/textures/injury_doll_base.png")),
        // ... etc
        _ => None,
    }
}
```

---

## 6. Implementation Phases & Milestones

### Phase 1: Foundation & Basic Layout (2-3 weeks)

#### Milestone 1.1: Egui Skeleton + Frontend Abstraction
**Goal:** Get egui running alongside TUI with CLI selection

**Deliverables:**
- [ ] Create `src/frontend/mod.rs` with `Frontend` trait
- [ ] Move current TUI code to `src/frontend/tui/`
- [ ] Create `src/frontend/gui/mod.rs` stub
- [ ] Implement `EguiApp` struct with basic event loop
- [ ] Add CLI flag: `--frontend [tui|gui]` (default: tui)
- [ ] Connect EguiApp to AppCore via `Arc<Mutex<AppCore>>`

**Test:** Launch with `--frontend gui`, see empty egui window

#### Milestone 1.2: Basic Layout Structure
**Goal:** Implement collapsible sidebars and central panel

**Deliverables:**
- [ ] Integrate `egui_dock` crate (add to Cargo.toml)
- [ ] Create `src/frontend/gui/layout_manager.rs`:
  - `LayoutManager` struct
  - `SidebarState` struct (left/right)
  - Basic panel rendering (left sidebar, right sidebar, central)
- [ ] Implement sidebar collapse buttons (◀ / ▶)
- [ ] Make sidebars resizable (drag divider)
- [ ] Save/load sidebar widths and visibility to config

**Test:** Toggle sidebars, resize, restart app and verify saved state

#### Milestone 1.3: Single Text Window Display
**Goal:** Display main game text in central area

**Deliverables:**
- [ ] Create `src/frontend/gui/widgets/mod.rs`
- [ ] Implement `GuiTextWindow` (basic version):
  - Pull text from `AppCore.text_buffers["main"]`
  - Render with `egui::ScrollArea`
  - Display plain text (no colors yet)
- [ ] Add GuiTextWindow to central DockArea
- [ ] Connect to Lich server
- [ ] Display incoming game text

**Test:** Launch GUI, connect to Lich, see game text flowing in central window

**Success Criteria:** Game is technically playable (can read text, even if minimal features)

### Phase 2: Theme System + Styled Text (2-3 weeks)

#### Milestone 2.1: Theme Structure & Loader
**Goal:** Load and apply basic themes (colors + fonts)

**Deliverables:**
- [ ] Create `src/frontend/gui/theme.rs`:
  - All theme structs (Theme, ThemePalette, ThemeTypography, etc.)
  - `ThemeLoader` with TOML parsing
  - Theme inheritance logic (`merge_themes`)
  - Asset path resolution
- [ ] Create default theme: `assets/themes/default/theme.toml`
- [ ] Implement `apply_theme()` to set egui Style/Visuals
- [ ] Load fonts from theme (FontDefinitions)

**Test:** Create custom theme TOML, verify colors/fonts apply

#### Milestone 2.2: Asset Cache + Textures
**Goal:** Load and cache textures for backgrounds

**Deliverables:**
- [ ] Create `src/frontend/gui/assets.rs`:
  - `AssetCache` struct
  - `load_texture()` with caching
  - `preload_theme()` for bulk loading
- [ ] Add wire grate texture to default theme
- [ ] Implement textured background in GuiTextWindow
- [ ] Support tile/stretch/nine-slice modes

**Test:** Apply textured background to text window

#### Milestone 2.3: Styled Text Rendering
**Goal:** Render text with colors, bold, and links

**Deliverables:**
- [ ] Update GuiTextWindow to render `StyledSegment` with colors
- [ ] Support bold text
- [ ] Render clickable links (different color, underline on hover)
- [ ] Apply theme colors to text (foreground, link_color, etc.)

**Test:** Game text displays with proper colors matching theme

#### Milestone 2.4: Hot Reload + Theme Commands
**Goal:** Runtime theme switching

**Deliverables:**
- [ ] Implement `.listthemes` command
- [ ] Implement `.settheme <name>` command
- [ ] Implement `.reloadtheme` command
- [ ] Clear asset cache on theme change
- [ ] Trigger full UI refresh on theme change

**Test:** Switch themes without restarting app

**Success Criteria:** GUI looks polished, matches theme aesthetic

### Phase 3: Core Widgets + Docking (3-4 weeks)

#### Milestone 3.1: Docking Implementation
**Goal:** Full drag-and-drop docking with splits and tabs

**Deliverables:**
- [ ] Implement `DockState` for each area (left/right/central)
- [ ] Enable drag-and-drop between areas
- [ ] Support tab creation (drag tab onto another to merge)
- [ ] Support splits (drag to edge to split horizontal/vertical)
- [ ] Visual docking indicators while dragging
- [ ] Save/load dock tree state to config
- [ ] Context menu: "Split Horizontal/Vertical", "Move to Sidebar"

**Test:** Drag windows between areas, create tabs, create splits

#### Milestone 3.2: Command Input Widget
**Goal:** Command entry with history and send

**Deliverables:**
- [ ] Create `src/frontend/gui/widgets/command_input.rs`:
  - `GuiCommandInput` widget
  - Text entry field at bottom of window
  - Up/Down arrow for history
  - Send command to game on Enter
  - Command echo with themed color
- [ ] Dock command input to bottom of main window (or as status bar)

**Test:** Type commands, navigate history, send to game

#### Milestone 3.3: Progress Bar Widget
**Goal:** Vitals display (health, mana, stamina, etc.)

**Deliverables:**
- [ ] Create `src/frontend/gui/widgets/progress_bar.rs`:
  - `GuiProgressBar` widget
  - Render bar with fill color, background, border
  - Display current/max numbers or custom text
  - Apply theme colors
  - Special logic for encumbrance (color by value)
- [ ] Create vitals bars: health, mana, stamina, spirit, mindstate, encumbrance
- [ ] Connect to `AppCore.vitals`
- [ ] Add to right sidebar by default

**Test:** Vitals update in real-time from game, colors match theme

#### Milestone 3.4: Countdown Widget
**Goal:** Roundtime and casttime display

**Deliverables:**
- [ ] Create `src/frontend/gui/widgets/countdown.rs`:
  - `GuiCountdown` widget
  - Character-based fill animation (or smooth progress bar)
  - Display remaining seconds
  - Apply theme colors (roundtime=red, casttime=blue, stun=yellow)
- [ ] Connect to `AppCore` roundtime/casttime timestamps
- [ ] Add to right sidebar by default

**Test:** Cast spell or attack, see countdown update

#### Milestone 3.5: Window Templates & Creation
**Goal:** Easy window creation from templates

**Deliverables:**
- [ ] Create window template system:
  - Dropdown menu "Add Window" with categories
  - Templates for all standard windows (thoughts, speech, deaths, etc.)
- [ ] Implement `.addwindow` command (opens GUI picker)
- [ ] Drag template to dock area to create
- [ ] Context menu on any dock tab: "Close Tab"

**Test:** Add multiple text windows, dock them in different areas

#### Milestone 3.6: Layout Save/Load
**Goal:** Persist layouts to disk

**Deliverables:**
- [ ] Serialize DockState trees to TOML
- [ ] Save layout to `~/.two-face/layouts/<name>.toml`
- [ ] Load layout on startup (auto_<character>.toml priority)
- [ ] Implement `.savelayout [name]` command
- [ ] Implement `.loadlayout <name>` command
- [ ] Implement `.layouts` command (list saved)
- [ ] Auto-save on exit

**Test:** Create complex layout, save, restart, verify loaded correctly

**Success Criteria:** Core gameplay fully supported (text, commands, vitals, timers, custom layouts)

### Phase 4: Enhanced Widgets + Detached Windows (3-4 weeks)

#### Milestone 4.1: Tabbed Text Windows
**Goal:** Chat tabs (speech, thoughts, whisper, etc.)

**Deliverables:**
- [ ] Create `src/frontend/gui/widgets/tabbed_window.rs`:
  - `GuiTabbedWindow` widget
  - Multiple tabs, each with own TextWindow
  - Tab bar at top/bottom (configurable)
  - Click to switch tabs
  - Unread indicators (bold, prefix, color)
  - Route streams to correct tabs
- [ ] Create default chat tabbed window (Speech/Thoughts/Whisper)
- [ ] Support `.addtab`, `.removetab`, `.switchtab` commands

**Test:** Verify streams route correctly, unread indicators work

#### Milestone 4.2: Context Menus for Links
**Goal:** Right-click links for game commands

**Deliverables:**
- [ ] Detect link clicks in GuiTextWindow
- [ ] Create `src/frontend/gui/widgets/popup_menu.rs`:
  - `GuiPopupMenu` widget
  - Render at click position
  - Nested submenus (up to 3 levels)
  - Keyboard navigation (arrows, Enter, Esc)
  - Mouse click handling
- [ ] Connect to cmdlist.xml parsing
- [ ] Request menus from server (`_menu` protocol)
- [ ] Parse menu responses and build menu tree
- [ ] Execute selected command

**Test:** Right-click link, navigate menu, select action

#### Milestone 4.3: Compass, Hands, Room Widgets
**Goal:** Essential status widgets

**Deliverables:**
- [ ] `GuiCompass`:
  - 9-direction display (N, NE, E, SE, S, SW, W, NW, Out)
  - Active/inactive colors from theme
  - Click to move
  - Connect to `AppCore.room.exits`
- [ ] `GuiHands`:
  - Left/right hand display
  - "Empty" when nothing held
  - Themed colors
  - Connect to `AppCore.left_hand` / `AppCore.right_hand`
- [ ] `GuiRoomWindow`:
  - Room title, description
  - Exits list
  - Objects/NPCs
  - Themed styling

**Test:** Navigate rooms, pick up items, verify displays update

#### Milestone 4.4: Detached Windows
**Goal:** Pop windows out to OS windows

**Deliverables:**
- [ ] Implement `DetachedWindow` struct
- [ ] Use `egui::ViewportBuilder` for OS windows
- [ ] Context menu: "Detach Window" on any tab
- [ ] Context menu: "Re-dock" in detached window
- [ ] Track detached window positions/sizes
- [ ] Save/load detached windows in layout files
- [ ] Multi-monitor support (detached windows can span monitors)

**Test:** Detach window, move to second monitor, re-dock, verify position saved

#### Milestone 4.5: Layout Perspectives
**Goal:** Quick-switch layouts for different activities

**Deliverables:**
- [ ] Create perspective system:
  - `LayoutPerspective` struct
  - Save complete dock tree + sidebar state per perspective
- [ ] Create default perspectives:
  - "Combat" (vitals + timers prominent)
  - "Social" (chat tabs prominent)
  - "Hunting" (loot + experience + active effects)
  - "Minimal" (main window + vitals only)
- [ ] Dropdown menu: "View > Perspectives"
- [ ] Keybinds for quick-switch (F1-F4?)
- [ ] Implement `.setperspective <name>` command

**Test:** Switch between perspectives, verify layouts change correctly

**Success Criteria:** Advanced UI features, full layout flexibility, matches/exceeds Wrayth

### Phase 5: Advanced Features + Wrayth Aesthetics (4-6 weeks)

#### Milestone 5.1: Injury Doll
**Goal:** Layered wound/scar visualization

**Deliverables:**
- [ ] Create `src/frontend/gui/widgets/injury_doll.rs`:
  - `GuiInjuryDoll` widget
  - Load base doll texture
  - Load wound/scar overlay textures (rank 1/2/3)
  - Layer rendering (base + active wounds/scars)
  - Body part regions from theme
  - Tooltip on hover showing injury details
- [ ] Connect to `AppCore.injuries`
- [ ] Parse injury updates from game XML
- [ ] Create default injury doll assets
- [ ] Support theme customization (different doll art)

**Test:** Take damage, verify injuries appear on doll with correct severity

#### Milestone 5.2: Dashboard Widget
**Goal:** Comprehensive status display

**Deliverables:**
- [ ] Create `GuiDashboard`:
  - All vitals in compact view
  - Experience tracking (TNL, recent gains)
  - Active effects list
  - Encumbrance
  - Stance
  - Customizable layout of sub-widgets
  - Themed styling

**Test:** Verify all stats display and update

#### Milestone 5.3: Active Effects Widget
**Goal:** Buffs/debuffs display

**Deliverables:**
- [ ] Create `GuiActiveEffects`:
  - List of active spells/effects
  - Durations (countdown or remaining time)
  - Icons (if available in theme)
  - Color coding (buff=green, debuff=red, neutral=yellow)
  - Tooltip with effect details
- [ ] Connect to `AppCore.active_effects`
- [ ] Parse effect updates from game XML

**Test:** Cast buffs, verify they appear and count down

#### Milestone 5.4: Map Widget
**Goal:** Local area navigation map

**Deliverables:**
- [ ] Create `GuiMap`:
  - Load map coordinate data
  - Render grid with visited/unvisited rooms
  - Current room highlighted
  - Connection lines between adjacent rooms
  - Click room to navigate
  - Centered viewport (current room always center)
  - Support multiple z-levels (up/down)
- [ ] Connect to `AppCore.room.id`
- [ ] Load map data from `defaults/map_coordinates.json`

**Test:** Move around, verify map updates, click rooms to navigate

#### Milestone 5.5: Custom Window Chrome
**Goal:** Themed window borders and title bars

**Deliverables:**
- [ ] Implement custom window frame rendering:
  - Nine-slice border textures
  - Custom title bar with themed colors/textures
  - Resize handles with themed appearance
  - Drop shadow (if enabled in theme)
- [ ] Apply to all dockable windows
- [ ] Support per-window overrides
- [ ] Create Wrayth-style chrome assets (metal borders, etc.)

**Test:** Apply Wrayth theme, verify custom chrome appears

#### Milestone 5.6: Wrayth Classic Theme
**Goal:** Full recreation of Wrayth aesthetic

**Deliverables:**
- [ ] Create `assets/themes/wrayth-classic/`:
  - theme.toml with Wrayth colors
  - Wire grate texture for backgrounds
  - Metal border textures for window chrome
  - Injury doll base + overlays (if we have assets)
  - Custom fonts (if available)
- [ ] Fine-tune styling to match Wrayth screenshots
- [ ] Document theme structure for community creators

**Test:** Load Wrayth Classic theme, compare to actual Wrayth, verify close match

**Success Criteria:** GUI feature-complete, Wrayth aesthetic recreated, community can create themes

### Phase 6: Polish, Performance, Documentation (2-3 weeks)

#### Milestone 6.1: Performance Optimization
**Goal:** Smooth 60 FPS, low memory usage

**Deliverables:**
- [ ] Profile with `cargo flamegraph`
- [ ] Optimize text rendering (virtual scrolling, caching)
- [ ] Optimize texture loading (lazy load, unload unused)
- [ ] Optimize layout calculations (cache where possible)
- [ ] Limit text buffer sizes
- [ ] Add FPS counter (debug mode)
- [ ] Test with long gameplay sessions (memory leaks?)

**Test:** Run for 2+ hours, verify stable FPS and memory

#### Milestone 6.2: Bug Fixes & Edge Cases
**Goal:** Rock-solid stability

**Deliverables:**
- [ ] Test all widgets in all dock positions
- [ ] Test detached windows on multi-monitor setups
- [ ] Test theme switching while game running
- [ ] Test rapid layout changes
- [ ] Test with empty streams (no crashes)
- [ ] Test with very long lines (wrapping, performance)
- [ ] Test with unusual window sizes (tiny, huge)
- [ ] Fix any crashes or visual glitches

**Test:** QA testing, fix all reported issues

#### Milestone 6.3: Accessibility
**Goal:** Usable by all players

**Deliverables:**
- [ ] Keyboard navigation for all UI (tab between widgets)
- [ ] Screen reader support (if egui provides)
- [ ] High contrast themes
- [ ] Configurable font sizes
- [ ] Configurable UI scale (125%, 150%, 200%)
- [ ] Colorblind-friendly themes
- [ ] Document accessibility features

**Test:** Navigate entire UI with keyboard only

#### Milestone 6.4: Documentation
**Goal:** Help users and theme creators

**Deliverables:**
- [ ] Update CLAUDE.md with GUI architecture
- [ ] Create THEMES.md guide for theme creation
- [ ] Create LAYOUTS.md guide for layout customization
- [ ] Create WIDGETS.md reference for all widgets
- [ ] Update README.md with GUI screenshots
- [ ] Create example themes (dark, light, high contrast, Wrayth)
- [ ] Create example layouts (combat, social, crafting, hunting)
- [ ] Video tutorial (optional)

**Test:** Have new user follow docs to create custom theme and layout

#### Milestone 6.5: Make GUI Default
**Goal:** Transition from TUI to GUI as primary frontend

**Deliverables:**
- [ ] Change default `--frontend` to `gui`
- [ ] Keep TUI available with `--frontend tui` flag
- [ ] Update all docs to reference GUI by default
- [ ] Announce in community
- [ ] Gather feedback
- [ ] Address any final issues

**Test:** Fresh install, verify GUI launches by default and works smoothly

**Success Criteria:** GUI is production-ready, documented, and default frontend

---

### Timeline Summary

| Phase | Duration | Cumulative | Key Milestone |
|-------|----------|------------|---------------|
| **Phase 1: Foundation** | 2-3 weeks | 3 weeks | Game playable in GUI (basic text) |
| **Phase 2: Theme System** | 2-3 weeks | 6 weeks | Styled text, themed UI, hot reload |
| **Phase 3: Core Widgets** | 3-4 weeks | 10 weeks | Full docking, vitals, timers, layouts |
| **Phase 4: Enhanced** | 3-4 weeks | 14 weeks | Tabs, menus, detached windows, perspectives |
| **Phase 5: Advanced** | 4-6 weeks | 20 weeks | Injury doll, dashboard, map, Wrayth theme |
| **Phase 6: Polish** | 2-3 weeks | **23 weeks** | Production-ready, GUI default |

**Total: ~23 weeks (5-6 months)**

---

## 6. Technical Considerations

### 6.1 Performance

**Text Rendering:**
- Use egui's built-in text layout caching
- Limit buffer size for text windows (configurable)
- Only render visible lines (virtual scrolling)
- Batch text segments for efficient rendering

**Texture Management:**
- Load textures asynchronously
- Use mipmaps for scaled textures
- Unload unused textures after theme switch
- Consider texture atlases for small assets

**Layout Calculations:**
- Cache layout calculations
- Only recalculate on window resize or layout change
- Use egui's built-in layout system

### 6.2 Platform Compatibility

**Windows:**
- Native file dialogs for theme browsing
- Test with different DPI scaling settings
- Verify texture formats are supported

**Linux:**
- Test with X11 and Wayland
- Verify font fallbacks work correctly
- Test with different desktop environments

**Mac:**
- Test on Intel and Apple Silicon
- Verify retina display support
- Test with macOS dark mode

### 6.3 Migration Path

**Dual Frontend Support:**
- Keep TUI and GUI backends available simultaneously
- Share AppCore and all game logic
- Allow users to switch with `--frontend` flag
- Maintain TUI for headless/SSH use cases

**Data Migration:**
- Convert TUI layouts to GUI layouts automatically
- Provide tool to migrate user configs
- Maintain backward compatibility for configs

### 6.4 Future Enhancements

**Post-Launch Features:**
- Plugin system for custom widgets
- Scripting support for theme animations
- Advanced injury doll features (animated bleeding, etc.)
- Multi-monitor support with detachable windows
- Built-in screenshot/recording tools
- Shader support for advanced visual effects

---

## 7. Summary

This implementation plan provides a comprehensive roadmap for migrating two-face from ratatui to egui with full theming support. The phased approach ensures:

1. **Incremental progress** - Each phase delivers working features
2. **Backward compatibility** - TUI remains available during transition
3. **Extensibility** - Theme system supports unlimited customization
4. **Quality** - Each phase includes testing and validation
5. **Community** - Shareable themes enable user creativity

**Estimated total timeline: ~23 weeks (5-6 months)**

The theme system is designed to be powerful yet approachable, allowing both simple color tweaks and comprehensive visual overhauls like the Wrayth Classic recreation.

---

## 8. Next Steps

To begin implementation:

1. **Create directory structure** for `src/frontend/gui/`
2. **Set up egui skeleton** with basic window
3. **Define Theme structs** in `src/frontend/gui/theme.rs`
4. **Create default theme** in `assets/themes/default/theme.toml`
5. **Implement ThemeLoader** with TOML parsing
6. **Connect to AppCore** and display test text

Once Phase 1 foundation is complete, we can proceed to Phase 2 and begin filling out the theme system with real assets and comprehensive styling.
