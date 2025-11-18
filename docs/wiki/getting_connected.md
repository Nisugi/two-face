# Getting Connected

Two-Face supports two connection modes: **Direct Connection** (standalone) and **Lich Proxy** (traditional). This page covers both modes, from CLI flags to parser behavior.

## Connection Modes

### Direct Connection (Standalone)

Two-Face can authenticate directly with GemStone IV's eAccess servers without requiring Lich:

```bash
two-face --direct \
  --direct-account YOUR_ACCOUNT \
  --direct-password YOUR_PASSWORD \
  --direct-game prime \
  --direct-character CHARACTER_NAME
```

**How it works:**
1. Connects to `eaccess.play.net:7910` via TLS
2. Performs challenge-response authentication
3. Retrieves character list and launch ticket
4. Connects directly to the game server

**Requirements:**
- OpenSSL installed via vcpkg (Windows)
- On first run, downloads and pins the eAccess certificate to `~/.two-face/simu.pem`

**Advantages:**
- No Lich dependency
- Lower latency
- Simpler deployment

### Lich Proxy (Traditional)

Connect through Lich for script integration and legacy compatibility:

```bash
two-face --host 127.0.0.1 --port 8000
```

**Advantages:**
- Access to Lich scripts
- Proven stability
- Shared session with other Lich clients

## Launch Workflow (Lich Proxy)

1. **CLI Parsing** (`main.rs`): command-line options set the port, character, config paths, and frontend.
2. **Network Bootstrap** (`network::LichConnection::start`):
   - Connects to `host:port` via Tokio TCP.
   - Sends `SET_FRONTEND_PID:<pid>` so Lich knows a client is attached.
   - Splits the socket into read/write halves.
3. **Server Message Loop**:
   - Reader task converts every newline-delimited string into a `ServerMessage::Text`, leaving blank lines intact.
   - Writer task pulls commands from an `mpsc` channel so game input is serialized.

If the reader sees EOF or an error, it emits `ServerMessage::Disconnected`, which AppCore turns into UI feedback.

## Connection Tips

- Default host is `127.0.0.1`; override via `config.connection.host`.
- Ports under 1024 usually require elevation; stick with the typical 8000+ range.
- If `Connected` never appears, confirm Lich is running and that no firewall is blocking the local port.
- Enable tracing (`RUST_LOG=info two-face ...`) to see connection lifecycle logs (`tracing::info!` in `network.rs`).

## Parser Primer

Once data arrives, `parser::XmlParser` converts the XML into strongly typed `ParsedElement`s. Highlights:

- Tracks nested streams (`<pushStream>`, `<popStream>`), presets, and style attributes.
- Emits special elements for prompts, spell lists, compass directions, progress bars, menus, and “active effect” blocks.
- Detects inventory terminators (strings like “You pick up …”) to auto-pop the `inv` stream even if the server forgets.
- Supports configurable event patterns (`config.event_patterns`) to turn combat text into countdowns/status indicators.

Understanding the parser explains why window names mirror stream IDs (`main`, `thoughts`, `inv`, etc.) and why highlight regexes operate on fully decoded text (HTML entities are unescaped).

## AppCore Responsibilities

`core::AppCore` subscribes to parser output and:

- Updates `data::ui_state` (window buffers, prompts, status indicators, menus).
- Maintains window focus, selection, search state, and deferred commands.
- Routes events to whichever frontend you launched.

Most of the “how does the UI know about X?” questions are answered by browsing `AppCore::handle_parsed_element`.

## Authentication

### Direct Mode
Credentials are provided via command-line flags. The eAccess protocol:
1. Sends "K" challenge to request a hash key
2. Receives 32-byte hash key from server
3. Obfuscates password: `((password[i] - 32) ^ hashkey[i]) + 32`
4. Sends login payload with encoded credentials
5. Retrieves character list and session ticket

**Security:** Passwords are obfuscated (not encrypted) during transmission over TLS. No credentials are stored.

### Lich Proxy Mode
Authentication happens in the Lich console (stormfront login, etc.). Two-Face inherits the session after you connect; there's no separate credential prompt. Keep your Lich scripts updated to the latest version if you see XML-incompatible changes.

## Error Handling & Recovery

### Direct Mode
- Check `~/.two-face/two-face.log` for detailed authentication debug output
- Delete `~/.two-face/simu.pem` to re-download the eAccess certificate if connection fails
- Verify OpenSSL is properly installed: `echo $VCPKG_ROOT` should point to vcpkg installation
- Test credentials work via Lich first if getting authentication errors

### Lich Proxy Mode
- If the TCP connection closes, Two-Face sets a "Disconnected" flag and drops back to idle. Re-run the binary to reconnect.
- Parser errors (malformed XML, invalid regex) are logged via `tracing::warn!` with enough context to fix the offending file.
- Network stats continue to display until the socket fully closes, so check the Performance widget if you suspect partial failure.

## Offline Mode

While not a full “replay” mode, you can point Two-Face at a log file by piping the contents into a fake Lich port (using `socat`, `netcat`, etc.). As long as the XML formatting is intact, the parser and UI respond like a live session.
