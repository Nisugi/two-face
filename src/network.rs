//! Tokio-based client for the Lich proxy.
//!
//! Handles connecting to the chosen host/port, wiring async reader/writer loops,
//! and funneling everything through mpsc channels so the rest of the app stays
//! decoupled from direct socket management.

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info};

use std::path::PathBuf;

/// Messages emitted by the TCP reader task.
#[derive(Debug, Clone)]
pub enum ServerMessage {
    Text(String),
    Connected,
    Disconnected,
}

/// Stub type that exposes the async `start` helper.
pub struct LichConnection;

/// Runtime configuration for direct (non-Lich) connections.
pub struct DirectConnectConfig {
    pub account: String,
    pub password: String,
    pub character: String,
    pub game_code: String,
    pub data_dir: PathBuf,
}

/// Direct connector that authenticates via eAccess and establishes the game socket.
pub struct DirectConnection;

impl LichConnection {
    /// Connect to Lich, spawn read loop, and forward commands supplied via the provided channel.
    pub async fn start(
        host: &str,
        port: u16,
        server_tx: mpsc::UnboundedSender<ServerMessage>,
        mut command_rx: mpsc::UnboundedReceiver<String>,
    ) -> Result<()> {
        info!("Connecting to Lich at {}:{}...", host, port);

        let mut stream = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .context("Failed to connect to Lich")?;

        info!("Connected successfully");

        send_pid_handshake(&mut stream).await?;

        run_stream(stream, server_tx, command_rx).await
    }
}

impl DirectConnection {
    pub async fn start(
        config: DirectConnectConfig,
        server_tx: mpsc::UnboundedSender<ServerMessage>,
        command_rx: mpsc::UnboundedReceiver<String>,
    ) -> Result<()> {
        let DirectConnectConfig {
            account,
            password,
            character,
            game_code,
            data_dir,
        } = config;

        info!(
            "Authenticating account '{}' for character '{}' via eAccess...",
            account, character
        );

        let ticket = tokio::task::spawn_blocking(move || {
            eaccess::authenticate(&account, &password, &character, &game_code, &data_dir)
        })
        .await?
        .context("Failed to authenticate with eAccess")?;

        info!(
            "Authentication successful (world: {}, host: {}:{})",
            ticket.game, ticket.game_host, ticket.game_port
        );

        let (host, port) = fix_game_host_port(&ticket.game_host, ticket.game_port);
        info!("Connecting directly to {}:{}...", host, port);
        let mut stream = TcpStream::connect(format!("{}:{}", host, port))
            .await
            .context("Failed to connect to game server")?;

        send_direct_handshake(&mut stream, &ticket).await?;

        run_stream(stream, server_tx, command_rx).await
    }
}

async fn run_stream(
    stream: TcpStream,
    server_tx: mpsc::UnboundedSender<ServerMessage>,
    mut command_rx: mpsc::UnboundedReceiver<String>,
) -> Result<()> {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut reader = BufReader::new(reader);

    let _ = server_tx.send(ServerMessage::Connected);

    let server_tx_clone = server_tx.clone();
    let read_handle = tokio::spawn(async move {
        loop {
            let mut line = String::new();
            match reader.read_line(&mut line).await {
                Ok(0) => {
                    info!("Connection closed by server");
                    let _ = server_tx_clone.send(ServerMessage::Disconnected);
                    break;
                }
                Ok(_) => {
                    let line = line.trim_end_matches(&['\r', '\n']);
                    let _ = server_tx_clone.send(ServerMessage::Text(line.to_string()));
                }
                Err(e) => {
                    error!("Error reading from server: {}", e);
                    let _ = server_tx_clone.send(ServerMessage::Disconnected);
                    break;
                }
            }
        }
    });

    let _ = async {
        while let Some(cmd) = command_rx.recv().await {
            debug!("Sending command: {}", cmd);
            if let Err(e) = writer.write_all(cmd.as_bytes()).await {
                error!("Failed to write command: {}", e);
                break;
            }
            if let Err(e) = writer.write_all(b"\n").await {
                error!("Failed to write newline: {}", e);
                break;
            }
            if let Err(e) = writer.flush().await {
                error!("Failed to flush: {}", e);
                break;
            }
        }
    }
    .await;

    let _ = read_handle.await;

    Ok(())
}

async fn send_pid_handshake(stream: &mut TcpStream) -> Result<()> {
    let pid = std::process::id();
    let msg = format!("SET_FRONTEND_PID:{}\n", pid);
    stream.write_all(msg.as_bytes()).await?;
    stream.flush().await?;
    debug!("Sent frontend PID: {}", pid);
    Ok(())
}

async fn send_direct_handshake(
    stream: &mut TcpStream,
    ticket: &eaccess::LaunchTicket,
) -> Result<()> {
    let key = ticket.key.trim();
    stream.write_all(key.as_bytes()).await?;
    stream.write_all(b"\n").await?;

    let fe_string = format!(
        "/FE:WIZARD /VERSION:1.0.1.22 /P:{} /XML",
        std::env::consts::OS
    );
    stream.write_all(fe_string.as_bytes()).await?;
    stream.write_all(b"\n").await?;
    stream.flush().await?;

    for _ in 0..2 {
        stream.write_all(b"<c>\n").await?;
        stream.flush().await?;
        sleep(Duration::from_millis(300)).await;
    }

    Ok(())
}

fn fix_game_host_port(host: &str, port: u16) -> (String, u16) {
    let lowered = host.to_ascii_lowercase();
    match (lowered.as_str(), port) {
        ("gs-plat.simutronics.net", 10121) => ("storm.gs4.game.play.net".to_string(), 10124),
        ("gs3.simutronics.net", 4900) => ("storm.gs4.game.play.net".to_string(), 10024),
        ("gs4.simutronics.net", 10321) => ("storm.gs4.game.play.net".to_string(), 10324),
        ("prime.dr.game.play.net", 4901) => ("dr.simutronics.net".to_string(), 11024),
        _ => (host.to_string(), port),
    }
}

mod eaccess {
    use anyhow::{anyhow, bail, Context, Result};
    use openssl::ssl::{SslConnector, SslMethod, SslStream, SslVerifyMode};
    use openssl::x509::X509;
    use std::collections::HashMap;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::path::Path;

    const HOST: &str = "eaccess.play.net";
    const PORT: u16 = 7910;
    const CERT_FILENAME: &str = "simu.pem";

    #[derive(Clone, Debug)]
    pub struct LaunchTicket {
        pub key: String,
        pub game_host: String,
        pub game_port: u16,
        pub game: String,
        pub character: String,
    }

    pub fn authenticate(
        account: &str,
        password: &str,
        character: &str,
        game_code: &str,
        data_dir: &Path,
    ) -> Result<LaunchTicket> {
        let cert_path = data_dir.join(CERT_FILENAME);
        ensure_certificate(&cert_path)?;

        tracing::debug!("TLS handshake to eAccess starting (cert: {:?})", cert_path);
        let mut stream = match connect_with_cert(&cert_path) {
            Ok(stream) => {
                tracing::debug!("TLS handshake to eAccess succeeded");
                stream
            }
            Err(err) => {
                tracing::warn!(error = ?err, "Handshake failed, refreshing stored cert");
                download_certificate(&cert_path)?;
                let stream = connect_with_cert(&cert_path)?;
                tracing::debug!("TLS handshake succeeded after refreshing cert");
                stream
            }
        };

        send_line(&mut stream, "K")?;
        let hash_key = read_response(&mut stream)?;
        let encoded_password = obfuscate_password(password, hash_key.trim());

        send_login_payload(&mut stream, account, &encoded_password)?;
        let auth_response = read_response(&mut stream)?;

        if !auth_response.contains("KEY") {
            bail!(
                "Authentication failed for account {}: {}",
                account,
                auth_response.trim()
            );
        }

        send_line(&mut stream, "M")?;
        read_response(&mut stream)?; // Available games (unused)

        send_line(&mut stream, &format!("F\t{}", game_code))?;
        read_response(&mut stream)?; // Subscription tier
        send_line(&mut stream, &format!("G\t{}", game_code))?;
        read_response(&mut stream)?; // Game status
        send_line(&mut stream, &format!("P\t{}", game_code))?;
        read_response(&mut stream)?; // Billing info

        send_line(&mut stream, "C")?;
        let characters_response = read_response(&mut stream)?;
        let char_code = parse_character_code(&characters_response, character).ok_or_else(|| {
            anyhow!(
                "Character '{}' not found in account '{}'",
                character,
                account
            )
        })?;

        send_line(&mut stream, &format!("L\t{}\tSTORM", char_code))?;
        let launch_response = read_response(&mut stream)?;
        parse_launch_response(&launch_response)
    }

    fn ensure_certificate(path: &Path) -> Result<()> {
        if path.exists() {
            return Ok(());
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        download_certificate(path)
    }

    fn download_certificate(path: &Path) -> Result<()> {
        // Create permissive connector to download cert
        let mut connector = SslConnector::builder(SslMethod::tls_client())?;
        connector.set_verify(SslVerifyMode::NONE);

        let stream = TcpStream::connect((HOST, PORT))?;
        stream.set_nodelay(true)?;
        let connector = connector.build();
        let tls_stream = connector.connect(HOST, stream)?;

        // Get peer certificate and save as PEM
        let cert = tls_stream
            .ssl()
            .peer_certificate()
            .ok_or_else(|| anyhow!("Server did not provide a certificate"))?;

        let pem = cert.to_pem()?;
        fs::write(path, pem).context("Failed to save certificate")?;
        Ok(())
    }

    fn connect_with_cert(cert_path: &Path) -> Result<SslStream<TcpStream>> {
        let cert_data = fs::read(cert_path).context("Failed to read stored certificate")?;
        let stored_cert = X509::from_pem(&cert_data)
            .context("Invalid PEM certificate")?;

        // Create connector with the stored certificate
        // Configure like Ruby's OpenSSL - allow both TLS 1.2 and 1.3
        let mut connector = SslConnector::builder(SslMethod::tls_client())?;

        // Add our stored cert as a trusted root
        connector.cert_store_mut().add_cert(stored_cert.clone())?;

        // Let OpenSSL negotiate - server will pick TLS 1.2, but we need to offer 1.3 too
        // (Server expects clients to advertise TLS 1.3 cipher suites even if it uses 1.2)

        // Disable session caching to avoid sending a Session ID (match Lich's empty Session ID)
        use openssl::ssl::SslSessionCacheMode;
        connector.set_session_cache_mode(SslSessionCacheMode::OFF);

        // Disable automatic verification - we do manual verification below (like Lich)
        // The cert has no hostname, so automatic verification would fail
        connector.set_verify(SslVerifyMode::NONE);

        let connector = connector.build();
        let stream = TcpStream::connect((HOST, PORT)).context("Failed to open TLS socket")?;
        stream.set_nodelay(true)?;

        // Disable SNI - Ruby doesn't send it by default for IP-based connections
        let mut config = connector.configure()?;
        config.set_use_server_name_indication(false);
        config.set_verify_hostname(false);

        let tls_stream = config
            .connect("", stream)
            .context("TLS handshake with eAccess failed")?;

        // Log TLS details
        tracing::debug!("TLS version: {:?}, Cipher: {:?}",
            tls_stream.ssl().version_str(),
            tls_stream.ssl().current_cipher().map(|c| c.name()));

        // Manually verify the peer certificate matches our stored one (like Lich's verify_pem)
        let peer_cert = tls_stream
            .ssl()
            .peer_certificate()
            .ok_or_else(|| anyhow!("Server did not provide a certificate"))?;

        let peer_pem = peer_cert.to_pem()?;
        if peer_pem != cert_data {
            tracing::warn!("Certificate mismatch - refreshing stored certificate");
            download_certificate(cert_path)?;
        }

        Ok(tls_stream)
    }

    fn send_line(stream: &mut SslStream<TcpStream>, line: &str) -> Result<()> {
        // Match Ruby's puts - sends string with newline in a SINGLE write
        // Build the complete message with newline, then write it all at once
        // to ensure it goes out as a single TLS record
        let mut message = Vec::with_capacity(line.len() + 1);
        message.extend_from_slice(line.as_bytes());
        message.push(b'\n');

        stream.write_all(&message)?;
        stream.flush()?;
        Ok(())
    }

    fn send_login_payload(
        stream: &mut SslStream<TcpStream>,
        account: &str,
        encoded_password: &[u8],
    ) -> Result<()> {
        // Build entire payload in memory first, then send as single write
        // to ensure it goes out as a single TLS record
        let mut payload = Vec::new();
        payload.extend_from_slice(b"A\t");
        payload.extend_from_slice(account.as_bytes());
        payload.extend_from_slice(b"\t");
        payload.extend_from_slice(encoded_password);
        payload.extend_from_slice(b"\n");

        stream.write_all(&payload)?;
        stream.flush()?;
        Ok(())
    }

    fn read_response(stream: &mut SslStream<TcpStream>) -> Result<String> {
        // Match Ruby's conn.sysread(PACKET_SIZE) behavior - read up to 8192 bytes in one blocking call
        const PACKET_SIZE: usize = 8192;
        let mut buf = vec![0u8; PACKET_SIZE];

        let bytes_read = stream.read(&mut buf)?;

        if bytes_read == 0 {
            return Ok(String::new());
        }

        // Truncate to actual bytes read
        buf.truncate(bytes_read);

        let response = String::from_utf8(buf).context("Response was not valid UTF-8")?;
        Ok(response)
    }

    fn obfuscate_password(password: &str, hash_key: &str) -> Vec<u8> {
        password
            .bytes()
            .zip(hash_key.bytes())
            .map(|(pwd, hash)| {
                // Match Ruby's behavior: ((pwd - 32) ^ hash) + 32
                // where the subtraction can go negative
                let pwd_adjusted = (pwd as i32) - 32;
                let xor_result = pwd_adjusted ^ (hash as i32);
                let final_result = xor_result + 32;
                final_result as u8
            })
            .collect()
    }

    fn parse_character_code(response: &str, target: &str) -> Option<String> {
        let trimmed = response.trim();
        let tokens: Vec<&str> = trimmed.split('\t').collect();
        if tokens.len() <= 5 || tokens.first().copied()? != "C" {
            return None;
        }
        let mut index = 5;
        while index + 1 < tokens.len() {
            let code = tokens[index];
            let name = tokens[index + 1];
            if name.eq_ignore_ascii_case(target) {
                return Some(code.to_string());
            }
            index += 2;
        }
        None
    }

    fn parse_launch_response(response: &str) -> Result<LaunchTicket> {
        let trimmed = response.trim();
        if !trimmed.starts_with('L') {
            bail!("Unexpected response to launch command: {}", trimmed);
        }

        let payload = trimmed
            .strip_prefix("L\t")
            .unwrap_or(trimmed)
            .strip_prefix("OK\t")
            .unwrap_or(trimmed);

        let mut values = HashMap::new();
        for pair in payload.split('\t') {
            if let Some((key, value)) = pair.split_once('=') {
                values.insert(key.to_uppercase(), value.to_string());
            }
        }

        let key = values
            .remove("KEY")
            .context("Launch response missing KEY")?;
        let host = values
            .remove("GAMEHOST")
            .context("Launch response missing GAMEHOST")?;
        let port = values
            .remove("GAMEPORT")
            .context("Launch response missing GAMEPORT")?
            .parse::<u16>()
            .context("Invalid GAMEPORT value")?;
        let game = values.get("GAME").cloned().unwrap_or_default();
        let character = values
            .get("CHARACTER")
            .cloned()
            .unwrap_or_else(|| "unknown".to_string());

        Ok(LaunchTicket {
            key,
            game_host: host,
            game_port: port,
            game,
            character,
        })
    }
}
