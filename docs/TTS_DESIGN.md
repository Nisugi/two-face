# Text-to-Speech (TTS) System Design

## High-Level Overview

The TTS system provides accessibility support for screen readers by converting game text to speech. It operates as a fully manual answering machine where users navigate through messages using keyboard controls.

### Key Principles

1. **Manual Control Only** - No automatic playback; users explicitly navigate messages
2. **Priority Queueing** - Important messages (damage, death) can jump the queue
3. **Zero-Cost When Disabled** - No performance impact if TTS is turned off in config
4. **Cross-Platform** - Uses platform-native TTS engines (Windows SAPI, macOS AVSpeechSynthesizer, Linux Speech Dispatcher)
5. **Non-Destructive Navigation** - Moving through the queue doesn't delete messages

### User Experience

**Initial State:**
- Messages queue silently as they arrive
- `current_index` starts at `None` (no position)
- User hears nothing until they press a navigation key

**Navigation:**
- **Ctrl+Alt+Right**: Speak next message (0→1→2... from None starts at 0)
- **Ctrl+Alt+Left**: Speak previous message (2→1→0... from None starts at end)
- **Ctrl+Alt+Up**: Jump to latest unread message (highest index with `!spoken`)
- **Ctrl+Alt+Down**: Stop speaking but stay at current position

**Controls:**
- **F7/F8**: Volume up/down (±0.1)
- **F9/F10**: Rate faster/slower (±0.1)
- **F11**: Mute toggle (stops speech, prevents new messages)

### Configuration

```toml
[tts]
enabled = true          # Master switch
voice = ""              # System default if empty
rate = 1.0              # 0.5 = slow, 1.0 = normal, 2.0 = fast
volume = 0.8            # 0.0 to 1.0
speak_thoughts = true   # Speak "thoughts" stream
speak_whispers = true   # Speak "speech" stream
speak_main = false      # Speak "main" stream (usually too noisy)
```

---

## Low-Level Architecture

### Component Structure

```
┌─────────────────┐
│   AppCore       │
│  (app_core.rs)  │
└────────┬────────┘
         │ owns
         ▼
┌─────────────────┐
│  TtsManager     │      ┌──────────────────┐
│   (tts/mod.rs)  │◄─────│ tts::Tts engine  │
└────────┬────────┘      └──────────────────┘
         │
         │ contains
         ▼
┌─────────────────┐      ┌──────────────────┐
│ VecDeque<Entry> │      │  Event Channel   │
│  (speech queue) │      │  (callbacks)     │
└─────────────────┘      └──────────────────┘
```

### Data Flow

#### 1. Message Arrival (Enqueue)

```
Game Server → Parser → AppCore::append_text_to_window()
                             │
                             ▼
                Check window config (speak_thoughts, speak_whispers, speak_main)
                             │
                             ▼
                   tts_manager.enqueue(SpeechEntry)
                             │
                             ▼
               ┌─────────────┴─────────────┐
               │  Muted or disabled?       │
               │  Yes: return              │
               │  No: continue             │
               └───────────────────────────┘
                             │
                             ▼
               ┌─────────────┴─────────────┐
               │  Queue full (100)?        │
               │  Yes: pop_front()         │
               │       adjust current_index│
               └───────────────────────────┘
                             │
                             ▼
               Find insertion position based on priority
               (Normal=0, High=1, Critical=2)
                             │
                             ▼
               Insert at position, adjust current_index if needed
                             │
                             ▼
                      Log: "Queued at index N"
```

**Key Code ([tts/mod.rs:171-203](c:\Gemstone\Projects\two-face\src\tts\mod.rs#L171-L203)):**
```rust
pub fn enqueue(&mut self, entry: SpeechEntry) {
    // Early return if disabled/muted
    if !self.enabled || self.muted { return; }

    // Queue overflow protection
    if self.queue.len() >= self.max_queue_size {
        self.queue.pop_front();
        // Adjust current_index (shifted by pop)
        if let Some(current) = self.current_index {
            self.current_index = if current > 0 {
                Some(current - 1)
            } else {
                None
            };
        }
    }

    // Priority insertion
    let insert_pos = self.queue
        .iter()
        .position(|e| e.priority < entry.priority)
        .unwrap_or(self.queue.len());

    self.queue.insert(insert_pos, entry);

    // Adjust current_index if insertion before current
    if let Some(current) = self.current_index {
        if insert_pos <= current {
            self.current_index = Some(current + 1);
        }
    }
}
```

#### 2. Manual Navigation (User Presses Ctrl+Alt+Right)

```
Keyboard Event → Frontend → AppCore::handle_event()
                                    │
                                    ▼
                          KeyAction::TtsNext
                                    │
                                    ▼
                     tts_manager.speak_next()
                                    │
                                    ▼
              ┌────────────────────┴────────────────────┐
              │ Muted/disabled? Yes: return             │
              └─────────────────────────────────────────┘
                                    │
                                    ▼
                        ensure_initialized()
                        (lazy-init TTS engine)
                                    │
                                    ▼
              ┌────────────────────┴────────────────────┐
              │ current_index?                          │
              │   None → next = 0 (start)               │
              │   Some(i) → next = i + 1                │
              └─────────────────────────────────────────┘
                                    │
                                    ▼
                        speak_at_index(next, true)
                                    │
                                    ▼
                          engine.stop()  (interrupt=true)
                                    │
                                    ▼
              match engine.speak(&text, false)
                                    │
                ┌───────────────────┴───────────────────┐
                │ Returns Some(UtteranceId)             │
                │   → Store id in current_utterance_id  │
                │   → Mark entry.spoken = true          │
                │   → Set current_index = next          │
                └───────────────────────────────────────┘
```

**Key Code ([tts/mod.rs:215-236](c:\Gemstone\Projects\two-face\src\tts\mod.rs#L215-L236)):**
```rust
pub fn speak_next(&mut self) -> Result<()> {
    if !self.enabled || self.muted { return Ok(()); }
    self.ensure_initialized()?;

    // Sequential navigation (answering machine style)
    let next_index = if let Some(current) = self.current_index {
        if current + 1 < self.queue.len() {
            Some(current + 1)
        } else {
            None  // At end
        }
    } else {
        // No position → start from beginning
        if !self.queue.is_empty() { Some(0) } else { None }
    };

    if let Some(index) = next_index {
        self.speak_at_index(index, true)?;  // interrupt=true
    }
    Ok(())
}
```

#### 3. Speech Callback (Utterance Finishes)

```
TTS Engine → on_utterance_end callback → event_tx.send(UtteranceEnded)
                                                │
                                                ▼
Main Loop: app_core.poll_tts_events()
                                                │
                                                ▼
                      event_rx.try_recv() → Ok(UtteranceEnded(id))
                                                │
                                                ▼
                      ┌───────────────────────┴───────────────────┐
                      │ is_current_utterance(id)?                 │
                      │   Yes: Log "ended (manual control)"       │
                      │   No:  Ignore (stale callback)            │
                      └───────────────────────────────────────────┘
                                                │
                                                ▼
                                    (Do nothing - user controls)
```

**Why no auto-play?** Original design had `auto_play_next()` here, but user feedback showed it was disruptive. Manual control means users decide when to hear the next message.

**Key Code ([app_core.rs:673-679](c:\Gemstone\Projects\two-face\src\core\app_core.rs#L673-L679)):**
```rust
crate::tts::TtsEvent::UtteranceEnded(id) => {
    if self.tts_manager.is_current_utterance(id) {
        tracing::debug!("Utterance {:?} ended (manual control - no auto-play)", id);
        // Auto-play disabled - user has full manual control
    }
}
```

### Critical Implementation Details

#### 1. Rate/Volume Normalization

**Problem:** Different TTS backends use different ranges:
- Windows WinRT: rate 0.5 to 6.0, volume 0.0 to 1.0
- macOS: rate 0.0 to 1.0, volume 0.0 to 1.0
- Linux: backend-specific

**Solution:** Query backend ranges at initialization, normalize config values

```rust
fn normalize_rate(&self, config_rate: f32) -> f32 {
    // Config: 0.5 = slow, 1.0 = normal, 2.0 = fast
    // Backend might expect different range - just clamp
    config_rate.clamp(self.backend_min_rate, self.backend_max_rate)
}

fn normalize_volume(&self, config_volume: f32) -> f32 {
    // Config: 0.0 to 1.0 → map to backend range
    let clamped = config_volume.clamp(0.0, 1.0);
    self.backend_min_volume + clamped * (self.backend_max_volume - self.backend_min_volume)
}
```

**History:** First implementation did complex ratio mapping (0.5→0%, 1.0→33%, 2.0→100%) which caused 1.0 to still be 2x speed. Simplified to direct clamping.

#### 2. Index Preservation During Queue Modifications

**Problem:** When new messages insert into the queue, or old messages drop, indices shift. If `current_index` isn't adjusted, the user ends up pointing at the wrong message.

**Scenarios:**

**Scenario A: High-priority message inserts before current position**
```
Before:           After insertion at position 2:
Index: 0 1 2 3 4 5          0 1 [NEW] 2 3 4 5 6
              ^                        ^
        current_index=3           current_index=4 (adjusted)
```

**Scenario B: Queue overflow drops front**
```
Before:           After pop_front():
Index: 0 1 2 3 4          0 1 2 3
          ^                  ^
    current_index=2      current_index=1 (adjusted)
```

**Code ([tts/mod.rs:173-189, 193-198](c:\Gemstone\Projects\two-face\src\tts\mod.rs)):**
```rust
// Queue overflow
if self.queue.len() >= self.max_queue_size {
    self.queue.pop_front();
    if let Some(current) = self.current_index {
        if current > 0 {
            self.current_index = Some(current - 1);
        } else {
            self.current_index = None;  // Dropped the message we were at
        }
    }
}

// High-priority insertion
if let Some(current) = self.current_index {
    if insert_pos <= current {
        self.current_index = Some(current + 1);
    }
}
```

#### 3. Lazy Initialization

**Why:** TTS engine initialization can fail (no audio device, permissions, etc.). Don't fail at startup - only initialize when first used.

```rust
fn ensure_initialized(&mut self) -> Result<()> {
    if self.enabled && self.engine.is_none() {
        tracing::info!("Initializing TTS engine...");
        let mut tts = Tts::default()?;  // Can fail!

        // Query backend capabilities
        self.backend_min_rate = tts.min_rate();
        self.backend_max_rate = tts.max_rate();
        // ... normalize and apply settings ...

        // Set up callback
        let tx = self.event_tx.clone();
        tts.on_utterance_end(Some(Box::new(move |id| {
            let _ = tx.send(TtsEvent::UtteranceEnded(id));
        })))?;

        self.engine = Some(tts);
    }
    Ok(())
}
```

**When called:** Every speak/control method calls `ensure_initialized()` at the top.

#### 4. Stop vs Pause

**Design choice:** `stop()` does NOT change `current_index`

**Why:** User may want to silence a long message temporarily, then continue navigating from where they were.

```rust
pub fn stop(&mut self) -> Result<()> {
    if let Some(ref mut engine) = self.engine {
        engine.stop()?;
    }
    // Don't touch current_index!
    Ok(())
}
```

**Alternative considered:** Reset `current_index` to `None`. Rejected because it breaks the "answering machine" metaphor - stop should be like pressing pause, not ejecting the tape.

### Data Structures

#### SpeechEntry
```rust
pub struct SpeechEntry {
    pub text: String,           // The text to speak
    pub source_window: String,  // "main", "thoughts", "speech", etc.
    pub priority: Priority,     // Normal, High, or Critical
    pub spoken: bool,           // Has this been read aloud?
}
```

**Why `spoken` flag?** Enables "next unread" (Ctrl+Alt+Up) to skip already-heard messages.

#### Priority Enum
```rust
pub enum Priority {
    Normal = 0,    // Regular game text
    High = 1,      // Thoughts, whispers, important events
    Critical = 2,  // Damage warnings, death, critical alerts
}
```

**Future use:** Currently all messages are `Normal`. Future enhancement could detect damage/death messages and set `Critical` to auto-prioritize them in the queue.

#### TtsEvent
```rust
pub enum TtsEvent {
    UtteranceStarted(UtteranceId),
    UtteranceEnded(UtteranceId),
    UtteranceStopped(UtteranceId),
}
```

**Channel pattern:** `tts::Tts` callbacks run on a different thread. We send events through `std::sync::mpsc` to the main loop, which polls via `try_recv()` (non-blocking).

### Integration Points

#### 1. AppCore Initialization ([app_core.rs:142-147](c:\Gemstone\Projects\two-face\src\core\app_core.rs))
```rust
let tts_manager = TtsManager::new(
    config.tts.enabled,
    config.tts.rate,
    config.tts.volume,
);
```

#### 2. Text Arrival ([app_core.rs - append_text_to_window](c:\Gemstone\Projects\two-face\src\core\app_core.rs))
```rust
// After appending to window buffer:
if self.config.tts.enabled {
    let should_speak = match window_name {
        "thoughts" => self.config.tts.speak_thoughts,
        "speech" => self.config.tts.speak_whispers,
        "main" => self.config.tts.speak_main,
        _ => false,
    };

    if should_speak {
        self.tts_manager.enqueue(SpeechEntry {
            text: rendered_text,
            source_window: window_name.to_string(),
            priority: Priority::Normal,
            spoken: false,
        });
    }
}
```

#### 3. Main Event Loop ([main.rs:1122](c:\Gemstone\Projects\two-face\src\main.rs))
```rust
while app_core.running {
    let events = frontend.poll_events()?;

    // Poll TTS callback events
    app_core.poll_tts_events();

    // Process frontend events...
}
```

### Performance Characteristics

**When Disabled:**
- `tts_manager.enqueue()` returns immediately (no queue operations)
- `poll_tts_events()` does nothing (channel never created)
- Zero allocations, zero syscalls

**When Enabled:**
- Enqueue: O(n) worst-case for priority insertion (typically O(1) for Normal priority)
- Speak: O(1) index lookup
- Poll events: O(k) where k = number of pending callbacks (usually 0 or 1)
- Memory: VecDeque capped at 100 entries × ~200 bytes each ≈ 20KB max

**Lazy Init Cost:**
- First speak: ~100-500ms to initialize TTS engine (platform-dependent)
- Subsequent speaks: <1ms

### Error Handling

**Philosophy:** TTS is a non-critical feature. Errors should log but not crash.

```rust
// In AppCore key handler
KeyAction::TtsNext => {
    if let Err(e) = self.tts_manager.speak_next() {
        tracing::warn!("TTS speak_next failed: {}", e);  // Log but continue
    }
}
```

**Possible errors:**
- `Tts::default()` fails: No audio device, permissions denied
- `speak()` fails: Invalid UTF-8, backend crash
- `set_rate()`/`set_volume()` fails: Out-of-range (prevented by normalization)

### Testing Strategy

**Manual Testing Checklist:**
1. Start with `enabled = false` → verify no performance impact
2. Enable TTS, login → messages queue silently
3. Press Ctrl+Alt+Right → first message speaks from index 0
4. Press Ctrl+Alt+Right again → second message speaks
5. Press Ctrl+Alt+Left → previous message speaks
6. Press Ctrl+Alt+Up → jumps to latest unread
7. While speaking, press Ctrl+Alt+Down → stops but preserves position
8. Press F9 repeatedly → rate increases (hear it speed up)
9. Press F10 repeatedly → rate decreases (hear it slow down)
10. Press F7/F8 → volume changes
11. Press F11 → mutes (stops speech, new messages don't queue)
12. Press F11 again → unmutes

**Edge Cases to Test:**
- Queue overflow (>100 messages) → oldest drops, index adjusts
- High-priority message during review → inserts before current, index adjusts
- Speak at end of queue → "At end of TTS queue" log
- Next unread when all are read → "No more unread" log
- Terminal resize while speaking → speech continues
- Network disconnect while speaking → speech continues (independent)

### Future Enhancements

1. **Auto-priority detection:**
   ```rust
   if text.contains("You are stunned!") || text.contains("You have been injured") {
       priority = Priority::Critical;
   }
   ```

2. **Voice selection:**
   ```rust
   if !config.tts.voice.is_empty() {
       let voices = tts.voices()?;
       if let Some(voice) = voices.iter().find(|v| v.name() == config.tts.voice) {
           tts.set_voice(voice)?;
       }
   }
   ```

3. **Stream-specific voices/rates:**
   ```toml
   [tts.thoughts]
   voice = "Microsoft David"
   rate = 1.2

   [tts.main]
   voice = "Microsoft Zira"
   rate = 0.8
   ```

4. **Replay mode:**
   ```rust
   pub fn replay_current(&mut self) -> Result<()> {
       if let Some(current) = self.current_index {
           self.speak_at_index(current, true)?;
       }
       Ok(())
   }
   ```

5. **Export queue:**
   ```rust
   pub fn export_queue(&self, path: &Path) -> Result<()> {
       let file = File::create(path)?;
       for entry in &self.queue {
           writeln!(file, "[{}] {}", entry.source_window, entry.text)?;
       }
       Ok(())
   }
   ```

---

## Debugging

### Enable TTS Logging

Set `RUST_LOG=debug` to see TTS events:

```bash
RUST_LOG=debug two-face.exe
```

**Expected log output:**
```
TTS engine initialized successfully with callbacks
Queued message at index 0 (total: 1)
Speaking [main]: Welcome to GemStone IV!
Speaking utterance UtteranceId(42) at index 0
Utterance UtteranceId(42) ended (manual control - no auto-play)
```

### Common Issues

**"TTS engine initialized but nothing speaks":**
- Check `config.tts.enabled = true`
- Check `speak_main/speak_thoughts/speak_whispers` for the relevant window
- Check F11 wasn't pressed (muted)

**"Rate changes don't work (F9/F10)":**
- Windows WinRT backend: Rate changes apply immediately
- macOS backend: May need to stop/restart speech
- Linux backend: Backend-dependent

**"Speech is garbled/too fast even at rate=1.0":**
- Check backend ranges in logs: `TTS backend ranges: rate=X to Y`
- If backend max is very high (e.g., 10.0), normalization may be wrong
- Manually set `rate = 0.5` in config to test

**"Crashes on startup with TTS enabled":**
- Likely no audio device or permissions issue
- Check logs for `Tts::default()` error
- Disable TTS and report issue with error message

---

## References

- **tts-rs Documentation:** https://docs.rs/tts/
- **Platform TTS APIs:**
  - Windows SAPI: https://learn.microsoft.com/en-us/previous-versions/windows/desktop/ms723627(v=vs.85)
  - macOS AVSpeechSynthesizer: https://developer.apple.com/documentation/avfoundation/avspeechsynthesizer
  - Linux Speech Dispatcher: https://freebsoft.org/doc/speechd/speech-dispatcher.html

- **Related Code:**
  - [src/tts/mod.rs](c:\Gemstone\Projects\two-face\src\tts\mod.rs) - TtsManager implementation
  - [src/core/app_core.rs](c:\Gemstone\Projects\two-face\src\core\app_core.rs) - Integration with AppCore
  - [src/config.rs](c:\Gemstone\Projects\two-face\src\config.rs) - KeyAction enum + TTS config
  - [defaults/keybinds.toml](c:\Gemstone\Projects\two-face\defaults\keybinds.toml) - Default keybinds
  - [defaults/config.toml](c:\Gemstone\Projects\two-face\defaults\keybinds.toml) - Default config

---

**Document Version:** 1.0
**Last Updated:** 2025-11-17
**Author:** Claude Code (via user collaboration)
