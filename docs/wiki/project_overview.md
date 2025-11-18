# Project Overview & Concepts

Two-Face is a modernization of VellumFE that keeps the beloved feature set of the classic GemStone IV front-end while splitting responsibilities into clear layers. Understanding the moving parts makes later customization much easier.

## Goals & Heritage

- **Multi-frontend**: Choose between the terminal interface (ratatui) and the upcoming egui desktop UI without changing the core game logic.
- **Frontend-agnostic core**: Parsing, connection management, app state, and automation live in `src/core` and `src/data`, so both frontends consume the same model.
- **Drop-in familiarity**: Layouts, highlights, command lists, sounds, and macros from VellumFE continue to work (files live under the same `.two-face` hierarchy).

## Architecture at a Glance

```
two-face/
├── src/
│   ├── main.rs            # CLI parsing, startup, frontend selection
│   ├── config.rs          # Typed configuration loader & persistence helpers
│   ├── parser.rs          # XML stream interpreter and event generator
│   ├── network.rs         # Lich/TCP connection loop
│   ├── core/              # AppCore, input router, menu actions, messages
│   ├── data/              # Pure data structures: windows, widgets, UI state
│   ├── frontend/
│   │   ├── tui/           # Ratatui widgets and window renderers
│   │   └── gui/           # Placeholder for the egui implementation
│   ├── sound.rs           # Optional rodio wrapper behind the `sound` feature
│   ├── theme.rs           # AppTheme definition + presets/custom loader
│   └── selection.rs       # Shared text-selection helpers
└── defaults/              # Embedded starter files extracted on first run
```

### Data Flow

1. **Network** (`network::LichConnection`) reads Lich XML.
2. **Parser** (`parser::XmlParser`) converts XML into typed `ParsedElement`s (text, prompts, UI updates, menu responses, etc.).
3. **Core/AppCore** consumes parsed elements, mutates `data::ui_state`, and dispatches high-level messages.
4. **Frontend** pulls from `AppCore`, renders widgets, and routes user input back through `core::input_router`.

Because the layers are cleanly separated, most user-facing customization (layouts, keybinds, colors) manipulates serialized data rather than code.

## Major Feature Buckets

- **Window/layout system** modeled after VellumFE with per-stream definitions, borders, and drag-to-edit pop-ups.
- **Command input experience** with history, multi-byte cursoring, word jumps, dot-command autocomplete, and optional selection/copy.
- **Automation & highlights** with regex validation, sound hooks, color overrides, and enumeration of active effects.
- **Themeing stack** combining full-application themes (`theme.rs`), reusable palette entries, UI color overrides, spell colors, and highlight colors.
- **Performance instrumentation** (frame/render/parse stats, network throughput, event timing) sourced from `performance.rs`.

Use the rest of the wiki to drill into each bucket; this page is the mental map.
