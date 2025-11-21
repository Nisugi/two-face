# Theme Field Reference

**Last Updated**: 2025-01-18
**Source**: Actual codebase analysis of usage in `src/frontend/tui/`

This document describes what each theme field **actually** affects in the application, based on real code usage. Fields marked as UNUSED are defined but not currently used anywhere in the codebase.

---

## Meta Fields

### `name` (String)
**Used**: Yes
**Purpose**: Theme display name
**Locations**: Theme browser, theme editor
**UI Impact**: Shows in `.themes` browser list

### `description` (String)
**Used**: Yes
**Purpose**: Theme description
**Locations**: Theme browser
**UI Impact**: Shows in `.themes` browser details

---

## Window Colors

### `window_border` (Color)
**Used**: Yes (~20 references)
**Purpose**: Default window border color
**Locations**:
- `src/frontend/tui/mod.rs`: Lines 260, 589, 1270, 1271
- `src/frontend/tui/text_window.rs`: Line 1637
- `src/frontend/tui/theme_editor.rs`: Multiple lines

**UI Impact**:
- Default border for all windows when not focused
- Fallback when window-specific `border_color` is not set
- Used by text windows, tabbed windows, compass, hands, etc.

### `window_border_focused` (Color)
**Used**: Yes (~5 references)
**Purpose**: Border color for focused window
**Locations**:
- `src/frontend/tui/mod.rs`: Line 1271

**UI Impact**:
- Border highlight for the currently focused window
- Provides visual feedback for window focus

### `window_background` (Color)
**Used**: Yes (~8 references)
**Purpose**: Background fill for windows
**Locations**:
- `src/frontend/tui/mod.rs`: Lines 265, 601, 613, 1276
- `src/frontend/tui/theme_editor.rs`: Multiple lines

**UI Impact**:
- Background color for window content areas
- Used in injury doll color derivation
- Fallback for various widgets

### `window_title` (Color)
**Used**: Minimal (1 reference)
**Purpose**: Window title text
**Locations**:
- `src/frontend/tui/theme_editor.rs`: Serialization only

**UI Impact**:
- Rarely used directly in rendering
- Most windows use `text_primary` for titles

---

## Text Colors

### `text_primary` (Color)
**Used**: Yes (~30 references)
**Purpose**: Primary text color for all content
**Locations**:
- `src/frontend/tui/mod.rs`: Lines 268, 608, 617, 1277, 1280
- `src/frontend/tui/settings_editor.rs`: Lines 728, 762, 840, 871
- `src/frontend/tui/theme_editor.rs`: Lines 840, 862, 871
- Most widget files

**UI Impact**:
- Main text throughout the application
- Labels, content, descriptions
- Default fallback for most text rendering

### `text_secondary` (Color)
**Used**: Yes (~15 references)
**Purpose**: Secondary/dimmed text
**Locations**:
- `src/frontend/tui/mod.rs`: Lines 268, 617
- `src/frontend/tui/settings_editor.rs`: Multiple lines
- `src/frontend/tui/theme_editor.rs`: Multiple lines

**UI Impact**:
- Less important information
- Descriptions, hints
- Used in injury doll color derivation

### `text_disabled` (Color)
**Used**: Yes (~15 references)
**Purpose**: Disabled/inactive text
**Locations**:
- `src/frontend/tui/keybind_browser.rs`: Line 273
- `src/frontend/tui/settings_editor.rs`: Lines 521, 543, 702
- `src/frontend/tui/theme_editor.rs`: Line 742

**UI Impact**:
- Disabled menu items
- Inactive options
- Grayed-out text

### `text_selected` (Color)
**Used**: Minimal (~3 references)
**Purpose**: Selected text highlight
**Locations**:
- `src/frontend/tui/theme_editor.rs`: Lines 723, 862

**UI Impact**:
- Text that is currently selected
- Used in theme editor color field rendering

---

## Background Colors

### `background_primary` (Color)
**Used**: UNUSED
**Purpose**: Intended for primary background
**UI Impact**: None - field is defined but never referenced

### `background_secondary` (Color)
**Used**: UNUSED
**Purpose**: Intended for secondary background
**UI Impact**: None - field is defined but never referenced

### `background_selected` (Color)
**Used**: UNUSED (only in `get_color` lookup)
**Purpose**: Intended for selected item backgrounds
**UI Impact**: None - not used in actual rendering

### `background_hover` (Color)
**Used**: UNUSED
**Purpose**: Intended for hover state backgrounds
**UI Impact**: None - field is defined but never referenced

---

## Editor Colors (via EditorTheme conversion)

These fields are converted via `theme.to_editor_theme()` in `src/theme.rs` and used by window editor.

### `editor_border` (Color)
**Used**: Yes (via EditorTheme)
**Purpose**: Window editor border
**UI Impact**: Border color for window editor popup

### `editor_label` (Color)
**Used**: Yes (via EditorTheme)
**Purpose**: Window editor field labels
**UI Impact**: Color for "Name:", "Title:", "Row:", etc. labels

### `editor_label_focused` (Color)
**Used**: Yes (via EditorTheme)
**Purpose**: Focused field label highlight
**UI Impact**: Label color when field has focus

### `editor_text` (Color)
**Used**: Yes (via EditorTheme)
**Purpose**: Input text in editor
**UI Impact**: Text color in input fields

### `editor_cursor` (Color)
**Used**: Yes (via EditorTheme)
**Purpose**: Cursor color
**UI Impact**: Cursor highlight in text inputs

### `editor_status` (Color)
**Used**: Yes (via EditorTheme)
**Purpose**: Status messages in editor
**UI Impact**: Currently not rendered (status message removed)

### `editor_background` (Color)
**Used**: Yes (via EditorTheme)
**Purpose**: Editor popup background
**UI Impact**: Background fill for window editor

---

## Browser/List Colors

### `browser_border` (Color)
**Used**: Yes (~10 references)
**Purpose**: Browser widget borders
**Locations**:
- `src/frontend/tui/highlight_browser.rs`
- `src/frontend/tui/keybind_browser.rs`
- `src/frontend/tui/color_palette_browser.rs`
- `src/frontend/tui/spell_color_browser.rs`
- `src/frontend/tui/uicolors_browser.rs`
- `src/frontend/tui/theme_browser.rs`

**UI Impact**: Border color for all browser widgets

### `browser_title` (Color)
**Used**: Minimal (1 reference)
**Purpose**: Browser title text
**Locations**:
- `src/frontend/tui/highlight_browser.rs`: Line 264

**UI Impact**: Title text in highlight browser (rarely used)

### `browser_item_normal` (Color)
**Used**: Yes (~20 references)
**Purpose**: Normal list item text color
**Locations**: All browser widgets

**UI Impact**:
- Text color for unselected items in lists
- Default item appearance

### `browser_item_selected` (Color)
**Used**: Yes (~15 references)
**Purpose**: Selected item background
**Locations**: All browser widgets

**UI Impact**:
- Background color when item is selected
- Usually with inverted text color

### `browser_item_focused` (Color)
**Used**: Yes (~40 references) - MOST USED FIELD
**Purpose**: Focused/highlighted item color
**Locations**:
- `src/frontend/tui/highlight_browser.rs`: Lines 297, 336, 414, 418
- `src/frontend/tui/keybind_browser.rs`: Lines 304, 337, 386, 388
- `src/frontend/tui/color_palette_browser.rs`: Lines 346, 382, 418
- All other browsers

**UI Impact**:
- Foreground color for the currently focused item
- Primary visual indicator in lists
- Creates the "→ Item" highlight effect

### `browser_background` (Color)
**Used**: Yes (~80+ references) - HEAVILY USED
**Purpose**: Browser/form background fill
**Locations**:
- All browser widgets (highlight, keybind, color palette, spell color, uicolors, theme)
- All form widgets (highlight, keybind, color, spell color)
- Settings editor
- Theme editor

**UI Impact**:
- Background fill for all browser popups
- Background fill for all form popups
- Background for settings editor
- One of the most important theme fields

### `browser_scrollbar` (Color)
**Used**: UNUSED
**Purpose**: Intended for scrollbar
**UI Impact**: None - scrollbars not currently rendered

---

## Form Colors

### `form_border` (Color)
**Used**: Yes (~10 references)
**Purpose**: Form widget borders
**Locations**: All form widgets

**UI Impact**: Border color for form popups

### `form_label` (Color)
**Used**: Yes (~60 references) - HEAVILY USED
**Purpose**: Form field labels
**Locations**:
- `src/frontend/tui/highlight_form.rs`: Lines 511, 540, 742, 750, 758, etc.
- `src/frontend/tui/keybind_form.rs`: Lines 431, 462, 517, 532, etc.
- `src/frontend/tui/color_form.rs`: Lines 336, 477, 480, etc.
- `src/frontend/tui/spell_color_form.rs`: Multiple lines

**UI Impact**:
- Labels for all form fields ("Pattern:", "Colors:", etc.)
- One of the most visible theme elements in forms

### `form_label_focused` (Color)
**Used**: Yes (~60 references) - HEAVILY USED
**Purpose**: Focused field label highlight
**Locations**: Same as `form_label`

**UI Impact**:
- Label color when field is focused
- Provides visual feedback for current field
- Critical for form navigation

### `form_field_background` (Color)
**Used**: Minimal (~5 references)
**Purpose**: Input field backgrounds
**Locations**:
- `src/frontend/tui/theme_editor.rs`: Lines 717, 837, 838, 947, 948

**UI Impact**:
- Background for color input fields in theme editor
- Not widely used in other forms

### `form_field_text` (Color)
**Used**: Minimal (~5 references)
**Purpose**: Input field text
**Locations**:
- `src/frontend/tui/theme_editor.rs`: Same lines as above

**UI Impact**:
- Text color in input fields (theme editor)
- Most forms use `text_primary` instead

### `form_checkbox_checked` (Color)
**Used**: Minimal (serialization only)
**Purpose**: Checked checkbox color
**Locations**:
- `src/frontend/tui/theme_editor.rs`: Serialization

**UI Impact**: Not used in actual rendering

### `form_checkbox_unchecked` (Color)
**Used**: Minimal (serialization only)
**Purpose**: Unchecked checkbox color
**Locations**:
- `src/frontend/tui/theme_editor.rs`: Serialization

**UI Impact**: Not used in actual rendering

### `form_error` (Color)
**Used**: Minimal (serialization only)
**Purpose**: Form error messages
**Locations**:
- `src/frontend/tui/theme_editor.rs`: Serialization

**UI Impact**: Not used in actual rendering

---

## Menu/Popup Colors

### `menu_border` (Color)
**Used**: Yes (~5 references)
**Purpose**: Popup menu borders
**Locations**:
- `src/frontend/tui/popup_menu.rs`: Line 153

**UI Impact**: Border for context menus and popups

### `menu_background` (Color)
**Used**: Yes (~5 references)
**Purpose**: Menu background fill
**Locations**:
- `src/frontend/tui/popup_menu.rs`

**UI Impact**: Background for popup menus

### `menu_item_normal` (Color)
**Used**: Yes (~5 references)
**Purpose**: Normal menu item text
**Locations**:
- `src/frontend/tui/popup_menu.rs`

**UI Impact**: Text color for unselected menu items

### `menu_item_selected` (Color)
**Used**: Yes (~10 references)
**Purpose**: Selected menu item background
**Locations**:
- `src/frontend/tui/popup_menu.rs`
- Some browser widgets

**UI Impact**: Background when menu item is selected

### `menu_item_focused` (Color)
**Used**: Yes (~5 references)
**Purpose**: Focused menu item highlight
**Locations**:
- `src/frontend/tui/popup_menu.rs`

**UI Impact**: Highlight for focused menu item

### `menu_separator` (Color)
**Used**: Yes (~10 references)
**Purpose**: Separator lines in menus/lists
**Locations**:
- `src/frontend/tui/color_palette_browser.rs`
- `src/frontend/tui/highlight_browser.rs`

**UI Impact**:
- Horizontal separator lines
- Section dividers in lists

---

## Status Colors

### `status_info` (Color)
**Used**: UNUSED
**Purpose**: Intended for info messages
**UI Impact**: None - not implemented

### `status_success` (Color)
**Used**: UNUSED
**Purpose**: Intended for success messages
**UI Impact**: None - not implemented

### `status_warning` (Color)
**Used**: UNUSED
**Purpose**: Intended for warning messages
**UI Impact**: None - not implemented

### `status_error` (Color)
**Used**: UNUSED
**Purpose**: Intended for error messages
**UI Impact**: None - not implemented

### `status_background` (Color)
**Used**: Minimal (serialization only)
**Purpose**: Intended for status bar background
**UI Impact**: Not used in rendering

---

## Interactive Elements

### `button_normal` (Color)
**Used**: Minimal (serialization only)
**Purpose**: Intended for normal button state
**UI Impact**: Not used - no buttons in TUI

### `button_hover` (Color)
**Used**: UNUSED
**Purpose**: Intended for button hover state
**UI Impact**: None - no buttons in TUI

### `button_active` (Color)
**Used**: UNUSED
**Purpose**: Intended for active/pressed button
**UI Impact**: None - no buttons in TUI

### `button_disabled` (Color)
**Used**: Minimal (serialization only)
**Purpose**: Intended for disabled buttons
**UI Impact**: Not used - no buttons in TUI

---

## Game-Specific Colors

### `command_echo` (Color)
**Used**: UNUSED
**Purpose**: Intended for command echo text
**UI Impact**: None - `config.colors.ui.command_echo_color` used instead

**Note**: There's a duplicate field in `UiColors` config that is actually used. This theme field is redundant.

### `selection_background` (Color)
**Used**: UNUSED
**Purpose**: Intended for text selection background
**UI Impact**: None - `config.colors.ui.selection_bg_color` used instead

**Note**: Same duplication issue as `command_echo`.

### `link_color` (Color)
**Used**: Minimal (only in `get_color` lookup)
**Purpose**: Intended for hyperlink text
**UI Impact**: Available via `theme.get_color("link_color")` but not directly used in rendering

### `speech_color` (Color)
**Used**: UNUSED
**Purpose**: Intended for NPC/player speech text
**UI Impact**: None - game text parsing not implemented

### `whisper_color` (Color)
**Used**: UNUSED
**Purpose**: Intended for whispered text
**UI Impact**: None - game text parsing not implemented

### `thought_color` (Color)
**Used**: UNUSED
**Purpose**: Intended for character thoughts
**UI Impact**: None - game text parsing not implemented

### `injury_default_color` (Color)
**Used**: Yes (1 reference, derived)
**Purpose**: Default color for injury doll widget
**Locations**:
- `src/frontend/tui/mod.rs`: Line 1335

**UI Impact**:
- Color for injury doll overlay
- Derived from blend of `window_background` and `text_secondary`

---

## Summary Statistics

| Category | Total Fields | Actually Used | Unused | Usage Rate |
|----------|-------------|---------------|---------|-----------|
| **Meta** | 2 | 2 | 0 | 100% |
| **Window** | 4 | 4 | 0 | 100% |
| **Text** | 4 | 4 | 0 | 100% |
| **Background** | 4 | 0 | 4 | 0% |
| **Editor** | 7 | 7 | 0 | 100% |
| **Browser** | 7 | 6 | 1 | 86% |
| **Form** | 8 | 3 | 5 | 38% |
| **Menu** | 6 | 6 | 0 | 100% |
| **Status** | 5 | 0 | 5 | 0% |
| **Buttons** | 4 | 0 | 4 | 0% |
| **Game** | 7 | 1 | 6 | 14% |
| **TOTAL** | **58** | **33** | **25** | **57%** |

(Plus 2 meta fields, 77 total)

---

## Top 10 Most Used Fields

1. **browser_background** (~80+ references) - Browser/form backgrounds
2. **form_label** + **form_label_focused** (~60 each) - Form field labels
3. **browser_item_focused** (~40) - List item highlighting
4. **text_primary** (~30) - General text
5. **window_border** (~20) - Window borders
6. **browser_item_normal** (~20) - Normal list items
7. **browser_item_selected** (~15) - Selected items
8. **text_disabled** (~15) - Disabled text
9. **text_secondary** (~15) - Secondary text
10. **menu_separator** (~10) - List separators

---

## Recommendations

### High Priority - Remove These
These fields have zero actual usage and should be removed:
- All `status_*` colors (5 fields)
- All `button_*` colors (4 fields)
- All `background_*` colors (4 fields)
- Game-specific: `speech_color`, `whisper_color`, `thought_color`
- Duplicates: `command_echo`, `selection_background`

**Total Removable**: 20 fields

### Medium Priority - Consider Removing
These fields are only in serialization, not actual rendering:
- `form_checkbox_checked`, `form_checkbox_unchecked`, `form_error`
- `window_title`, `browser_title`, `browser_scrollbar`

### High Priority - Actually Use These
If keeping these fields, implement them properly:
- Status colors → Form validation, error messages
- Game-specific colors → Text window parsing
- `form_field_background` → All forms, not just theme editor
