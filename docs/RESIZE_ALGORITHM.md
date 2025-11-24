# VellumFE-Compatible Automatic Resize Algorithm

## Overview

Two-Face implements the VellumFE resize algorithm for automatic, proportional window resizing when the terminal size changes. This algorithm ensures that all windows resize smoothly and maintain their relative proportions, matching the behavior of the original VellumFE client.

## Core Principles

The VellumFE algorithm is based on three fundamental principles:

1. **Column-by-Column Height Calculation**: Process each terminal column independently to calculate height deltas
2. **Row-by-Row Width Calculation**: Process each terminal row independently to calculate width deltas
3. **Separate Calculation and Application**: Calculate all deltas first, then apply them with cascading
4. **Baseline Snapshot for Proportions**: Use the original (baseline) rows/cols for proportional math to avoid feedback from earlier steps
5. **Local Leftovers Only**: Leftovers are distributed within each row/column independently; no global leftover reconciliation
6. **Already-Applied Windows**: Windows still participate in proportional math on later rows/cols, but if they are already applied their assigned delta is discarded (only the cascade advances)

## Algorithm Phases

### Phase 1: Height Resize (Column-by-Column)

#### Step 1: Calculate Height Deltas

For each column from 0 to max_col:

1. **Find windows occupying this column**
   - A window occupies column `C` if: `window.col <= C < window.col + window.cols`
   - Windows already applied still participate in proportional calculation; their deltas are discarded later if already applied

2. **Calculate total scalable height**
   - Sum the *baseline* heights of all non-static windows at this column (baseline snapshot prevents feedback from earlier columns)
   - Skip `static_both` and `static_height` windows (they don't resize)

3. **Distribute height_delta proportionally**
   - For each non-static window:
     - `proportion = baseline_rows / total_scalable_height`
     - `delta = floor(proportion * height_delta)`
     - Record delta for this column
   - For static windows: record delta of 0

4. **First encounter wins**
   - Each window's delta is applied the FIRST time it's encountered at any column
   - Subsequent columns still see the window in proportional math, but their assigned deltas are discarded if the window is already applied

5. **Leftover distribution** (VellumFE-compatible)
   - After proportional distribution **within the column**, calculate leftover: `leftover = height_delta - sum(all_deltas_for_column)`
   - Distribute leftover rows one-by-one to first windows (sorted top to bottom) in this column only
   - Skip static windows for leftovers

#### Step 2: Apply Height Deltas with Column-by-Column Cascading

**Cascading** means: within each column, windows stack vertically with each window starting where the previous one ended.

**Algorithm**: Iterate through each column independently

```
height_applied = {}  // Track which windows have been processed

for current_col in 0..max_col:
    // Find all windows occupying this column (applied or not)
    windows_at_col = filter windows where current_col is within their column range

    if windows_at_col is empty:
        continue

    // Sort by row (top to bottom)
    sort windows_at_col by row

    // Cascade vertically within this column
    current_row = windows_at_col[0].row  // Start at first window's row

    for window in windows_at_col:
        if window already in height_applied:
            // discard delta, just advance cascade by existing size
            current_row = window.row + window.rows
            continue
        window.row = current_row
        window.rows = original_rows + delta  // constrained by min/max
        height_applied.add(window_name)
        current_row += window.rows  // Next window cascades
```

**Example - Column 0**:
- Windows: active_spells (row 1, 18 rows), inventory (row 19, 13 rows)
- active_spells: row 1, height 18+2=20 → ends at row 21
- inventory: row 21 (cascaded), height 13+2=15 → ends at row 36
- Both marked as applied

**Example - Columns 1-19**:
- Windows: active_spells, inventory
- Both already in `height_applied` → skip

**Example - Column 20**:
- Windows: main (row 1, 40 rows), room (row 41, 10 rows), command (row 51, 3 rows)
- main: row 1, height 40+5=45 → ends at row 46
- room: row 46 (cascaded), height 10+2=12 → ends at row 58
- command: row 58 (cascaded), height 3+1=4 → ends at row 62

**Key Point**: Each column processes independently. Windows already processed are skipped in subsequent columns.

### Phase 2: Width Resize (Row-by-Row)

#### Step 1: Calculate Width Deltas

For each row from 0 to max_row:

1. **Find windows occupying this row**
   - A window occupies row `R` if: `window.row <= R < window.row + window.rows`
   - Windows already applied still participate in proportional calculation; their deltas are discarded later if already applied

2. **Calculate total scalable width**
   - Sum the *baseline* widths of all non-static windows at this row (baseline snapshot prevents feedback from earlier rows)
   - Skip `static_both` windows (they don't resize horizontally)

3. **Distribute width_delta proportionally**
   - For each non-static window:
     - `proportion = baseline_cols / total_scalable_width`
     - `delta = floor(proportion * width_delta)`
     - Record delta for this row
   - For static windows: record delta of 0

4. **First encounter wins**
   - Each window's delta is applied the FIRST time it's encountered at any row
   - Subsequent rows still see the window in proportional math, but their assigned deltas are discarded if the window is already applied

5. **Leftover distribution** (VellumFE-compatible)
   - After proportional distribution **within the row**, calculate leftover: `leftover = width_delta - sum(all_deltas_for_row)`
   - Distribute leftover columns one-by-one to first windows (sorted left to right) in this row only
   - Skip static windows for leftovers

#### Step 2: Apply Width Deltas with Row-by-Row Cascading

**Cascading** means: within each row, windows align horizontally with each window starting where the previous one ended.

**Algorithm**: Iterate through each row independently

```
width_applied = {}  // Track which windows have been processed

for current_row in 0..max_row:
    // Find all windows occupying this row (applied or not)
    windows_at_row = filter windows where current_row is within their row range

    if windows_at_row is empty:
        continue

    // Sort by column (left to right)
    sort windows_at_row by col

    // Cascade horizontally within this row
    current_col = windows_at_row[0].col  // Start at first window's column

    for window in windows_at_row:
        if window already in width_applied:
            // discard delta, just advance cascade by existing size
            current_col = window.col + window.cols
            continue
        window.col = current_col
        window.cols = original_cols + delta  // constrained by min/max
        width_applied.add(window_name)
        current_col += window.cols  // Next window cascades
```

**Example - Row 1**:
- Windows: active_spells (col 0, 20 cols), main (col 20, 95 cols)
- active_spells: col 0, width 20+2=22 → ends at col 22
- main: col 22 (cascaded from 20), width 95+8=103 → ends at col 125
- Both marked as applied

**Example - Rows 2-18**:
- Windows: active_spells, main
- Both already in `width_applied` → skip

**Example - Row 19**:
- Windows: inventory (col 0, 20 cols), main (col 20, 95 cols)
- inventory: col 0, width 20+2=22 → ends at col 22
- main: already in `width_applied` → skip (already processed at row 1)

**Key Point**: Each row processes independently. Windows already processed are skipped in subsequent rows.

## Example: test3.toml Layout

### Initial Layout (60x115 terminal)

```
Row   0: spacer_2 (1 row × 115 cols)
Row   1: active_spells (18 rows × 20 cols), main (40 rows × 95 cols)
Row  19: inventory (13 rows × 20 cols), main (40 rows × 95 cols)
Row  32: spacer_1 (15 rows × 20 cols), main (40 rows × 95 cols)
Row  41: spacer_1 (15 rows × 20 cols), room (10 rows × 95 cols)
Row  47: left_hand (3 rows × 20 cols), room (10 rows × 95 cols)
Row  50: spell_hand (3 rows × 20 cols), room (10 rows × 95 cols)
Row  51: spell_hand (3 rows × 20 cols), command (3 rows × 95 cols)
Row  53: health (3 rows × 20 cols), command (3 rows × 95 cols)
Row  54: health (3 rows × 20 cols), thoughts (10 rows × 95 cols)
Row  56: injury (8 rows × 10 cols), spacer_3 (3 rows × 10 cols), thoughts (10 rows × 95 cols)
Row  59: injury (8 rows × 10 cols), compass (5 rows × 10 cols), thoughts (10 rows × 95 cols)
```

### Resize to 65x120 (+5 rows, +5 cols)

#### Phase 1: Height Resize (Row-by-Row)

**Row 0**:
- Windows: spacer_2
- spacer_2 is static_height → delta = 0

**Row 1**:
- Windows: active_spells, main
- Total scalable: 18 + 40 = 58 rows
- active_spells: proportion = 18/58 = 0.31, delta = floor(5 * 0.31) = 1
- main: proportion = 40/58 = 0.69, delta = floor(5 * 0.69) = 3

**Row 19**:
- Windows: inventory, main
- active_spells already processed, main already processed
- Total scalable: 13 rows (only inventory)
- inventory: proportion = 13/13 = 1.0, delta = floor(5 * 1.0) = 5

*And so on for each row...*

**After all deltas calculated**:
```
spacer_2: +0 rows (static)
active_spells: +1 row → 19 rows
inventory: +5 rows → 18 rows (some got more due to distribution)
main: +3 rows → 43 rows
...
```

**Apply with Cascading**:
- Windows sorted by row: spacer_2, active_spells, inventory, spacer_1, ...
- Each window placed at original row if columns free, else cascaded

#### Phase 2: Width Resize (Column-by-Column)

**Column 0**:
- Windows: spacer_2, active_spells, inventory, spacer_1, left_hand, spell_hand, health, injury
- Total scalable width at column 0: sum of all non-static window widths
- Each unprocessed window gets proportional delta

**Column 10**:
- Windows: spacer_2, active_spells, inventory, spacer_1, left_hand, spell_hand, health, spacer_3, compass
- Windows already processed from column 0 are skipped
- New windows (spacer_3, compass) get their deltas

**Column 20**:
- Windows: spacer_2, main, room, command, thoughts
- All new windows get their proportional deltas

*And so on for each column...*

**Apply with row-by-row cascading**:
- Iterate through each row independently
- Windows in each row cascade horizontally (left to right)
- Track `width_applied` to skip already-processed windows

## Implementation Details

### Data Structures

```rust
// Height calculation and application
let mut height_deltas: HashMap<String, i32> = HashMap::new();
let mut height_applied: HashSet<String> = HashSet::new();

// Width calculation and application
let mut width_deltas: HashMap<String, i32> = HashMap::new();
let mut width_applied: HashSet<String> = HashSet::new();
```

### Key Functions

#### `apply_height_resize()`
- **Input**: height_delta, static_both set, static_height set
- **Output**: Modified window positions and heights
- **Lines**: ~200 lines in [src/core/app_core.rs](../src/core/app_core.rs#L1818-L2026)

#### `apply_width_resize()`
- **Input**: width_delta, static_both set
- **Output**: Modified window widths
- **Lines**: ~150 lines in [src/core/app_core.rs](../src/core/app_core.rs#L2031-L2177)

### Window Constraints

Both functions respect:

- **min_rows / min_cols**: Minimum size from widget type or explicit constraint
- **max_rows / max_cols**: Maximum size from explicit constraint
- **static_both**: No height or width changes
- **static_height**: No height changes (width can change)

## Differences from Previous Implementation

### Old Approach (WRONG)
- **Critical Bug**: Found ALL windows at each row/column, including already-processed windows
- ALL windows participated in proportional calculation at every row/column they touched
- Result: Over-distributed deltas (e.g., distributing 26 columns when terminal only grew by 7)
- Leftover was negative, requiring subtraction to compensate

### New Approach (VellumFE-compatible - CORRECT)
- **Key Fix**: Skip windows that already have deltas when building windows list
- Only UNPROCESSED windows participate in calculation at each row/column
- Each window gets its delta exactly ONCE (first encounter wins)
- Result: Correct total delta distribution matching terminal size change
- Leftover is positive (for rounding compensation) or zero

## Testing

### Manual Testing

1. Load test3.toml layout
2. Resize terminal from 60×115 to various sizes
3. Verify all windows resize proportionally
4. Check logs with `RUST_LOG=two_face=debug`

### Expected Log Output

```
--- HEIGHT SCALING (VellumFE ROW-BY-ROW) ---
Processing rows 0..64
Row 0: 1 windows present
  Total scalable height at row 0: 0
Row 1: 2 windows present
  Total scalable height at row 1: 58
    active_spells (rows=18): proportion=0.3103, delta=1
    main (rows=40): proportion=0.6897, delta=3
...
Height deltas calculated for 14 windows

--- WIDTH SCALING (VellumFE COLUMN-BY-COLUMN) ---
Processing columns 0..115
Column 0: 8 windows present
  Total scalable width at column 0: 115
    spacer_2 (cols=115): proportion=1.0000, delta=5
...
Width deltas calculated for 14 windows
```

## Performance

- **Time Complexity**: O(rows × windows + cols × windows)
  - Typically: O(60 × 15 + 115 × 15) = ~2,500 operations
  - Acceptable for interactive terminal resize

- **Space Complexity**: O(windows)
  - Two HashMaps storing deltas for ~15-20 windows
  - Column occupation map: O(rows × average_window_width)

## Leftover Distribution

VellumFE's leftover distribution algorithm ensures that rounding errors don't leave gaps. After proportional distribution using `floor()`, any remaining rows/columns are distributed one-by-one to the first windows (sorted by position).

**Example** (width resize from 115 to 122, +7 columns):
- Row 0: 2 windows (active_spells 20 cols + main 95 cols) = 115 total
  - active_spells: `floor(20/115 * 7) = floor(1.217) = 1`
  - main: `floor(95/115 * 7) = floor(5.783) = 5`
  - Total distributed: 1 + 5 = 6
  - Leftover: 7 - 6 = 1 column
  - First window (active_spells) gets +1 extra → final: active_spells +2, main +5

This ensures windows always fill the entire terminal space with no gaps.

## Future Improvements

1. **Performance**: Cache window lookups to reduce iterations
2. **Testing**: Add unit tests with known layouts and expected results

## References

- VellumFE Ruby source: Original resize algorithm
- [src/core/app_core.rs](../src/core/app_core.rs): Two-Face implementation
- [test3.toml](../layouts/test3.toml): Test layout configuration
