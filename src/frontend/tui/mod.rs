//! TUI Frontend (ratatui-based)
//!
//! This module implements the Frontend trait using ratatui for terminal rendering.
//! It wraps crossterm for event handling and terminal management.

pub mod app;
pub mod widgets;

pub use app::TuiFrontend;
