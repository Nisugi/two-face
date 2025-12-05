//! Frontend-agnostic rectangular region type.
//!
//! This module provides a simple Rect structure that represents a rectangular
//! area with position (x, y) and dimensions (width, height). It's used across
//! both TUI and GUI frontends for layout calculations and bounds checking.

/// A rectangular region with position and dimensions.
///
/// Used for layout calculations, bounds checking, and coordinate transformations
/// across different frontend implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    /// X coordinate (column) of the top-left corner
    pub x: u16,
    /// Y coordinate (row) of the top-left corner
    pub y: u16,
    /// Width of the rectangle
    pub width: u16,
    /// Height of the rectangle
    pub height: u16,
}

impl Rect {
    /// Create a new rectangle with the given position and dimensions
    pub fn new(x: u16, y: u16, width: u16, height: u16) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Check if a point (x, y) is inside this rectangle
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x
            && x < self.x + self.width
            && y >= self.y
            && y < self.y + self.height
    }

    /// Get the right edge coordinate (exclusive)
    pub fn right(&self) -> u16 {
        self.x + self.width
    }

    /// Get the bottom edge coordinate (exclusive)
    pub fn bottom(&self) -> u16 {
        self.y + self.height
    }
}

impl Default for Rect {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
        }
    }
}
