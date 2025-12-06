//! Menu Action Handler
//!
//! Processes menu action commands from the TUI.

use crate::config;
use crate::core::AppCore;
use crate::data::ui_state::{InputMode, PopupMenu, PopupMenuItem};
use crate::frontend::tui::menu_builders;
use crate::frontend::tui::TuiFrontend;
use anyhow::Result;

/// Handle menu action commands
pub fn handle_menu_action(
    app_core: &mut AppCore,
    frontend: &mut TuiFrontend,
    command: &str,
) -> Result<()> {
    if let Some(layout_name) = command.strip_prefix("action:loadlayout:") {
        // Load a layout with proper terminal size
        tracing::info!("[MENU_ACTIONS] Menu action loadlayout: '{}'", layout_name);
        let (width, height) = frontend.size();
        tracing::info!(
            "[MENU_ACTIONS] Terminal size from frontend: {}x{}",
            width,
            height
        );
        if let Some((theme_id, theme)) = app_core.load_layout(layout_name, width, height) {
            frontend.update_theme_cache(theme_id, theme);
        }
    } else if let Some(widget_type) = command.strip_prefix("action:createwindow:") {
        // Create a new window with the specified widget type
        // Get template for this widget type (use widget type name as template name)
        if let Some(_template) = config::Config::get_window_template(widget_type) {
            // Open window editor with template (proper defaults + marked as new)
            // Use new_window_with_layout for spacers to enable auto-naming
            frontend.window_editor = Some(
                crate::frontend::tui::window_editor::WindowEditor::new_window_with_layout(
                    widget_type.to_string(),
                    &app_core.layout,
                ),
            );
            app_core.ui_state.input_mode = InputMode::WindowEditor;
        } else {
            tracing::warn!("No template found for widget type: {}", widget_type);
        }
    } else if let Some(window_name) = command.strip_prefix("action:editwindow:") {
        // Edit an existing window
        // Find the window definition
        if let Some(window_def) = app_core
            .layout
            .windows
            .iter()
            .find(|w| w.name() == window_name)
            .cloned()
        {
            // Open window editor
            frontend.window_editor = Some(
                crate::frontend::tui::window_editor::WindowEditor::new(window_def)
            );
            app_core.ui_state.input_mode = InputMode::WindowEditor;
        } else {
            tracing::warn!("Window not found for editing: {}", window_name);
        }
    } else if let Some(window_name) = command.strip_prefix("action:showwindow:") {
        // Add/show the window (from template)
        // Get terminal size for window positioning
        let (width, height) = frontend.size();

        // Show window from layout template
        app_core.show_window(window_name, width, height);

        // Close menus
        app_core.ui_state.popup_menu = None;
        app_core.ui_state.submenu = None;
        app_core.ui_state.input_mode = InputMode::Normal;
        app_core.needs_render = true;
    } else if let Some(window_name) = command.strip_prefix("action:hidewindow:") {
        // Hide a visible window
        app_core.hide_window(window_name);
    } else {
        match command {
            "action:addwindow" => {
                // Close submenu if it exists
                app_core.ui_state.submenu = None;
                // Show widget type picker using proper build_add_window_menu
                app_core.ui_state.popup_menu = Some(PopupMenu::new(
                    app_core.build_add_window_menu(),
                    (40, 12),
                ));
                // Stay in Menu mode
                app_core.ui_state.input_mode = InputMode::Menu;
            }
            "action:hidewindow" => {
                // Close submenu if it exists
                app_core.ui_state.submenu = None;
                // Show window picker for hiding
                app_core.ui_state.popup_menu = Some(PopupMenu::new(
                    menu_builders::build_hidewindow_picker(app_core),
                    (40, 12),
                ));
                // Stay in Menu mode
                app_core.ui_state.input_mode = InputMode::Menu;
            }
            "action:listwindows" => {
                // List all windows
                app_core.send_command(".windows".to_string())?;

                // Close menu and return to normal mode
                app_core.ui_state.popup_menu = None;
                app_core.ui_state.input_mode = InputMode::Normal;
                app_core.needs_render = true;
            }
            "action:highlights" => {
                // Open highlight browser
                frontend.highlight_browser =
                    Some(crate::frontend::tui::highlight_browser::HighlightBrowser::new(
                        &app_core.config.highlights,
                    ));
                app_core.ui_state.input_mode = InputMode::HighlightBrowser;
            }
            "action:addhighlight" => {
                // Open highlight form for creating new highlight
                frontend.highlight_form =
                    Some(crate::frontend::tui::highlight_form::HighlightFormWidget::new());
                app_core.ui_state.input_mode = InputMode::HighlightForm;
            }
            "action:keybinds" => {
                // Open keybind browser
                frontend.keybind_browser = Some(
                    crate::frontend::tui::keybind_browser::KeybindBrowser::new(
                        &app_core.config.keybinds,
                    ),
                );
                app_core.ui_state.input_mode = InputMode::KeybindBrowser;
            }
            "action:addkeybind" => {
                // Open keybind form for creating new keybind
                frontend.keybind_form = Some(
                    crate::frontend::tui::keybind_form::KeybindFormWidget::new()
                );
                app_core.ui_state.input_mode = InputMode::KeybindForm;
            }
            "action:colors" => {
                // Open color palette browser
                frontend.color_palette_browser = Some(
                    crate::frontend::tui::color_palette_browser::ColorPaletteBrowser::new(
                        app_core.config.colors.color_palette.clone(),
                    ),
                );
                app_core.ui_state.input_mode = InputMode::ColorPaletteBrowser;
            }
            "action:addcolor" => {
                // Open color form for creating new palette color
                frontend.color_form = Some(
                    crate::frontend::tui::color_form::ColorForm::new_create()
                );
                app_core.ui_state.input_mode = InputMode::ColorForm;
            }
            "action:uicolors" => {
                // Open UI colors browser
                frontend.uicolors_browser = Some(
                    crate::frontend::tui::uicolors_browser::UIColorsBrowser::new(&app_core.config.colors),
                );
                app_core.ui_state.input_mode = InputMode::UIColorsBrowser;
            }
            "action:spellcolors" => {
                // Open spell colors browser
                frontend.spell_color_browser =
                    Some(crate::frontend::tui::spell_color_browser::SpellColorBrowser::new(
                        &app_core.config.colors.spell_colors,
                    ));
                app_core.ui_state.input_mode = InputMode::SpellColorsBrowser;
            }
            "action:addspellcolor" => {
                // Open spell color form for creating new spell color
                frontend.spell_color_form =
                    Some(crate::frontend::tui::spell_color_form::SpellColorFormWidget::new());
                app_core.ui_state.input_mode = InputMode::SpellColorForm;
            }
            "action:settings" => {
                // Open settings editor
                let settings_items = menu_builders::build_settings_items(&app_core.config);
                frontend.settings_editor = Some(
                    crate::frontend::tui::settings_editor::SettingsEditor::new(settings_items),
                );
                app_core.ui_state.input_mode = InputMode::SettingsEditor;
            }
            "action:themes" => {
                // Open theme browser (includes built-in and custom themes)
                frontend.theme_browser = Some(
                    crate::frontend::tui::theme_browser::ThemeBrowser::new(
                        app_core.config.active_theme.clone(),
                        app_core.config.character.as_deref(),
                    )
                );
                app_core.ui_state.input_mode = InputMode::ThemeBrowser;
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
                frontend.theme_editor = Some(
                    crate::frontend::tui::theme_editor::ThemeEditor::new_edit(&current_theme)
                );
                app_core.ui_state.input_mode = InputMode::ThemeEditor;
            }
            "action:editwindow" => {
                // Open window picker for editing
                let window_names: Vec<String> = app_core
                    .layout
                    .windows
                    .iter()
                    .map(|w| w.name().to_string())
                    .collect();

                let items: Vec<PopupMenuItem> = window_names
                    .iter()
                    .map(|name| PopupMenuItem {
                        text: name.clone(),
                        command: format!("action:editwindow:{}", name),
                        disabled: false,
                    })
                    .collect();

                // Close submenu if it exists
                app_core.ui_state.submenu = None;
                // Create new popup menu for window selection
                app_core.ui_state.popup_menu = Some(PopupMenu::new(items, (40, 12)));
                // Stay in Menu mode
                app_core.ui_state.input_mode = InputMode::Menu;
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
