use crate::frontend::{Frontend, FrontendEvent};
use crate::core::AppCore;
use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyEventKind, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;
use std::time::Duration;

/// TUI Frontend using ratatui
///
/// This frontend renders the application using ratatui (terminal UI library)
/// and handles events via crossterm.
pub struct TuiFrontend {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    poll_timeout: Duration,
}

impl TuiFrontend {
    /// Create a new TUI frontend
    ///
    /// Initializes terminal in raw mode, enables mouse capture, and enters alternate screen.
    pub fn new() -> Result<Self> {
        // Setup terminal
        enable_raw_mode().context("Failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .context("Failed to setup terminal")?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;
        terminal.hide_cursor()?;

        Ok(Self {
            terminal,
            poll_timeout: Duration::from_millis(16), // ~60 FPS
        })
    }

    /// Set poll timeout (for controlling frame rate)
    pub fn set_poll_timeout(&mut self, timeout: Duration) {
        self.poll_timeout = timeout;
    }

    /// Convert crossterm event to FrontendEvent
    fn convert_event(event: Event) -> Option<FrontendEvent> {
        match event {
            Event::Key(key_event) => {
                // Only process key press events (ignore repeats and releases for now)
                if key_event.kind != KeyEventKind::Press {
                    return None;
                }
                Some(FrontendEvent::Key {
                    code: key_event.code,
                    modifiers: key_event.modifiers,
                })
            }
            Event::Mouse(mouse_event) => {
                Some(FrontendEvent::Mouse {
                    kind: mouse_event.kind,
                    x: mouse_event.column,
                    y: mouse_event.row,
                    modifiers: mouse_event.modifiers,
                })
            }
            Event::Resize(w, h) => {
                Some(FrontendEvent::Resize {
                    width: w,
                    height: h,
                })
            }
            Event::Paste(text) => {
                Some(FrontendEvent::Paste { text })
            }
            _ => None,
        }
    }
}

impl Frontend for TuiFrontend {
    fn poll_events(&mut self) -> Result<Vec<FrontendEvent>> {
        let mut events = Vec::new();

        // Poll events with timeout
        while event::poll(self.poll_timeout)? {
            if let Ok(ev) = event::read() {
                if let Some(frontend_event) = Self::convert_event(ev) {
                    events.push(frontend_event);
                }
            }
        }

        Ok(events)
    }

    fn render(&mut self, core: &mut dyn std::any::Any) -> Result<()> {
        use crate::core::AppCore;
        use ratatui::layout::Rect;

        // Downcast to mutable AppCore
        let core = core
            .downcast_mut::<AppCore>()
            .expect("render() called with wrong type - expected AppCore");

        self.terminal.draw(|f| {
            // Calculate window layouts
            let terminal_area = f.area();

            // For now, use all available space for windows (no command input in experimental mode yet)
            let window_layouts = core.window_manager.calculate_layout(terminal_area);

            // Render all windows in order
            let window_names = core.window_manager.get_window_names();

            for (idx, name) in window_names.iter().enumerate() {
                if let Some(rect) = window_layouts.get(name) {
                    // Skip windows that are completely out of bounds
                    if rect.y >= terminal_area.height || rect.x >= terminal_area.width {
                        continue;
                    }

                    // Clip windows that extend beyond terminal bounds
                    let clipped_rect = if rect.y + rect.height > terminal_area.height
                        || rect.x + rect.width > terminal_area.width {
                        let clipped_height = rect.height.min(terminal_area.height.saturating_sub(rect.y));
                        let clipped_width = rect.width.min(terminal_area.width.saturating_sub(rect.x));

                        if clipped_height > 0 && clipped_width > 0 {
                            Rect::new(rect.x, rect.y, clipped_width, clipped_height)
                        } else {
                            continue; // Skip if clipped to zero size
                        }
                    } else {
                        *rect
                    };

                    // Render the window (no focus indicator, no selection for now)
                    if let Some(window) = core.window_manager.get_window(name) {
                        window.render_with_focus(
                            clipped_rect,
                            f.buffer_mut(),
                            false, // No focus highlighting in experimental mode yet
                            core.server_time_offset,
                            None,  // No selection state yet
                            "#ffffff", // Placeholder selection color
                            idx,
                        );
                    }
                }
            }

            // Render simple command input at the bottom
            // Calculate position: bottom of terminal, full width
            if terminal_area.height > 0 {
                use ratatui::widgets::{Block, Borders, Paragraph};
                use ratatui::style::{Style, Color as RatColor};
                use ratatui::text::{Line, Span};

                let cmd_area = Rect::new(
                    0,
                    terminal_area.height.saturating_sub(1),
                    terminal_area.width,
                    1,
                );

                // Build command line with cursor
                let mut spans = vec![];

                // Add prompt
                spans.push(Span::styled("> ", Style::default().fg(RatColor::Green)));

                // Add text before cursor
                if core.command_cursor > 0 {
                    spans.push(Span::raw(&core.command_input[..core.command_cursor]));
                }

                // Add cursor
                let cursor_char = if core.command_cursor < core.command_input.len() {
                    core.command_input.chars().nth(core.command_cursor).unwrap().to_string()
                } else {
                    " ".to_string()
                };
                spans.push(Span::styled(
                    cursor_char,
                    Style::default().fg(RatColor::Black).bg(RatColor::White)
                ));

                // Add text after cursor
                if core.command_cursor + 1 < core.command_input.len() {
                    spans.push(Span::raw(&core.command_input[core.command_cursor + 1..]));
                }

                let line = Line::from(spans);
                let paragraph = Paragraph::new(line);

                f.render_widget(paragraph, cmd_area);
            }
        })?;

        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    fn size(&self) -> (u16, u16) {
        let size = self.terminal.size().unwrap_or_default();
        (size.width, size.height)
    }
}

impl Drop for TuiFrontend {
    fn drop(&mut self) {
        // Ensure terminal is restored even if cleanup() wasn't called
        let _ = self.cleanup();
    }
}
