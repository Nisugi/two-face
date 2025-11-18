//! Widget data structures - State for all widget types
//!
//! These are pure data structures with NO rendering logic.
//! Frontends read from these to render appropriately.

use std::collections::VecDeque;

/// Styled text content for text-based widgets
#[derive(Clone, Debug)]
pub struct TextContent {
    /// Wrapped lines ready for display
    pub lines: VecDeque<StyledLine>,
    /// Scroll offset from bottom (0 = live view, showing newest)
    pub scroll_offset: usize,
    /// Maximum lines to keep in buffer
    pub max_lines: usize,
    /// Title for the window
    pub title: String,
    /// Generation counter - increments on every add_line call
    /// Used to detect changes even when line count stays constant (at max_lines)
    pub generation: u64,
}

/// A single display line with styled segments
#[derive(Clone, Debug)]
pub struct StyledLine {
    pub segments: Vec<TextSegment>,
}

/// A segment of text with styling
#[derive(Clone, Debug, PartialEq)]
pub struct TextSegment {
    pub text: String,
    pub fg: Option<String>, // Hex color "#RRGGBB"
    pub bg: Option<String>, // Hex color "#RRGGBB"
    pub bold: bool,
    pub span_type: SpanType, // Semantic type for priority layering
    pub link_data: Option<LinkData>,
}

/// Semantic type of text span (for highlight priority)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanType {
    Normal,      // Regular text
    Link,        // <a> tag from parser (clickable game objects)
    Monsterbold, // <preset id="monsterbold"> from parser (monsters)
    Spell,       // <spell> tag from parser (spells)
    Speech,      // <preset id="speech"> from parser (player speech)
}

/// Link metadata for clickable text
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LinkData {
    pub exist_id: String,
    pub noun: String,
    pub text: String,
    pub coord: Option<String>, // Optional coord for direct commands (e.g., "2524,1864" for movement)
}

/// Progress bar state
#[derive(Clone, Debug)]
pub struct ProgressData {
    pub value: u32,            // Current value (actual value, not percentage)
    pub max: u32,              // Maximum value (actual max, not percentage)
    pub label: String,         // Display label
    pub color: Option<String>, // Hex color override (or custom text like "clear as a bell")
}

/// Countdown timer state
#[derive(Clone, Debug)]
pub struct CountdownData {
    pub end_time: i64, // Unix timestamp when timer expires
    pub label: String, // Display label
}

/// Compass directions
#[derive(Clone, Debug)]
pub struct CompassData {
    pub directions: Vec<String>, // Available exits: "n", "s", "e", "w", etc.
}

/// Injury doll state
#[derive(Clone, Debug)]
pub struct InjuryDollData {
    pub injuries: std::collections::HashMap<String, u8>, // body_part -> level (0-6)
                                                         // Injury levels: 0=none, 1-3=injury levels, 4-6=scar levels
}

impl InjuryDollData {
    pub fn new() -> Self {
        Self {
            injuries: std::collections::HashMap::new(),
        }
    }

    pub fn set_injury(&mut self, body_part: String, level: u8) {
        self.injuries.insert(body_part, level.min(6));
    }

    pub fn get_injury(&self, body_part: &str) -> u8 {
        self.injuries.get(body_part).copied().unwrap_or(0)
    }

    pub fn clear_all(&mut self) {
        self.injuries.clear();
    }
}

/// Status indicator state
#[derive(Clone, Debug)]
pub struct IndicatorData {
    pub status: String,        // "standing", "kneeling", "sitting", etc.
    pub color: Option<String>, // Color for this status
}

/// Room description content
#[derive(Clone, Debug)]
pub struct RoomContent {
    pub name: String,
    pub description: Vec<StyledLine>,
    pub exits: Vec<String>,
    pub players: Vec<String>,
    pub objects: Vec<String>,
}

/// Active effect (buff/debuff/cooldown/active spell)
#[derive(Clone, Debug)]
pub struct ActiveEffect {
    pub id: String,   // Unique identifier
    pub text: String, // Display text (e.g., "Fasthr's Reward")
    pub value: u32,   // Progress/percentage (0-100)
    pub time: String, // Time remaining (e.g., "03:06:54")
    pub bar_color: Option<String>,
    pub text_color: Option<String>,
}

/// Active effects content (for buffs, debuffs, cooldowns, active spells)
#[derive(Clone, Debug)]
pub struct ActiveEffectsContent {
    pub category: String, // "Buffs", "Debuffs", "Cooldowns", "ActiveSpells"
    pub effects: Vec<ActiveEffect>,
}

/// Tab definition for tabbed text window
#[derive(Clone, Debug)]
pub struct TabDefinition {
    pub name: String,   // Display name of tab
    pub stream: String, // Stream ID this tab listens to
}

/// Tabbed text window content
#[derive(Clone, Debug)]
pub struct TabbedTextContent {
    pub tabs: Vec<TabDefinition>,
    pub active_tab_index: usize,
    pub max_lines_per_tab: usize,
}

impl TabbedTextContent {
    pub fn new(tabs: Vec<(String, String)>, max_lines_per_tab: usize) -> Self {
        let tabs = tabs
            .into_iter()
            .map(|(name, stream)| TabDefinition { name, stream })
            .collect();
        Self {
            tabs,
            active_tab_index: 0,
            max_lines_per_tab,
        }
    }
}

impl TextContent {
    pub fn new(title: impl Into<String>, max_lines: usize) -> Self {
        Self {
            lines: VecDeque::with_capacity(max_lines),
            scroll_offset: 0,
            max_lines,
            title: title.into(),
            generation: 0,
        }
    }

    pub fn add_line(&mut self, line: StyledLine) {
        self.lines.push_back(line);
        if self.lines.len() > self.max_lines {
            self.lines.pop_front();
        }
        // Increment generation counter on every add_line call
        // This allows frontend to detect changes even when line count stays constant
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn scroll_up(&mut self, amount: usize) {
        let max_scroll = self.lines.len().saturating_sub(1);
        self.scroll_offset = (self.scroll_offset + amount).min(max_scroll);
    }

    pub fn scroll_down(&mut self, amount: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(amount);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }
}

impl StyledLine {
    pub fn from_text(text: impl Into<String>) -> Self {
        Self {
            segments: vec![TextSegment {
                text: text.into(),
                fg: None,
                bg: None,
                bold: false,
                span_type: SpanType::Normal,
                link_data: None,
            }],
        }
    }
}
