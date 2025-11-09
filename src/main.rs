mod app;
mod validator;
mod cmdlist;
mod config;
mod core;
mod frontend;
mod map_data;
mod network;
mod parser;
mod performance;
mod selection;
mod sound;
mod ui;
mod widget_state;
mod widgets;

use anyhow::Result;
use app::App;
use clap::Parser;
use config::Config;
use std::fs::OpenOptions;
use tracing_subscriber;

/// VellumFE - A modern, high-performance terminal frontend for GemStone IV
#[derive(Parser, Debug)]
#[command(name = "vellum-fe")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to connect to (Lich detached mode port)
    #[arg(short, long, default_value = "8000")]
    port: u16,

    /// Character name / config file to load (loads ./config/<character>.toml or default.toml)
    #[arg(short, long)]
    character: Option<String>,

    /// Enable link highlighting (required for proper game feed with clickable links)
    #[arg(long, default_value = "false")]
    links: bool,

    /// Disable startup music on connection
    #[arg(long, default_value = "false")]
    nomusic: bool,

    /// Validate a layout file against multiple sizes and exit
    #[arg(long, value_name = "PATH", required = false)]
    validate_layout: Option<String>,

    /// Baseline terminal size for validation (e.g., 120x40). Defaults to layout's designed size or 120x40.
    #[arg(long, value_name = "WxH", required = false)]
    baseline: Option<String>,

    /// Comma-separated list of sizes to test (e.g., 80x24,100x30,140x40)
    #[arg(long, value_name = "WxH[,WxH...]", required = false)]
    sizes: Option<String>,

    /// Use experimental refactored architecture (AppCore + TuiFrontend)
    #[arg(long, default_value = "false")]
    experimental: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command-line arguments
    let args = Args::parse();

    // Initialize logging to character-specific file instead of stderr to not mess up TUI
    let log_file = Config::get_log_path(args.character.as_deref())?;

    if let Some(parent) = log_file.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)?;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(file)
        .with_ansi(false)
        .init();

    // Load configuration (with character override if specified)
    let config = Config::load_with_options(args.character.as_deref(), args.port)?;

    // Layout validation mode
    if let Some(layout_path) = args.validate_layout.as_ref() {
        // Parse sizes
        let sizes = parse_sizes_arg(args.sizes.as_deref());

        // Determine baseline
        let baseline = if let Some(b) = args.baseline.as_deref() {
            parse_size(b).unwrap_or((120, 40))
        } else {
            // Try to load layout to read designed size; else default
            let lp = std::path::Path::new(layout_path);
            let layout = config::Layout::load_from_file(lp).ok();
            if let Some(l) = layout {
                let w = l.terminal_width.unwrap_or(120);
                let h = l.terminal_height.unwrap_or(40);
                (w, h)
            } else {
                (120, 40)
            }
        };

        let results = crate::validator::validate_layout_path(std::path::Path::new(layout_path), baseline, &sizes)?;
        let mut total_errors = 0usize;
        println!("Layout validation for {} (baseline {}x{}):", layout_path, baseline.0, baseline.1);
        for r in &results {
            if r.issues.is_empty() {
                println!("- {}x{}: OK", r.width, r.height);
            } else {
                println!("- {}x{}:", r.width, r.height);
                for issue in &r.issues {
                    let kind = match issue.kind { crate::validator::IssueKind::Error => "ERR", crate::validator::IssueKind::Warning => "WARN" };
                    println!("    {} [{}] {}", kind, issue.window, issue.message);
                    if matches!(issue.kind, crate::validator::IssueKind::Error) { total_errors += 1; }
                }
            }
        }
        if total_errors > 0 { std::process::exit(2); } else { return Ok(()); }
    }

    // Create and run the application
    if args.experimental {
        tracing::info!("🧪 Running in EXPERIMENTAL mode (new architecture)");
        run_experimental(config, args.nomusic).await?;
    } else {
        tracing::info!("Running in STABLE mode (old App)");
        let mut app = App::new(config, args.nomusic)?;
        app.check_and_auto_resize()?;
        app.run().await?;
    }

    Ok(())
}

/// Experimental main loop using new architecture (AppCore + TuiFrontend)
///
/// This is the new refactored code path that separates business logic from rendering.
/// Eventually this will replace App::run() entirely.
async fn run_experimental(config: Config, nomusic: bool) -> Result<()> {
    use frontend::{Frontend, TuiFrontend};
    use core::AppCore;
    use network::LichConnection;
    use tracing::info;

    info!("Initializing App (for now, to leverage existing initialization)...");
    let app = App::new(config.clone(), nomusic)?;

    info!("Extracting AppCore from App...");
    let mut core = AppCore::from_app(&app);

    info!("Initializing TUI frontend...");
    let mut frontend = TuiFrontend::new()?;

    // Connect to Lich server
    info!("Connecting to {}:{}", config.connection.host, config.connection.port);

    // Create channels for server communication
    let (server_tx, mut server_rx) = tokio::sync::mpsc::unbounded_channel();
    let (command_tx, command_rx) = tokio::sync::mpsc::unbounded_channel();

    // Spawn connection task
    let host = config.connection.host.clone();
    let port = config.connection.port;
    tokio::spawn(async move {
        if let Err(e) = LichConnection::start(&host, port, server_tx, command_rx).await {
            tracing::error!("Connection error: {}", e);
        }
    });

    info!("Starting main event loop...");

    // Track last countdown update for periodic timer rendering
    let mut last_countdown_update = std::time::Instant::now();
    let countdown_update_interval = std::time::Duration::from_millis(1000); // Update countdown timers every second

    // Event loop with frame rate limiting
    loop {
        // Render first (like stable VellumFE) - shows previous frame while processing new data
        // This creates smooth incremental updates during message floods
        if core.needs_render {
            frontend.render(&mut core as &mut dyn std::any::Any)?;
            core.needs_render = false;
        }

        // Poll events (use config poll_timeout for frame limiting)
        let events = frontend.poll_events()?;

        // Check if countdown timers need updating
        let now = std::time::Instant::now();
        if now.duration_since(last_countdown_update) >= countdown_update_interval {
            core.needs_render = true; // Force render for countdown timer updates
            last_countdown_update = now;
        }

        for event in events {
            match event {
                frontend::FrontendEvent::Quit => {
                    info!("Quit event received");
                    core.running = false;
                    core.needs_render = true;
                }
                frontend::FrontendEvent::Key { code, .. } => {
                    tracing::debug!("Key event: {:?}", code);

                    // Handle keyboard input for command line
                    if let Some(command) = core.handle_key_input(code) {
                        tracing::info!("Command entered: {}", command);

                        // Process command (dot commands handled locally, game commands sent to server)
                        if let Some(game_command) = core.process_command(command) {
                            // Send to server
                            if let Err(e) = command_tx.send(game_command.clone()) {
                                tracing::error!("Failed to send command: {}", e);
                                core.add_system_message(&format!("Failed to send: {}", e));
                            } else {
                                tracing::debug!("Sent command to server: {}", game_command);
                            }
                        }
                    }
                }
                frontend::FrontendEvent::Resize { width, height } => {
                    tracing::debug!("Resize event: {}x{}", width, height);
                    core.needs_render = true;
                }
                _ => {}
            }
        }

        // Handle server messages (non-blocking) - process ALL available messages
        // Render happens at top of next loop iteration, creating smooth incremental display
        while let Ok(msg) = server_rx.try_recv() {
            if let Err(e) = core.handle_server_message(msg) {
                tracing::error!("Error handling server message: {}", e);
            }
        }

        // Exit if not running
        if !core.running {
            break;
        }
    }

    info!("Cleaning up frontend...");
    frontend.cleanup()?;

    info!("Experimental mode completed successfully!");
    Ok(())
}

fn parse_sizes_arg(arg: Option<&str>) -> Vec<(u16, u16)> {
    let default = vec![(80, 24), (100, 30), (120, 40), (140, 40), (160, 50)];
    match arg {
        None => default,
        Some(s) if s.trim().is_empty() => default,
        Some(s) => s.split(',').filter_map(|p| parse_size(p.trim())).collect::<Vec<_>>()
    }
}

fn parse_size(s: &str) -> Option<(u16, u16)> {
    let (w, h) = s.split_once('x')?;
    let w = w.parse::<u16>().ok()?;
    let h = h.parse::<u16>().ok()?;
    Some((w, h))
}
