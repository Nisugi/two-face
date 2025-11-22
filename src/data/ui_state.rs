//! UI State - Focus, selection, and interaction state
//!
//! This module contains UI state that is independent of rendering.
//! Both TUI and GUI frontends read from these structures.

use super::window::WindowState;
use crate::data::LinkData;
use crate::selection::SelectionState;
use std::collections::HashMap;

/// Application UI state
#[derive(Clone, Debug)]
pub struct UiState {
    /// All windows in the application
    pub windows: HashMap<String, WindowState>,

    /// Widget type index - cached mapping of widget types to window names
    /// Rebuilt when windows are added/removed
    widget_type_index: HashMap<super::window::WidgetType, Vec<String>>,

    /// Currently focused window name
    pub focused_window: Option<String>,

    /// Current input mode
    pub input_mode: InputMode,

    /// Search input (when in Search mode)
    pub search_input: String,
    pub search_cursor: usize,

    /// Popup menu state (main menu or level 1)
    pub popup_menu: Option<PopupMenu>,

    /// Submenu (level 2) - shown when clicking category in popup_menu
    pub submenu: Option<PopupMenu>,

    /// Nested submenu (level 3) - shown when clicking subcategory in submenu
    pub nested_submenu: Option<PopupMenu>,

    /// Status bar text
    pub status_text: String,

    /// Mouse drag state for window resize/move
    pub mouse_drag: Option<MouseDragState>,

    /// Text selection state
    pub selection_state: Option<SelectionState>,

    /// Mouse position when drag started (for detecting drag vs click)
    pub selection_drag_start: Option<(u16, u16)>,

    /// Link drag state (Ctrl+drag from link)
    pub link_drag_state: Option<LinkDragState>,

    /// Pending link click (released without drag = send _menu)
    pub pending_link_click: Option<PendingLinkClick>,
}

/// Mouse drag state for window operations
#[derive(Clone, Debug)]
pub struct MouseDragState {
    pub operation: DragOperation,
    pub window_name: String,
    pub start_pos: (u16, u16),
    pub original_window_pos: (u16, u16, u16, u16), // x, y, width, height
}

/// Type of mouse drag operation
#[derive(Clone, Debug, PartialEq)]
pub enum DragOperation {
    Move,
    ResizeRight,
    ResizeBottom,
    ResizeBottomRight,
}

/// Link drag state (Ctrl+drag on a link)
#[derive(Clone, Debug)]
pub struct LinkDragState {
    pub link_data: LinkData,
    pub start_pos: (u16, u16),
    pub current_pos: (u16, u16),
}

/// Pending link click (mouse down on link, waiting for mouse up to send _menu)
#[derive(Clone, Debug)]
pub struct PendingLinkClick {
    pub link_data: LinkData,
    pub click_pos: (u16, u16),
}

/// Input mode for the application
#[derive(Clone, Debug, PartialEq)]
pub enum InputMode {
    /// Normal command input
    Normal,
    /// Vi-style navigation mode
    Navigation,
    /// Scrolling through history
    History,
    /// Search mode (Ctrl+F)
    Search,
    /// Popup menu is active (Tab/Shift+Tab navigation)
    Menu,
    /// Window editor is open
    WindowEditor,
    /// Highlight browser is open
    HighlightBrowser,
    /// Highlight form is open (create/edit highlight)
    HighlightForm,
    /// Keybind browser is open
    KeybindBrowser,
    /// Keybind form is open (create/edit keybind)
    KeybindForm,
    /// Color palette browser is open
    ColorPaletteBrowser,
    /// Color form is open (create/edit palette color)
    ColorForm,
    /// UI colors browser is open
    UIColorsBrowser,
    /// Spell colors browser is open
    SpellColorsBrowser,
    /// Spell color form is open (create/edit spell color)
    SpellColorForm,
    /// Theme browser is open
    ThemeBrowser,
    /// Theme editor is open (create/edit theme)
    ThemeEditor,
    /// Settings editor is open
    SettingsEditor,
}

/// Popup menu state
#[derive(Clone, Debug)]
pub struct PopupMenu {
    pub items: Vec<PopupMenuItem>,
    pub selected: usize,
    pub position: (u16, u16), // x, y position
}

/// A single popup menu item
#[derive(Clone, Debug)]
pub struct PopupMenuItem {
    pub text: String,
    pub command: String,
    pub disabled: bool,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            windows: HashMap::new(),
            widget_type_index: HashMap::new(),
            focused_window: None,
            input_mode: InputMode::Normal,
            search_input: String::new(),
            search_cursor: 0,
            popup_menu: None,
            submenu: None,
            nested_submenu: None,
            status_text: String::from("Ready"),
            mouse_drag: None,
            selection_state: None,
            selection_drag_start: None,
            link_drag_state: None,
            pending_link_click: None,
        }
    }

    /// Get a window by name
    pub fn get_window(&self, name: &str) -> Option<&WindowState> {
        self.windows.get(name)
    }

    /// Get a mutable window by name
    pub fn get_window_mut(&mut self, name: &str) -> Option<&mut WindowState> {
        self.windows.get_mut(name)
    }

    /// Add or update a window
    pub fn set_window(&mut self, name: String, window: WindowState) {
        self.windows.insert(name, window);
        self.rebuild_widget_index();
    }

    /// Remove a window by name
    pub fn remove_window(&mut self, name: &str) -> Option<WindowState> {
        let result = self.windows.remove(name);
        if result.is_some() {
            self.rebuild_widget_index();
        }
        result
    }

    /// Rebuild the widget type index cache
    /// Called whenever windows are added/removed
    pub fn rebuild_widget_index(&mut self) {
        self.widget_type_index.clear();
        for (name, window) in &self.windows {
            self.widget_type_index
                .entry(window.widget_type.clone())
                .or_insert_with(Vec::new)
                .push(name.clone());
        }
    }

    /// Get a window by widget type and optional name
    /// For singletons (Compass, InjuryDoll): pass None for name
    /// For multi-instance (Countdown, Text, etc): pass Some(name) to specify which one
    pub fn get_window_by_type(
        &self,
        widget_type: super::window::WidgetType,
        name: Option<&str>,
    ) -> Option<&WindowState> {
        let candidates = self.widget_type_index.get(&widget_type)?;

        match name {
            Some(specific_name) => {
                // Multi-instance: find the specific named window
                self.windows.get(specific_name)
            }
            None => {
                // Singleton: return the first (only) window of this type
                candidates.first().and_then(|n| self.windows.get(n))
            }
        }
    }

    /// Get a mutable window by widget type and optional name
    /// For singletons (Compass, InjuryDoll): pass None for name
    /// For multi-instance (Countdown, Text, etc): pass Some(name) to specify which one
    pub fn get_window_by_type_mut(
        &mut self,
        widget_type: super::window::WidgetType,
        name: Option<&str>,
    ) -> Option<&mut WindowState> {
        let candidates = self.widget_type_index.get(&widget_type)?;

        match name {
            Some(specific_name) => {
                // Multi-instance: find the specific named window
                self.windows.get_mut(specific_name)
            }
            None => {
                // Singleton: return the first (only) window of this type
                let window_name = candidates.first()?.clone();
                self.windows.get_mut(&window_name)
            }
        }
    }

    /// Set the focused window
    pub fn set_focus(&mut self, name: Option<String>) {
        // Clear old focus
        if let Some(old_name) = &self.focused_window {
            if let Some(window) = self.windows.get_mut(old_name) {
                window.focused = false;
            }
        }

        // Set new focus
        if let Some(new_name) = &name {
            if let Some(window) = self.windows.get_mut(new_name) {
                window.focused = true;
            }
        }

        self.focused_window = name;
    }

    /// Get the currently focused window
    pub fn focused_window(&self) -> Option<&WindowState> {
        self.focused_window
            .as_ref()
            .and_then(|name| self.windows.get(name))
    }

    /// Get the currently focused window mutably
    pub fn focused_window_mut(&mut self) -> Option<&mut WindowState> {
        let name = self.focused_window.clone();
        name.as_ref().and_then(|n| self.windows.get_mut(n))
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupMenu {
    pub fn new(items: Vec<PopupMenuItem>, position: (u16, u16)) -> Self {
        Self {
            items,
            selected: 0,
            position,
        }
    }

    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected = (self.selected + 1) % self.items.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.items.is_empty() {
            self.selected = if self.selected == 0 {
                self.items.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn selected_item(&self) -> Option<&PopupMenuItem> {
        self.items.get(self.selected)
    }

    pub fn get_selected(&self) -> Option<&PopupMenuItem> {
        self.items.get(self.selected)
    }

    pub fn get_items(&self) -> &[PopupMenuItem] {
        &self.items
    }

    pub fn get_position(&self) -> (u16, u16) {
        self.position
    }

    pub fn get_selected_index(&self) -> usize {
        self.selected
    }

    /// Check if a mouse click at (x, y) hits a menu item
    /// Returns the index of the clicked item if any
    pub fn check_click(&self, x: u16, y: u16, area: ratatui::layout::Rect) -> Option<usize> {
        // Check if click is within the menu area
        if x < area.x || x >= area.x + area.width || y < area.y || y >= area.y + area.height {
            return None;
        }

        // Calculate which item was clicked (accounting for border and title)
        let relative_y = (y - area.y) as usize;

        // Border takes 1 row at top and bottom
        if relative_y == 0 || relative_y >= area.height as usize - 1 {
            return None; // Clicked on border
        }

        let item_index = relative_y - 1; // Subtract top border

        if item_index < self.items.len() {
            Some(item_index)
        } else {
            None
        }
    }
}
