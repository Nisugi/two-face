# Migration Guides

Guides for transitioning to Two-Face from other clients.

## Overview

If you're coming from another GemStone IV or DragonRealms client, these guides help you translate your existing setup to Two-Face.

## Migration Guides

| Previous Client | Guide |
|-----------------|-------|
| Profanity | [From Profanity](./from-profanity.md) |
| Wizard Front End | [From WFE](./from-wfe.md) |
| StormFront | [From StormFront](./from-stormfront.md) |

## Common Migration Tasks

### Configuration Translation

Each client has different configuration formats. Common concepts to translate:

| Concept | Profanity | WFE | Two-Face |
|---------|-----------|-----|----------|
| Layout | Window positions | Window setup | `layout.toml` |
| Colors | Theme files | Color settings | `colors.toml` |
| Macros | Scripts/aliases | Macros | `keybinds.toml` |
| Triggers | Triggers | Triggers | `triggers.toml` |
| Highlights | Highlights | Highlights | `highlights.toml` |

### Feature Mapping

| Feature | Common In | Two-Face Equivalent |
|---------|-----------|---------------------|
| Text windows | All clients | `text` widgets |
| Health bars | All clients | `progress` widgets |
| Compass | All clients | `compass` widget |
| Macros | All clients | Keybind macros |
| Triggers | All clients | `triggers.toml` |
| Scripts | Profanity/Lich | Via Lich proxy |

## General Migration Steps

### 1. Install Two-Face

Follow the [installation guide](../getting-started/installation.md).

### 2. Connect to Game

Test connection before migrating configuration:

```bash
# Via Lich (existing setup)
two-face --host 127.0.0.1 --port 8000
```

### 3. Create Basic Layout

Start with default layout, then customize:

```bash
# Use built-in defaults first
two-face --dump-config > ~/.two-face/config.toml
```

### 4. Translate Configuration

Migrate settings from your previous client:

1. Layout/window positions
2. Colors/theme
3. Macros/keybinds
4. Triggers
5. Highlights

### 5. Test and Refine

- Test each migrated feature
- Adjust as needed
- Take advantage of new features

## What Carries Over

### Lich Scripts

If you use Lich, your scripts continue to work. Two-Face connects through Lich just like other clients.

### Game Settings

In-game settings (character options, game preferences) are server-side and unaffected.

### Account Information

Your Simutronics account, characters, and subscriptions are unchanged.

## What Changes

### Configuration Files

Each client uses different file formats. Configuration must be recreated (not copied).

### Keybinds

Keybind syntax differs. Translate your macros to Two-Face format.

### Visual Layout

Screen arrangement uses different systems. Recreate your preferred layout.

## Migration Tips

### Start Fresh vs. Replicate

**Option A: Start Fresh**
- Use Two-Face defaults
- Learn new features
- Gradually customize

**Option B: Replicate**
- Match previous client exactly
- Familiar environment
- Then explore new features

### Document Your Previous Setup

Before migrating, note:
- Window positions and sizes
- Important macros
- Color preferences
- Trigger patterns

### Take Your Time

Migration doesn't have to be immediate:
- Run Two-Face alongside previous client
- Migrate incrementally
- Test each feature

## Version History

See [Version History](./version-history.md) for:
- Release notes
- Breaking changes
- Upgrade instructions

## Getting Help

If you encounter issues:
- Check [Troubleshooting](../troubleshooting/README.md)
- Search GitHub issues
- Ask the community

## See Also

- [Getting Started](../getting-started/README.md)
- [Configuration](../configuration/README.md)
- [Tutorials](../tutorials/README.md)

