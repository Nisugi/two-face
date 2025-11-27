//! XML message processing
//!
//! Handles parsing and routing of XML messages from the game server.
//! Updates GameState and UiState based on incoming messages.

use crate::config::{Config, SpellColorStyle};
use crate::core::GameState;
use crate::data::*;
use crate::parser::ParsedElement;
use std::collections::HashMap;

/// Processes incoming game messages and updates state
pub struct MessageProcessor {
    /// Configuration (for presets, highlights, etc.)
    config: Config,

    /// Parser for parsing XML content
    parser: crate::parser::XmlParser,

    /// Current text stream (for multi-line messages)
    current_stream: String,

    /// Accumulated styled text for current stream
    current_segments: Vec<TextSegment>,

    /// Track if chunk (since last prompt) has main stream text
    chunk_has_main_text: bool,

    /// Track if chunk (since last prompt) has silent updates
    pub chunk_has_silent_updates: bool,

    /// If true, discard text because no window exists for current stream
    discard_current_stream: bool,

    /// Server time offset for countdown synchronization
    pub server_time_offset: i64,

    /// Buffer for accumulating inventory stream lines (double-buffer system)
    inventory_buffer: Vec<Vec<TextSegment>>,

    /// Previous inventory buffer for comparison (avoid unnecessary updates)
    previous_inventory: Vec<Vec<TextSegment>>,

    /// Buffer for accumulating combat stream lines (for targets widget)
    combat_buffer: Vec<Vec<TextSegment>>,

    /// Buffer for accumulating playerlist stream lines (for players widget)
    playerlist_buffer: Vec<Vec<TextSegment>>,

    /// Previous room component values (for change detection to avoid unnecessary processing)
    previous_room_components: std::collections::HashMap<String, String>,
}

impl MessageProcessor {
    pub fn new(config: Config) -> Self {
        // Create parser with presets from config
        let preset_list = config
            .colors
            .presets
            .iter()
            .map(|(id, preset)| (id.clone(), preset.fg.clone(), preset.bg.clone()))
            .collect();
        let event_patterns = config.event_patterns.clone();
        let parser = crate::parser::XmlParser::with_presets(preset_list, event_patterns);

        Self {
            config,
            parser,
            current_stream: String::from("main"),
            current_segments: Vec::new(),
            chunk_has_main_text: false,
            chunk_has_silent_updates: false,
            discard_current_stream: false,
            server_time_offset: 0,
            inventory_buffer: Vec::new(),
            previous_inventory: Vec::new(),
            combat_buffer: Vec::new(),
            playerlist_buffer: Vec::new(),
            previous_room_components: std::collections::HashMap::new(),
        }
    }

    /// Process a parsed XML element and update states
    pub fn process_element(
        &mut self,
        element: &ParsedElement,
        game_state: &mut GameState,
        ui_state: &mut UiState,
        room_components: &mut std::collections::HashMap<String, Vec<Vec<TextSegment>>>,
        current_room_component: &mut Option<String>,
        room_window_dirty: &mut bool,
        nav_room_id: &mut Option<String>,
        lich_room_id: &mut Option<String>,
        room_subtitle: &mut Option<String>,
        mut tts_manager: Option<&mut crate::tts::TtsManager>,
    ) {
        match element {
            ParsedElement::StreamWindow { id, subtitle } => {
                self.handle_stream_window(
                    id,
                    subtitle.as_deref(),
                    ui_state,
                    room_subtitle,
                    room_window_dirty,
                );
            }
            ParsedElement::Component { id, value } => {
                self.handle_component(
                    id,
                    value,
                    room_components,
                    current_room_component,
                    room_window_dirty,
                );
            }
            ParsedElement::RoomId { id } => {
                *nav_room_id = Some(id.clone());
                *room_window_dirty = true;
                tracing::debug!("Room ID updated: {}", id);
            }
            ParsedElement::StreamPush { id } => {
                self.flush_current_stream_with_tts(ui_state, tts_manager.as_deref_mut());
                self.current_stream = id.clone();

                // Check if this is a stream that should be discarded when window doesn't exist
                // Streams like spells, bounty, inv, room should be discarded
                // But streams like thoughts, speech should fallback to main window
                let should_discard_if_no_window = matches!(id.as_str(), "spell" | "bounty" | "room");

                // Check if a window exists for this stream (map stream to window name first)
                let window_name = self.map_stream_to_window(&id);
                if should_discard_if_no_window && ui_state.get_window(&window_name).is_none() {
                    self.discard_current_stream = true;
                    tracing::debug!("No window exists for stream '{}' (maps to window '{}'), discarding content", id, window_name);
                } else {
                    self.discard_current_stream = false;
                }

                // Clear room components when room stream is pushed (only if window exists)
                if id == "room" && !self.discard_current_stream {
                    room_components.clear();
                    *current_room_component = None;
                    self.previous_room_components.clear(); // Clear change detection cache
                    *room_window_dirty = true;
                    tracing::debug!("Room stream pushed - cleared all room components");
                }

                // Clear inventory buffer when inv stream is pushed
                if id == "inv" {
                    self.inventory_buffer.clear();
                    tracing::debug!("Inventory stream pushed - cleared inventory buffer");
                }

                // Clear combat buffer when combat stream is pushed
                if id == "combat" {
                    self.combat_buffer.clear();
                    tracing::debug!("Combat stream pushed - cleared combat buffer");
                }

                // Clear playerlist buffer when playerlist stream is pushed
                if id == "playerlist" {
                    self.playerlist_buffer.clear();
                    tracing::debug!("Playerlist stream pushed - cleared playerlist buffer");
                }
            }
            ParsedElement::StreamPop => {
                self.flush_current_stream_with_tts(ui_state, tts_manager.as_deref_mut());

                // Flush inventory buffer if we're leaving inv stream
                if self.current_stream == "inv" {
                    self.flush_inventory_buffer(ui_state);
                }

                // Flush combat buffer if we're leaving combat stream
                if self.current_stream == "combat" {
                    self.flush_combat_buffer(ui_state);
                }

                // Flush playerlist buffer if we're leaving playerlist stream
                if self.current_stream == "playerlist" {
                    self.flush_playerlist_buffer(ui_state);
                }

                // Check if stream was routed to a non-main window that actually exists
                // If so, skip the next prompt to avoid duplication in main window
                let stream_window = self.map_stream_to_window(&self.current_stream);

                // Only skip if: (1) maps to non-main AND (2) that window actually exists
                if stream_window != "main" && ui_state.get_window(&stream_window).is_some() {
                    self.chunk_has_silent_updates = true;
                    tracing::debug!(
                        "Stream '{}' routed to existing '{}' window - will skip next prompt",
                        self.current_stream,
                        stream_window
                    );
                } else if stream_window != "main" {
                    tracing::debug!("Stream '{}' would map to '{}' but window doesn't exist - content went to main, won't skip prompt",
                        self.current_stream, stream_window);
                }

                // Reset discard flag when returning to main stream
                self.discard_current_stream = false;
                self.current_stream = String::from("main");
            }
            ParsedElement::Prompt { time, text } => {
                // Finish current stream before prompt
                self.flush_current_stream_with_tts(ui_state, tts_manager.as_deref_mut());

                // Decide whether to show this prompt based on chunk tracking
                // Skip if: chunk had ONLY silent updates (no main text)
                let should_skip = self.chunk_has_silent_updates && !self.chunk_has_main_text;

                if should_skip {
                    tracing::debug!("Skipping prompt '{}' - chunk had only silent updates", text);
                } else if !text.trim().is_empty() {
                    // Store the prompt in game state for command echoes
                    game_state.last_prompt = text.clone();

                    // Reset to main stream
                    self.current_stream = String::from("main");

                    // Render prompt with per-character coloring
                    for ch in text.chars() {
                        let char_str = ch.to_string();

                        // Find color for this character in prompt_colors config
                        let color = self
                            .config
                            .colors
                            .prompt_colors
                            .iter()
                            .find(|pc| pc.character == char_str)
                            .and_then(|pc| {
                                // Prefer fg, fallback to color (legacy)
                                pc.fg.as_ref().or(pc.color.as_ref()).cloned()
                            })
                            .unwrap_or_else(|| "#808080".to_string()); // Default dark gray

                        self.current_segments.push(TextSegment {
                            text: char_str,
                            fg: Some(color),
                            bg: None,
                            bold: false,
                            span_type: SpanType::Normal,
                            link_data: None,
                        });
                    }

                    // Finish prompt line
                    self.flush_current_stream_with_tts(ui_state, tts_manager.as_deref_mut());
                }

                // Extract server time offset for countdown synchronization
                if let Ok(server_time) = time.parse::<i64>() {
                    let local_time = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64;
                    self.server_time_offset = server_time - local_time;
                }

                // Reset chunk tracking for next prompt
                self.chunk_has_main_text = false;
                self.chunk_has_silent_updates = false;

                // Reset discard flag - prompts always return to main stream
                self.discard_current_stream = false;
            }
            ParsedElement::Text {
                content,
                fg_color,
                bg_color,
                bold,
                span_type,
                link_data,
                ..
            } => {
                // Track main stream text for prompt skip logic
                if self.current_stream == "main" && !content.trim().is_empty() {
                    self.chunk_has_main_text = true;
                }

                // Discard text if we're in a discarded stream (e.g., no Spells/inv/room window)
                if self.discard_current_stream {
                    tracing::debug!(
                        "Discarding text from stream '{}': {:?}",
                        self.current_stream,
                        content.chars().take(50).collect::<String>()
                    );
                    return;
                }

                // Try to extract Lich room ID from room name format: [Name - ID]
                // Example: "[Emberthorn Refuge, Bowery - 33711]"
                if self.current_stream == "main" && content.contains('[') && content.contains(" - ")
                {
                    // Try to match pattern: [...  - NUMBER]
                    if let Some(dash_pos) = content.rfind(" - ") {
                        if let Some(bracket_pos) = content[dash_pos..].find(']') {
                            let id_start = dash_pos + 3; // After " - "
                            let id_end = dash_pos + bracket_pos;
                            if id_start < content.len() && id_end <= content.len() {
                                let potential_id = &content[id_start..id_end].trim();

                                // Check if it's all digits (room ID)
                                if !potential_id.is_empty()
                                    && potential_id.chars().all(|c| c.is_ascii_digit())
                                {
                                    *lich_room_id = Some(potential_id.to_string());
                                    *room_window_dirty = true;
                                    tracing::debug!(
                                        "Extracted Lich room ID from room name: {}",
                                        potential_id
                                    );
                                }
                            }
                        }
                    }
                }

                // Map parser SpanType to data layer SpanType
                use crate::data::SpanType as DataSpanType;
                use crate::parser::SpanType as ParserSpanType;
                let data_span_type = match span_type {
                    ParserSpanType::Normal => DataSpanType::Normal,
                    ParserSpanType::Link => DataSpanType::Link,
                    ParserSpanType::Monsterbold => DataSpanType::Monsterbold,
                    ParserSpanType::Spell => DataSpanType::Spell,
                    ParserSpanType::Speech => DataSpanType::Speech,
                };

                self.current_segments.push(TextSegment {
                    text: content.clone(),
                    fg: fg_color.clone(),
                    bg: bg_color.clone(),
                    bold: *bold,
                    span_type: data_span_type,
                    link_data: link_data.clone(),
                });
            }
            ParsedElement::RoundTime { value } => {
                // value is the server timestamp when roundtime ends
                // Convert from server time to local time by subtracting the offset
                let end_time_local = *value as i64 - self.server_time_offset;
                game_state.roundtime_end = Some(end_time_local);

                // Update roundtime widget if it exists
                if let Some(rt_window) = ui_state
                    .get_window_by_type_mut(crate::data::WidgetType::Countdown, Some("roundtime"))
                {
                    if let WindowContent::Countdown(ref mut countdown_data) = rt_window.content {
                        countdown_data.end_time = end_time_local;
                    }
                }
            }
            ParsedElement::CastTime { value } => {
                // value is the server timestamp when casttime ends
                // Convert from server time to local time by subtracting the offset
                let end_time_local = *value as i64 - self.server_time_offset;
                game_state.casttime_end = Some(end_time_local);

                // Update casttime widget if it exists
                if let Some(ct_window) = ui_state
                    .get_window_by_type_mut(crate::data::WidgetType::Countdown, Some("casttime"))
                {
                    if let WindowContent::Countdown(ref mut countdown_data) = ct_window.content {
                        countdown_data.end_time = end_time_local;
                    }
                }
            }
            ParsedElement::LeftHand { item, link } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                game_state.left_hand = if item.is_empty() {
                    None
                } else {
                    Some(item.clone())
                };

                // Update left_hand widget if it exists
                if let Some(left_hand_window) = ui_state
                    .get_window_by_type_mut(crate::data::WidgetType::Hand, Some("left_hand"))
                {
                    if let WindowContent::Hand {
                        item: ref mut window_item,
                        link: ref mut window_link,
                    } = left_hand_window.content
                    {
                        *window_item = game_state.left_hand.clone();
                        *window_link = link.clone();
                    }
                }
            }
            ParsedElement::RightHand { item, link } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                game_state.right_hand = if item.is_empty() {
                    None
                } else {
                    Some(item.clone())
                };

                // Update right_hand widget if it exists
                if let Some(right_hand_window) = ui_state
                    .get_window_by_type_mut(crate::data::WidgetType::Hand, Some("right_hand"))
                {
                    if let WindowContent::Hand {
                        item: ref mut window_item,
                        link: ref mut window_link,
                    } = right_hand_window.content
                    {
                        *window_item = game_state.right_hand.clone();
                        *window_link = link.clone();
                    }
                }
            }
            ParsedElement::SpellHand { spell } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                game_state.spell = if spell.is_empty() {
                    None
                } else {
                    Some(spell.clone())
                };

                // Update spell_hand widget if it exists
                if let Some(spell_hand_window) = ui_state
                    .get_window_by_type_mut(crate::data::WidgetType::Hand, Some("spell_hand"))
                {
                    if let WindowContent::Hand { ref mut item, .. } = spell_hand_window.content {
                        *item = game_state.spell.clone();
                    }
                }

                tracing::debug!("Updated spell hand: {:?}", game_state.spell);
            }
            ParsedElement::Compass { directions } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                game_state.compass_dirs = directions.clone();

                // Update compass widget if it exists (singleton)
                if let Some(compass_window) =
                    ui_state.get_window_by_type_mut(crate::data::WidgetType::Compass, None)
                {
                    if let WindowContent::Compass(ref mut compass_data) = compass_window.content {
                        compass_data.directions = directions.clone();
                    }
                }
            }
            ParsedElement::InjuryImage { id, name } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                // Convert injury name to level: Injury1-3 = 1-3, Scar1-3 = 4-6
                // When name equals body part ID, it means cleared (level 0)
                let level = if name == id {
                    0 // Cleared - name equals body part ID
                } else if name.starts_with("Injury") {
                    match name.chars().last() {
                        Some('1') => 1,
                        Some('2') => 2,
                        Some('3') => 3,
                        _ => 0,
                    }
                } else if name.starts_with("Scar") {
                    match name.chars().last() {
                        Some('1') => 4,
                        Some('2') => 5,
                        Some('3') => 6,
                        _ => 0,
                    }
                } else {
                    0 // Unknown injury type - treat as cleared
                };

                // Update injury doll widget if it exists (singleton)
                if let Some(injury_window) =
                    ui_state.get_window_by_type_mut(crate::data::WidgetType::InjuryDoll, None)
                {
                    if let WindowContent::InjuryDoll(ref mut injury_data) = injury_window.content {
                        injury_data.set_injury(id.clone(), level);
                        tracing::debug!("Updated injury: {} to level {} ({})", id, level, name);
                    }
                }
            }
            ParsedElement::ProgressBar {
                id,
                value,
                max,
                text,
            } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                // Update progress bar widget
                if let Some(window) = ui_state.get_window_mut(id) {
                    if let WindowContent::Progress(ref mut data) = window.content {
                        data.value = *value; // Store actual values, not percentages
                        data.max = *max;
                        data.label = text.clone();
                    }
                }

                // Also update vitals if it's a known vital
                match id.as_str() {
                    "health" => game_state.vitals.health = (*value * 100 / *max) as u8,
                    "mana" => game_state.vitals.mana = (*value * 100 / *max) as u8,
                    "stamina" => game_state.vitals.stamina = (*value * 100 / *max) as u8,
                    "spirit" => game_state.vitals.spirit = (*value * 100 / *max) as u8,
                    _ => {}
                }
            }
            ParsedElement::Spell { text } => {
                self.chunk_has_silent_updates = true; // Mark as silent update
                game_state.spell = Some(text.clone());
            }
            ParsedElement::StatusIndicator { id, active } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                // Update game state (legacy)
                match id.as_str() {
                    "stunned" => game_state.status.stunned = *active,
                    "bleeding" => game_state.status.bleeding = *active,
                    "hidden" => game_state.status.hidden = *active,
                    "invisible" => game_state.status.invisible = *active,
                    "webbed" => game_state.status.webbed = *active,
                    "dead" => game_state.status.dead = *active,
                    _ => {}
                }

                // Update Indicator windows that match this status
                // Try multiple naming conventions: "hidden", "icon_hidden", "indicator_hidden"
                let possible_names = vec![
                    id.clone(),
                    format!("icon_{}", id),
                    format!("indicator_{}", id),
                ];

                for name in possible_names {
                    if let Some(window) = ui_state.get_window_mut(&name) {
                        if let crate::data::WindowContent::Indicator(ref mut indicator_data) =
                            window.content
                        {
                            // Set status to the id when active, empty when inactive
                            indicator_data.status =
                                if *active { id.clone() } else { String::new() };
                            tracing::trace!(
                                "Updated indicator window '{}': active={}",
                                name,
                                active
                            );
                            break; // Found and updated, stop searching
                        }
                    }
                }
            }
            ParsedElement::ActiveEffect {
                category,
                id,
                value,
                text,
                time,
            } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                // Find the window for this category
                let window_name = match category.as_str() {
                    "Buffs" => "buffs",
                    "Debuffs" => "debuffs",
                    "Cooldowns" => "cooldowns",
                    "ActiveSpells" => "active_spells",
                    _ => return, // Unknown category
                };

                // Update the window content if it exists
                if let Some(window) = ui_state.get_window_mut(window_name) {
                    if let crate::data::WindowContent::ActiveEffects(ref mut effects_content) =
                        window.content
                    {
                        let spell_style = id
                            .parse::<u32>()
                            .ok()
                            .and_then(|spell_id| self.config.get_spell_color_style(spell_id));
                        let default_style = SpellColorStyle {
                            bar_color: None,
                            text_color: None,
                        };
                        let style = spell_style.unwrap_or(default_style);

                        // Find existing effect or add new one
                        if let Some(effect) =
                            effects_content.effects.iter_mut().find(|e| e.id == *id)
                        {
                            // Update existing effect
                            effect.text = text.clone();
                            effect.value = *value;
                            effect.time = time.clone();
                            effect.bar_color = style.bar_color.clone();
                            effect.text_color = style.text_color.clone();
                        } else {
                            // Add new effect
                            effects_content.effects.push(crate::data::ActiveEffect {
                                id: id.clone(),
                                text: text.clone(),
                                value: *value,
                                time: time.clone(),
                                bar_color: style.bar_color.clone(),
                                text_color: style.text_color.clone(),
                            });
                        }
                    }
                }
            }
            ParsedElement::ClearActiveEffects { category } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                // Find the window for this category
                let window_name = match category.as_str() {
                    "Buffs" => "buffs",
                    "Debuffs" => "debuffs",
                    "Cooldowns" => "cooldowns",
                    "ActiveSpells" => "active_spells",
                    _ => return, // Unknown category
                };

                // Clear the window content if it exists
                if let Some(window) = ui_state.get_window_mut(window_name) {
                    if let crate::data::WindowContent::ActiveEffects(ref mut effects_content) =
                        window.content
                    {
                        effects_content.effects.clear();
                    }
                }
            }
            ParsedElement::SwitchQuickBar { id } => {
                self.chunk_has_silent_updates = true; // Mark as silent update

                // Switch QuickBar is handled at AppCore level (needs access to layout cache)
                // This is just a placeholder to prevent the catch-all from triggering
            }
            _ => {
                // Other elements handled elsewhere or not yet implemented
            }
        }
    }

    /// Handle stream window (DO NOT auto-create windows!)
    fn handle_stream_window(
        &mut self,
        id: &str,
        subtitle: Option<&str>,
        ui_state: &mut UiState,
        room_subtitle_out: &mut Option<String>,
        room_window_dirty: &mut bool,
    ) {
        // Push the stream (streamWindow acts like pushStream)
        self.current_stream = id.to_string();

        // Check if a window exists for this stream
        // For inv and Spells streams, check by content type (allows any window name)
        // For other streams, check by mapped window name
        let has_target_window = match id {
            "inv" => {
                // Check if ANY window has Inventory content type
                ui_state
                    .windows
                    .values()
                    .any(|w| matches!(w.content, crate::data::WindowContent::Inventory(_)))
            }
            "Spells" => {
                // Check if ANY window has Spells content type
                ui_state
                    .windows
                    .values()
                    .any(|w| matches!(w.content, crate::data::WindowContent::Spells(_)))
            }
            _ => {
                // For other streams, check by mapped window name
                let window_name = self.map_stream_to_window(id);
                ui_state.get_window(&window_name).is_some()
            }
        };

        if !has_target_window {
            self.discard_current_stream = true;
            tracing::debug!("No window exists for stream '{}', discarding content", id);
        } else {
            self.discard_current_stream = false;
        }

        // Update room subtitle if this is the room window AND window exists
        if id == "room" && !self.discard_current_stream {
            if let Some(subtitle_text) = subtitle {
                // Remove leading " - " if present (matches VellumFE behavior)
                let clean_subtitle = subtitle_text.trim_start_matches(" - ");
                *room_subtitle_out = Some(clean_subtitle.to_string());
                *room_window_dirty = true;
                tracing::debug!(
                    "Room subtitle updated: {} (cleaned from: {})",
                    clean_subtitle,
                    subtitle_text
                );
            }
        }
    }

    /// Handle component data for room window
    fn handle_component(
        &mut self,
        id: &str,
        value: &str,
        room_components: &mut std::collections::HashMap<String, Vec<Vec<TextSegment>>>,
        current_room_component: &mut Option<String>,
        room_window_dirty: &mut bool,
    ) {
        // Only handle room-related components
        if !id.starts_with("room ") {
            tracing::trace!("Ignoring non-room component: {}", id);
            return;
        }

        // Skip processing if we're discarding the current stream (no window exists)
        if self.discard_current_stream {
            tracing::debug!("Skipping room component {} - no room window exists", id);
            return;
        }

        // Mark as silent update (room components shouldn't trigger prompts in main window)
        self.chunk_has_silent_updates = true;

        // Check if component value has changed (avoid unnecessary processing)
        if let Some(previous_value) = self.previous_room_components.get(id) {
            if previous_value == value {
                tracing::trace!("Room component {} unchanged - skipping processing", id);
                return;
            }
        }

        tracing::debug!(
            "Processing room component: {} (value length: {})",
            id,
            value.len()
        );

        // Store current value for next comparison
        self.previous_room_components
            .insert(id.to_string(), value.to_string());

        // If we're starting a new component, finish the current one first
        if current_room_component
            .as_ref()
            .map(|c| c != id)
            .unwrap_or(false)
        {
            // Finish current component
            *current_room_component = None;
        }

        // ALWAYS clear the component buffer when receiving new data (game sends full replacement, not append)
        room_components
            .entry(id.to_string())
            .or_insert_with(Vec::new)
            .clear();
        *current_room_component = Some(id.to_string());
        tracing::debug!("Started/replaced room component: {}", id);

        // Parse the component value to extract styled segments
        if !value.trim().is_empty() {
            // Save parser state before parsing component (components are self-contained)
            let saved_color_stack = self.parser.color_stack.clone();
            let saved_preset_stack = self.parser.preset_stack.clone();
            let saved_style_stack = self.parser.style_stack.clone();
            let saved_bold_stack = self.parser.bold_stack.clone();
            let saved_link_depth = self.parser.link_depth;
            let saved_spell_depth = self.parser.spell_depth;
            let saved_link_data = self.parser.current_link_data.clone();

            // Clear stacks for component parsing (start with clean state)
            self.parser.color_stack.clear();
            self.parser.preset_stack.clear();
            self.parser.style_stack.clear();
            self.parser.bold_stack.clear();
            self.parser.link_depth = 0;
            self.parser.spell_depth = 0;
            self.parser.current_link_data = None;

            // Parse the component value as XML to get styled elements
            let parsed_elements = self.parser.parse_line(value);

            // Extract text segments from parsed elements
            let mut current_line_segments = Vec::new();

            for element in parsed_elements {
                match element {
                    crate::parser::ParsedElement::Text {
                        content,
                        fg_color,
                        bg_color,
                        bold,
                        span_type,
                        link_data,
                        ..
                    } => {
                        // Map parser SpanType to data layer SpanType
                        use crate::data::SpanType as DataSpanType;
                        use crate::parser::SpanType as ParserSpanType;
                        let data_span_type = match span_type {
                            ParserSpanType::Normal => DataSpanType::Normal,
                            ParserSpanType::Link => DataSpanType::Link,
                            ParserSpanType::Monsterbold => DataSpanType::Monsterbold,
                            ParserSpanType::Spell => DataSpanType::Spell,
                            ParserSpanType::Speech => DataSpanType::Speech,
                        };

                        // Link data is already the correct type from parser
                        let link = link_data.clone();

                        let segment = TextSegment {
                            text: content.clone(),
                            fg: fg_color.clone(),
                            bg: bg_color.clone(),
                            bold,
                            span_type: data_span_type,
                            link_data: link.clone(),
                        };

                        // Debug logging for room exits to understand link coloring
                        if id == "room exits" {
                            tracing::debug!(
                                "Room exits segment: text='{}', fg={:?}, span_type={:?}, has_link={}",
                                content,
                                fg_color,
                                data_span_type,
                                link.is_some()
                            );
                        }

                        current_line_segments.push(segment);
                    }
                    _ => {
                        // Ignore other parsed elements (we only care about Text)
                    }
                }
            }

            // Add the line if we got any segments
            if !current_line_segments.is_empty() {
                if let Some(buffer) = room_components.get_mut(id) {
                    buffer.push(current_line_segments);
                    *room_window_dirty = true;
                }
            }

            // Restore parser state after parsing component
            self.parser.color_stack = saved_color_stack;
            self.parser.preset_stack = saved_preset_stack;
            self.parser.style_stack = saved_style_stack;
            self.parser.bold_stack = saved_bold_stack;
            self.parser.link_depth = saved_link_depth;
            self.parser.spell_depth = saved_spell_depth;
            self.parser.current_link_data = saved_link_data;
        }
    }

    /// Flush current text to appropriate window
    pub fn flush_current_stream(&mut self, ui_state: &mut UiState) {
        self.flush_current_stream_with_tts(ui_state, None);
    }

    /// Flush current stream with optional TTS enqueuing
    pub fn flush_current_stream_with_tts(
        &mut self,
        ui_state: &mut UiState,
        mut tts_manager: Option<&mut crate::tts::TtsManager>,
    ) {
        if self.current_segments.is_empty() {
            return;
        }

        let mut line = StyledLine {
            segments: std::mem::take(&mut self.current_segments),
        };

        // Filter out Speech-typed segments if no speech window exists
        // This prevents duplicate speech text when the game sends it both in speech preset AND normally
        if !ui_state.windows.contains_key("speech") {
            let original_count = line.segments.len();
            line.segments
                .retain(|seg| seg.span_type != crate::data::SpanType::Speech);
            if line.segments.len() < original_count {
                tracing::trace!(
                    "Filtered out {} Speech segments (no speech window)",
                    original_count - line.segments.len()
                );
            }
        }

        // If all segments were filtered out, nothing to add
        if line.segments.is_empty() {
            return;
        }

        // Determine target window based on stream
        let window_name = self.map_stream_to_window(&self.current_stream);

        // Special handling for room stream - room uses components, not text segments
        // Discard text from room stream (room data flows through components only)
        if self.current_stream == "room" {
            tracing::debug!(
                "Discarding text segment from room stream (room uses components, not text)"
            );
            return;
        }

        // Special handling for inv stream - buffer instead of directly adding to window
        // Inventory updates are sent constantly with same items, so we buffer and compare
        // Inventory stream is always a silent update (shouldn't trigger prompts in main window)
        if self.current_stream == "inv" {
            self.chunk_has_silent_updates = true;
            // Check if ANY window has Inventory content type
            if !ui_state
                .windows
                .values()
                .any(|w| matches!(w.content, WindowContent::Inventory(_)))
            {
                tracing::trace!("Discarding inv stream content - no inventory window exists");
                return;
            }
            // Add line to inventory buffer instead of window
            let num_segments = line.segments.len();
            self.inventory_buffer.push(line.segments);
            tracing::trace!("Buffered inventory line ({} segments)", num_segments);
            return;
        }

        // Special handling for combat stream - buffer for targets widget
        // Combat stream is always a silent update (shouldn't trigger prompts in main window)
        if self.current_stream == "combat" {
            self.chunk_has_silent_updates = true;
            // Check if ANY window has Targets content type
            if !ui_state
                .windows
                .values()
                .any(|w| matches!(w.content, WindowContent::Targets { .. }))
            {
                tracing::trace!("Discarding combat stream content - no targets window exists");
                return;
            }
            // Add line to combat buffer instead of window
            let num_segments = line.segments.len();
            self.combat_buffer.push(line.segments);
            tracing::trace!("Buffered combat line ({} segments)", num_segments);
            return;
        }

        // Special handling for playerlist stream - buffer for players widget
        // Playerlist stream is always a silent update (shouldn't trigger prompts in main window)
        if self.current_stream == "playerlist" {
            self.chunk_has_silent_updates = true;
            // Check if ANY window has Players content type
            if !ui_state
                .windows
                .values()
                .any(|w| matches!(w.content, WindowContent::Players { .. }))
            {
                tracing::trace!("Discarding playerlist stream content - no players window exists");
                return;
            }
            // Add line to playerlist buffer instead of window
            let num_segments = line.segments.len();
            self.playerlist_buffer.push(line.segments);
            tracing::trace!("Buffered playerlist line ({} segments)", num_segments);
            return;
        }

        // Add line to window, fallback to main if target doesn't exist (except for inv/combat/playerlist streams)
        let mut text_added_to_window = None; // Track (window_name, line_text) for TTS

        if let Some(window) = ui_state.get_window_mut(&window_name) {
            match window.content {
                WindowContent::Text(ref mut content) => {
                    content.add_line(line.clone());
                    text_added_to_window = Some(window_name.clone());
                }
                WindowContent::Inventory(ref mut content) => {
                    content.add_line(line.clone());
                    text_added_to_window = Some(window_name.clone());
                }
                WindowContent::Spells(ref mut content) => {
                    content.add_line(line.clone());
                    text_added_to_window = Some(window_name.clone());
                }
                _ => {
                    // Other content types don't support text lines
                    tracing::trace!(
                        "Window '{}' doesn't support text content (type: {:?})",
                        window_name,
                        std::mem::discriminant(&window.content)
                    );
                }
            }
        } else if window_name != "main" {
            // Target window doesn't exist, fallback to main (but NOT for inv stream!)
            tracing::trace!(
                "Window '{}' doesn't exist, routing content to main window",
                window_name
            );
            if let Some(main_window) = ui_state.get_window_mut("main") {
                if let WindowContent::Text(ref mut content) = main_window.content {
                    content.add_line(line.clone());
                    text_added_to_window = Some("main".to_string());
                }
            }
        }

        // Enqueue for TTS if enabled and text was added to a window
        if let (Some(window_name), Some(tts_mgr)) = (text_added_to_window, tts_manager) {
            self.enqueue_tts(tts_mgr, &window_name, &line);
        }
    }

    /// Flush inventory buffer to window (only if content changed)
    pub fn flush_inventory_buffer(&mut self, ui_state: &mut UiState) {
        // If buffer is empty, nothing to do
        if self.inventory_buffer.is_empty() {
            return;
        }

        // Compare to previous inventory
        let inventory_changed = self.inventory_buffer != self.previous_inventory;

        if inventory_changed {
            tracing::debug!(
                "Inventory changed - updating window ({} lines)",
                self.inventory_buffer.len()
            );

            // Find ALL inventory windows and update them (supports multiple inventory windows)
            let mut updated_count = 0;
            for (name, window) in ui_state.windows.iter_mut() {
                if let WindowContent::Inventory(ref mut content) = window.content {
                    // Clear existing content
                    content.lines.clear();

                    // Add all buffered lines
                    for line_segments in &self.inventory_buffer {
                        content.add_line(StyledLine {
                            segments: line_segments.clone(),
                        });
                    }
                    tracing::debug!(
                        "Updated inventory window '{}' with {} lines",
                        name,
                        content.lines.len()
                    );
                    updated_count += 1;
                }
            }

            if updated_count == 0 {
                tracing::warn!("No inventory windows found to update!");
            } else {
                tracing::debug!("Updated {} inventory window(s)", updated_count);
            }

            // Store as new previous inventory
            self.previous_inventory = self.inventory_buffer.clone();
        } else {
            tracing::debug!(
                "Inventory unchanged - skipping update ({} lines)",
                self.inventory_buffer.len()
            );
        }

        // Clear buffer for next update
        self.inventory_buffer.clear();
    }

    /// Flush combat buffer to targets window
    pub fn flush_combat_buffer(&mut self, ui_state: &mut UiState) {
        // If buffer is empty, nothing to do
        if self.combat_buffer.is_empty() {
            return;
        }

        // Concatenate all text segments into a single string
        let mut full_text = String::new();
        for line_segments in &self.combat_buffer {
            for segment in line_segments {
                full_text.push_str(&segment.text);
            }
        }

        tracing::debug!(
            "Flushing combat buffer - {} lines, {} chars total",
            self.combat_buffer.len(),
            full_text.len()
        );

        // Find ALL targets windows and update them (supports multiple targets windows)
        let mut updated_count = 0;
        for (name, window) in ui_state.windows.iter_mut() {
            if let WindowContent::Targets {
                ref mut targets_text,
            } = window.content
            {
                *targets_text = full_text.clone();
                tracing::debug!(
                    "Updated targets window '{}' with {} chars",
                    name,
                    targets_text.len()
                );
                updated_count += 1;
            }
        }

        if updated_count == 0 {
            tracing::debug!("No targets windows found to update");
        } else {
            tracing::debug!("Updated {} targets window(s)", updated_count);
        }

        // Clear buffer for next update
        self.combat_buffer.clear();
    }

    /// Flush playerlist buffer to players window
    pub fn flush_playerlist_buffer(&mut self, ui_state: &mut UiState) {
        // If buffer is empty, nothing to do
        if self.playerlist_buffer.is_empty() {
            return;
        }

        // Concatenate all text segments into a single string
        let mut full_text = String::new();
        for line_segments in &self.playerlist_buffer {
            for segment in line_segments {
                full_text.push_str(&segment.text);
            }
        }

        tracing::debug!(
            "Flushing playerlist buffer - {} lines, {} chars total",
            self.playerlist_buffer.len(),
            full_text.len()
        );

        // Find ALL players windows and update them (supports multiple players windows)
        let mut updated_count = 0;
        for (name, window) in ui_state.windows.iter_mut() {
            if let WindowContent::Players {
                ref mut players_text,
            } = window.content
            {
                *players_text = full_text.clone();
                tracing::debug!(
                    "Updated players window '{}' with {} chars",
                    name,
                    players_text.len()
                );
                updated_count += 1;
            }
        }

        if updated_count == 0 {
            tracing::debug!("No players windows found to update");
        } else {
            tracing::debug!("Updated {} players window(s)", updated_count);
        }

        // Clear buffer for next update
        self.playerlist_buffer.clear();
    }

    /// Enqueue text for TTS if enabled and configured for this window
    fn enqueue_tts(&self, tts_manager: &mut crate::tts::TtsManager, window_name: &str, line: &StyledLine) {
        // Early exit if TTS not enabled
        if !self.config.tts.enabled {
            return;
        }

        // Check if this window should be spoken based on config
        let should_speak = match window_name {
            "thoughts" => self.config.tts.speak_thoughts,
            "speech" => self.config.tts.speak_whispers, // Whispers go to speech window
            "main" => self.config.tts.speak_main,
            _ => false, // Don't speak other windows by default
        };

        if !should_speak {
            return;
        }

        // Extract clean text from line segments
        let text: String = line.segments.iter().map(|seg| seg.text.as_str()).collect();

        // Skip empty text
        if text.trim().is_empty() {
            return;
        }

        // Skip prompts (single character lines like ">")
        if text.trim().len() <= 1 {
            tracing::trace!("Skipping TTS for single-character prompt: {:?}", text.trim());
            return;
        }

        // Determine priority based on window
        let priority = match window_name {
            "thoughts" => crate::tts::Priority::High, // Thoughts are important
            "speech" => crate::tts::Priority::High,   // Whispers are important
            "main" => crate::tts::Priority::Normal,   // Regular game text
            _ => crate::tts::Priority::Normal,
        };

        // Enqueue speech entry
        tts_manager.enqueue(crate::tts::SpeechEntry {
            text,
            source_window: window_name.to_string(),
            priority,
            spoken: false,
        });

        // Auto-speak the next item in queue (if not currently speaking)
        // This ensures new text gets spoken immediately
        if let Err(e) = tts_manager.speak_next() {
            tracing::warn!("Failed to speak TTS entry: {}", e);
        }
    }

    /// Map stream ID to window name
    fn map_stream_to_window(&self, stream: &str) -> String {
        match stream {
            "main" => "main",
            "room" => "room",
            "inv" => "inventory",
            "thoughts" => "thoughts",
            "speech" => "speech",
            "announcements" => "announcements",
            "loot" => "loot",
            "death" => "death",
            "logons" => "logons",
            "familiar" => "familiar",
            "ambients" => "ambients",
            "bounty" => "bounty",
            "Spells" => "spells",
            "combat" => "targets",
            "playerlist" => "players",
            _ => "main", // Default to main window
        }
        .to_string()
    }

    /// Clear inventory cache to force next inventory update to render
    /// Should be called when a new inventory window is added
    pub fn clear_inventory_cache(&mut self) {
        self.previous_inventory.clear();
        tracing::debug!("Cleared inventory cache - next inventory update will render");
    }
}
