#!/usr/bin/env python3
"""
Fix color type mismatches in TUI widget files by adding conversion calls.
Only converts Style methods .fg() and .bg() when called with theme fields.
"""

import re
import glob
import os

def fix_color_conversions(file_path):
    """Fix color conversions in a single file."""
    with open(file_path, 'r', encoding='utf-8') as f:
        lines = f.readlines()

    original_lines = lines.copy()
    changes_made = False

    # Process line by line to avoid corrupting syntax
    for i, line in enumerate(lines):
        original_line = line

        # Only process lines that have .fg( or .bg( followed by theme.
        # Use word boundaries to avoid matching things like Some(theme.xxx)
        if '.fg(theme.' in line or '.bg(theme.' in line:
            # Replace .fg(theme.xxx) with .fg(crossterm_bridge::to_ratatui_color(theme.xxx))
            # But only if it's not already wrapped
            if 'crossterm_bridge::to_ratatui_color' not in line:
                # Pattern: .fg(theme.FIELD) or .bg(theme.FIELD)
                # where FIELD is letters, underscores
                line = re.sub(
                    r'(\.(?:fg|bg)\()(theme\.[a-z_]+)(\))',
                    r'\1crossterm_bridge::to_ratatui_color(\2)\3',
                    line
                )

        if line != original_line:
            lines[i] = line
            changes_made = True

    if not changes_made:
        return False

    # Check if we need to add the import
    has_import = any('use crate::frontend::tui::crossterm_bridge' in line or
                     'use super::crossterm_bridge' in line for line in lines)

    if not has_import:
        # Find the first use statement block and add our import
        for i, line in enumerate(lines):
            if line.strip().startswith('use ') and not line.strip().startswith('use crate::'):
                # Insert after this line
                lines.insert(i + 1, 'use crate::frontend::tui::crossterm_bridge;\n')
                break

    # Write the modified content
    with open(file_path, 'w', encoding='utf-8') as f:
        f.writelines(lines)

    return True

def main():
    """Process all TUI widget files."""
    tui_dir = r'c:\Gemstone\Projects\two-face\src\frontend\tui'
    pattern = os.path.join(tui_dir, '*.rs')

    files_changed = 0
    for file_path in glob.glob(pattern):
        # Skip crossterm_bridge itself
        if 'crossterm_bridge' in file_path:
            continue

        if fix_color_conversions(file_path):
            print(f"Fixed: {os.path.basename(file_path)}")
            files_changed += 1
        else:
            print(f"No changes: {os.path.basename(file_path)}")

    print(f"\nTotal files changed: {files_changed}")

if __name__ == '__main__':
    main()
