# Installation & First Run

Two-Face is a Rust application, so you get native binaries on every platform Rust supports. This page walks through prerequisites, builds, and the very first launch.

## Prerequisites

| Requirement | Notes |
|-------------|-------|
| Rust toolchain 1.74+ | Install via [rustup](https://rustup.rs). The project uses edition 2021 and async networking, so stay current. |
| Lich / GemStone IV account | Two-Face speaks the same XML protocol as StormFront via Lich. Have your account and port handy. |
| Git (optional) | Only needed if you are cloning the repo rather than downloading a source archive. |
| Audio (optional) | Enable the `sound` Cargo feature if you want rodio-backed effects. |

## Building from Source

```bash
git clone https://github.com/<your-fork>/two-face.git
cd two-face
cargo build --release        # produces target/release/two-face(.exe)
```

- Release builds include full optimizations and embed default configuration assets from the `defaults/` directory.
- Use `cargo build --release --features sound` if you want baked-in rodio support.

## Installing Binaries

After building, either:

- copy `target/release/two-face` into a directory on your PATH, or
- keep the repository intact and run `cargo run --release -- ...args` for quick iteration.

## First Launch

```bash
two-face --character Zoleta --port 8000
```

This:

1. Creates `~/.two-face/` (or `%USERPROFILE%\.two-face\` on Windows) if it does not already exist.
2. Extracts default files: `config.toml`, `colors.toml`, `highlights.toml`, `keybinds.toml`, `layouts/`, `sounds/`, and `cmdlist1.xml`.
3. Attaches to Lich on the specified port (default is `8000`).
4. Starts the TUI frontend. Add `--frontend gui` once the egui mode ships.

If you have multiple characters, pass `--character <Name>` so per-character directories are created automatically.

## CLI Reference

Command-line arguments are defined in `src/main.rs` via `clap`:

- `--config <FILE>` – override the global config path.
- `--frontend <tui|gui>` – select the frontend (default `tui`).
- `--port <u16>` – TCP port for the Lich connection (default `8000`).
- `--character <Name>` – character identifier used when loading/saving config.
- `--data-dir <DIR>` – override `~/.two-face`. Environment variable `TWO_FACE_DIR` is also honored.

Every flag can be combined with subcommands (future expansion for utility tooling); for regular play you just pass the options shown above.
