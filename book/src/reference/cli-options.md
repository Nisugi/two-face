# CLI Options

Complete command line interface reference for Two-Face.

## Usage

```bash
two-face [OPTIONS]
```

## Connection Options

### Lich Proxy Mode

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--host` | string | `127.0.0.1` | Lich proxy hostname |
| `--port` | integer | `8000` | Lich proxy port |

```bash
two-face --host 127.0.0.1 --port 8000
```

### Direct eAccess Mode

| Option | Type | Required | Description |
|--------|------|----------|-------------|
| `--direct` | flag | Yes | Enable direct mode |
| `--account` | string | Yes | Simutronics account |
| `--password` | string | Yes | Account password |
| `--game` | string | Yes | Game instance |
| `--character` | string | Yes | Character name |

```bash
two-face --direct \
  --account myaccount \
  --password mypassword \
  --game prime \
  --character Mycharacter
```

### Game Instance Values

| Value | Game |
|-------|------|
| `prime` | GemStone IV (Prime) |
| `test` | GemStone IV (Test) |
| `plat` | GemStone IV (Platinum) |
| `dr` | DragonRealms (Prime) |
| `drt` | DragonRealms (Test) |

## Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--config` | path | `~/.two-face/config.toml` | Main config file |
| `--layout` | path | `~/.two-face/layout.toml` | Layout config file |
| `--colors` | path | `~/.two-face/colors.toml` | Color theme file |
| `--keybinds` | path | `~/.two-face/keybinds.toml` | Keybinds file |
| `--profile` | string | none | Load named profile |

```bash
# Custom config location
two-face --config /path/to/config.toml

# Load profile
two-face --profile warrior
```

## Logging Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--debug` | flag | false | Enable debug logging |
| `--log-file` | path | `~/.two-face/two-face.log` | Log file location |
| `--log-level` | string | `info` | Log verbosity |

### Log Levels

| Level | Description |
|-------|-------------|
| `error` | Errors only |
| `warn` | Warnings and errors |
| `info` | General information (default) |
| `debug` | Debug information |
| `trace` | Verbose tracing |

```bash
# Debug logging
two-face --debug

# Custom log file
two-face --log-file /tmp/debug.log --log-level trace
```

## Display Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--no-color` | flag | false | Disable colors |
| `--force-color` | flag | false | Force color output |

```bash
# Disable colors (for logging/piping)
two-face --no-color
```

## Miscellaneous Options

| Option | Type | Description |
|--------|------|-------------|
| `--version`, `-V` | flag | Print version |
| `--help`, `-h` | flag | Print help |
| `--dump-config` | flag | Print default config |

```bash
# Show version
two-face --version

# Show help
two-face --help

# Print default configuration
two-face --dump-config > ~/.two-face/config.toml
```

## Option Precedence

Options are applied in this order (later overrides earlier):

1. Default values
2. Configuration files
3. Environment variables
4. Command line arguments

## Examples

### Basic Lich Connection

```bash
two-face --host 127.0.0.1 --port 8000
```

### Direct Connection with Debug

```bash
two-face --direct \
  --account myaccount \
  --password mypassword \
  --game prime \
  --character Warrior \
  --debug
```

### Custom Configuration

```bash
two-face \
  --config ~/games/two-face/config.toml \
  --layout ~/games/two-face/hunting.toml
```

### Profile-Based Launch

```bash
# Assumes ~/.two-face/profiles/wizard.toml exists
two-face --profile wizard
```

### Scripted Launch (Lich)

```bash
#!/bin/bash
# launch_gs4.sh

# Start Lich in background
lich &
sleep 5

# Connect Two-Face
two-face --host 127.0.0.1 --port 8000
```

### Environment-Based (Direct)

```bash
#!/bin/bash
# Credentials from environment
export TF_ACCOUNT="myaccount"
export TF_PASSWORD="mypassword"
export TF_GAME="prime"

two-face --direct --character "$1"
```

## Short Options

| Short | Long | Description |
|-------|------|-------------|
| `-h` | `--host` | Host (Lich mode) |
| `-p` | `--port` | Port (Lich mode) |
| `-c` | `--config` | Config file |
| `-d` | `--debug` | Debug mode |
| `-V` | `--version` | Version |

```bash
# Short form
two-face -h 127.0.0.1 -p 8000 -d
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Connection error |
| 4 | Authentication error |

## See Also

- [Configuration](../configuration/README.md)
- [Environment Variables](./environment-vars.md)
- [Network Overview](../network/README.md)

