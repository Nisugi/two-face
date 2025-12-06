use super::*;
use super::performance_stats;
use super::colors::{blend_colors_hex, color_to_hex_string, normalize_color, parse_hex_color};

impl TuiFrontend {
    pub(crate) fn sync_text_windows(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Text(text_content) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create TextWindow for this window
                let text_window = self.widget_manager.text_windows.entry(name.clone()).or_insert_with(|| {
                    let mut tw =
                        text_window::TextWindow::new(&text_content.title, text_content.max_lines);

                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        tw.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        tw.set_border_sides(def.base().border_sides.clone());
                        tw.set_background_color(colors.background.clone());
                        tw.set_text_color(colors.text.clone());
                        tw.set_content_align(def.base().content_align.clone());
                        if let crate::config::WindowDef::Text { data, .. } = def {
                            tw.set_show_timestamps(data.show_timestamps);
                        } else {
                            tw.set_show_timestamps(app_core.config.ui.show_timestamps);
                        }
                    } else {
                        tw.set_show_timestamps(app_core.config.ui.show_timestamps);
                    }

                    // Set highlights from config
                    let highlights_vec: Vec<_> =
                        app_core.config.highlights.values().cloned().collect();
                    tw.set_highlights(highlights_vec);

                    tw
                });

                // Existing text windows need to reapply theme-derived settings when themes change
                if let Some(def) = window_def {
                    let colors = resolve_window_colors(def.base(), theme);
                    text_window.set_border_config(
                        def.base().show_border,
                        Some(def.base().border_style.clone()),
                        colors.border.clone(),
                    );
                    text_window.set_border_sides(def.base().border_sides.clone());
                    text_window.set_background_color(colors.background.clone());
                    text_window.set_text_color(colors.text.clone());
                    text_window.set_content_align(def.base().content_align.clone());
                    if let crate::config::WindowDef::Text { data, .. } = def {
                        text_window.set_show_timestamps(data.show_timestamps);
                    } else {
                        text_window.set_show_timestamps(app_core.config.ui.show_timestamps);
                    }
                }

                // Update width for proper wrapping
                text_window.set_width(window.position.width);

                // Get last synced generation
                let last_synced_gen = self.widget_manager.last_synced_generation.get(name).copied().unwrap_or(0);
                let current_gen = text_content.generation;

                // Check if there are new lines to sync (generation changed)
                if current_gen > last_synced_gen {
                    // Calculate how many lines to add
                    // If generation delta > line count, we need to resync entire buffer
                    let gen_delta = (current_gen - last_synced_gen) as usize;
                    let needs_full_resync = gen_delta > text_content.lines.len();

                    if needs_full_resync {
                        // Full resync - clear and add all lines
                        tracing::trace!(
                            "Text window '{}': full resync (gen delta {} > line count {})",
                            name,
                            gen_delta,
                            text_content.lines.len()
                        );
                        text_window.clear();
                    }

                    // Determine how many lines to add
                    let lines_to_add = if needs_full_resync {
                        text_content.lines.len() // Add all lines
                    } else {
                        gen_delta.min(text_content.lines.len()) // Add only new lines
                    };

                    let skip_count = text_content.lines.len().saturating_sub(lines_to_add);
                    for line in text_content.lines.iter().skip(skip_count) {
                        // Convert our data format to TextWindow's format
                        for segment in &line.segments {
                            // Map data layer SpanType to TextWindow SpanType
                            use crate::data::SpanType as DataSpanType;
                            let tw_span_type = match segment.span_type {
                                DataSpanType::Normal => text_window::SpanType::Normal,
                                DataSpanType::Link => text_window::SpanType::Link,
                                DataSpanType::Monsterbold => text_window::SpanType::Monsterbold,
                                DataSpanType::Spell => text_window::SpanType::Spell,
                                DataSpanType::Speech => text_window::SpanType::Speech,
                            };

                            let styled_text = text_window::StyledText {
                                content: segment.text.clone(),
                                fg: segment
                                    .fg
                                    .as_ref()
                                    .and_then(|hex| parse_hex_color(hex).ok()),
                                bg: segment
                                    .bg
                                    .as_ref()
                                    .and_then(|hex| parse_hex_color(hex).ok()),
                                bold: segment.bold,
                                span_type: tw_span_type,
                                link_data: segment.link_data.as_ref().map(|ld| {
                                    text_window::LinkData {
                                        exist_id: ld.exist_id.clone(),
                                        noun: ld.noun.clone(),
                                        text: ld.text.clone(),
                                        coord: ld.coord.clone(),
                                    }
                                }),
                            };
                            text_window.add_text(styled_text);
                        }
                        // Finish the line with actual window width
                        text_window.finish_line(window.position.width);
                    }

                    // Update last synced generation
                    self.widget_manager.last_synced_generation
                        .insert(name.clone(), current_gen);
                }

                // Sync scroll offset from data layer to TextWindow
                // TextContent scroll_offset is lines from bottom (0 = live view)
                // TextWindow scroll methods handle this the same way
                // Note: TextWindow doesn't have a direct set_scroll_offset, so we'd need to
                // track the last known offset and call scroll_up/scroll_down as needed
                // For now, this is handled by user input events that modify both layers
            } else if let crate::data::WindowContent::Room(_room_content) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create RoomWindow for this window
                if !self.widget_manager.room_windows.contains_key(name) {
                    let mut room_window = room_window::RoomWindow::new("Room".to_string());

                    // Configure RoomWindow with settings from WindowDef
                    if let Some(crate::config::WindowDef::Room { data, .. }) = window_def {
                        // Set component visibility from config
                        room_window.set_component_visible("room desc", data.show_desc);
                        room_window.set_component_visible("room objs", data.show_objs);
                        room_window.set_component_visible("room players", data.show_players);
                        room_window.set_component_visible("room exits", data.show_exits);
                    }

                    self.widget_manager.room_windows.insert(name.clone(), room_window);
                    tracing::debug!("Created RoomWindow widget for '{}' during sync", name);
                }
            }
            // TODO: Add similar widget creation for other complex widget types as they're implemented:
            // - Progress bars (if they need stateful widgets beyond simple rendering)
            // - Countdown timers (if they need stateful widgets)
            // - Compass (if it needs stateful widgets)
            // - Indicator (if it needs stateful widgets)
            // - Hands/Inventory (if they need stateful widgets)
            // - Dashboard (if it needs stateful widgets)
            // Currently these render directly in the render loop without needing persistent widget state,
            // but if they gain more complex behavior (animations, interactions, etc.), they'll need
            // to be created here during sync just like Room and Text windows.
        }
    }

    /// Sync command input widgets with window configuration
    pub(crate) fn sync_command_inputs(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if !matches!(
                window.content,
                crate::data::WindowContent::CommandInput { .. }
            ) {
                continue;
            }

            let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);
            let (base_config, cmd_data) = match window_def {
                Some(crate::config::WindowDef::CommandInput { base, data }) => {
                    (Some(base.clone()), Some(data.clone()))
                }
                Some(def) => (Some(def.base().clone()), None),
                None => (None, None),
            };

            // Ensure the backing widget exists so we can apply configuration
            let cmd_input = self.widget_manager.command_inputs.entry(name.clone()).or_insert_with(|| {
                let mut widget = command_input::CommandInput::new(1000);
                if let Some(base) = base_config.as_ref() {
                    let title = base
                        .title
                        .clone()
                        .or_else(|| {
                            if base.name.is_empty() {
                                None
                            } else {
                                Some(base.name.clone())
                            }
                        })
                        .unwrap_or_else(|| "Command".to_string());
                    widget.set_title(title);
                } else {
                    widget.set_title("Command".to_string());
                }
                widget
            });

            if let Some(base) = base_config.as_ref() {
                let title = base
                    .title
                    .clone()
                    .or_else(|| {
                        if base.name.is_empty() {
                            None
                        } else {
                            Some(base.name.clone())
                        }
                    })
                    .unwrap_or_else(|| "Command".to_string());
                cmd_input.set_title(title);
                let border_color = normalize_color(&base.border_color)
                    .or_else(|| color_to_hex_string(&theme.window_border));
                cmd_input.set_border_config(
                    base.show_border,
                    Some(base.border_style.clone()),
                    border_color,
                );
                cmd_input.set_border_sides(base.border_sides.clone());
                cmd_input.set_show_title(base.show_title);
                let background_color = if base.transparent_background {
                    None
                } else {
                    normalize_color(&base.background_color)
                        .or_else(|| color_to_hex_string(&theme.window_background))
                };
                cmd_input.set_background_color(background_color);
                let text_color = cmd_data
                    .as_ref()
                    .and_then(|d| normalize_color(&d.text_color))
                    .or_else(|| normalize_color(&base.text_color))
                    .or_else(|| color_to_hex_string(&theme.text_primary));
                cmd_input.set_text_color(text_color);
                let cursor_fg = cmd_data
                    .as_ref()
                    .and_then(|d| normalize_color(&d.cursor_color))
                    .or_else(|| color_to_hex_string(&theme.window_background));
                let cursor_bg = cmd_data
                    .as_ref()
                    .and_then(|d| normalize_color(&d.cursor_background_color))
                    .or_else(|| color_to_hex_string(&theme.text_primary));
                cmd_input.set_cursor_colors(cursor_fg, cursor_bg);
            }
        }
    }

    /// Sync inventory window data - create/configure widgets
    pub(crate) fn sync_inventory_windows(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find inventory windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            // Check for both Inventory and Text content types
            let text_content = match &window.content {
                crate::data::WindowContent::Inventory(content) => Some(content),
                crate::data::WindowContent::Text(content)
                    if name == "inventory"
                        || content.title.to_lowercase().contains("inventory") =>
                {
                    Some(content)
                }
                _ => None,
            };

            if let Some(text_content) = text_content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create InventoryWindow for this window
                if !self.widget_manager.inventory_windows.contains_key(name) {
                    let inv_window =
                        inventory_window::InventoryWindow::new(text_content.title.clone());
                    self.widget_manager.inventory_windows.insert(name.clone(), inv_window);
                    tracing::debug!("Created InventoryWindow widget for '{}'", name);
                }

                // Update configuration and content from WindowDef if present
                if let Some(inv_window) = self.widget_manager.inventory_windows.get_mut(name) {
                    inv_window.set_title(text_content.title.clone());
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        inv_window.set_border_config(def.base().show_border, colors.border.clone());
                        inv_window.set_transparent_background(def.base().transparent_background);
                        inv_window.set_background_color(colors.background.clone());
                        inv_window.set_text_color(colors.text.clone());
                    }

                    // Change detection: only sync if content changed (using generation)
                    let last_synced_gen =
                        self.widget_manager.last_synced_generation.get(name).copied().unwrap_or(0);
                    let current_gen = text_content.generation;

                    if current_gen != last_synced_gen {
                        // Content changed - sync text lines from WindowContent to widget
                        inv_window.clear();
                        tracing::debug!("Syncing inventory widget '{}' with {} lines (gen changed from {} to {})",
                            name, text_content.lines.len(), last_synced_gen, current_gen);
                        for line in &text_content.lines {
                            for segment in &line.segments {
                                inv_window.add_segment(segment.clone());
                            }
                            inv_window.finish_line();
                        }
                        // Update last synced generation
                        self.widget_manager.last_synced_generation
                            .insert(name.clone(), current_gen);
                    }
                } else {
                    tracing::warn!(
                        "Inventory widget '{}' not found in inventory_windows HashMap!",
                        name
                    );
                }
            }
        }
    }

    /// Sync spells window data - create/configure widgets
    pub(crate) fn sync_spells_windows(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find spells windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            // Check for Spells content type
            let text_content = match &window.content {
                crate::data::WindowContent::Spells(content) => Some(content),
                _ => None,
            };

            if let Some(text_content) = text_content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create SpellsWindow for this window
                if !self.widget_manager.spells_windows.contains_key(name) {
                    let spells_window =
                        spells_window::SpellsWindow::new(text_content.title.clone());
                    self.widget_manager.spells_windows.insert(name.clone(), spells_window);
                    tracing::debug!("Created SpellsWindow widget for '{}'", name);
                }

                // Update configuration and content from WindowDef if present
                if let Some(spells_window) = self.widget_manager.spells_windows.get_mut(name) {
                    spells_window.set_title(text_content.title.clone());
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        spells_window.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        spells_window.set_transparent_background(def.base().transparent_background);
                        spells_window.set_background_color(colors.background.clone());
                        spells_window.set_text_color(colors.text.clone());
                    }

                    // Change detection: only sync if content changed (using generation)
                    let last_synced_gen =
                        self.widget_manager.last_synced_generation.get(name).copied().unwrap_or(0);
                    let current_gen = text_content.generation;

                    if current_gen != last_synced_gen {
                        // Content changed - sync text lines from WindowContent to widget
                        spells_window.clear();
                        tracing::debug!(
                            "Syncing spells widget '{}' with {} lines (gen changed from {} to {})",
                            name,
                            text_content.lines.len(),
                            last_synced_gen,
                            current_gen
                        );
                        for line in &text_content.lines {
                            for segment in &line.segments {
                                spells_window.add_text(
                                    segment.text.clone(),
                                    segment.fg.clone(),
                                    segment.bg.clone(),
                                    segment.bold,
                                    segment.span_type,
                                    segment.link_data.clone(),
                                );
                            }
                            spells_window.finish_line();
                        }
                        // Update last synced generation
                        self.widget_manager.last_synced_generation
                            .insert(name.clone(), current_gen);
                    }
                } else {
                    tracing::warn!(
                        "Spells widget '{}' not found in spells_windows HashMap!",
                        name
                    );
                }
            }
        }
    }

    /// Sync progress bar data - create/configure widgets
    pub(crate) fn sync_progress_bars(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find progress bar windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Progress(progress_data) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create ProgressBar for this window
                if !self.widget_manager.progress_bars.contains_key(name) {
                    let label = window_def
                        .and_then(|def| def.base().title.as_ref()).cloned()
                        .unwrap_or_else(|| progress_data.label.clone());

                    let bar = progress_bar::ProgressBar::new(&label);
                    self.widget_manager.progress_bars.insert(name.clone(), bar);
                    tracing::debug!("Created ProgressBar widget for '{}'", name);
                }

                // Update configuration and value
                if let Some(progress_bar) = self.widget_manager.progress_bars.get_mut(name) {
                    // Set value from game data
                    if let Some(ref custom_text) = progress_data.color {
                        // color field is being used as custom text (e.g., "clear as a bell")
                        progress_bar.set_value_with_text(
                            progress_data.value,
                            progress_data.max,
                            Some(custom_text.clone()),
                        );
                    } else {
                        progress_bar
                            .set_value(progress_data.value, progress_data.max);
                    }

                    // Apply window config from WindowDef
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        progress_bar.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );

                        // Get bar color from ProgressWidgetData, or fallback to VellumFE defaults
                        if let crate::config::WindowDef::Progress { data, .. } = def {
                            let bar_color = if let Some(ref color) = data.color {
                                Some(color.clone())
                            } else {
                                // Fallback to VellumFE template colors for known progress bars
                                match name.as_str() {
                                    "health" => Some("#6e0202".to_string()),     // Dark red
                                    "mana" => Some("#08086d".to_string()),       // Dark blue
                                    "stamina" => Some("#bd7b00".to_string()),    // Orange
                                    "spirit" => Some("#6e727c".to_string()),     // Gray
                                    "encumlevel" => Some("#ffff00".to_string()), // Yellow
                                    "pbarStance" => Some("#ffa500".to_string()), // Orange
                                    "mindState" => Some("#9370db".to_string()),  // Purple
                                    "lblBPs" => Some("#ff4500".to_string()),     // Orange-red
                                    _ => None,
                                }
                            };

                            if let Some(color) = bar_color {
                                progress_bar.set_colors(Some(color), None);
                            }
                        }

                        // Apply text color
                        progress_bar.set_text_color(colors.text.clone());

                        // Apply transparent background setting
                        progress_bar.set_transparent_background(def.base().transparent_background);
                        progress_bar.set_background_color(colors.background.clone());
                    }
                }
            }
        }
    }

    /// Sync countdown data - create/configure countdown widgets
    pub(crate) fn sync_countdowns(&mut self, app_core: &crate::core::AppCore, theme: &crate::theme::AppTheme) {
        // Find countdown windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Countdown(countdown_data) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create Countdown for this window
                if !self.widget_manager.countdowns.contains_key(name) {
                    let label = window_def
                        .and_then(|def| def.base().title.as_ref()).cloned()
                        .unwrap_or_else(|| name.clone());

                    let countdown = countdown::Countdown::new(&label);
                    self.widget_manager.countdowns.insert(name.clone(), countdown);
                    tracing::debug!("Created Countdown widget for '{}'", name);
                }

                // Update configuration and value
                if let Some(countdown_widget) = self.widget_manager.countdowns.get_mut(name) {
                    // Set end time from game data
                    countdown_widget.set_end_time(countdown_data.end_time);

                    // Apply window config from WindowDef
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        countdown_widget.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );

                        // Get icon from CountdownWidgetData
                        if let crate::config::WindowDef::Countdown { data, .. } = def {
                            if let Some(icon) = data.icon {
                                countdown_widget.set_icon(icon);
                            }
                        }

                        countdown_widget.set_text_color(colors.text.clone());
                        countdown_widget
                            .set_transparent_background(def.base().transparent_background);
                    }
                }
            }
        }
    }

    /// Sync active effects data - create/configure active effects widgets
    pub(crate) fn sync_active_effects(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find active effects windows in ui_state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::ActiveEffects(effects_content) = &window.content {
                // Look up the WindowDef from layout to get config
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Get or create ActiveEffects for this window
                if !self.widget_manager.active_effects_windows.contains_key(name) {
                    let label = window_def
                        .and_then(|def| def.base().title.as_ref()).cloned()
                        .unwrap_or_else(|| name.clone());

                    let widget = active_effects::ActiveEffects::new(
                        &label,
                        effects_content.category.clone(),
                    );
                    self.widget_manager.active_effects_windows.insert(name.clone(), widget);
                    tracing::debug!("Created ActiveEffects widget for '{}'", name);
                }

                // Update effects data and configuration
                if let Some(widget) = self.widget_manager.active_effects_windows.get_mut(name) {
                    let previous_scroll = widget.scroll_position();

                    // Clear existing effects
                    widget.clear();

                    // Add all effects from content
                    for effect in &effects_content.effects {
                        widget.add_or_update_effect(
                            effect.id.clone(),
                            effect.text.clone(),
                            effect.value,
                            effect.time.clone(),
                            effect.bar_color.clone(),
                            effect.text_color.clone(),
                        );
                    }

                    widget.restore_scroll_position(previous_scroll);

                    // Apply window config from WindowDef
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        widget.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(def.base().border_sides.clone());
                        widget.set_transparent_background(def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                        widget.set_text_color(colors.text.clone());
                    }
                }
            }
        }
    }

    /// Sync spacer widget data from AppCore to spacer widgets
    pub(crate) fn sync_spacer_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find all Spacer windows in the UI state (Empty content + Spacer widget type)
        for (name, window) in &app_core.ui_state.windows {
            if window.widget_type == crate::data::WidgetType::Spacer {
                // Ensure spacer widget exists in cache
                if !self.widget_manager.spacer_widgets.contains_key(name) {
                    let widget = spacer::Spacer::new();
                    self.widget_manager.spacer_widgets.insert(name.clone(), widget);
                }

                // Update spacer widget configuration
                if let Some(spacer_widget) = self.widget_manager.spacer_widgets.get_mut(name) {
                    // Apply window configuration from layout
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        spacer_widget.set_background_color(colors.background.clone());
                        spacer_widget
                            .set_transparent_background(window_def.base().transparent_background);
                    }
                }
            }
        }
    }

    /// Sync indicator widget data from AppCore to indicator widgets
    pub(crate) fn sync_indicator_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find all Indicator windows in the UI state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Indicator(indicator_data) = &window.content {
                // Ensure indicator widget exists in cache
                if !self.widget_manager.indicator_widgets.contains_key(name) {
                    let widget = indicator::Indicator::new(name);
                    self.widget_manager.indicator_widgets.insert(name.clone(), widget);
                }

                // Update indicator widget content and configuration
                if let Some(indicator_widget) = self.widget_manager.indicator_widgets.get_mut(name) {
                    // Set status (which determines if it's active/shown)
                    indicator_widget.set_status(&indicator_data.status);

                    // Apply window configuration from layout
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        indicator_widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        indicator_widget.set_border_sides(window_def.base().border_sides.clone());
                        indicator_widget.set_title(
                            window_def
                                .base()
                                .title
                                .clone()
                                .unwrap_or_else(|| name.clone()),
                        );
                        indicator_widget.set_background_color(colors.background.clone());
                        indicator_widget
                            .set_transparent_background(window_def.base().transparent_background);

                        // Set custom colors if provided
                        if let Some(ref color) = indicator_data.color {
                            indicator_widget.set_colors("#555555".to_string(), color.clone());
                        }
                    }
                }
            }
        }
    }

    /// Sync targets widget data from AppCore to targets widgets
    pub(crate) fn sync_targets_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Targets { targets_text } = &window.content {
                // Ensure widget exists
                if !self.widget_manager.targets_widgets.contains_key(name) {
                    let widget = targets::Targets::new(name);
                    self.widget_manager.targets_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.targets_widgets.get_mut(name) {
                    widget.set_targets_from_text(targets_text);

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        if let Some(ref color) = colors.text {
                            widget.set_bar_color(color.clone());
                        }
                    }
                }
            }
        }
    }

    /// Sync players widget data from AppCore to players widgets
    pub(crate) fn sync_players_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Players { players_text } = &window.content {
                // Ensure widget exists
                if !self.widget_manager.players_widgets.contains_key(name) {
                    let widget = players::Players::new(name);
                    self.widget_manager.players_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.players_widgets.get_mut(name) {
                    widget.set_players_from_text(players_text);

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        if let Some(ref color) = colors.text {
                            widget.set_bar_color(color.clone());
                        }
                    }
                }
            }
        }
    }

    /// Sync dashboard widget data from AppCore to dashboard widgets
    pub(crate) fn sync_dashboard_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Dashboard { indicators } = &window.content {
                // Ensure widget exists
                if !self.widget_manager.dashboard_widgets.contains_key(name) {
                    // Default to horizontal layout - can be configured via WindowDef later
                    let widget =
                        dashboard::Dashboard::new(name, dashboard::DashboardLayout::Horizontal);
                    self.widget_manager.dashboard_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.dashboard_widgets.get_mut(name) {
                    // Update indicator values
                    for (id, value) in indicators {
                        widget.set_indicator_value(id, *value);
                    }

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                        widget.set_content_align(window_def.base().content_align.clone());

                        if let crate::config::WindowDef::Dashboard { data, .. } = window_def {
                            widget.set_layout(dashboard::DashboardLayout::from_str(&data.layout));
                            widget.set_spacing(data.spacing);
                            widget.set_hide_inactive(data.hide_inactive);
                            widget.clear_indicators();
                            for def in &data.indicators {
                                let colors = if def.colors.is_empty() {
                                    vec!["#ffffff".to_string()]
                                } else {
                                    def.colors.clone()
                                };
                                widget.add_indicator(def.id.clone(), def.icon.clone(), colors);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Sync tabbed text window data from AppCore to tabbed text widgets
    pub(crate) fn sync_tabbed_text_windows(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::TabbedText(tabbed_content) = &window.content {
                let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);

                // Ensure widget exists - create if needed
                if !self.widget_manager.tabbed_text_windows.contains_key(name) {
                    let tabs: Vec<(String, Vec<String>, bool)> = tabbed_content
                        .tabs
                        .iter()
                        .map(|t| {
                            (
                                t.definition.name.clone(),
                                t.definition.streams.clone(),
                                t.definition.show_timestamps,
                            )
                        })
                        .collect();

                    let max_lines =
                        if let Some(crate::config::WindowDef::TabbedText { data, .. }) = window_def
                        {
                            data.buffer_size
                        } else {
                            1000 // fallback
                        };

                    let widget =
                        tabbed_text_window::TabbedTextWindow::with_tabs(name, tabs, max_lines);
                    self.widget_manager.tabbed_text_windows.insert(name.clone(), widget);
                }

                // Apply configuration and sync content
                if let Some(widget) = self.widget_manager.tabbed_text_windows.get_mut(name) {
                    if let Some(def) = window_def {
                        let colors = resolve_window_colors(def.base(), theme);
                        widget.set_border_config(
                            def.base().show_border,
                            Some(def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(def.base().border_sides.clone());
                        widget.set_transparent_background(def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                        widget.set_content_align(def.base().content_align.clone());
                        widget.apply_window_colors(colors.text.clone(), colors.background.clone());

                        if let crate::config::WindowDef::TabbedText { data, .. } = def {
                            let tab_position = tabbed_text_window::TabBarPosition::from_str(
                                &data.tab_bar_position,
                            );
                            widget.set_tab_bar_position(tab_position);
                            widget.set_tab_colors(
                                data.tab_active_color.clone(),
                                data.tab_inactive_color.clone(),
                                data.tab_unread_color.clone(),
                            );
                            if let Some(prefix) = data.tab_unread_prefix.clone() {
                                widget.set_unread_prefix(prefix);
                            }
                        }
                    }

                    // Set active tab
                    widget.switch_to_tab(tabbed_content.active_tab_index);

                    // Sync content for each tab
                    for (i, tab_state) in tabbed_content.tabs.iter().enumerate() {
                        if let Some(text_window) = widget.get_tab_window_mut(i) {
                            text_window
                                .set_show_timestamps(tab_state.definition.show_timestamps);
                            let tab_sync_key = format!("{}:{}", name, tab_state.definition.name);
                            let last_synced_gen = self
                                .widget_manager
                                .last_synced_generation
                                .get(&tab_sync_key)
                                .copied()
                                .unwrap_or(0);
                            let current_gen = tab_state.content.generation;

                            if current_gen > last_synced_gen {
                                let gen_delta = (current_gen - last_synced_gen) as usize;
                                let needs_full_resync =
                                    gen_delta > tab_state.content.lines.len();
                                let mut lines_added = 0usize;

                                if needs_full_resync {
                                    text_window.clear();
                                }

                                let lines_to_add = if needs_full_resync {
                                    tab_state.content.lines.len()
                                } else {
                                    gen_delta.min(tab_state.content.lines.len())
                                };

                                let skip_count =
                                    tab_state.content.lines.len().saturating_sub(lines_to_add);
                                for line in tab_state.content.lines.iter().skip(skip_count) {
                                    lines_added = lines_added.saturating_add(1);
                                    for segment in &line.segments {
                                        let tw_span_type = match segment.span_type {
                                            crate::data::SpanType::Normal => text_window::SpanType::Normal,
                                            crate::data::SpanType::Link => text_window::SpanType::Link,
                                            crate::data::SpanType::Monsterbold => text_window::SpanType::Monsterbold,
                                            crate::data::SpanType::Spell => text_window::SpanType::Spell,
                                            crate::data::SpanType::Speech => text_window::SpanType::Speech,
                                        };

                                        let styled_text = text_window::StyledText {
                                            content: segment.text.clone(),
                                            fg: segment
                                                .fg
                                                .as_ref()
                                                .and_then(|hex| parse_hex_color(hex).ok()),
                                            bg: segment
                                                .bg
                                                .as_ref()
                                                .and_then(|hex| parse_hex_color(hex).ok()),
                                            bold: segment.bold,
                                            span_type: tw_span_type,
                                            link_data: segment.link_data.as_ref().map(|ld| {
                                                text_window::LinkData {
                                                    exist_id: ld.exist_id.clone(),
                                                    noun: ld.noun.clone(),
                                                    text: ld.text.clone(),
                                                    coord: ld.coord.clone(),
                                                }
                                            }),
                                        };
                                        text_window.add_text(styled_text);
                                    }
                                    text_window.finish_line(window.position.width);
                                }
                                self.widget_manager
                                    .last_synced_generation
                                    .insert(tab_sync_key, current_gen);

                                if i != tabbed_content.active_tab_index && lines_added > 0 {
                                    widget.mark_tab_unread(i, lines_added);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Sync compass widget data from AppCore to compass widgets
    pub(crate) fn sync_compass_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Compass(compass_data) = &window.content {
                // Ensure widget exists
                if !self.widget_manager.compass_widgets.contains_key(name) {
                    let widget = compass::Compass::new(name);
                    self.widget_manager.compass_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.compass_widgets.get_mut(name) {
                    widget.set_directions(compass_data.directions.clone());

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                        widget.set_content_align(window_def.base().content_align.clone());
                        widget.set_title(
                            window_def
                                .base()
                                .title
                                .clone()
                                .unwrap_or_else(|| name.clone()),
                        );

                        // Apply compass-specific colors if configured
                        if let crate::config::WindowDef::Compass { data, .. } = window_def {
                            let active_color = normalize_color(&data.active_color).or_else(|| {
                                color_to_hex_string(&theme.window_border_focused)
                                    .or_else(|| color_to_hex_string(&theme.window_border))
                            });
                            let inactive_color =
                                normalize_color(&data.inactive_color).or_else(|| {
                                    blend_colors_hex(
                                        &theme.window_background,
                                        &theme.text_secondary,
                                        0.25,
                                    )
                                    .or_else(|| color_to_hex_string(&theme.text_secondary))
                                });
                            widget.set_colors(active_color, inactive_color);
                        }
                    }
                }
            }
        }
    }

    /// Sync injury doll widget data from AppCore to injury doll widgets
    pub(crate) fn sync_injury_doll_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::InjuryDoll(injury_data) = &window.content {
                // Ensure widget exists
                if !self.widget_manager.injury_doll_widgets.contains_key(name) {
                    let widget = injury_doll::InjuryDoll::new(name);
                    self.widget_manager.injury_doll_widgets.insert(name.clone(), widget);
                }

                // Update widget
                if let Some(widget) = self.widget_manager.injury_doll_widgets.get_mut(name) {
                    // Update all injuries
                    for (body_part, level) in &injury_data.injuries {
                        widget.set_injury(body_part.clone(), *level);
                    }

                    // Apply configuration
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        widget.set_border_sides(window_def.base().border_sides.clone());
                        widget.set_transparent_background(window_def.base().transparent_background);
                        widget.set_background_color(colors.background.clone());
                        widget.set_title(
                            window_def
                                .base()
                                .title
                                .clone()
                                .unwrap_or_else(|| name.clone()),
                        );

                        // Apply injury doll color configuration if specified
                        if let crate::config::WindowDef::InjuryDoll { data, .. } = window_def {
                            let resolved_default = normalize_color(&data.injury_default_color)
                                .or_else(|| color_to_hex_string(&theme.injury_default_color))
                                .unwrap_or_else(|| "#333333".to_string());
                            // Build colors vec with defaults if not specified
                            let colors = vec![
                                resolved_default,
                                data.injury1_color
                                    .clone()
                                    .unwrap_or_else(|| "#aa5500".to_string()),
                                data.injury2_color
                                    .clone()
                                    .unwrap_or_else(|| "#ff8800".to_string()),
                                data.injury3_color
                                    .clone()
                                    .unwrap_or_else(|| "#ff0000".to_string()),
                                data.scar1_color
                                    .clone()
                                    .unwrap_or_else(|| "#999999".to_string()),
                                data.scar2_color
                                    .clone()
                                    .unwrap_or_else(|| "#777777".to_string()),
                                data.scar3_color
                                    .clone()
                                    .unwrap_or_else(|| "#555555".to_string()),
                            ];
                            widget.set_colors(colors);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn sync_performance_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        use crate::data::WindowContent;

        for (name, window) in &app_core.ui_state.windows {
            if !matches!(window.content, WindowContent::Performance) {
                continue;
            }

            let window_def = app_core.layout.windows.iter().find(|wd| wd.name() == *name);
            let (base, perf_data) = match window_def {
                Some(crate::config::WindowDef::Performance { base, data }) => (Some(base.clone()), Some(data.clone())),
                Some(def) => (Some(def.base().clone()), None),
                None => (None, None),
            };

            let widget = self
                .widget_manager
                .performance_widgets
                .entry(name.clone())
                .or_insert_with(|| {
                    let mut w = performance_stats::PerformanceStatsWidget::new();
                    w.set_title(name.clone());
                    w
                });

            if let Some(base) = base.as_ref() {
                let colors = resolve_window_colors(base, theme);
                let title = base
                    .title
                    .clone()
                    .unwrap_or_else(|| base.name.clone());
                widget.set_title(title);
                widget.set_border_config(
                    base.show_border,
                    Some(base.border_style.clone()),
                    colors.border.clone(),
                );
                widget.set_border_sides(base.border_sides.clone());
                widget.set_background_color(colors.background.clone());
                widget.set_transparent_background(base.transparent_background);
                widget.set_text_color(colors.text.clone());
            }

            if let Some(data) = perf_data.as_ref() {
                widget.apply_flags(data);
            }
        }
    }

    /// Sync hand widget data from AppCore to hand widgets
    pub(crate) fn sync_hand_widgets(
        &mut self,
        app_core: &crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        // Find all Hand windows in the UI state
        for (name, window) in &app_core.ui_state.windows {
            if let crate::data::WindowContent::Hand { item, link } = &window.content {
                // Ensure hand widget exists in cache
                if !self.widget_manager.hand_widgets.contains_key(name) {
                    // Determine hand type based on window name
                    let hand_type = match name.as_str() {
                        "left_hand" => hand::HandType::Left,
                        "right_hand" => hand::HandType::Right,
                        "spell_hand" => hand::HandType::Spell,
                        _ => hand::HandType::Left, // Default fallback
                    };

                    let widget = hand::Hand::new(name, hand_type);
                    self.widget_manager.hand_widgets.insert(name.clone(), widget);
                }

                // Update hand widget content
                if let Some(hand_widget) = self.widget_manager.hand_widgets.get_mut(name) {
                    // Set content (or empty if None)
                    let content = item.clone().unwrap_or_default();
                    hand_widget.set_content(content);

                    // Apply window configuration from layout
                    if let Some(window_def) =
                        app_core.layout.windows.iter().find(|w| w.name() == name)
                    {
                        let colors = resolve_window_colors(window_def.base(), theme);
                        hand_widget.set_border_config(
                            window_def.base().show_border,
                            Some(window_def.base().border_style.clone()),
                            colors.border.clone(),
                        );
                        hand_widget.set_border_sides(window_def.base().border_sides.clone());
                        hand_widget.set_title(
                            window_def
                                .base()
                                .title
                                .clone()
                                .unwrap_or_else(|| name.clone()),
                        );
                        hand_widget.set_text_color(colors.text.clone());
                        hand_widget.set_content_highlight_color(None);
                        if let Some(link_ref) = link {
                            hand_widget.set_link_data(Some(link_ref.clone()));
                            if let Some(preset) = app_core.config.colors.presets.get("links") {
                                if let Some(link_fg) = preset.fg.clone() {
                                    hand_widget.set_content_highlight_color(Some(link_fg));
                                }
                            }
                        } else {
                            hand_widget.set_link_data(None);
                        }
                        hand_widget.set_background_color(colors.background.clone());
                        hand_widget
                            .set_transparent_background(window_def.base().transparent_background);
                    }
                }
            }
        }
    }

    /// Sync room window data from AppCore to room window widgets
    pub(crate) fn sync_room_windows(
        &mut self,
        app_core: &mut crate::core::AppCore,
        theme: &crate::theme::AppTheme,
    ) {
        let new_title = if app_core.room_window_dirty {
            Some(self.build_room_title(
                &app_core.room_subtitle,
                &app_core.lich_room_id,
                &app_core.nav_room_id,
            ))
        } else {
            None
        };

        for window_def in app_core
            .layout
            .windows
            .iter()
            .filter(|w| w.widget_type() == "room")
        {
            let window_name = window_def.name();
            self.ensure_room_window_exists(window_name, window_def);

            if let Some(room_window) = self.widget_manager.room_windows.get_mut(window_name) {
                let colors = resolve_window_colors(window_def.base(), theme);
                room_window.set_border_config(
                    window_def.base().show_border,
                    Some(window_def.base().border_style.clone()),
                    colors.border.clone(),
                );
                room_window.set_border_sides(window_def.base().border_sides.clone());
                room_window.set_background_color(colors.background.clone());
                room_window.set_text_color(colors.text.clone());
                if let crate::config::WindowDef::Room { data, .. } = window_def {
                    room_window.set_component_visible("room desc", data.show_desc);
                    room_window.set_component_visible("room objs", data.show_objs);
                    room_window.set_component_visible("room players", data.show_players);
                    room_window.set_component_visible("room exits", data.show_exits);
                    room_window.set_show_name(data.show_name);
                }

                if let Some(ref title) = new_title {
                    room_window.clear_all_components();

                    for (component_id, lines) in &app_core.room_components {
                        room_window.start_component(component_id.clone());

                        for line_segments in lines {
                            for segment in line_segments {
                                room_window.add_segment(segment.clone());
                            }
                            room_window.finish_line();
                        }

                        room_window.finish_component();
                    }

                    room_window.set_title(title.clone());
                }
            }
        }

        if new_title.is_some() {
            app_core.room_window_dirty = false;
        }
    }

    /// Build room window title from room data
    /// Format: "[subtitle - lich_id] (u<nav_id>)"
    /// Example: "[Emberthorn Refuge, Bowery - 33711] (u2022628)"
    fn build_room_title(
        &self,
        subtitle: &Option<String>,
        lich_id: &Option<String>,
        nav_id: &Option<String>,
    ) -> String {
        // Format: [subtitle - lich_room_id] (u_nav_room_id)
        if let Some(ref subtitle_text) = subtitle {
            if let Some(ref lich) = lich_id {
                if let Some(ref nav) = nav_id {
                    format!("[{} - {}] (u{})", subtitle_text, lich, nav)
                } else {
                    format!("[{} - {}]", subtitle_text, lich)
                }
            } else if let Some(ref nav) = nav_id {
                format!("[{}] (u{})", subtitle_text, nav)
            } else {
                format!("[{}]", subtitle_text)
            }
        } else if let Some(ref lich) = lich_id {
            if let Some(ref nav) = nav_id {
                format!("[{}] (u{})", lich, nav)
            } else {
                format!("[{}]", lich)
            }
        } else if let Some(ref nav) = nav_id {
            format!("(u{})", nav)
        } else {
            String::new() // No title to set
        }
    }

}
