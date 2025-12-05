#!/usr/bin/env python3
"""
Fix color type mismatches in TUI widget files by adding conversion calls.
Converts: .fg(theme.xxx) -> .fg(to_ratatui_color(theme.xxx))
Converts: .bg(theme.xxx) -> .bg(to_ratatui_color(theme.xxx))
"""

import re
import glob
import os

def fix_color_conversions(file_path):
    """Fix color conversions in a single file."""
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()

    original_content = content

    # Pattern 1: .fg(theme.xxx) -> .fg(to_ratatui_color(theme.xxx))
    # But NOT if already wrapped
    content = re.sub(
        r'\.fg\((theme\.[a-z_]+)\)',
        r'.fg(crossterm_bridge::to_ratatui_color(\1))',
        content
    )

    # Pattern 2: .bg(theme.xxx) -> .bg(to_ratatui_color(theme.xxx))
    content = re.sub(
        r'\.bg\((theme\.[a-z_]+)\)',
        r'.bg(crossterm_bridge::to_ratatui_color(\1))',
        content
    )

    # Fix double-wrapping if it occurred
    content = content.replace(
        'crossterm_bridge::to_ratatui_color(crossterm_bridge::to_ratatui_color',
        'crossterm_bridge::to_ratatui_color'
    )
    content = content.replace(')))', '))')

    # Check if we need to add the import
    needs_import = 'crossterm_bridge::to_ratatui_color' in content
    has_import = 'use crate::frontend::tui::crossterm_bridge' in content or 'use super::crossterm_bridge' in content

    if needs_import and not has_import:
        # Find the first use statement and add our import after it
        use_match = re.search(r'(use [^;]+;)', content)
        if use_match:
            insert_pos = use_match.end()
            import_statement = '\nuse crate::frontend::tui::crossterm_bridge;'
            content = content[:insert_pos] + import_statement + content[insert_pos:]

    # Only write if changed
    if content != original_content:
        with open(file_path, 'w', encoding='utf-8') as f:
            f.write(content)
        return True
    return False

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

    print(f"\nTotal files changed: {files_changed}")

if __name__ == '__main__':
    main()
