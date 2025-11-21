//! TUI Frontend - Terminal UI using ratatui
//!
//! This module implements the Frontend trait for terminal rendering.

mod active_effects;
pub mod color_form;
pub mod color_palette_browser;
mod color_picker;
mod command_input;
mod compass;
mod countdown;
mod dashboard;
mod hand;
pub mod highlight_browser;
pub mod highlight_form;
mod indicator;
mod injury_doll;
mod inventory_window;
pub mod keybind_browser;
pub mod keybind_form;
mod performance_stats;
mod players;
mod popup_menu;
mod progress_bar;
mod room_window;
mod scrollable_container;
pub mod settings_editor;
mod spacer;
pub mod spell_color_browser;
pub mod spell_color_form;
mod spells_window;
mod tabbed_text_window;
mod targets;
mod text_window;
pub mod theme_browser;
pub mod theme_editor;
pub mod uicolors_browser;
pub mod window_editor;

use crate::frontend::{Frontend, FrontendEvent};
pub mod widget_traits;
use crate::core::AppCore;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::collections::HashMap;
use std::io;

pub struct TuiFrontend {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    /// Cache of TextWindow widgets per window name
    text_windows: HashMap<String, text_window::TextWindow>,
    /// Cache of CommandInput widgets per window name
    command_inputs: HashMap<String, command_input::CommandInput>,
    /// Cache of RoomWindow widgets per window name
    room_windows: HashMap<String, room_window::RoomWindow>,
    /// Cache of InventoryWindow widgets per window name
    inventory_windows: HashMap<String, inventory_window::InventoryWindow>,
    /// Cache of SpellsWindow widgets per window name
    spells_windows: HashMap<String, spells_window::SpellsWindow>,
    /// Cache of ProgressBar widgets per window name
    progress_bars: HashMap<String, progress_bar::ProgressBar>,
    /// Cache of Countdown widgets per window name
    countdowns: HashMap<String, countdown::Countdown>,
    /// Cache of ActiveEffects widgets per window name
    active_effects_windows: HashMap<String, active_effects::ActiveEffects>,
    /// Cache of Hand widgets per window name
    hand_widgets: HashMap<String, hand::Hand>,
    /// Cache of Spacer widgets per window name
    spacer_widgets: HashMap<String, spacer::Spacer>,
    /// Cache of Indicator widgets per window name
    indicator_widgets: HashMap<String, indicator::Indicator>,
    /// Cache of Targets widgets per window name
    targets_widgets: HashMap<String, targets::Targets>,
    /// Cache of Players widgets per window name
    players_widgets: HashMap<String, players::Players>,
    /// Cache of Dashboard widgets per window name
    dashboard_widgets: HashMap<String, dashboard::Dashboard>,
    /// Cache of TabbedTextWindow widgets per window name
    tabbed_text_windows: HashMap<String, tabbed_text_window::TabbedTextWindow>,
    /// Cache of Compass widgets per window name
    compass_widgets: HashMap<String, compass::Compass>,
    /// Cache of InjuryDoll widgets per window name
    injury_doll_widgets: HashMap<String, injury_doll::InjuryDoll>,
    /// Performance stats widget (singleton overlay)
    performance_stats_widget: Option<performance_stats::PerformanceStatsWidget>,
    /// Track last synced generation per text window to know what's new
    /// Using generation instead of line count to handle buffer rotation at max_lines
    last_synced_generation: HashMap<String, u64>,
    /// Active popup menu (if any)
    popup_menu: Option<popup_menu::PopupMenu>,
    /// Active submenu (if any)
    submenu: Option<popup_menu::PopupMenu>,
    /// Cached submenu items for the main menu
    menu_categories: HashMap<String, Vec<popup_menu::MenuItem>>,
    /// Active window editor (if any)
    pub window_editor: Option<window_editor::WindowEditor>,
    /// Active highlight browser (if any)
    pub highlight_browser: Option<highlight_browser::HighlightBrowser>,
    /// Active highlight form (if any)
    pub highlight_form: Option<highlight_form::HighlightFormWidget>,
    /// Active keybind browser (if any)
    pub keybind_browser: Option<keybind_browser::KeybindBrowser>,
    /// Active keybind form (if any)
    pub keybind_form: Option<keybind_form::KeybindFormWidget>,
    /// Active color palette browser (if any)
    pub color_palette_browser: Option<color_palette_browser::ColorPaletteBrowser>,
    /// Active color form (if any)
    pub color_form: Option<color_form::ColorForm>,
    /// Active UI colors browser (if any)
    pub uicolors_browser: Option<uicolors_browser::UIColorsBrowser>,
    /// Active spell color browser (if any)
    pub spell_color_browser: Option<spell_color_browser::SpellColorBrowser>,
    /// Active spell color form (if any)
    pub spell_color_form: Option<spell_color_form::SpellColorFormWidget>,
    /// Active theme browser (if any)
    pub theme_browser: Option<theme_browser::ThemeBrowser>,
    /// Active theme editor (if any)
    pub theme_editor: Option<theme_editor::ThemeEditor>,
    /// Active settings editor (if any)
    pub settings_editor: Option<settings_editor::SettingsEditor>,
    /// Debouncer for terminal resize events (100ms debounce)
    resize_debouncer: ResizeDebouncer,
    /// Cached theme to avoid HashMap lookup + clone every render
    cached_theme: crate::theme::AppTheme,
    /// Cached theme ID to detect theme changes
    cached_theme_id: String,
}

/// Parse a hex color string like "#RRGGBB" into ratatui Color
fn parse_hex_color(hex: &str) -> Result<ratatui::style::Color> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(anyhow::anyhow!("Invalid hex color length"));
    }

    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok(ratatui::style::Color::Rgb(r, g, b))
}

fn color_to_hex_string(color: &ratatui::style::Color) -> Option<String> {
    color_to_rgb(color).map(|(r, g, b)| format!("#{:02x}{:02x}{:02x}", r, g, b))
}

fn color_to_rgb(color: &ratatui::style::Color) -> Option<(u8, u8, u8)> {
    use ratatui::style::Color;

    match color {
        Color::Rgb(r, g, b) => Some((*r, *g, *b)),
        Color::Indexed(index) => Some(indexed_color_to_rgb(*index)),
        Color::Reset => None,
        Color::Black => Some((0, 0, 0)),
        Color::Red => Some((205, 0, 0)),
        Color::Green => Some((0, 205, 0)),
        Color::Yellow => Some((205, 205, 0)),
        Color::Blue => Some((0, 0, 205)),
        Color::Magenta => Some((205, 0, 205)),
        Color::Cyan => Some((0, 205, 205)),
        Color::Gray => Some((192, 192, 192)),
        Color::DarkGray => Some((128, 128, 128)),
        Color::LightRed => Some((255, 102, 102)),
        Color::LightGreen => Some((144, 238, 144)),
        Color::LightYellow => Some((255, 255, 102)),
        Color::LightBlue => Some((173, 216, 230)),
        Color::LightMagenta => Some((255, 119, 255)),
        Color::LightCyan => Some((224, 255, 255)),
        Color::White => Some((255, 255, 255)),
        _ => None,
    }
}

fn indexed_color_to_rgb(index: u8) -> (u8, u8, u8) {
    const STANDARD_COLORS: [(u8, u8, u8); 16] = [
        (0, 0, 0),
        (128, 0, 0),
        (0, 128, 0),
        (128, 128, 0),
        (0, 0, 128),
        (128, 0, 128),
        (0, 128, 128),
        (192, 192, 192),
        (128, 128, 128),
        (255, 0, 0),
        (0, 255, 0),
        (255, 255, 0),
        (0, 0, 255),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];

    if index < 16 {
        return STANDARD_COLORS[index as usize];
    }

    if index <= 231 {
        let level = index as usize - 16;
        let r = level / 36;
        let g = (level % 36) / 6;
        let b = level % 6;
        let levels = [0, 95, 135, 175, 215, 255];
        return (levels[r], levels[g], levels[b]);
    }

    // Grayscale ramp
    let gray = 8 + (index.saturating_sub(232)) * 10;
    (gray, gray, gray)
}

fn blend_colors_hex(
    base: &ratatui::style::Color,
    target: &ratatui::style::Color,
    ratio: f32,
) -> Option<String> {
    let (br, bg, bb) = color_to_rgb(base)?;
    let (tr, tg, tb) = color_to_rgb(target)?;
    let ratio = ratio.clamp(0.0, 1.0);
    let blend = |b: u8, t: u8| -> u8 {
        (b as f32 + (t as f32 - b as f32) * ratio)
            .round()
            .clamp(0.0, 255.0) as u8
    };
    Some(format!(
        "#{:02x}{:02x}{:02x}",
        blend(br, tr),
        blend(bg, tg),
        blend(bb, tb)
    ))
}

fn normalize_color(opt: &Option<String>) -> Option<String> {
    opt.as_ref().and_then(|s| {
        let trimmed = s.trim();
        if trimmed.is_empty() || trimmed == "-" {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

#[derive(Clone)]
struct WindowColors {
    border: Option<String>,
    background: Option<String>,
    text: Option<String>,
}

pub fn resolve_window_colors(
    base: &crate::config::WindowBase,
    theme: &crate::theme::AppTheme,
) -> WindowColors {
    let border =
        normalize_color(&base.border_color).or_else(|| color_to_hex_string(&theme.window_border));
    let background = if base.transparent_background {
        None
    } else {
        normalize_color(&base.background_color)
            .or_else(|| color_to_hex_string(&theme.window_background))
    };
    let text =
        normalize_color(&base.text_color).or_else(|| color_to_hex_string(&theme.text_primary));

    WindowColors {
        border,
        background,
        text,
    }
}

/// Debouncer for terminal resize events to prevent excessive layout recalculations
struct ResizeDebouncer {
    last_resize_time: Option<std::time::Instant>,
    debounce_duration: std::time::Duration,
    pending_size: Option<(u16, u16)>, // (width, height)
}

impl ResizeDebouncer {
    fn new(debounce_ms: u64) -> Self {
        Self {
            last_resize_time: None,
            debounce_duration: std::time::Duration::from_millis(debounce_ms),
            pending_size: None,
        }
    }

    /// Check if a resize event should be processed or debounced.
    ///
    /// Returns `Some((width, height))` if the resize should be processed immediately:
    /// - Always returns Some() for the first resize
    /// - Returns Some() if debounce_duration has elapsed since the last processed resize
    /// - Returns None() if the resize is within the debounce window (and stores as pending)
    ///
    /// When None is returned, the resize dimensions are stored as pending and will be
    /// checked on the next call to `check_pending()`.
    fn check_resize(&mut self, width: u16, height: u16) -> Option<(u16, u16)> {
        let now = std::time::Instant::now();

        // First resize is always processed immediately
        if self.last_resize_time.is_none() {
            self.last_resize_time = Some(now);
            self.pending_size = None;
            return Some((width, height));
        }

        let last_time = self.last_resize_time.unwrap();
        let elapsed = now.duration_since(last_time);

        if elapsed >= self.debounce_duration {
            // Debounce window has passed - process this resize immediately
            self.last_resize_time = Some(now);
            self.pending_size = None;
            Some((width, height))
        } else {
            // Still within debounce window - store as pending for later
            self.pending_size = Some((width, height));
            None
        }
    }

    /// Check if there's a pending resize that should be processed.
    ///
    /// Returns `Some((width, height))` if a pending resize exists and the debounce period
    /// has elapsed since the last processed resize. Returns `None()` otherwise.
    ///
    /// This should be called on every event loop iteration to ensure pending resizes are
    /// eventually processed even if no new resize events arrive.
    fn check_pending(&mut self) -> Option<(u16, u16)> {
        let now = std::time::Instant::now();

        // If no resize has been processed yet, there's nothing pending
        let last_time = match self.last_resize_time {
            Some(t) => t,
            None => return None,
        };

        let elapsed = now.duration_since(last_time);

        if elapsed >= self.debounce_duration {
            if let Some(size) = self.pending_size.take() {
                self.last_resize_time = Some(now);
                return Some(size);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_resize_processed_immediately() {
        let mut debouncer = ResizeDebouncer::new(100);
        let result = debouncer.check_resize(80, 24);

        assert_eq!(result, Some((80, 24)), "First resize should be processed immediately");
    }

    #[test]
    fn test_rapid_resizes_debounced() {
        let mut debouncer = ResizeDebouncer::new(100);

        // First resize is always processed
        let result1 = debouncer.check_resize(80, 24);
        assert_eq!(result1, Some((80, 24)));

        // Rapid resizes within 100ms should be debounced
        let result2 = debouncer.check_resize(81, 24);
        assert_eq!(result2, None, "Rapid resize should be debounced");

        let result3 = debouncer.check_resize(82, 24);
        assert_eq!(result3, None, "Rapid resize should be debounced");
    }

    #[test]
    fn test_pending_resize_stored() {
        let mut debouncer = ResizeDebouncer::new(100);

        debouncer.check_resize(80, 24);
        debouncer.check_resize(90, 30);

        // The second resize should be stored as pending with latest dimensions
        assert_eq!(debouncer.pending_size, Some((90, 30)));
    }

    #[test]
    fn test_multiple_pending_resizes_store_latest() {
        let mut debouncer = ResizeDebouncer::new(100);

        debouncer.check_resize(80, 24);
        debouncer.check_resize(90, 25);
        debouncer.check_resize(100, 26);
        debouncer.check_resize(110, 27);

        // Only the latest size should be stored
        assert_eq!(debouncer.pending_size, Some((110, 27)));
    }

    #[test]
    fn test_no_pending_resize_returns_none() {
        let mut debouncer = ResizeDebouncer::new(100);

        debouncer.check_resize(80, 24);

        // Immediately calling check_pending should return None (not enough time elapsed)
        let result = debouncer.check_pending();
        assert_eq!(result, None, "check_pending should return None when debounce period not elapsed");
    }

    #[test]
    fn test_pending_resize_processed_after_debounce() {
        let mut debouncer = ResizeDebouncer::new(10); // Use 10ms for faster test

        debouncer.check_resize(80, 24);
        debouncer.check_resize(90, 30);

        // Wait for debounce period to elapse
        std::thread::sleep(std::time::Duration::from_millis(15));

        let result = debouncer.check_pending();
        assert_eq!(result, Some((90, 30)), "Pending resize should be processed after debounce period");

        // After processing, pending should be cleared
        assert_eq!(debouncer.pending_size, None);
    }

    #[test]
    fn test_resize_after_debounce_period_immediate() {
        let mut debouncer = ResizeDebouncer::new(10);

        debouncer.check_resize(80, 24);
        debouncer.check_resize(90, 30);

        // Wait for debounce period to elapse
        std::thread::sleep(std::time::Duration::from_millis(15));

        // New resize should be processed immediately
        let result = debouncer.check_resize(100, 35);
        assert_eq!(result, Some((100, 35)), "Resize after debounce period should be processed immediately");
    }
}

impl TuiFrontend {
    pub fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            EnterAlternateScreen,
            crossterm::event::EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            text_windows: HashMap::new(),
            command_inputs: HashMap::new(),
            room_windows: HashMap::new(),
            inventory_windows: HashMap::new(),
            spells_windows: HashMap::new(),
            progress_bars: HashMap::new(),
            countdowns: HashMap::new(),
            active_effects_windows: HashMap::new(),
            hand_widgets: HashMap::new(),
            spacer_widgets: HashMap::new(),
            indicator_widgets: HashMap::new(),
            targets_widgets: HashMap::new(),
            players_widgets: HashMap::new(),
            dashboard_widgets: HashMap::new(),
            tabbed_text_windows: HashMap::new(),
            compass_widgets: HashMap::new(),
            injury_doll_widgets: HashMap::new(),
            performance_stats_widget: None,
            last_synced_generation: HashMap::new(),
            popup_menu: None,
            submenu: None,
            menu_categories: HashMap::new(),
            window_editor: None,
            highlight_browser: None,
            highlight_form: None,
            keybind_browser: None,
            keybind_form: None,
            color_palette_browser: None,
            color_form: None,
            uicolors_browser: None,
            spell_color_browser: None,
            spell_color_form: None,
            theme_browser: None,
            theme_editor: None,
            settings_editor: None,
            resize_debouncer: ResizeDebouncer::new(100), // 100ms debounce
            cached_theme: crate::theme::ThemePresets::dark(),
            cached_theme_id: "dark".to_string(),
        })
    }

    /// Update cached theme (call this when theme changes via command/browser)
    pub fn update_theme_cache(&mut self, theme_id: String, theme: crate::theme::AppTheme) {
        self.cached_theme = theme;
        self.cached_theme_id = theme_id;
    }

    /// Navigate to next tab in all tabbed windows
    pub fn next_tab_all(&mut self) {
        for widget in self.tabbed_text_windows.values_mut() {
            widget.next_tab();
        }
    }

    /// Navigate to previous tab in all tabbed windows
    pub fn prev_tab_all(&mut self) {
        for widget in self.tabbed_text_windows.values_mut() {
            widget.prev_tab();
        }
    }

    /// Navigate to next tab with unread messages (searches all tabbed windows)
    /// Returns true if found, false if no unread tabs
    pub fn go_to_next_unread_tab(&mut self) -> bool {
        for widget in self.tabbed_text_windows.values_mut() {
            if widget.next_tab_with_unread() {
                return true; // Found and switched
            }
        }
        false
    }

    /// Sync data from TextContent into TextWindow widgets
    fn sync_text_windows(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Text(text_content) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create TextWindow for this window
                let text_window = self.text_windows.entry(name.clone()).or_insert_with(|| {
                    let mut tw =
                        text_window::TextWindow::new(&text_content.title, text_content.max_lines);

                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        tw.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        tw.set_border_sides(def.base().border_sides.clone());
                        tw.set_background_color(colors.background.clone());
                        tw.set_text_color(colors.text.clone());
                    }

                    // Set highlights from config
                    let highlights_vec: Vec<_> =
                        app_core.config.highlights.values().cloned().collect();
                    tw.set_highlights(highlights_vec);

                    tw
                });

                // Existing text windows need to reapply theme-derived settings when themes change
                if let Some(def) = window_def {
                    let colors = resolve_window_colors(def.base(), theme);
                    text_window.set_border_config(
                        def.base().show_border,
                        Some(def.base().border_style.clone()),
                        colors.border.clone(),
                    );
                    text_window.set_border_sides(def.base().border_sides.clone());
                    text_window.set_background_color(colors.background.clone());
                    text_window.set_text_color(colors.text.clone());
                }

                // Update width for proper wrapping
                text_window.set_width(window.position.width);

                // Get last synced generation
                let last_synced_gen = self.last_synced_generation.get(name).copied().unwrap_or(0);
                let current_gen = text_content.generation;

                // Check if there are new lines to sync (generation changed)
                if current_gen > last_synced_gen {
                    // Calculate how many lines to add
                    // If generation delta > line count, we need to resync entire buffer
                    let gen_delta = (current_gen - last_synced_gen) as usize;
                    let needs_full_resync = gen_delta > text_content.lines.len();

                    if needs_full_resync {
                        // Full resync - clear and add all lines
                        tracing::trace!(
                            "Text window '{}': full resync (gen delta {} > line count {})",
                            name,
                            gen_delta,
                            text_content.lines.len()
                        );
                        text_window.clear();
                    }

                    // Determine how many lines to add
                    let lines_to_add = if needs_full_resync {
                        text_content.lines.len() // Add all lines
                    } else {
                        gen_delta.min(text_content.lines.len()) // Add only new lines
                    };

                    let skip_count = text_content.lines.len().saturating_sub(lines_to_add);
                    for line in text_content.lines.iter().skip(skip_count) {
                        // Convert our data format to TextWindow's format
                        for segment in &line.segments {
                            // Map data layer SpanType to TextWindow SpanType
                            use crate::data::SpanType as DataSpanType;
                            let tw_span_type = match segment.span_type {
                                DataSpanType::Normal => text_window::SpanType::Normal,
                                DataSpanType::Link => text_window::SpanType::Link,
                                DataSpanType::Monsterbold => text_window::SpanType::Monsterbold,
                                DataSpanType::Spell => text_window::SpanType::Spell,
                                DataSpanType::Speech => text_window::SpanType::Speech,
                            };

                            let styled_text = text_window::StyledText {
                                content: segment.text.clone(),
                                fg: segment
                                    .fg
                                    .as_ref()
                                    .and_then(|hex| parse_hex_color(hex).ok()),
                                bg: segment
                                    .bg
                                    .as_ref()
                                    .and_then(|hex| parse_hex_color(hex).ok()),
                                bold: segment.bold,
                                span_type: tw_span_type,
                                link_data: segment.link_data.as_ref().map(|ld| {
                                    text_window::LinkData {
                                        exist_id: ld.exist_id.clone(),
                                        noun: ld.noun.clone(),
                                        text: ld.text.clone(),
                                        coord: ld.coord.clone(),
                                    }
                                }),
                            };
                            text_window.add_text(styled_text);
                        }
                        // Finish the line with actual window width
                        text_window.finish_line(window.position.width);
                    }

                    // Update last synced generation
                    self.last_synced_generation
                        .insert(name.clone(), current_gen);
                }

                // Sync scroll offset from data layer to TextWindow
                // TextContent scroll_offset is lines from bottom (0 = live view)
                // TextWindow scroll methods handle this the same way
                // Note: TextWindow doesn't have a direct set_scroll_offset, so we'd need to
                // track the last known offset and call scroll_up/scroll_down as needed
                // For now, this is handled by user input events that modify both layers
            } else if let crate::data::WindowContent::Room(_room_content) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create RoomWindow for this window
                if !self.room_windows.contains_key(name) {
                    let mut room_window = room_window::RoomWindow::new("Room".to_string());

                    // Configure RoomWindow with settings from WindowDef
                    if let Some(crate::config::WindowDef::Room { data, .. }) = window_def {
                        // Set component visibility from config
                        room_window.set_component_visible("room desc", data.show_desc);
                        room_window.set_component_visible("room objs", data.show_objs);
                        room_window.set_component_visible("room players", data.show_players);
                        room_window.set_component_visible("room exits", data.show_exits);
                    }

                    self.room_windows.insert(name.clone(), room_window);
                    tracing::debug!("Created RoomWindow widget for '{}' during sync", name);
                }
            }
            // TODO: Add similar widget creation for other complex widget types as they're implemented:
            // - Progress bars (if they need stateful widgets beyond simple rendering)
            // - Countdown timers (if they need stateful widgets)
            // - Compass (if it needs stateful widgets)
            // - Indicator (if it needs stateful widgets)
            // - Hands/Inventory (if they need stateful widgets)
            // - Dashboard (if it needs stateful widgets)
            // Currently these render directly in the render loop without needing persistent widget state,
            // but if they gain more complex behavior (animations, interactions, etc.), they'll need
            // to be created here during sync just like Room and Text windows.
        }
    }

    /// Sync command input widgets with window configuration
    fn sync_command_inputs(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if !matches!(
                window.content,
                crate::data::WindowContent::CommandInput { .. }
            ) {
                continue;
            }

            let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);
            let (base_config, cmd_data) = match window_def {
                Some(crate::config::WindowDef::CommandInput { base, data }) => {
                    (Some(base.clone()), Some(data.clone()))
                }
                Some(def) => (Some(def.base().clone()), None),
                None => (None, None),
            };

            // Ensure the backing widget exists so we can apply configuration
            let cmd_input = self.command_inputs.entry(name.clone()).or_insert_with(|| {
                let mut widget = command_input::CommandInput::new(1000);
                if let Some(base) = base_config.as_ref() {
                    let title = base
                        .title
                        .clone()
                        .or_else(|| {
                            if base.name.is_empty() {
                                None
                            } else {
                                Some(base.name.clone())
                            }
                        })
                        .unwrap_or_else(|| "Command".to_string());
                    widget.set_title(title);
                } else {
                    widget.set_title("Command".to_string());
                }
                widget
            });

            if let Some(base) = base_config.as_ref() {
                let title = base
                    .title
                    .clone()
                    .or_else(|| {
                        if base.name.is_empty() {
                            None
                        } else {
                            Some(base.name.clone())
                        }
                    })
                    .unwrap_or_else(|| "Command".to_string());
                cmd_input.set_title(title);
                let border_color = normalize_color(&base.border_color)
                    .or_else(|| color_to_hex_string(&theme.window_border));
                cmd_input.set_border_config(
                    base.show_border,
                    Some(base.border_style.clone()),
                    border_color,
                );
                cmd_input.set_border_sides(base.border_sides.clone());
                cmd_input.set_show_title(base.show_title);
                let background_color = if base.transparent_background {
                    None
                } else {
                    normalize_color(&base.background_color)
                        .or_else(|| color_to_hex_string(&theme.window_background))
                };
                cmd_input.set_background_color(background_color);
                let text_color = cmd_data
                    .as_ref()
                    .and_then(|d| normalize_color(&d.text_color))
                    .or_else(|| normalize_color(&base.text_color))
                    .or_else(|| color_to_hex_string(&theme.text_primary));
                cmd_input.set_text_color(text_color);
                let cursor_fg = cmd_data
                    .as_ref()
                    .and_then(|d| normalize_color(&d.cursor_color))
                    .or_else(|| color_to_hex_string(&theme.window_background));
                let cursor_bg = cmd_data
                    .as_ref()
                    .and_then(|d| normalize_color(&d.cursor_background_color))
                    .or_else(|| color_to_hex_string(&theme.text_primary));
                cmd_input.set_cursor_colors(cursor_fg, cursor_bg);
            }
        }
    }

    /// Sync inventory window data - create/configure widgets
    fn sync_inventory_windows(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find inventory windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            // Check for both Inventory and Text content types
            let text_content = match &window.content {
                crate::data::WindowContent::Inventory(content) => Some(content),
                crate::data::WindowContent::Text(content)
                    if name == "inventory"
                        || content.title.to_lowercase().contains("inventory") =>
                {
                    Some(content)
                }
                _ => None,
            };

            if let Some(text_content) = text_content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create InventoryWindow for this window
                if !self.inventory_windows.contains_key(name) {
                    let inv_window =
                        inventory_window::InventoryWindow::new(text_content.title.clone());
                    self.inventory_windows.insert(name.clone(), inv_window);
                    tracing::debug!("Created InventoryWindow widget for '{}'", name);
                }

                // Update configuration and content from WindowDef if present
                if let Some(inv_window) = self.inventory_windows.get_mut(name) {
                    inv_window.set_title(text_content.title.clone());
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        inv_window.set_border_config(def.base().show_border, colors.border.clone());
                        inv_window.set_transparent_background(def.base().transparent_background);
                        inv_window.set_background_color(colors.background.clone());
                        inv_window.set_text_color(colors.text.clone());
                    }

                    // Change detection: only sync if content changed (using generation)
                    let last_synced_gen =
                        self.last_synced_generation.get(name).copied().unwrap_or(0);
                    let current_gen = text_content.generation;

                    if current_gen != last_synced_gen {
                        // Content changed - sync text lines from WindowContent to widget
                        inv_window.clear();
                        tracing::debug!("Syncing inventory widget '{}' with {} lines (gen changed from {} to {})",
                            name, text_content.lines.len(), last_synced_gen, current_gen);
                        for line in &text_content.lines {
                            for segment in &line.segments {
                                inv_window.add_segment(segment.clone());
                            }
                            inv_window.finish_line();
                        }
                        // Update last synced generation
                        self.last_synced_generation
                            .insert(name.clone(), current_gen);
                    }
                } else {
                    tracing::warn!(
                        "Inventory widget '{}' not found in inventory_windows HashMap!",
                        name
                    );
                }
            }
        }
    }

    /// Sync spells window data - create/configure widgets
    fn sync_spells_windows(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find spells windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            // Check for Spells content type
            let text_content = match &window.content {
                crate::data::WindowContent::Spells(content) => Some(content),
                _ => None,
            };

            if let Some(text_content) = text_content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create SpellsWindow for this window
                if !self.spells_windows.contains_key(name) {
                    let spells_window =
                        spells_window::SpellsWindow::new(text_content.title.clone());
                    self.spells_windows.insert(name.clone(), spells_window);
                    tracing::debug!("Created SpellsWindow widget for '{}'", name);
                }

                // Update configuration and content from WindowDef if present
                if let Some(spells_window) = self.spells_windows.get_mut(name) {
                    spells_window.set_title(text_content.title.clone());
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        spells_window.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        spells_window.set_transparent_background(def.base().transparent_background);
                        spells_window.set_background_color(colors.background.clone());
                        spells_window.set_text_color(colors.text.clone());
                    }

                    // Change detection: only sync if content changed (using generation)
                    let last_synced_gen =
                        self.last_synced_generation.get(name).copied().unwrap_or(0);
                    let current_gen = text_content.generation;

                    if current_gen != last_synced_gen {
                        // Content changed - sync text lines from WindowContent to widget
                        spells_window.clear();
                        tracing::debug!(
                            "Syncing spells widget '{}' with {} lines (gen changed from {} to {})",
                            name,
                            text_content.lines.len(),
                            last_synced_gen,
                            current_gen
                        );
                        for line in &text_content.lines {
                            for segment in &line.segments {
                                spells_window.add_text(
                                    segment.text.clone(),
                                    segment.fg.clone(),
                                    segment.bg.clone(),
                                    segment.bold,
                                    segment.span_type,
                                    segment.link_data.clone(),
                                );
                            }
                            spells_window.finish_line();
                        }
                        // Update last synced generation
                        self.last_synced_generation
                            .insert(name.clone(), current_gen);
                    }
                } else {
                    tracing::warn!(
                        "Spells widget '{}' not found in spells_windows HashMap!",
                        name
                    );
                }
            }
        }
    }

    /// Sync progress bar data - create/configure widgets
    fn sync_progress_bars(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find progress bar windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Progress(progress_data) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create ProgressBar for this window
                if !self.progress_bars.contains_key(name) {
                    let label = window_def
                        .and_then(|def| def.base().title.as_ref())
                        .map(|t| t.clone())
                        .unwrap_or_else(|| progress_data.label.clone());

                    let bar = progress_bar::ProgressBar::new(&label);
                    self.progress_bars.insert(name.clone(), bar);
                    tracing::debug!("Created ProgressBar widget for '{}'", name);
                }

                // Update configuration and value
                if let Some(progress_bar) = self.progress_bars.get_mut(name) {
                    // Set value from game data
                    if let Some(ref custom_text) = progress_data.color {
                        // color field is being used as custom text (e.g., "clear as a bell")
                        progress_bar.set_value_with_text(
                            progress_data.value as u32,
                            progress_data.max as u32,
                            Some(custom_text.clone()),
                        );
                    } else {
                        progress_bar
                            .set_value(progress_data.value as u32, progress_data.max as u32);
                    }

                    // Apply window config from WindowDef
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        progress_bar.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );

                        // Get bar color from ProgressWidgetData, or fallback to VellumFE defaults
                        if let crate::config::WindowDef::Progress { data, .. } = def {
                            let bar_color = if let Some(ref color) = data.color {
                                Some(color.clone())
                            } else {
                                // Fallback to VellumFE template colors for known progress bars
                                match name.as_str() {
                                    "health" => Some("#6e0202".to_string()),     // Dark red
                                    "mana" => Some("#08086d".to_string()),       // Dark blue
                                    "stamina" => Some("#bd7b00".to_string()),    // Orange
                                    "spirit" => Some("#6e727c".to_string()),     // Gray
                                    "encumlevel" => Some("#ffff00".to_string()), // Yellow
                                    "pbarStance" => Some("#ffa500".to_string()), // Orange
                                    "mindState" => Some("#9370db".to_string()),  // Purple
                                    "lblBPs" => Some("#ff4500".to_string()),     // Orange-red
                                    _ => None,
                                }
                            };

                            if let Some(color) = bar_color {
                                progress_bar.set_colors(Some(color), None);
                            }
                        }

                        // Apply text color
                        progress_bar.set_text_color(colors.text.clone());

                        // Apply transparent background setting
                        progress_bar.set_transparent_background(def.base().transparent_background);
                        progress_bar.set_background_color(colors.background.clone());
                    }
                }
            }
        }
    }

    /// Sync countdown data - create/configure countdown widgets
    fn sync_countdowns(&mut self, app_core: &crate::core::AppCore, theme: &crate::theme::AppTheme) {
        // Find countdown windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Countdown(countdown_data) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create Countdown for this window
                if !self.countdowns.contains_key(name) {
                    let label = window_def
                        .and_then(|def| def.base().title.as_ref())
                        .map(|t| t.clone())
                        .unwrap_or_else(|| name.clone());

                    let countdown = countdown::Countdown::new(&label);
                    self.countdowns.insert(name.clone(), countdown);
                    tracing::debug!("Created Countdown widget for '{}'", name);
                }

                // Update configuration and value
                if let Some(countdown_widget) = self.countdowns.get_mut(name) {
                    // Set end time from game data
                    countdown_widget.set_end_time(countdown_data.end_time);

                    // Apply window config from WindowDef
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        countdown_widget.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );

                        // Get icon from CountdownWidgetData
                        if let crate::config::WindowDef::Countdown { data, .. } = def {
                            if let Some(icon) = data.icon {
                                countdown_widget.set_icon(icon);
                            }
                        }

                        countdown_widget.set_text_color(colors.text.clone());
                        countdown_widget
                            .set_transparent_background(def.base().transparent_background);
                    }
                }
            }
        }
    }

    /// Sync active effects data - create/configure active effects widgets
    fn sync_active_effects(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find active effects windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::ActiveEffects(effects_content) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create ActiveEffects for this window
                if !self.active_effects_windows.contains_key(name) {
                    let label = window_def
                        .and_then(|def| def.base().title.as_ref())
                        .map(|t| t.clone())
                        .unwrap_or_else(|| name.clone());

                    let widget = active_effects::ActiveEffects::new(
                        &label,
                        effects_content.category.clone(),
                    );
                    self.active_effects_windows.insert(name.clone(), widget);
                    tracing::debug!("Created ActiveEffects widget for '{}'", name);
                }

                // Update effects data and configuration
                if let Some(widget) = self.active_effects_windows.get_mut(name) {
                    let previous_scroll = widget.scroll_position();

                    // Clear existing effects
                    widget.clear();

                    // Add all effects from content
                    for effect in &effects_content.effects {
                        widget.add_or_update_effect(
                            effect.id.clone(),
                            effect.text.clone(),
                            effect.value,
                            effect.time.clone(),
                            effect.bar_color.clone(),
                            effect.text_color.clone(),
                        );
                    }

                    widget.restore_scroll_position(previous_scroll);

                    // Apply window config from WindowDef
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        widget.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(def.base().border_sides.clone());
                        widget.set_transparent_background(def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                        widget.set_text_color(colors.text.clone());
                    }
                }
            }
        }
    }

    /// Sync spacer widget data from AppCore to spacer widgets
    fn sync_spacer_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find all Spacer windows in the UI state (Empty content + Spacer widget type)
        for (name, window) in &app_core.ui_state.windows {
            if window.widget_type == crate::data::WidgetType::Spacer {
                // Ensure spacer widget exists in cache
                if !self.spacer_widgets.contains_key(name) {
                    let widget = spacer::Spacer::new();
                    self.spacer_widgets.insert(name.clone(), widget);
                }

                // Update spacer widget configuration
                if let Some(spacer_widget) = self.spacer_widgets.get_mut(name) {
                    // Apply window configuration from layout
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        spacer_widget.set_background_color(colors.background.clone());
                        spacer_widget
                            .set_transparent_background(window_def.base().transparent_background);
                    }
                }
            }
        }
    }

    /// Sync indicator widget data from AppCore to indicator widgets
    fn sync_indicator_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find all Indicator windows in the UI state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Indicator(indicator_data) = &window.content {
                // Ensure indicator widget exists in cache
                if !self.indicator_widgets.contains_key(name) {
                    let widget = indicator::Indicator::new(name);
                    self.indicator_widgets.insert(name.clone(), widget);
                }

                // Update indicator widget content and configuration
                if let Some(indicator_widget) = self.indicator_widgets.get_mut(name) {
                    // Set status (which determines if it's active/shown)
                    indicator_widget.set_status(&indicator_data.status);

                    // Apply window configuration from layout
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        indicator_widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        indicator_widget.set_border_sides(window_def.base().border_sides.clone());
                        indicator_widget.set_title(
                            window_def
                                .base()
                                .title
                                .clone()
                                .unwrap_or_else(|| name.clone()),
                        );
                        indicator_widget.set_background_color(colors.background.clone());
                        indicator_widget
                            .set_transparent_background(window_def.base().transparent_background);

                        // Set custom colors if provided
                        if let Some(ref color) = indicator_data.color {
                            indicator_widget.set_colors("#555555".to_string(), color.clone());
                        }
                    }
                }
            }
        }
    }

    /// Sync targets widget data from AppCore to targets widgets
    fn sync_targets_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Targets { targets_text } = &window.content {
                // Ensure widget exists
                if !self.targets_widgets.contains_key(name) {
                    let widget = targets::Targets::new(name);
                    self.targets_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.targets_widgets.get_mut(name) {
                    widget.set_targets_from_text(targets_text);

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        if let Some(ref color) = colors.text {
                            widget.set_bar_color(color.clone());
                        }
                    }
                }
            }
        }
    }

    /// Sync players widget data from AppCore to players widgets
    fn sync_players_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Players { players_text } = &window.content {
                // Ensure widget exists
                if !self.players_widgets.contains_key(name) {
                    let widget = players::Players::new(name);
                    self.players_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.players_widgets.get_mut(name) {
                    widget.set_players_from_text(players_text);

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        if let Some(ref color) = colors.text {
                            widget.set_bar_color(color.clone());
                        }
                    }
                }
            }
        }
    }

    /// Sync dashboard widget data from AppCore to dashboard widgets
    fn sync_dashboard_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Dashboard { indicators } = &window.content {
                // Ensure widget exists
                if !self.dashboard_widgets.contains_key(name) {
                    // Default to horizontal layout - can be configured via WindowDef later
                    let widget =
                        dashboard::Dashboard::new(name, dashboard::DashboardLayout::Horizontal);
                    self.dashboard_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.dashboard_widgets.get_mut(name) {
                    // Update indicator values
                    for (id, value) in indicators {
                        widget.set_indicator_value(id, *value);
                    }

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                    }
                }
            }
        }
    }

    /// Sync tabbed text window data from AppCore to tabbed text widgets
    fn sync_tabbed_text_windows(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::TabbedText(tabbed_content) = &window.content {
                // Ensure widget exists - create if needed
                if !self.tabbed_text_windows.contains_key(name) {
                    // Create widget with tab definitions
                    let tabs: Vec<(String, String)> = tabbed_content
                        .tabs
                        .iter()
                        .map(|t| (t.name.clone(), t.stream.clone()))
                        .collect();

                    let widget = tabbed_text_window::TabbedTextWindow::with_tabs(
                        name,
                        tabs,
                        tabbed_content.max_lines_per_tab,
                    );
                    self.tabbed_text_windows.insert(name.clone(), widget);
                }

                // Apply configuration
                if let Some(widget) = self.tabbed_text_windows.get_mut(name) {
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                        widget.apply_window_colors(colors.text.clone(), colors.background.clone());
                    }
                }
            }
        }
    }

    /// Sync compass widget data from AppCore to compass widgets
    fn sync_compass_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Compass(compass_data) = &window.content {
                // Ensure widget exists
                if !self.compass_widgets.contains_key(name) {
                    let widget = compass::Compass::new(name);
                    self.compass_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.compass_widgets.get_mut(name) {
                    widget.set_directions(compass_data.directions.clone());

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                        widget.set_title(
                            window_def
                                .base()
                                .title
                                .clone()
                                .unwrap_or_else(|| name.clone()),
                        );

                        // Apply compass-specific colors if configured
                        if let crate::config::WindowDef::Compass { data, .. } = window_def {
                            let active_color = normalize_color(&data.active_color).or_else(|| {
                                color_to_hex_string(&theme.window_border_focused)
                                    .or_else(|| color_to_hex_string(&theme.window_border))
                            });
                            let inactive_color =
                                normalize_color(&data.inactive_color).or_else(|| {
                                    blend_colors_hex(
                                        &theme.window_background,
                                        &theme.text_secondary,
                                        0.25,
                                    )
                                    .or_else(|| color_to_hex_string(&theme.text_secondary))
                                });
                            widget.set_colors(active_color, inactive_color);
                        }
                    }
                }
            }
        }
    }

    /// Sync injury doll widget data from AppCore to injury doll widgets
    fn sync_injury_doll_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::InjuryDoll(injury_data) = &window.content {
                // Ensure widget exists
                if !self.injury_doll_widgets.contains_key(name) {
                    let widget = injury_doll::InjuryDoll::new(name);
                    self.injury_doll_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.injury_doll_widgets.get_mut(name) {
                    // Update all injuries
                    for (body_part, level) in &injury_data.injuries {
                        widget.set_injury(body_part.clone(), *level);
                    }

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                        widget.set_title(
                            window_def
                                .base()
                                .title
                                .clone()
                                .unwrap_or_else(|| name.clone()),
                        );

                        // Apply injury doll color configuration if specified
                        if let crate::config::WindowDef::InjuryDoll { data, .. } = window_def {
                            let resolved_default = normalize_color(&data.injury_default_color)
                                .or_else(|| color_to_hex_string(&theme.injury_default_color))
                                .unwrap_or_else(|| "#333333".to_string());
                            // Build colors vec with defaults if not specified
                            let colors = vec![
                                resolved_default,
                                data.injury1_color
                                    .clone()
                                    .unwrap_or_else(|| "#aa5500".to_string()),
                                data.injury2_color
                                    .clone()
                                    .unwrap_or_else(|| "#ff8800".to_string()),
                                data.injury3_color
                                    .clone()
                                    .unwrap_or_else(|| "#ff0000".to_string()),
                                data.scar1_color
                                    .clone()
                                    .unwrap_or_else(|| "#999999".to_string()),
                                data.scar2_color
                                    .clone()
                                    .unwrap_or_else(|| "#777777".to_string()),
                                data.scar3_color
                                    .clone()
                                    .unwrap_or_else(|| "#555555".to_string()),
                            ];
                            widget.set_colors(colors);
                        }
                    }
                }
            }
        }
    }

    /// Sync hand widget data from AppCore to hand widgets
    fn sync_hand_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find all Hand windows in the UI state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Hand { item, link } = &window.content {
                // Ensure hand widget exists in cache
                if !self.hand_widgets.contains_key(name) {
                    // Determine hand type based on window name
                    let hand_type = match name.as_str() {
                        "left_hand" => hand::HandType::Left,
                        "right_hand" => hand::HandType::Right,
                        "spell_hand" => hand::HandType::Spell,
                        _ => hand::HandType::Left, // Default fallback
                    };

                    let widget = hand::Hand::new(name, hand_type);
                    self.hand_widgets.insert(name.clone(), widget);
                }

                // Update hand widget content
                if let Some(hand_widget) = self.hand_widgets.get_mut(name) {
                    // Set content (or empty if None)
                    let content = item.clone().unwrap_or_default();
                    hand_widget.set_content(content);

                    // Apply window configuration from layout
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        hand_widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        hand_widget.set_border_sides(window_def.base().border_sides.clone());
                        hand_widget.set_title(
                            window_def
                                .base()
                                .title
                                .clone()
                                .unwrap_or_else(|| name.clone()),
                        );
                        hand_widget.set_text_color(colors.text.clone());
                        hand_widget.set_content_highlight_color(None);
                        if let Some(link_ref) = link {
                            hand_widget.set_link_data(Some(link_ref.clone()));
                            if let Some(preset) = app_core.config.colors.presets.get("links") {
                                if let Some(link_fg) = preset.fg.clone() {
                                    hand_widget.set_content_highlight_color(Some(link_fg));
                                }
                            }
                        } else {
                            hand_widget.set_link_data(None);
                        }
                        hand_widget.set_background_color(colors.background.clone());
                        hand_widget
                            .set_transparent_background(window_def.base().transparent_background);
                    }
                }
            }
        }
    }

    /// Sync room window data from AppCore to room window widgets
    fn sync_room_windows(
        &mut self,
        app_core: &mut crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        let new_title = if app_core.room_window_dirty {
            Some(self.build_room_title(
                &app_core.room_subtitle,
                &app_core.lich_room_id,
                &app_core.nav_room_id,
            ))
        } else {
            None
        };

        for window_def in app_core
            .layout
            .windows
            .iter()
            .filter(|w| w.widget_type() == "room")
        {
            let window_name = window_def.name();
            self.ensure_room_window_exists(window_name, window_def);

            if let Some(room_window) = self.room_windows.get_mut(window_name) {
                let colors = resolve_window_colors(window_def.base(), theme);
                room_window.set_border_config(
                    window_def.base().show_border,
                    Some(window_def.base().border_style.clone()),
                    colors.border.clone(),
                );
                room_window.set_border_sides(window_def.base().border_sides.clone());
                room_window.set_background_color(colors.background.clone());
                room_window.set_text_color(colors.text.clone());
                if let crate::config::WindowDef::Room { data, .. } = window_def {
                    room_window.set_component_visible("room desc", data.show_desc);
                    room_window.set_component_visible("room objs", data.show_objs);
                    room_window.set_component_visible("room players", data.show_players);
                    room_window.set_component_visible("room exits", data.show_exits);
                    room_window.set_show_name(data.show_name);
                }

                if let Some(ref title) = new_title {
                    room_window.clear_all_components();

                    for (component_id, lines) in &app_core.room_components {
                        room_window.start_component(component_id.clone());

                        for line_segments in lines {
                            for segment in line_segments {
                                room_window.add_segment(segment.clone());
                            }
                            room_window.finish_line();
                        }

                        room_window.finish_component();
                    }

                    room_window.set_title(title.clone());
                }
            }
        }

        if new_title.is_some() {
            app_core.room_window_dirty = false;
        }
    }

    /// Build room window title from room data
    /// Format: "[subtitle - lich_id] (u<nav_id>)"
    /// Example: "[Emberthorn Refuge, Bowery - 33711] (u2022628)"
    fn build_room_title(
        &self,
        subtitle: &Option<String>,
        lich_id: &Option<String>,
        nav_id: &Option<String>,
    ) -> String {
        // Format: [subtitle - lich_room_id] (u_nav_room_id)
        if let Some(ref subtitle_text) = subtitle {
            if let Some(ref lich) = lich_id {
                if let Some(ref nav) = nav_id {
                    format!("[{} - {}] (u{})", subtitle_text, lich, nav)
                } else {
                    format!("[{} - {}]", subtitle_text, lich)
                }
            } else if let Some(ref nav) = nav_id {
                format!("[{}] (u{})", subtitle_text, nav)
            } else {
                format!("[{}]", subtitle_text)
            }
        } else if let Some(ref lich) = lich_id {
            if let Some(ref nav) = nav_id {
                format!("[{}] (u{})", lich, nav)
            } else {
                format!("[{}]", lich)
            }
        } else if let Some(ref nav) = nav_id {
            format!("(u{})", nav)
        } else {
            String::new() // No title to set
        }
    }

    /// Scroll a text window by name
    pub fn scroll_window(&mut self, window_name: &str, lines: i32) {
        // Try text window first
        if let Some(text_window) = self.text_windows.get_mut(window_name) {
            if lines > 0 {
                text_window.scroll_up(lines as usize);
            } else if lines < 0 {
                text_window.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try room window
        if let Some(room_window) = self.room_windows.get_mut(window_name) {
            if lines > 0 {
                room_window.scroll_up(lines as usize);
            } else if lines < 0 {
                room_window.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try inventory window
        if let Some(inventory_window) = self.inventory_windows.get_mut(window_name) {
            if lines > 0 {
                inventory_window.scroll_up(lines as usize);
            } else if lines < 0 {
                inventory_window.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try spells window
        if let Some(spells_window) = self.spells_windows.get_mut(window_name) {
            if lines > 0 {
                spells_window.scroll_up(lines as usize);
            } else if lines < 0 {
                spells_window.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try active_effects widget
        if let Some(active_effects) = self.active_effects_windows.get_mut(window_name) {
            if lines > 0 {
                active_effects.scroll_up(lines as usize);
            } else if lines < 0 {
                active_effects.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try targets widget
        if let Some(targets) = self.targets_widgets.get_mut(window_name) {
            if lines > 0 {
                targets.scroll_up(lines as usize);
            } else if lines < 0 {
                targets.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try players widget
        if let Some(players) = self.players_widgets.get_mut(window_name) {
            if lines > 0 {
                players.scroll_up(lines as usize);
            } else if lines < 0 {
                players.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try tabbed text window
        if let Some(tabbed_window) = self.tabbed_text_windows.get_mut(window_name) {
            if lines > 0 {
                tabbed_window.scroll_up(lines as usize);
            } else if lines < 0 {
                tabbed_window.scroll_down((-lines) as usize);
            }
            return;
        }
    }

    /// Convert mouse position to text coordinates (line, col) in a text window
    pub fn mouse_to_text_coords(
        &self,
        window_name: &str,
        mouse_col: u16,
        mouse_row: u16,
        window_rect: ratatui::layout::Rect,
    ) -> Option<(usize, usize)> {
        let text_window = self.text_windows.get(window_name)?;
        text_window.mouse_to_text_coords(mouse_col, mouse_row, window_rect)
    }

    /// Extract selected text from a text window
    pub fn extract_selection_text(
        &self,
        window_name: &str,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Option<String> {
        let text_window = self.text_windows.get(window_name)?;
        Some(text_window.extract_selection_text(start_line, start_col, end_line, end_col))
    }

    /// Ensure a command input widget exists (should be called during init)
    pub fn ensure_command_input_exists(&mut self, window_name: &str) {
        if !self.command_inputs.contains_key(window_name) {
            let mut cmd_input = command_input::CommandInput::new(1000);
            cmd_input.set_title("Command".to_string());
            self.command_inputs
                .insert(window_name.to_string(), cmd_input);
            tracing::debug!("Created CommandInput widget for '{}'", window_name);
        }
    }

    /// Handle keyboard input for command input widget
    pub fn command_input_key(
        &mut self,
        window_name: &str,
        code: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
        available_commands: &[String],
        available_window_names: &[String],
    ) {
        use crossterm::event::{KeyCode, KeyModifiers};

        // Widget should already exist (created during init)
        if !self.command_inputs.contains_key(window_name) {
            tracing::warn!(
                "CommandInput widget '{}' doesn't exist, creating it now",
                window_name
            );
            self.ensure_command_input_exists(window_name);
        }

        if let Some(cmd_input) = self.command_inputs.get_mut(window_name) {
            match code {
                KeyCode::Char(c) => {
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        match c {
                            'a' => cmd_input.move_cursor_home(),
                            'e' => cmd_input.move_cursor_end(),
                            'u' => cmd_input.clear(),
                            'w' => {
                                // Delete word backwards (Ctrl+W)
                                // Get current input state
                                if let Some(input) = cmd_input.get_input() {
                                    let chars: Vec<char> = input.chars().collect();
                                    let mut count = 0;

                                    // Count characters to delete
                                    let mut pos = chars.len();

                                    // Skip trailing whitespace
                                    while pos > 0
                                        && chars
                                            .get(pos.saturating_sub(1))
                                            .map_or(false, |c| c.is_whitespace())
                                    {
                                        count += 1;
                                        pos -= 1;
                                    }

                                    // Delete word
                                    while pos > 0
                                        && chars
                                            .get(pos.saturating_sub(1))
                                            .map_or(false, |c| !c.is_whitespace())
                                    {
                                        count += 1;
                                        pos -= 1;
                                    }

                                    // Delete the counted characters
                                    for _ in 0..count {
                                        cmd_input.delete_char();
                                    }
                                }
                            }
                            _ => {}
                        }
                    } else {
                        cmd_input.insert_char(c);
                    }
                }
                KeyCode::Backspace => cmd_input.delete_char(),
                KeyCode::Delete => cmd_input.delete_word(), // Delete forward is delete word
                KeyCode::Left => {
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        cmd_input.move_cursor_word_left();
                    } else {
                        cmd_input.move_cursor_left();
                    }
                }
                KeyCode::Right => {
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        cmd_input.move_cursor_word_right();
                    } else {
                        cmd_input.move_cursor_right();
                    }
                }
                KeyCode::Home => cmd_input.move_cursor_home(),
                KeyCode::End => cmd_input.move_cursor_end(),
                KeyCode::Up => cmd_input.history_previous(),
                KeyCode::Down => cmd_input.history_next(),
                KeyCode::Tab => {
                    // Tab completion for commands and window names
                    cmd_input.try_complete(available_commands, available_window_names);
                }
                _ => {}
            }
        }
    }

    /// Submit command from command input and return the command string
    pub fn command_input_submit(&mut self, window_name: &str) -> Option<String> {
        self.command_inputs.get_mut(window_name)?.submit()
    }

    /// Load command history for a character
    pub fn command_input_load_history(
        &mut self,
        window_name: &str,
        character: Option<&str>,
    ) -> Result<()> {
        if let Some(cmd_input) = self.command_inputs.get_mut(window_name) {
            cmd_input.load_history(character)?;
        }
        Ok(())
    }

    /// Save command history for a character
    pub fn command_input_save_history(
        &self,
        window_name: &str,
        character: Option<&str>,
    ) -> Result<()> {
        if let Some(cmd_input) = self.command_inputs.get(window_name) {
            cmd_input.save_history(character)?;
        }
        Ok(())
    }

    /// Ensure a room window widget exists (should be called during init)
    pub fn ensure_room_window_exists(
        &mut self,
        window_name: &str,
        window_def: &crate::config::WindowDef,
    ) {
        if !self.room_windows.contains_key(window_name) {
            let mut room_window = room_window::RoomWindow::new("Room".to_string());

            // Configure RoomWindow with settings from WindowDef
            if let crate::config::WindowDef::Room { data, .. } = window_def {
                // Set component visibility from config
                room_window.set_component_visible("room desc", data.show_desc);
                room_window.set_component_visible("room objs", data.show_objs);
                room_window.set_component_visible("room players", data.show_players);
                room_window.set_component_visible("room exits", data.show_exits);
                room_window.set_show_name(data.show_name);
            }

            self.room_windows
                .insert(window_name.to_string(), room_window);
            tracing::debug!("Created RoomWindow widget for '{}'", window_name);
        }
    }

    /// Clear all components in a room window (called when pushStream id="room")
    pub fn room_window_clear_components(&mut self, window_name: &str) {
        if let Some(room_window) = self.room_windows.get_mut(window_name) {
            room_window.clear_all_components();
            tracing::debug!("Cleared all components for room window '{}'", window_name);
        }
    }

    /// Start building a room component
    pub fn room_window_start_component(&mut self, window_name: &str, component_id: String) {
        if let Some(room_window) = self.room_windows.get_mut(window_name) {
            room_window.start_component(component_id);
        }
    }

    /// Add a segment to the current component in a room window
    pub fn room_window_add_segment(
        &mut self,
        window_name: &str,
        segment: crate::data::widget::TextSegment,
    ) {
        if let Some(room_window) = self.room_windows.get_mut(window_name) {
            room_window.add_segment(segment);
        }
    }

    /// Finish the current line in a room component
    pub fn room_window_finish_line(&mut self, window_name: &str) {
        if let Some(room_window) = self.room_windows.get_mut(window_name) {
            room_window.finish_line();
        }
    }

    /// Finish building the current component in a room window
    pub fn room_window_finish_component(&mut self, window_name: &str) {
        if let Some(room_window) = self.room_windows.get_mut(window_name) {
            room_window.finish_component();
        }
    }

    /// Set the title of a room window
    pub fn room_window_set_title(&mut self, window_name: &str, title: String) {
        if let Some(room_window) = self.room_windows.get_mut(window_name) {
            room_window.set_title(title);
        }
    }

    /// Find a link at a given mouse position in a text or room window
    pub fn link_at_position(
        &self,
        window_name: &str,
        mouse_col: u16,
        mouse_row: u16,
        window_rect: ratatui::layout::Rect,
    ) -> Option<crate::data::LinkData> {
        // Try text window first
        if let Some(text_window) = self.text_windows.get(window_name) {
            let border_offset = if text_window.has_border() { 1 } else { 0 };

            // Bounds check within content area
            if mouse_col < window_rect.x + border_offset
                || mouse_col >= window_rect.x + window_rect.width - border_offset
                || mouse_row < window_rect.y + border_offset
                || mouse_row >= window_rect.y + window_rect.height - border_offset
            {
                return None;
            }

            let visible_height = (window_rect.height.saturating_sub(2 * border_offset)) as usize;
            let (_start_idx, visible_lines) = text_window.get_visible_lines_info(visible_height);

            let line_idx = (mouse_row - window_rect.y - border_offset) as usize;
            let col_offset = (mouse_col - window_rect.x - border_offset) as usize;

            if line_idx >= visible_lines.len() {
                return None;
            }

            let line = &visible_lines[line_idx];
            let mut col = 0usize;
            for seg in &line.segments {
                let seg_len = seg.text.chars().count();
                if col_offset >= col && col_offset < col + seg_len {
                    // Inside this segment
                    if let Some(link) = seg.link_data.clone() {
                        // Convert from TextWindow's LinkData to data layer's LinkData
                        let mut data_link = crate::data::LinkData {
                            exist_id: link.exist_id,
                            noun: link.noun,
                            text: link.text,
                            coord: link.coord,
                        };
                        // For <d> tags without cmd attribute, populate text from segment
                        if data_link.text.is_empty() {
                            data_link.text = seg.text.clone();
                        }
                        return Some(data_link);
                    }
                    return None;
                }
                col += seg_len;
            }

            return None;
        }

        // Try room window
        if let Some(room_window) = self.room_windows.get(window_name) {
            tracing::debug!(
                "Checking room window '{}' for link at ({}, {})",
                window_name,
                mouse_col,
                mouse_row
            );
            let border_offset = 1u16; // Room windows always have borders

            // Bounds check within content area
            if mouse_col < window_rect.x + border_offset
                || mouse_col >= window_rect.x + window_rect.width - border_offset
                || mouse_row < window_rect.y + border_offset
                || mouse_row >= window_rect.y + window_rect.height - border_offset
            {
                tracing::debug!("Mouse click outside room window content area");
                return None;
            }

            let wrapped_lines = room_window.get_wrapped_lines();
            let start_line = room_window.get_start_line(); // Get scroll offset
            tracing::debug!(
                "Room window has {} wrapped lines, start_line={}",
                wrapped_lines.len(),
                start_line
            );

            // Map visual row to actual wrapped line index (accounting for scroll/overflow)
            let visual_line_idx = (mouse_row - window_rect.y - border_offset) as usize;
            let line_idx = start_line + visual_line_idx;
            let col_offset = (mouse_col - window_rect.x - border_offset) as usize;

            if line_idx >= wrapped_lines.len() {
                tracing::debug!(
                    "Line index {} (visual={}, start={}) out of range",
                    line_idx,
                    visual_line_idx,
                    start_line
                );
                return None;
            }

            let line = &wrapped_lines[line_idx];
            tracing::debug!(
                "Checking line {} with {} segments, col_offset={}",
                line_idx,
                line.len(),
                col_offset
            );
            let mut col = 0usize;
            for (seg_idx, seg) in line.iter().enumerate() {
                let seg_len = seg.text.chars().count();
                tracing::debug!(
                    "  Segment {}: text='{}', col={}, len={}, has_link={}",
                    seg_idx,
                    seg.text,
                    col,
                    seg_len,
                    seg.link_data.is_some()
                );

                if col_offset >= col && col_offset < col + seg_len {
                    // Inside this segment
                    tracing::debug!("  Click is inside this segment!");
                    if let Some(link) = seg.link_data.clone() {
                        tracing::debug!(
                            "  Found link: exist_id={}, noun={}",
                            link.exist_id,
                            link.noun
                        );
                        let mut data_link = crate::data::LinkData {
                            exist_id: link.exist_id.clone(),
                            noun: link.noun.clone(),
                            text: link.text.clone(),
                            coord: link.coord.clone(),
                        };
                        // For <d> tags without cmd attribute, populate text from segment
                        if data_link.text.is_empty() {
                            data_link.text = seg.text.clone();
                        }
                        return Some(data_link);
                    }
                    tracing::debug!("  Segment has no link data");
                    return None;
                }
                col += seg_len;
            }

            tracing::debug!("No segment matched at col_offset={}", col_offset);
            return None;
        }

        // Try inventory window
        if let Some(inventory_window) = self.inventory_windows.get(window_name) {
            tracing::debug!(
                "Checking inventory window '{}' for link at ({}, {})",
                window_name,
                mouse_col,
                mouse_row
            );
            let border_offset = 1u16; // Inventory windows always have borders

            // Bounds check within content area
            if mouse_col < window_rect.x + border_offset
                || mouse_col >= window_rect.x + window_rect.width - border_offset
                || mouse_row < window_rect.y + border_offset
                || mouse_row >= window_rect.y + window_rect.height - border_offset
            {
                tracing::debug!("Mouse click outside inventory window content area");
                return None;
            }

            let wrapped_lines = inventory_window.get_wrapped_lines();
            let start_line = inventory_window.get_start_line(); // Get scroll offset
            tracing::debug!(
                "Inventory window has {} wrapped lines, start_line={}",
                wrapped_lines.len(),
                start_line
            );

            // Map visual row to actual line index (accounting for scroll/overflow)
            let visual_line_idx = (mouse_row - window_rect.y - border_offset) as usize;
            let line_idx = start_line + visual_line_idx;
            let col_offset = (mouse_col - window_rect.x - border_offset) as usize;

            if line_idx >= wrapped_lines.len() {
                tracing::debug!(
                    "Line index {} (visual={}, start={}) out of range",
                    line_idx,
                    visual_line_idx,
                    start_line
                );
                return None;
            }

            let line = &wrapped_lines[line_idx];
            tracing::debug!(
                "Checking line {} with {} segments, col_offset={}",
                line_idx,
                line.len(),
                col_offset
            );
            let mut col = 0usize;
            for (seg_idx, seg) in line.iter().enumerate() {
                let seg_len = seg.text.chars().count();
                tracing::debug!(
                    "  Segment {}: text='{}', col={}, len={}, has_link={}",
                    seg_idx,
                    seg.text,
                    col,
                    seg_len,
                    seg.link_data.is_some()
                );

                if col_offset >= col && col_offset < col + seg_len {
                    // Inside this segment
                    tracing::debug!("  Click is inside this segment!");
                    if let Some(link) = seg.link_data.clone() {
                        tracing::debug!(
                            "  Found link: exist_id={}, noun={}",
                            link.exist_id,
                            link.noun
                        );
                        let data_link = crate::data::LinkData {
                            exist_id: link.exist_id.clone(),
                            noun: link.noun.clone(),
                            text: link.text.clone(),
                            coord: link.coord.clone(),
                        };
                        return Some(data_link);
                    }
                    tracing::debug!("  Segment has no link data");
                    return None;
                }
                col += seg_len;
            }

            tracing::debug!("No segment matched at col_offset={}", col_offset);
            return None;
        }

        // Try hand widget
        if let Some(hand_widget) = self.hand_widgets.get(window_name) {
            if let Some(link) = hand_widget.link_data() {
                let border_offset = if hand_widget.has_border() { 1 } else { 0 };
                if mouse_col >= window_rect.x + border_offset
                    && mouse_col < window_rect.x + window_rect.width - border_offset
                    && mouse_row >= window_rect.y + border_offset
                    && mouse_row < window_rect.y + window_rect.height - border_offset
                {
                    return Some(link);
                }
            }
        }

        None
    }

    /// Execute search on the focused window (or main if no focus)
    pub fn execute_search(
        &mut self,
        window_name: &str,
        pattern: &str,
    ) -> Result<usize, regex::Error> {
        if let Some(text_window) = self.text_windows.get_mut(window_name) {
            // Make search case-insensitive by prepending (?i) unless user already specified flags
            let case_insensitive_pattern = if pattern.starts_with("(?") {
                pattern.to_string()
            } else {
                format!("(?i){}", pattern)
            };
            text_window.start_search(&case_insensitive_pattern)
        } else {
            Ok(0)
        }
    }

    /// Go to next search match
    pub fn next_search_match(&mut self, window_name: &str) -> bool {
        if let Some(text_window) = self.text_windows.get_mut(window_name) {
            text_window.next_match()
        } else {
            false
        }
    }

    /// Go to previous search match
    pub fn prev_search_match(&mut self, window_name: &str) -> bool {
        if let Some(text_window) = self.text_windows.get_mut(window_name) {
            text_window.prev_match()
        } else {
            false
        }
    }

    /// Clear search from all text windows
    pub fn clear_all_searches(&mut self) {
        for text_window in self.text_windows.values_mut() {
            text_window.clear_search();
        }
    }

    /// Get search info from a window (current match, total matches)
    pub fn get_search_info(&self, window_name: &str) -> Option<(usize, usize)> {
        self.text_windows
            .get(window_name)
            .and_then(|tw| tw.search_info())
    }
}

impl Frontend for TuiFrontend {
    fn poll_events(&mut self) -> Result<Vec<FrontendEvent>> {
        let mut events = Vec::new();

        // Poll for events (non-blocking)
        if event::poll(std::time::Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    // Only process key press events, not release events
                    if key.kind == KeyEventKind::Press {
                        events.push(FrontendEvent::Key {
                            code: key.code,
                            modifiers: key.modifiers,
                        });
                    }
                }
                Event::Resize(width, height) => {
                    // Apply resize debouncing to prevent excessive layout recalculations
                    if let Some((w, h)) = self.resize_debouncer.check_resize(width, height) {
                        events.push(FrontendEvent::Resize { width: w, height: h });
                    }
                }
                Event::Mouse(mouse) => {
                    events.push(FrontendEvent::Mouse {
                        kind: mouse.kind,
                        x: mouse.column,
                        y: mouse.row,
                        modifiers: mouse.modifiers,
                    });
                }
                Event::Paste(text) => {
                    events.push(FrontendEvent::Paste { text });
                }
                _ => {}
            }
        }

        // Check for pending resize (if debounce period has passed)
        if let Some((width, height)) = self.resize_debouncer.check_pending() {
            events.push(FrontendEvent::Resize { width, height });
        }

        Ok(events)
    }

    fn render(&mut self, app: &mut dyn std::any::Any) -> Result<()> {
        // Downcast to AppCore
        let app_core = app
            .downcast_mut::<AppCore>()
            .ok_or_else(|| anyhow::anyhow!("Invalid app type"))?;

        // Clone theme once so all sync tasks share the same palette
        let theme = self.cached_theme.clone();

        // Sync data from data layer into TextWindows
        self.sync_text_windows(app_core, &theme);

        // Sync CommandInput widget configuration from layout
        self.sync_command_inputs(app_core, &theme);

        // Sync room window data from AppCore
        self.sync_room_windows(app_core, &theme);

        // Sync inventory window data from AppCore
        self.sync_inventory_windows(app_core, &theme);

        // Sync spells window data from AppCore
        self.sync_spells_windows(app_core, &theme);

        // Sync progress bar data from AppCore
        self.sync_progress_bars(app_core, &theme);
        self.sync_countdowns(app_core, &theme);
        self.sync_active_effects(app_core, &theme);
        self.sync_hand_widgets(app_core, &theme);
        self.sync_spacer_widgets(app_core, &theme);
        self.sync_indicator_widgets(app_core, &theme);
        self.sync_targets_widgets(app_core, &theme);
        self.sync_players_widgets(app_core, &theme);
        self.sync_dashboard_widgets(app_core, &theme);
        self.sync_tabbed_text_windows(app_core, &theme);
        self.sync_compass_widgets(app_core, &theme);
        self.sync_injury_doll_widgets(app_core, &theme);

        // Temporarily take ownership of widgets to use in render
        let mut text_windows = std::mem::take(&mut self.text_windows);
        let mut command_inputs = std::mem::take(&mut self.command_inputs);
        let mut room_windows = std::mem::take(&mut self.room_windows);
        let mut inventory_windows = std::mem::take(&mut self.inventory_windows);
        let mut spells_windows = std::mem::take(&mut self.spells_windows);
        let mut progress_bars = std::mem::take(&mut self.progress_bars);
        let mut countdowns = std::mem::take(&mut self.countdowns);
        let mut active_effects_windows = std::mem::take(&mut self.active_effects_windows);
        let mut hand_widgets = std::mem::take(&mut self.hand_widgets);
        let mut spacer_widgets = std::mem::take(&mut self.spacer_widgets);
        let mut indicator_widgets = std::mem::take(&mut self.indicator_widgets);
        let mut targets_widgets = std::mem::take(&mut self.targets_widgets);
        let mut players_widgets = std::mem::take(&mut self.players_widgets);
        let mut dashboard_widgets = std::mem::take(&mut self.dashboard_widgets);
        let mut tabbed_text_windows = std::mem::take(&mut self.tabbed_text_windows);
        let mut compass_widgets = std::mem::take(&mut self.compass_widgets);
        let mut injury_doll_widgets = std::mem::take(&mut self.injury_doll_widgets);

        // Clone cached theme for use in render closure (cheaper than HashMap lookup + clone per widget)
        let theme_for_render = theme.clone();

        self.terminal.draw(|f| {
            use crate::data::WindowContent;
            use ratatui::layout::Rect;
            use ratatui::style::{Color, Style};
            use ratatui::text::{Line, Span};
            use ratatui::widgets::{Block, Borders, Paragraph};

            let theme = theme_for_render.clone();
            let screen_area = f.area();

            // Create stable window index mapping (sorted by window name for consistency)
            let mut window_names: Vec<&String> = app_core.ui_state.windows.keys().collect();
            window_names.sort();
            let window_index_map: std::collections::HashMap<&String, usize> = window_names
                .iter()
                .enumerate()
                .map(|(idx, name)| (*name, idx))
                .collect();

            // Render each window at its position
            for (name, window) in &app_core.ui_state.windows {
                if !window.visible {
                    continue;
                }

                let pos = &window.position;
                let area = Rect {
                    x: pos.x,
                    y: pos.y,
                    width: pos.width.min(screen_area.width.saturating_sub(pos.x)),
                    height: pos.height.min(screen_area.height.saturating_sub(pos.y)),
                };

                // Skip if area is too small
                if area.width < 1 || area.height < 1 {
                    continue;
                }

                match &window.content {
                    WindowContent::Text(_) => {
                        // Use the TextWindow widget for proper text rendering with wrapping, scrolling, etc.
                        if let Some(text_window) = text_windows.get_mut(name) {
                            // Render with selection highlighting if active
                            let focused = app_core.ui_state.focused_window.as_ref() == Some(name);
                            let window_index = window_index_map.get(name).copied().unwrap_or(0);
                            text_window.render_with_focus(
                                area,
                                f.buffer_mut(),
                                focused,
                                app_core.ui_state.selection_state.as_ref(),
                                "#4a4a4a", // Selection background color
                                window_index,
                                &theme,
                            );
                        }
                    }
                    WindowContent::CommandInput { .. } => {
                        use crate::data::ui_state::InputMode;

                        // If in Search mode, render search input instead of command input
                        if app_core.ui_state.input_mode == InputMode::Search {
                            // Get search info from focused window (if any)
                            let search_info = if let Some(focused_name) =
                                &app_core.ui_state.focused_window
                            {
                                if let Some(window) = app_core.ui_state.windows.get(focused_name) {
                                    if let WindowContent::Text(_) = &window.content {
                                        text_windows
                                            .get(focused_name)
                                            .and_then(|tw| tw.search_info())
                                            .map(|(current, total)| {
                                                format!(" [{}/{}]", current + 1, total)
                                            })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            } else {
                                // No focused window, try main
                                if let Some(window) = app_core.ui_state.windows.get("main") {
                                    if let WindowContent::Text(_) = &window.content {
                                        text_windows
                                            .get("main")
                                            .and_then(|tw| tw.search_info())
                                            .map(|(current, total)| {
                                                format!(" [{}/{}]", current + 1, total)
                                            })
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            }
                            .unwrap_or_default();

                            // Create search prompt with info
                            let prompt = format!("Search{}: ", search_info);
                            let input_text = &app_core.ui_state.search_input;
                            let cursor_pos = app_core.ui_state.search_cursor;

                            // Build display text with cursor
                            let display_text = if cursor_pos < input_text.len() {
                                format!(
                                    "{}{}{}",
                                    &input_text[..cursor_pos],
                                    "█",
                                    &input_text[cursor_pos..]
                                )
                            } else {
                                format!("{}█", input_text)
                            };

                            let search_text = Line::from(vec![
                                Span::styled(prompt, Style::default().fg(Color::Yellow)),
                                Span::raw(display_text),
                            ]);

                            let search_block = Block::default()
                                .borders(Borders::ALL)
                                .title("Search (Enter:Search, Esc:Cancel, Ctrl+PgUp/PgDn:Navigate)")
                                .style(Style::default().bg(Color::Black));

                            let search_paragraph = Paragraph::new(search_text).block(search_block);
                            f.render_widget(search_paragraph, area);
                        } else {
                            // Normal mode - render command input
                            if let Some(cmd_input) = command_inputs.get(name) {
                                cmd_input.render(area, f.buffer_mut());
                            } else {
                                tracing::error!(
                                    "CommandInput widget '{}' doesn't exist during render!",
                                    name
                                );
                                // Render error message
                                let block = Block::default()
                                    .title("Command (ERROR: widget not initialized)")
                                    .borders(Borders::ALL);
                                f.render_widget(block, area);
                            }
                        }
                    }
                    WindowContent::Progress(_) => {
                        // Use the ProgressBar widget for proper rendering
                        if let Some(progress_bar) = progress_bars.get_mut(name) {
                            progress_bar.render_themed(area, f.buffer_mut(), &theme);
                        }
                    }
                    WindowContent::Countdown(_) => {
                        // Use the Countdown widget for proper rendering
                        if let Some(countdown_widget) = countdowns.get_mut(name) {
                            countdown_widget.render(
                                area,
                                f.buffer_mut(),
                                app_core.message_processor.server_time_offset,
                                &theme,
                            );
                        }
                    }
                    WindowContent::Indicator(_) => {
                        // Use the Indicator widget for proper rendering
                        if let Some(indicator_widget) = indicator_widgets.get_mut(name) {
                            indicator_widget.render(area, f.buffer_mut());
                        }
                    }
                    WindowContent::ActiveEffects(effects_content) => {
                        // Use the ActiveEffects widget for proper rendering
                        if let Some(active_effects_widget) = active_effects_windows.get_mut(name) {
                            active_effects_widget.render(area, f.buffer_mut());
                        }
                    }
                    WindowContent::Indicator(indicator_data) => {
                        let color = if let Some(hex) = &indicator_data.color {
                            parse_hex_color(hex).unwrap_or(Color::White)
                        } else {
                            Color::White
                        };

                        let block = Block::default()
                            .title(window.name.as_str())
                            .borders(Borders::ALL);

                        let text = Span::styled(&indicator_data.status, Style::default().fg(color));
                        let paragraph = Paragraph::new(Line::from(vec![text])).block(block);
                        f.render_widget(paragraph, area);
                    }
                    WindowContent::Hand { .. } => {
                        // Use the Hand widget for proper component-based rendering
                        if let Some(hand_widget) = hand_widgets.get_mut(name) {
                            hand_widget.render(area, f.buffer_mut());
                        }
                    }
                    WindowContent::Room(_) => {
                        // Use the RoomWindow widget for proper component-based rendering
                        if let Some(room_window) = room_windows.get_mut(name) {
                            room_window.render_themed(area, f.buffer_mut(), &theme);
                        }
                    }
                    WindowContent::Inventory(_) => {
                        // Use the InventoryWindow widget for proper link rendering
                        if let Some(inventory_window) = inventory_windows.get_mut(name) {
                            inventory_window.render_themed(area, f.buffer_mut(), &theme);
                        }
                    }
                    WindowContent::Spells(_) => {
                        // Use the SpellsWindow widget for proper link rendering
                        if let Some(spells_window) = spells_windows.get_mut(name) {
                            spells_window.render_themed(area, f.buffer_mut(), &theme);
                        }
                    }
                    WindowContent::Targets { .. } => {
                        // Use the Targets widget
                        if let Some(targets_widget) = targets_widgets.get_mut(name) {
                            targets_widget.render(area, f.buffer_mut());
                        }
                    }
                    WindowContent::Players { .. } => {
                        // Use the Players widget
                        if let Some(players_widget) = players_widgets.get_mut(name) {
                            players_widget.render(area, f.buffer_mut());
                        }
                    }
                    WindowContent::Dashboard { .. } => {
                        // Use the Dashboard widget
                        if let Some(dashboard_widget) = dashboard_widgets.get_mut(name) {
                            dashboard_widget.render(area, f.buffer_mut());
                        }
                    }
                    WindowContent::TabbedText(_) => {
                        // Use the TabbedTextWindow widget
                        if let Some(tabbed_window) = tabbed_text_windows.get_mut(name) {
                            tabbed_window.render(area, f.buffer_mut());
                        }
                    }
                    WindowContent::Compass(_) => {
                        // Use the Compass widget
                        if let Some(compass_widget) = compass_widgets.get_mut(name) {
                            compass_widget.render(area, f.buffer_mut());
                        }
                    }
                    WindowContent::InjuryDoll(_) => {
                        // Use the InjuryDoll widget
                        if let Some(injury_doll_widget) = injury_doll_widgets.get_mut(name) {
                            injury_doll_widget.render(area, f.buffer_mut());
                        }
                    }
                    WindowContent::Empty => {
                        // Check if this is a spacer widget
                        if window.widget_type == crate::data::WidgetType::Spacer {
                            if let Some(spacer_widget) = spacer_widgets.get_mut(name) {
                                spacer_widget.render(area, f.buffer_mut());
                            }
                        }
                        // Otherwise render nothing (empty placeholder)
                    }
                    _ => {
                        // Other widget types not yet implemented
                        let block = Block::default()
                            .title(window.name.as_str())
                            .borders(Borders::ALL);
                        f.render_widget(block, area);
                    }
                }
            }

            // Render popup menu if active
            if let Some(ref popup_menu) = app_core.ui_state.popup_menu {
                // Convert from ui_state::PopupMenu to rendering popup_menu::PopupMenu
                // Filter out disabled items
                let menu_items: Vec<popup_menu::MenuItem> = popup_menu
                    .items
                    .iter()
                    .filter(|item| !item.disabled)
                    .map(|item| popup_menu::MenuItem {
                        text: item.text.clone(),
                        command: item.command.clone(),
                    })
                    .collect();

                let render_menu = popup_menu::PopupMenu::with_selected(
                    menu_items,
                    popup_menu.position,
                    popup_menu.selected,
                );
                render_menu.render(screen_area, f.buffer_mut(), &theme);
            }

            // Render submenu if active (level 2)
            if let Some(ref submenu) = app_core.ui_state.submenu {
                // Filter out disabled items
                let menu_items: Vec<popup_menu::MenuItem> = submenu
                    .items
                    .iter()
                    .filter(|item| !item.disabled)
                    .map(|item| popup_menu::MenuItem {
                        text: item.text.clone(),
                        command: item.command.clone(),
                    })
                    .collect();

                let render_submenu = popup_menu::PopupMenu::with_selected(
                    menu_items,
                    submenu.position,
                    submenu.selected,
                );
                render_submenu.render(screen_area, f.buffer_mut(), &theme);
            }

            // Render nested submenu if active (level 3)
            if let Some(ref nested_submenu) = app_core.ui_state.nested_submenu {
                // Filter out disabled items
                let menu_items: Vec<popup_menu::MenuItem> = nested_submenu
                    .items
                    .iter()
                    .filter(|item| !item.disabled)
                    .map(|item| popup_menu::MenuItem {
                        text: item.text.clone(),
                        command: item.command.clone(),
                    })
                    .collect();

                let render_nested = popup_menu::PopupMenu::with_selected(
                    menu_items,
                    nested_submenu.position,
                    nested_submenu.selected,
                );
                render_nested.render(screen_area, f.buffer_mut(), &theme);
            }

            // Render browsers and forms if active
            if let Some(ref mut highlight_browser) = self.highlight_browser {
                highlight_browser.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref mut highlight_form) = self.highlight_form {
                highlight_form.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref mut keybind_browser) = self.keybind_browser {
                keybind_browser.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref mut keybind_form) = self.keybind_form {
                keybind_form.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref mut color_palette_browser) = self.color_palette_browser {
                color_palette_browser.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref mut color_form) = self.color_form {
                color_form.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref mut uicolors_browser) = self.uicolors_browser {
                uicolors_browser.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref mut spell_color_browser) = self.spell_color_browser {
                spell_color_browser.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref mut spell_color_form) = self.spell_color_form {
                spell_color_form.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref mut theme_editor) = self.theme_editor {
                theme_editor.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }
            if let Some(ref theme_browser) = self.theme_browser {
                use ratatui::widgets::Widget;
                f.render_widget(theme_browser, screen_area);
            }
            if let Some(ref mut settings_editor) = self.settings_editor {
                settings_editor.render(screen_area, f.buffer_mut(), &app_core.config, &theme);
            }

            // Render window editor if active
            if let Some(ref mut window_editor) = self.window_editor {
                // Window editor handles its own positioning and sizing (70x20)
                let editor_theme = theme.to_editor_theme();
                window_editor.render(screen_area, f.buffer_mut(), &editor_theme);
            }
        })?;

        // Restore widgets
        self.text_windows = text_windows;
        self.command_inputs = command_inputs;
        self.room_windows = room_windows;
        self.inventory_windows = inventory_windows;
        self.spells_windows = spells_windows;
        self.progress_bars = progress_bars;
        self.countdowns = countdowns;
        self.active_effects_windows = active_effects_windows;
        self.hand_widgets = hand_widgets;
        self.spacer_widgets = spacer_widgets;
        self.indicator_widgets = indicator_widgets;
        self.targets_widgets = targets_widgets;
        self.players_widgets = players_widgets;
        self.dashboard_widgets = dashboard_widgets;
        self.tabbed_text_windows = tabbed_text_windows;
        self.compass_widgets = compass_widgets;
        self.injury_doll_widgets = injury_doll_widgets;

        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            crossterm::event::DisableMouseCapture
        )?;
        Ok(())
    }

    fn size(&self) -> (u16, u16) {
        let rect = self.terminal.size().unwrap_or_default();
        (rect.width, rect.height)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
