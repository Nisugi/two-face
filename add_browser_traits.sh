#!/bin/bash

# Add Navigable and Selectable traits to all browser widgets

# KeybindBrowser
cat >> src/frontend/tui/keybind_browser.rs << 'EOF'

// Trait implementations for KeybindBrowser
use super::widget_traits::{Navigable, Selectable};

impl Navigable for KeybindBrowser {
    fn navigate_up(&mut self) {
        self.previous();
    }

    fn navigate_down(&mut self) {
        self.next();
    }

    fn page_up(&mut self) {
        self.page_up();
    }

    fn page_down(&mut self) {
        self.page_down();
    }
}

impl Selectable for KeybindBrowser {
    fn get_selected(&self) -> Option<String> {
        self.get_selected()
    }

    fn delete_selected(&mut self) -> Option<String> {
        let combo = self.get_selected()?;
        self.entries.retain(|e| e.key_combo != combo);
        let filtered = self.filtered_entries();
        if self.selected_index >= filtered.len() && self.selected_index > 0 {
            self.selected_index = filtered.len() - 1;
        }
        self.adjust_scroll();
        Some(combo)
    }
}
EOF

# ColorPaletteBrowser
cat >> src/frontend/tui/color_palette_browser.rs << 'EOF'

// Trait implementations for ColorPaletteBrowser
use super::widget_traits::{Navigable, Selectable};

impl Navigable for ColorPaletteBrowser {
    fn navigate_up(&mut self) {
        self.previous();
    }

    fn navigate_down(&mut self) {
        self.next();
    }

    fn page_up(&mut self) {
        self.page_up();
    }

    fn page_down(&mut self) {
        self.page_down();
    }
}

impl Selectable for ColorPaletteBrowser {
    fn get_selected(&self) -> Option<String> {
        self.get_selected()
    }

    fn delete_selected(&mut self) -> Option<String> {
        let name = self.get_selected()?;
        self.colors.retain(|c| c.name != name);
        let filtered = self.filtered_colors();
        if self.selected_index >= filtered.len() && self.selected_index > 0 {
            self.selected_index = filtered.len() - 1;
        }
        self.adjust_scroll();
        Some(name)
    }
}
EOF

# SpellColorBrowser
cat >> src/frontend/tui/spell_color_browser.rs << 'EOF'

// Trait implementations for SpellColorBrowser
use super::widget_traits::{Navigable, Selectable};

impl Navigable for SpellColorBrowser {
    fn navigate_up(&mut self) {
        self.previous();
    }

    fn navigate_down(&mut self) {
        self.next();
    }

    fn page_up(&mut self) {
        self.page_up();
    }

    fn page_down(&mut self) {
        self.page_down();
    }
}

impl Selectable for SpellColorBrowser {
    fn get_selected(&self) -> Option<String> {
        self.selected_index.to_string().into()
    }

    fn delete_selected(&mut self) -> Option<String> {
        if self.selected_index < self.spell_colors.len() {
            self.spell_colors.remove(self.selected_index);
            if self.selected_index >= self.spell_colors.len() && self.selected_index > 0 {
                self.selected_index -= 1;
            }
            self.adjust_scroll();
            Some(self.selected_index.to_string())
        } else {
            None
        }
    }
}
EOF

# UIColorsBrowser
cat >> src/frontend/tui/uicolors_browser.rs << 'EOF'

// Trait implementations for UIColorsBrowser
use super::widget_traits::{Navigable, Selectable};

impl Navigable for UIColorsBrowser {
    fn navigate_up(&mut self) {
        self.previous();
    }

    fn navigate_down(&mut self) {
        self.next();
    }

    fn page_up(&mut self) {
        self.page_up();
    }

    fn page_down(&mut self) {
        self.page_down();
    }
}

impl Selectable for UIColorsBrowser {
    fn get_selected(&self) -> Option<String> {
        self.get_selected()
    }

    fn delete_selected(&mut self) -> Option<String> {
        // UI colors can't be deleted, only edited
        None
    }
}
EOF

echo "Added trait implementations to all 5 browser widgets"
