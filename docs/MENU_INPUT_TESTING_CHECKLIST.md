# Menu Input System Testing Checklist

Complete testing checklist for the unified menu input system implemented across all browser, form, and editor widgets.

---

## Pre-Testing Setup

- [ ] Build the project: `cargo build --release`
- [ ] Run the application: `cargo run -- --character test`
- [ ] Verify application starts without validation errors
- [ ] Check logs for any auto-fix messages about menu keybinds

---

## 1. Browser Widget Testing

### 1.1 Highlight Browser (`.highlights` or `.hl`)

**Navigation:**
- [ ] Press `.highlights` to open browser
- [ ] `Up` arrow moves selection up
- [ ] `Down` arrow moves selection down
- [ ] `PageUp` scrolls up by ~10 items
- [ ] `PageDown` scrolls down by ~10 items
- [ ] Selection wraps correctly at boundaries

**Actions:**
- [ ] `Esc` closes browser and returns to Normal mode
- [ ] `Delete` key removes selected highlight (check config)
- [ ] `Enter` (Edit action) logs edit intent (form not yet wired)
- [ ] Deleted highlight disappears from browser immediately

**Rendering:**
- [ ] Browser displays properly over game output
- [ ] Selected item is highlighted
- [ ] Scroll position updates correctly
- [ ] Browser title shows "Highlights"

### 1.2 Keybind Browser (`.keybinds` or `.kb`)

**Navigation:**
- [ ] Press `.keybinds` to open browser
- [ ] `Up`/`Down` navigation works
- [ ] `PageUp`/`PageDown` scrolling works
- [ ] All keybinds display with correct format

**Actions:**
- [ ] `Esc` closes browser
- [ ] `Delete` removes selected keybind
- [ ] Deleted keybind no longer appears in browser
- [ ] Game keybind is actually removed (test by trying to use it)

### 1.3 Color Palette Browser (`.colors`)

**Navigation:**
- [ ] Press `.colors` to open browser
- [ ] Navigation keys work correctly
- [ ] Colors display with preview swatches
- [ ] Category filtering works (if implemented)

**Actions:**
- [ ] `Esc` closes browser
- [ ] `Delete` removes selected color from palette
- [ ] Favorite colors marked correctly

### 1.4 Spell Colors Browser (`.spellcolors`)

**Navigation:**
- [ ] Press `.spellcolors` to open browser
- [ ] Navigate through spell color entries
- [ ] Spell IDs display correctly
- [ ] Color preview shows bar/text/bg colors

**Actions:**
- [ ] `Esc` closes browser
- [ ] No delete action (verify Delete key does nothing)

### 1.5 UI Colors Browser (`.uicolors`)

**Navigation:**
- [ ] Press `.uicolors` to open browser
- [ ] Navigate through UI color categories
- [ ] All UI color types visible

**Actions:**
- [ ] `Esc` closes browser
- [ ] No delete action (UI colors are system colors)

---

## 2. Form Widget Testing

### 2.1 Highlight Form (`.addhighlight` or `.addhl`)

**Field Navigation:**
- [ ] Press `.addhighlight` to open form
- [ ] `Tab` moves to next field
- [ ] `Shift+Tab` moves to previous field
- [ ] Field order: Name → Pattern → Category → FG → BG → Sound → Volume → Bold → ColorLine → FastParse

**Text Editing (Fields 0-6):**
- [ ] Type text in Name field
- [ ] `Ctrl+A` selects all text in current field
- [ ] `Ctrl+C` copies selected text to clipboard
- [ ] `Ctrl+X` cuts selected text to clipboard
- [ ] `Ctrl+V` pastes text from clipboard
- [ ] Text appears correctly while typing

**Toggle Fields (Fields 7-9):**
- [ ] `Tab` to Bold checkbox
- [ ] `Space` or `Enter` toggles Bold on/off
- [ ] Visual indicator shows checkbox state
- [ ] Same for ColorEntireLine checkbox
- [ ] Same for FastParse checkbox

**Dropdown (Field 5 - Sound):**
- [ ] `Tab` to Sound field
- [ ] `Up` arrow cycles through sound files backward
- [ ] `Down` arrow cycles through sound files forward
- [ ] Selected sound file displays

**Save/Cancel:**
- [ ] `Ctrl+S` saves highlight
- [ ] Highlight appears in config
- [ ] Form closes after save
- [ ] `Esc` cancels without saving
- [ ] Form closes without saving changes

**Delete (Edit Mode):**
- [ ] Open form in edit mode (via browser Edit)
- [ ] `Ctrl+D` deletes the highlight
- [ ] Highlight removed from config
- [ ] Form closes

### 2.2 Keybind Form (`.addkeybind`)

**Field Navigation:**
- [ ] `Tab`/`Shift+Tab` cycles through fields
- [ ] Field order: Action Type (Action) → Action Type (Macro) → Key Combo → Action/Macro Value

**Action Type Toggle:**
- [ ] Focus on "Action" radio button (field 0)
- [ ] `Space` selects Action type
- [ ] Focus on "Macro" radio button (field 1)
- [ ] `Space` selects Macro type
- [ ] Only one type selected at a time

**Text Input:**
- [ ] `Tab` to Key Combo field
- [ ] Type key combination (e.g., "ctrl+e")
- [ ] `Ctrl+A/C/X/V` clipboard operations work
- [ ] Placeholder text shows when empty

**Action Dropdown:**
- [ ] Select Action type
- [ ] `Tab` to action dropdown
- [ ] `Up`/`Down` cycles through available actions
- [ ] Action list shows all built-in actions

**Macro Text:**
- [ ] Select Macro type
- [ ] `Tab` to macro text field
- [ ] Type macro text (e.g., "run left\r")
- [ ] `Ctrl+A/C/X/V` work in macro field

**Save/Delete:**
- [ ] `Ctrl+S` saves keybind
- [ ] Keybind appears in config and works
- [ ] `Ctrl+D` deletes in edit mode
- [ ] `Esc` cancels

### 2.3 Color Form (`.addcolor`)

**Field Navigation:**
- [ ] `Tab`/`Shift+Tab` works
- [ ] Field order: Name → Color → Category → Favorite

**Text Input:**
- [ ] Type color name
- [ ] Type hex color (e.g., "#ff0000")
- [ ] Color preview swatch updates
- [ ] `Ctrl+A/C/X/V` work in all text fields

**Favorite Toggle:**
- [ ] `Tab` to Favorite checkbox
- [ ] `Space`/`Enter` toggles favorite status
- [ ] Visual checkbox updates

**Save/Cancel:**
- [ ] `Ctrl+S` saves color to palette
- [ ] Color appears in color browser
- [ ] `Esc` cancels without saving

### 2.4 Spell Color Form (`.addspellcolor`)

**Field Navigation:**
- [ ] `Tab`/`Shift+Tab` cycles fields
- [ ] Field order: Spell IDs → Bar Color → Text Color → BG Color

**Text Input:**
- [ ] Type spell IDs (e.g., "905, 509, 1720")
- [ ] Type bar color (hex or palette name)
- [ ] Type text color
- [ ] Type background color
- [ ] `Ctrl+A/C/X/V` work in all fields

**Save/Delete:**
- [ ] `Ctrl+S` saves spell color range
- [ ] Spell colors applied to appropriate spells
- [ ] `Ctrl+D` deletes in edit mode
- [ ] `Esc` cancels

---

## 3. Settings Editor Testing (`.settings`)

**Navigation:**
- [ ] Press `.settings` to open editor
- [ ] `Up`/`Down` navigate through settings
- [ ] `PageUp`/`PageDown` scroll by page
- [ ] Settings grouped by category
- [ ] Category headers display correctly

**Boolean Settings:**
- [ ] Navigate to boolean setting
- [ ] `Space` or `Enter` toggles value
- [ ] Value changes from true ↔ false
- [ ] Visual indicator updates

**Enum Settings:**
- [ ] Navigate to enum setting
- [ ] `Left` arrow cycles backward through options
- [ ] `Right` arrow cycles forward through options
- [ ] `Space` cycles forward
- [ ] All enum values accessible

**Text/Number Settings:**
- [ ] Navigate to text or number setting
- [ ] `Enter` starts editing
- [ ] Type new value
- [ ] `Enter` saves value
- [ ] `Esc` cancels edit without saving
- [ ] Cursor shows during edit

**Category Filter:**
- [ ] If category filter active, only category settings shown
- [ ] Filter can be changed
- [ ] "All" shows all settings

**Close:**
- [ ] `Esc` closes settings editor
- [ ] Changes are saved
- [ ] Returns to Normal mode

---

## 4. Keybind Customization Testing

### 4.1 Menu Keybind Configuration

**Modify keybinds in config:**
- [ ] Edit `~/.two-face/menu_keybinds.toml` (or similar)
- [ ] Change `navigate_up` from "Up" to "k"
- [ ] Change `navigate_down` from "Down" to "j"
- [ ] Save config and restart application
- [ ] Open any browser
- [ ] `k` moves selection up
- [ ] `j` moves selection down
- [ ] Old bindings (Up/Down) no longer work

### 4.2 Clipboard Keybinds

**Modify clipboard operations:**
- [ ] Change `select_all` from "Ctrl+A" to "Ctrl+Shift+A"
- [ ] Change `copy` from "Ctrl+C" to "Alt+C"
- [ ] Restart application
- [ ] Open any form
- [ ] `Ctrl+Shift+A` selects all
- [ ] `Alt+C` copies text
- [ ] Old bindings don't work

### 4.3 Field Navigation

**Modify tab behavior:**
- [ ] Change `next_field` from "Tab" to "Ctrl+N"
- [ ] Change `previous_field` from "Shift+Tab" to "Ctrl+P"
- [ ] Restart application
- [ ] Open any form
- [ ] `Ctrl+N` moves to next field
- [ ] `Ctrl+P` moves to previous field

---

## 5. Validator Testing

### 5.1 Missing Critical Bindings

**Test auto-fix:**
- [ ] Edit config, set `cancel = ""`
- [ ] Edit config, set `navigate_up = ""`
- [ ] Restart application
- [ ] Check logs for validation errors
- [ ] Check logs for "Auto-fixed 2 menu keybind issues"
- [ ] Open browser
- [ ] `Esc` still works (cancel restored to "Esc")
- [ ] `Up` still works (navigate_up restored to "Up")

### 5.2 Duplicate Bindings

**Test duplicate detection:**
- [ ] Edit config, set `navigate_up = "j"`
- [ ] Edit config, set `navigate_down = "j"`
- [ ] Restart application
- [ ] Check logs for duplicate binding warning
- [ ] Warning shows: "Keybind 'j' is assigned to multiple actions: navigate_up, navigate_down"
- [ ] Application still works (last binding wins)

### 5.3 Empty Non-Critical Bindings

**Test graceful degradation:**
- [ ] Edit config, set `move_up = ""`
- [ ] Edit config, set `move_down = ""`
- [ ] Restart application
- [ ] No errors (these aren't critical)
- [ ] Reorder actions not available but app works

---

## 6. Edge Cases & Error Handling

### 6.1 Escape Key Handling

**Test Esc priority:**
- [ ] Open browser
- [ ] Customize `cancel` to something else
- [ ] `Esc` still closes browser (hardcoded in event handler)
- [ ] Custom cancel binding also works

### 6.2 Modal Layering

**Test multiple widgets:**
- [ ] Open browser
- [ ] Try to open another widget (should not allow or handle correctly)
- [ ] Esc closes top widget
- [ ] Background still updates

### 6.3 Clipboard Errors

**Test clipboard failure:**
- [ ] Open form
- [ ] `Ctrl+C` with no selection (should not crash)
- [ ] `Ctrl+V` with invalid clipboard data (should handle gracefully)
- [ ] Check logs for clipboard errors

### 6.4 Long Lists

**Test scrolling:**
- [ ] Open browser with 100+ items
- [ ] Navigate to bottom
- [ ] `PageDown` at bottom doesn't crash
- [ ] Navigate to top
- [ ] `PageUp` at top doesn't crash
- [ ] Scroll position correct throughout

### 6.5 Empty Browsers

**Test empty state:**
- [ ] Delete all highlights
- [ ] Open highlight browser
- [ ] No crash with empty list
- [ ] Message or empty state shown
- [ ] Navigation keys handled gracefully

### 6.6 Form Validation

**Test invalid input:**
- [ ] Open highlight form
- [ ] Leave name field empty
- [ ] Try to save with `Ctrl+S`
- [ ] Validation error shown or save prevented
- [ ] Form remains open for correction

---

## 7. Integration Testing

### 7.1 Game Keybinds vs Menu Keybinds

**Test dual keybind system:**
- [ ] Set game keybind: `Ctrl+S` = "shield bash"
- [ ] Set menu keybind: `save = "Ctrl+S"`
- [ ] In Normal mode, `Ctrl+S` performs shield bash
- [ ] Open form
- [ ] In form, `Ctrl+S` saves form (not shield bash)
- [ ] Close form
- [ ] Back in Normal mode, `Ctrl+S` performs shield bash again

### 7.2 Mode Transitions

**Test InputMode transitions:**
- [ ] Start in Normal mode
- [ ] Open browser → InputMode changes
- [ ] Normal mode keybinds don't work
- [ ] Menu keybinds work
- [ ] Close browser → Back to Normal mode
- [ ] Normal mode keybinds work again

### 7.3 Config Persistence

**Test save/load:**
- [ ] Create highlight via form
- [ ] Exit application
- [ ] Restart application
- [ ] Highlight persists in config
- [ ] Open highlight browser
- [ ] Highlight appears in list

---

## 8. Performance Testing

### 8.1 Input Responsiveness

- [ ] Open browser with 1000+ items
- [ ] Navigation keys respond instantly
- [ ] No input lag
- [ ] Smooth scrolling

### 8.2 Memory Usage

- [ ] Open/close widgets 100 times
- [ ] Check memory usage
- [ ] No memory leaks
- [ ] Widget cleanup works

### 8.3 Render Performance

- [ ] Open widget over busy game output
- [ ] Widget renders cleanly
- [ ] No flickering
- [ ] Background updates continue

---

## 9. Accessibility & UX

### 9.1 Visual Feedback

- [ ] Selected items clearly highlighted
- [ ] Focus indicator visible
- [ ] Checkboxes show state clearly
- [ ] Dropdowns show current value
- [ ] Color previews render correctly

### 9.2 Help Text

- [ ] Widget titles descriptive
- [ ] Placeholder text helpful
- [ ] Error messages clear
- [ ] Status messages shown

### 9.3 Keyboard-Only Operation

- [ ] All widgets operable without mouse
- [ ] Tab order logical
- [ ] No trapped focus
- [ ] All actions accessible via keyboard

---

## 10. Regression Testing

### 10.1 Existing Features Still Work

- [ ] Window editor still works
- [ ] Popup menu still works
- [ ] Search mode still works
- [ ] History mode still works
- [ ] Normal navigation still works
- [ ] Game commands still work

### 10.2 Config Compatibility

- [ ] Old configs without menu_keybinds load
- [ ] Defaults applied correctly
- [ ] No breaking changes to existing config

---

## Testing Summary

**Total Test Cases:** ~150+

**Critical Tests (Must Pass):**
- [ ] All browsers open and close
- [ ] All forms save/cancel correctly
- [ ] Esc always returns to Normal mode
- [ ] No crashes or panics
- [ ] Keybinds route correctly by mode
- [ ] Validator auto-fixes critical issues

**Priority Levels:**
- **P0 (Critical):** Navigation, Save/Cancel, Mode transitions
- **P1 (High):** Clipboard operations, Toggles, Dropdowns
- **P2 (Medium):** Edge cases, Performance
- **P3 (Low):** Polish, UX improvements

---

## Bug Report Template

If issues found during testing, use this template:

```markdown
## Bug Report

**Widget:** [Browser/Form/Editor name]
**Action:** [What were you trying to do?]
**Expected:** [What should happen?]
**Actual:** [What actually happened?]
**Steps to Reproduce:**
1.
2.
3.

**Logs:** [Relevant log output]
**Config:** [Any custom keybinds?]
**Severity:** [Critical/High/Medium/Low]
```

---

## Testing Notes

- Test on clean config first (delete `~/.two-face` or `~/.vellum-fe`)
- Test with custom config after baseline
- Check logs (`two-face.log`) for warnings/errors
- Use debug builds for better error messages
- Test both Windows and Linux if possible
- Document any workarounds needed

**Happy Testing! 🧪**
