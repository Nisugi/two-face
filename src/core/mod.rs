//! Core application logic (frontend-agnostic)
//!
//! This module contains the core business logic that is shared between
//! all frontends (TUI, GUI). It handles game state, configuration,
//! XML parsing, stream routing, and other application logic that is
//! independent of the rendering layer.

pub mod app_core;

pub use app_core::AppCore;
