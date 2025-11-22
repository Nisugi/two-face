//! Two-Face - Multi-frontend GemStone IV client
//!
//! Supports both TUI (ratatui) and GUI (egui) frontends with shared core logic.

mod clipboard;
mod cmdlist;
mod config;
mod core;
mod data;
mod frontend;
mod network;
mod parser;
mod performance;
mod selection;
mod sound;
mod theme;
mod tts;

use anyhow::{bail, Context, Result};
use clap::{Parser as ClapParser, Subcommand};
use frontend::Frontend;
use std::path::PathBuf;

#[derive(ClapParser)]
#[command(name = "two-face")]
#[command(about = "Multi-frontend GemStone IV client", long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Frontend to use
    #[arg(short, long, default_value = "tui")]
    frontend: FrontendType,

    /// Port number to connect to (default: 8000)
    #[arg(short, long)]
    port: Option<u16>,

    /// Character name for loading character-specific settings
    #[arg(long)]
    character: Option<String>,

    /// Custom data directory (default: ~/.two-face)
    /// Can also be set via TWO_FACE_DIR environment variable
    #[arg(long, value_name = "DIR")]
    data_dir: Option<PathBuf>,

    /// Connect directly without Lich
    #[arg(long)]
    direct: bool,

    /// Account name for direct connections
    #[arg(long, requires = "direct")]
    direct_account: Option<String>,

    /// Password for direct connections (omit to be prompted securely)
    #[arg(long, requires = "direct")]
    direct_password: Option<String>,

    /// Game world for direct connections (prime, platinum, shattered)
    #[arg(long, value_enum, requires = "direct")]
    direct_game: Option<DirectGameArg>,

    /// Character name for direct connections (falls back to --character or config)
    #[arg(long, requires = "direct")]
    direct_character: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum FrontendType {
    Tui,
    Gui,
}

#[derive(Clone, Copy, Debug, clap::ValueEnum)]
enum DirectGameArg {
    Prime,
    Platinum,
    Shattered,
}

impl DirectGameArg {
    fn code(self) -> &'static str {
        match self {
            DirectGameArg::Prime => "GS3",
            DirectGameArg::Platinum => "GSX",
            DirectGameArg::Shattered => "GSF",
        }
    }
}

fn build_direct_config(
    cli: &Cli,
    config: &config::Config,
) -> Result<Option<network::DirectConnectConfig>> {
    if !cli.direct {
        return Ok(None);
    }

    let account = cli
        .direct_account
        .as_ref()
        .cloned()
        .context("`--direct-account` is required when using --direct")?;

    let password = match &cli.direct_password {
        Some(pwd) => pwd.clone(),
        None => {
            let prompt = format!("Password for account {}: ", account);
            rpassword::prompt_password(prompt).context("Failed to read password")?
        }
    };

    let character = cli
        .direct_character
        .as_ref()
        .cloned()
        .or_else(|| cli.character.clone())
        .or_else(|| config.connection.character.clone())
        .context(
            "Specify --direct-character, --character, or set connection.character in config for direct mode",
        )?;

    let game_code = cli
        .direct_game
        .unwrap_or(DirectGameArg::Prime)
        .code()
        .to_string();

    let data_dir = config::Config::base_dir()?;

    Ok(Some(network::DirectConnectConfig {
        account,
        password,
        character,
        game_code,
        data_dir,
    }))
}

/// Convert KeyCode + KeyModifiers to a string format matching the keybind HashMap
///
/// NOTE: Crossterm doesn't distinguish numpad from regular number keys.
/// So we format keys as-is. Numpad-specific keybinds (num_0, etc.) won't match.
fn format_key_for_keybind(
    code: crossterm::event::KeyCode,
    modifiers: crossterm::event::KeyModifiers,
) -> String {
    use crossterm::event::KeyCode;

    let base_key = match code {
        KeyCode::Char(c) => {
            // Return character as-is (don't map to numpad format)
            if modifiers.is_empty() {
                return c.to_string();
            } else {
                return format!("{}+{}", format_modifiers(modifiers), c);
            }
        }
        KeyCode::Enter => "enter",
        KeyCode::Esc => "esc",
        KeyCode::Tab => "tab",
        KeyCode::Backspace => "backspace",
        KeyCode::Delete => "delete",
        KeyCode::Up => "up",
        KeyCode::Down => "down",
        KeyCode::Left => "left",
        KeyCode::Right => "right",
        KeyCode::Home => "home",
        KeyCode::End => "end",
        KeyCode::PageUp => "pageup",
        KeyCode::PageDown => "pagedown",
        KeyCode::F(n) => return format!("f{}", n),
        _ => return String::new(), // Unhandled key
    };

    // Add modifiers if present
    if modifiers.is_empty() {
        base_key.to_string()
    } else {
        format!("{}+{}", format_modifiers(modifiers), base_key)
    }
}

/// Format modifiers as a string (helper for format_key_for_keybind)
fn format_modifiers(modifiers: crossterm::event::KeyModifiers) -> String {
    use crossterm::event::KeyModifiers;
    let mut parts = Vec::new();

    if modifiers.contains(KeyModifiers::CONTROL) {
        parts.push("ctrl");
    }
    if modifiers.contains(KeyModifiers::ALT) {
        parts.push("alt");
    }
    if modifiers.contains(KeyModifiers::SHIFT) {
        parts.push("shift");
    }

    parts.join("+")
}

/// Build windows submenu
fn build_windows_submenu(_app_core: &core::AppCore) -> Vec<data::ui_state::PopupMenuItem> {
    let items = vec![
        data::ui_state::PopupMenuItem {
            text: "Add Window...".to_string(),
            command: "action:addwindow".to_string(),
            disabled: false,
        },
        data::ui_state::PopupMenuItem {
            text: "Hide Window...".to_string(),
            command: "action:hidewindow".to_string(),
            disabled: false,
        },
        data::ui_state::PopupMenuItem {
            text: "Edit Window...".to_string(),
            command: "action:editwindow".to_string(),
            disabled: false,
        },
        data::ui_state::PopupMenuItem {
            text: "List Windows".to_string(),
            command: "action:listwindows".to_string(),
            disabled: false,
        },
    ];
    items
}

/// Build window picker for editing (shows only visible windows)
fn build_window_picker(app_core: &core::AppCore) -> Vec<data::ui_state::PopupMenuItem> {
    let mut items = Vec::new();

    // Collect visible window names
    let mut visible_names: Vec<String> = app_core
        .ui_state
        .windows
        .keys()
        .map(|name| name.to_string())
        .collect();

    // Sort alphabetically by display name
    visible_names.sort_by_key(|name| get_window_display_name(name));

    for name in visible_names {
        let display_name = get_window_display_name(&name);
        items.push(data::ui_state::PopupMenuItem {
            text: display_name,
            command: format!("action:editwindow:{}", name),
            disabled: false,
        });
    }

    if items.is_empty() {
        items.push(data::ui_state::PopupMenuItem {
            text: "No visible windows to edit".to_string(),
            command: String::new(),
            disabled: true,
        });
    }
    items
}

/// Build configuration submenu
fn build_config_submenu() -> Vec<data::ui_state::PopupMenuItem> {
    vec![
        data::ui_state::PopupMenuItem {
            text: "Layouts".to_string(),
            command: "menu:layouts".to_string(),
            disabled: false,
        },
        data::ui_state::PopupMenuItem {
            text: "Highlights".to_string(),
            command: "action:highlights".to_string(),
            disabled: false,
        },
    ]
}

/// Build settings items from config
fn build_settings_items(
    config: &config::Config,
) -> Vec<frontend::tui::settings_editor::SettingItem> {
    use frontend::tui::settings_editor::{SettingItem, SettingValue};

    let mut items = Vec::new();

    // Connection settings
    items.push(SettingItem {
        category: "Connection".to_string(),
        key: "connection.host".to_string(),
        display_name: "Host".to_string(),
        value: SettingValue::String(config.connection.host.clone()),
        description: Some("Game server hostname or IP address".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "Connection".to_string(),
        key: "connection.port".to_string(),
        display_name: "Port".to_string(),
        value: SettingValue::Number(config.connection.port as i64),
        description: Some("Game server port number".to_string()),
        editable: true,
        name_width: None,
    });

    if let Some(ref character) = config.connection.character {
        items.push(SettingItem {
            category: "Connection".to_string(),
            key: "connection.character".to_string(),
            display_name: "Character".to_string(),
            value: SettingValue::String(character.clone()),
            description: Some("Default character name".to_string()),
            editable: true,
            name_width: None,
        });
    }

    // UI settings
    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.buffer_size".to_string(),
        display_name: "Buffer Size".to_string(),
        value: SettingValue::Number(config.ui.buffer_size as i64),
        description: Some("Number of lines to keep in text window buffers".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.show_timestamps".to_string(),
        display_name: "Show Timestamps".to_string(),
        value: SettingValue::Boolean(config.ui.show_timestamps),
        description: Some("Display timestamps in text windows".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.border_style".to_string(),
        display_name: "Border Style".to_string(),
        value: SettingValue::Enum(
            config.ui.border_style.clone(),
            vec![
                "single".to_string(),
                "double".to_string(),
                "rounded".to_string(),
                "thick".to_string(),
                "none".to_string(),
            ],
        ),
        description: Some("Widget border style".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.countdown_icon".to_string(),
        display_name: "Countdown Icon".to_string(),
        value: SettingValue::String(config.ui.countdown_icon.clone()),
        description: Some("Unicode character for countdown blocks".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.poll_timeout_ms".to_string(),
        display_name: "Poll Timeout (ms)".to_string(),
        value: SettingValue::Number(config.ui.poll_timeout_ms as i64),
        description: Some("Event poll timeout - lower = higher FPS, higher CPU".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.startup_music".to_string(),
        display_name: "Startup Music".to_string(),
        value: SettingValue::Boolean(config.ui.startup_music),
        description: Some("Play music when connecting".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.startup_music_file".to_string(),
        display_name: "Startup Music File".to_string(),
        value: SettingValue::String(config.ui.startup_music_file.clone()),
        description: Some("Sound file to play on startup (without extension)".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.selection_enabled".to_string(),
        display_name: "Selection Enabled".to_string(),
        value: SettingValue::Boolean(config.ui.selection_enabled),
        description: Some("Enable text selection with mouse".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.selection_respect_window_boundaries".to_string(),
        display_name: "Selection Respects Windows".to_string(),
        value: SettingValue::Boolean(config.ui.selection_respect_window_boundaries),
        description: Some("Prevent selection from crossing window boundaries".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.drag_modifier_key".to_string(),
        display_name: "Drag Modifier Key".to_string(),
        value: SettingValue::Enum(
            config.ui.drag_modifier_key.clone(),
            vec!["ctrl".to_string(), "alt".to_string(), "shift".to_string()],
        ),
        description: Some("Modifier key required for drag and drop".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "UI".to_string(),
        key: "ui.min_command_length".to_string(),
        display_name: "Min Command Length".to_string(),
        value: SettingValue::Number(config.ui.min_command_length as i64),
        description: Some("Minimum command length to save to history".to_string()),
        editable: true,
        name_width: None,
    });

    // Sound settings
    items.push(SettingItem {
        category: "Sound".to_string(),
        key: "sound.enabled".to_string(),
        display_name: "Sound Enabled".to_string(),
        value: SettingValue::Boolean(config.sound.enabled),
        description: Some("Enable sound effects".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "Sound".to_string(),
        key: "sound.volume".to_string(),
        display_name: "Master Volume".to_string(),
        value: SettingValue::Float(config.sound.volume as f64),
        description: Some("Master volume (0.0 to 1.0)".to_string()),
        editable: true,
        name_width: None,
    });

    items.push(SettingItem {
        category: "Sound".to_string(),
        key: "sound.cooldown_ms".to_string(),
        display_name: "Sound Cooldown (ms)".to_string(),
        value: SettingValue::Number(config.sound.cooldown_ms as i64),
        description: Some("Cooldown between same sound plays".to_string()),
        editable: true,
        name_width: None,
    });

    // Theme settings
    items.push(SettingItem {
        category: "Theme".to_string(),
        key: "active_theme".to_string(),
        display_name: "Active Theme".to_string(),
        value: SettingValue::String(config.active_theme.clone()),
        description: Some("Currently active color theme".to_string()),
        editable: true,
        name_width: None,
    });

    items
}

/// Build layouts submenu
fn build_layouts_submenu() -> Vec<data::ui_state::PopupMenuItem> {
    let mut items = Vec::new();

    // Get list of saved layouts
    match config::Config::list_layouts() {
        Ok(layouts) => {
            for layout_name in layouts {
                items.push(data::ui_state::PopupMenuItem {
                    text: layout_name.clone(),
                    command: format!("action:loadlayout:{}", layout_name),
                    disabled: false,
                });
            }
        }
        Err(e) => {
            tracing::warn!("Failed to list layouts: {}", e);
            items.push(data::ui_state::PopupMenuItem {
                text: "No layouts found".to_string(),
                command: String::new(),
                disabled: true,
            });
        }
    }

    items
}

/// Build widget type picker menu (shows hidden templates that can be shown)
/// Get clean display name for a window
fn get_window_display_name(name: &str) -> String {
    match name {
        "encumlevel" => "encumbrance".to_string(),
        "pbarStance" => "stance".to_string(),
        "mindState" => "mind".to_string(),
        "lblBPs" => "blood".to_string(),
        "active_spells" => "active spells".to_string(),
        "left_hand" => "left hand".to_string(),
        "right_hand" => "right hand".to_string(),
        "spell_hand" => "spell hand".to_string(),
        _ => name.to_string(),
    }
}

fn build_widget_picker(_app_core: &core::AppCore) -> Vec<data::ui_state::PopupMenuItem> {
    // Return category menu items
    vec![
        data::ui_state::PopupMenuItem {
            text: "Countdown".to_string(),
            command: "action:addwindow:countdown".to_string(),
            disabled: false,
        },
        data::ui_state::PopupMenuItem {
            text: "Hand".to_string(),
            command: "action:addwindow:hand".to_string(),
            disabled: false,
        },
        data::ui_state::PopupMenuItem {
            text: "Other".to_string(),
            command: "action:addwindow:other".to_string(),
            disabled: false,
        },
        data::ui_state::PopupMenuItem {
            text: "Progress Bar".to_string(),
            command: "action:addwindow:progressbar".to_string(),
            disabled: false,
        },
        data::ui_state::PopupMenuItem {
            text: "Text".to_string(),
            command: "action:addwindow:text".to_string(),
            disabled: false,
        },
    ]
}

/// Build window list for a specific category
fn build_widget_category_picker(app_core: &core::AppCore, category: &str) -> Vec<data::ui_state::PopupMenuItem> {
    let mut items = Vec::new();

    // Define which windows belong to each category
    let category_windows: Vec<&str> = match category {
        "countdown" => vec!["roundtime", "casttime", "stuntime"],
        "hand" => vec!["left_hand", "right_hand", "spell_hand"],
        "other" => vec!["compass", "inventory", "room", "spells", "injuries", "spacer"],
        "progressbar" => vec!["health", "mana", "stamina", "spirit", "encumlevel", "pbarStance", "mindState", "lblBPs"],
        "text" => vec!["thoughts", "speech", "announcements", "loot", "death", "logons", "familiar", "ambients", "bounty"],
        _ => vec![],
    };

    // Filter out windows that are already visible
    for template_name in category_windows {
        // Special case: spacer is always createable (not a singleton window)
        let is_spacer = template_name == "spacer";

        // Skip if this window is already visible in ui_state (unless it's spacer)
        if !is_spacer && app_core.ui_state.windows.contains_key(template_name) {
            continue;
        }

        // Add to menu
        let display_name = get_window_display_name(template_name);
        let command = if is_spacer {
            // Spacers are created (not shown from layout)
            format!("action:createwindow:{}", template_name)
        } else {
            // Other windows are shown from layout
            format!("action:showwindow:{}", template_name)
        };

        items.push(data::ui_state::PopupMenuItem {
            text: display_name.clone(),
            command,
            disabled: false,
        });
    }

    // If no available windows in this category
    if items.is_empty() {
        items.push(data::ui_state::PopupMenuItem {
            text: "All windows in this category are visible".to_string(),
            command: String::new(),
            disabled: true,
        });
    }

    items
}

/// Build hide window menu (shows currently visible windows that can be hidden)
fn build_hidewindow_picker(app_core: &core::AppCore) -> Vec<data::ui_state::PopupMenuItem> {
    let mut items = Vec::new();

    // Get all currently visible window names from ui_state (except main and command_input)
    let mut visible_names: Vec<String> = app_core
        .ui_state
        .windows
        .keys()
        .filter(|name| *name != "main" && *name != "command_input")
        .map(|name| name.to_string())
        .collect();

    // Sort alphabetically by display name
    visible_names.sort_by_key(|name| get_window_display_name(name));

    for name in visible_names {
        let display_name = get_window_display_name(&name);
        items.push(data::ui_state::PopupMenuItem {
            text: display_name,
            command: format!("action:hidewindow:{}", name),
            disabled: false,
        });
    }

    // If no windows can be hidden
    if items.is_empty() {
        items.push(data::ui_state::PopupMenuItem {
            text: "No windows to hide".to_string(),
            command: String::new(),
            disabled: true,
        });
    }

    items
}

/// Handle menu action commands
fn handle_menu_action(
    app_core: &mut core::AppCore,
    frontend: &mut frontend::tui::TuiFrontend,
    command: &str,
) -> Result<()> {
    if command.starts_with("action:loadlayout:") {
        // Load a layout with proper terminal size
        let layout_name = &command[18..];
        tracing::info!("[MAIN.RS] Menu action loadlayout: '{}'", layout_name);
        let (width, height) = frontend.size();
        tracing::info!(
            "[MAIN.RS] Terminal size from frontend: {}x{}",
            width,
            height
        );
        if let Some((theme_id, theme)) = app_core.load_layout(layout_name, width, height) {
            frontend.update_theme_cache(theme_id, theme);
        }
    } else if command.starts_with("action:createwindow:") {
        // Create a new window with the specified widget type
        let widget_type = &command[20..];

        // Get template for this widget type (use widget type name as template name)
        if let Some(template) = config::Config::get_window_template(widget_type) {
            // Open window editor with template (proper defaults + marked as new)
            frontend.window_editor =
                Some(frontend::tui::window_editor::WindowEditor::new_from_template(template));
            app_core.ui_state.input_mode = data::ui_state::InputMode::WindowEditor;
        } else {
            tracing::warn!("No template found for widget type: {}", widget_type);
        }
    } else if command.starts_with("action:editwindow:") {
        // Edit an existing window
        let window_name = &command[18..];

        // Find the window definition
        if let Some(window_def) = app_core
            .layout
            .windows
            .iter()
            .find(|w| w.name() == window_name)
            .cloned()
        {
            // Open window editor
            frontend.window_editor =
                Some(frontend::tui::window_editor::WindowEditor::new(window_def));
            app_core.ui_state.input_mode = data::ui_state::InputMode::WindowEditor;
        } else {
            tracing::warn!("Window not found for editing: {}", window_name);
        }
    } else if command.starts_with("action:showwindow:") {
        // Add/show the window (from template)
        let window_name = &command[18..];

        // Get terminal size for window positioning
        let (width, height) = frontend.size();

        // Show window from layout template
        app_core.show_window(window_name, width, height);

        // Close menus
        app_core.ui_state.popup_menu = None;
        app_core.ui_state.submenu = None;
        app_core.ui_state.input_mode = data::ui_state::InputMode::Normal;
        app_core.needs_render = true;
    } else if command.starts_with("action:addwindow:") {
        // Show submenu with windows for this category
        let category = &command[17..];

        // Close main menu, show submenu with windows in this category
        app_core.ui_state.popup_menu = None;
        app_core.ui_state.submenu = Some(data::ui_state::PopupMenu::new(
            build_widget_category_picker(app_core, category),
            (40, 12),
        ));
        app_core.ui_state.input_mode = data::ui_state::InputMode::Menu;
    } else if command.starts_with("action:hidewindow:") {
        // Hide a visible window
        let window_name = &command[18..];
        app_core.hide_window(window_name);
    } else {
        match command {
            "action:addwindow" => {
                // Close submenu if it exists
                app_core.ui_state.submenu = None;
                // Show widget type picker
                app_core.ui_state.popup_menu = Some(data::ui_state::PopupMenu::new(
                    build_widget_picker(app_core),
                    (40, 12),
                ));
                // Stay in Menu mode
                app_core.ui_state.input_mode = data::ui_state::InputMode::Menu;
            }
            "action:hidewindow" => {
                // Close submenu if it exists
                app_core.ui_state.submenu = None;
                // Show window picker for hiding
                app_core.ui_state.popup_menu = Some(data::ui_state::PopupMenu::new(
                    build_hidewindow_picker(app_core),
                    (40, 12),
                ));
                // Stay in Menu mode
                app_core.ui_state.input_mode = data::ui_state::InputMode::Menu;
            }
            "action:listwindows" => {
                // List all windows
                app_core.send_command(".windows".to_string())?;
            }
            "action:highlights" => {
                // Open highlight browser
                frontend.highlight_browser =
                    Some(frontend::tui::highlight_browser::HighlightBrowser::new(
                        &app_core.config.highlights,
                    ));
                app_core.ui_state.input_mode = data::ui_state::InputMode::HighlightBrowser;
            }
            "action:addhighlight" => {
                // Open highlight form for creating new highlight
                frontend.highlight_form =
                    Some(frontend::tui::highlight_form::HighlightFormWidget::new());
                app_core.ui_state.input_mode = data::ui_state::InputMode::HighlightForm;
            }
            "action:keybinds" => {
                // Open keybind browser
                frontend.keybind_browser = Some(
                    frontend::tui::keybind_browser::KeybindBrowser::new(&app_core.config.keybinds),
                );
                app_core.ui_state.input_mode = data::ui_state::InputMode::KeybindBrowser;
            }
            "action:addkeybind" => {
                // Open keybind form for creating new keybind
                frontend.keybind_form = Some(frontend::tui::keybind_form::KeybindFormWidget::new());
                app_core.ui_state.input_mode = data::ui_state::InputMode::KeybindForm;
            }
            "action:colors" => {
                // Open color palette browser
                frontend.color_palette_browser = Some(
                    frontend::tui::color_palette_browser::ColorPaletteBrowser::new(
                        app_core.config.colors.color_palette.clone(),
                    ),
                );
                app_core.ui_state.input_mode = data::ui_state::InputMode::ColorPaletteBrowser;
            }
            "action:addcolor" => {
                // Open color form for creating new palette color
                frontend.color_form = Some(frontend::tui::color_form::ColorForm::new_create());
                app_core.ui_state.input_mode = data::ui_state::InputMode::ColorForm;
            }
            "action:uicolors" => {
                // Open UI colors browser
                frontend.uicolors_browser = Some(
                    frontend::tui::uicolors_browser::UIColorsBrowser::new(&app_core.config.colors),
                );
                app_core.ui_state.input_mode = data::ui_state::InputMode::UIColorsBrowser;
            }
            "action:spellcolors" => {
                // Open spell colors browser
                frontend.spell_color_browser =
                    Some(frontend::tui::spell_color_browser::SpellColorBrowser::new(
                        &app_core.config.colors.spell_colors,
                    ));
                app_core.ui_state.input_mode = data::ui_state::InputMode::SpellColorsBrowser;
            }
            "action:addspellcolor" => {
                // Open spell color form for creating new spell color
                frontend.spell_color_form =
                    Some(frontend::tui::spell_color_form::SpellColorFormWidget::new());
                app_core.ui_state.input_mode = data::ui_state::InputMode::SpellColorForm;
            }
            "action:settings" => {
                // Open settings editor
                let settings_items = build_settings_items(&app_core.config);
                frontend.settings_editor = Some(
                    frontend::tui::settings_editor::SettingsEditor::new(settings_items),
                );
                app_core.ui_state.input_mode = data::ui_state::InputMode::SettingsEditor;
            }
            "action:themes" => {
                // Open theme browser (includes built-in and custom themes)
                frontend.theme_browser = Some(frontend::tui::theme_browser::ThemeBrowser::new(
                    app_core.config.active_theme.clone(),
                    app_core.config.character.as_deref(),
                ));
                app_core.ui_state.input_mode = data::ui_state::InputMode::ThemeBrowser;
            }
            action if action.starts_with("action:settheme:") => {
                // Update frontend theme cache when theme changes via .settheme command
                let theme_id = action.strip_prefix("action:settheme:").unwrap().to_string();
                let theme = app_core.config.get_theme();
                frontend.update_theme_cache(theme_id, theme);
                app_core.needs_render = true;
            }
            "action:edittheme" => {
                // Open theme editor with current theme
                let current_theme = app_core.config.get_theme();
                frontend.theme_editor = Some(frontend::tui::theme_editor::ThemeEditor::new_edit(
                    &current_theme,
                ));
                app_core.ui_state.input_mode = data::ui_state::InputMode::ThemeEditor;
            }
            "action:editwindow" => {
                // Open window picker for editing
                let window_names: Vec<String> = app_core
                    .layout
                    .windows
                    .iter()
                    .map(|w| w.name().to_string())
                    .collect();

                let items: Vec<data::ui_state::PopupMenuItem> = window_names
                    .iter()
                    .map(|name| data::ui_state::PopupMenuItem {
                        text: name.clone(),
                        command: format!("action:editwindow:{}", name),
                        disabled: false,
                    })
                    .collect();

                // Close submenu if it exists
                app_core.ui_state.submenu = None;
                // Create new popup menu for window selection
                app_core.ui_state.popup_menu =
                    Some(data::ui_state::PopupMenu::new(items, (40, 12)));
                // Stay in Menu mode
                app_core.ui_state.input_mode = data::ui_state::InputMode::Menu;
            }
            "action:nexttab" => {
                // Navigate to next tab in all tabbed windows
                frontend.next_tab_all();
                app_core.needs_render = true;
            }
            "action:prevtab" => {
                // Navigate to previous tab in all tabbed windows
                frontend.prev_tab_all();
                app_core.needs_render = true;
            }
            "action:gonew" => {
                // Navigate to next tab with unread messages
                if !frontend.go_to_next_unread_tab() {
                    app_core.add_system_message("No tabs with new messages");
                }
                app_core.needs_render = true;
            }
            _ => {
                tracing::warn!("Unknown menu action: {}", command);
            }
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum Commands {
    /// Validate layout configuration
    ValidateLayout {
        /// Layout file to validate
        #[arg(value_name = "FILE")]
        layout: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    // Initialize logging to file (use RUST_LOG env var to control level, e.g. RUST_LOG=debug)
    // TUI apps can't log to stdout, so we write to a file
    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("two-face.log")?;

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug")),
        )
        .with_writer(std::sync::Mutex::new(log_file))
        .with_ansi(false) // No color codes in log file
        .init();

    // Parse CLI arguments
    let cli = Cli::parse();

    if cli.direct && matches!(cli.frontend, FrontendType::Gui) {
        bail!("Direct mode is currently only supported with the TUI frontend");
    }

    // Handle subcommands
    if let Some(command) = cli.command {
        match command {
            Commands::ValidateLayout { layout } => {
                // Load the layout file
                let layout_result = if let Some(path) = layout {
                    println!("Validating layout file: {:?}", path);
                    config::Layout::load_from_file(&path)
                } else {
                    println!("Validating default layout");
                    config::Layout::load(cli.character.as_deref())
                };

                match layout_result {
                    Ok(layout) => {
                        println!("✓ Layout loaded successfully");
                        println!("  {} windows defined", layout.windows.len());

                        // Basic validation checks
                        let mut errors = 0;
                        let mut warnings = 0;

                        for window in &layout.windows {
                            let name = window.name();
                            let base = window.base();

                            // Check for zero dimensions
                            if base.rows == 0 {
                                eprintln!("✗ Error: Window '{}' has zero height", name);
                                errors += 1;
                            }
                            if base.cols == 0 {
                                eprintln!("✗ Error: Window '{}' has zero width", name);
                                errors += 1;
                            }

                            // Check for empty names
                            if name.is_empty() {
                                eprintln!("✗ Error: Window has empty name");
                                errors += 1;
                            }

                            // Warn about very small windows
                            if base.rows == 1 && base.cols < 10 {
                                eprintln!(
                                    "⚠ Warning: Window '{}' is very small ({}x{})",
                                    name, base.cols, base.rows
                                );
                                warnings += 1;
                            }
                        }

                        // Summary
                        if errors == 0 && warnings == 0 {
                            println!("✓ Layout is valid with no issues");
                        } else {
                            if errors > 0 {
                                eprintln!("\n✗ Found {} error(s)", errors);
                            }
                            if warnings > 0 {
                                println!("⚠ Found {} warning(s)", warnings);
                            }
                        }

                        if errors > 0 {
                            std::process::exit(1);
                        }
                    }
                    Err(e) => {
                        eprintln!("✗ Failed to load layout: {}", e);
                        std::process::exit(1);
                    }
                }

                return Ok(());
            }
        }
    }

    // Set custom data directory if specified (via CLI or environment variable)
    if let Some(data_dir) = &cli.data_dir {
        std::env::set_var("TWO_FACE_DIR", data_dir);
        tracing::info!("Using custom data directory: {:?}", data_dir);
    } else if let Ok(env_dir) = std::env::var("TWO_FACE_DIR") {
        tracing::info!("Using data directory from TWO_FACE_DIR: {}", env_dir);
    }

    // Load configuration
    let port = cli.port.unwrap_or(8000);
    let character = cli.character.as_deref();
    let config = if let Some(config_path) = &cli.config {
        config::Config::load_from_path(config_path, character, port)?
    } else {
        config::Config::load_with_options(character, port)?
    };

    let direct_config = build_direct_config(&cli, &config)?;

    // Run appropriate frontend
    let character = cli.character.clone();
    match cli.frontend {
        FrontendType::Tui => run_tui(config, character, direct_config)?,
        FrontendType::Gui => run_gui(config)?,
    }

    Ok(())
}

/// Run TUI frontend
fn run_tui(
    config: config::Config,
    character: Option<String>,
    direct: Option<network::DirectConnectConfig>,
) -> Result<()> {
    // Use tokio runtime for async network I/O
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async_run_tui(config, character, direct))
}

/// Async TUI main loop with network support
async fn async_run_tui(
    config: config::Config,
    character: Option<String>,
    direct: Option<network::DirectConnectConfig>,
) -> Result<()> {
    use core::AppCore;
    use frontend::{Frontend, TuiFrontend};
    use network::{DirectConnection, LichConnection, ServerMessage};
    use tokio::sync::mpsc;

    // Create channels for network communication
    let (server_tx, mut server_rx) = mpsc::unbounded_channel::<ServerMessage>();
    let (command_tx, command_rx) = mpsc::unbounded_channel::<String>();

    // Store connection info
    let host = config.connection.host.clone();
    let port = config.connection.port;

    // Create core application state
    let mut app_core = AppCore::new(config)?;

    // Create TUI frontend
    let mut frontend = TuiFrontend::new()?;
    // Ensure frontend theme cache matches whatever layout/theme AppCore activated
    let initial_theme_id = app_core.config.active_theme.clone();
    let initial_theme = app_core.config.get_theme();
    frontend.update_theme_cache(initial_theme_id, initial_theme);

    // Initialize command input widget BEFORE any rendering
    // This ensures it exists when we start routing keys to it
    frontend.ensure_command_input_exists("command_input");

    // Load command history
    if let Err(e) = frontend.command_input_load_history("command_input", character.as_deref()) {
        tracing::warn!("Failed to load command history: {}", e);
    }

    // Get terminal size and initialize windows
    let (width, height) = frontend.size();
    app_core.init_windows(width, height);

    // Spawn network connection task
    let network_handle = match direct {
        Some(cfg) => tokio::spawn(async move {
            if let Err(e) = DirectConnection::start(cfg, server_tx, command_rx).await {
                tracing::error!(error = ?e, "Network connection error");
            }
        }),
        None => {
            let host_clone = host.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    LichConnection::start(&host_clone, port, server_tx, command_rx).await
                {
                    tracing::error!(error = ?e, "Network connection error");
                }
            })
        }
    };

    // Track time for periodic countdown updates
    let mut last_countdown_update = std::time::Instant::now();

    // Main event loop
    while app_core.running {
        // Poll for frontend events (keyboard, mouse, resize)
        let events = frontend.poll_events()?;

        // Poll TTS callback events for auto-play
        app_core.poll_tts_events();

        // Process frontend events
        for event in events {
            // Handle events that need frontend access directly
            match &event {
                frontend::FrontendEvent::Mouse {
                    kind,
                    x,
                    y,
                    modifiers,
                } => {
                    use crate::data::ui_state::InputMode;
                    use crossterm::event::{KeyModifiers, MouseEventKind};
                    use data::{DragOperation, LinkDragState, MouseDragState, PendingLinkClick};

                    // Create stable window index mapping (sorted by window name for consistency)
                    let mut window_names: Vec<&String> = app_core.ui_state.windows.keys().collect();
                    window_names.sort();
                    let window_index_map: std::collections::HashMap<&String, usize> = window_names
                        .iter()
                        .enumerate()
                        .map(|(idx, name)| (*name, idx))
                        .collect();

                    // Handle window editor mouse events first (if open)
                    if frontend.window_editor.is_some() {
                        let (width, height) = frontend.size();
                        let area = ratatui::layout::Rect {
                            x: 0,
                            y: 0,
                            width,
                            height,
                        };

                        if let Some(ref mut window_editor) = frontend.window_editor {
                            match kind {
                                MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                                    window_editor.handle_mouse(*x, *y, true, area);
                                    app_core.needs_render = true;
                                    continue;
                                }
                                MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                                    window_editor.handle_mouse(*x, *y, true, area);
                                    app_core.needs_render = true;
                                    continue;
                                }
                                MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                                    window_editor.handle_mouse(*x, *y, false, area);
                                    app_core.needs_render = true;
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }

                    match kind {
                        MouseEventKind::ScrollUp => {
                            // Find which window the mouse is over
                            let mut target_window = "main".to_string();
                            for (name, window) in &app_core.ui_state.windows {
                                let pos = &window.position;
                                if *x >= pos.x
                                    && *x < pos.x + pos.width
                                    && *y >= pos.y
                                    && *y < pos.y + pos.height
                                {
                                    target_window = name.clone();
                                    break;
                                }
                            }
                            frontend.scroll_window(&target_window, 10);
                            app_core.needs_render = true;
                            continue;
                        }
                        MouseEventKind::ScrollDown => {
                            // Find which window the mouse is over
                            let mut target_window = "main".to_string();
                            for (name, window) in &app_core.ui_state.windows {
                                let pos = &window.position;
                                if *x >= pos.x
                                    && *x < pos.x + pos.width
                                    && *y >= pos.y
                                    && *y < pos.y + pos.height
                                {
                                    target_window = name.clone();
                                    break;
                                }
                            }
                            frontend.scroll_window(&target_window, -10);
                            app_core.needs_render = true;
                            continue;
                        }
                        MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                            // If in menu mode, handle menu clicks first
                            if app_core.ui_state.input_mode == InputMode::Menu {
                                let mut clicked_item = None;

                                // Check popup menu first (top layer)
                                if let Some(ref menu) = app_core.ui_state.popup_menu {
                                    let pos = menu.get_position();
                                    let menu_height = menu.get_items().len() as u16 + 2; // +2 for borders
                                    let menu_width = menu
                                        .get_items()
                                        .iter()
                                        .map(|item| item.text.len())
                                        .max()
                                        .unwrap_or(10)
                                        as u16
                                        + 4; // +4 for borders and padding

                                    let menu_area = ratatui::layout::Rect {
                                        x: pos.0,
                                        y: pos.1,
                                        width: menu_width,
                                        height: menu_height,
                                    };

                                    if let Some(index) = menu.check_click(*x, *y, menu_area) {
                                        clicked_item = menu.get_items().get(index).cloned();
                                    }
                                }

                                if let Some(item) = clicked_item {
                                    let command = item.command.clone();
                                    tracing::info!(
                                        "Menu item clicked: {} (command: {})",
                                        item.text,
                                        command
                                    );

                                    // Handle command same way as Enter key
                                    if command.starts_with("menu:") {
                                        // Config menu submenu
                                        let submenu_name = &command[5..];
                                        // ... handle like in keyboard code ...
                                        tracing::debug!("Clicked config submenu: {}", submenu_name);
                                        app_core.ui_state.popup_menu = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    } else if command.starts_with("__SUBMENU__") {
                                        // Context menu or .menu submenu
                                        let category = &command[11..];

                                        // Try build_submenu first (for .menu categories)
                                        let items = app_core.build_submenu(category);
                                        let items = if !items.is_empty() {
                                            items
                                        } else if let Some(items) = app_core.menu_categories.get(category) {
                                            items.clone()
                                        } else {
                                            Vec::new()
                                        };

                                        if !items.is_empty() {
                                            let position = app_core
                                                .ui_state
                                                .popup_menu
                                                .as_ref()
                                                .map(|m| m.get_position())
                                                .unwrap_or((40, 12));
                                            let submenu_pos = (position.0 + 2, position.1);
                                            app_core.ui_state.submenu =
                                                Some(crate::data::ui_state::PopupMenu::new(
                                                    items,
                                                    submenu_pos,
                                                ));
                                            tracing::info!(
                                                "Opened submenu: {}",
                                                category
                                            );
                                        }
                                    } else if !command.is_empty() {
                                        // Close menu first
                                        app_core.ui_state.popup_menu = None;
                                        app_core.ui_state.submenu = None;
                                        app_core.ui_state.nested_submenu = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;

                                        // Check if this is an internal action or game command
                                        if command.starts_with("action:") {
                                            // Internal action - handle it
                                            if let Err(e) = handle_menu_action(
                                                &mut app_core,
                                                &mut frontend,
                                                &command,
                                            ) {
                                                tracing::error!("Menu action error: {}", e);
                                            }
                                        } else {
                                            // Game command - send to server
                                            let _ = command_tx.send(format!("{}\n", command));
                                            tracing::info!(
                                                "Sent context menu command via click: {}",
                                                command
                                            );
                                        }
                                    }
                                    app_core.needs_render = true;
                                } else {
                                    // Click outside menu - close it
                                    app_core.ui_state.popup_menu = None;
                                    app_core.ui_state.submenu = None;
                                    app_core.ui_state.nested_submenu = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                    app_core.needs_render = true;
                                }

                                // Don't process other clicks while in menu mode
                                continue;
                            }

                            // Mouse down handling (find links, start drags)
                            app_core.ui_state.selection_state = None;

                            let mut found_window = None;
                            let mut drag_op = None;
                            let mut clicked_window_name: Option<String> = None;

                            for (name, window) in &app_core.ui_state.windows {
                                let pos = &window.position;
                                if *x >= pos.x
                                    && *x < pos.x + pos.width
                                    && *y >= pos.y
                                    && *y < pos.y + pos.height
                                {
                                    clicked_window_name = Some(name.clone());

                                    let right_col = pos.x + pos.width - 1;
                                    let bottom_row = pos.y + pos.height - 1;
                                    let has_horizontal_space = pos.width > 1;
                                    let has_vertical_space = pos.height > 1;

                                    if has_horizontal_space
                                        && has_vertical_space
                                        && *x == right_col
                                        && *y == bottom_row
                                    {
                                        drag_op = Some(DragOperation::ResizeBottomRight);
                                        found_window = Some(name.clone());
                                        break;
                                    } else if has_horizontal_space && *x == right_col {
                                        drag_op = Some(DragOperation::ResizeRight);
                                        found_window = Some(name.clone());
                                        break;
                                    } else if has_vertical_space && *y == bottom_row {
                                        drag_op = Some(DragOperation::ResizeBottom);
                                        found_window = Some(name.clone());
                                        break;
                                    } else if *y == pos.y {
                                        drag_op = Some(DragOperation::Move);
                                        found_window = Some(name.clone());
                                        break;
                                    }
                                }
                            }

                            if let (Some(window_name), Some(operation)) = (found_window, drag_op) {
                                if let Some(window) = app_core.ui_state.get_window(&window_name) {
                                    let pos = &window.position;
                                    app_core.ui_state.mouse_drag = Some(MouseDragState {
                                        operation,
                                        window_name,
                                        start_pos: (*x, *y),
                                        original_window_pos: (pos.x, pos.y, pos.width, pos.height),
                                    });
                                }
                            } else if let Some(window_name) = clicked_window_name {
                                if let Some(window) = app_core.ui_state.get_window(&window_name) {
                                    let pos = &window.position;
                                    let window_rect = ratatui::layout::Rect {
                                        x: pos.x,
                                        y: pos.y,
                                        width: pos.width,
                                        height: pos.height,
                                    };

                                    if let Some(link_data) =
                                        frontend.link_at_position(&window_name, *x, *y, window_rect)
                                    {
                                        let has_ctrl = modifiers.contains(KeyModifiers::CONTROL);

                                        if has_ctrl {
                                            app_core.ui_state.link_drag_state =
                                                Some(LinkDragState {
                                                    link_data,
                                                    start_pos: (*x, *y),
                                                    current_pos: (*x, *y),
                                                });
                                        } else {
                                            app_core.ui_state.pending_link_click =
                                                Some(PendingLinkClick {
                                                    link_data,
                                                    click_pos: (*x, *y),
                                                });
                                        }
                                    } else {
                                        // Start text selection
                                        app_core.ui_state.selection_drag_start = Some((*x, *y));

                                        // Convert mouse coords to text coords for selection
                                        if let Some((line, col)) = frontend.mouse_to_text_coords(
                                            &window_name,
                                            *x,
                                            *y,
                                            window_rect,
                                        ) {
                                            // Find window index from the stable mapping
                                            let window_index = window_index_map
                                                .get(&window_name)
                                                .copied()
                                                .unwrap_or(0);
                                            app_core.ui_state.selection_state =
                                                Some(crate::selection::SelectionState::new(
                                                    window_index,
                                                    line,
                                                    col,
                                                ));
                                        }
                                    }
                                }
                            }
                            continue;
                        }
                        MouseEventKind::Drag(crossterm::event::MouseButton::Left) => {
                            if let Some(ref mut link_drag) = app_core.ui_state.link_drag_state {
                                link_drag.current_pos = (*x, *y);
                                app_core.needs_render = true;
                            } else if let Some(drag_state) = app_core.ui_state.mouse_drag.clone() {
                                let dx = *x as i32 - drag_state.start_pos.0 as i32;
                                let dy = *y as i32 - drag_state.start_pos.1 as i32;

                                // Get terminal size for clamping windows within bounds
                                let (term_width, term_height) = frontend.size();

                                let (min_width_constraint, min_height_constraint) =
                                    app_core.window_min_size(&drag_state.window_name);

                                if let Some(window) =
                                    app_core.ui_state.get_window_mut(&drag_state.window_name)
                                {
                                    let min_width_i32 = min_width_constraint as i32;
                                    let min_height_i32 = min_height_constraint as i32;

                                    match drag_state.operation {
                                        DragOperation::Move => {
                                            // Calculate new position
                                            let new_x = (drag_state.original_window_pos.0 as i32
                                                + dx)
                                                .max(0)
                                                as u16;
                                            let new_y = (drag_state.original_window_pos.1 as i32
                                                + dy)
                                                .max(0)
                                                as u16;

                                            // Clamp to prevent overflow beyond terminal boundaries
                                            let max_x =
                                                term_width.saturating_sub(window.position.width);
                                            let max_y =
                                                term_height.saturating_sub(window.position.height);

                                            window.position.x = new_x.min(max_x);
                                            window.position.y = new_y.min(max_y);
                                        }
                                        DragOperation::ResizeRight => {
                                            // Calculate new width
                                            let new_width =
                                                (drag_state.original_window_pos.2 as i32 + dx)
                                                    .max(min_width_i32)
                                                    as u16;

                                            // Clamp to prevent overflow beyond terminal edge
                                            let max_width =
                                                term_width.saturating_sub(window.position.x);
                                            window.position.width = new_width.min(max_width);
                                        }
                                        DragOperation::ResizeBottom => {
                                            // Calculate new height
                                            let new_height =
                                                (drag_state.original_window_pos.3 as i32 + dy)
                                                    .max(min_height_i32)
                                                    as u16;

                                            // Clamp to prevent overflow beyond terminal edge
                                            let max_height =
                                                term_height.saturating_sub(window.position.y);
                                            window.position.height = new_height.min(max_height);
                                        }
                                        DragOperation::ResizeBottomRight => {
                                            // Calculate new dimensions
                                            let new_width =
                                                (drag_state.original_window_pos.2 as i32 + dx)
                                                    .max(min_width_i32)
                                                    as u16;
                                            let new_height =
                                                (drag_state.original_window_pos.3 as i32 + dy)
                                                    .max(min_height_i32)
                                                    as u16;

                                            // Clamp to prevent overflow beyond terminal edges
                                            let max_width =
                                                term_width.saturating_sub(window.position.x);
                                            let max_height =
                                                term_height.saturating_sub(window.position.y);

                                            window.position.width = new_width.min(max_width);
                                            window.position.height = new_height.min(max_height);
                                        }
                                    }
                                    app_core.needs_render = true;
                                }
                            } else if app_core.ui_state.pending_link_click.is_some() {
                                app_core.ui_state.pending_link_click = None;
                            } else if let Some(_drag_start) = app_core.ui_state.selection_drag_start
                            {
                                // Update text selection on drag
                                if let Some(ref mut selection) = app_core.ui_state.selection_state {
                                    // Find which window we're dragging in
                                    for (name, window) in &app_core.ui_state.windows {
                                        let pos = &window.position;
                                        if *x >= pos.x
                                            && *x < pos.x + pos.width
                                            && *y >= pos.y
                                            && *y < pos.y + pos.height
                                        {
                                            let window_rect = ratatui::layout::Rect {
                                                x: pos.x,
                                                y: pos.y,
                                                width: pos.width,
                                                height: pos.height,
                                            };
                                            if let Some((line, col)) = frontend
                                                .mouse_to_text_coords(name, *x, *y, window_rect)
                                            {
                                                let window_index = window_index_map
                                                    .get(name)
                                                    .copied()
                                                    .unwrap_or(0);
                                                selection.update_end(window_index, line, col);
                                                app_core.needs_render = true;
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                            continue;
                        }
                        MouseEventKind::Up(crossterm::event::MouseButton::Left) => {
                            if let Some(link_drag) = app_core.ui_state.link_drag_state.take() {
                                let dx = (*x as i16 - link_drag.start_pos.0 as i16).abs();
                                let dy = (*y as i16 - link_drag.start_pos.1 as i16).abs();

                                if dx > 2 || dy > 2 {
                                    let mut drop_target_hand: Option<String> = None;
                                    let mut drop_target_id: Option<String> = None;

                                    for (name, window) in &app_core.ui_state.windows {
                                        let pos = &window.position;
                                        if *x >= pos.x
                                            && *x < pos.x + pos.width
                                            && *y >= pos.y
                                            && *y < pos.y + pos.height
                                        {
                                            // First check if this is a hand widget (left or right only)
                                            if name == "left_hand" {
                                                drop_target_hand = Some("left".to_string());
                                                break;
                                            } else if name == "right_hand" {
                                                drop_target_hand = Some("right".to_string());
                                                break;
                                            }

                                            // Otherwise check if we dropped on a link
                                            let window_rect = ratatui::layout::Rect {
                                                x: pos.x,
                                                y: pos.y,
                                                width: pos.width,
                                                height: pos.height,
                                            };
                                            if let Some(target_link) =
                                                frontend.link_at_position(name, *x, *y, window_rect)
                                            {
                                                drop_target_id = Some(target_link.exist_id);
                                                break;
                                            }
                                        }
                                    }

                                    let command = if let Some(hand_type) = drop_target_hand {
                                        format!(
                                            "_drag #{} {}\n",
                                            link_drag.link_data.exist_id, hand_type
                                        )
                                    } else if let Some(target_id) = drop_target_id {
                                        format!(
                                            "_drag #{} #{}\n",
                                            link_drag.link_data.exist_id, target_id
                                        )
                                    } else {
                                        format!("_drag #{} drop\n", link_drag.link_data.exist_id)
                                    };
                                    let _ = command_tx.send(command);
                                }
                            } else if let Some(pending_click) =
                                app_core.ui_state.pending_link_click.take()
                            {
                                let dx = (*x as i16 - pending_click.click_pos.0 as i16).abs();
                                let dy = (*y as i16 - pending_click.click_pos.1 as i16).abs();

                                if dx <= 2 && dy <= 2 {
                                    // Handle <d> tags differently (direct commands vs context menus)
                                    if pending_click.link_data.exist_id == "_direct_" {
                                        // <d> tag: Send text/noun as direct command
                                        let command = if !pending_click.link_data.noun.is_empty() {
                                            format!("{}\n", pending_click.link_data.noun)
                                        // Use cmd attribute
                                        } else {
                                            format!("{}\n", pending_click.link_data.text)
                                            // Use text content
                                        };
                                        tracing::info!(
                                            "Executing <d> direct command: {}",
                                            command.trim()
                                        );
                                        let _ = command_tx.send(command);
                                    } else {
                                        // Regular <a> tag: Request context menu
                                        let command = app_core.request_menu(
                                            pending_click.link_data.exist_id.clone(),
                                            pending_click.link_data.noun.clone(),
                                            pending_click.click_pos,
                                        );
                                        tracing::info!(
                                            "Sending _menu command for '{}' (exist_id: {})",
                                            pending_click.link_data.noun,
                                            pending_click.link_data.exist_id
                                        );
                                        let _ = command_tx.send(command);
                                    }
                                } else {
                                    tracing::debug!(
                                        "Link click cancelled - dragged {} pixels",
                                        dx.max(dy)
                                    );
                                }
                            }

                            // Sync UI state positions back to layout WindowDefs after mouse resize/move
                            if let Some(drag_state) = &app_core.ui_state.mouse_drag {
                                if let Some(window) =
                                    app_core.ui_state.get_window(&drag_state.window_name)
                                {
                                    // Find the corresponding WindowDef in layout and update it
                                    if let Some(window_def) = app_core
                                        .layout
                                        .windows
                                        .iter_mut()
                                        .find(|w| w.name() == drag_state.window_name)
                                    {
                                        let base = window_def.base_mut();
                                        base.col = window.position.x;
                                        base.row = window.position.y;
                                        base.cols = window.position.width;
                                        base.rows = window.position.height;
                                        tracing::info!("Synced mouse resize/move for '{}' to layout: pos=({},{}) size={}x{}",
                                            drag_state.window_name, base.col, base.row, base.cols, base.rows);
                                        app_core.layout_modified_since_save = true;
                                    }
                                }
                            }

                            app_core.ui_state.mouse_drag = None;
                            app_core.ui_state.selection_drag_start = None;

                            // Handle text selection copy to clipboard
                            if let Some(ref selection) = app_core.ui_state.selection_state {
                                if !selection.is_empty() {
                                    // Extract text from selection
                                    let (start, end) = selection.normalized_range();

                                    // Find the window (for now assume main window)
                                    if let Some((_line, _col)) = frontend.mouse_to_text_coords(
                                        "main",
                                        *x,
                                        *y,
                                        ratatui::layout::Rect {
                                            x: app_core
                                                .ui_state
                                                .windows
                                                .get("main")
                                                .map(|w| w.position.x)
                                                .unwrap_or(0),
                                            y: app_core
                                                .ui_state
                                                .windows
                                                .get("main")
                                                .map(|w| w.position.y)
                                                .unwrap_or(0),
                                            width: app_core
                                                .ui_state
                                                .windows
                                                .get("main")
                                                .map(|w| w.position.width)
                                                .unwrap_or(80),
                                            height: app_core
                                                .ui_state
                                                .windows
                                                .get("main")
                                                .map(|w| w.position.height)
                                                .unwrap_or(24),
                                        },
                                    ) {
                                        if let Some(text) = frontend.extract_selection_text(
                                            "main", start.line, start.col, end.line, end.col,
                                        ) {
                                            // Copy to clipboard
                                            match arboard::Clipboard::new() {
                                                Ok(mut clipboard) => {
                                                    if let Err(e) = clipboard.set_text(&text) {
                                                        tracing::warn!(
                                                            "Failed to copy to clipboard: {}",
                                                            e
                                                        );
                                                    } else {
                                                        tracing::info!(
                                                            "Copied {} chars to clipboard",
                                                            text.len()
                                                        );
                                                    }
                                                }
                                                Err(e) => {
                                                    tracing::warn!(
                                                        "Failed to access clipboard: {}",
                                                        e
                                                    );
                                                }
                                            }
                                        }
                                    }
                                }
                                // Clear selection
                                app_core.ui_state.selection_state = None;
                                app_core.needs_render = true;
                            }

                            continue;
                        }
                        _ => {}
                    }
                }
                frontend::FrontendEvent::Key { code, modifiers } => {
                    // Key events are handled in handle_frontend_event()
                    // No early intercepts - let the 3-layer routing handle everything
                }
                _ => {}
            }

            if let Some(command) = handle_frontend_event(&mut app_core, &mut frontend, event)? {
                let _ = command_tx.send(command);
            }
        }

        // Poll for server messages (non-blocking)
        while let Ok(msg) = server_rx.try_recv() {
            match msg {
                ServerMessage::Text(line) => {
                    // Process incoming server data through parser
                    if let Err(e) = app_core.process_server_data(&line) {
                        tracing::error!("Error processing server data: {}", e);
                    }
                    // Check for highlight sound triggers
                    app_core.check_sound_triggers(&line);
                }
                ServerMessage::Connected => {
                    tracing::info!("Connected to game server");
                    app_core.game_state.connected = true;
                    app_core.needs_render = true;
                }
                ServerMessage::Disconnected => {
                    tracing::info!("Disconnected from game server");
                    app_core.game_state.connected = false;
                    app_core.needs_render = true;
                }
            }
        }

        // Force render every second for countdown widgets
        if last_countdown_update.elapsed().as_secs() >= 1 {
            app_core.needs_render = true;
            last_countdown_update = std::time::Instant::now();
        }

        // Render if needed
        if app_core.needs_render {
            frontend.render(&mut app_core)?;
            app_core.needs_render = false;
        }

        // No sleep needed - event::poll() timeout already limits frame rate to ~60 FPS
    }

    // Save command history
    if let Err(e) = frontend.command_input_save_history("command_input", character.as_deref()) {
        tracing::warn!("Failed to save command history: {}", e);
    }

    // Cleanup
    frontend.cleanup()?;

    // Wait for network task to finish (or abort it)
    network_handle.abort();
    let _ = network_handle.await;

    Ok(())
}

/// Run GUI frontend
fn run_gui(config: config::Config) -> Result<()> {
    use core::AppCore;
    use frontend::EguiApp;

    // Create core application state
    let app_core = AppCore::new(config)?;

    // Create and run GUI
    let app = EguiApp::new(app_core);
    app.run()?;

    Ok(())
}

/// Handle a frontend event
/// Returns Some(command) if a command should be sent to the server
fn handle_frontend_event(
    app_core: &mut core::AppCore,
    frontend: &mut frontend::TuiFrontend,
    event: frontend::FrontendEvent,
) -> Result<Option<String>> {
    use crossterm::event::{KeyCode, KeyModifiers};
    use frontend::FrontendEvent;

    match event {
        FrontendEvent::Key { code, modifiers } => {
            use crate::data::ui_state::InputMode;

            tracing::debug!(
                "Key event: code={:?}, modifiers={:?}, input_mode={:?}",
                code,
                modifiers,
                app_core.ui_state.input_mode
            );

            // ═══════════════════════════════════════════════════════════════
            // 3-LAYER KEYBIND ROUTING SYSTEM
            // ═══════════════════════════════════════════════════════════════
            // 1. PRIORITY: Global keybinds (Ctrl+C, Ctrl+F, Esc)
            // 2. PRIORITY: High-priority windows (editors, browsers, forms)
            //              → Checked via has_priority_window()
            //              → Routes to widgets via menu keybinds
            // 3. NORMAL:   User keybinds (keybinds.toml)
            // 4. FALLBACK: CommandInput (typing)
            // ═══════════════════════════════════════════════════════════════

            // LAYER 1: Global keybinds (always checked first)

            // Handle Ctrl+C to quit
            if (code == KeyCode::Char('c') || code == KeyCode::Char('C'))
                && modifiers.contains(KeyModifiers::CONTROL)
            {
                app_core.quit();
                return Ok(None);
            }

            // Handle Ctrl+F to start search
            if (code == KeyCode::Char('f') || code == KeyCode::Char('F'))
                && modifiers.contains(KeyModifiers::CONTROL)
            {
                app_core.start_search_mode();
                return Ok(None);
            }

            // Handle Ctrl+PageUp/PageDown for search navigation
            if modifiers.contains(KeyModifiers::CONTROL) {
                match code {
                    KeyCode::PageDown => {
                        tracing::debug!("Ctrl+PageDown detected - next search match");
                        let window_name = app_core.get_focused_window_name();
                        if frontend.next_search_match(&window_name) {
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    KeyCode::PageUp => {
                        tracing::debug!("Ctrl+PageUp detected - previous search match");
                        let window_name = app_core.get_focused_window_name();
                        if frontend.prev_search_match(&window_name) {
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    _ => {}
                }
            }

            // Debug log for PageUp/PageDown without Ctrl
            if matches!(code, KeyCode::PageDown | KeyCode::PageUp) {
                tracing::debug!(
                    "PageUp/PageDown without Ctrl: code={:?}, modifiers={:?}",
                    code,
                    modifiers
                );
            }

            // Handle Esc
            if code == KeyCode::Esc {
                // If in window editor mode, handle context-aware navigation
                if app_core.ui_state.input_mode == InputMode::WindowEditor {
                    if let Some(ref mut editor) = frontend.window_editor {
                        if editor.is_on_meta() {
                            // On meta section - close editor
                            frontend.window_editor = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                            app_core.needs_render = true;
                            return Ok(None);
                        } else {
                            // In a numbered section - return to meta section
                            editor.return_to_meta();
                            app_core.needs_render = true;
                            return Ok(None);
                        }
                    }
                }
                // If in menu mode, close menus one layer at a time
                if app_core.ui_state.input_mode == InputMode::Menu {
                    // If submenu is open, close it first
                    if app_core.ui_state.submenu.is_some() {
                        app_core.ui_state.submenu = None;
                        app_core.needs_render = true;
                        return Ok(None);
                    }
                    // Otherwise close main menu and return to normal mode
                    app_core.ui_state.popup_menu = None;
                    app_core.ui_state.input_mode = InputMode::Normal;
                    app_core.needs_render = true;
                    return Ok(None);
                }
                // If in search mode, clear search and exit search mode
                if app_core.ui_state.input_mode == InputMode::Search {
                    frontend.clear_all_searches();
                    app_core.clear_search_mode();
                    return Ok(None);
                }
                // For browser/form modes, close the widget and return to normal
                if input_router::has_priority_window(&app_core.ui_state.input_mode) {
                    // Close the browser/form widget
                    frontend.highlight_browser = None;
                    frontend.highlight_form = None;
                    frontend.keybind_browser = None;
                    frontend.keybind_form = None;
                    frontend.color_palette_browser = None;
                    frontend.color_form = None;
                    frontend.spell_color_browser = None;
                    frontend.spell_color_form = None;
                    frontend.uicolors_browser = None;
                    frontend.theme_browser = None;
                    frontend.theme_editor = None;
                    frontend.settings_editor = None;
                    app_core.ui_state.input_mode = InputMode::Normal;
                    app_core.needs_render = true;
                    return Ok(None);
                }
                // Otherwise do nothing - Escape does not quit (use .quit or Ctrl+C instead)
                return Ok(None);
            }

            // LAYER 2: Priority windows (editors, browsers, forms)
            use crate::core::input_router;
            if input_router::has_priority_window(&app_core.ui_state.input_mode) {
                let key_event = crossterm::event::KeyEvent::new(code, modifiers);

                // Browser widgets
                match app_core.ui_state.input_mode {
                    InputMode::HighlightBrowser => {
                        if let Some(ref mut browser) = frontend.highlight_browser {
                            use crate::frontend::tui::widget_traits::{Navigable, Selectable};
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NavigateUp => {
                                    browser.navigate_up()
                                }
                                crate::core::menu_actions::MenuAction::NavigateDown => {
                                    browser.navigate_down()
                                }
                                crate::core::menu_actions::MenuAction::PageUp => browser.page_up(),
                                crate::core::menu_actions::MenuAction::PageDown => {
                                    browser.page_down()
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.highlight_browser = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                crate::core::menu_actions::MenuAction::Delete => {
                                    if let Some(name) = browser.delete_selected() {
                                        app_core.config.highlights.retain(|k, _v| k != &name);
                                        tracing::info!("Deleted highlight: {}", name);
                                    }
                                }
                                crate::core::menu_actions::MenuAction::Edit => {
                                    if let Some(name) = browser.get_selected() {
                                        if let Some(pattern) = app_core.config.highlights.get(&name)
                                        {
                                            frontend.highlight_form = Some(
                                                frontend::tui::highlight_form::HighlightFormWidget::new_edit(
                                                    name.clone(),
                                                    pattern
                                                )
                                            );
                                            frontend.highlight_browser = None;
                                            app_core.ui_state.input_mode =
                                                data::ui_state::InputMode::HighlightForm;
                                            tracing::info!(
                                                "Opening highlight for editing: {}",
                                                name
                                            );
                                        }
                                    }
                                }
                                _ => {}
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::KeybindBrowser => {
                        if let Some(ref mut browser) = frontend.keybind_browser {
                            use crate::frontend::tui::widget_traits::{Navigable, Selectable};
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NavigateUp => {
                                    browser.navigate_up()
                                }
                                crate::core::menu_actions::MenuAction::NavigateDown => {
                                    browser.navigate_down()
                                }
                                crate::core::menu_actions::MenuAction::PageUp => browser.page_up(),
                                crate::core::menu_actions::MenuAction::PageDown => {
                                    browser.page_down()
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.keybind_browser = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                crate::core::menu_actions::MenuAction::Delete => {
                                    if let Some(combo) = browser.delete_selected() {
                                        app_core.config.keybinds.remove(&combo);
                                        app_core.rebuild_keybind_map();
                                        tracing::info!("Deleted keybind: {}", combo);
                                    }
                                }
                                _ => {}
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::ColorPaletteBrowser => {
                        if let Some(ref mut browser) = frontend.color_palette_browser {
                            use crate::frontend::tui::widget_traits::{Navigable, Selectable};
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NavigateUp => {
                                    browser.navigate_up()
                                }
                                crate::core::menu_actions::MenuAction::NavigateDown => {
                                    browser.navigate_down()
                                }
                                crate::core::menu_actions::MenuAction::PageUp => browser.page_up(),
                                crate::core::menu_actions::MenuAction::PageDown => {
                                    browser.page_down()
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.color_palette_browser = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                crate::core::menu_actions::MenuAction::Delete => {
                                    if let Some(name) = browser.delete_selected() {
                                        app_core
                                            .config
                                            .colors
                                            .color_palette
                                            .retain(|c| c.name != name);
                                        tracing::info!("Deleted color: {}", name);
                                    }
                                }
                                _ => {}
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::SpellColorsBrowser => {
                        if let Some(ref mut browser) = frontend.spell_color_browser {
                            use crate::frontend::tui::widget_traits::{Navigable, Selectable};
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NavigateUp => {
                                    browser.navigate_up()
                                }
                                crate::core::menu_actions::MenuAction::NavigateDown => {
                                    browser.navigate_down()
                                }
                                crate::core::menu_actions::MenuAction::PageUp => browser.page_up(),
                                crate::core::menu_actions::MenuAction::PageDown => {
                                    browser.page_down()
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.spell_color_browser = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                _ => {}
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::UIColorsBrowser => {
                        if let Some(ref mut browser) = frontend.uicolors_browser {
                            use crate::frontend::tui::widget_traits::{Navigable, Selectable};
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NavigateUp => {
                                    browser.navigate_up()
                                }
                                crate::core::menu_actions::MenuAction::NavigateDown => {
                                    browser.navigate_down()
                                }
                                crate::core::menu_actions::MenuAction::PageUp => browser.page_up(),
                                crate::core::menu_actions::MenuAction::PageDown => {
                                    browser.page_down()
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.uicolors_browser = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                _ => {}
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::ThemeBrowser => {
                        if let Some(ref mut browser) = frontend.theme_browser {
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NavigateUp => {
                                    browser.previous()
                                }
                                crate::core::menu_actions::MenuAction::NavigateDown => {
                                    browser.next()
                                }
                                crate::core::menu_actions::MenuAction::PageUp => browser.page_up(),
                                crate::core::menu_actions::MenuAction::PageDown => {
                                    browser.page_down()
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.theme_browser = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                crate::core::menu_actions::MenuAction::Select => {
                                    // Apply selected theme
                                    if let Some(theme_id) = browser.get_selected_theme_id() {
                                        app_core.config.active_theme = theme_id.clone();
                                        let theme = app_core.config.get_theme();
                                        frontend.update_theme_cache(theme_id.clone(), theme);
                                        app_core.needs_render = true;
                                        tracing::info!("Switched to theme: {}", theme_id);
                                        app_core.add_system_message(&format!(
                                            "Theme switched to: {}",
                                            theme_id
                                        ));

                                        // Save config with new theme
                                        if let Err(e) = app_core
                                            .config
                                            .save(app_core.config.character.as_deref())
                                        {
                                            tracing::error!(
                                                "Failed to save config after theme change: {}",
                                                e
                                            );
                                            app_core.add_system_message(&format!(
                                                "Warning: Failed to save theme preference: {}",
                                                e
                                            ));
                                        }

                                        // Close browser
                                        frontend.theme_browser = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                }
                                _ => {
                                    // Check for 'D' key to delete custom theme
                                    if code == KeyCode::Char('d') || code == KeyCode::Char('D') {
                                        if browser.is_selected_custom() {
                                            if let Some(theme_id) = browser.get_selected_theme_id()
                                            {
                                                match browser.delete_selected_custom(
                                                    app_core.config.character.as_deref(),
                                                ) {
                                                    Ok(_) => {
                                                        tracing::info!(
                                                            "Deleted custom theme: {}",
                                                            theme_id
                                                        );
                                                        app_core.add_system_message(&format!(
                                                            "Deleted custom theme: {}",
                                                            theme_id
                                                        ));
                                                    }
                                                    Err(e) => {
                                                        tracing::error!(
                                                            "Failed to delete custom theme: {}",
                                                            e
                                                        );
                                                        app_core.add_system_message(&format!(
                                                            "Error deleting theme: {}",
                                                            e
                                                        ));
                                                    }
                                                }
                                            }
                                        } else {
                                            app_core
                                                .add_system_message("Cannot delete built-in theme");
                                        }
                                    }
                                }
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::SettingsEditor => {
                        if let Some(ref mut editor) = frontend.settings_editor {
                            use crate::frontend::tui::widget_traits::{
                                Cyclable, Navigable, Toggleable,
                            };
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NavigateUp => {
                                    editor.navigate_up()
                                }
                                crate::core::menu_actions::MenuAction::NavigateDown => {
                                    editor.navigate_down()
                                }
                                crate::core::menu_actions::MenuAction::PageUp => editor.page_up(),
                                crate::core::menu_actions::MenuAction::PageDown => {
                                    editor.page_down()
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.settings_editor = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                crate::core::menu_actions::MenuAction::Toggle => {
                                    editor.toggle_focused();
                                }
                                _ => {
                                    // Let the editor handle input internally
                                    let key = crossterm::event::KeyEvent::new(code, modifiers);
                                    editor.handle_input(key);
                                }
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::HighlightForm => {
                        if let Some(ref mut form) = frontend.highlight_form {
                            use crate::frontend::tui::widget_traits::{
                                Cyclable, FieldNavigable, TextEditable, Toggleable,
                            };
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NextField => {
                                    form.next_field()
                                }
                                crate::core::menu_actions::MenuAction::PreviousField => {
                                    form.previous_field()
                                }
                                crate::core::menu_actions::MenuAction::SelectAll => {
                                    form.select_all()
                                }
                                crate::core::menu_actions::MenuAction::Copy => {
                                    let _ = form.copy_to_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Cut => {
                                    let _ = form.cut_to_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Paste => {
                                    let _ = form.paste_from_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Toggle => {
                                    form.toggle_focused();
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.highlight_form = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                _ => {
                                    // Let form handle other input (typing, save, etc.)
                                    let key = crossterm::event::KeyEvent::new(code, modifiers);
                                    if let Some(result) = form.handle_key(key) {
                                        match result {
                                            crate::frontend::tui::highlight_form::FormResult::Save { name, pattern } => {
                                                app_core.config.highlights.insert(name.clone(), pattern);
                                                frontend.highlight_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                                tracing::info!("Saved highlight: {}", name);
                                            }
                                            crate::frontend::tui::highlight_form::FormResult::Delete { name } => {
                                                app_core.config.highlights.remove(&name);
                                                frontend.highlight_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                                tracing::info!("Deleted highlight: {}", name);
                                            }
                                            crate::frontend::tui::highlight_form::FormResult::Cancel => {
                                                frontend.highlight_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                            }
                                        }
                                    }
                                }
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::KeybindForm => {
                        if let Some(ref mut form) = frontend.keybind_form {
                            use crate::frontend::tui::widget_traits::{
                                Cyclable, FieldNavigable, TextEditable, Toggleable,
                            };
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NextField => {
                                    form.next_field()
                                }
                                crate::core::menu_actions::MenuAction::PreviousField => {
                                    form.previous_field()
                                }
                                crate::core::menu_actions::MenuAction::SelectAll => {
                                    form.select_all()
                                }
                                crate::core::menu_actions::MenuAction::Copy => {
                                    let _ = form.copy_to_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Cut => {
                                    let _ = form.cut_to_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Paste => {
                                    let _ = form.paste_from_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Toggle => {
                                    form.toggle_focused();
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.keybind_form = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                _ => {
                                    // Let form handle other input (typing, cycling dropdown, save, etc.)
                                    let key = crossterm::event::KeyEvent::new(code, modifiers);
                                    if let Some(result) = form.handle_key(key) {
                                        match result {
                                            crate::frontend::tui::keybind_form::KeybindFormResult::Save { key_combo, action_type, value } => {
                                                use crate::frontend::tui::keybind_form::KeybindActionType;
                                                let action = match action_type {
                                                    KeybindActionType::Action => crate::config::KeyBindAction::Action(value),
                                                    KeybindActionType::Macro => crate::config::KeyBindAction::Macro(
                                                        crate::config::MacroAction { macro_text: value }
                                                    ),
                                                };
                                                app_core.config.keybinds.insert(key_combo.clone(), action);
                                                app_core.rebuild_keybind_map();
                                                frontend.keybind_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                                tracing::info!("Saved keybind: {}", key_combo);
                                            }
                                            crate::frontend::tui::keybind_form::KeybindFormResult::Delete { key_combo } => {
                                                app_core.config.keybinds.remove(&key_combo);
                                                app_core.rebuild_keybind_map();
                                                frontend.keybind_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                                tracing::info!("Deleted keybind: {}", key_combo);
                                            }
                                            crate::frontend::tui::keybind_form::KeybindFormResult::Cancel => {
                                                frontend.keybind_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                            }
                                        }
                                    }
                                }
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::ColorForm => {
                        if let Some(ref mut form) = frontend.color_form {
                            use crate::frontend::tui::widget_traits::{
                                FieldNavigable, TextEditable, Toggleable,
                            };
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NextField => {
                                    form.next_field()
                                }
                                crate::core::menu_actions::MenuAction::PreviousField => {
                                    form.previous_field()
                                }
                                crate::core::menu_actions::MenuAction::SelectAll => {
                                    form.select_all()
                                }
                                crate::core::menu_actions::MenuAction::Copy => {
                                    let _ = form.copy_to_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Cut => {
                                    let _ = form.cut_to_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Paste => {
                                    let _ = form.paste_from_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Toggle => {
                                    form.toggle_focused();
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.color_form = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                _ => {
                                    // Let form handle other input (typing, save, etc.)
                                    let key = crossterm::event::KeyEvent::new(code, modifiers);
                                    if let Some(result) = form.handle_input(key) {
                                        match result {
                                            crate::frontend::tui::color_form::FormAction::Save { color, original_name } => {
                                                // Remove old entry if name changed
                                                if let Some(old_name) = original_name {
                                                    if old_name != color.name {
                                                        app_core.config.colors.color_palette.retain(|c| c.name != old_name);
                                                    }
                                                }
                                                // Add/update color
                                                if let Some(existing) = app_core.config.colors.color_palette.iter_mut().find(|c| c.name == color.name) {
                                                    *existing = color.clone();
                                                } else {
                                                    app_core.config.colors.color_palette.push(color.clone());
                                                }
                                                frontend.color_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                                tracing::info!("Saved color: {}", color.name);
                                            }
                                            crate::frontend::tui::color_form::FormAction::Delete => {
                                                // Delete handled by original_name
                                                frontend.color_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                            }
                                            crate::frontend::tui::color_form::FormAction::Cancel => {
                                                frontend.color_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                            }
                                            crate::frontend::tui::color_form::FormAction::Error(_) => {
                                                // Error is displayed in the form, don't close
                                            }
                                        }
                                    }
                                }
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::SpellColorForm => {
                        if let Some(ref mut form) = frontend.spell_color_form {
                            use crate::frontend::tui::widget_traits::{
                                FieldNavigable, TextEditable,
                            };
                            let action = input_router::route_input(
                                key_event,
                                &app_core.ui_state.input_mode,
                                &app_core.config,
                            );

                            match action {
                                crate::core::menu_actions::MenuAction::NextField => {
                                    form.next_field()
                                }
                                crate::core::menu_actions::MenuAction::PreviousField => {
                                    form.previous_field()
                                }
                                crate::core::menu_actions::MenuAction::SelectAll => {
                                    form.select_all()
                                }
                                crate::core::menu_actions::MenuAction::Copy => {
                                    let _ = form.copy_to_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Cut => {
                                    let _ = form.cut_to_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Paste => {
                                    let _ = form.paste_from_clipboard();
                                }
                                crate::core::menu_actions::MenuAction::Cancel => {
                                    frontend.spell_color_form = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                                _ => {
                                    // Let form handle other input (typing, save, etc.)
                                    let key = crossterm::event::KeyEvent::new(code, modifiers);
                                    if let Some(result) = form.input(key) {
                                        match result {
                                            crate::frontend::tui::spell_color_form::SpellColorFormResult::Save(spell_color) => {
                                                // Add/update spell color range
                                                app_core.config.colors.spell_colors.push(spell_color);
                                                frontend.spell_color_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                                tracing::info!("Saved spell color range");
                                            }
                                            crate::frontend::tui::spell_color_form::SpellColorFormResult::Delete(index) => {
                                                if index < app_core.config.colors.spell_colors.len() {
                                                    app_core.config.colors.spell_colors.remove(index);
                                                    tracing::info!("Deleted spell color range");
                                                }
                                                frontend.spell_color_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                            }
                                            crate::frontend::tui::spell_color_form::SpellColorFormResult::Cancel => {
                                                frontend.spell_color_form = None;
                                                app_core.ui_state.input_mode = InputMode::Normal;
                                            }
                                        }
                                    }
                                }
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    InputMode::ThemeEditor => {
                        if let Some(ref mut editor) = frontend.theme_editor {
                            // Theme editor handles its own input logic
                            let key = crossterm::event::KeyEvent::new(code, modifiers);
                            if let Some(result) = editor.handle_input(key) {
                                match result {
                                    crate::frontend::tui::theme_editor::ThemeEditorResult::Save(theme_data) => {
                                        // Save custom theme to ~/.two-face/themes/
                                        match theme_data.save_to_file(app_core.config.character.as_deref()) {
                                            Ok(path) => {
                                                tracing::info!("Saved custom theme '{}' to {:?}", theme_data.name, path);
                                                app_core.add_system_message(&format!("Saved custom theme: {}", theme_data.name));

                                                // Convert ThemeData to AppTheme and add to config
                                                if let Some(app_theme) = theme_data.to_app_theme() {
                                                    // Switch to the new theme
                                                    app_core.config.active_theme = theme_data.name.clone();
                                                    let theme = app_core.config.get_theme();
                                                    frontend.update_theme_cache(theme_data.name.clone(), theme);
                                                    app_core.needs_render = true;
                                                }
                                            }
                                            Err(e) => {
                                                tracing::error!("Failed to save custom theme: {}", e);
                                                app_core.add_system_message(&format!("Error saving theme: {}", e));
                                            }
                                        }
                                        frontend.theme_editor = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    crate::frontend::tui::theme_editor::ThemeEditorResult::Cancel => {
                                        frontend.theme_editor = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                }
                            }
                            app_core.needs_render = true;
                        }
                        return Ok(None);
                    }
                    _ => {
                        // Other modes fall through
                    }
                }
            }

            // Route keys based on input mode
            if app_core.ui_state.input_mode == InputMode::Menu {
                tracing::debug!(
                    "Menu mode active - handling key: {:?}, modifiers: {:?}",
                    code,
                    modifiers
                );
                // Handle menu navigation
                // If submenu is open, route navigation to submenu instead of main menu
                match code {
                    KeyCode::Tab | KeyCode::Down => {
                        tracing::debug!("Tab/Down pressed in menu mode - selecting next");
                        // Next item - prioritize submenu if it exists
                        if let Some(ref mut submenu) = app_core.ui_state.submenu {
                            submenu.select_next();
                            app_core.needs_render = true;
                        } else if let Some(ref mut menu) = app_core.ui_state.popup_menu {
                            menu.select_next();
                            app_core.needs_render = true;
                        }
                    }
                    KeyCode::BackTab | KeyCode::Up => {
                        tracing::debug!("BackTab/Up pressed in menu mode - selecting previous");
                        // Previous item - prioritize submenu if it exists
                        if let Some(ref mut submenu) = app_core.ui_state.submenu {
                            submenu.select_prev();
                            app_core.needs_render = true;
                        } else if let Some(ref mut menu) = app_core.ui_state.popup_menu {
                            menu.select_prev();
                            app_core.needs_render = true;
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        // Select current menu item - prioritize submenu if it exists
                        let menu_to_use = if app_core.ui_state.submenu.is_some() {
                            &app_core.ui_state.submenu
                        } else {
                            &app_core.ui_state.popup_menu
                        };

                        if let Some(menu) = menu_to_use {
                            if let Some(item) = menu.selected_item() {
                                let command = item.command.clone();

                                tracing::info!("Menu command selected: {}", command);

                                // Handle submenu commands (from config menus)
                                if command.starts_with("menu:") {
                                    // This is a submenu - open it
                                    let submenu_name = &command[5..];
                                    match submenu_name {
                                        "windows" => {
                                            // Build windows submenu
                                            let items = build_windows_submenu(&app_core);
                                            app_core.ui_state.popup_menu =
                                                Some(crate::data::ui_state::PopupMenu::new(
                                                    items,
                                                    (40, 12),
                                                ));
                                        }
                                        "config" => {
                                            // Build config submenu
                                            let items = build_config_submenu();
                                            app_core.ui_state.popup_menu =
                                                Some(crate::data::ui_state::PopupMenu::new(
                                                    items,
                                                    (40, 12),
                                                ));
                                        }
                                        "layouts" => {
                                            // Build layouts submenu
                                            let items = build_layouts_submenu();
                                            app_core.ui_state.popup_menu =
                                                Some(crate::data::ui_state::PopupMenu::new(
                                                    items,
                                                    (40, 12),
                                                ));
                                        }
                                        "widgetpicker" => {
                                            // Build widget type picker for adding windows
                                            let items = build_widget_picker(&app_core);
                                            app_core.ui_state.popup_menu =
                                                Some(crate::data::ui_state::PopupMenu::new(
                                                    items,
                                                    (40, 12),
                                                ));
                                        }
                                        "addwindow" => {
                                            // Build hierarchical add window menu (categories)
                                            let items = app_core.build_add_window_menu();
                                            app_core.ui_state.popup_menu =
                                                Some(crate::data::ui_state::PopupMenu::new(
                                                    items,
                                                    (40, 12),
                                                ));
                                        }
                                        "hidewindow" => {
                                            // Build hide window menu (flat list)
                                            let items = app_core.build_hide_window_menu();
                                            app_core.ui_state.popup_menu =
                                                Some(crate::data::ui_state::PopupMenu::new(
                                                    items,
                                                    (40, 12),
                                                ));
                                        }
                                        "editwindow" => {
                                            // Build edit window menu (flat list)
                                            let items = app_core.build_edit_window_menu();
                                            app_core.ui_state.popup_menu =
                                                Some(crate::data::ui_state::PopupMenu::new(
                                                    items,
                                                    (40, 12),
                                                ));
                                        }
                                        _ => {
                                            // Unknown submenu - close menu
                                            app_core.ui_state.popup_menu = None;
                                            app_core.ui_state.input_mode = InputMode::Normal;
                                        }
                                    }
                                    app_core.needs_render = true;
                                } else if command.starts_with("__SUBMENU__") {
                                    // Context menu or .menu submenu
                                    let category = &command[11..]; // Skip "__SUBMENU__" prefix

                                    // Try build_submenu first (for .menu categories)
                                    let items = app_core.build_submenu(category);
                                    let items = if !items.is_empty() {
                                        items
                                    } else if let Some(items) = app_core.menu_categories.get(category) {
                                        items.clone()
                                    } else {
                                        Vec::new()
                                    };

                                    if !items.is_empty() {
                                        // Get current menu position
                                        let position = app_core
                                            .ui_state
                                            .popup_menu
                                            .as_ref()
                                            .map(|m| m.get_position())
                                            .unwrap_or((40, 12));

                                        // Create submenu offset to the right (small offset for overlap effect)
                                        let submenu_pos = (position.0 + 2, position.1);

                                        app_core.ui_state.submenu =
                                            Some(crate::data::ui_state::PopupMenu::new(
                                                items,
                                                submenu_pos,
                                            ));
                                        tracing::info!("Opened submenu: {}", category);
                                    } else {
                                        tracing::warn!(
                                            "Submenu category not found: {}",
                                            category
                                        );
                                        app_core.ui_state.popup_menu = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    }
                                    app_core.needs_render = true;
                                } else if command.starts_with("__SUBMENU_ADD__") {
                                    // Add Window category submenu - parse category and show windows
                                    let category_str = &command[15..]; // Skip "__SUBMENU_ADD__" prefix

                                    use config::WidgetCategory;
                                    let category = match category_str {
                                        "ProgressBar" => WidgetCategory::ProgressBar,
                                        "TextWindow" => WidgetCategory::TextWindow,
                                        "Countdown" => WidgetCategory::Countdown,
                                        "Hand" => WidgetCategory::Hand,
                                        "ActiveEffects" => WidgetCategory::ActiveEffects,
                                        "Other" => WidgetCategory::Other,
                                        _ => {
                                            tracing::warn!(
                                                "Unknown widget category: {}",
                                                category_str
                                            );
                                            app_core.ui_state.popup_menu = None;
                                            app_core.ui_state.input_mode = InputMode::Normal;
                                            app_core.needs_render = true;
                                            WidgetCategory::Other // Fallback
                                        }
                                    };

                                    // Build window list for this category
                                    let items = app_core.build_add_window_category_menu(&category);

                                    if items.is_empty() {
                                        tracing::info!(
                                            "No windows available in category: {:?}",
                                            category
                                        );
                                        app_core.ui_state.popup_menu = None;
                                        app_core.ui_state.input_mode = InputMode::Normal;
                                    } else {
                                        app_core.ui_state.popup_menu = Some(
                                            crate::data::ui_state::PopupMenu::new(items, (40, 12)),
                                        );
                                    }
                                    app_core.needs_render = true;
                                } else if command.starts_with("__ADD__") {
                                    // Add window command
                                    let window_name = &command[7..]; // Skip "__ADD__" prefix

                                    match app_core.layout.add_window(window_name) {
                                        Ok(_) => {
                                            let (width, height) = frontend.size();
                                            app_core.sync_layout_to_ui_state(
                                                width,
                                                height,
                                                &app_core.layout.clone(),
                                            );
                                            app_core.layout_modified_since_save = true;
                                            app_core.add_system_message(&format!(
                                                "Window '{}' added",
                                                window_name
                                            ));
                                            tracing::info!("Added window: {}", window_name);
                                        }
                                        Err(e) => {
                                            app_core.add_system_message(&format!(
                                                "Failed to add window: {}",
                                                e
                                            ));
                                            tracing::error!(
                                                "Failed to add window '{}': {}",
                                                window_name,
                                                e
                                            );
                                        }
                                    }

                                    app_core.ui_state.popup_menu = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                    app_core.needs_render = true;
                                } else if command.starts_with("__HIDE__") {
                                    // Hide window command
                                    let window_name = &command[8..]; // Skip "__HIDE__" prefix

                                    match app_core.layout.hide_window(window_name) {
                                        Ok(_) => {
                                            app_core.ui_state.remove_window(window_name);
                                            app_core.layout_modified_since_save = true;
                                            app_core.add_system_message(&format!(
                                                "Window '{}' hidden",
                                                window_name
                                            ));
                                            tracing::info!("Hidden window: {}", window_name);

                                            // Optional: Clean up if window is unmodified
                                            app_core.layout.remove_window_if_default(window_name);
                                        }
                                        Err(e) => {
                                            app_core.add_system_message(&format!(
                                                "Failed to hide window: {}",
                                                e
                                            ));
                                            tracing::error!(
                                                "Failed to hide window '{}': {}",
                                                window_name,
                                                e
                                            );
                                        }
                                    }

                                    app_core.ui_state.popup_menu = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                    app_core.needs_render = true;
                                } else if command.starts_with("__EDIT__") {
                                    // Edit window command
                                    let window_name = &command[8..]; // Skip "__EDIT__" prefix

                                    if let Some(window_def) =
                                        app_core.layout.get_window(window_name)
                                    {
                                        // Open window editor
                                        frontend.window_editor =
                                            Some(frontend::tui::window_editor::WindowEditor::new(
                                                window_def.clone(),
                                            ));
                                        app_core.ui_state.input_mode =
                                            data::ui_state::InputMode::WindowEditor;
                                        tracing::info!(
                                            "Opening window editor for: {}",
                                            window_name
                                        );
                                    } else {
                                        app_core.add_system_message(&format!(
                                            "Window '{}' not found",
                                            window_name
                                        ));
                                        tracing::warn!(
                                            "Window '{}' not found in layout",
                                            window_name
                                        );
                                    }

                                    app_core.ui_state.popup_menu = None;
                                    app_core.needs_render = true;
                                } else {
                                    // Close menu first
                                    app_core.ui_state.popup_menu = None;
                                    app_core.ui_state.submenu = None;
                                    app_core.ui_state.nested_submenu = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                    app_core.needs_render = true;

                                    // Check if this is an internal action, dot command, or game command
                                    if command.starts_with("action:") {
                                        // Internal action - handle it
                                        handle_menu_action(app_core, frontend, &command)?;
                                    } else if command.starts_with(".") {
                                        // Dot command - handle locally via handle_menu_action
                                        let action_command = format!("action:{}", &command[1..]); // Convert .addcolor to action:addcolor
                                        handle_menu_action(app_core, frontend, &action_command)?;
                                    } else if !command.is_empty() {
                                        // Game command - send to server
                                        tracing::info!("Sending context menu command: {}", command);
                                        return Ok(Some(format!("{}\n", command)));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
                return Ok(None);
            } else if app_core.ui_state.input_mode == InputMode::WindowEditor {
                // Handle window editor navigation and input
                if let Some(ref mut editor) = frontend.window_editor {
                    // Handle Ctrl+1..9 for section jumping (high priority - before menu keybinds)
                    if modifiers.contains(KeyModifiers::CONTROL) {
                        match code {
                            KeyCode::Char('1') => {
                                editor.jump_to_section(1);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('2') => {
                                editor.jump_to_section(2);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('3') => {
                                editor.jump_to_section(3);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('4') => {
                                editor.jump_to_section(4);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('5') => {
                                editor.jump_to_section(5);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('6') => {
                                editor.jump_to_section(6);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('7') => {
                                editor.jump_to_section(7);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('8') => {
                                editor.jump_to_section(8);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            KeyCode::Char('9') => {
                                editor.jump_to_section(9);
                                app_core.needs_render = true;
                                return Ok(None);
                            }
                            _ => {}
                        }
                    }

                    let key_event = crossterm::event::KeyEvent::new(code, modifiers);
                    let action = input_router::route_input(
                        key_event,
                        &app_core.ui_state.input_mode,
                        &app_core.config,
                    );

                    match action {
                        crate::core::menu_actions::MenuAction::NextField => {
                            editor.next();
                            app_core.needs_render = true;
                        }
                        crate::core::menu_actions::MenuAction::PreviousField => {
                            editor.previous();
                            app_core.needs_render = true;
                        }
                        crate::core::menu_actions::MenuAction::Toggle => {
                            if editor.is_on_checkbox() {
                                // Toggle only works on checkboxes
                                editor.toggle_field();
                                app_core.needs_render = true;
                            } else if editor.is_on_border_style() {
                                // Cycle border style dropdown
                                editor.cycle_border_style();
                                app_core.needs_render = true;
                            }
                        }
                        crate::core::menu_actions::MenuAction::Select => {
                            if editor.is_on_checkbox() {
                                // Enter/Select also toggles checkboxes
                                editor.toggle_field();
                                app_core.needs_render = true;
                            } else if editor.is_on_border_style() {
                                // Cycle border style dropdown
                                editor.cycle_border_style();
                                app_core.needs_render = true;
                            }
                        }
                        crate::core::menu_actions::MenuAction::Save => {
                            // Save window definition
                            let (width, height) = frontend.size();
                            if let Some(ref mut editor) = frontend.window_editor {
                                let window_def = editor.get_window_def().clone();

                                // Add to layout
                                if editor.is_new() {
                                    // New window - add to front of layout (so it appears on top)
                                    app_core.layout.windows.insert(0, window_def.clone());
                                    tracing::info!("Added new window: {}", window_def.name());

                                    // Create WindowState for new window WITHOUT destroying existing ones
                                    app_core.add_new_window(&window_def, width, height);
                                } else {
                                    // Editing existing - update it
                                    if let Some(existing) = app_core
                                        .layout
                                        .windows
                                        .iter_mut()
                                        .find(|w| w.name() == window_def.name())
                                    {
                                        *existing = window_def.clone();
                                        tracing::info!("Updated window: {}", window_def.name());

                                        // Update window position if size/position changed
                                        app_core.update_window_position(&window_def, width, height);
                                    }
                                }

                                // Mark layout as modified
                                app_core.mark_layout_modified();

                                // Close editor
                                frontend.window_editor = None;
                                app_core.ui_state.input_mode = InputMode::Normal;
                                app_core.needs_render = true;
                            }
                        }
                        crate::core::menu_actions::MenuAction::Delete => {
                            // Delete/Hide window
                            if let Some(ref mut editor) = frontend.window_editor {
                                let window_name = editor.get_window_def().name().to_string();

                                // Don't delete main window or command_input
                                if window_name == "main" || window_name == "command_input" {
                                    tracing::warn!("Cannot hide {} window", window_name);
                                } else {
                                    // Hide window (keep in layout, remove from UI)
                                    app_core.hide_window(&window_name);

                                    // Close editor
                                    frontend.window_editor = None;
                                    app_core.ui_state.input_mode = InputMode::Normal;
                                }
                            }
                            app_core.needs_render = true;
                        }
                        crate::core::menu_actions::MenuAction::Cancel => {
                            // Cancel editing and close
                            frontend.window_editor = None;
                            app_core.ui_state.input_mode = InputMode::Normal;
                            app_core.needs_render = true;
                        }
                        _ => {
                            // Route all other input to the editor's TextArea widgets
                            let key_event = crossterm::event::KeyEvent::new(code, modifiers);
                            let rt_key = crate::core::event_bridge::to_textarea_event(key_event);
                            editor.input(rt_key);
                            app_core.needs_render = true;
                        }
                    }
                }
                return Ok(None);
            } else if app_core.ui_state.input_mode == InputMode::Search {
                // Handle search input
                match code {
                    KeyCode::Enter => {
                        // Execute search
                        let pattern = app_core.ui_state.search_input.clone();
                        if !pattern.is_empty() {
                            let window_name = app_core.get_focused_window_name();
                            match frontend.execute_search(&window_name, &pattern) {
                                Ok(count) => {
                                    if count > 0 {
                                        tracing::info!("Found {} matches for '{}'", count, pattern);
                                    } else {
                                        tracing::info!("No matches found for '{}'", pattern);
                                    }
                                    app_core.needs_render = true;
                                }
                                Err(e) => {
                                    tracing::warn!("Invalid search regex '{}': {}", pattern, e);
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        // Insert character into search input
                        let pos = app_core.ui_state.search_cursor;
                        app_core.ui_state.search_input.insert(pos, c);
                        app_core.ui_state.search_cursor += 1;
                        app_core.needs_render = true;
                    }
                    KeyCode::Backspace => {
                        // Delete character before cursor
                        if app_core.ui_state.search_cursor > 0 {
                            app_core.ui_state.search_cursor -= 1;
                            app_core
                                .ui_state
                                .search_input
                                .remove(app_core.ui_state.search_cursor);
                            app_core.needs_render = true;
                        }
                    }
                    KeyCode::Left => {
                        // Move cursor left
                        if app_core.ui_state.search_cursor > 0 {
                            app_core.ui_state.search_cursor -= 1;
                            app_core.needs_render = true;
                        }
                    }
                    KeyCode::Right => {
                        // Move cursor right
                        if app_core.ui_state.search_cursor < app_core.ui_state.search_input.len() {
                            app_core.ui_state.search_cursor += 1;
                            app_core.needs_render = true;
                        }
                    }
                    KeyCode::Home => {
                        app_core.ui_state.search_cursor = 0;
                        app_core.needs_render = true;
                    }
                    KeyCode::End => {
                        app_core.ui_state.search_cursor = app_core.ui_state.search_input.len();
                        app_core.needs_render = true;
                    }
                    _ => {}
                }
            } else {
                // LAYER 3 & 4: Normal mode (no priority window)
                // Layer 3: Check user keybinds (keybinds.toml)
                // Layer 4: Fallback to CommandInput (typing)

                // Handle Enter key specially - always submit command, never keybind
                match code {
                    KeyCode::Enter => {
                        // Submit command from CommandInput widget
                        if let Some(command) = frontend.command_input_submit("command_input") {
                            // Special handling for .savelayout - needs terminal size
                            if command.starts_with(".savelayout ") || command == ".savelayout" {
                                let name = command
                                    .strip_prefix(".savelayout ")
                                    .unwrap_or("default")
                                    .trim();
                                tracing::info!(
                                    "[MAIN.RS] User entered .savelayout command: '{}'",
                                    name
                                );
                                let (width, height) = frontend.size();
                                tracing::info!(
                                    "[MAIN.RS] Terminal size from frontend: {}x{}",
                                    width,
                                    height
                                );
                                app_core.save_layout(name, width, height);
                                app_core.needs_render = true;
                            }
                            // Special handling for .loadlayout - needs terminal size
                            else if command.starts_with(".loadlayout ")
                                || command == ".loadlayout"
                            {
                                let name = command
                                    .strip_prefix(".loadlayout ")
                                    .unwrap_or("default")
                                    .trim();
                                tracing::info!(
                                    "[MAIN.RS] User entered .loadlayout command: '{}'",
                                    name
                                );
                                let (width, height) = frontend.size();
                                tracing::info!(
                                    "[MAIN.RS] Terminal size from frontend: {}x{}",
                                    width,
                                    height
                                );
                                if let Some((theme_id, theme)) =
                                    app_core.load_layout(name, width, height)
                                {
                                    frontend.update_theme_cache(theme_id, theme);
                                }
                                app_core.needs_render = true;
                            }
                            // Special handling for .resize - scales windows proportionally
                            else if command == ".resize" {
                                tracing::info!("[MAIN.RS] User entered .resize command");
                                let (width, height) = frontend.size();
                                tracing::info!(
                                    "[MAIN.RS] Terminal size from frontend: {}x{}",
                                    width,
                                    height
                                );
                                app_core.resize_windows(width, height);
                                app_core.needs_render = true;
                            } else {
                                let to_send = app_core.send_command(command)?;
                                // Check if this is an action command
                                if to_send.starts_with("action:") {
                                    handle_menu_action(app_core, frontend, &to_send)?;
                                    app_core.needs_render = true;
                                } else {
                                    app_core.needs_render = true;
                                    return Ok(Some(to_send));
                                }
                            }
                        }
                    }
                    _ => {
                        // Check for non-command-input keybinds first (Tab, F12, Ctrl+R, Ctrl+T, etc.)
                        let key_event = crossterm::event::KeyEvent::new(code, modifiers);
                        if let Some(action) = app_core.keybind_map.get(&key_event).cloned() {
                            // Check if this is a command-input action that should be handled by the widget
                            let is_command_input_action = matches!(&action,
                                config::KeyBindAction::Action(s) if matches!(s.as_str(),
                                    "cursor_left" | "cursor_right" | "cursor_word_left" | "cursor_word_right" |
                                    "cursor_home" | "cursor_end" | "cursor_backspace" | "cursor_delete" |
                                    "previous_command" | "next_command" | "send_last_command" | "send_second_last_command"
                                )
                            );

                            if is_command_input_action {
                                // Route to CommandInput widget instead of app_core
                                let available_commands = app_core.get_available_commands();
                                let available_window_names = app_core.get_window_names();
                                frontend.command_input_key(
                                    "command_input",
                                    code,
                                    modifiers,
                                    &available_commands,
                                    &available_window_names,
                                );
                                app_core.needs_render = true;
                            } else {
                                // Execute non-command-input keybind actions
                                match app_core.execute_keybind_action(&action) {
                                    Ok(commands) => {
                                        // Return first command from macro (if any) to be sent to server
                                        if let Some(cmd) = commands.into_iter().next() {
                                            app_core.needs_render = true;
                                            return Ok(Some(cmd));
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!("Keybind action failed: {}", e);
                                    }
                                }
                                app_core.needs_render = true;
                            }
                        } else {
                            // No keybind - route to CommandInput widget for typing
                            let available_commands = app_core.get_available_commands();
                            let available_window_names = app_core.get_window_names();
                            frontend.command_input_key(
                                "command_input",
                                code,
                                modifiers,
                                &available_commands,
                                &available_window_names,
                            );
                            app_core.needs_render = true;
                        }
                    }
                }
            }
        }
        FrontendEvent::Resize { width, height } => {
            // Automatically resize layout when terminal is resized (using VellumFE algorithm)
            app_core.resize_windows(width, height);
            app_core.needs_render = true;
        }
        _ => {}
    }

    Ok(None)
}
