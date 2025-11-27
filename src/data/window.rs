//! Window state - Layout and content management
//!
//! Windows are the containers for widgets. They have position, size, and content.

use super::widget::*;

/// Window state - combines layout position with content
#[derive(Clone, Debug)]
pub struct WindowState {
    pub name: String,
    pub widget_type: WidgetType,
    pub content: WindowContent,
    pub position: WindowPosition,
    pub visible: bool,
    pub focused: bool,
}

/// Types of widgets that can be displayed
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WidgetType {
    Text,
    TabbedText,
    Progress,
    Countdown,
    Compass,
    Indicator,
    Room,
    Inventory,
    CommandInput,
    Dashboard,
    InjuryDoll,
    Hand,
    ActiveEffects,
    Targets,
    Players,
    Map,
    Spells,
    Spacer,
    QuickBar,
}

// helper maybe not needed currently

/// Window content - what the window displays
#[derive(Clone, Debug)]
pub enum WindowContent {
    Text(TextContent),
    TabbedText(TabbedTextContent),
    Progress(ProgressData),
    Countdown(CountdownData),
    Compass(CompassData),
    InjuryDoll(InjuryDollData),
    Indicator(IndicatorData),
    Room(RoomContent),
    Inventory(TextContent),
    CommandInput {
        text: String,
        cursor: usize,
        history: Vec<String>,
        history_index: Option<usize>,
    },
    Hand {
        item: Option<String>,
        link: Option<LinkData>,
    },
    Spells(TextContent), // Spells window - similar to Inventory but with link caching
    ActiveEffects(ActiveEffectsContent), // Active effects (buffs, debuffs, cooldowns, active spells)
    Targets {
        targets_text: String, // Raw text from game (XML formatted)
    },
    Players {
        players_text: String, // Raw text from game (XML formatted)
    },
    Dashboard {
        indicators: Vec<(String, u8)>, // (id, value) pairs
    },
    QuickBar {
        content: String, // Raw content for currently active bar
    },
    Empty, // For spacers or not-yet-implemented widgets
}

/// Window position and size
#[derive(Clone, Debug)]
pub struct WindowPosition {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

impl WindowState {
    pub fn new_text(name: impl Into<String>, max_lines: usize) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            widget_type: WidgetType::Text,
            content: WindowContent::Text(TextContent::new(name, max_lines)),
            position: WindowPosition {
                x: 0,
                y: 0,
                width: 80,
                height: 24,
            },
            visible: true,
            focused: false,
        }
    }

    pub fn new_command_input(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            widget_type: WidgetType::CommandInput,
            content: WindowContent::CommandInput {
                text: String::new(),
                cursor: 0,
                history: Vec::new(),
                history_index: None,
            },
            position: WindowPosition {
                x: 0,
                y: 23,
                width: 80,
                height: 1,
            },
            visible: true,
            focused: false,
        }
    }
}
