# Upgrading

How to update Two-Face to newer versions while preserving your configuration.

## Before Upgrading

### 1. Backup Your Configuration

Your configuration files are precious. Back them up:

```bash
# Linux/macOS
cp -r ~/.two-face ~/.two-face.backup

# Windows (PowerShell)
Copy-Item -Recurse $env:USERPROFILE\.two-face $env:USERPROFILE\.two-face.backup
```

### 2. Check the Changelog

Review [Version History](../migration/version-history.md) for:
- Breaking changes
- New features
- Deprecated settings
- Migration requirements

---

## Upgrade Methods

### Pre-built Binaries

1. Download the new version from [GitHub Releases](https://github.com/nisugi/two-face/releases)
2. Replace the old binary with the new one
3. Launch Two-Face

Your configuration files in `~/.two-face/` are preserved automatically.

### Building from Source

```bash
cd two-face

# Pull latest changes
git pull origin master

# Rebuild
cargo build --release

# New binary is at target/release/two-face
```

---

## Configuration Compatibility

### Automatic Migration

Two-Face attempts to handle configuration changes automatically:
- Missing keys use default values
- Deprecated keys are ignored with warnings
- New features are disabled until configured

### Manual Migration

For breaking changes, you may need to update config files manually. Check the version-specific notes below.

---

## Version-Specific Upgrade Notes

### v0.1.x → v0.2.x

*No breaking changes expected during alpha.*

### Future Versions

Check [Version History](../migration/version-history.md) for detailed migration guides when released.

---

## Troubleshooting Upgrades

### Config file errors after upgrade

1. Check the error message for the problematic key
2. Compare your config with the [default files](../reference/default-files.md)
3. Update or remove the problematic setting

### Reset to defaults

If needed, reset configuration:

```bash
# Backup first!
cp -r ~/.two-face ~/.two-face.backup

# Remove old config (keeps profiles)
rm ~/.two-face/*.toml

# Launch Two-Face to regenerate defaults
two-face
```

### Partial reset

Reset specific files:

```bash
# Reset just keybinds
rm ~/.two-face/keybinds.toml
two-face  # Regenerates default keybinds
```

---

## Downgrading

To revert to an older version:

1. Download the older release
2. Replace the binary
3. Restore your backed-up configuration if needed

**Note:** Config files from newer versions may not work with older versions. Use your backup.

---

## See Also

- [Version History](../migration/version-history.md) - Changelog and migration guides
- [Default Files](../reference/default-files.md) - Default configuration reference
- [Troubleshooting](../troubleshooting/README.md) - General troubleshooting
