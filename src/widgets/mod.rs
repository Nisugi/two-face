//! Widget state structs (rendering-agnostic)
//!
//! This module contains state-only structures for all widget types.
//! These structs hold data and provide methods for state manipulation,
//! but contain no rendering logic. Both TUI (ratatui) and GUI (egui)
//! frontends can render these states using their respective frameworks.

pub mod text_window;
pub mod progress_bar;
pub mod countdown;

pub use text_window::TextWindowState;
pub use progress_bar::ProgressBarState;
pub use countdown::CountdownState;

// TODO: Add remaining widget states:
// - TabbedTextWindowState
// - CompassState
// - IndicatorState
// - InjuryDollState
// - HandsState
// - DashboardState
// - ActiveEffectsState
// - InventoryWindowState
// - RoomWindowState
// - MapWidgetState
// - SpellsWindowState
