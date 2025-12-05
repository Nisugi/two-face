# GUI Integration Roadmap

## Overview

This document outlines the step-by-step plan for adding a GUI frontend to Two-Face using the existing dual-frontend architecture.

## Framework Selection

### Recommended: egui

**Pros**:
- Pure Rust, no C dependencies
- Immediate mode (simple state management)
- Cross-platform (Windows, Linux, macOS)
- Good performance
- Active development
- Similar rendering model to ratatui (draw each frame)

**Cons**:
- Less native look/feel
- Fewer complex widgets than Qt/GTK

### Alternative: iced

**Pros**:
- More native-looking
- Retained mode (automatic state management)
- Inspired by Elm (clean architecture)

**Cons**:
- Less mature than egui
- More complex state management
- Different rendering model from ratatui

**Decision**: Start with **egui** for faster initial implementation

## Implementation Phases

### Phase 1: Basic GUI Framework Setup (Estimated: 2-4 hours)

**Goal**: Get a basic egui window running with Frontend trait

**Tasks**:
1. Add egui dependencies to Cargo.toml
   ```toml
   [dependencies]
   egui = "0.28"
   eframe = "0.28"  # egui framework
   ```

2. Create `EguiApp` struct in `frontend/gui/mod.rs`
3. Implement `Frontend` trait for `EguiApp`
   - `poll_events()` - Convert egui input → FrontendEvent
   - `render()` - Basic window rendering
   - `cleanup()` - Window cleanup
   - `size()` - Window dimensions

4. Add CLI flag to select frontend
   ```rust
   let frontend: Box<dyn Frontend> = if args.gui {
       Box::new(EguiApp::new())
   } else {
       Box::new(TuiFrontend::new()?)
   };
   ```

**Success Criteria**:
- Empty egui window opens
- Responds to quit events
- Compiles without errors

### Phase 2: Event Bridge (Estimated: 2-3 hours)

**Goal**: Translate egui events to FrontendEvent

**Tasks**:
1. Create `frontend/gui/egui_bridge.rs`
2. Implement event conversion functions:
   ```rust
   pub fn convert_key(key: egui::Key) -> Option<KeyCode>
   pub fn convert_modifiers(mods: egui::Modifiers) -> KeyModifiers
   pub fn convert_pointer(ptr: &egui::PointerState) -> Vec<MouseEvent>
   ```

3. Update `EguiApp::poll_events()` to use bridge
4. Test keyboard shortcuts work (Ctrl+Q quit, etc.)

**Success Criteria**:
- All keyboard input captured
- Mouse clicks/movement captured
- Modifiers (Ctrl/Shift/Alt) work correctly

### Phase 3: TextInput Adapter (Estimated: 1-2 hours)

**Goal**: Implement TextInput trait for egui::TextEdit

**Tasks**:
1. Create `frontend/gui/text_input_adapter.rs`
2. Implement TextInput for egui text widgets
   ```rust
   impl TextInput for EguiTextAdapter {
       fn text(&self) -> String { /* ... */ }
       fn insert_str(&mut self, s: &str) { /* ... */ }
       // ...
   }
   ```

3. Test copy/paste, selection, cursor movement

**Success Criteria**:
- TextEditor trait works with egui widgets
- Copy/cut/paste operations work
- Cursor movement matches TUI behavior

### Phase 4: Widget Renderers (Estimated: 4-6 hours)

**Goal**: Create GUI renderers for all widget data structures

**Tasks**:

**4.1 ProgressBar Renderer**
```rust
// frontend/gui/renderers/progress_bar.rs
pub fn render_progress_bar(data: &ProgressBarData, ui: &mut egui::Ui) {
    let percentage = data.percentage as f32 / 100.0;

    let bar = egui::ProgressBar::new(percentage)
        .text(&data.display_text);

    // Apply custom colors if specified
    if let Some(ref fill_color) = data.bar_fill_color {
        let color = parse_hex_color(fill_color);
        // Apply color to bar
    }

    ui.add(bar);
}
```

**4.2 Indicator Renderer**
```rust
pub fn render_indicator(data: &IndicatorData, ui: &mut egui::Ui) {
    if !data.active {
        return; // Hidden when inactive
    }

    let color = parse_hex_color(&data.on_color);
    ui.label(egui::RichText::new(&data.label).color(color));
}
```

**4.3 Countdown Renderer**
```rust
pub fn render_countdown(data: &CountdownData, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(&data.display_text);

        // Render block glyphs
        for i in 0..data.blocks_to_show {
            ui.label(data.icon.to_string());
        }
    });
}
```

**4.4 Hand Renderer**
```rust
pub fn render_hand(data: &HandData, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.label(&data.icon);
        ui.label(&data.content);
    });
}
```

**Success Criteria**:
- All 4 widget types render correctly
- Colors apply properly
- Layout matches design intent

### Phase 5: Layout System (Estimated: 3-5 hours)

**Goal**: Implement the main application layout in GUI

**Tasks**:
1. Create main panel layout
   ```rust
   egui::TopBottomPanel::top("top").show(ctx, |ui| {
       // Menu bar, toolbar
   });

   egui::SidePanel::left("left").show(ctx, |ui| {
       // Vitals, indicators
   });

   egui::CentralPanel::default().show(ctx, |ui| {
       // Main game text
   });
   ```

2. Map TUI window layout to GUI panels
3. Implement resizable splits
4. Add scrolling for game text

**Success Criteria**:
- Layout resembles TUI layout
- Panels can be resized
- All widgets fit properly

### Phase 6: Theme Support (Estimated: 2-3 hours)

**Goal**: Apply theme colors to GUI

**Tasks**:
1. Convert theme hex colors to egui::Color32
2. Apply theme to widgets
3. Implement theme switching
4. Test all themes work in GUI

**Success Criteria**:
- Theme colors apply correctly
- Can switch themes at runtime
- Custom theme JSON files work

### Phase 7: Testing & Polish (Estimated: 4-6 hours)

**Goal**: Ensure GUI frontend is production-ready

**Tasks**:
1. Test all game functionality in GUI
2. Compare with TUI behavior
3. Fix visual bugs
4. Add GUI-specific features (if any)
5. Performance testing
6. Documentation

**Success Criteria**:
- All features work in both frontends
- No visual glitches
- Smooth 60 FPS rendering
- User documentation updated

## Total Estimated Time

**Minimum**: 18 hours (optimistic, experienced with egui)
**Maximum**: 29 hours (including polish and edge cases)
**Realistic**: ~24 hours spread over 3-4 days

## Dependencies

```toml
[dependencies]
egui = "0.28"
eframe = "0.28"

[features]
default = ["tui"]
tui = []  # Terminal UI (default)
gui = []  # Desktop GUI
both = ["tui", "gui"]  # Both frontends (debugging)
```

## CLI Usage After Implementation

```bash
# TUI mode (default)
two-face

# GUI mode
two-face --gui

# Force TUI even if GUI compiled
two-face --tui
```

## Testing Strategy

### Unit Tests
- Event conversion (egui → FrontendEvent)
- Widget renderers (data → visual output)
- TextInput adapter

### Integration Tests
- Full render cycle
- Input handling end-to-end
- Theme application

### Manual Testing
- Connect to game via GUI
- Test all widgets render correctly
- Test all keyboard shortcuts work
- Test mouse interaction
- Compare with TUI behavior

## Known Challenges

### 1. Terminal vs GUI Paradigm Shift
- **Challenge**: TUI is character-based, GUI is pixel-based
- **Solution**: Widget data already uses abstract units (percentages, etc.)

### 2. Text Rendering
- **Challenge**: Game uses ANSI color codes
- **Solution**: Parse ANSI codes, apply as egui::RichText styles

### 3. Layout Differences
- **Challenge**: TUI uses fixed grid, GUI is flexible
- **Solution**: Use egui panels with minimum sizes

### 4. Performance
- **Challenge**: Rendering thousands of text lines
- **Solution**: Use egui::ScrollArea with virtual scrolling

## Success Metrics

- [ ] GUI frontend compiles without errors
- [ ] All widgets render correctly
- [ ] All keyboard shortcuts work
- [ ] Mouse interaction works
- [ ] Themes apply correctly
- [ ] Performance ≥60 FPS with 1000+ lines
- [ ] Code reuse ≥80% (shared data structures)
- [ ] User documentation complete

## Future Enhancements (Post-MVP)

1. **Native Menus**: Use egui's menu bar for settings
2. **Dockable Panels**: Let users rearrange layout
3. **Multi-Window**: Separate windows for different game streams
4. **Rich Text Editor**: Better text input with syntax highlighting
5. **Visual Theme Editor**: GUI for creating custom themes
6. **Overlay Mode**: Transparent window overlay for other apps
7. **Mobile Support**: Port to iOS/Android via egui-winit

## Getting Started

To begin implementation:

1. Read [dual-frontend-architecture.md](dual-frontend-architecture.md) for context
2. Review Phase 1 tasks
3. Create feature branch: `git checkout -b feature/gui-frontend`
4. Add egui dependencies
5. Start with basic `EguiApp` struct
6. Follow phases sequentially

## References

- [egui Documentation](https://docs.rs/egui/)
- [eframe Examples](https://github.com/emilk/egui/tree/master/examples)
- [dual-frontend-architecture.md](dual-frontend-architecture.md) - Architecture overview
- [text-input-abstraction.md](text-input-abstraction.md) - TextInput trait usage

---

**Status**: Architecture ready, awaiting GUI implementation
**Estimated Effort**: 18-29 hours (3-4 days)
**Risk Level**: Low (architecture proven)
**Dependencies**: egui 0.28+
