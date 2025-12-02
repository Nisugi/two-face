If we want VellumFE to be a first-class frontend for visually impaired players, we need native text‑to‑speech hooks across all three desktop targets we already support (Windows, macOS, Linux) and surface them in the Rust core so any window or parser event can trigger speech.

Here’s what that entails:

1. Pick a cross-platform abstraction.

Expose a tts::SpeakRequest struct (text, voice, rate, volume, interrupt flag) and a tts::Speaker trait in the Rust core.
Frontend layer (TUI/GUI) registers the appropriate backend at startup based on OS and config.
2. Platform backends.

Windows: Use SAPI (either via windows crate or sapi-lite). Spin up a background thread with CoInitializeEx, create ISpVoice, and handle Speak calls there. Allow optional voice selection (list voices via SpVoice::GetVoices).
macOS: Call the built-in say command or use the AVSpeechSynthesizer API via Objective-C FFI (the objc crate) so we can avoid external processes.
Linux: Depending on distro, pick Speech Dispatcher (DBus API) or espeak-ng. The speech-dispatcher crate wraps the DBus interface; otherwise spawn espeak/festival as a subprocess.
All backends live behind the tts::Speaker trait, so the rest of the code just sends events.

3. Config & Hotkeys.

Add a speech section to config (enabled flag, default voice/rate, queueing behavior).
Provide commands/bindings (e.g., .speakwindow main) so users can manually read a window aloud, and automatic triggers (focused thought, incoming whispers, highlight events).
4. Event integration.

Parser marks lines with roles (thought, whisper, combat, warnings). Each role can have a speech profile (speak only when window unfocused, include prefix, etc.).
The frontend’s watcher sends these events to the tts::Speaker, which handles queueing (interrupt vs. overlap) so speech doesn’t become unreadable.
5. Testing & fallback.

Make speech optional: if backend init fails, log it and keep running silently.
Allow logging the exact command text to help troubleshoot (like we just did in Ruby).
Provide a “TTY mode” for remote setups: if the frontend runs on machine 2 but Lich on machine 1, expose a small RPC so speech runs locally (e.g., WebSocket or simple TCP server that streams text events to a companion app).
Implementation-wise, the first milestone is to add the trait + Windows backend (since SAPI is straightforward). Then add macOS (Objective‑C bridge) and Linux (Speech Dispatcher or espeak). Once the abstraction is in place, anything in the app can call tts::speak("Focused thought…"), and we can map user settings to voice/rate/filters.

Let me know which platform you want to start with (likely Windows since it’s easiest), and we can scaffold the Rust module accordingly.

Planning Speech Support

Screen-Reader Model: Screen readers announce whatever becomes “focused.” With multiple panes, users need to choose which window (main text, thoughts, combat, command input) feeds speech. We should treat each window as a channel that can be: auto-read on updates, read-on-demand (hotkey), or muted. Provide per-window settings (enabled, prepend window name, delay, interrupt behavior).
Granularity: Offer two levels: a) full window flow (every new line) and b) targeted triggers (regex matches, alerts, critical events). Users can enable both, but we must dedupe — e.g., mark a line as “already spoken” when a trigger fires so the full-window reader skips it, or provide an ordering rule (triggers first, then window backlog).
Queue & Priority: Maintain a speech queue where each entry has priority (critical alert, focused thought, general chatter). Let users set whether lower-priority speech can be interrupted. Provide a “silence all” toggle and a “repeat last” command.
Configuration UX: In addition to config files, add in-app menus to toggle speech per window, adjust voice/rate, and create custom triggers. Consider profiles (combat vs. exploration).
Context Awareness: Allow automatically pausing speech when user is typing or when the window is focused (if they’re reading visually already). Conversely, if a window is hidden/offscreen, speech should be the only way to know what’s happening there.
Multi-Screen Layouts: Instead of tying speech to physical screens, tie it to window definitions. If a user clones a window (main text in two places), they still configure “Main” once. Provide a way to group windows (e.g., “All combat panes share settings”).
Event Metadata: When the parser emits a line, include tags: source_window, category (thought, whisper, spell, combat). Speech rules operate on these tags rather than raw text so we can scale.
User Control: Add shortcuts: “Read focused window,” “Read next unread line,” “Mute/unmute window,” “Change voice.” Provide an indicator when speech is muted so users don’t think it broke.
Accessibility Beyond Speech: Consider high-contrast themes, keyboard-only navigation, and optional large-text modes; visually impaired users often need more than TTS.
Testing Strategy: Work with screen-reader users to validate heuristics. Possibly log speech events (timestamp, source) so testers can review what triggered.
Extensibility: Keep the speech engine modular so we can plug in Braille output or haptic alerts later.
With this plan, the next steps are designing the configuration schema and event tags, then building the speech queue/prioritizer before wiring in OS-specific speech backends. Once that foundation exists, we can add UI controls and ensure lines don’t double-trigger.






Use the buffers to your advantage: treat each window’s history as the source of truth so speech can “rewind” without special casing. Beyond that:

Keep metadata with every line (timestamp, tags, whether it was spoken). That makes features like “repeat last thought,” “skip combat chatter,” or “read everything since 5s ago” straightforward.
Design for interruptibility. A visually impaired user might need to silence speech instantly (incoming damage) or prioritize a new alert over a long narration. Build the queue with priorities and an easy “panic mute.”
Plan configuration around workflows, not just settings. e.g., presets for “solo hunting,” “group RP,” “monitoring alts” so users aren’t overwhelmed toggling dozens of checkboxes.
Test with real users early. Simulating a screen-reader workflow is useful, but nothing beats feedback from someone who relies on it daily.
With the buffers and rich metadata you already maintain, you’re well-positioned to provide a polished experience—just keep the event model clean, queue speech intelligently, and give users tight control.

Yeah I kind of view it like an answering machine. Something starts playing hit next to skip to the next message, or previous for the previous message, or just stop, ect.


Exactly—an answering-machine metaphor maps perfectly: queue each “message” (line/event), offer Next/Previous to step through the buffer, and provide Pause/Stop to control playback instantly. Build the speech queue around that mental model and it’ll feel intuitive for users juggling multiple windows.

how does pausing work? Can we actually pause it and pick back up where it left off?


Depends on the backend:

Windows SAPI (SpVoice): supports Pause/Resume methods. If we keep the COM voice object alive, we can pause mid-utterance and then resume exactly where it left off. If we stop and restart, we’ll need to track the queue ourselves and restart from the beginning of that entry.

macOS AVSpeechSynthesizer: has built-in pauseSpeaking(at:) and continueSpeaking. We can pause immediately and resume mid‑sentence. If we rely on the say CLI, there’s no pause—only stop and replay.

Linux: Speech Dispatcher exposes pause/resume over DBus for continuous speech. If we use espeak as a subprocess, pause isn’t offered; we’d need to stop and replay.

So pausing is realistic as long as we pick backends with native pause/resume (SAPI, AVSpeechSynthesizer, Speech Dispatcher). For backends that don’t, we simulate it by stopping and replaying from the buffered text, which isn’t a true mid-utterance resume but still gives users a “stop and replay” control.

Ok, and if they are async does that mean we can pause one, play a different one, then resume the first one?


On most TTS APIs, “async” just means the call returns immediately while the engine keeps speaking—but a single SpVoice (or AVSpeechSynthesizer, Speech Dispatcher connection, etc.) still runs one utterance at a time. You can pause the current utterance, queue another (or switch voices), then resume the original by managing multiple voice instances. In practice:

If you keep a single voice object, pausing mid‑message and speaking something else will usually interrupt the first message (unless you carefully queue another instance).
To truly pause message A, play message B, and resume A from the same spot, you need either:
Multiple voice objects (one per “channel”), each with its own queue/pause state; or
A queue manager that pauses voice 1, records its state (if the API supports Resume), switches to voice 2 for the alert, then calls Resume on voice 1.
SAPI and AVSpeechSynthesizer do let you Pause/Resume that way if you keep the voice alive, so you can build a priority system: pause the low-priority channel, play the urgent alert from another channel, then resume the first where it left off. The key is managing separate queues (or separate voice instances) so you don’t lose your place when the urgent event interrupts.

Ok, so a queue manager would allow us to pause, say something different, resume. That means the api tells us where they are in the line to resume from the line or you mean it would start tat line over upon resuming?


Most native TTS APIs (SAPI, AVSpeechSynthesizer, Speech Dispatcher) let you pause and resume the same utterance without restarting it—they keep the playback position internally. When you call Pause, the engine stops mid-word; Resume picks up right there. To speak something else in between, you either:

Use multiple voice instances: Voice A pauses, voice B speaks the urgent alert, voice A resumes. Each instance keeps its own playback position.
Use one voice but queue priority: Pause voice A, speak alert (same voice), then replay A from the start because the original utterance was interrupted. Not ideal.
So when I mention a queue manager, I mean a layer that tracks which voice/channel is playing what and coordinates pausing/resuming per instance. As long as the underlying API supports Pause/Resume and you keep that instance alive, resuming continues mid-line. If you tear down the voice or use a backend without pause (e.g., the say CLI), you’d have to restart the entire message.

Ok, we would definitely need some kind of queue manager and priority I think. But I get it, mid sentence resume when interuppted was a pipe dream and probably not that valuable.


Queueing + priorities are definitely worth it; whether we resume mid-sentence or restart the line is secondary. Most built-in engines can resume mid-utterance if we keep the same voice instance alive, but the real UX win is simply letting high-priority alerts interrupt lower-priority narration, skip/back up through the queue, and mute when needed. As long as we model speech events like an answering machine (ordered list with controls) and tag them with priority, visually impaired users can manage the flow regardless of whether the resumed message restarts the sentence or not.

Last question. Performance. You know I'm big on performance and I realize this whole speech system should be configured with an on off switch to remove it's toll on performance for most users. But for those who need screen reader support. What is the performance toll with this implementation?


When speech is disabled, we keep it completely out of the hot path—no queue work, no TTS initialization—so the hit is near zero. When it’s enabled, the overhead comes from three places:

Event tagging and queueing: tagging each line with metadata and pushing it onto a speech queue adds minor allocations (a few dozen bytes per spoken line). Even on heavy scrolls, it’s tiny compared to the line buffers we already keep.
Backend calls: invoking SAPI/AVSpeech/Speech Dispatcher happens on a background thread; the main loop just hands text to a channel. As long as we reuse the voice instance and queue, there’s no measurable effect on parser throughput. If we reinitialize the COM object per line we’d pay a big cost, so we’ll keep it warm.
Priority logic: managing pause/resume flags per channel is cheap bookkeeping. It’s roughly the same complexity as our existing sound-trigger queue.
Net: even for blind users running speech on multiple windows, the runtime cost is small compared to rendering and logging, and for everyone else the off switch removes it entirely. The key is not to do heavy work (voice enumeration, COM init) in the critical path; do it once at startup and keep the queue bounded so we don’t pile up hundreds of messages.

That's what I like to hear


Same here—clean opt-in design means we don’t slow anyone down, and the folks who need it still get a robust feature. Once you’re ready to code the Rust side, we’ll keep the speech stack modular so it stays fast for the 90 % who leave it off.