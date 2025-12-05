# Text Input Abstraction

## Overview

The text input abstraction separates text editing logic from frontend-specific implementations, enabling code reuse across TUI and future GUI frontends.

## Architecture

```
┌─────────────────────────────────────┐
│  Frontend-Agnostic Layer            │
│  (frontend::common::text_input)     │
├─────────────────────────────────────┤
│  - TextInput trait                  │
│  - TextEditor trait                 │
│  - CursorMove enum                  │
│  - handle_text_input_key()          │
└──────────────┬──────────────────────┘
               │
        ┌──────┴──────┐
        │             │
┌───────▼───────┐ ┌──▼─────────────┐
│  TUI Adapter  │ │  GUI Widgets   │
│  (TextArea)   │ │  (Future)      │
└───────────────┘ └────────────────┘
```

## Key Components

### 1. TextInput Trait ([frontend/common/text_input.rs](../src/frontend/common/text_input.rs))

Core interface for text editing operations:

```rust
pub trait TextInput {
    // Content access
    fn text(&self) -> String;
    fn lines(&self) -> Vec<String>;
    fn set_text(&mut self, text: String);

    // Editing
    fn insert_char(&mut self, c: char);
    fn insert_str(&mut self, s: &str);
    fn delete_char(&mut self);
    fn delete_forward_char(&mut self);

    // Cursor movement
    fn move_cursor(&mut self, mv: CursorMove);
    fn cursor_position(&self) -> (u16, u16);

    // Selection
    fn start_selection(&mut self);
    fn cancel_selection(&mut self);
    fn selected_text(&self) -> String;

    // Internal clipboard
    fn yank_text(&self) -> String;
    fn set_yank_text(&mut self, text: String);
}
```

### 2. TextEditor Trait

High-level editing operations built on TextInput:

```rust
pub trait TextEditor: TextInput {
    fn select_all(&mut self);
    fn copy(&mut self) -> String;
    fn cut(&mut self) -> String;
    fn paste(&mut self, text: &str);
    fn undo(&mut self);  // Backend-specific
    fn redo(&mut self);  // Backend-specific
}
```

Auto-implemented for any type that implements `TextInput`.

### 3. TUI Adapter ([frontend/tui/text_area_adapter.rs](../src/frontend/tui/text_area_adapter.rs))

Wraps `tui-textarea::TextArea` to implement `TextInput`:

```rust
use crate::frontend::tui::text_area_adapter::TextAreaExt;
use tui_textarea::TextArea;

let mut text_area = TextArea::default();

// Use the extension trait to get an adapter
let mut adapter = text_area.as_input();

// Now works with frontend-agnostic code
adapter.insert_str("Hello");
adapter.move_cursor(CursorMove::End);
```

## Usage Patterns

### Pattern 1: Direct TextInput Usage

For widgets that want to be frontend-agnostic:

```rust
use crate::frontend::common::text_input::{TextInput, CursorMove};

fn process_input(field: &mut dyn TextInput, content: &str) {
    field.clear();
    field.insert_str(content);
    field.move_cursor(CursorMove::Home);
}
```

### Pattern 2: TUI Widget Using Adapter

For existing TUI widgets with TextArea fields:

```rust
use tui_textarea::TextArea;
use crate::frontend::common::text_input::{TextInput, TextEditor};
use crate::frontend::tui::text_area_adapter::TextAreaExt;

struct MyForm {
    name_field: TextArea<'static>,
}

impl MyForm {
    fn select_all_name(&mut self) {
        // Use the extension trait to get frontend-agnostic interface
        self.name_field.as_input().select_all();
    }

    fn copy_name(&mut self) -> String {
        self.name_field.as_input().copy()
    }
}
```

### Pattern 3: Key Handling Helper

Using the provided key handling helper:

```rust
use crate::frontend::common::text_input::handle_text_input_key;
use crate::frontend::common::{KeyCode, KeyModifiers};

fn handle_key(field: &mut dyn TextInput, key: KeyCode, mods: KeyModifiers) {
    if handle_text_input_key(field, key, mods) {
        // Key was handled by text input
        return;
    }

    // Handle other keys...
}
```

## Migration Strategy

### Current State (Phase 4)

- ✅ TextInput trait defined
- ✅ TextEditor trait auto-implemented
- ✅ TUI adapter for tui-textarea created
- ✅ Helper functions for key handling
- 🔄 Existing widgets continue using TextArea directly

### Future (GUI Support)

When adding GUI frontend:

1. Implement `TextInput` for GUI text widgets (egui::TextEdit, iced::TextInput, etc.)
2. Form widgets can then work with both frontends via the trait
3. Shared validation, formatting, and editing logic works everywhere

Example future implementation:

```rust
// Future: egui adapter
impl TextInput for egui::TextEdit {
    fn text(&self) -> String { /* ... */ }
    fn insert_str(&mut self, s: &str) { /* ... */ }
    // ... other methods
}

// Same form code works with both TUI and GUI!
fn validate_email(field: &dyn TextInput) -> Result<String> {
    let email = field.text();
    if email.contains('@') {
        Ok(email)
    } else {
        Err("Invalid email")
    }
}
```

## Benefits

1. **Code Reuse**: Text editing logic can be shared across frontends
2. **Testability**: Easy to test with mock implementations
3. **Flexibility**: Switch text editing backends without changing widget code
4. **Type Safety**: Trait bounds ensure correct usage
5. **Progressive Adoption**: Existing code continues working, new code can use abstraction

## Related Files

- [src/frontend/common/text_input.rs](../src/frontend/common/text_input.rs) - Core traits and types
- [src/frontend/tui/text_area_adapter.rs](../src/frontend/tui/text_area_adapter.rs) - TUI adapter
- [src/frontend/tui/widget_traits.rs](../src/frontend/tui/widget_traits.rs) - Existing TUI-specific traits
- [src/frontend/common/input.rs](../src/frontend/common/input.rs) - Key/mouse input abstraction
