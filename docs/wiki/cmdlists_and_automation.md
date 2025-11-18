# Command Lists & Action Automation

Command menus and automation bridges keep Two-Face efficient during hunts, events, and roleplay. Here’s how to customize them.

## Cmdlist Integration

- **Source File**: `cmdlist1.xml` (copied from `defaults/cmdlist1.xml` on first run).
- **Loader**: `cmdlist::CmdList::load` reads the XML once at startup and stores entries in a hash map keyed by coordinate.
- **Entry Structure**:
  ```xml
  <cli coord="2524,2061" menu="look @" command="look #" menu_cat="5_roleplay" />
  ```
  - `coord`: radial menu position (controls ordering in the browser).
  - `menu`: text shown to the user (`@` placeholder replaced with noun).
  - `command`: actual command sent to the game (supports placeholders).
  - `menu_cat`: category for filtering.

### Placeholders

`CmdList::substitute_command` handles:

- `@` – replaced with the noun (display text) from the parser.
- `#` – replaced with `#<exist_id>` referencing the object in the XML feed.
- `%` – optional secondary item (e.g., `transfer # %`).

Menus use the same substitution rules for display text.

### Editing Cmdlist Entries

Because `cmdlist1.xml` can be large, we recommend:

1. Editing in an external XML-aware editor.
2. Validating with `xmllint` or similar to avoid parser failures.
3. Restarting Two-Face to reload the updated file (hot-reload is on the roadmap).

## Popup Menus

- Widgets can host context menus (window chrome, layout manager, highlights, etc.).
- `PopupMenu` centralizes rendering and selection so you can open a menu with either keyboard shortcuts or mouse clicks.
- Global menus (e.g., `menu:windows`) are built in `main.rs`. You can extend them by editing the vector of `PopupMenuItem`s.

## Menu Keybinds

- Managed automatically in `menu_keybinds.toml`.
- Defines navigation shortcuts for browsers and forms. Customize through the Settings Editor to match your habits.

## Automation Hooks

- **Highlights with Sounds**: Trigger audio cues when regex/substring matches fire.
- **Event Patterns** (`config.event_patterns`): Convert parsed combat text into structured events (`ParsedElement::Event`). Each pattern defines `pattern`, `event_type`, `action` (Set/Clear/Increment), and optional duration extraction.
- **Active Effects Widget**: Uses the event output to show timers and durations.
- **Countdowns**: Roundtime/casttime updates are nominally triggered by `<roundTime>`/`<castTime>` XML, but event patterns can keep them in sync when custom scripts push non-standard text.

## Future Automation Ideas

- **Macro Bar**: With the foundation in `cmdlist.rs` and keybind macros, it’s straightforward to add GUI buttons or TUI shortcuts for frequently used scripts.
- **Script Hooks**: Because commands are funneled through `AppCore`, adding pre-send hooks (e.g., logging, alias expansion) is simply a matter of extending the input router.

Automating tasks responsibly will make hunts smoother without giving up the immersive feel of the official client.
