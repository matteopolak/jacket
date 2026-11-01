make_grid :: (cols: int, rows: int) -> Grid {
    assert(cols > 0 && rows > 0);
    g := Grid.{ cols = cols, rows = rows, bottom = rows - 1 };
    g.cells = NewArray(cols * rows, Cell);
    g.saved = NewArray(cols * rows, Cell);

    return g;
}

free_grid :: (g: *Grid) {
    array_free(g.cells);
    array_free(g.saved);
    array_free(g.history);
    array_free(g.osc);
    array_free(g.replies);
    free(g.title);
    g.* = .{};
}

clear_row :: (g: *Grid, row: int) {
    for x: 0..g.cols - 1 g.cells[row * g.cols + x] = .{ attr = g.attr };
}

scroll_up :: (g: *Grid, count: int = 1, record_history: bool = true) {
    for 1..min(count, g.bottom - g.top + 1) {
        if record_history && g.top == 0 && g.bottom == g.rows - 1 && !g.alternate && g.history_limit > 0 {
            if g.history.count < g.history_limit * g.cols {
                for x: 0..g.cols - 1 array_add(*g.history, g.cells[x]);
            } else {
                // Recycle the oldest row instead of copying the entire scrollback on each LF.
                for x: 0..g.cols - 1 g.history[g.history_head + x] = g.cells[x];
                g.history_head = (g.history_head + g.cols) % g.history.count;
            }
        }

        for y: g.top..g.bottom - 1 {
            for x: 0..g.cols - 1 g.cells[y * g.cols + x] = g.cells[(y + 1) * g.cols + x];
        }

        clear_row(g, g.bottom);
    }
}

scroll_down :: (g: *Grid, count: int = 1) {
    for 1..min(count, g.bottom - g.top + 1) {
        for < y: g.top + 1..g.bottom {
            for x: 0..g.cols - 1 g.cells[y * g.cols + x] = g.cells[(y - 1) * g.cols + x];
        }

        clear_row(g, g.top);
    }
}

linefeed :: (g: *Grid) {
    g.wrap_pending = false;
    if g.y == g.bottom scroll_up(g);
    else g.y = min(g.rows - 1, g.y + 1);
}

// Erase both halves when overwriting a wide glyph.
erase_cell :: (g: *Grid, x: int, y: int) {
    at := y * g.cols + x;
    old := g.cells[at];

    if old.width == 0 && x > 0 g.cells[at - 1] = .{ attr = g.attr };
    if old.width == 2 && x + 1 < g.cols g.cells[at + 1] = .{ attr = g.attr };

    g.cells[at] = .{ attr = g.attr };
}

// East Asian and emoji ranges have two columns; combining marks are currently ignored.
cell_width :: (code: u32) -> int {
    if (code >= 0x300 && code <= 0x36f) || (code >= 0xfe00 && code <= 0xfe0f) return 0;
    if (code >= 0x1100 && code <= 0x115f) || (code >= 0x2e80 && code <= 0xa4cf) || (code >= 0xac00 && code <= 0xd7a3) || (code >= 0xf900 && code <= 0xfaff) || (code >= 0xfe10 && code <= 0xfe6f) || (code >= 0xff01 && code <= 0xff60) || (code >= 0x1f300 && code <= 0x1faff) || (code >= 0x20000 && code <= 0x3ffff) return 2;
    return 1;
}

put_code :: (g: *Grid, code: u32) {
    width := min(cell_width(code), g.cols);
    if width == 0 return;

    if g.wrap_pending || g.x + width > g.cols {
        g.x = 0;
        linefeed(g);
    }

    erase_cell(g, g.x, g.y);
    if width == 2 erase_cell(g, g.x + 1, g.y);

    g.cells[g.y * g.cols + g.x] = .{ code = code, width = width, attr = g.attr };
    if width == 2 g.cells[g.y * g.cols + g.x + 1] = .{ code = 0, width = 0, attr = g.attr };

    g.x += width;
    if g.x >= g.cols {
        g.x = g.cols - 1;
        g.wrap_pending = true;
    }
}

resize_grid :: (g: *Grid, cols: int, rows: int) {
    if cols == g.cols && rows == g.rows return;

    cells := NewArray(cols * rows, Cell);
    saved := NewArray(cols * rows, Cell);

    for y: 0..min(rows, g.rows) - 1 {
        for x: 0..min(cols, g.cols) - 1 {
            cells[y * cols + x] = g.cells[y * g.cols + x];
            saved[y * cols + x] = g.saved[y * g.cols + x];
        }

        // A wide cell clipped by the new edge becomes blank.
        if cells[y * cols + cols - 1].width == 2 cells[y * cols + cols - 1] = .{};
        if saved[y * cols + cols - 1].width == 2 saved[y * cols + cols - 1] = .{};
    }

    array_free(g.cells);
    array_free(g.saved);
    array_free(g.history); // old history rows have the old width; no reflow yet
    g.history = .{};
    g.history_head = 0;
    g.cells = cells;
    g.saved = saved;
    g.cols = cols;
    g.rows = rows;
    g.x = min(g.x, cols - 1);
    g.y = min(g.y, rows - 1);
    g.top = 0;
    g.bottom = rows - 1;
    g.wrap_pending = false;
}

row_text :: (g: *Grid, row: int) -> string {
    b: String_Builder;

    for x: 0..g.cols - 1 {
        c := g.cells[row * g.cols + x];
        if c.width == 0 continue;

        buf: [4] u8;
        text := string.{ 4, buf.data };

        U.character_utf32_to_utf8(c.code, *text);
        append(*b, text);
    }

    return builder_to_string(*b);
}

// A logical cell in scrollback, oldest first; storage is a row-aligned ring.
history_cell :: (g: *Grid, index: int) -> Cell {
    return g.history[(g.history_head + index) % g.history.count];
}
