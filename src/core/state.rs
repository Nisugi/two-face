//! Game state management
//!
//! Tracks the current state of the game session: connection status,
//! character info, room state, inventory, etc.

use std::collections::HashMap;

/// Game session state
#[derive(Clone, Debug)]
pub struct GameState {
    /// Connection status
    pub connected: bool,

    /// Character name
    pub character_name: Option<String>,

    /// Current room ID
    pub room_id: Option<String>,

    /// Current room name
    pub room_name: Option<String>,

    /// Available exits from current room
    pub exits: Vec<String>,

    /// Roundtime end timestamp (Unix time)
    pub roundtime_end: Option<i64>,

    /// Casttime end timestamp (Unix time)
    pub casttime_end: Option<i64>,

    /// Current spell being prepared
    pub spell: Option<String>,

    /// Active game streams (tags like "inv", "assess", etc.)
    pub active_streams: HashMap<String, bool>,

    /// Player status indicators
    pub status: StatusInfo,

    /// Vitals (health, mana, etc.)
    pub vitals: Vitals,

    /// Inventory items
    pub inventory: Vec<String>,

    /// Current left hand item
    pub left_hand: Option<String>,

    /// Current right hand item
    pub right_hand: Option<String>,

    /// Active effects/buffs
    pub active_effects: Vec<String>,

    /// Compass directions
    pub compass_dirs: Vec<String>,

    /// Last prompt text (for command echoes)
    pub last_prompt: String,
}

/// Player status information
#[derive(Clone, Debug, Default)]
pub struct StatusInfo {
    pub standing: bool,
    pub kneeling: bool,
    pub sitting: bool,
    pub prone: bool,
    pub stunned: bool,
    pub bleeding: bool,
    pub hidden: bool,
    pub invisible: bool,
    pub webbed: bool,
    pub joined: bool,
    pub dead: bool,
}

/// Player vitals
#[derive(Clone, Debug)]
pub struct Vitals {
    pub health: u8,
    pub mana: u8,
    pub stamina: u8,
    pub spirit: u8,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            connected: false,
            character_name: None,
            room_id: None,
            room_name: None,
            exits: Vec::new(),
            roundtime_end: None,
            casttime_end: None,
            spell: None,
            active_streams: HashMap::new(),
            status: StatusInfo::default(),
            vitals: Vitals::default(),
            inventory: Vec::new(),
            left_hand: None,
            right_hand: None,
            active_effects: Vec::new(),
            compass_dirs: Vec::new(),
            last_prompt: String::from(">"), // Default prompt
        }
    }

    /// Check if currently in roundtime
    pub fn in_roundtime(&self) -> bool {
        if let Some(end_time) = self.roundtime_end {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            now < end_time
        } else {
            false
        }
    }

    /// Check if currently in casttime
    pub fn in_casttime(&self) -> bool {
        if let Some(end_time) = self.casttime_end {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            now < end_time
        } else {
            false
        }
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Vitals {
    fn default() -> Self {
        Self {
            health: 100,
            mana: 100,
            stamina: 100,
            spirit: 100,
        }
    }
}
