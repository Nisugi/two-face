//! Modal editor for window definitions used by the TUI layout manager.
//!
//! Presents a VellumFE-inspired popup that lets the user tweak geometry,
//! borders, and stream assignments for a given window definition.

use crate::config::WindowDef;
use crate::theme::EditorTheme;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use tui_textarea::TextArea;

/// Window editor widget - 70x20 popup following VellumFE style guide
pub struct WindowEditor {
    popup_x: u16,
    popup_y: u16,
    dragging: bool,
    drag_offset_x: u16,
    drag_offset_y: u16,

    pub focused_field: usize, // 0-based field index

    // Text inputs (field indices 0-12)
    name_input: TextArea<'static>,         // 0
    title_input: TextArea<'static>,        // 1
    row_input: TextArea<'static>,          // 2
    col_input: TextArea<'static>,          // 3
    rows_input: TextArea<'static>,         // 4
    cols_input: TextArea<'static>,         // 5
    min_rows_input: TextArea<'static>,     // 6
    min_cols_input: TextArea<'static>,     // 7
    max_rows_input: TextArea<'static>,     // 8
    max_cols_input: TextArea<'static>,     // 9
    bg_color_input: TextArea<'static>,     // 20
    border_color_input: TextArea<'static>, // 21
    streams_input: TextArea<'static>,      // 22
    text_color_input: TextArea<'static>,   // 23
    cursor_color_input: TextArea<'static>, // 24
    cursor_bg_input: TextArea<'static>,    // 25

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

        Self {
            popup_x: 0,
            popup_y: 0,
            dragging: false,
            drag_offset_x: 0,
            drag_offset_y: 0,
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
            status_message: "Tab: Next | Shift+Tab: Prev | Space: Toggle | Ctrl+S: Save | Ctrl+D: Delete | Esc: Cancel".to_string(),
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

        Self {
            popup_x: 0,
            popup_y: 0,
            dragging: false,
            drag_offset_x: 0,
            drag_offset_y: 0,
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
            status_message: "Tab: Next | Shift+Tab: Prev | Space: Toggle | Ctrl+S: Save | Ctrl+D: Delete | Esc: Cancel".to_string(),
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

    pub fn next(&mut self) {
        let total = self.total_fields();
        self.focused_field = (self.focused_field + 1) % total;
    }

    pub fn previous(&mut self) {
        let total = self.total_fields();
        self.focused_field = if self.focused_field == 0 {
            total - 1
        } else {
            self.focused_field - 1
        };
    }

    /// Check if the currently focused field is a checkbox (fields 12-19)
    pub fn is_on_checkbox(&self) -> bool {
        matches!(self.focused_field, 12..=19)
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

    pub fn handle_mouse(&mut self, mouse_col: u16, mouse_row: u16, mouse_down: bool, area: Rect) {
        const POPUP_WIDTH: u16 = 70;
        const POPUP_HEIGHT: u16 = 20;

        if !mouse_down {
            self.dragging = false;
            return;
        }

        let popup_area = Rect {
            x: self.popup_x,
            y: self.popup_y,
            width: POPUP_WIDTH,
            height: POPUP_HEIGHT,
        };

        let on_title_bar = mouse_row == self.popup_y
            && mouse_col > popup_area.x
            && mouse_col < popup_area.x + popup_area.width.saturating_sub(1);

        if on_title_bar && !self.dragging {
            self.dragging = true;
            self.drag_offset_x = mouse_col.saturating_sub(self.popup_x);
            self.drag_offset_y = mouse_row.saturating_sub(self.popup_y);
        }

        if self.dragging {
            self.popup_x = mouse_col.saturating_sub(self.drag_offset_x);
            self.popup_y = mouse_row.saturating_sub(self.drag_offset_y);
            self.popup_x = self.popup_x.min(area.width.saturating_sub(POPUP_WIDTH));
            self.popup_y = self.popup_y.min(area.height.saturating_sub(POPUP_HEIGHT));
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, theme: &EditorTheme) {
        let popup_width = 70;
        let popup_height = 20;

        if self.popup_x == 0 && self.popup_y == 0 {
            self.popup_x = (area.width.saturating_sub(popup_width)) / 2;
            self.popup_y = (area.height.saturating_sub(popup_height)) / 2;
        }

        self.popup_x = self.popup_x.min(area.width.saturating_sub(popup_width));
        self.popup_y = self.popup_y.min(area.height.saturating_sub(popup_height));

        let popup_area = Rect {
            x: self.popup_x,
            y: self.popup_y,
            width: popup_width,
            height: popup_height,
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
            " Add Window (drag title to move) "
        } else {
            " Edit Window (drag title to move) "
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

        let status_y = popup_area.y + popup_area.height - 2;
        let status = Paragraph::new(&self.status_message as &str)
            .style(Style::default().fg(theme.status_color));
        status.render(
            Rect {
                x: content.x,
                y: status_y,
                width: content.width,
                height: 1,
            },
            buf,
        );
    }

    fn render_fields(&mut self, area: Rect, buf: &mut Buffer, theme: &EditorTheme) {
        let left_x = area.x;
        let right_x = area.x + 36;
        let mut y = area.y;

        // Row 0: Name + Show Title checkbox
        self.render_textarea(0, "Name:", &self.name_input, left_x, y, 23, 2, buf, theme);
        self.render_checkbox(
            12,
            "Show Title",
            self.window_def.base().show_title,
            right_x,
            y,
            buf,
            theme,
        );
        y += 1;

        // Row 1: Lock Window checkbox
        self.render_checkbox(
            13,
            "Lock Window",
            self.window_def.base().locked,
            right_x,
            y,
            buf,
            theme,
        );
        y += 1;

        // Row 2: Title + Transparent BG checkbox
        self.render_textarea(1, "Title:", &self.title_input, left_x, y, 23, 1, buf, theme);
        self.render_checkbox(
            14,
            "Transparent BG",
            self.window_def.base().transparent_background,
            right_x,
            y,
            buf,
            theme,
        );
        y += 1;

        // Row 3: BG Color + preview
        self.render_textarea_with_preview(
            20,
            "BG Color:",
            &self.bg_color_input,
            right_x,
            y,
            10,
            5,
            buf,
            theme,
        );
        y += 1;

        if self.is_command_input() {
            self.render_textarea_with_preview(
                23,
                "Text Color:",
                &self.text_color_input,
                right_x,
                y,
                10,
                3,
                buf,
                theme,
            );
            y += 1;

            self.render_textarea_with_preview(
                24,
                "Cursor FG:",
                &self.cursor_color_input,
                right_x,
                y,
                10,
                3,
                buf,
                theme,
            );
            y += 1;

            self.render_textarea_with_preview(
                25,
                "Cursor BG:",
                &self.cursor_bg_input,
                right_x,
                y,
                10,
                3,
                buf,
                theme,
            );
            y += 1;
        }

        // Row 4: Row + Col
        self.render_textarea(2, "Row:", &self.row_input, left_x, y, 8, 2, buf, theme);
        self.render_textarea(3, "Col:", &self.col_input, left_x + 16, y, 8, 2, buf, theme);
        y += 1;

        // Row 5: Rows + Cols + Show Border
        self.render_textarea(4, "Rows:", &self.rows_input, left_x, y, 8, 1, buf, theme);
        self.render_textarea(
            5,
            "Cols:",
            &self.cols_input,
            left_x + 16,
            y,
            8,
            1,
            buf,
            theme,
        );
        self.render_checkbox(
            15,
            "Show Border",
            self.window_def.base().show_border,
            right_x,
            y,
            buf,
            theme,
        );
        y += 1;

        // Row 6: Min + Min + Top Border
        self.render_textarea(6, "Min:", &self.min_rows_input, left_x, y, 8, 2, buf, theme);
        self.render_textarea(
            7,
            "Min:",
            &self.min_cols_input,
            left_x + 16,
            y,
            8,
            2,
            buf,
            theme,
        );
        let has_top = self.window_def.base().border_sides.top;
        self.render_checkbox(16, "Top Border", has_top, right_x, y, buf, theme);
        y += 1;

        // Row 7: Max + Max + Bottom Border
        self.render_textarea(8, "Max:", &self.max_rows_input, left_x, y, 8, 2, buf, theme);
        self.render_textarea(
            9,
            "Max:",
            &self.max_cols_input,
            left_x + 16,
            y,
            8,
            2,
            buf,
            theme,
        );
        let has_bottom = self.window_def.base().border_sides.bottom;
        self.render_checkbox(17, "Bottom Border", has_bottom, right_x, y, buf, theme);
        y += 1;

        // Row 8: Left Border
        let has_left = self.window_def.base().border_sides.left;
        self.render_checkbox(18, "Left Border", has_left, right_x, y, buf, theme);
        y += 1;

        // Row 9: Right Border (removed Content Align as it doesn't exist)
        let has_right = self.window_def.base().border_sides.right;
        self.render_checkbox(19, "Right Border", has_right, right_x, y, buf, theme);
        y += 1;

        // Row 10: Border Style + Border Color + preview
        self.render_dropdown(
            11,
            "Border Style:",
            &self.window_def.base().border_style,
            left_x,
            y,
            buf,
            theme,
        );
        self.render_textarea_with_preview(
            21,
            "Border Color:",
            &self.border_color_input,
            right_x,
            y,
            10,
            1,
            buf,
            theme,
        );
        y += 1;

        // Row 11: Blank
        y += 1;

        // Row 12+: Widget-specific
        if self.window_def.widget_type() == "text" {
            self.render_textarea(
                22,
                "Streams:",
                &self.streams_input,
                right_x,
                y,
                21,
                1,
                buf,
                theme,
            );
        }
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
