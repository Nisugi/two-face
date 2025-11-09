# Two-Face

Multi-frontend (TUI/GUI) client for GemStone IV - Refactored architecture from VellumFE.

## Project Status

**Code Name:** Two-Face
**Status:** In Development - Phase 1+2 (Architecture Refactoring)
**Parent Project:** [VellumFE](https://github.com/Nisugi/VellumFE)

## Goals

1. Refactor VellumFE into a clean frontend-agnostic architecture
2. Implement TUI frontend (ratatui) maintaining all existing functionality
3. Implement GUI frontend (egui) with proportional font support
4. Allow users to choose between TUI (`--tui`) or GUI (`--gui`) modes

## Architecture

```
two-face/
├── src/
│   ├── core/           # Business logic (frontend-agnostic)
│   ├── widgets/        # Widget state (data only)
│   ├── frontend/
│   │   ├── tui/       # Ratatui rendering
│   │   └── gui/       # egui rendering (future)
│   ├── config.rs
│   ├── network.rs
│   ├── parser.rs
│   └── main.rs
```

## Building

```bash
cargo build --release
```

## Running

```bash
# TUI mode (default)
two-face.exe --character Zoleta --port 8000

# GUI mode (future)
two-face.exe --gui --character Zoleta --port 8000
```

## Development Roadmap

**Phase 1+2: TUI Refactor (Foundation)**
- [x] Milestone 1: Project Setup & Bootstrap
- [x] Milestone 2: Create Abstraction Layer
- [x] Milestone 3: Extract AppCore (Initial Structure)
- [x] Milestone 4: Create Widget State Structs (Core Widgets)
- [x] Milestone 5: Create TUI Frontend Module
- [ ] Milestone 6: Wire Everything Together (IN PROGRESS)
- [ ] Milestone 7: Testing & Verification

**Phase 3: GUI Frontend (Future)**
- [ ] Add egui dependencies
- [ ] Create GUI frontend module
- [ ] Implement GUI rendering
- [ ] Add --gui flag
- [ ] Polish and release

See [PROGRESS.md](PROGRESS.md) for detailed status and next steps.

## License

MIT OR Apache-2.0
