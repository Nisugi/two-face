# Adding Widgets

Guide to creating new widget types for Two-Face.

## Overview

Widgets are the visual building blocks of Two-Face. Each widget type:

- Displays specific data
- Has its own rendering logic
- Responds to state changes
- May handle user input

## Widget Architecture

### Widget Lifecycle

```
Configuration → Creation → State Sync → Render → Input (optional)
     │             │            │          │           │
     │             │            │          │           └─ Handle focus/keys
     │             │            │          └─ Draw to terminal frame
     │             │            └─ Check generation, update data
     │             └─ Instantiate from config
     └─ TOML defines type and properties
```

### Key Traits

Widgets implement several traits:

```rust
// Core widget behavior
pub trait Widget {
    fn render(&self, frame: &mut Frame, area: Rect);
    fn name(&self) -> &str;
    fn can_focus(&self) -> bool;
}

// State synchronization
pub trait Syncable {
    fn sync(&mut self, state: &AppState) -> bool;
    fn last_generation(&self) -> u64;
}

// Input handling (optional)
pub trait Interactive {
    fn handle_key(&mut self, key: KeyEvent) -> bool;
    fn handle_mouse(&mut self, mouse: MouseEvent) -> bool;
}
```

## Step-by-Step: New Widget

Let's create a "SpellTimer" widget that displays active spell durations.

### Step 1: Define the Widget Type

Add to `src/data/widget.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WidgetType {
    Text,
    Progress,
    Compass,
    // ... existing types ...
    SpellTimer,  // Add new type
}
```

### Step 2: Create Widget Configuration

```rust
// In src/data/widget.rs or dedicated file

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellTimerConfig {
    pub name: String,
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,

    // Widget-specific options
    pub show_duration: bool,
    pub group_by_circle: bool,
    pub max_display: usize,
}

impl Default for SpellTimerConfig {
    fn default() -> Self {
        Self {
            name: "spell_timer".into(),
            x: 0,
            y: 0,
            width: 20,
            height: 10,
            show_duration: true,
            group_by_circle: false,
            max_display: 10,
        }
    }
}
```

### Step 3: Create Widget Structure

Create `src/frontend/tui/spell_timer.rs`:

```rust
use crate::core::AppState;
use crate::data::SpellTimerConfig;
use ratatui::prelude::*;
use ratatui::widgets::*;

pub struct SpellTimerWidget {
    config: SpellTimerConfig,
    spells: Vec<ActiveSpell>,
    last_generation: u64,
}

#[derive(Clone)]
struct ActiveSpell {
    name: String,
    duration: u32,
    circle: u8,
}

impl SpellTimerWidget {
    pub fn new(config: SpellTimerConfig) -> Self {
        Self {
            config,
            spells: Vec::new(),
            last_generation: 0,
        }
    }
}
```

### Step 4: Implement Widget Trait

```rust
impl Widget for SpellTimerWidget {
    fn render(&self, frame: &mut Frame, area: Rect) {
        // Create border
        let block = Block::default()
            .title("Spells")
            .borders(Borders::ALL);

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Render spell list
        let items: Vec<ListItem> = self.spells
            .iter()
            .take(self.config.max_display)
            .map(|spell| {
                let text = if self.config.show_duration {
                    format!("{}: {}s", spell.name, spell.duration)
                } else {
                    spell.name.clone()
                };
                ListItem::new(text)
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, inner);
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    fn can_focus(&self) -> bool {
        false  // This widget doesn't need focus
    }
}
```

### Step 5: Implement Syncable Trait

```rust
impl Syncable for SpellTimerWidget {
    fn sync(&mut self, state: &AppState) -> bool {
        // Check if state has changed
        if state.generation() == self.last_generation {
            return false;  // No changes
        }

        // Update spell list from state
        self.spells = state.active_spells()
            .iter()
            .map(|s| ActiveSpell {
                name: s.name.clone(),
                duration: s.remaining_seconds,
                circle: s.circle,
            })
            .collect();

        // Apply grouping if configured
        if self.config.group_by_circle {
            self.spells.sort_by_key(|s| s.circle);
        }

        self.last_generation = state.generation();
        true  // Widget needs redraw
    }

    fn last_generation(&self) -> u64 {
        self.last_generation
    }
}
```

### Step 6: Register Widget Type

In the widget factory (typically `src/frontend/tui/mod.rs` or dedicated factory):

```rust
pub fn create_widget(config: &WidgetConfig) -> Box<dyn Widget> {
    match config.widget_type {
        WidgetType::Text => Box::new(TextWidget::new(config.into())),
        WidgetType::Progress => Box::new(ProgressWidget::new(config.into())),
        // ... existing types ...
        WidgetType::SpellTimer => Box::new(SpellTimerWidget::new(config.into())),
    }
}
```

### Step 7: Add to Module

In `src/frontend/tui/mod.rs`:

```rust
mod spell_timer;
pub use spell_timer::SpellTimerWidget;
```

### Step 8: Document Configuration

Update TOML documentation:

```toml
# Example spell timer configuration
[[widgets]]
type = "spell_timer"
name = "spells"
x = 80
y = 0
width = 20
height = 15
show_duration = true
group_by_circle = true
max_display = 10
```

## Widget Best Practices

### State Management

```rust
// GOOD: Check generation before updating
fn sync(&mut self, state: &AppState) -> bool {
    if state.generation() == self.last_generation {
        return false;
    }
    // ... update ...
    self.last_generation = state.generation();
    true
}

// BAD: Always update (wasteful)
fn sync(&mut self, state: &AppState) -> bool {
    // ... update ...
    true  // Always redraws
}
```

### Efficient Rendering

```rust
// GOOD: Cache computed values
struct MyWidget {
    cached_lines: Vec<Line<'static>>,
    // ...
}

fn sync(&mut self, state: &AppState) -> bool {
    // Rebuild cache only when data changes
    self.cached_lines = self.compute_lines(state);
    true
}

fn render(&self, frame: &mut Frame, area: Rect) {
    // Use cached data
    let paragraph = Paragraph::new(self.cached_lines.clone());
    frame.render_widget(paragraph, area);
}
```

### Configuration Validation

```rust
impl SpellTimerConfig {
    pub fn validate(&self) -> Result<(), String> {
        if self.width == 0 || self.height == 0 {
            return Err("Widget must have non-zero dimensions".into());
        }
        if self.max_display == 0 {
            return Err("max_display must be at least 1".into());
        }
        Ok(())
    }
}
```

### Error Handling

```rust
fn render(&self, frame: &mut Frame, area: Rect) {
    // Handle edge cases gracefully
    if area.width < 3 || area.height < 3 {
        // Area too small to render
        return;
    }

    if self.spells.is_empty() {
        // Show placeholder
        let text = Paragraph::new("No active spells");
        frame.render_widget(text, area);
        return;
    }

    // Normal rendering...
}
```

## Testing Widgets

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_widget_creation() {
        let config = SpellTimerConfig::default();
        let widget = SpellTimerWidget::new(config);
        assert_eq!(widget.name(), "spell_timer");
    }

    #[test]
    fn test_sync_updates_generation() {
        let mut widget = SpellTimerWidget::new(Default::default());
        let mut state = AppState::new();

        state.set_generation(1);
        assert!(widget.sync(&state));
        assert_eq!(widget.last_generation(), 1);

        // Same generation shouldn't trigger update
        assert!(!widget.sync(&state));
    }
}
```

### Visual Testing

Create a test layout that isolates your widget:

```toml
# test_layout.toml
[[widgets]]
type = "spell_timer"
name = "test"
x = 0
y = 0
width = 100
height = 100
```

## See Also

- [Widget Reference](../widgets/README.md) - Existing widgets
- [Project Structure](./project-structure.md) - Code organization
- [Architecture](../architecture/widget-sync.md) - Sync system

