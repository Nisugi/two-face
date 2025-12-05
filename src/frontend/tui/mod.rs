//! TUI Frontend - Terminal UI using ratatui
//!
//! This module implements the Frontend trait for terminal rendering.

mod active_effects;
pub mod crossterm_bridge;
pub mod textarea_bridge;
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
pub mod menu_actions;
pub mod menu_builders;
mod performance_stats;
mod players;
mod popup_menu;
mod progress_bar;
mod quickbar;
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
mod theme_cache;
pub mod uicolors_browser;
pub mod window_editor;
mod widget_manager;
mod input_handlers;

use crate::frontend::{Frontend, FrontendEvent};
pub mod widget_traits;
use theme_cache::ThemeCache;
use widget_manager::WidgetManager;
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
    /// Widget manager - handles all widget caches and synchronization
    widget_manager: WidgetManager,
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
    /// Theme cache to avoid HashMap lookup + clone every render
    theme_cache: ThemeCache,
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

fn color_to_hex_string(color: &crate::frontend::common::Color) -> Option<String> {
    // Color is now a simple RGB struct
    Some(format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b))
}

// OLD functions no longer needed after Phase 2 refactoring
#[allow(dead_code)]
fn _old_color_to_hex_string(color: &ratatui::style::Color) -> Option<String> {
    _old_color_to_rgb(color).map(|(r, g, b)| format!("#{:02x}{:02x}{:02x}", r, g, b))
}

#[allow(dead_code)]
fn _old_color_to_rgb(color: &ratatui::style::Color) -> Option<(u8, u8, u8)> {
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
    base: &crate::frontend::common::Color,
    target: &crate::frontend::common::Color,
    ratio: f32,
) -> Option<String> {
    // Color is now a simple RGB struct
    let (br, bg, bb) = (base.r, base.g, base.b);
    let (tr, tg, tb) = (target.r, target.g, target.b);
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
        let last_time = self.last_resize_time?;

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
            widget_manager: WidgetManager::new(),
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
            resize_debouncer: ResizeDebouncer::new(300), // 300ms debounce
            theme_cache: ThemeCache::new(),
        })
    }

    /// Update cached theme (call this when theme changes via command/browser)
    pub fn update_theme_cache(&mut self, theme_id: String, theme: crate::theme::AppTheme) {
        self.theme_cache.update(theme_id, theme);
    }

    /// Get the terminal size (width, height)
    pub fn size(&self) -> (u16, u16) {
        let size = self.terminal.size().unwrap_or_default();
        (size.width, size.height)
    }

    /// Navigate to next tab in all tabbed windows
    pub fn next_tab_all(&mut self) {
        for widget in self.widget_manager.tabbed_text_windows.values_mut() {
            widget.next_tab();
        }
    }

    /// Navigate to previous tab in all tabbed windows
    pub fn prev_tab_all(&mut self) {
        for widget in self.widget_manager.tabbed_text_windows.values_mut() {
            widget.prev_tab();
        }
    }

    /// Navigate to next tab with unread messages (searches all tabbed windows)
    /// Returns true if found, false if no unread tabs
    pub fn go_to_next_unread_tab(&mut self) -> bool {
        for widget in self.widget_manager.tabbed_text_windows.values_mut() {
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
                let text_window = self.widget_manager.text_windows.entry(name.clone()).or_insert_with(|| {
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
                        tw.set_content_align(def.base().content_align.clone());
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
                    text_window.set_content_align(def.base().content_align.clone());
                }

                // Update width for proper wrapping
                text_window.set_width(window.position.width);

                // Get last synced generation
                let last_synced_gen = self.widget_manager.last_synced_generation.get(name).copied().unwrap_or(0);
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
                    self.widget_manager.last_synced_generation
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
                if !self.widget_manager.room_windows.contains_key(name) {
                    let mut room_window = room_window::RoomWindow::new("Room".to_string());

                    // Configure RoomWindow with settings from WindowDef
                    if let Some(crate::config::WindowDef::Room { data, .. }) = window_def {
                        // Set component visibility from config
                        room_window.set_component_visible("room desc", data.show_desc);
                        room_window.set_component_visible("room objs", data.show_objs);
                        room_window.set_component_visible("room players", data.show_players);
                        room_window.set_component_visible("room exits", data.show_exits);
                    }

                    self.widget_manager.room_windows.insert(name.clone(), room_window);
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
            let cmd_input = self.widget_manager.command_inputs.entry(name.clone()).or_insert_with(|| {
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
                if !self.widget_manager.inventory_windows.contains_key(name) {
                    let inv_window =
                        inventory_window::InventoryWindow::new(text_content.title.clone());
                    self.widget_manager.inventory_windows.insert(name.clone(), inv_window);
                    tracing::debug!("Created InventoryWindow widget for '{}'", name);
                }

                // Update configuration and content from WindowDef if present
                if let Some(inv_window) = self.widget_manager.inventory_windows.get_mut(name) {
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
                        self.widget_manager.last_synced_generation.get(name).copied().unwrap_or(0);
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
                        self.widget_manager.last_synced_generation
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
                if !self.widget_manager.spells_windows.contains_key(name) {
                    let spells_window =
                        spells_window::SpellsWindow::new(text_content.title.clone());
                    self.widget_manager.spells_windows.insert(name.clone(), spells_window);
                    tracing::debug!("Created SpellsWindow widget for '{}'", name);
                }

                // Update configuration and content from WindowDef if present
                if let Some(spells_window) = self.widget_manager.spells_windows.get_mut(name) {
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
                        self.widget_manager.last_synced_generation.get(name).copied().unwrap_or(0);
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
                        self.widget_manager.last_synced_generation
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

    /// Sync QuickBar widgets - create/configure and update content
    fn sync_quickbar_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find QuickBar windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::QuickBar { content } = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create QuickBar widget for this window
                if !self.widget_manager.quickbar_widgets.contains_key(name) {
                    // Extract QuickBarWidgetData from WindowDef
                    let data = if let Some(crate::config::WindowDef::QuickBar { data, .. }) =
                        window_def
                    {
                        data.clone()
                    } else {
                        // Fallback: create default data
                        crate::config::QuickBarWidgetData {
                            active_bar: "quick".to_string(),
                            bars: std::collections::HashMap::new(),
                            default_bar: "quick".to_string(),
                            scroll_offset: 0,
                        }
                    };

                    let quickbar_widget = quickbar::QuickBar::new(data);
                    self.widget_manager.quickbar_widgets.insert(name.clone(), quickbar_widget);
                    tracing::debug!("Created QuickBar widget for '{}'", name);
                }

                // Update configuration and content from WindowDef if present
                if let Some(quickbar_widget) = self.widget_manager.quickbar_widgets.get_mut(name) {
                    if let Some(def) = window_def {
                        // Update content if it changed
                        if !content.is_empty() {
                            let available_width = def.base().cols.saturating_sub(if def.base().show_border { 2 } else { 0 }) as usize;
                            quickbar_widget.set_content(content, available_width);
                            tracing::debug!(
                                "Updated QuickBar '{}' content: {} chars, wrapped to {} lines",
                                name,
                                content.len(),
                                quickbar_widget.total_lines
                            );
                        }
                    }
                } else {
                    tracing::warn!(
                        "QuickBar widget '{}' not found in quickbar_widgets HashMap!",
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
                if !self.widget_manager.progress_bars.contains_key(name) {
                    let label = window_def
                        .and_then(|def| def.base().title.as_ref()).cloned()
                        .unwrap_or_else(|| progress_data.label.clone());

                    let bar = progress_bar::ProgressBar::new(&label);
                    self.widget_manager.progress_bars.insert(name.clone(), bar);
                    tracing::debug!("Created ProgressBar widget for '{}'", name);
                }

                // Update configuration and value
                if let Some(progress_bar) = self.widget_manager.progress_bars.get_mut(name) {
                    // Set value from game data
                    if let Some(ref custom_text) = progress_data.color {
                        // color field is being used as custom text (e.g., "clear as a bell")
                        progress_bar.set_value_with_text(
                            progress_data.value,
                            progress_data.max,
                            Some(custom_text.clone()),
                        );
                    } else {
                        progress_bar
                            .set_value(progress_data.value, progress_data.max);
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
                if !self.widget_manager.countdowns.contains_key(name) {
                    let label = window_def
                        .and_then(|def| def.base().title.as_ref()).cloned()
                        .unwrap_or_else(|| name.clone());

                    let countdown = countdown::Countdown::new(&label);
                    self.widget_manager.countdowns.insert(name.clone(), countdown);
                    tracing::debug!("Created Countdown widget for '{}'", name);
                }

                // Update configuration and value
                if let Some(countdown_widget) = self.widget_manager.countdowns.get_mut(name) {
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
                if !self.widget_manager.active_effects_windows.contains_key(name) {
                    let label = window_def
                        .and_then(|def| def.base().title.as_ref()).cloned()
                        .unwrap_or_else(|| name.clone());

                    let widget = active_effects::ActiveEffects::new(
                        &label,
                        effects_content.category.clone(),
                    );
                    self.widget_manager.active_effects_windows.insert(name.clone(), widget);
                    tracing::debug!("Created ActiveEffects widget for '{}'", name);
                }

                // Update effects data and configuration
                if let Some(widget) = self.widget_manager.active_effects_windows.get_mut(name) {
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
                if !self.widget_manager.spacer_widgets.contains_key(name) {
                    let widget = spacer::Spacer::new();
                    self.widget_manager.spacer_widgets.insert(name.clone(), widget);
                }

                // Update spacer widget configuration
                if let Some(spacer_widget) = self.widget_manager.spacer_widgets.get_mut(name) {
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
                if !self.widget_manager.indicator_widgets.contains_key(name) {
                    let widget = indicator::Indicator::new(name);
                    self.widget_manager.indicator_widgets.insert(name.clone(), widget);
                }

                // Update indicator widget content and configuration
                if let Some(indicator_widget) = self.widget_manager.indicator_widgets.get_mut(name) {
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
                if !self.widget_manager.targets_widgets.contains_key(name) {
                    let widget = targets::Targets::new(name);
                    self.widget_manager.targets_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.targets_widgets.get_mut(name) {
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
                if !self.widget_manager.players_widgets.contains_key(name) {
                    let widget = players::Players::new(name);
                    self.widget_manager.players_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.players_widgets.get_mut(name) {
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
                if !self.widget_manager.dashboard_widgets.contains_key(name) {
                    // Default to horizontal layout - can be configured via WindowDef later
                    let widget =
                        dashboard::Dashboard::new(name, dashboard::DashboardLayout::Horizontal);
                    self.widget_manager.dashboard_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.dashboard_widgets.get_mut(name) {
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
                        widget.set_content_align(window_def.base().content_align.clone());
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
                if !self.widget_manager.tabbed_text_windows.contains_key(name) {
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
                    self.widget_manager.tabbed_text_windows.insert(name.clone(), widget);
                }

                // Apply configuration
                if let Some(widget) = self.widget_manager.tabbed_text_windows.get_mut(name) {
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
                        widget.set_content_align(window_def.base().content_align.clone());
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
                if !self.widget_manager.compass_widgets.contains_key(name) {
                    let widget = compass::Compass::new(name);
                    self.widget_manager.compass_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.compass_widgets.get_mut(name) {
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
                        widget.set_content_align(window_def.base().content_align.clone());
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
                if !self.widget_manager.injury_doll_widgets.contains_key(name) {
                    let widget = injury_doll::InjuryDoll::new(name);
                    self.widget_manager.injury_doll_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.injury_doll_widgets.get_mut(name) {
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
                if !self.widget_manager.hand_widgets.contains_key(name) {
                    // Determine hand type based on window name
                    let hand_type = match name.as_str() {
                        "left_hand" => hand::HandType::Left,
                        "right_hand" => hand::HandType::Right,
                        "spell_hand" => hand::HandType::Spell,
                        _ => hand::HandType::Left, // Default fallback
                    };

                    let widget = hand::Hand::new(name, hand_type);
                    self.widget_manager.hand_widgets.insert(name.clone(), widget);
                }

                // Update hand widget content
                if let Some(hand_widget) = self.widget_manager.hand_widgets.get_mut(name) {
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

            if let Some(room_window) = self.widget_manager.room_windows.get_mut(window_name) {
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
        if let Some(text_window) = self.widget_manager.text_windows.get_mut(window_name) {
            if lines > 0 {
                text_window.scroll_up(lines as usize);
            } else if lines < 0 {
                text_window.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try room window
        if let Some(room_window) = self.widget_manager.room_windows.get_mut(window_name) {
            if lines > 0 {
                room_window.scroll_up(lines as usize);
            } else if lines < 0 {
                room_window.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try inventory window
        if let Some(inventory_window) = self.widget_manager.inventory_windows.get_mut(window_name) {
            if lines > 0 {
                inventory_window.scroll_up(lines as usize);
            } else if lines < 0 {
                inventory_window.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try spells window
        if let Some(spells_window) = self.widget_manager.spells_windows.get_mut(window_name) {
            if lines > 0 {
                spells_window.scroll_up(lines as usize);
            } else if lines < 0 {
                spells_window.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try active_effects widget
        if let Some(active_effects) = self.widget_manager.active_effects_windows.get_mut(window_name) {
            if lines > 0 {
                active_effects.scroll_up(lines as usize);
            } else if lines < 0 {
                active_effects.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try targets widget
        if let Some(targets) = self.widget_manager.targets_widgets.get_mut(window_name) {
            if lines > 0 {
                targets.scroll_up(lines as usize);
            } else if lines < 0 {
                targets.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try players widget
        if let Some(players) = self.widget_manager.players_widgets.get_mut(window_name) {
            if lines > 0 {
                players.scroll_up(lines as usize);
            } else if lines < 0 {
                players.scroll_down((-lines) as usize);
            }
            return;
        }

        // Try quickbar widget
        if let Some(quickbar) = self.widget_manager.quickbar_widgets.get_mut(window_name) {
            // QuickBar scrolls 1 row at a time
            // Use a safe default for visible height (will be accurate during actual display)
            let visible_rows = 5;
            if lines > 0 {
                for _ in 0..lines {
                    quickbar.scroll_up();
                }
            } else if lines < 0 {
                for _ in 0..(-lines) {
                    quickbar.scroll_down(visible_rows);
                }
            }
            return;
        }

        // Try tabbed text window
        if let Some(tabbed_window) = self.widget_manager.tabbed_text_windows.get_mut(window_name) {
            if lines > 0 {
                tabbed_window.scroll_up(lines as usize);
            } else if lines < 0 {
                tabbed_window.scroll_down((-lines) as usize);
            }
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
        let text_window = self.widget_manager.text_windows.get(window_name)?;
        text_window.mouse_to_text_coords(mouse_col, mouse_row, window_rect)
    }

    /// Handle a tab click for a tabbed text window; returns true if a tab was activated.
    pub fn handle_tabbed_click(
        &mut self,
        window_name: &str,
        window_rect: ratatui::layout::Rect,
        mouse_col: u16,
        mouse_row: u16,
    ) -> bool {
        if let Some(tabbed_window) = self.widget_manager.tabbed_text_windows.get_mut(window_name) {
            return tabbed_window.handle_mouse_click(window_rect, mouse_col, mouse_row);
        }
        false
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
        let text_window = self.widget_manager.text_windows.get(window_name)?;
        Some(text_window.extract_selection_text(start_line, start_col, end_line, end_col))
    }

    /// Ensure a command input widget exists (should be called during init)
    pub fn ensure_command_input_exists(&mut self, window_name: &str) {
        if !self.widget_manager.command_inputs.contains_key(window_name) {
            let mut cmd_input = command_input::CommandInput::new(1000);
            cmd_input.set_title("Command".to_string());
            self.widget_manager.command_inputs
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
        if !self.widget_manager.command_inputs.contains_key(window_name) {
            tracing::warn!(
                "CommandInput widget '{}' doesn't exist, creating it now",
                window_name
            );
            self.ensure_command_input_exists(window_name);
        }

        if let Some(cmd_input) = self.widget_manager.command_inputs.get_mut(window_name) {
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
                                            .is_some_and(|c| c.is_whitespace())
                                    {
                                        count += 1;
                                        pos -= 1;
                                    }

                                    // Delete word
                                    while pos > 0
                                        && chars
                                            .get(pos.saturating_sub(1))
                                            .is_some_and(|c| !c.is_whitespace())
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
        self.widget_manager.command_inputs.get_mut(window_name)?.submit()
    }

    /// Load command history for a character
    pub fn command_input_load_history(
        &mut self,
        window_name: &str,
        character: Option<&str>,
    ) -> Result<()> {
        if let Some(cmd_input) = self.widget_manager.command_inputs.get_mut(window_name) {
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
        if let Some(cmd_input) = self.widget_manager.command_inputs.get(window_name) {
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
        if !self.widget_manager.room_windows.contains_key(window_name) {
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

            self.widget_manager.room_windows
                .insert(window_name.to_string(), room_window);
            tracing::debug!("Created RoomWindow widget for '{}'", window_name);
        }
    }

    /// Clear all components in a room window (called when pushStream id="room")
    pub fn room_window_clear_components(&mut self, window_name: &str) {
        if let Some(room_window) = self.widget_manager.room_windows.get_mut(window_name) {
            room_window.clear_all_components();
            tracing::debug!("Cleared all components for room window '{}'", window_name);
        }
    }

    /// Start building a room component
    pub fn room_window_start_component(&mut self, window_name: &str, component_id: String) {
        if let Some(room_window) = self.widget_manager.room_windows.get_mut(window_name) {
            room_window.start_component(component_id);
        }
    }

    /// Add a segment to the current component in a room window
    pub fn room_window_add_segment(
        &mut self,
        window_name: &str,
        segment: crate::data::widget::TextSegment,
    ) {
        if let Some(room_window) = self.widget_manager.room_windows.get_mut(window_name) {
            room_window.add_segment(segment);
        }
    }

    /// Finish the current line in a room component
    pub fn room_window_finish_line(&mut self, window_name: &str) {
        if let Some(room_window) = self.widget_manager.room_windows.get_mut(window_name) {
            room_window.finish_line();
        }
    }

    /// Finish building the current component in a room window
    pub fn room_window_finish_component(&mut self, window_name: &str) {
        if let Some(room_window) = self.widget_manager.room_windows.get_mut(window_name) {
            room_window.finish_component();
        }
    }

    /// Set the title of a room window
    pub fn room_window_set_title(&mut self, window_name: &str, title: String) {
        if let Some(room_window) = self.widget_manager.room_windows.get_mut(window_name) {
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
        if let Some(text_window) = self.widget_manager.text_windows.get(window_name) {
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
        if let Some(room_window) = self.widget_manager.room_windows.get(window_name) {
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
        if let Some(inventory_window) = self.widget_manager.inventory_windows.get(window_name) {
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
        if let Some(hand_widget) = self.widget_manager.hand_widgets.get(window_name) {
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

        // Try quickbar widget
        if let Some(quickbar) = self.widget_manager.quickbar_widgets.get(window_name) {
            let border_offset = 1u16; // Assume border for now

            // Bounds check within content area
            if mouse_col >= window_rect.x + border_offset
                && mouse_col < window_rect.x + window_rect.width - border_offset
                && mouse_row >= window_rect.y + border_offset
                && mouse_row < window_rect.y + window_rect.height - border_offset
            {
                // Convert to widget-relative coordinates
                let widget_row = mouse_row - window_rect.y - border_offset;
                let widget_col = mouse_col - window_rect.x - border_offset;

                if let Some(link) = quickbar.get_link_at(widget_row, widget_col) {
                    return Some(crate::data::LinkData {
                        exist_id: link.exist_id.clone(),
                        noun: link.noun.clone(),
                        text: link.text.clone(),
                        coord: link.coord.clone(),
                    });
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
        if let Some(text_window) = self.widget_manager.text_windows.get_mut(window_name) {
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
        if let Some(text_window) = self.widget_manager.text_windows.get_mut(window_name) {
            text_window.next_match()
        } else {
            false
        }
    }

    /// Go to previous search match
    pub fn prev_search_match(&mut self, window_name: &str) -> bool {
        if let Some(text_window) = self.widget_manager.text_windows.get_mut(window_name) {
            text_window.prev_match()
        } else {
            false
        }
    }

    /// Clear search from all text windows
    pub fn clear_all_searches(&mut self) {
        for text_window in self.widget_manager.text_windows.values_mut() {
            text_window.clear_search();
        }
    }

    /// Get search info from a window (current match, total matches)
    pub fn get_search_info(&self, window_name: &str) -> Option<(usize, usize)> {
        self.widget_manager.text_windows
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
                        if let Some(code) = crossterm_bridge::convert_keycode(key.code) {
                            events.push(FrontendEvent::Key {
                                code,
                                modifiers: crossterm_bridge::convert_modifiers(key.modifiers),
                            });
                        }
                    }
                }
                Event::Resize(width, height) => {
                    // Apply resize debouncing to prevent excessive layout recalculations
                    if let Some((w, h)) = self.resize_debouncer.check_resize(width, height) {
                        events.push(FrontendEvent::Resize { width: w, height: h });
                    }
                }
                Event::Mouse(mouse) => {
                    // Convert crossterm MouseEvent to frontend-agnostic MouseEvent
                    if let Some(kind) = crossterm_bridge::convert_mouse_kind(mouse.kind) {
                        let modifiers = crossterm_bridge::convert_modifiers(mouse.modifiers);
                        let mouse_event = crate::frontend::common::MouseEvent::new(
                            kind,
                            mouse.column,
                            mouse.row,
                            modifiers,
                        );
                        events.push(FrontendEvent::Mouse(mouse_event));
                    }
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
        let theme = self.theme_cache.get_theme().clone();

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

        // Sync quickbar widgets from AppCore
        self.sync_quickbar_widgets(app_core, &theme);

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
        let mut text_windows = std::mem::take(&mut self.widget_manager.text_windows);
        let command_inputs = std::mem::take(&mut self.widget_manager.command_inputs);
        let mut room_windows = std::mem::take(&mut self.widget_manager.room_windows);
        let mut inventory_windows = std::mem::take(&mut self.widget_manager.inventory_windows);
        let mut spells_windows = std::mem::take(&mut self.widget_manager.spells_windows);
        let mut progress_bars = std::mem::take(&mut self.widget_manager.progress_bars);
        let mut countdowns = std::mem::take(&mut self.widget_manager.countdowns);
        let mut active_effects_windows = std::mem::take(&mut self.widget_manager.active_effects_windows);
        let mut hand_widgets = std::mem::take(&mut self.widget_manager.hand_widgets);
        let mut spacer_widgets = std::mem::take(&mut self.widget_manager.spacer_widgets);
        let mut indicator_widgets = std::mem::take(&mut self.widget_manager.indicator_widgets);
        let mut targets_widgets = std::mem::take(&mut self.widget_manager.targets_widgets);
        let mut players_widgets = std::mem::take(&mut self.widget_manager.players_widgets);
        let mut dashboard_widgets = std::mem::take(&mut self.widget_manager.dashboard_widgets);
        let mut tabbed_text_windows = std::mem::take(&mut self.widget_manager.tabbed_text_windows);
        let mut compass_widgets = std::mem::take(&mut self.widget_manager.compass_widgets);
        let mut injury_doll_widgets = std::mem::take(&mut self.widget_manager.injury_doll_widgets);
        let mut quickbar_widgets = std::mem::take(&mut self.widget_manager.quickbar_widgets);

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
                    WindowContent::QuickBar { .. } => {
                        // Use the QuickBar widget
                        if let Some(quickbar_widget) = quickbar_widgets.get_mut(name) {
                            // QuickBar render method handles border styling internally
                            quickbar_widget.render(area, f.buffer_mut(), None);
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
        self.widget_manager.text_windows = text_windows;
        self.widget_manager.command_inputs = command_inputs;
        self.widget_manager.room_windows = room_windows;
        self.widget_manager.inventory_windows = inventory_windows;
        self.widget_manager.spells_windows = spells_windows;
        self.widget_manager.progress_bars = progress_bars;
        self.widget_manager.countdowns = countdowns;
        self.widget_manager.active_effects_windows = active_effects_windows;
        self.widget_manager.hand_widgets = hand_widgets;
        self.widget_manager.spacer_widgets = spacer_widgets;
        self.widget_manager.indicator_widgets = indicator_widgets;
        self.widget_manager.targets_widgets = targets_widgets;
        self.widget_manager.players_widgets = players_widgets;
        self.widget_manager.dashboard_widgets = dashboard_widgets;
        self.widget_manager.tabbed_text_windows = tabbed_text_windows;
        self.widget_manager.compass_widgets = compass_widgets;
        self.widget_manager.injury_doll_widgets = injury_doll_widgets;
        self.widget_manager.quickbar_widgets = quickbar_widgets;

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

// TUI-specific methods (not part of Frontend trait)
impl TuiFrontend {
    /// Handle mouse events (extracted from main.rs Phase 4.1)
    /// Returns (handled, optional_command)
    pub fn handle_mouse_event(
        &mut self,
        mouse_event: &crate::frontend::MouseEvent,
        app_core: &mut crate::core::AppCore,
        handle_menu_action_fn: impl Fn(&mut crate::core::AppCore, &mut Self, &str) -> Result<()>,
    ) -> Result<(bool, Option<String>)> {
        use crate::data::ui_state::InputMode;
        use crate::frontend::MouseEventKind;
        use crate::data::{DragOperation, LinkDragState, MouseDragState, PendingLinkClick, window::WidgetType};
        use ratatui::layout::Rect;

        let kind = &mouse_event.kind;
        let x = &mouse_event.column;
        let y = &mouse_event.row;
        let modifiers = &mouse_event.modifiers;

        // Create stable window index mapping (sorted by window name for consistency)
        let mut window_names: Vec<&String> = app_core.ui_state.windows.keys().collect();
        window_names.sort();
        let window_index_map: std::collections::HashMap<&String, usize> = window_names
            .iter()
            .enumerate()
            .map(|(idx, name)| (*name, idx))
            .collect();

        // Handle window editor mouse events first (if open)
        if self.window_editor.is_some() {
            let (width, height) = self.size();
            let area = ratatui::layout::Rect {
                x: 0,
                y: 0,
                width,
                height,
            };

            if let Some(ref mut window_editor) = self.window_editor {
                match kind {
                    MouseEventKind::Down(crate::frontend::MouseButton::Left) => {
                        window_editor.handle_mouse(*x, *y, true, area);
                        app_core.needs_render = true;
                        return Ok((true, None));
                    }
                    MouseEventKind::Drag(crate::frontend::MouseButton::Left) => {
                        window_editor.handle_mouse(*x, *y, true, area);
                        app_core.needs_render = true;
                        return Ok((true, None));
                    }
                    MouseEventKind::Up(crate::frontend::MouseButton::Left) => {
                        window_editor.handle_mouse(*x, *y, false, area);
                        app_core.needs_render = true;
                        return Ok((true, None));
                    }
                    _ => {}
                }
            }
        }

        match kind {
            MouseEventKind::ScrollUp => {
                // Find which window the mouse is over
                let mut target_window = "main".to_string();
                for (name, window) in &app_core.ui_state.windows {
                    let pos = &window.position;
                    if *x >= pos.x
                        && *x < pos.x + pos.width
                        && *y >= pos.y
                        && *y < pos.y + pos.height
                    {
                        target_window = name.clone();
                        break;
                    }
                }
                self.scroll_window(&target_window, 10);
                app_core.needs_render = true;
                return Ok((true, None));
            }
            MouseEventKind::ScrollDown => {
                // Find which window the mouse is over
                let mut target_window = "main".to_string();
                for (name, window) in &app_core.ui_state.windows {
                    let pos = &window.position;
                    if *x >= pos.x
                        && *x < pos.x + pos.width
                        && *y >= pos.y
                        && *y < pos.y + pos.height
                    {
                        target_window = name.clone();
                        break;
                    }
                }
                self.scroll_window(&target_window, -10);
                app_core.needs_render = true;
                return Ok((true, None));
            }
            MouseEventKind::Down(crate::frontend::MouseButton::Left) => {
                // If in menu mode, handle menu clicks first
                if app_core.ui_state.input_mode == InputMode::Menu {
                    let mut clicked_item = None;

                    // Check popup menu first (top layer)
                    if let Some(ref menu) = app_core.ui_state.popup_menu {
                        let pos = menu.get_position();
                        let menu_height = menu.get_items().len() as u16 + 2; // +2 for borders
                        let menu_width = menu
                            .get_items()
                            .iter()
                            .map(|item| item.text.len())
                            .max()
                            .unwrap_or(10)
                            as u16
                            + 4; // +4 for borders and padding

                        let menu_area = (pos.0, pos.1, menu_width, menu_height);

                        if let Some(index) = menu.check_click(*x, *y, menu_area) {
                            clicked_item = menu.get_items().get(index).cloned();
                        }
                    }

                    if let Some(item) = clicked_item {
                        let command = item.command.clone();
                        tracing::info!(
                            "Menu item clicked: {} (command: {})",
                            item.text,
                            command
                        );

                        // Handle command same way as Enter key
                        if let Some(submenu_name) = command.strip_prefix("menu:") {
                            // Config menu submenu
                            tracing::debug!("Clicked config submenu: {}", submenu_name);
                            app_core.ui_state.popup_menu = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        } else if let Some(category) = command.strip_prefix("__SUBMENU__") {
                            // Context menu or .menu submenu
                            // Try build_submenu first (for .menu categories)
                            let items = app_core.build_submenu(category);
                            let items = if !items.is_empty() {
                                items
                            } else if let Some(items) = app_core.menu_categories.get(category) {
                                items.clone()
                            } else {
                                Vec::new()
                            };

                            if !items.is_empty() {
                                let position = app_core
                                    .ui_state
                                    .popup_menu
                                    .as_ref()
                                    .map(|m| m.get_position())
                                    .unwrap_or((40, 12));
                                let submenu_pos = (position.0 + 2, position.1);
                                app_core.ui_state.submenu =
                                    Some(crate::data::ui_state::PopupMenu::new(
                                        items,
                                        submenu_pos,
                                    ));
                                tracing::info!(
                                    "Opened submenu: {}",
                                    category
                                );
                            }
                        } else if !command.is_empty() {
                            // Close menu first
                            app_core.ui_state.popup_menu = None;
                            app_core.ui_state.submenu = None;
                            app_core.ui_state.nested_submenu = None;
                            app_core.ui_state.input_mode = InputMode::Normal;

                            // Check if this is an internal action or game command
                            if command.starts_with("action:") {
                                // Internal action - handle it
                                if let Err(e) = handle_menu_action_fn(app_core, self, &command) {
                                    tracing::error!("Menu action error: {}", e);
                                }
                                app_core.needs_render = true;
                                return Ok((true, None));
                            } else {
                                // Game command - return it for sending to server
                                app_core.needs_render = true;
                                return Ok((true, Some(format!("{}\n", command))));
                            }
                        }
                        app_core.needs_render = true;
                    } else {
                        // Click outside menu - close it
                        app_core.ui_state.popup_menu = None;
                        app_core.ui_state.submenu = None;
                        app_core.ui_state.nested_submenu = None;
                        app_core.ui_state.input_mode = InputMode::Normal;
                        app_core.needs_render = true;
                    }

                    // Don't process other clicks while in menu mode
                    return Ok((true, None));
                }

                // Mouse down handling (find links, start drags)
                app_core.ui_state.selection_state = None;

                let mut found_window = None;
                let mut drag_op = None;
                let mut clicked_window_name: Option<String> = None;
                let mut handled_tab_click = false;

                for (name, window) in &app_core.ui_state.windows {
                    let pos = &window.position;
                    if *x >= pos.x
                        && *x < pos.x + pos.width
                        && *y >= pos.y
                        && *y < pos.y + pos.height
                    {
                        clicked_window_name = Some(name.clone());

                        // Handle tabbed text tab switching on click
                        if window.widget_type == WidgetType::TabbedText {
                            let rect = Rect {
                                x: pos.x,
                                y: pos.y,
                                width: pos.width,
                                height: pos.height,
                            };
                            if self.handle_tabbed_click(name, rect, *x, *y) {
                                handled_tab_click = true;
                                break;
                            }
                        }

                        let right_col = pos.x + pos.width - 1;
                        let bottom_row = pos.y + pos.height - 1;
                        let has_horizontal_space = pos.width > 1;
                        let has_vertical_space = pos.height > 1;

                        if has_horizontal_space
                            && has_vertical_space
                            && *x == right_col
                            && *y == bottom_row
                        {
                            drag_op = Some(DragOperation::ResizeBottomRight);
                            found_window = Some(name.clone());
                            break;
                        } else if has_horizontal_space && *x == right_col {
                            drag_op = Some(DragOperation::ResizeRight);
                            found_window = Some(name.clone());
                            break;
                        } else if has_vertical_space && *y == bottom_row {
                            drag_op = Some(DragOperation::ResizeBottom);
                            found_window = Some(name.clone());
                            break;
                        } else if *y == pos.y {
                            drag_op = Some(DragOperation::Move);
                            found_window = Some(name.clone());
                            break;
                        }
                    }
                }

                if handled_tab_click {
                    app_core.needs_render = true;
                    return Ok((true, None));
                }

                if let (Some(window_name), Some(operation)) = (found_window, drag_op) {
                    if let Some(window) = app_core.ui_state.get_window(&window_name) {
                        let pos = &window.position;
                        app_core.ui_state.mouse_drag = Some(MouseDragState {
                            operation,
                            window_name,
                            start_pos: (*x, *y),
                            original_window_pos: (pos.x, pos.y, pos.width, pos.height),
                        });
                    }
                } else if let Some(window_name) = clicked_window_name {
                    if let Some(window) = app_core.ui_state.get_window(&window_name) {
                        let pos = &window.position;
                        let window_rect = ratatui::layout::Rect {
                            x: pos.x,
                            y: pos.y,
                            width: pos.width,
                            height: pos.height,
                        };

                        if let Some(link_data) =
                            self.link_at_position(&window_name, *x, *y, window_rect)
                        {
                            let has_ctrl = modifiers.ctrl;

                            if has_ctrl {
                                app_core.ui_state.link_drag_state =
                                    Some(LinkDragState {
                                        link_data,
                                        start_pos: (*x, *y),
                                        current_pos: (*x, *y),
                                    });
                            } else {
                                app_core.ui_state.pending_link_click =
                                    Some(PendingLinkClick {
                                        link_data,
                                        click_pos: (*x, *y),
                                    });
                            }
                        } else {
                            // Start text selection
                            app_core.ui_state.selection_drag_start = Some((*x, *y));

                            // Convert mouse coords to text coords for selection
                            if let Some((line, col)) = self.mouse_to_text_coords(
                                &window_name,
                                *x,
                                *y,
                                window_rect,
                            ) {
                                // Find window index from the stable mapping
                                let window_index = window_index_map
                                    .get(&window_name)
                                    .copied()
                                    .unwrap_or(0);
                                app_core.ui_state.selection_state =
                                    Some(crate::selection::SelectionState::new(
                                        window_index,
                                        line,
                                        col,
                                    ));
                            }
                        }
                    }
                }
                return Ok((true, None));
            }
            MouseEventKind::Drag(crate::frontend::MouseButton::Left) => {
                if let Some(ref mut link_drag) = app_core.ui_state.link_drag_state {
                    link_drag.current_pos = (*x, *y);
                    app_core.needs_render = true;
                } else if let Some(drag_state) = app_core.ui_state.mouse_drag.clone() {
                    let dx = *x as i32 - drag_state.start_pos.0 as i32;
                    let dy = *y as i32 - drag_state.start_pos.1 as i32;

                    // Get terminal size for clamping windows within bounds
                    let (term_width, term_height) = self.size();

                    let (min_width_constraint, min_height_constraint) =
                        app_core.window_min_size(&drag_state.window_name);

                    if let Some(window) =
                        app_core.ui_state.get_window_mut(&drag_state.window_name)
                    {
                        let min_width_i32 = min_width_constraint as i32;
                        let min_height_i32 = min_height_constraint as i32;

                        match drag_state.operation {
                            DragOperation::Move => {
                                // Calculate new position
                                let new_x = (drag_state.original_window_pos.0 as i32
                                    + dx)
                                    .max(0)
                                    as u16;
                                let new_y = (drag_state.original_window_pos.1 as i32
                                    + dy)
                                    .max(0)
                                    as u16;

                                // Clamp to prevent overflow beyond terminal boundaries
                                let max_x =
                                    term_width.saturating_sub(window.position.width);
                                let max_y =
                                    term_height.saturating_sub(window.position.height);

                                window.position.x = new_x.min(max_x);
                                window.position.y = new_y.min(max_y);
                            }
                            DragOperation::ResizeRight => {
                                // Calculate new width
                                let new_width =
                                    (drag_state.original_window_pos.2 as i32 + dx)
                                        .max(min_width_i32)
                                        as u16;

                                // Clamp to prevent overflow beyond terminal edge
                                let max_width =
                                    term_width.saturating_sub(window.position.x);
                                window.position.width = new_width.min(max_width);
                            }
                            DragOperation::ResizeBottom => {
                                // Calculate new height
                                let new_height =
                                    (drag_state.original_window_pos.3 as i32 + dy)
                                        .max(min_height_i32)
                                        as u16;

                                // Clamp to prevent overflow beyond terminal edge
                                let max_height =
                                    term_height.saturating_sub(window.position.y);
                                window.position.height = new_height.min(max_height);
                            }
                            DragOperation::ResizeBottomRight => {
                                // Calculate new dimensions
                                let new_width =
                                    (drag_state.original_window_pos.2 as i32 + dx)
                                        .max(min_width_i32)
                                        as u16;
                                let new_height =
                                    (drag_state.original_window_pos.3 as i32 + dy)
                                        .max(min_height_i32)
                                        as u16;

                                // Clamp to prevent overflow beyond terminal edges
                                let max_width =
                                    term_width.saturating_sub(window.position.x);
                                let max_height =
                                    term_height.saturating_sub(window.position.y);

                                window.position.width = new_width.min(max_width);
                                window.position.height = new_height.min(max_height);
                            }
                        }
                        app_core.needs_render = true;
                    }
                } else if app_core.ui_state.pending_link_click.is_some() {
                    app_core.ui_state.pending_link_click = None;
                } else if let Some(_drag_start) = app_core.ui_state.selection_drag_start
                {
                    // Update text selection on drag
                    if let Some(ref mut selection) = app_core.ui_state.selection_state {
                        // Find which window we're dragging in
                        for (name, window) in &app_core.ui_state.windows {
                            let pos = &window.position;
                            if *x >= pos.x
                                && *x < pos.x + pos.width
                                && *y >= pos.y
                                && *y < pos.y + pos.height
                            {
                                let window_rect = ratatui::layout::Rect {
                                    x: pos.x,
                                    y: pos.y,
                                    width: pos.width,
                                    height: pos.height,
                                };
                                if let Some((line, col)) = self
                                    .mouse_to_text_coords(name, *x, *y, window_rect)
                                {
                                    let window_index = window_index_map
                                        .get(name)
                                        .copied()
                                        .unwrap_or(0);
                                    selection.update_end(window_index, line, col);
                                    app_core.needs_render = true;
                                }
                                break;
                            }
                        }
                    }
                }
                return Ok((true, None));
            }
            MouseEventKind::Up(crate::frontend::MouseButton::Left) => {
                let mut command_to_send: Option<String> = None;

                if let Some(link_drag) = app_core.ui_state.link_drag_state.take() {
                    let dx = (*x as i16 - link_drag.start_pos.0 as i16).abs();
                    let dy = (*y as i16 - link_drag.start_pos.1 as i16).abs();

                    if dx > 2 || dy > 2 {
                        let mut drop_target_hand: Option<String> = None;
                        let mut drop_target_id: Option<String> = None;

                        for (name, window) in &app_core.ui_state.windows {
                            let pos = &window.position;
                            if *x >= pos.x
                                && *x < pos.x + pos.width
                                && *y >= pos.y
                                && *y < pos.y + pos.height
                            {
                                // First check if this is a hand widget (left or right only)
                                if name == "left_hand" {
                                    drop_target_hand = Some("left".to_string());
                                    break;
                                } else if name == "right_hand" {
                                    drop_target_hand = Some("right".to_string());
                                    break;
                                }

                                // Otherwise check if we dropped on a link
                                let window_rect = ratatui::layout::Rect {
                                    x: pos.x,
                                    y: pos.y,
                                    width: pos.width,
                                    height: pos.height,
                                };
                                if let Some(target_link) =
                                    self.link_at_position(name, *x, *y, window_rect)
                                {
                                    drop_target_id = Some(target_link.exist_id);
                                    break;
                                }
                            }
                        }

                        let command = if let Some(hand_type) = drop_target_hand {
                            format!(
                                "_drag #{} {}\n",
                                link_drag.link_data.exist_id, hand_type
                            )
                        } else if let Some(target_id) = drop_target_id {
                            format!(
                                "_drag #{} #{}\n",
                                link_drag.link_data.exist_id, target_id
                            )
                        } else {
                            format!("_drag #{} drop\n", link_drag.link_data.exist_id)
                        };
                        command_to_send = Some(command);
                    }
                } else if let Some(pending_click) =
                    app_core.ui_state.pending_link_click.take()
                {
                    let dx = (*x as i16 - pending_click.click_pos.0 as i16).abs();
                    let dy = (*y as i16 - pending_click.click_pos.1 as i16).abs();

                    if dx <= 2 && dy <= 2 {
                        // Handle <d> tags differently (direct commands vs context menus)
                        if pending_click.link_data.exist_id == "_direct_" {
                            // <d> tag: Send text/noun as direct command
                            let command = if !pending_click.link_data.noun.is_empty() {
                                format!("{}\n", pending_click.link_data.noun)
                            // Use cmd attribute
                            } else {
                                format!("{}\n", pending_click.link_data.text)
                                // Use text content
                            };
                            tracing::info!(
                                "Executing <d> direct command: {}",
                                command.trim()
                            );
                            command_to_send = Some(command);
                        } else {
                            // Regular <a> tag: Request context menu
                            let command = app_core.request_menu(
                                pending_click.link_data.exist_id.clone(),
                                pending_click.link_data.noun.clone(),
                                pending_click.click_pos,
                            );
                            tracing::info!(
                                "Sending _menu command for '{}' (exist_id: {})",
                                pending_click.link_data.noun,
                                pending_click.link_data.exist_id
                            );
                            command_to_send = Some(command);
                        }
                    } else {
                        tracing::debug!(
                            "Link click cancelled - dragged {} pixels",
                            dx.max(dy)
                        );
                    }
                }

                // Sync UI state positions back to layout WindowDefs after mouse resize/move
                if let Some(drag_state) = &app_core.ui_state.mouse_drag {
                    if let Some(window) =
                        app_core.ui_state.get_window(&drag_state.window_name)
                    {
                        // Find the corresponding WindowDef in layout and update it
                        if let Some(window_def) = app_core
                            .layout
                            .windows
                            .iter_mut()
                            .find(|w| w.name() == drag_state.window_name)
                        {
                            let base = window_def.base_mut();
                            base.col = window.position.x;
                            base.row = window.position.y;
                            base.cols = window.position.width;
                            base.rows = window.position.height;
                            tracing::info!("Synced mouse resize/move for '{}' to layout: pos=({},{}) size={}x{}",
                                drag_state.window_name, base.col, base.row, base.cols, base.rows);
                            app_core.layout_modified_since_save = true;
                        }
                    }
                }

                app_core.ui_state.mouse_drag = None;
                app_core.ui_state.selection_drag_start = None;

                // Handle text selection copy to clipboard
                if let Some(ref selection) = app_core.ui_state.selection_state {
                    if !selection.is_empty() {
                        // Extract text from selection
                        let (start, end) = selection.normalized_range();

                        // Find the window (for now assume main window)
                        if let Some((_line, _col)) = self.mouse_to_text_coords(
                            "main",
                            *x,
                            *y,
                            ratatui::layout::Rect {
                                x: app_core
                                    .ui_state
                                    .windows
                                    .get("main")
                                    .map(|w| w.position.x)
                                    .unwrap_or(0),
                                y: app_core
                                    .ui_state
                                    .windows
                                    .get("main")
                                    .map(|w| w.position.y)
                                    .unwrap_or(0),
                                width: app_core
                                    .ui_state
                                    .windows
                                    .get("main")
                                    .map(|w| w.position.width)
                                    .unwrap_or(80),
                                height: app_core
                                    .ui_state
                                    .windows
                                    .get("main")
                                    .map(|w| w.position.height)
                                    .unwrap_or(24),
                            },
                        ) {
                            if let Some(text) = self.extract_selection_text(
                                "main", start.line, start.col, end.line, end.col,
                            ) {
                                // Copy to clipboard
                                match arboard::Clipboard::new() {
                                    Ok(mut clipboard) => {
                                        if let Err(e) = clipboard.set_text(&text) {
                                            tracing::warn!(
                                                "Failed to copy to clipboard: {}",
                                                e
                                            );
                                        } else {
                                            tracing::info!(
                                                "Copied {} chars to clipboard",
                                                text.len()
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Failed to access clipboard: {}",
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    }
                    // Clear selection
                    app_core.ui_state.selection_state = None;
                    app_core.needs_render = true;
                }

                return Ok((true, command_to_send));
            }
            _ => {}
        }

        Ok((false, None))
    }

    /// Handle keyboard events (extracted from main.rs Phase 4.2)
    /// Returns optional command to send to server
    pub fn handle_key_event(
        &mut self,
        code: crate::frontend::KeyCode,
        modifiers: crate::frontend::KeyModifiers,
        app_core: &mut crate::core::AppCore,
        handle_menu_action_fn: impl Fn(&mut crate::core::AppCore, &mut Self, &str) -> Result<()>,
    ) -> Result<Option<String>> {
        use crate::data::ui_state::InputMode;
        use crate::frontend::{KeyCode, KeyModifiers};
        use crate::core::input_router;

        tracing::debug!(
            "Key event: code={:?}, modifiers={:?}, input_mode={:?}",
            code,
            modifiers,
            app_core.ui_state.input_mode
        );

        // LAYER 1 & 2: Priority windows (browsers, forms, editors) - handle ALL keys
        // These modes get first priority and consume most input
        match app_core.ui_state.input_mode {
            InputMode::HighlightBrowser => {
                if let Some(ref mut browser) = self.highlight_browser {
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextItem
                        | crate::core::menu_actions::MenuAction::NavigateDown => browser.navigate_down(),
                        crate::core::menu_actions::MenuAction::PreviousItem
                        | crate::core::menu_actions::MenuAction::NavigateUp => browser.navigate_up(),
                        crate::core::menu_actions::MenuAction::NextPage => {
                            browser.next_page()
                        }
                        crate::core::menu_actions::MenuAction::PreviousPage => {
                            browser.previous_page()
                        }
                        crate::core::menu_actions::MenuAction::Select
                        | crate::core::menu_actions::MenuAction::Edit => {
                            if let Some(name) = browser.get_selected() {
                                if let Some(pattern) = app_core.config.highlights.get(&name) {
                                    self.highlight_form = Some(
                                        crate::frontend::tui::highlight_form::HighlightFormWidget::new_edit(
                                            name, pattern,
                                        ),
                                    );
                                    app_core.ui_state.input_mode = InputMode::HighlightForm;
                                }
                            }
                        }
                        crate::core::menu_actions::MenuAction::New
                        | crate::core::menu_actions::MenuAction::Add => {
                            self.highlight_form = Some(
                                crate::frontend::tui::highlight_form::HighlightFormWidget::new(),
                            );
                            app_core.ui_state.input_mode = InputMode::HighlightForm;
                        }
                        crate::core::menu_actions::MenuAction::Delete => {
                            if let Some(name) = browser.get_selected() {
                                app_core.config.highlights.remove(&name);
                                browser.update_items(&app_core.config.highlights);
                                tracing::info!("Deleted highlight: {}", name);
                            }
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.highlight_browser = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::KeybindBrowser => {
                if let Some(ref mut browser) = self.keybind_browser {
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextItem
                        | crate::core::menu_actions::MenuAction::NavigateDown => browser.navigate_down(),
                        crate::core::menu_actions::MenuAction::PreviousItem
                        | crate::core::menu_actions::MenuAction::NavigateUp => browser.navigate_up(),
                        crate::core::menu_actions::MenuAction::NextPage => {
                            browser.next_page()
                        }
                        crate::core::menu_actions::MenuAction::PreviousPage => {
                            browser.previous_page()
                        }
                        crate::core::menu_actions::MenuAction::ToggleFilter => {
                            browser.toggle_filter()
                        }
                        crate::core::menu_actions::MenuAction::Select
                        | crate::core::menu_actions::MenuAction::Edit => {
                            if let Some(entry) = browser.get_selected_entry() {
                                use crate::frontend::tui::keybind_form::KeybindActionType;
                                let action_type = if entry.action_type == "Action" {
                                    KeybindActionType::Action
                                } else {
                                    KeybindActionType::Macro
                                };
                                self.keybind_form = Some(
                                    crate::frontend::tui::keybind_form::KeybindFormWidget::new_edit(
                                        entry.key_combo.clone(),
                                        action_type,
                                        entry.action_value.clone(),
                                    ),
                                );
                                app_core.ui_state.input_mode = InputMode::KeybindForm;
                            }
                        }
                        crate::core::menu_actions::MenuAction::New
                        | crate::core::menu_actions::MenuAction::Add => {
                            self.keybind_form =
                                Some(crate::frontend::tui::keybind_form::KeybindFormWidget::new());
                            app_core.ui_state.input_mode = InputMode::KeybindForm;
                        }
                        crate::core::menu_actions::MenuAction::Delete => {
                            if let Some(key_combo) = browser.get_selected() {
                                app_core.config.keybinds.remove(&key_combo);
                                app_core.rebuild_keybind_map();
                                browser.update_items(&app_core.config.keybinds);
                                tracing::info!("Deleted keybind: {}", key_combo);
                            }
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.keybind_browser = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::ColorPaletteBrowser => {
                if let Some(ref mut browser) = self.color_palette_browser {
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextItem
                        | crate::core::menu_actions::MenuAction::NavigateDown => browser.navigate_down(),
                        crate::core::menu_actions::MenuAction::PreviousItem
                        | crate::core::menu_actions::MenuAction::NavigateUp => browser.navigate_up(),
                        crate::core::menu_actions::MenuAction::NextPage => {
                            browser.next_page()
                        }
                        crate::core::menu_actions::MenuAction::PreviousPage => {
                            browser.previous_page()
                        }
                        crate::core::menu_actions::MenuAction::Select
                        | crate::core::menu_actions::MenuAction::Edit => {
                            if let Some(color) = browser.get_selected_color() {
                                self.color_form = Some(
                                    crate::frontend::tui::color_form::ColorForm::new_edit(
                                        color,
                                    ),
                                );
                                app_core.ui_state.input_mode = InputMode::ColorForm;
                            }
                        }
                        crate::core::menu_actions::MenuAction::New
                        | crate::core::menu_actions::MenuAction::Add => {
                            self.color_form =
                                Some(crate::frontend::tui::color_form::ColorForm::new_create());
                            app_core.ui_state.input_mode = InputMode::ColorForm;
                        }
                        crate::core::menu_actions::MenuAction::Delete => {
                            if let Some(color_name) = browser.get_selected() {
                                app_core
                                    .config
                                    .colors
                                    .color_palette
                                    .retain(|c| c.name != color_name);
                                browser
                                    .update_items(app_core.config.colors.color_palette.clone());
                                tracing::info!("Deleted color: {}", color_name);
                            }
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.color_palette_browser = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::UIColorsBrowser => {
                if let Some(ref mut browser) = self.uicolors_browser {
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextItem
                        | crate::core::menu_actions::MenuAction::NavigateDown => browser.navigate_down(),
                        crate::core::menu_actions::MenuAction::PreviousItem
                        | crate::core::menu_actions::MenuAction::NavigateUp => browser.navigate_up(),
                        crate::core::menu_actions::MenuAction::NextPage => {
                            browser.next_page()
                        }
                        crate::core::menu_actions::MenuAction::PreviousPage => {
                            browser.previous_page()
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.uicolors_browser = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::SpellColorsBrowser => {
                if let Some(ref mut browser) = self.spell_color_browser {
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextItem
                        | crate::core::menu_actions::MenuAction::NavigateDown => browser.navigate_down(),
                        crate::core::menu_actions::MenuAction::PreviousItem
                        | crate::core::menu_actions::MenuAction::NavigateUp => browser.navigate_up(),
                        crate::core::menu_actions::MenuAction::NextPage => {
                            browser.next_page()
                        }
                        crate::core::menu_actions::MenuAction::PreviousPage => {
                            browser.previous_page()
                        }
                        crate::core::menu_actions::MenuAction::Select
                        | crate::core::menu_actions::MenuAction::Edit => {
                            if let Some(index) = browser.get_selected() {
                                let spell_color =
                                    app_core.config.colors.spell_colors.get(index).cloned();
                                if let Some(sc) = spell_color {
                                    self.spell_color_form = Some(
                                        crate::frontend::tui::spell_color_form::SpellColorFormWidget::new_edit(
                                            index, &sc,
                                        ),
                                    );
                                    app_core.ui_state.input_mode = InputMode::SpellColorForm;
                                }
                            }
                        }
                        crate::core::menu_actions::MenuAction::New
                        | crate::core::menu_actions::MenuAction::Add => {
                            self.spell_color_form = Some(
                                crate::frontend::tui::spell_color_form::SpellColorFormWidget::new(
                                ),
                            );
                            app_core.ui_state.input_mode = InputMode::SpellColorForm;
                        }
                        crate::core::menu_actions::MenuAction::Delete => {
                            if let Some(index) = browser.get_selected() {
                                if index < app_core.config.colors.spell_colors.len() {
                                    app_core.config.colors.spell_colors.remove(index);
                                    browser
                                        .update_items(&app_core.config.colors.spell_colors);
                                    tracing::info!("Deleted spell color range at index {}", index);
                                }
                            }
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.spell_color_browser = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::ThemeBrowser => {
                if let Some(ref mut browser) = self.theme_browser {
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextItem
                        | crate::core::menu_actions::MenuAction::NavigateDown => browser.navigate_down(),
                        crate::core::menu_actions::MenuAction::PreviousItem
                        | crate::core::menu_actions::MenuAction::NavigateUp => browser.navigate_up(),
                        crate::core::menu_actions::MenuAction::NextPage => {
                            browser.next_page()
                        }
                        crate::core::menu_actions::MenuAction::PreviousPage => {
                            browser.previous_page()
                        }
                        crate::core::menu_actions::MenuAction::Select => {
                            if let Some(theme_name) = browser.get_selected() {
                                app_core.config.active_theme = theme_name.clone();
                                let theme = app_core.config.get_theme();
                                self.update_theme_cache(theme_name, theme);
                                self.theme_browser = None;
                                app_core.ui_state.input_mode = InputMode::Normal;
                                tracing::info!("Switched to theme: {}", app_core.config.active_theme);
                            }
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.theme_browser = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::SettingsEditor => {
                if let Some(ref mut editor) = self.settings_editor {
                    use crate::frontend::tui::widget_traits::Navigable;
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextItem
                        | crate::core::menu_actions::MenuAction::NavigateDown => editor.navigate_down(),
                        crate::core::menu_actions::MenuAction::PreviousItem
                        | crate::core::menu_actions::MenuAction::NavigateUp => editor.navigate_up(),
                        crate::core::menu_actions::MenuAction::NextPage => {
                            editor.next_page()
                        }
                        crate::core::menu_actions::MenuAction::PreviousPage => {
                            editor.previous_page()
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.settings_editor = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        _ => {}
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::HighlightForm => {
                if let Some(ref mut form) = self.highlight_form {
                    use crate::frontend::tui::widget_traits::{
                        FieldNavigable, TextEditable, Toggleable,
                    };
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextField => form.next_field(),
                        crate::core::menu_actions::MenuAction::PreviousField => {
                            form.previous_field()
                        }
                        crate::core::menu_actions::MenuAction::SelectAll => form.select_all(),
                        crate::core::menu_actions::MenuAction::Copy => {
                            let _ = form.copy_to_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Cut => {
                            let _ = form.cut_to_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Paste => {
                            let _ = form.paste_from_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Toggle => {
                            form.toggle_focused();
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.highlight_form = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        crate::core::menu_actions::MenuAction::NavigateUp |
                        crate::core::menu_actions::MenuAction::NavigateDown |
                        crate::core::menu_actions::MenuAction::CycleBackward |
                        crate::core::menu_actions::MenuAction::CycleForward |
                        crate::core::menu_actions::MenuAction::Select |
                        crate::core::menu_actions::MenuAction::Save |
                        crate::core::menu_actions::MenuAction::Delete => {
                            // Handle navigation, cycling, and save/delete via handle_action
                            if let Some(result) = form.handle_action(action.clone()) {
                                match result {
                                    crate::frontend::tui::highlight_form::FormResult::Save {
                                        name,
                                        mut pattern,
                                    } => {
                                        // Resolve palette color names to hex codes
                                        if let Some(ref fg) = pattern.fg {
                                            pattern.fg = Some(app_core.config.resolve_palette_color(fg));
                                        }
                                        if let Some(ref bg) = pattern.bg {
                                            pattern.bg = Some(app_core.config.resolve_palette_color(bg));
                                        }

                                        app_core.config.highlights.insert(name.clone(), pattern);
                                        tracing::info!("Saved highlight: {}", name);
                                        self.highlight_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::highlight_form::FormResult::Delete {
                                        name,
                                    } => {
                                        app_core.config.highlights.remove(&name);
                                        self.highlight_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Deleted highlight: {}", name);
                                    }
                                    crate::frontend::tui::highlight_form::FormResult::Cancel => {
                                        self.highlight_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                }
                            }
                        }
                        _ => {
                            use crate::frontend::tui::crossterm_bridge;
                            let ct_code = crossterm_bridge::to_crossterm_keycode(code);
                            let ct_mods = crossterm_bridge::to_crossterm_modifiers(modifiers);
                            let key = crossterm::event::KeyEvent::new(ct_code, ct_mods);
                            if let Some(result) = form.handle_key(key) {
                                match result {
                                    crate::frontend::tui::highlight_form::FormResult::Save {
                                        name,
                                        pattern,
                                    } => {
                                        // Save to current config (save_as_common feature removed)
                                        app_core.config.highlights.insert(name.clone(), pattern);
                                        tracing::info!("Saved highlight: {}", name);
                                        self.highlight_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::highlight_form::FormResult::Delete {
                                        name,
                                    } => {
                                        app_core.config.highlights.remove(&name);
                                        self.highlight_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Deleted highlight: {}", name);
                                    }
                                    crate::frontend::tui::highlight_form::FormResult::Cancel => {
                                        self.highlight_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                }
                            }
                        }
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::KeybindForm => {
                if let Some(ref mut form) = self.keybind_form {
                    use crate::frontend::tui::widget_traits::{
                        FieldNavigable, TextEditable, Toggleable,
                    };
                    use crate::frontend::tui::keybind_form::ActionSection;

                    let no_mods = matches!(modifiers, KeyModifiers { ctrl: false, shift: false, alt: false });
                    let ctrl_only = matches!(modifiers, KeyModifiers { ctrl: true, shift: false, alt: false });

                    if ctrl_only {
                        match code {
                            KeyCode::Char('1') => {
                                form.go_to_section(ActionSection::CommandInput);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('2') => {
                                form.go_to_section(ActionSection::CommandHistory);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('3') => {
                                form.go_to_section(ActionSection::WindowScrolling);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('4') => {
                                form.go_to_section(ActionSection::TabNavigation);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('5') => {
                                form.go_to_section(ActionSection::Search);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('6') => {
                                form.go_to_section(ActionSection::Clipboard);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('7') => {
                                form.go_to_section(ActionSection::TTS);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('8') => {
                                form.go_to_section(ActionSection::SystemToggles);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('m') | KeyCode::Char('M') => {
                                form.go_to_section(ActionSection::Meta);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            _ => {}
                        }
                    }

                    // Section navigation removed - not applicable to simple form widget
                    // (This code was likely meant for KeybindBrowser, not KeybindForm)

                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextField => form.next_field(),
                        crate::core::menu_actions::MenuAction::PreviousField => {
                            form.previous_field()
                        }
                        crate::core::menu_actions::MenuAction::SelectAll => form.select_all(),
                        crate::core::menu_actions::MenuAction::Copy => {
                            let _ = form.copy_to_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Cut => {
                            let _ = form.cut_to_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Paste => {
                            let _ = form.paste_from_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Toggle => {
                            form.toggle_focused();
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.keybind_form = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        // Route navigation, cycling, select, save, and delete through handle_action
                        crate::core::menu_actions::MenuAction::NavigateUp
                        | crate::core::menu_actions::MenuAction::NavigateDown
                        | crate::core::menu_actions::MenuAction::CycleBackward
                        | crate::core::menu_actions::MenuAction::CycleForward
                        | crate::core::menu_actions::MenuAction::Select
                        | crate::core::menu_actions::MenuAction::Save
                        | crate::core::menu_actions::MenuAction::Delete => {
                            if let Some(result) = form.handle_action(action.clone()) {
                                match result {
                                    crate::frontend::tui::keybind_form::KeybindFormResult::Save {
                                        key_combo,
                                        action_type,
                                        value,
                                    } => {
                                        use crate::frontend::tui::keybind_form::KeybindActionType;
                                        let action = match action_type {
                                            KeybindActionType::Action => {
                                                crate::config::KeyBindAction::Action(value)
                                            }
                                            KeybindActionType::Macro => {
                                                crate::config::KeyBindAction::Macro(
                                                    crate::config::MacroAction { macro_text: value },
                                                )
                                            }
                                        };
                                        app_core.config.keybinds.insert(key_combo.clone(), action);
                                        app_core.rebuild_keybind_map();
                                        self.keybind_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Saved keybind: {}", key_combo);
                                    }
                                    crate::frontend::tui::keybind_form::KeybindFormResult::Delete {
                                        key_combo,
                                    } => {
                                        app_core.config.keybinds.remove(&key_combo);
                                        app_core.rebuild_keybind_map();
                                        self.keybind_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Deleted keybind: {}", key_combo);
                                    }
                                    crate::frontend::tui::keybind_form::KeybindFormResult::Cancel => {
                                        self.keybind_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                }
                            }
                        }
                        _ => {
                            use crate::frontend::tui::crossterm_bridge;
                            let ct_code = crossterm_bridge::to_crossterm_keycode(code);
                            let ct_mods = crossterm_bridge::to_crossterm_modifiers(modifiers);
                            let key = crossterm::event::KeyEvent::new(ct_code, ct_mods);
                            if let Some(result) = form.handle_key(key) {
                                match result {
                                    crate::frontend::tui::keybind_form::KeybindFormResult::Save {
                                        key_combo,
                                        action_type,
                                        value,
                                    } => {
                                        use crate::frontend::tui::keybind_form::KeybindActionType;
                                        let action = match action_type {
                                            KeybindActionType::Action => {
                                                crate::config::KeyBindAction::Action(value)
                                            }
                                            KeybindActionType::Macro => {
                                                crate::config::KeyBindAction::Macro(
                                                    crate::config::MacroAction { macro_text: value },
                                                )
                                            }
                                        };
                                        app_core.config.keybinds.insert(key_combo.clone(), action);
                                        app_core.rebuild_keybind_map();
                                        self.keybind_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Saved keybind: {}", key_combo);
                                    }
                                    crate::frontend::tui::keybind_form::KeybindFormResult::Delete {
                                        key_combo,
                                    } => {
                                        app_core.config.keybinds.remove(&key_combo);
                                        app_core.rebuild_keybind_map();
                                        self.keybind_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Deleted keybind: {}", key_combo);
                                    }
                                    crate::frontend::tui::keybind_form::KeybindFormResult::Cancel => {
                                        self.keybind_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                }
                            }
                        }
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::ColorForm => {
                if let Some(ref mut form) = self.color_form {
                    use crate::frontend::tui::widget_traits::{
                        FieldNavigable, TextEditable, Toggleable,
                    };
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextField => form.next_field(),
                        crate::core::menu_actions::MenuAction::PreviousField => {
                            form.previous_field()
                        }
                        crate::core::menu_actions::MenuAction::SelectAll => form.select_all(),
                        crate::core::menu_actions::MenuAction::Copy => {
                            let _ = form.copy_to_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Cut => {
                            let _ = form.cut_to_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Paste => {
                            let _ = form.paste_from_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Toggle => {
                            form.toggle_focused();
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.color_form = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        crate::core::menu_actions::MenuAction::Select |
                        crate::core::menu_actions::MenuAction::Save => {
                            // Handle Select (Enter) and Save (Ctrl+S) via handle_action
                            if let Some(result) = form.handle_action(action.clone()) {
                                match result {
                                    crate::frontend::tui::color_form::FormAction::Save {
                                        color,
                                        original_name,
                                    } => {
                                        if let Some(old_name) = original_name {
                                            if old_name != color.name {
                                                app_core
                                                    .config
                                                    .colors
                                                    .color_palette
                                                    .retain(|c| c.name != old_name);
                                            }
                                        }
                                        if let Some(existing) = app_core
                                            .config
                                            .colors
                                            .color_palette
                                            .iter_mut()
                                            .find(|c| c.name == color.name)
                                        {
                                            *existing = color.clone();
                                        } else {
                                            app_core.config.colors.color_palette.push(color.clone());
                                        }
                                        self.color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Saved color: {}", color.name);
                                    }
                                    crate::frontend::tui::color_form::FormAction::Delete => {
                                        self.color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::color_form::FormAction::Cancel => {
                                        self.color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::color_form::FormAction::Error(_) => {}
                                }
                            }
                        }
                        _ => {
                            use crate::frontend::tui::crossterm_bridge;
                            let ct_code = crossterm_bridge::to_crossterm_keycode(code);
                            let ct_mods = crossterm_bridge::to_crossterm_modifiers(modifiers);
                            let key = crossterm::event::KeyEvent::new(ct_code, ct_mods);
                            if let Some(result) = form.handle_input(key) {
                                match result {
                                    crate::frontend::tui::color_form::FormAction::Save {
                                        color,
                                        original_name,
                                    } => {
                                        if let Some(old_name) = original_name {
                                            if old_name != color.name {
                                                app_core
                                                    .config
                                                    .colors
                                                    .color_palette
                                                    .retain(|c| c.name != old_name);
                                            }
                                        }
                                        if let Some(existing) = app_core
                                            .config
                                            .colors
                                            .color_palette
                                            .iter_mut()
                                            .find(|c| c.name == color.name)
                                        {
                                            *existing = color.clone();
                                        } else {
                                            app_core.config.colors.color_palette.push(color.clone());
                                        }
                                        self.color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Saved color: {}", color.name);
                                    }
                                    crate::frontend::tui::color_form::FormAction::Delete => {
                                        self.color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::color_form::FormAction::Cancel => {
                                        self.color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::color_form::FormAction::Error(_) => {}
                                }
                            }
                        }
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::SpellColorForm => {
                if let Some(ref mut form) = self.spell_color_form {
                    use crate::frontend::tui::widget_traits::{FieldNavigable, TextEditable};
                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextField => form.next_field(),
                        crate::core::menu_actions::MenuAction::PreviousField => {
                            form.previous_field()
                        }
                        crate::core::menu_actions::MenuAction::SelectAll => form.select_all(),
                        crate::core::menu_actions::MenuAction::Copy => {
                            let _ = form.copy_to_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Cut => {
                            let _ = form.cut_to_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Paste => {
                            let _ = form.paste_from_clipboard();
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.spell_color_form = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                        // Route navigation, select, save, and delete through handle_action
                        crate::core::menu_actions::MenuAction::NavigateUp
                        | crate::core::menu_actions::MenuAction::NavigateDown
                        | crate::core::menu_actions::MenuAction::Select
                        | crate::core::menu_actions::MenuAction::Save
                        | crate::core::menu_actions::MenuAction::Delete => {
                            if let Some(result) = form.handle_action(action.clone()) {
                                match result {
                                    crate::frontend::tui::spell_color_form::SpellColorFormResult::Save(
                                        mut spell_color,
                                    ) => {
                                        // Resolve palette color names to hex codes
                                        spell_color.color = app_core.config.resolve_palette_color(&spell_color.color);
                                        if let Some(ref bar) = spell_color.bar_color {
                                            spell_color.bar_color = Some(app_core.config.resolve_palette_color(bar));
                                        }
                                        if let Some(ref text) = spell_color.text_color {
                                            spell_color.text_color = Some(app_core.config.resolve_palette_color(text));
                                        }
                                        if let Some(ref bg) = spell_color.bg_color {
                                            spell_color.bg_color = Some(app_core.config.resolve_palette_color(bg));
                                        }

                                        app_core.config.colors.spell_colors.push(spell_color);
                                        self.spell_color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Saved spell color range");
                                    }
                                    crate::frontend::tui::spell_color_form::SpellColorFormResult::Delete(
                                        index,
                                    ) => {
                                        if index < app_core.config.colors.spell_colors.len() {
                                            app_core.config.colors.spell_colors.remove(index);
                                            tracing::info!("Deleted spell color range");
                                        }
                                        self.spell_color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::spell_color_form::SpellColorFormResult::Cancel => {
                                        self.spell_color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                }
                            }
                        }
                        _ => {
                            use crate::frontend::tui::crossterm_bridge;
                            let ct_code = crossterm_bridge::to_crossterm_keycode(code);
                            let ct_mods = crossterm_bridge::to_crossterm_modifiers(modifiers);
                            let key = crossterm::event::KeyEvent::new(ct_code, ct_mods);
                            if let Some(result) = form.input(key) {
                                match result {
                                    crate::frontend::tui::spell_color_form::SpellColorFormResult::Save(
                                        mut spell_color,
                                    ) => {
                                        // Resolve palette color names to hex codes
                                        spell_color.color = app_core.config.resolve_palette_color(&spell_color.color);
                                        if let Some(ref bar) = spell_color.bar_color {
                                            spell_color.bar_color = Some(app_core.config.resolve_palette_color(bar));
                                        }
                                        if let Some(ref text) = spell_color.text_color {
                                            spell_color.text_color = Some(app_core.config.resolve_palette_color(text));
                                        }
                                        if let Some(ref bg) = spell_color.bg_color {
                                            spell_color.bg_color = Some(app_core.config.resolve_palette_color(bg));
                                        }

                                        app_core.config.colors.spell_colors.push(spell_color);
                                        self.spell_color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                        tracing::info!("Saved spell color range");
                                    }
                                    crate::frontend::tui::spell_color_form::SpellColorFormResult::Delete(
                                        index,
                                    ) => {
                                        if index < app_core.config.colors.spell_colors.len() {
                                            app_core.config.colors.spell_colors.remove(index);
                                            tracing::info!("Deleted spell color range");
                                        }
                                        self.spell_color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::spell_color_form::SpellColorFormResult::Cancel => {
                                        self.spell_color_form = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                }
                            }
                        }
                    }
                    app_core.needs_render = true;
                }
                return Ok(None);
            }
            InputMode::ThemeEditor => {
                if let Some(ref mut editor) = self.theme_editor {
                    use crate::core::input_router;

                    // Ctrl+1-6 section jumping (high priority)
                    if modifiers.ctrl {
                        match code {
                            crate::frontend::KeyCode::Char(c @ '1'..='6') => {
                                let section = c.to_digit(10).unwrap() as usize;
                                editor.jump_to_section(section);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            _ => {}
                        }
                    }

                    let key_event = crate::frontend::common::KeyEvent { code, modifiers };
                    let action = input_router::route_input(
                        &key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextField => {
                            editor.next_field();
                            app_core.needs_render = true;
                        }
                        crate::core::menu_actions::MenuAction::PreviousField => {
                            editor.previous_field();
                            app_core.needs_render = true;
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            self.theme_editor = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                            app_core.needs_render = true;
                        }
                        // Route navigation and save through handle_action
                        crate::core::menu_actions::MenuAction::NavigateUp
                        | crate::core::menu_actions::MenuAction::NavigateDown
                        | crate::core::menu_actions::MenuAction::Save => {
                            if let Some(result) = editor.handle_action(action.clone()) {
                                match result {
                                    crate::frontend::tui::theme_editor::ThemeEditorResult::Save(mut theme_data) => {
                                        // Resolve palette color names to hex codes
                                        theme_data.resolve_palette_colors(&app_core.config);

                                        match theme_data.save_to_file(app_core.config.character.as_deref()) {
                                            Ok(path) => {
                                                tracing::info!("Saved custom theme '{}' to {:?}", theme_data.name, path);
                                                app_core.add_system_message(&format!(
                                                    "Saved custom theme: {}",
                                                    theme_data.name
                                                ));

                                                if let Some(_app_theme) = theme_data.to_app_theme() {
                                                    app_core.config.active_theme = theme_data.name.clone();
                                                    let theme = app_core.config.get_theme();
                                                    self.update_theme_cache(theme_data.name.clone(), theme);
                                                    app_core.needs_render = true;
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to save custom theme: {}", e);
                                                app_core.add_system_message(&format!("Error saving theme: {}", e));
                                            }
                                        }
                                        self.theme_editor = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::theme_editor::ThemeEditorResult::Cancel => {
                                        self.theme_editor = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                }
                            }
                        }
                        _ => {
                            use crate::frontend::tui::crossterm_bridge;
                            let ct_code = crossterm_bridge::to_crossterm_keycode(code);
                            let ct_mods = crossterm_bridge::to_crossterm_modifiers(modifiers);
                            let key = crossterm::event::KeyEvent::new(ct_code, ct_mods);
                            let _ = editor.handle_input(key);
                            app_core.needs_render = true;
                        }
                    }
                }
                return Ok(None);
            }
            _ => {}
        }

        // Menu mode keyboard navigation
        if app_core.ui_state.input_mode == InputMode::Menu {
            return self.handle_menu_mode_keys(code, modifiers, app_core, handle_menu_action_fn);
        }

        // WindowEditor mode keyboard handling
        if app_core.ui_state.input_mode == InputMode::WindowEditor {
            return self.handle_window_editor_keys(code, modifiers, app_core);
        }

        // Search mode keyboard handling
        if app_core.ui_state.input_mode == InputMode::Search {
            return self.handle_search_mode_keys(code, app_core);
        }

        // Normal mode: user keybinds + CommandInput fallback
        self.handle_normal_mode_keys(code, modifiers, app_core)
    }

    /// Handle Menu mode keyboard navigation (extracted from main.rs Phase 4.2)
    fn handle_menu_mode_keys(
        &mut self,
        code: crate::frontend::KeyCode,
        _modifiers: crate::frontend::KeyModifiers,
        app_core: &mut crate::core::AppCore,
        handle_menu_action_fn: impl Fn(&mut crate::core::AppCore, &mut Self, &str) -> Result<()>,
    ) -> Result<Option<String>> {
        
        
        use crate::frontend::KeyCode;

        tracing::debug!("Menu mode active - handling key: {:?}", code);

        match code {
            KeyCode::Tab | KeyCode::Down => {
                if let Some(ref mut submenu) = app_core.ui_state.submenu {
                    submenu.select_next();
                } else if let Some(ref mut menu) = app_core.ui_state.popup_menu {
                    menu.select_next();
                }
                app_core.needs_render = true;
            }
            KeyCode::BackTab | KeyCode::Up => {
                if let Some(ref mut submenu) = app_core.ui_state.submenu {
                    submenu.select_prev();
                } else if let Some(ref mut menu) = app_core.ui_state.popup_menu {
                    menu.select_prev();
                }
                app_core.needs_render = true;
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let menu_to_use = if app_core.ui_state.submenu.is_some() {
                    &app_core.ui_state.submenu
                } else {
                    &app_core.ui_state.popup_menu
                };

                if let Some(menu) = menu_to_use {
                    if let Some(item) = menu.selected_item() {
                        let command = item.command.clone();
                        tracing::info!("Menu command selected: {}", command);

                        return self.handle_menu_command(command, app_core, handle_menu_action_fn);
                    }
                }
            }
            _ => {}
        }
        Ok(None)
    }

    /// Handle menu command execution (extracted from main.rs Phase 4.2)
    fn handle_menu_command(
        &mut self,
        command: String,
        app_core: &mut crate::core::AppCore,
        handle_menu_action_fn: impl Fn(&mut crate::core::AppCore, &mut Self, &str) -> Result<()>,
    ) -> Result<Option<String>> {
        
        use crate::data::ui_state::{InputMode, PopupMenu};

        if let Some(submenu_name) = command.strip_prefix("menu:") {
            let items = match submenu_name {
                "windows" => app_core.build_windows_submenu(),
                "config" => Self::build_config_submenu(),
                "layouts" => app_core.build_layouts_submenu(),
                "widgetpicker" | "addwindow" => app_core.build_add_window_menu(),
                "hidewindow" => app_core.build_hide_window_menu(),
                "editwindow" => app_core.build_edit_window_menu(),
                _ => {
                    app_core.ui_state.popup_menu = None;
                    app_core.ui_state.input_mode = InputMode::Normal;
                    return Ok(None);
                }
            };
            app_core.ui_state.popup_menu = Some(PopupMenu::new(items, (40, 12)));
            app_core.needs_render = true;
        } else if let Some(category) = command.strip_prefix("__SUBMENU__") {
            let items = app_core.build_submenu(category);
            let items = if !items.is_empty() {
                items
            } else if let Some(items) = app_core.menu_categories.get(category) {
                items.clone()
            } else {
                Vec::new()
            };

            if !items.is_empty() {
                let position = app_core
                    .ui_state
                    .popup_menu
                    .as_ref()
                    .map(|m| m.get_position())
                    .unwrap_or((40, 12));
                let submenu_pos = (position.0 + 2, position.1);
                app_core.ui_state.submenu = Some(PopupMenu::new(items, submenu_pos));
                tracing::info!("Opened submenu: {}", category);
            } else {
                app_core.ui_state.popup_menu = None;
                app_core.ui_state.input_mode = InputMode::Normal;
            }
            app_core.needs_render = true;
        } else if let Some(category_str) = command.strip_prefix("__SUBMENU_ADD__") {
            let category = Self::parse_widget_category(category_str, app_core)?;
            let items = app_core.build_add_window_category_menu(&category);
            if items.is_empty() {
                app_core.ui_state.popup_menu = None;
                app_core.ui_state.input_mode = InputMode::Normal;
            } else {
                app_core.ui_state.popup_menu = Some(PopupMenu::new(items, (40, 12)));
            }
            app_core.needs_render = true;
        } else if let Some(category_str) = command.strip_prefix("__SUBMENU_HIDE__") {
            let category = Self::parse_widget_category(category_str, app_core)?;
            let items = app_core.build_hide_window_category_menu(&category);
            if items.is_empty() {
                app_core.ui_state.popup_menu = None;
                app_core.ui_state.input_mode = InputMode::Normal;
            } else {
                app_core.ui_state.popup_menu = Some(PopupMenu::new(items, (40, 12)));
            }
            app_core.needs_render = true;
        } else if let Some(category_str) = command.strip_prefix("__SUBMENU_EDIT__") {
            let category = Self::parse_widget_category(category_str, app_core)?;
            let items = app_core.build_edit_window_category_menu(&category);
            if items.is_empty() {
                app_core.ui_state.popup_menu = None;
                app_core.ui_state.input_mode = InputMode::Normal;
            } else {
                app_core.ui_state.popup_menu = Some(PopupMenu::new(items, (40, 12)));
            }
            app_core.needs_render = true;
        } else if let Some(window_name) = command.strip_prefix("__ADD__") {
            match app_core.layout.add_window(window_name) {
                Ok(_) => {
                    let (width, height) = self.size();
                    app_core.sync_layout_to_ui_state(width, height, &app_core.layout.clone());
                    app_core.layout_modified_since_save = true;
                    app_core.add_system_message(&format!("Window '{}' added", window_name));
                    tracing::info!("Added window: {}", window_name);
                }
                Err(e) => {
                    app_core.add_system_message(&format!("Failed to add window: {}", e));
                    tracing::error!("Failed to add window '{}': {}", window_name, e);
                }
            }
            app_core.ui_state.popup_menu = None;
            app_core.ui_state.input_mode = InputMode::Normal;
            app_core.needs_render = true;
        } else if let Some(window_name) = command.strip_prefix("__HIDE__") {
            match app_core.layout.hide_window(window_name) {
                Ok(_) => {
                    app_core.ui_state.remove_window(window_name);
                    app_core.layout_modified_since_save = true;
                    app_core.add_system_message(&format!("Window '{}' hidden", window_name));
                    tracing::info!("Hidden window: {}", window_name);
                    app_core.layout.remove_window_if_default(window_name);
                }
                Err(e) => {
                    app_core.add_system_message(&format!("Failed to hide window: {}", e));
                    tracing::error!("Failed to hide window '{}': {}", window_name, e);
                }
            }
            app_core.ui_state.popup_menu = None;
            app_core.ui_state.input_mode = InputMode::Normal;
            app_core.needs_render = true;
        } else if let Some(window_name) = command.strip_prefix("__EDIT__") {
            if let Some(window_def) = app_core.layout.get_window(window_name) {
                self.window_editor = Some(crate::frontend::tui::window_editor::WindowEditor::new(
                    window_def.clone(),
                ));
                app_core.ui_state.input_mode = InputMode::WindowEditor;
                tracing::info!("Opening window editor for: {}", window_name);
            } else {
                app_core.add_system_message(&format!("Window '{}' not found", window_name));
                tracing::warn!("Window '{}' not found in layout", window_name);
            }
            app_core.ui_state.popup_menu = None;
            app_core.needs_render = true;
        } else {
            app_core.ui_state.popup_menu = None;
            app_core.ui_state.submenu = None;
            app_core.ui_state.nested_submenu = None;
            app_core.ui_state.input_mode = InputMode::Normal;
            app_core.needs_render = true;

            if command.starts_with("action:") {
                handle_menu_action_fn(app_core, self, &command)?;
            } else if command.starts_with(".") {
                let action_command = format!("action:{}", &command[1..]);
                handle_menu_action_fn(app_core, self, &action_command)?;
            } else if !command.is_empty() {
                tracing::info!("Sending context menu command: {}", command);
                return Ok(Some(format!("{}\n", command)));
            }
        }
        Ok(None)
    }

    /// Parse widget category from string (helper for menu commands)
    fn parse_widget_category(
        category_str: &str,
        app_core: &mut crate::core::AppCore,
    ) -> Result<crate::config::WidgetCategory> {
        use crate::config::WidgetCategory;
        use crate::data::ui_state::InputMode;

        match category_str {
            "ProgressBar" => Ok(WidgetCategory::ProgressBar),
            "TextWindow" => Ok(WidgetCategory::TextWindow),
            "Countdown" => Ok(WidgetCategory::Countdown),
            "Hand" => Ok(WidgetCategory::Hand),
            "ActiveEffects" => Ok(WidgetCategory::ActiveEffects),
            "Other" => Ok(WidgetCategory::Other),
            _ => {
                tracing::warn!("Unknown widget category: {}", category_str);
                app_core.ui_state.popup_menu = None;
                app_core.ui_state.input_mode = InputMode::Normal;
                app_core.needs_render = true;
                Ok(WidgetCategory::Other)
            }
        }
    }

    /// Build configuration submenu (delegates to menu_builders module)
    fn build_config_submenu() -> Vec<crate::data::ui_state::PopupMenuItem> {
        menu_builders::build_config_submenu()
    }

    /// Handle WindowEditor mode keyboard events (extracted from main.rs Phase 4.2)
    fn handle_window_editor_keys(
        &mut self,
        code: crate::frontend::KeyCode,
        modifiers: crate::frontend::KeyModifiers,
        app_core: &mut crate::core::AppCore,
    ) -> Result<Option<String>> {
        use crate::core::input_router;
        use crate::data::ui_state::InputMode;
        use crate::frontend::KeyCode;

        if let Some(ref mut editor) = self.window_editor {
            // Ctrl+1-9 section jumping (high priority)
            if modifiers.ctrl {
                match code {
                    KeyCode::Char(c @ '1'..='9') => {
                        let section = c.to_digit(10).unwrap() as usize;
                        editor.jump_to_section(section);
                        app_core.needs_render = true;
                        return Ok(None);
                    }
                    _ => {}
                }
            }

            let key_event = crate::frontend::common::KeyEvent { code, modifiers };
            let action = input_router::route_input(
                &key_event,
                &app_core.ui_state.input_mode,
                &app_core.config,
            );

            match action {
                crate::core::menu_actions::MenuAction::NextField
                | crate::core::menu_actions::MenuAction::NavigateDown => {
                    editor.navigate_down();
                    app_core.needs_render = true;
                }
                crate::core::menu_actions::MenuAction::PreviousField
                | crate::core::menu_actions::MenuAction::NavigateUp => {
                    editor.navigate_up();
                    app_core.needs_render = true;
                }
                crate::core::menu_actions::MenuAction::Toggle => {
                    if editor.is_on_checkbox() {
                        editor.toggle_field();
                        app_core.needs_render = true;
                    } else if editor.is_on_content_align() {
                        editor.cycle_content_align(false);
                        app_core.needs_render = true;
                    } else if editor.is_on_border_style() {
                        editor.cycle_border_style();
                        app_core.needs_render = true;
                    }
                }
                crate::core::menu_actions::MenuAction::Select => {
                    if editor.is_on_checkbox()
                        || editor.is_on_content_align()
                        || editor.is_on_border_style()
                    {
                        if editor.is_on_checkbox() {
                            editor.toggle_field();
                        } else if editor.is_on_content_align() {
                            editor.cycle_content_align(false);
                        } else if editor.is_on_border_style() {
                            editor.cycle_border_style();
                        }
                        app_core.needs_render = true;
                    }
                }
                crate::core::menu_actions::MenuAction::Save => {
                    let (width, height) = self.size();
                    if let Some(ref mut editor) = self.window_editor {
                        let window_def = editor.get_window_def().clone();

                        if editor.is_new() {
                            app_core.layout.windows.insert(0, window_def.clone());
                            tracing::info!("Added new window: {}", window_def.name());
                            app_core.add_new_window(&window_def, width, height);
                        } else {
                            if let Some(existing) = app_core
                                .layout
                                .windows
                                .iter_mut()
                                .find(|w| w.name() == window_def.name())
                            {
                                *existing = window_def.clone();
                                tracing::info!("Updated window: {}", window_def.name());
                                app_core.update_window_position(&window_def, width, height);
                            }
                        }
                        app_core.mark_layout_modified();
                        self.window_editor = None;
                        app_core.ui_state.input_mode = InputMode::Normal;
                        app_core.needs_render = true;
                    }
                }
                crate::core::menu_actions::MenuAction::Delete => {
                    if let Some(ref mut editor) = self.window_editor {
                        let window_name = editor.get_window_def().name().to_string();
                        let is_locked = editor.get_window_def().base().locked;

                        if !is_locked {
                            app_core.hide_window(&window_name);
                            self.window_editor = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                        }
                    }
                    app_core.needs_render = true;
                }
                crate::core::menu_actions::MenuAction::Cancel => {
                    self.window_editor = None;
                    app_core.ui_state.input_mode = InputMode::Normal;
                    app_core.needs_render = true;
                }
                _ => {
                    use crate::frontend::tui::crossterm_bridge;
                    let ct_code = crossterm_bridge::to_crossterm_keycode(code);
                    let ct_mods = crossterm_bridge::to_crossterm_modifiers(modifiers);
                    let key_event = crossterm::event::KeyEvent::new(ct_code, ct_mods);
                    let rt_key = crate::frontend::tui::textarea_bridge::to_textarea_event(key_event);
                    editor.input(rt_key);
                    app_core.needs_render = true;
                }
            }
        }
        Ok(None)
    }
}

// ============================================================================
// TUI RUNTIME ORCHESTRATION
// ============================================================================

/// Run the TUI frontend with the given configuration.
/// This is the main entry point for TUI mode.
pub fn run(
    config: crate::config::Config,
    character: Option<String>,
    direct: Option<crate::network::DirectConnectConfig>,
) -> anyhow::Result<()> {
    // Use tokio runtime for async network I/O
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async_run(config, character, direct))
}

/// Async TUI main loop with network support
async fn async_run(
    config: crate::config::Config,
    character: Option<String>,
    direct: Option<crate::network::DirectConnectConfig>,
) -> anyhow::Result<()> {
    use crate::core::AppCore;
    use crate::network::{DirectConnection, LichConnection, ServerMessage};
    use tokio::sync::mpsc;

    // Create channels for network communication
    let (server_tx, mut server_rx) = mpsc::unbounded_channel::<ServerMessage>();
    let (command_tx, command_rx) = mpsc::unbounded_channel::<String>();

    // Store connection info
    let host = config.connection.host.clone();
    let port = config.connection.port;

    // Create core application state
    let mut app_core = AppCore::new(config)?;

    // Create TUI frontend
    let mut frontend = TuiFrontend::new()?;
    // Ensure frontend theme cache matches whatever layout/theme AppCore activated
    let initial_theme_id = app_core.config.active_theme.clone();
    let initial_theme = app_core.config.get_theme();
    frontend.update_theme_cache(initial_theme_id, initial_theme);

    // Initialize command input widget BEFORE any rendering
    // This ensures it exists when we start routing keys to it
    frontend.ensure_command_input_exists("command_input");

    // Load command history
    if let Err(e) = frontend.command_input_load_history("command_input", character.as_deref()) {
        tracing::warn!("Failed to load command history: {}", e);
    }

    // Get terminal size and initialize windows
    let (width, height) = frontend.size();
    app_core.init_windows(width, height);

    // Spawn network connection task
    let network_handle = match direct {
        Some(cfg) => tokio::spawn(async move {
            if let Err(e) = DirectConnection::start(cfg, server_tx, command_rx).await {
                tracing::error!(error = ?e, "Network connection error");
            }
        }),
        None => {
            let host_clone = host.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    LichConnection::start(&host_clone, port, server_tx, command_rx).await
                {
                    tracing::error!(error = ?e, "Network connection error");
                }
            })
        }
    };

    // Track time for periodic countdown updates
    let mut last_countdown_update = std::time::Instant::now();

    // Main event loop
    while app_core.running {
        // Poll for frontend events (keyboard, mouse, resize)
        let events = frontend.poll_events()?;

        // Poll TTS callback events for auto-play
        app_core.poll_tts_events();

        // Process frontend events
        for event in events {
            // Handle events that need frontend access directly
            match &event {
                crate::frontend::FrontendEvent::Mouse(mouse_event) => {
                    // Phase 4.1: Delegate to TuiFrontend::handle_mouse_event
                    let (handled, command) = frontend.handle_mouse_event(
                        mouse_event,
                        &mut app_core,
                        menu_actions::handle_menu_action
                    )?;

                    if let Some(cmd) = command {
                        let _ = command_tx.send(cmd);
                    }

                    if handled {
                        continue;
                    }
                }
                crate::frontend::FrontendEvent::Key { code, modifiers } => {
                    // Key events are handled in handle_event()
                    // No early intercepts - let the 3-layer routing handle everything
                }
                _ => {}
            }

            if let Some(command) = handle_event(&mut app_core, &mut frontend, event)? {
                let _ = command_tx.send(command);
            }
        }

        // Poll for server messages (non-blocking)
        while let Ok(msg) = server_rx.try_recv() {
            match msg {
                ServerMessage::Text(line) => {
                    // Process incoming server data through parser
                    if let Err(e) = app_core.process_server_data(&line) {
                        tracing::error!("Error processing server data: {}", e);
                    }
                    // Check for highlight sound triggers
                    app_core.check_sound_triggers(&line);
                }
                ServerMessage::Connected => {
                    tracing::info!("Connected to game server");
                    app_core.game_state.connected = true;
                    app_core.needs_render = true;
                }
                ServerMessage::Disconnected => {
                    tracing::info!("Disconnected from game server");
                    app_core.game_state.connected = false;
                    app_core.needs_render = true;
                }
            }
        }

        // Force render every second for countdown widgets
        if last_countdown_update.elapsed().as_secs() >= 1 {
            app_core.needs_render = true;
            last_countdown_update = std::time::Instant::now();
        }

        // Render if needed
        if app_core.needs_render {
            frontend.render(&mut app_core)?;
            app_core.needs_render = false;
        }

        // No sleep needed - event::poll() timeout already limits frame rate to ~60 FPS
    }

    // Save command history
    if let Err(e) = frontend.command_input_save_history("command_input", character.as_deref()) {
        tracing::warn!("Failed to save command history: {}", e);
    }

    // Cleanup
    frontend.cleanup()?;

    // Wait for network task to finish (or abort it)
    network_handle.abort();
    let _ = network_handle.await;

    Ok(())
}

/// Handle a frontend event
/// Returns Some(command) if a command should be sent to the server
fn handle_event(
    app_core: &mut crate::core::AppCore,
    frontend: &mut TuiFrontend,
    event: crate::frontend::FrontendEvent,
) -> anyhow::Result<Option<String>> {
    use crate::frontend::FrontendEvent;

    match event {
        FrontendEvent::Key { code, modifiers } => {
            // Phase 4.2: Delegate all keyboard handling to TuiFrontend::handle_key_event()
            return frontend.handle_key_event(code, modifiers, app_core, menu_actions::handle_menu_action);
        }
        FrontendEvent::Resize { width, height } => {
            // DISABLED: Automatic resize on terminal resize (manual .resize command only)
            tracing::info!("Terminal resized to {}x{} (auto-resize disabled, use .resize command)", width, height);
        }
        _ => {}
    }

    Ok(None)
}
