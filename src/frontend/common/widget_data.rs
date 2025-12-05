//! Frontend-agnostic widget data structures.
//!
//! These structs contain only the calculated/prepared data needed to render widgets,
//! with no rendering logic or frontend-specific types (like ratatui's Buffer/Rect).
//!
//! The data preparation logic is shared across all frontends (TUI, GUI),
//! while each frontend has its own renderer that consumes this data.

/// Border configuration for widgets
#[derive(Debug, Clone, PartialEq)]
pub struct BorderConfig {
    pub show_border: bool,
    pub border_style: Option<String>,
    pub border_color: Option<String>,
}

impl Default for BorderConfig {
    fn default() -> Self {
        Self {
            show_border: false,
            border_style: None,
            border_color: None,
        }
    }
}

/// Color configuration (hex strings like "#FF0000")
#[derive(Debug, Clone, PartialEq)]
pub struct ColorConfig {
    pub foreground: Option<String>,
    pub background: Option<String>,
    pub transparent_background: bool,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            foreground: None,
            background: None,
            transparent_background: true,
        }
    }
}

/// Data for a progress bar widget
#[derive(Debug, Clone)]
pub struct ProgressBarData {
    pub label: String,
    pub current: u32,
    pub max: u32,
    pub custom_text: Option<String>,
    pub percentage: u32,
    pub display_text: String,
    pub border: BorderConfig,
    pub bar_fill_color: Option<String>,
    pub bar_background_color: Option<String>,
    pub text_color: Option<String>,
    pub window_background_color: Option<String>,
    pub transparent_background: bool,
}

impl ProgressBarData {
    /// Create progress bar data from raw values
    pub fn new(label: String, current: u32, max: u32, custom_text: Option<String>) -> Self {
        let percentage = if max > 0 {
            ((current as f64 / max as f64) * 100.0) as u32
        } else {
            0
        };

        let display_text = custom_text
            .clone()
            .unwrap_or_else(|| format!("{}/{}", current, max));

        Self {
            label,
            current,
            max,
            custom_text,
            percentage,
            display_text,
            border: BorderConfig::default(),
            bar_fill_color: Some("#00ff00".to_string()), // Green default
            bar_background_color: None,
            text_color: Some("#ffffff".to_string()), // White default
            window_background_color: None,
            transparent_background: true,
        }
    }

    pub fn with_border(mut self, border: BorderConfig) -> Self {
        self.border = border;
        self
    }

    pub fn with_colors(
        mut self,
        bar_fill: Option<String>,
        bar_background: Option<String>,
    ) -> Self {
        self.bar_fill_color = bar_fill;
        self.bar_background_color = bar_background;
        self
    }
}

/// Data for an indicator widget (boolean on/off display)
#[derive(Debug, Clone)]
pub struct IndicatorData {
    pub label: String,
    pub active: bool,
    pub border: BorderConfig,
    pub off_color: String,
    pub on_color: String,
    pub background_color: Option<String>,
    pub transparent_background: bool,
}

impl IndicatorData {
    pub fn new(label: String, active: bool) -> Self {
        Self {
            label,
            active,
            border: BorderConfig::default(),
            off_color: "#555555".to_string(), // Dark gray
            on_color: "#00ff00".to_string(),   // Green
            background_color: None,
            transparent_background: true,
        }
    }

    pub fn with_colors(mut self, off_color: String, on_color: String) -> Self {
        self.off_color = off_color;
        self.on_color = on_color;
        self
    }
}

/// Data for a countdown timer widget
#[derive(Debug, Clone)]
pub struct CountdownData {
    pub label: String,
    pub remaining_seconds: u32,
    pub display_text: String,
    pub blocks_to_show: u32,
    pub icon: char,
    pub border: BorderConfig,
    pub text_color: Option<String>,
    pub background_color: Option<String>,
    pub transparent_background: bool,
}

impl CountdownData {
    /// Create countdown data from remaining seconds and available width
    pub fn new(
        label: String,
        remaining_seconds: u32,
        available_width: u16,
        icon: char,
    ) -> Self {
        // Right-align the number so it doesn't shift when going from 10->9
        // Format: " 9 " or "10 " (always 3 chars)
        let display_text = format!("{:>2} ", remaining_seconds);
        let text_width = 3u16; // Always 3 chars

        // Calculate max blocks based on available space after the number
        let max_blocks = if available_width > text_width {
            (available_width - text_width) as u32
        } else {
            0
        };
        let blocks_to_show = remaining_seconds.min(max_blocks);

        Self {
            label,
            remaining_seconds,
            display_text,
            blocks_to_show,
            icon,
            border: BorderConfig::default(),
            text_color: Some("#ffffff".to_string()), // White default
            background_color: None,
            transparent_background: true,
        }
    }

    pub fn with_border(mut self, border: BorderConfig) -> Self {
        self.border = border;
        self
    }
}

/// Hand type for hand widget
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandType {
    Left,
    Right,
    Spell,
}

/// Data for a hand widget (displays item in left/right/spell hand)
#[derive(Debug, Clone)]
pub struct HandData {
    pub label: String,
    pub hand_type: HandType,
    pub content: String,
    pub icon: String,
    pub border: BorderConfig,
    pub border_sides: crate::config::BorderSides,
    pub text_color: Option<String>,
    pub content_highlight_color: Option<String>,
    pub background_color: Option<String>,
    pub transparent_background: bool,
}

impl HandData {
    pub fn new(label: String, hand_type: HandType, content: String) -> Self {
        let default_icon = match hand_type {
            HandType::Left => "L:",
            HandType::Right => "R:",
            HandType::Spell => "S:",
        };

        // Truncate content to 24 characters
        let truncated_content = if content.chars().count() > 24 {
            content.chars().take(24).collect()
        } else {
            content
        };

        Self {
            label,
            hand_type,
            content: truncated_content,
            icon: default_icon.to_string(),
            border: BorderConfig::default(),
            border_sides: crate::config::BorderSides::default(),
            text_color: None,
            content_highlight_color: None,
            background_color: None,
            transparent_background: true,
        }
    }

    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = icon;
        self
    }

    pub fn with_colors(
        mut self,
        text_color: Option<String>,
        content_highlight: Option<String>,
    ) -> Self {
        self.text_color = text_color;
        self.content_highlight_color = content_highlight;
        self
    }
}
