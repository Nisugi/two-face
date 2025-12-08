# Quick Start Guide

Get Two-Face running in 5 minutes!

## 1. Download

Get the latest release from [GitHub Releases](https://github.com/nisugi/two-face/releases).

| Platform | File |
|----------|------|
| Windows | `two-face-windows.zip` |
| macOS | `two-face-macos.tar.gz` |
| Linux | `two-face-linux.tar.gz` |

## 2. Connect

### Option A: Via Lich (Recommended)

```bash
# Start Lich first, then:
two-face --host 127.0.0.1 --port 8000
```

### Option B: Direct eAccess

```bash
two-face --direct \
  --account YOUR_ACCOUNT \
  --password YOUR_PASSWORD \
  --game prime \
  --character CHARACTER_NAME
```

## 3. Basic Controls

| Key | Action |
|-----|--------|
| `Enter` | Send command |
| `↑` / `↓` | Command history |
| `Ctrl+L` | Clear screen |
| `Ctrl+Q` | Quit |
| `Tab` | Next window |
| `Shift+Tab` | Previous window |
| `PgUp` / `PgDn` | Scroll window |
| `F1` | Open menu |

## 4. Configuration Files

Located in your config directory:

```
~/.config/two-face/          # Linux
~/Library/Application Support/two-face/  # macOS
%APPDATA%\two-face\          # Windows
```

| File | Purpose |
|------|---------|
| `config.toml` | Main settings |
| `layout.toml` | Window layout |
| `keybinds.toml` | Key bindings |
| `highlights.toml` | Text highlighting |
| `colors.toml` | Color theme |

## 5. First Steps

1. **Try the menu**: Press `F1`
2. **Edit layout**: `F1` → Layout → Edit Windows
3. **Add highlights**: `F1` → Highlights → Add Pattern
4. **Set keybinds**: `F1` → Keybinds → Add Binding

## Next Steps

- [Full Documentation](https://nisugi.github.io/two-face/)
- [Keybind Cheatsheet](Keybind-Cheatsheet)
- [Widget Reference](Widget-Quick-Reference)
