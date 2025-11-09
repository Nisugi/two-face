use std::time::{SystemTime, UNIX_EPOCH};

/// Countdown timer type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountdownType {
    Roundtime,  // Combat roundtime (red)
    Casttime,   // Spell casting (blue)
    Stun,       // Stun recovery (yellow)
}

/// Countdown timer state (rendering-agnostic)
///
/// Holds end time and type for countdown timers.
/// Can be rendered by TUI (ratatui) or GUI (egui) frontends.
///
/// TODO: Migrate full implementation from src/ui/countdown.rs
pub struct CountdownState {
    /// Unix timestamp when countdown ends
    pub end_time: i64,

    /// Type of countdown (affects color)
    pub countdown_type: CountdownType,

    /// Server time offset for accurate countdowns
    pub server_time_offset: i64,
}

impl CountdownState {
    /// Create a new countdown state
    pub fn new(countdown_type: CountdownType) -> Self {
        Self {
            end_time: 0,
            countdown_type,
            server_time_offset: 0,
        }
    }

    /// Set countdown end time (Unix timestamp)
    pub fn set_countdown(&mut self, end_time: i64) {
        self.end_time = end_time;
    }

    /// Set countdown duration in seconds from now
    pub fn set_duration(&mut self, seconds: i64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.end_time = now + self.server_time_offset + seconds;
    }

    /// Get remaining seconds
    pub fn remaining_seconds(&self) -> i64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let adjusted_now = now + self.server_time_offset;
        (self.end_time - adjusted_now).max(0)
    }

    /// Check if countdown is active
    pub fn is_active(&self) -> bool {
        self.remaining_seconds() > 0
    }

    /// Clear countdown
    pub fn clear(&mut self) {
        self.end_time = 0;
    }

    /// Get color for this countdown type
    pub fn get_color(&self) -> &'static str {
        match self.countdown_type {
            CountdownType::Roundtime => "#ff0000", // Red
            CountdownType::Casttime => "#0000ff",  // Blue
            CountdownType::Stun => "#ffff00",      // Yellow
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_countdown() {
        let mut cd = CountdownState::new(CountdownType::Roundtime);
        assert!(!cd.is_active());

        cd.set_duration(5);
        assert!(cd.is_active());
        assert!(cd.remaining_seconds() >= 4); // Allow for timing variance

        cd.clear();
        assert!(!cd.is_active());
    }

    #[test]
    fn test_countdown_colors() {
        assert_eq!(CountdownState::new(CountdownType::Roundtime).get_color(), "#ff0000");
        assert_eq!(CountdownState::new(CountdownType::Casttime).get_color(), "#0000ff");
        assert_eq!(CountdownState::new(CountdownType::Stun).get_color(), "#ffff00");
    }
}
