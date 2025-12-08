# Frequently Asked Questions

## General

### What is Two-Face?

Two-Face is a modern, multi-frontend terminal client for GemStone IV (and potentially DragonRealms). It features:
- Customizable layouts and themes
- Text highlighting and triggers
- Multiple connection modes (Lich proxy or direct eAccess)
- Cross-platform support (Windows, macOS, Linux)

### Why "Two-Face"?

The name comes from the dual-frontend architecture - Two-Face can render to both TUI (terminal) and GUI backends from the same core.

### Is Two-Face free?

Yes! Two-Face is open source under the MIT license.

### Does Two-Face work with DragonRealms?

The parser supports the Stormfront XML protocol used by both GS4 and DR. DR support is planned but not fully tested.

## Connection

### Do I need Lich to use Two-Face?

No! Two-Face supports two connection modes:
1. **Lich proxy** (recommended) - Connect through Lich for script support
2. **Direct eAccess** - Connect directly without Lich

### How do I connect via Lich?

1. Start Lich and log into the game
2. Note the port Lich is listening on (default: 8000)
3. Run: `two-face --host 127.0.0.1 --port 8000`

### How do I connect directly?

```bash
two-face --direct \
  --account YOUR_ACCOUNT \
  --password YOUR_PASSWORD \
  --game prime \
  --character CHARACTER_NAME
```

### Can I save my login credentials?

For security, Two-Face doesn't save passwords. Use environment variables:
```bash
export GS4_ACCOUNT=your_account
export GS4_PASSWORD=your_password
two-face --direct --game prime --character NAME
```

## Configuration

### Where are config files stored?

| Platform | Location |
|----------|----------|
| Linux | `~/.config/two-face/` |
| macOS | `~/Library/Application Support/two-face/` |
| Windows | `%APPDATA%\two-face\` |

### How do I reset to default config?

Delete the config directory and restart Two-Face:
```bash
rm -rf ~/.config/two-face/
two-face
```

### Can I have multiple layouts?

Yes! Create multiple layout files and switch between them:
```bash
two-face --layout hunting.toml
two-face --layout merchant.toml
```

### How do I edit windows visually?

Press `F1` → Layout → Edit Windows, or use keyboard shortcuts to resize/move windows while in edit mode.

## Features

### Does Two-Face support scripts?

Two-Face itself doesn't run scripts. Use Lich for scripting and connect Two-Face as a frontend.

### Can I use Two-Face with Stormfront scripts?

If using Lich, yes - Lich handles all scripting. Two-Face just displays the output.

### Does Two-Face support macros?

Yes! Define macros in keybinds.toml:
```toml
[[keybinds]]
key = "F5"
action = "send"
command = "stance defensive;hide"
```

### Can I have sound alerts?

Yes! Configure in triggers.toml:
```toml
[[triggers]]
pattern = "(?i)you are stunned"
sound = "alert.wav"
```

### Does Two-Face support text-to-speech?

Yes! Enable in config.toml:
```toml
[tts]
enabled = true
```

And add TTS triggers:
```toml
[[triggers]]
pattern = "feel your life fading"
tts = "Health critical!"
```

## Troubleshooting

### Why is my display garbled?

Common causes:
1. Terminal doesn't support UTF-8
2. Font missing Unicode characters
3. Wrong TERM environment variable

Try: `export TERM=xterm-256color`

### Why are my keybinds not working?

1. Check keybinds.toml syntax
2. Some keys may be captured by your terminal
3. Try different key combinations

### Why can't I see colors?

1. Ensure terminal supports 256 colors
2. Check `TERM` environment variable
3. Verify colors.toml is valid

### Why does Two-Face crash on startup?

1. Check config file syntax (use a TOML validator)
2. Try resetting to defaults
3. Run with `--debug` flag for more info

## Comparison

### Two-Face vs Profanity?

| Feature | Two-Face | Profanity |
|---------|----------|-----------|
| TUI | Yes | Yes |
| GUI | Planned | No |
| Cross-platform | Yes | Unix only |
| Direct eAccess | Yes | No |
| Active development | Yes | Maintenance |

### Two-Face vs Wizard FE?

| Feature | Two-Face | Wizard FE |
|---------|----------|-----------|
| Platform | All | Windows |
| Open source | Yes | No |
| Customization | High | Medium |
| Scripting | Via Lich | Built-in |

## Contributing

### How can I contribute?

- Report bugs on [GitHub Issues](https://github.com/nisugi/two-face/issues)
- Submit pull requests
- Improve documentation
- Share your layouts/themes

### Where's the source code?

GitHub: https://github.com/nisugi/two-face

### What language is Two-Face written in?

Rust, using ratatui for the TUI frontend.
