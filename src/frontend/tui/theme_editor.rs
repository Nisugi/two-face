//! Interactive theme editor used to author custom `AppTheme` files.
//!
//! Presents meta fields plus grouped color sections, supports dragging, and
//! serializes/deserializes `ThemeData` structs for persistence.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget as RatatuiWidget},
};
use tui_textarea::TextArea;

/// Result of theme editor form submission
#[derive(Debug, Clone)]
pub enum ThemeEditorResult {
    Save(ThemeData),
    Cancel,
}

/// Theme data structure for editing
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThemeData {
    pub name: String,
    pub description: String,

    // Window colors
    pub window_border: String,
    pub window_border_focused: String,
    pub window_background: String,
    pub window_title: String,

    // Text colors
    pub text_primary: String,
    pub text_secondary: String,
    pub text_disabled: String,
    pub text_selected: String,

    // Browser/List colors
    pub browser_border: String,
    pub browser_title: String,
    pub browser_item_normal: String,
    pub browser_item_selected: String,
    pub browser_item_focused: String,
    pub browser_background: String,
    pub browser_scrollbar: String,

    // Form colors
    pub form_border: String,
    pub form_label: String,
    pub form_label_focused: String,
    pub form_field_background: String,
    pub form_field_text: String,
    pub form_checkbox_checked: String,
    pub form_checkbox_unchecked: String,
    pub form_error: String,

    // Editor colors
    pub editor_border: String,
    pub editor_label: String,
    pub editor_label_focused: String,
    pub editor_text: String,
    pub editor_cursor: String,
    pub editor_status: String,
    pub editor_background: String,

    // Menu colors
    pub menu_border: String,
    pub menu_background: String,
    pub menu_item_normal: String,
    pub menu_item_selected: String,
    // menu_item_disabled: String,
    pub menu_separator: String,

    // Status bar colors
    pub status_background: String,
    // status_text: String,
    // status_highlight: String,

    // Button colors
    pub button_normal: String,
    // button_focused: String,
    pub button_disabled: String,
    // Game-specific colors
    // health_bar: String,
    // mana_bar: String,
    // stamina_bar: String,
    // experience_bar: String,
    // progress_bar_fill: String,
    // progress_bar_background: String,
    // countdown_text: String,
    // countdown_background: String,
}

impl Default for ThemeData {
    fn default() -> Self {
        Self::from_theme(&crate::theme::ThemePresets::dark())
    }
}

impl ThemeData {
    /// Create ThemeData from an existing AppTheme
    pub fn from_theme(theme: &crate::theme::AppTheme) -> Self {
        Self {
            name: theme.name.clone(),
            description: theme.description.clone(),

            window_border: Self::color_to_hex(&theme.window_border),
            window_border_focused: Self::color_to_hex(&theme.window_border_focused),
            window_background: Self::color_to_hex(&theme.window_background),
            window_title: Self::color_to_hex(&theme.window_title),

            text_primary: Self::color_to_hex(&theme.text_primary),
            text_secondary: Self::color_to_hex(&theme.text_secondary),
            text_disabled: Self::color_to_hex(&theme.text_disabled),
            text_selected: Self::color_to_hex(&theme.text_selected),

            browser_border: Self::color_to_hex(&theme.browser_border),
            browser_title: Self::color_to_hex(&theme.browser_title),
            browser_item_normal: Self::color_to_hex(&theme.browser_item_normal),
            browser_item_selected: Self::color_to_hex(&theme.browser_item_selected),
            browser_item_focused: Self::color_to_hex(&theme.browser_item_focused),
            browser_background: Self::color_to_hex(&theme.browser_background),
            browser_scrollbar: Self::color_to_hex(&theme.browser_scrollbar),

            form_border: Self::color_to_hex(&theme.form_border),
            form_label: Self::color_to_hex(&theme.form_label),
            form_label_focused: Self::color_to_hex(&theme.form_label_focused),
            form_field_background: Self::color_to_hex(&theme.form_field_background),
            form_field_text: Self::color_to_hex(&theme.form_field_text),
            form_checkbox_checked: Self::color_to_hex(&theme.form_checkbox_checked),
            form_checkbox_unchecked: Self::color_to_hex(&theme.form_checkbox_unchecked),
            form_error: Self::color_to_hex(&theme.form_error),

            editor_border: Self::color_to_hex(&theme.editor_border),
            editor_label: Self::color_to_hex(&theme.editor_label),
            editor_label_focused: Self::color_to_hex(&theme.editor_label_focused),
            editor_text: Self::color_to_hex(&theme.editor_text),
            editor_cursor: Self::color_to_hex(&theme.editor_cursor),
            editor_status: Self::color_to_hex(&theme.editor_status),
            editor_background: Self::color_to_hex(&theme.editor_background),

            menu_border: Self::color_to_hex(&theme.menu_border),
            menu_background: Self::color_to_hex(&theme.menu_background),
            menu_item_normal: Self::color_to_hex(&theme.menu_item_normal),
            menu_item_selected: Self::color_to_hex(&theme.menu_item_selected),
            // menu_item_disabled: Self::color_to_hex(&// theme.menu_item_disabled),
            menu_separator: Self::color_to_hex(&theme.menu_separator),

            status_background: Self::color_to_hex(&theme.status_background),
            // status_text: Self::color_to_hex(&// theme.status_text),
            // status_highlight: Self::color_to_hex(&// theme.status_highlight),
            button_normal: Self::color_to_hex(&theme.button_normal),
            // button_focused: Self::color_to_hex(&// theme.button_focused),
            button_disabled: Self::color_to_hex(&theme.button_disabled),
            // health_bar: Self::color_to_hex(&// theme.health_bar),
            // mana_bar: Self::color_to_hex(&// theme.mana_bar),
            // stamina_bar: Self::color_to_hex(&// theme.stamina_bar),
            // experience_bar: Self::color_to_hex(&// theme.experience_bar),
            // progress_bar_fill: Self::color_to_hex(&// theme.progress_bar_fill),
            // progress_bar_background: Self::color_to_hex(&// theme.progress_bar_background),
            // countdown_text: Self::color_to_hex(&// theme.countdown_text),
            // countdown_background: Self::color_to_hex(&// theme.countdown_background),
        }
    }

    /// Convert ratatui Color to hex string
    fn color_to_hex(color: &Color) -> String {
        match color {
            Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b),
            Color::Reset => "#000000".to_string(), // Default to black
            Color::Black => "#000000".to_string(),
            Color::Red => "#ff0000".to_string(),
            Color::Green => "#00ff00".to_string(),
            Color::Yellow => "#ffff00".to_string(),
            Color::Blue => "#0000ff".to_string(),
            Color::Magenta => "#ff00ff".to_string(),
            Color::Cyan => "#00ffff".to_string(),
            Color::Gray => "#808080".to_string(),
            Color::DarkGray => "#404040".to_string(),
            Color::LightRed => "#ff8080".to_string(),
            Color::LightGreen => "#80ff80".to_string(),
            Color::LightYellow => "#ffff80".to_string(),
            Color::LightBlue => "#8080ff".to_string(),
            Color::LightMagenta => "#ff80ff".to_string(),
            Color::LightCyan => "#80ffff".to_string(),
            Color::White => "#ffffff".to_string(),
            _ => "#000000".to_string(),
        }
    }

    /// Convert to AppTheme
    pub fn to_app_theme(&self) -> Option<crate::theme::AppTheme> {
        // Parse all colors
        let window_border = Self::parse_color(&self.window_border)?;
        let window_border_focused = Self::parse_color(&self.window_border_focused)?;
        let window_background = Self::parse_color(&self.window_background)?;
        let window_title = Self::parse_color(&self.window_title)?;

        // ... (would parse all other colors)

        Some(crate::theme::AppTheme {
            name: self.name.clone(),
            description: self.description.clone(),
            window_border,
            window_border_focused,
            window_background,
            window_title,
            // ... (would set all other fields)
            ..crate::theme::ThemePresets::dark() // Fallback for now
        })
    }

    fn parse_color(hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Color::Rgb(r, g, b))
    }

    /// Save this theme to a TOML file in ~/.two-face/themes/
    pub fn save_to_file(&self, config_base: Option<&str>) -> anyhow::Result<std::path::PathBuf> {
        use std::fs;
        use std::path::PathBuf;

        // Determine themes directory path
        let themes_dir = if let Some(base) = config_base {
            PathBuf::from(base).join("themes")
        } else {
            let home = dirs::home_dir()
                .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
            home.join(".two-face").join("themes")
        };

        // Create themes directory if it doesn't exist
        fs::create_dir_all(&themes_dir)?;

        // Sanitize filename (remove invalid characters)
        let filename = self
            .name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        let filepath = themes_dir.join(format!("{}.toml", filename));

        // Serialize to TOML
        let toml_string = toml::to_string_pretty(self)?;

        // Write to file
        fs::write(&filepath, toml_string)?;

        Ok(filepath)
    }

    /// Load a theme from a TOML file
    pub fn load_from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let theme_data: ThemeData = toml::from_str(&contents)?;
        Ok(theme_data)
    }
}

pub struct ThemeEditor {
    // Basic fields
    name: TextArea<'static>,
    description: TextArea<'static>,

    // Color sections with editable fields
    color_sections: Vec<ColorSection>,

    // Current section being edited (0=meta, 1+=color sections)
    current_section: usize,

    // Current field within section
    current_field: usize,

    // Position
    popup_x: u16,
    popup_y: u16,
}

struct ColorSection {
    name: &'static str,
    fields: Vec<ColorFieldEditor>,
}

struct ColorFieldEditor {
    label: &'static str,
    field_name: &'static str, // Maps to ThemeData field name
    textarea: TextArea<'static>,
}

impl ThemeEditor {
    pub fn new() -> Self {
        Self::new_create()
    }

    pub fn new_create() -> Self {
        let theme_data = ThemeData::default();

        let mut name = TextArea::default();
        name.set_placeholder_text("My Custom Theme");

        let mut description = TextArea::default();
        description.set_placeholder_text("A beautiful custom theme");

        let color_sections = Self::build_color_sections(&theme_data);

        Self {
            name,
            description,
            color_sections,
            current_section: 0,
            current_field: 0,
            popup_x: 0,
            popup_y: 0,
        }
    }

    pub fn new_edit(theme: &crate::theme::AppTheme) -> Self {
        let theme_data = ThemeData::from_theme(theme);

        let mut name = TextArea::default();
        name.insert_str(&theme.name);

        let mut description = TextArea::default();
        description.insert_str(&theme.description);

        let color_sections = Self::build_color_sections(&theme_data);

        Self {
            name,
            description,
            color_sections,
            current_section: 0,
            current_field: 0,
            popup_x: 0,
            popup_y: 0,
        }
    }

    fn build_color_sections(theme_data: &ThemeData) -> Vec<ColorSection> {
        // Helper macro to create a color field with its current value
        macro_rules! color_field {
            ($label:expr, $field_name:expr, $value:expr) => {{
                let mut textarea = TextArea::default();
                textarea.insert_str($value);
                ColorFieldEditor {
                    label: $label,
                    field_name: $field_name,
                    textarea,
                }
            }};
        }

        vec![
            ColorSection {
                name: "Window Colors",
                fields: vec![
                    color_field!("Border", "window_border", &theme_data.window_border),
                    color_field!(
                        "Border (Focused)",
                        "window_border_focused",
                        &theme_data.window_border_focused
                    ),
                    color_field!(
                        "Background",
                        "window_background",
                        &theme_data.window_background
                    ),
                    color_field!("Title", "window_title", &theme_data.window_title),
                ],
            },
            ColorSection {
                name: "Text Colors",
                fields: vec![
                    color_field!("Primary", "text_primary", &theme_data.text_primary),
                    color_field!("Secondary", "text_secondary", &theme_data.text_secondary),
                    color_field!("Disabled", "text_disabled", &theme_data.text_disabled),
                    color_field!("Selected", "text_selected", &theme_data.text_selected),
                ],
            },
            ColorSection {
                name: "Browser Colors",
                fields: vec![
                    color_field!("Border", "browser_border", &theme_data.browser_border),
                    color_field!("Title", "browser_title", &theme_data.browser_title),
                    color_field!(
                        "Item Normal",
                        "browser_item_normal",
                        &theme_data.browser_item_normal
                    ),
                    color_field!(
                        "Item Selected",
                        "browser_item_selected",
                        &theme_data.browser_item_selected
                    ),
                    color_field!(
                        "Item Focused",
                        "browser_item_focused",
                        &theme_data.browser_item_focused
                    ),
                    color_field!(
                        "Background",
                        "browser_background",
                        &theme_data.browser_background
                    ),
                    color_field!(
                        "Scrollbar",
                        "browser_scrollbar",
                        &theme_data.browser_scrollbar
                    ),
                ],
            },
            ColorSection {
                name: "Form Colors",
                fields: vec![
                    color_field!("Border", "form_border", &theme_data.form_border),
                    color_field!("Label", "form_label", &theme_data.form_label),
                    color_field!(
                        "Label (Focused)",
                        "form_label_focused",
                        &theme_data.form_label_focused
                    ),
                    color_field!(
                        "Field Background",
                        "form_field_background",
                        &theme_data.form_field_background
                    ),
                    color_field!("Field Text", "form_field_text", &theme_data.form_field_text),
                    color_field!(
                        "Checkbox Checked",
                        "form_checkbox_checked",
                        &theme_data.form_checkbox_checked
                    ),
                    color_field!(
                        "Checkbox Unchecked",
                        "form_checkbox_unchecked",
                        &theme_data.form_checkbox_unchecked
                    ),
                    color_field!("Error", "form_error", &theme_data.form_error),
                ],
            },
            ColorSection {
                name: "Editor Colors",
                fields: vec![
                    color_field!("Border", "editor_border", &theme_data.editor_border),
                    color_field!("Label", "editor_label", &theme_data.editor_label),
                    color_field!(
                        "Label (Focused)",
                        "editor_label_focused",
                        &theme_data.editor_label_focused
                    ),
                    color_field!("Text", "editor_text", &theme_data.editor_text),
                    color_field!("Cursor", "editor_cursor", &theme_data.editor_cursor),
                    color_field!("Status", "editor_status", &theme_data.editor_status),
                    color_field!(
                        "Background",
                        "editor_background",
                        &theme_data.editor_background
                    ),
                ],
            },
            ColorSection {
                name: "Menu Colors",
                fields: vec![
                    color_field!("Border", "menu_border", &theme_data.menu_border),
                    color_field!("Background", "menu_background", &theme_data.menu_background),
                    color_field!(
                        "Item Normal",
                        "menu_item_normal",
                        &theme_data.menu_item_normal
                    ),
                    color_field!(
                        "Item Selected",
                        "menu_item_selected",
                        &theme_data.menu_item_selected
                    ),
                    color_field!("Separator", "menu_separator", &theme_data.menu_separator),
                ],
            },
            ColorSection {
                name: "Button Colors",
                fields: vec![
                    color_field!("Normal", "button_normal", &theme_data.button_normal),
                    color_field!("Disabled", "button_disabled", &theme_data.button_disabled),
                ],
            },
            ColorSection {
                name: "Status Colors",
                fields: vec![color_field!(
                    "Background",
                    "status_background",
                    &theme_data.status_background
                )],
            },
        ]
    }

    pub fn handle_input(&mut self, key_event: KeyEvent) -> Option<ThemeEditorResult> {
        match key_event.code {
            KeyCode::Esc => {
                return Some(ThemeEditorResult::Cancel);
            }
            KeyCode::Enter if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                // Save theme - collect all edited values
                return Some(ThemeEditorResult::Save(self.collect_theme_data()));
            }
            KeyCode::Tab => {
                self.next_field();
            }
            KeyCode::BackTab => {
                self.previous_field();
            }
            _ => {
                // Forward to active field (convert KeyEvent for tui-textarea compatibility)
                let rt_key = crate::core::event_bridge::to_textarea_event(key_event);
                if self.current_section == 0 {
                    // Meta fields (name/description)
                    if self.current_field == 0 {
                        self.name.input(rt_key);
                    } else if self.current_field == 1 {
                        self.description.input(rt_key);
                    }
                } else {
                    // Color fields
                    let section_idx = self.current_section - 1;
                    if let Some(section) = self.color_sections.get_mut(section_idx) {
                        if let Some(field) = section.fields.get_mut(self.current_field) {
                            field.textarea.input(rt_key);
                        }
                    }
                }
            }
        }

        None
    }

    /// Collect all edited values into a ThemeData struct
    fn collect_theme_data(&self) -> ThemeData {
        let mut data = ThemeData::default();

        // Set name and description
        data.name = self
            .name
            .lines()
            .get(0)
            .map(|s| s.to_string())
            .unwrap_or_default();
        data.description = self
            .description
            .lines()
            .get(0)
            .map(|s| s.to_string())
            .unwrap_or_default();

        // Collect all color fields
        for section in &self.color_sections {
            for field in &section.fields {
                let value = field
                    .textarea
                    .lines()
                    .get(0)
                    .map(|s| s.to_string())
                    .unwrap_or_default();

                // Use field_name to set the correct field in ThemeData
                match field.field_name {
                    "window_border" => data.window_border = value,
                    "window_border_focused" => data.window_border_focused = value,
                    "window_background" => data.window_background = value,
                    "window_title" => data.window_title = value,
                    "text_primary" => data.text_primary = value,
                    "text_secondary" => data.text_secondary = value,
                    "text_disabled" => data.text_disabled = value,
                    "text_selected" => data.text_selected = value,
                    "browser_border" => data.browser_border = value,
                    "browser_title" => data.browser_title = value,
                    "browser_item_normal" => data.browser_item_normal = value,
                    "browser_item_selected" => data.browser_item_selected = value,
                    "browser_item_focused" => data.browser_item_focused = value,
                    "browser_background" => data.browser_background = value,
                    "browser_scrollbar" => data.browser_scrollbar = value,
                    "form_border" => data.form_border = value,
                    "form_label" => data.form_label = value,
                    "form_label_focused" => data.form_label_focused = value,
                    "form_field_background" => data.form_field_background = value,
                    "form_field_text" => data.form_field_text = value,
                    "form_checkbox_checked" => data.form_checkbox_checked = value,
                    "form_checkbox_unchecked" => data.form_checkbox_unchecked = value,
                    "form_error" => data.form_error = value,
                    "editor_border" => data.editor_border = value,
                    "editor_label" => data.editor_label = value,
                    "editor_label_focused" => data.editor_label_focused = value,
                    "editor_text" => data.editor_text = value,
                    "editor_cursor" => data.editor_cursor = value,
                    "editor_status" => data.editor_status = value,
                    "editor_background" => data.editor_background = value,
                    "menu_border" => data.menu_border = value,
                    "menu_background" => data.menu_background = value,
                    "menu_item_normal" => data.menu_item_normal = value,
                    "menu_item_selected" => data.menu_item_selected = value,
                    "menu_separator" => data.menu_separator = value,
                    "button_normal" => data.button_normal = value,
                    "button_disabled" => data.button_disabled = value,
                    "status_background" => data.status_background = value,
                    _ => {} // Ignore unknown fields
                }
            }
        }

        data
    }

    fn next_field(&mut self) {
        if self.current_section == 0 {
            // Meta section (name, description)
            if self.current_field == 0 {
                self.current_field = 1; // Move from name to description
            } else {
                // Move to first color section
                self.current_section = 1;
                self.current_field = 0;
            }
        } else {
            // Color sections
            let section_idx = self.current_section - 1;
            if let Some(section) = self.color_sections.get(section_idx) {
                self.current_field += 1;
                if self.current_field >= section.fields.len() {
                    // Move to next section
                    self.current_section += 1;
                    self.current_field = 0;

                    // Wrap around if we've gone past the last section
                    if self.current_section > self.color_sections.len() {
                        self.current_section = 0;
                        self.current_field = 0;
                    }
                }
            }
        }
    }

    fn previous_field(&mut self) {
        if self.current_field > 0 {
            self.current_field -= 1;
        } else {
            // Move to previous section
            if self.current_section > 0 {
                self.current_section -= 1;

                if self.current_section == 0 {
                    // Back to meta section
                    self.current_field = 1; // description
                } else {
                    // Back to previous color section
                    let section_idx = self.current_section - 1;
                    if let Some(section) = self.color_sections.get(section_idx) {
                        self.current_field = section.fields.len().saturating_sub(1);
                    }
                }
            } else {
                // Wrap around to last field of last section
                if !self.color_sections.is_empty() {
                    self.current_section = self.color_sections.len();
                    let section_idx = self.current_section - 1;
                    if let Some(section) = self.color_sections.get(section_idx) {
                        self.current_field = section.fields.len().saturating_sub(1);
                    }
                }
            }
        }
    }

    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        config: &crate::config::Config,
        theme: &crate::theme::AppTheme,
    ) {
        let width = 80;
        let height = 30;

        // Center popup
        if self.popup_x == 0 && self.popup_y == 0 {
            self.popup_x = (area.width.saturating_sub(width)) / 2;
            self.popup_y = (area.height.saturating_sub(height)) / 2;
        }

        let x = self.popup_x;
        let y = self.popup_y;

        // Clear area
        for row in y..y + height {
            for col in x..x + width {
                if col < area.width && row < area.height {
                    buf[(col, row)]
                        .set_char(' ')
                        .set_bg(theme.form_field_background);
                }
            }
        }

        // Draw border
        self.draw_border(x, y, width, height, buf, theme);

        // Title
        let title = " Theme Editor ";
        buf.set_string(
            x + 2,
            y,
            title,
            Style::default()
                .fg(theme.form_label)
                .add_modifier(Modifier::BOLD),
        );

        // Instructions
        let footer = "Tab:Next  Shift+Tab:Prev  Ctrl+Enter:Save  Esc:Cancel";
        buf.set_string(
            x + 2,
            y + height - 1,
            footer,
            Style::default().fg(theme.text_disabled),
        );

        // Render fields
        let mut current_y = y + 2;

        // Extract section and field to avoid borrow checker issues
        let current_section = self.current_section;
        let current_field = self.current_field;

        // Name field
        Self::render_text_field(
            "Name:",
            &mut self.name,
            x + 2,
            current_y,
            40,
            buf,
            theme,
            current_section == 0 && current_field == 0,
        );
        current_y += 2;

        // Description field
        Self::render_text_field(
            "Description:",
            &mut self.description,
            x + 2,
            current_y,
            40,
            buf,
            theme,
            current_section == 0 && current_field == 1,
        );
        current_y += 2;

        // Color section display
        if current_section > 0 && current_section <= self.color_sections.len() {
            let section_idx = current_section - 1;
            let section_name = self.color_sections[section_idx].name;

            // Section header
            buf.set_string(
                x + 2,
                current_y,
                &format!("--- {} ---", section_name),
                Style::default()
                    .fg(theme.browser_title)
                    .add_modifier(Modifier::BOLD),
            );
            current_y += 1;

            // Show fields from current section (scroll if needed)
            let max_visible_fields = 10;
            let scroll_offset = if current_field >= max_visible_fields {
                current_field - max_visible_fields + 1
            } else {
                0
            };

            for (i, field) in self.color_sections[section_idx]
                .fields
                .iter()
                .enumerate()
                .skip(scroll_offset)
                .take(max_visible_fields)
            {
                let is_focused = current_section > 0 && current_field == i;

                // Label
                let label_style = if is_focused {
                    Style::default()
                        .fg(theme.form_label_focused)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(theme.form_label)
                };
                buf.set_string(
                    x + 4,
                    current_y,
                    &format!("{:<20}", field.label),
                    label_style,
                );

                // Get value from textarea
                let value = field
                    .textarea
                    .lines()
                    .get(0)
                    .map(|s| s.as_str())
                    .unwrap_or("");

                // Value with focus indicator
                let value_style = if is_focused {
                    Style::default()
                        .fg(theme.form_field_text)
                        .bg(theme.form_field_background)
                } else {
                    Style::default().fg(theme.text_primary)
                };
                buf.set_string(x + 26, current_y, &format!("{:<10}", value), value_style);

                // Color preview box (3 characters wide)
                if let Some(color) = ThemeData::parse_color(value) {
                    for offset in 0..3 {
                        buf[(x + 38 + offset, current_y)]
                            .set_char(' ')
                            .set_bg(color);
                    }
                }

                current_y += 1;
            }
        } else {
            // Show section list when in meta section
            current_y += 1;
            buf.set_string(
                x + 2,
                current_y,
                "Available Sections:",
                Style::default().fg(theme.text_secondary),
            );
            current_y += 1;

            for (i, section) in self.color_sections.iter().enumerate().take(8) {
                buf.set_string(
                    x + 4,
                    current_y,
                    &format!("{}. {}", i + 1, section.name),
                    Style::default().fg(theme.text_primary),
                );
                current_y += 1;
            }
        }
    }

    fn draw_border(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        buf: &mut Buffer,
        theme: &crate::theme::AppTheme,
    ) {
        let border_style = Style::default().fg(theme.form_border);

        // Top
        buf[(x, y)].set_char('┌').set_style(border_style);
        for col in 1..width - 1 {
            buf[(x + col, y)].set_char('─').set_style(border_style);
        }
        buf[(x + width - 1, y)]
            .set_char('┐')
            .set_style(border_style);

        // Sides
        for row in 1..height - 1 {
            buf[(x, y + row)].set_char('│').set_style(border_style);
            buf[(x + width - 1, y + row)]
                .set_char('│')
                .set_style(border_style);
        }

        // Bottom
        buf[(x, y + height - 1)]
            .set_char('└')
            .set_style(border_style);
        for col in 1..width - 1 {
            buf[(x + col, y + height - 1)]
                .set_char('─')
                .set_style(border_style);
        }
        buf[(x + width - 1, y + height - 1)]
            .set_char('┘')
            .set_style(border_style);
    }

    fn render_text_field(
        label: &str,
        textarea: &mut TextArea,
        x: u16,
        y: u16,
        width: u16,
        buf: &mut Buffer,
        theme: &crate::theme::AppTheme,
        focused: bool,
    ) {
        let label_style = if focused {
            Style::default().fg(theme.form_label_focused)
        } else {
            Style::default().fg(theme.form_label)
        };

        buf.set_string(x, y, label, label_style);

        let input_area = Rect {
            x: x + 15,
            y,
            width,
            height: 1,
        };

        textarea.set_style(
            Style::default()
                .fg(theme.form_field_text)
                .bg(theme.form_field_background),
        );
        textarea.set_cursor_style(Style::default().bg(theme.editor_cursor));
        RatatuiWidget::render(&*textarea, input_area, buf);
    }
}
