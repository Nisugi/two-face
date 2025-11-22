//! Modal editor for window definitions used by the TUI layout manager.
//!
//! Presents a VellumFE-inspired popup that lets the user tweak geometry,
//! borders, and stream assignments for a given window definition.
//!
//! Uses a section-based navigation system inspired by the theme editor:
//! - Section 1: Identity (name, title, show title, locked)
//! - Section 2: Position/Size (row, col, rows, cols)
//! - Section 3: Constraints (min/max rows/cols)
//! - Section 4: Border (show, color, style, sides)
//! - Section 5: Special (widget-specific: streams, cursor colors)

use crate::config::WindowDef;
use crate::theme::EditorTheme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use tui_textarea::TextArea;

/// Field reference for section-based navigation
#[derive(Debug, Clone, Copy, PartialEq)]
enum FieldRef {
    // Text inputs
    Name,
    Title,
    Row,
    Col,
    Rows,
    Cols,
    MinRows,
    MinCols,
    MaxRows,
    MaxCols,
    BgColor,
    BorderColor,
    BorderStyle,
    Streams,
    TextColor,
    CursorColor,
    CursorBg,

    // Checkboxes
    ShowTitle,
    Locked,
    TransparentBg,
    ShowBorder,
    BorderTop,
    BorderBottom,
    BorderLeft,
    BorderRight,
}

impl FieldRef {
    /// Get the legacy field ID for this field (for compatibility with existing toggle/input logic)
    fn legacy_field_id(&self) -> usize {
        match self {
            FieldRef::Name => 0,
            FieldRef::Title => 1,
            FieldRef::Row => 2,
            FieldRef::Col => 3,
            FieldRef::Rows => 4,
            FieldRef::Cols => 5,
            FieldRef::MinRows => 6,
            FieldRef::MinCols => 7,
            FieldRef::MaxRows => 8,
            FieldRef::MaxCols => 9,
            FieldRef::BorderStyle => 11,
            FieldRef::ShowTitle => 12,
            FieldRef::Locked => 13,
            FieldRef::TransparentBg => 14,
            FieldRef::ShowBorder => 15,
            FieldRef::BorderTop => 16,
            FieldRef::BorderBottom => 17,
            FieldRef::BorderLeft => 18,
            FieldRef::BorderRight => 19,
            FieldRef::BgColor => 20,
            FieldRef::BorderColor => 21,
            FieldRef::Streams => 22,
            FieldRef::TextColor => 23,
            FieldRef::CursorColor => 24,
            FieldRef::CursorBg => 25,
        }
    }
}

/// A section of related fields in the window editor
struct WindowSection {
    name: &'static str,
    fields: Vec<FieldRef>,
}

/// Global field reference - includes meta fields and all section fields
#[derive(Debug, Clone, Copy, PartialEq)]
struct GlobalFieldRef {
    field: FieldRef,
    section: usize, // 0 = meta section, 1-5 = numbered sections
}

/// Window editor widget - 50x25 popup with section-based navigation
pub struct WindowEditor {
    popup_x: u16,
    popup_y: u16,
    popup_width: u16,
    popup_height: u16,
    dragging: bool,
    drag_offset_x: u16,
    drag_offset_y: u16,
    resizing: bool,
    resize_start_width: u16,
    resize_start_height: u16,
    resize_start_mouse_x: u16,
    resize_start_mouse_y: u16,

    // Section-based navigation (0 = meta section with Name/Title, 1-5 = numbered sections)
    sections: Vec<WindowSection>,
    current_section: usize,         // 0 = meta, 1-5 = sections
    global_fields: Vec<GlobalFieldRef>, // Flattened list of all fields for Tab navigation
    current_field_global: usize,    // Index in global_fields list

    pub focused_field: usize, // Legacy field index (for compatibility)

    // Text inputs
    name_input: TextArea<'static>,
    title_input: TextArea<'static>,
    row_input: TextArea<'static>,
    col_input: TextArea<'static>,
    rows_input: TextArea<'static>,
    cols_input: TextArea<'static>,
    min_rows_input: TextArea<'static>,
    min_cols_input: TextArea<'static>,
    max_rows_input: TextArea<'static>,
    max_cols_input: TextArea<'static>,
    bg_color_input: TextArea<'static>,
    border_color_input: TextArea<'static>,
    streams_input: TextArea<'static>,
    text_color_input: TextArea<'static>,
    cursor_color_input: TextArea<'static>,
    cursor_bg_input: TextArea<'static>,

    window_def: WindowDef,
    original_window_def: WindowDef,
    is_new: bool,
    status_message: String,
}

impl WindowEditor {
    fn create_textarea() -> TextArea<'static> {
        let mut ta = TextArea::default();
        ta.set_cursor_line_style(Style::default());
        ta.set_max_histories(0);
        ta
    }

    fn textarea_with_value(value: u16) -> TextArea<'static> {
        let mut ta = Self::create_textarea();
        ta.insert_str(&value.to_string());
        ta
    }

    /// Build the 5 sections based on widget type
    fn build_sections(is_command_input: bool) -> Vec<WindowSection> {
        vec![
            // Section 1: Identity
            WindowSection {
                name: "Identity",
                fields: vec![
                    FieldRef::ShowTitle,
                    FieldRef::Locked,
                    FieldRef::TransparentBg,
                    FieldRef::BgColor,
                ],
            },
            // Section 2: Position & Size
            WindowSection {
                name: "Position & Size",
                fields: vec![FieldRef::Row, FieldRef::Col, FieldRef::Rows, FieldRef::Cols],
            },
            // Section 3: Constraints
            WindowSection {
                name: "Constraints",
                fields: vec![
                    FieldRef::MinRows,
                    FieldRef::MinCols,
                    FieldRef::MaxRows,
                    FieldRef::MaxCols,
                ],
            },
            // Section 4: Border
            WindowSection {
                name: "Border",
                fields: vec![
                    FieldRef::ShowBorder,
                    FieldRef::BorderStyle,
                    FieldRef::BorderColor,
                    FieldRef::BorderTop,
                    FieldRef::BorderBottom,
                    FieldRef::BorderLeft,
                    FieldRef::BorderRight,
                ],
            },
            // Section 5: Special (widget-specific)
            WindowSection {
                name: "Special",
                fields: if is_command_input {
                    vec![
                        FieldRef::TextColor,
                        FieldRef::CursorColor,
                        FieldRef::CursorBg,
                    ]
                } else {
                    vec![FieldRef::Streams]
                },
            },
        ]
    }

    /// Build the global fields list for Tab navigation
    /// Section 0 (meta): Name, Title
    /// Sections 1-5: All fields in each section
    fn build_global_fields(sections: &[WindowSection]) -> Vec<GlobalFieldRef> {
        let mut global_fields = Vec::new();

        // Meta section (section 0) - Name and Title only
        global_fields.push(GlobalFieldRef {
            field: FieldRef::Name,
            section: 0,
        });
        global_fields.push(GlobalFieldRef {
            field: FieldRef::Title,
            section: 0,
        });

        // Add all fields from numbered sections (1-5)
        for (section_idx, section) in sections.iter().enumerate() {
            for field_ref in &section.fields {
                global_fields.push(GlobalFieldRef {
                    field: *field_ref,
                    section: section_idx + 1, // Section 1-5 (0 is meta)
                });
            }
        }

        global_fields
    }

    fn refresh_size_inputs(&mut self) {
        self.rows_input = Self::textarea_with_value(self.window_def.base().content_rows().max(1));
        self.cols_input = Self::textarea_with_value(self.window_def.base().content_cols().max(1));
    }

    pub fn new(window_def: WindowDef) -> Self {
        let mut name_input = Self::create_textarea();
        name_input.insert_str(&window_def.name());

        let mut title_input = Self::create_textarea();
        if let Some(ref title) = window_def.base().title {
            title_input.insert_str(title);
        }

        let mut row_input = Self::create_textarea();
        row_input.insert_str(&window_def.base().row.to_string());

        let mut col_input = Self::create_textarea();
        col_input.insert_str(&window_def.base().col.to_string());

        let rows_input = Self::textarea_with_value(window_def.base().content_rows().max(1));

        let cols_input = Self::textarea_with_value(window_def.base().content_cols().max(1));

        let mut min_rows_input = Self::create_textarea();
        if let Some(min_rows) = window_def.base().min_rows {
            min_rows_input.insert_str(&min_rows.to_string());
        }

        let mut min_cols_input = Self::create_textarea();
        if let Some(min_cols) = window_def.base().min_cols {
            min_cols_input.insert_str(&min_cols.to_string());
        }

        let mut max_rows_input = Self::create_textarea();
        if let Some(max_rows) = window_def.base().max_rows {
            max_rows_input.insert_str(&max_rows.to_string());
        }

        let mut max_cols_input = Self::create_textarea();
        if let Some(max_cols) = window_def.base().max_cols {
            max_cols_input.insert_str(&max_cols.to_string());
        }

        let mut bg_color_input = Self::create_textarea();
        if let Some(ref bg_color) = window_def.base().background_color {
            bg_color_input.insert_str(bg_color);
        }

        let mut border_color_input = Self::create_textarea();
        if let Some(ref border_color) = window_def.base().border_color {
            border_color_input.insert_str(border_color);
        }

        let mut streams_input = Self::create_textarea();
        if let crate::config::WindowDef::Text { data, .. } = &window_def {
            streams_input.insert_str(&data.streams.join(", "));
        }

        let mut text_color_input = Self::create_textarea();
        let mut cursor_color_input = Self::create_textarea();
        let mut cursor_bg_input = Self::create_textarea();
        if let crate::config::WindowDef::CommandInput { data, .. } = &window_def {
            if let Some(ref color) = data.text_color {
                text_color_input.insert_str(color);
            }
            if let Some(ref color) = data.cursor_color {
                cursor_color_input.insert_str(color);
            }
            if let Some(ref color) = data.cursor_background_color {
                cursor_bg_input.insert_str(color);
            }
        }

        let is_command_input = matches!(window_def, WindowDef::CommandInput{..});
        let sections = Self::build_sections(is_command_input);
        let global_fields = Self::build_global_fields(&sections);

        Self {
            popup_x: 0,
            popup_y: 0,
            popup_width: 50,
            popup_height: 25,
            dragging: false,
            drag_offset_x: 0,
            drag_offset_y: 0,
            resizing: false,
            resize_start_width: 0,
            resize_start_height: 0,
            resize_start_mouse_x: 0,
            resize_start_mouse_y: 0,
            sections,
            current_section: 0, // Start on meta section
            global_fields,
            current_field_global: 0, // Start on Name field
            focused_field: 0,
            name_input,
            title_input,
            row_input,
            col_input,
            rows_input,
            cols_input,
            min_rows_input,
            min_cols_input,
            max_rows_input,
            max_cols_input,
            bg_color_input,
            border_color_input,
            streams_input,
            text_color_input,
            cursor_color_input,
            cursor_bg_input,
            window_def: window_def.clone(),
            original_window_def: window_def,
            is_new: false,
            status_message: "Tab/Shift+Tab: Navigate | Ctrl+1..9: Jump to section | Ctrl+S: Save | Esc: Back/Cancel".to_string(),
        }
    }

    /// Create editor for a new window from a template
    pub fn new_from_template(template: WindowDef) -> Self {
        // Create editor with template (reuse new() logic)
        let mut editor = Self::new(template);
        // Mark as new so Ctrl+S adds instead of updates
        editor.is_new = true;
        editor
    }

    pub fn new_window(widget_type: String) -> Self {
        use crate::config::{
            BorderSides, CommandInputWidgetData, RoomWidgetData, TextWidgetData, WindowBase,
            WindowDef,
        };

        // Create base configuration with defaults
        let base = WindowBase {
            name: String::new(),
            row: 0,
            col: 0,
            rows: 10,
            cols: 40,
            show_border: true,
            border_style: "single".to_string(),
            border_sides: BorderSides::default(),
            border_color: None,
            show_title: false,
            title: None,
            background_color: None,
            text_color: None,
            transparent_background: true,
            locked: false,
            min_rows: None,
            max_rows: None,
            min_cols: None,
            max_cols: None,
            visible: true,
        };

        // Create window_def based on widget type
        let window_def = match widget_type.to_lowercase().as_str() {
            "text" => WindowDef::Text {
                base,
                data: TextWidgetData {
                    streams: vec![],
                    buffer_size: 10000,
                },
            },
            "room" => WindowDef::Room {
                base,
                data: RoomWidgetData {
                    buffer_size: 0,
                    show_desc: true,
                    show_objs: true,
                    show_players: true,
                    show_exits: true,
                    show_name: false,
                },
            },
            "command_input" => WindowDef::CommandInput {
                base,
                data: CommandInputWidgetData::default(),
            },
            _ => WindowDef::Text {
                base,
                data: TextWidgetData {
                    streams: vec![],
                    buffer_size: 10000,
                },
            },
        };

        let name_input = Self::create_textarea();
        let title_input = Self::create_textarea();

        let mut row_input = Self::create_textarea();
        row_input.insert_str("0");

        let mut col_input = Self::create_textarea();
        col_input.insert_str("0");

        let rows_input = Self::textarea_with_value(window_def.base().content_rows().max(1));

        let cols_input = Self::textarea_with_value(window_def.base().content_cols().max(1));

        let min_rows_input = Self::create_textarea();
        let min_cols_input = Self::create_textarea();
        let max_rows_input = Self::create_textarea();
        let max_cols_input = Self::create_textarea();
        let bg_color_input = Self::create_textarea();
        let border_color_input = Self::create_textarea();
        let streams_input = Self::create_textarea();
        let text_color_input = Self::create_textarea();
        let cursor_color_input = Self::create_textarea();
        let cursor_bg_input = Self::create_textarea();

        let is_command_input = matches!(window_def, WindowDef::CommandInput{..});
        let sections = Self::build_sections(is_command_input);
        let global_fields = Self::build_global_fields(&sections);

        Self {
            popup_x: 0,
            popup_y: 0,
            popup_width: 50,
            popup_height: 25,
            dragging: false,
            drag_offset_x: 0,
            drag_offset_y: 0,
            resizing: false,
            resize_start_width: 0,
            resize_start_height: 0,
            resize_start_mouse_x: 0,
            resize_start_mouse_y: 0,
            sections,
            current_section: 0, // Start on meta section
            global_fields,
            current_field_global: 0, // Start on Name field
            focused_field: 0,
            name_input,
            title_input,
            row_input,
            col_input,
            rows_input,
            cols_input,
            min_rows_input,
            min_cols_input,
            max_rows_input,
            max_cols_input,
            bg_color_input,
            border_color_input,
            streams_input,
            text_color_input,
            cursor_color_input,
            cursor_bg_input,
            window_def: window_def.clone(),
            original_window_def: window_def,
            is_new: true,
            status_message: "Tab/Shift+Tab: Navigate | Ctrl+1..9: Jump to section | Ctrl+S: Save | Esc: Back/Cancel".to_string(),
        }
    }

    fn is_command_input(&self) -> bool {
        matches!(self.window_def, WindowDef::CommandInput { .. })
    }

    fn total_fields(&self) -> usize {
        if self.is_command_input() {
            26
        } else {
            23
        }
    }

    /// Jump to a specific section (Ctrl+1..5)
    /// Jumps to the first field of the target section
    pub fn jump_to_section(&mut self, section_idx: usize) {
        // section_idx is 1-5, we need to find the first field of that section
        if section_idx == 0 || section_idx > self.sections.len() {
            return;
        }

        // Find the first field in the target section
        for (global_idx, global_field) in self.global_fields.iter().enumerate() {
            if global_field.section == section_idx {
                self.current_field_global = global_idx;
                self.current_section = section_idx;
                self.sync_focused_field();
                return;
            }
        }
    }

    /// Return to meta section (Esc from within a numbered section)
    pub fn return_to_meta(&mut self) {
        self.current_section = 0;
        self.current_field_global = 0; // First field (Name)
        self.sync_focused_field();
    }

    /// Check if we're viewing the meta section
    pub fn is_on_meta(&self) -> bool {
        self.current_section == 0
    }

    /// Move to next field (Tab)
    pub fn next_field(&mut self) {
        if self.global_fields.is_empty() {
            return;
        }

        self.current_field_global = (self.current_field_global + 1) % self.global_fields.len();

        // Update current_section based on the new field
        if let Some(global_field) = self.global_fields.get(self.current_field_global) {
            self.current_section = global_field.section;
        }

        self.sync_focused_field();
    }

    /// Move to previous field (Shift+Tab)
    pub fn previous_field(&mut self) {
        if self.global_fields.is_empty() {
            return;
        }

        self.current_field_global = if self.current_field_global == 0 {
            self.global_fields.len() - 1
        } else {
            self.current_field_global - 1
        };

        // Update current_section based on the new field
        if let Some(global_field) = self.global_fields.get(self.current_field_global) {
            self.current_section = global_field.section;
        }

        self.sync_focused_field();
    }

    /// Sync the legacy focused_field index with current global field
    fn sync_focused_field(&mut self) {
        if let Some(global_field) = self.global_fields.get(self.current_field_global) {
            self.focused_field = global_field.field.legacy_field_id();
        }
    }

    /// Tab navigation (calls next_field for compatibility)
    pub fn next(&mut self) {
        self.next_field();
    }

    /// Shift+Tab navigation (calls previous_field for compatibility)
    pub fn previous(&mut self) {
        self.previous_field();
    }

    /// Check if the currently focused field is a checkbox (fields 12-19)
    pub fn is_on_checkbox(&self) -> bool {
        matches!(self.focused_field, 12..=19)
    }

    /// Check if the currently focused field is the border style dropdown
    pub fn is_on_border_style(&self) -> bool {
        self.focused_field == 11
    }

    /// Cycle to the next border style
    pub fn cycle_border_style(&mut self) {
        let current = &self.window_def.base().border_style;
        let next = match current.as_str() {
            "single" => "double",
            "double" => "rounded",
            "rounded" => "thick",
            "thick" => "single",
            _ => "single", // Default fallback
        };
        self.window_def.base_mut().border_style = next.to_string();
    }

    pub fn input(&mut self, input: ratatui::crossterm::event::KeyEvent) {
        // Route input to appropriate TextArea based on focused_field
        match self.focused_field {
            0 => {
                self.name_input.input(input.clone());
            }
            1 => {
                self.title_input.input(input.clone());
            }
            2 => {
                self.row_input.input(input.clone());
            }
            3 => {
                self.col_input.input(input.clone());
            }
            4 => {
                self.rows_input.input(input.clone());
            }
            5 => {
                self.cols_input.input(input.clone());
            }
            6 => {
                self.min_rows_input.input(input.clone());
            }
            7 => {
                self.min_cols_input.input(input.clone());
            }
            8 => {
                self.max_rows_input.input(input.clone());
            }
            9 => {
                self.max_cols_input.input(input.clone());
            }
            20 => {
                self.bg_color_input.input(input.clone());
            }
            21 => {
                self.border_color_input.input(input.clone());
            }
            22 => {
                self.streams_input.input(input.clone());
            }
            23 => {
                self.text_color_input.input(input.clone());
            }
            24 => {
                self.cursor_color_input.input(input.clone());
            }
            25 => {
                self.cursor_bg_input.input(input.clone());
            }
            _ => {} // Checkboxes/dropdowns don't handle text input
        }
    }

    pub fn toggle_field(&mut self) {
        match self.focused_field {
            12 => {
                let current = self.window_def.base().show_title;
                self.window_def.base_mut().show_title = !current;
            }
            13 => {
                let current = self.window_def.base().locked;
                self.window_def.base_mut().locked = !current;
            }
            14 => {
                let current = self.window_def.base().transparent_background;
                self.window_def.base_mut().transparent_background = !current;
            }
            15 => {
                let new_show = !self.window_def.base().show_border;
                let sides = self.window_def.base().border_sides.clone();
                self.window_def
                    .base_mut()
                    .apply_border_configuration(new_show, sides);
                self.refresh_size_inputs();
            }
            16 => {
                let show_border = self.window_def.base().show_border;
                let mut sides = self.window_def.base().border_sides.clone();
                sides.top = !sides.top;
                self.window_def
                    .base_mut()
                    .apply_border_configuration(show_border, sides);
                self.refresh_size_inputs();
            }
            17 => {
                let show_border = self.window_def.base().show_border;
                let mut sides = self.window_def.base().border_sides.clone();
                sides.bottom = !sides.bottom;
                self.window_def
                    .base_mut()
                    .apply_border_configuration(show_border, sides);
                self.refresh_size_inputs();
            }
            18 => {
                let show_border = self.window_def.base().show_border;
                let mut sides = self.window_def.base().border_sides.clone();
                sides.left = !sides.left;
                self.window_def
                    .base_mut()
                    .apply_border_configuration(show_border, sides);
                self.refresh_size_inputs();
            }
            19 => {
                let show_border = self.window_def.base().show_border;
                let mut sides = self.window_def.base().border_sides.clone();
                sides.right = !sides.right;
                self.window_def
                    .base_mut()
                    .apply_border_configuration(show_border, sides);
                self.refresh_size_inputs();
            }
            _ => {}
        }
    }

    pub fn sync_to_window_def(&mut self) {
        self.window_def.base_mut().name = self.name_input.lines()[0].to_string();
        self.window_def.base_mut().title =
            Some(self.title_input.lines()[0].to_string()).filter(|s| !s.is_empty());
        self.window_def.base_mut().row = self.row_input.lines()[0].parse().unwrap_or(0);
        self.window_def.base_mut().col = self.col_input.lines()[0].parse().unwrap_or(0);
        let rows_lines = self.rows_input.lines();
        let content_rows = rows_lines
            .get(0)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(1);
        let cols_lines = self.cols_input.lines();
        let content_cols = cols_lines
            .get(0)
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(40);
        let border_rows = self.window_def.base().horizontal_border_units();
        let border_cols = self.window_def.base().vertical_border_units();
        self.window_def.base_mut().rows = content_rows.saturating_add(border_rows).max(1);
        self.window_def.base_mut().cols = content_cols.saturating_add(border_cols).max(1);
        self.window_def.base_mut().min_rows = self.min_rows_input.lines()[0].parse().ok();
        self.window_def.base_mut().min_cols = self.min_cols_input.lines()[0].parse().ok();
        self.window_def.base_mut().max_rows = self.max_rows_input.lines()[0].parse().ok();
        self.window_def.base_mut().max_cols = self.max_cols_input.lines()[0].parse().ok();
        self.window_def.base_mut().background_color =
            Some(self.bg_color_input.lines()[0].to_string()).filter(|s| !s.is_empty());
        self.window_def.base_mut().border_color =
            Some(self.border_color_input.lines()[0].to_string()).filter(|s| !s.is_empty());

        // Update streams only for Text variant
        if let crate::config::WindowDef::Text { data, .. } = &mut self.window_def {
            let streams: Vec<String> = self.streams_input.lines()[0]
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            data.streams = streams;
        }

        if let crate::config::WindowDef::CommandInput { data, .. } = &mut self.window_def {
            data.text_color =
                Some(self.text_color_input.lines()[0].trim().to_string()).filter(|s| !s.is_empty());
            data.cursor_color = Some(self.cursor_color_input.lines()[0].trim().to_string())
                .filter(|s| !s.is_empty());
            data.cursor_background_color =
                Some(self.cursor_bg_input.lines()[0].trim().to_string()).filter(|s| !s.is_empty());
        }
    }

    pub fn get_window_def(&mut self) -> &WindowDef {
        self.sync_to_window_def();
        &self.window_def
    }

    pub fn is_new(&self) -> bool {
        self.is_new
    }

    pub fn cancel(&mut self) {
        self.window_def = self.original_window_def.clone();
    }

    /// Get the current editor window position and size for persistence
    pub fn get_editor_geometry(&self) -> (u16, u16, u16, u16) {
        (self.popup_x, self.popup_y, self.popup_width, self.popup_height)
    }

    pub fn handle_mouse(&mut self, mouse_col: u16, mouse_row: u16, mouse_down: bool, area: Rect) {
        if !mouse_down {
            self.dragging = false;
            self.resizing = false;
            return;
        }

        let popup_area = Rect {
            x: self.popup_x,
            y: self.popup_y,
            width: self.popup_width,
            height: self.popup_height,
        };

        // Check if mouse is on the resize handle (bottom-right corner, 3x3 area)
        let resize_handle_x = self.popup_x + self.popup_width.saturating_sub(3);
        let resize_handle_y = self.popup_y + self.popup_height.saturating_sub(3);
        let on_resize_handle = mouse_col >= resize_handle_x
            && mouse_col < self.popup_x + self.popup_width
            && mouse_row >= resize_handle_y
            && mouse_row < self.popup_y + self.popup_height;

        // Check if mouse is on the title bar (for dragging)
        let on_title_bar = mouse_row == self.popup_y
            && mouse_col > popup_area.x
            && mouse_col < popup_area.x + popup_area.width.saturating_sub(1);

        // Start resize if on resize handle
        if on_resize_handle && !self.resizing && !self.dragging {
            self.resizing = true;
            self.resize_start_width = self.popup_width;
            self.resize_start_height = self.popup_height;
            self.resize_start_mouse_x = mouse_col;
            self.resize_start_mouse_y = mouse_row;
        }

        // Start drag if on title bar and not resizing
        if on_title_bar && !self.dragging && !self.resizing {
            self.dragging = true;
            self.drag_offset_x = mouse_col.saturating_sub(self.popup_x);
            self.drag_offset_y = mouse_row.saturating_sub(self.popup_y);
        }

        // Handle dragging
        if self.dragging {
            self.popup_x = mouse_col.saturating_sub(self.drag_offset_x);
            self.popup_y = mouse_row.saturating_sub(self.drag_offset_y);
            self.popup_x = self.popup_x.min(area.width.saturating_sub(self.popup_width));
            self.popup_y = self.popup_y.min(area.height.saturating_sub(self.popup_height));
        }

        // Handle resizing
        if self.resizing {
            let delta_x = (mouse_col as i32) - (self.resize_start_mouse_x as i32);
            let delta_y = (mouse_row as i32) - (self.resize_start_mouse_y as i32);

            // Calculate new dimensions with minimum constraints
            let new_width = ((self.resize_start_width as i32) + delta_x).max(40) as u16;
            let new_height = ((self.resize_start_height as i32) + delta_y).max(20) as u16;

            // Apply maximum constraints based on available screen space
            self.popup_width = new_width.min(area.width.saturating_sub(self.popup_x));
            self.popup_height = new_height.min(area.height.saturating_sub(self.popup_y));
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, theme: &EditorTheme) {
        // Center the popup on first render
        if self.popup_x == 0 && self.popup_y == 0 {
            self.popup_x = (area.width.saturating_sub(self.popup_width)) / 2;
            self.popup_y = (area.height.saturating_sub(self.popup_height)) / 2;
        }

        // Constrain position to screen bounds
        self.popup_x = self.popup_x.min(area.width.saturating_sub(self.popup_width));
        self.popup_y = self.popup_y.min(area.height.saturating_sub(self.popup_height));

        let popup_area = Rect {
            x: self.popup_x,
            y: self.popup_y,
            width: self.popup_width,
            height: self.popup_height,
        };

        Clear.render(popup_area, buf);

        for y in popup_area.y..popup_area.y + popup_area.height {
            for x in popup_area.x..popup_area.x + popup_area.width {
                if x < area.width && y < area.height {
                    let cell = &mut buf[(x, y)];
                    cell.set_char(' ').set_bg(Color::Black);
                }
            }
        }

        let title = if self.is_new {
            " Add Window "
        } else {
            " Edit Window "
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .title(title)
            .style(Style::default().bg(Color::Black).fg(theme.border_color));
        block.render(popup_area, buf);

        let content = Rect {
            x: popup_area.x + 1,
            y: popup_area.y + 1,
            width: popup_area.width.saturating_sub(2),
            height: popup_area.height.saturating_sub(3),
        };

        self.render_fields(content, buf, theme);

        // Render resize handle indicator at bottom-right corner
        let resize_x = self.popup_x + self.popup_width.saturating_sub(2);
        let resize_y = self.popup_y + self.popup_height.saturating_sub(1);
        if resize_x < area.width && resize_y < area.height {
            buf.set_string(
                resize_x,
                resize_y,
                "╬",
                Style::default().fg(theme.border_color),
            );
        }
    }

    fn render_fields(&mut self, area: Rect, buf: &mut Buffer, theme: &EditorTheme) {
        let x = area.x;
        let mut y = area.y + 1; // Start with spacing from top border

        if self.current_section == 0 {
            // Meta section - show Name, Title, and Available Sections list
            let current_global_field = self.global_fields.get(self.current_field_global);

            // Render Name field
            let is_name_focused = current_global_field
                .map(|gf| gf.field == FieldRef::Name)
                .unwrap_or(false);
            self.render_textarea_compact(
                0,
                "Name:",
                &self.name_input,
                x,
                y,
                20,
                buf,
                theme,
                is_name_focused,
            );
            y += 2; // Skip one empty row between Name and Title

            // Render Title field
            let is_title_focused = current_global_field
                .map(|gf| gf.field == FieldRef::Title)
                .unwrap_or(false);
            self.render_textarea_compact(
                1,
                "Title:",
                &self.title_input,
                x,
                y,
                20,
                buf,
                theme,
                is_title_focused,
            );
            y += 3; // Skip two empty rows before "Available Sections:"

            // Available Sections header
            buf.set_string(
                x,
                y,
                "Available Sections:",
                Style::default().fg(theme.label_color),
            );
            y += 1; // Normal increment to section list

            // List sections 1-5 with Ctrl+# shortcuts
            for (idx, section) in self.sections.iter().enumerate() {
                if y >= area.y + area.height {
                    break;
                }

                let shortcut = format!("  Ctrl+{}  {}", idx + 1, section.name);
                buf.set_string(x, y, &shortcut, Style::default().fg(theme.text_color));
                y += 1;
            }
        } else if let Some(section) = self.sections.get(self.current_section - 1) {
            // Viewing a numbered section (1-5) - render its fields
            // Section header
            let header = format!("--- {} ---", section.name);
            buf.set_string(x, y, &header, Style::default().fg(theme.section_header_color));
            y += 1;

            // Render only fields in the current section
            for field_ref in &section.fields {
                if y >= area.y + area.height {
                    break;
                }

                // Check if this field is the currently focused field
                let is_current = self
                    .global_fields
                    .get(self.current_field_global)
                    .map(|gf| gf.field == *field_ref)
                    .unwrap_or(false);
                let field_id = field_ref.legacy_field_id();

                // Render based on field type
                match field_ref {
                    FieldRef::Name => {
                        self.render_textarea_compact(field_id, "Name:", &self.name_input, x, y, 20, buf, theme, is_current);
                    }
                    FieldRef::Title => {
                        self.render_textarea_compact(field_id, "Title:", &self.title_input, x, y, 20, buf, theme, is_current);
                    }
                    FieldRef::Row => {
                        self.render_textarea_compact(field_id, "Row:", &self.row_input, x, y, 8, buf, theme, is_current);
                    }
                    FieldRef::Col => {
                        self.render_textarea_compact(field_id, "Col:", &self.col_input, x, y, 8, buf, theme, is_current);
                    }
                    FieldRef::Rows => {
                        self.render_textarea_compact(field_id, "Rows:", &self.rows_input, x, y, 8, buf, theme, is_current);
                    }
                    FieldRef::Cols => {
                        self.render_textarea_compact(field_id, "Cols:", &self.cols_input, x, y, 8, buf, theme, is_current);
                    }
                    FieldRef::MinRows => {
                        self.render_textarea_compact(field_id, "Min Rows:", &self.min_rows_input, x, y, 8, buf, theme, is_current);
                    }
                    FieldRef::MinCols => {
                        self.render_textarea_compact(field_id, "Min Cols:", &self.min_cols_input, x, y, 8, buf, theme, is_current);
                    }
                    FieldRef::MaxRows => {
                        self.render_textarea_compact(field_id, "Max Rows:", &self.max_rows_input, x, y, 8, buf, theme, is_current);
                    }
                    FieldRef::MaxCols => {
                        self.render_textarea_compact(field_id, "Max Cols:", &self.max_cols_input, x, y, 8, buf, theme, is_current);
                    }
                    FieldRef::BgColor => {
                        self.render_color_field(field_id, "BG Color:", &self.bg_color_input, x, y, buf, theme, is_current);
                    }
                    FieldRef::BorderColor => {
                        self.render_color_field(field_id, "Border Color:", &self.border_color_input, x, y, buf, theme, is_current);
                    }
                    FieldRef::BorderStyle => {
                        self.render_dropdown_compact(field_id, "Style:", &self.window_def.base().border_style, x, y, buf, theme, is_current);
                    }
                    FieldRef::Streams => {
                        self.render_textarea_compact(field_id, "Streams:", &self.streams_input, x, y, 20, buf, theme, is_current);
                    }
                    FieldRef::TextColor => {
                        self.render_color_field(field_id, "Text Color:", &self.text_color_input, x, y, buf, theme, is_current);
                    }
                    FieldRef::CursorColor => {
                        self.render_color_field(field_id, "Cursor FG:", &self.cursor_color_input, x, y, buf, theme, is_current);
                    }
                    FieldRef::CursorBg => {
                        self.render_color_field(field_id, "Cursor BG:", &self.cursor_bg_input, x, y, buf, theme, is_current);
                    }
                    FieldRef::ShowTitle => {
                        self.render_checkbox_compact(field_id, "Show Title", self.window_def.base().show_title, x, y, buf, theme, is_current);
                    }
                    FieldRef::Locked => {
                        self.render_checkbox_compact(field_id, "Locked", self.window_def.base().locked, x, y, buf, theme, is_current);
                    }
                    FieldRef::TransparentBg => {
                        self.render_checkbox_compact(field_id, "Transparent BG", self.window_def.base().transparent_background, x, y, buf, theme, is_current);
                    }
                    FieldRef::ShowBorder => {
                        self.render_checkbox_compact(field_id, "Show Border", self.window_def.base().show_border, x, y, buf, theme, is_current);
                    }
                    FieldRef::BorderTop => {
                        self.render_checkbox_compact(field_id, "Top Border", self.window_def.base().border_sides.top, x, y, buf, theme, is_current);
                    }
                    FieldRef::BorderBottom => {
                        self.render_checkbox_compact(field_id, "Bottom Border", self.window_def.base().border_sides.bottom, x, y, buf, theme, is_current);
                    }
                    FieldRef::BorderLeft => {
                        self.render_checkbox_compact(field_id, "Left Border", self.window_def.base().border_sides.left, x, y, buf, theme, is_current);
                    }
                    FieldRef::BorderRight => {
                        self.render_checkbox_compact(field_id, "Right Border", self.window_def.base().border_sides.right, x, y, buf, theme, is_current);
                    }
                }
                y += 1;
            }
        }
    }

    /// Render a text input field (compact format for section-based layout)
    fn render_textarea_compact(
        &self,
        field_id: usize,
        label: &str,
        textarea: &TextArea,
        x: u16,
        y: u16,
        width: usize,
        buf: &mut Buffer,
        theme: &EditorTheme,
        is_current: bool,
    ) {
        let label_color = if is_current {
            theme.focused_label_color
        } else {
            theme.label_color
        };

        let prefix = if is_current { "→ " } else { "  " };
        buf.set_string(x, y, prefix, Style::default().fg(label_color));
        buf.set_string(x + 2, y, label, Style::default().fg(label_color));

        let value = if textarea.lines().is_empty() {
            ""
        } else {
            &textarea.lines()[0]
        };
        let text_color = if is_current {
            theme.cursor_color
        } else {
            theme.text_color
        };
        let input_x = x + 2 + label.len() as u16 + 1;
        buf.set_string(input_x, y, value, Style::default().fg(text_color));
    }

    /// Render a color field with preview
    fn render_color_field(
        &self,
        field_id: usize,
        label: &str,
        textarea: &TextArea,
        x: u16,
        y: u16,
        buf: &mut Buffer,
        theme: &EditorTheme,
        is_current: bool,
    ) {
        self.render_textarea_compact(field_id, label, textarea, x, y, 10, buf, theme, is_current);

        let value = if textarea.lines().is_empty() {
            ""
        } else {
            &textarea.lines()[0]
        };
        let preview_x = x + 2 + label.len() as u16 + 1 + 10;
        self.render_color_preview(value, preview_x, y, buf, theme);
    }

    /// Render a checkbox field (compact format)
    fn render_checkbox_compact(
        &self,
        field_id: usize,
        label: &str,
        checked: bool,
        x: u16,
        y: u16,
        buf: &mut Buffer,
        theme: &EditorTheme,
        is_current: bool,
    ) {
        let label_color = if is_current {
            theme.focused_label_color
        } else {
            theme.label_color
        };

        let prefix = if is_current { "→ " } else { "  " };
        buf.set_string(x, y, prefix, Style::default().fg(label_color));

        let checkbox = if checked { "[✓]" } else { "[ ]" };
        buf.set_string(x + 2, y, checkbox, Style::default().fg(label_color));
        buf.set_string(x + 5, y, label, Style::default().fg(label_color));
    }

    /// Render a dropdown field (compact format)
    fn render_dropdown_compact(
        &self,
        field_id: usize,
        label: &str,
        value: &str,
        x: u16,
        y: u16,
        buf: &mut Buffer,
        theme: &EditorTheme,
        is_current: bool,
    ) {
        let label_color = if is_current {
            theme.focused_label_color
        } else {
            theme.label_color
        };

        let prefix = if is_current { "→ " } else { "  " };
        buf.set_string(x, y, prefix, Style::default().fg(label_color));
        buf.set_string(x + 2, y, label, Style::default().fg(label_color));

        let display = format!("{} ▼", value);
        let input_x = x + 2 + label.len() as u16 + 1;
        buf.set_string(input_x, y, &display, Style::default().fg(theme.text_color));
    }

    fn render_textarea(
        &self,
        field_id: usize,
        label: &str,
        textarea: &TextArea,
        x: u16,
        y: u16,
        width: usize,
        spacing: u16,
        buf: &mut Buffer,
        theme: &EditorTheme,
    ) {
        let is_focused = self.focused_field == field_id;
        let label_color = if is_focused {
            theme.focused_label_color
        } else {
            theme.label_color
        };

        buf.set_string(x, y, label, Style::default().fg(label_color));
        let input_x = x + label.len() as u16 + spacing;

        // Render textarea content inline
        let value = if textarea.lines().is_empty() {
            ""
        } else {
            &textarea.lines()[0]
        };
        let text_color = if is_focused {
            theme.cursor_color
        } else {
            theme.text_color
        };
        buf.set_string(input_x, y, value, Style::default().fg(text_color));
    }

    fn render_textarea_with_preview(
        &self,
        field_id: usize,
        label: &str,
        textarea: &TextArea,
        x: u16,
        y: u16,
        width: usize,
        spacing: u16,
        buf: &mut Buffer,
        theme: &EditorTheme,
    ) {
        self.render_textarea(field_id, label, textarea, x, y, width, spacing, buf, theme);
        let input_x = x + label.len() as u16 + spacing;
        let preview_x = input_x + width as u16 + 2;
        let value = if textarea.lines().is_empty() {
            ""
        } else {
            &textarea.lines()[0]
        };
        self.render_color_preview(value, preview_x, y, buf, theme);
    }

    fn render_color_preview(
        &self,
        color_str: &str,
        x: u16,
        y: u16,
        buf: &mut Buffer,
        theme: &EditorTheme,
    ) {
        let color = if color_str.starts_with('#') && color_str.len() == 7 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&color_str[1..3], 16),
                u8::from_str_radix(&color_str[3..5], 16),
                u8::from_str_radix(&color_str[5..7], 16),
            ) {
                Some(Color::Rgb(r, g, b))
            } else {
                None
            }
        } else {
            None
        };

        buf.set_string(x, y, "[", Style::default().fg(theme.label_color));
        if let Some(color) = color {
            let style = Style::default().bg(color);
            buf[(x + 1, y)].set_char(' ').set_style(style);
            buf[(x + 2, y)].set_char(' ').set_style(style);
        } else {
            buf[(x + 1, y)].set_char(' ').reset();
            buf[(x + 2, y)].set_char(' ').reset();
        }
        buf.set_string(x + 3, y, "]", Style::default().fg(theme.label_color));
    }

    fn render_checkbox(
        &self,
        field_id: usize,
        label: &str,
        checked: bool,
        x: u16,
        y: u16,
        buf: &mut Buffer,
        theme: &EditorTheme,
    ) {
        let is_focused = self.focused_field == field_id;
        let label_color = if is_focused {
            theme.focused_label_color
        } else {
            theme.label_color
        };

        buf.set_string(x, y, label, Style::default().fg(label_color));
        let checkbox = if checked { "[✓]" } else { "[ ]" };
        let checkbox_x = x + 15;
        buf.set_string(checkbox_x, y, checkbox, Style::default().fg(label_color));
    }

    fn render_dropdown(
        &self,
        field_id: usize,
        label: &str,
        value: &str,
        x: u16,
        y: u16,
        buf: &mut Buffer,
        theme: &EditorTheme,
    ) {
        let is_focused = self.focused_field == field_id;
        let label_color = if is_focused {
            theme.focused_label_color
        } else {
            theme.label_color
        };

        buf.set_string(x, y, label, Style::default().fg(label_color));
        let input_x = x + label.len() as u16 + 1;
        let display = format!("{} ▼", value);
        buf.set_string(input_x, y, &display, Style::default().fg(theme.text_color));
    }
}
