# Two-Face Development Notes

## Direct eAccess Authentication

Two-Face now supports **direct authentication** with GemStone IV's eAccess servers, bypassing the need for Lich as a proxy. This allows standalone operation.

### Implementation Details

The direct authentication feature is implemented in [src/network.rs](../src/network.rs#L205-L554) using the eAccess protocol:

1. **TLS Handshake**: Connects to `eaccess.play.net:7910` using OpenSSL
   - Disables SNI (Server Name Indication) to match Lich's behavior
   - Disables session caching (Session ID = 0 bytes)
   - Stores and verifies the self-signed certificate at `~/.two-face/simu.pem`

2. **Challenge-Response Authentication**:
   - Sends "K" to request a hash key
   - Receives 32-byte hash key from server
   - Obfuscates password using: `((password[i] - 32) ^ hashkey[i]) + 32`
   - Sends login payload: `A\t{account}\t{encoded_password}\n`

3. **Session Establishment**:
   - Retrieves character list and subscription info
   - Requests launch ticket with character code
   - Connects to game server with the received key

### Key Technical Fix

The critical fix that made authentication work was ensuring **single-write TLS records**. The `send_line` function at [src/network.rs#L402-L413](../src/network.rs#L402-L413) builds the complete message (including newline) in memory before writing, ensuring it goes out as a single TLS Application Data record instead of two separate records.

```rust
fn send_line(stream: &mut SslStream<TcpStream>, line: &str) -> Result<()> {
    // Match Ruby's puts - sends string with newline in a SINGLE write
    let mut message = Vec::with_capacity(line.len() + 1);
    message.extend_from_slice(line.as_bytes());
    message.push(b'\n');

    stream.write_all(&message)?;
    stream.flush()?;
    Ok(())
}
```

### Usage

```bash
# Direct connection mode
two-face --direct \
  --direct-account YOUR_ACCOUNT \
  --direct-password YOUR_PASSWORD \
  --direct-game prime \
  --direct-character CHARACTER_NAME

# Or via Lich (traditional mode)
two-face --host 127.0.0.1 --port 8000
```

### Dependencies

Direct mode requires OpenSSL, installed via vcpkg:

```toml
[dependencies]
openssl = "0.10"
vcpkg = "0.2"
```

On Windows, ensure `VCPKG_ROOT` is set:
```bash
export VCPKG_ROOT="C:/path/to/vcpkg"
```

### Security Notes

- Passwords are obfuscated (not encrypted) during transmission over TLS
- The self-signed certificate is pinned after first download
- Certificate verification ensures we're talking to the authentic eAccess server
- No credentials are stored; they must be provided each launch

### Troubleshooting

If authentication fails:
1. Check `~/.two-face/two-face.log` for debug output
2. Verify OpenSSL is properly installed via vcpkg
3. Delete `~/.two-face/simu.pem` to re-download the certificate
4. Ensure account credentials are correct (test via Lich first)

### Performance

Direct mode with release build optimizations:
- Full compiler optimizations (`opt-level = 3`)
- Link-time optimization (`lto = true`)
- Single codegen unit for maximum performance

Provides smooth, low-latency gameplay comparable to native clients.
