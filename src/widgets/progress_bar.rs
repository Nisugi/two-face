/// Progress bar state (rendering-agnostic)
///
/// Holds current/max values, colors, and display text for a progress bar.
/// Can be rendered by TUI (ratatui) or GUI (egui) frontends.
///
/// TODO: Migrate full implementation from src/ui/progress_bar.rs
pub struct ProgressBarState {
    /// Current value
    pub current: i32,

    /// Maximum value
    pub max: i32,

    /// Custom display text (e.g., "clear as a bell" for mind state)
    pub text: Option<String>,

    /// Bar fill color (foreground)
    pub bar_color: Option<String>,

    /// Bar background color
    pub bg_color: Option<String>,

    /// Text color overlaid on bar
    pub text_color: Option<String>,

    /// Whether to display as numbers only
    pub numbers_only: bool,
}

impl ProgressBarState {
    /// Create a new progress bar state
    pub fn new() -> Self {
        Self {
            current: 0,
            max: 100,
            text: None,
            bar_color: None,
            bg_color: None,
            text_color: None,
            numbers_only: false,
        }
    }

    /// Update progress values
    pub fn set_progress(&mut self, current: i32, max: i32) {
        self.current = current;
        self.max = max;
        self.text = None; // Clear custom text when numbers are set
    }

    /// Update progress with custom text
    pub fn set_progress_with_text(&mut self, current: i32, max: i32, text: String) {
        self.current = current;
        self.max = max;
        self.text = Some(text);
    }

    /// Set bar colors
    pub fn set_colors(&mut self, bar_color: Option<String>, bg_color: Option<String>) {
        self.bar_color = bar_color;
        self.bg_color = bg_color;
    }

    /// Get progress percentage (0.0 to 1.0)
    pub fn percentage(&self) -> f64 {
        if self.max == 0 {
            0.0
        } else {
            (self.current as f64 / self.max as f64).clamp(0.0, 1.0)
        }
    }

    /// Get display text
    pub fn display_text(&self) -> String {
        if let Some(ref text) = self.text {
            text.clone()
        } else if self.numbers_only {
            format!("{}/{}", self.current, self.max)
        } else {
            format!("{}/{}", self.current, self.max)
        }
    }
}

impl Default for ProgressBarState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar() {
        let mut pb = ProgressBarState::new();
        pb.set_progress(50, 100);
        assert_eq!(pb.percentage(), 0.5);
        assert_eq!(pb.display_text(), "50/100");

        pb.set_progress_with_text(75, 100, "Three quarters".to_string());
        assert_eq!(pb.percentage(), 0.75);
        assert_eq!(pb.display_text(), "Three quarters");
    }
}
