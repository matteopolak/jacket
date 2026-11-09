feed :: (g: *Grid, bytes: string) {
    Vt.vt_feed(*g.parser, bytes, terminal_action, g);
}

terminal_action :: (data: *void, action: Vt.Vt_Action) {
    g := cast(*Grid) data;

    if action.kind == .PRINT {
        put_code(g, action.code);
        return;
    }

    byte := cast(u8) action.code;

    if action.kind == .EXECUTE {
        if byte == 8 g.x = max(0, g.x - 1);
        if byte == 9 g.x = min(g.cols - 1, (g.x / 8 + 1) * 8);
        if byte == 10 || byte == 11 || byte == 12 linefeed(g);
        if byte == 13 g.x = 0;

        g.wrap_pending = false;
    } else if action.kind == .CSI_DISPATCH {
        if action.intermediates.count != 0 return;

        g.params = .{};
        g.param_count = 0;

        for action.params {
            if it.subparam return; // The grid supports semicolon SGR only.
            g.params[g.param_count] = it.value;
            g.param_count += 1;
        }

        if action.private_marker != 0 && action.private_marker != #char "?" return;

        g.private_mode = action.private_marker == #char "?";
        csi(g, byte);
    } else if action.kind == .OSC_DISPATCH {
        if action.osc.count < 2 return;
        if action.osc[0] != "0" && action.osc[0] != "2" return;

        // Preserve semicolons in window titles; fields borrow one contiguous OSC buffer.
        first := action.osc[1];
        last := action.osc[action.osc.count - 1];

        free(g.title);
        g.title = copy_string(string.{ last.data + last.count - first.data, first.data });
    } else if action.kind == .ESC_DISPATCH {
        if action.intermediates.count != 0 return;

        if byte == #char "7" {
            g.save_x = g.x;
            g.save_y = g.y;
        } else if byte == #char "8" {
            g.x = min(g.save_x, g.cols - 1);
            g.y = min(g.save_y, g.rows - 1);
        } else if byte == #char "D" {
            linefeed(g);
        } else if byte == #char "E" {
            g.x = 0;
            linefeed(g);
        } else if byte == #char "M" {
            if g.y == g.top scroll_down(g);
            else g.y = max(0, g.y - 1);
        } else if byte == #char "c" {
            g.attr = .{};
            g.x = 0;
            g.y = 0;
            g.top = 0;
            g.bottom = g.rows - 1;
            for y: 0..g.rows - 1 clear_row(g, y);

        }
    }
}

csi :: (g: *Grid, final: u8) {
    n := max(1, g.params[0]);

    if final != #char "m" && !g.private_mode g.wrap_pending = false;

    if g.private_mode {
        if final == #char "h" || final == #char "l" {
            enabled := final == #char "h";

            for i: 0..g.param_count - 1 {
                p := g.params[i];
                if p == 1 g.application_cursor = enabled;
                if p == 2004 g.bracketed = enabled;
                if p == 25 g.cursor_visible = enabled;

                if p == 1049 && enabled != g.alternate {
                    swap(*g.cells, *g.saved);
                    g.alternate = enabled;

                    if enabled {
                        g.primary_x = g.x;
                        g.primary_y = g.y;
                        g.x = 0;
                        g.y = 0;
                        for y: 0..g.rows - 1 clear_row(g, y);
                    } else {
                        g.x = min(g.primary_x, g.cols - 1);
                        g.y = min(g.primary_y, g.rows - 1);
                    }

                    g.top = 0;
                    g.bottom = g.rows - 1;
                }
            }
        }

        return;
    }

    if final == {
        case #char "A"; g.y = max(0, g.y - n);
        case #char "B"; g.y = min(g.rows - 1, g.y + n);
        case #char "C"; g.x = min(g.cols - 1, g.x + n);
        case #char "D"; g.x = max(0, g.x - n);
        case #char "E";
            g.y = min(g.rows - 1, g.y + n);
            g.x = 0;
        case #char "F";
            g.y = max(0, g.y - n);
            g.x = 0;
        case #char "G"; g.x = min(g.cols - 1, n - 1);
        case #char "d"; g.y = min(g.rows - 1, n - 1);
        case #char "H"; #through;
        case #char "f";
            g.y = min(g.rows - 1, n - 1);
            g.x = min(g.cols - 1, max(1, g.params[1]) - 1);
        case #char "s";
            g.save_x = g.x;
            g.save_y = g.y;
        case #char "u";
            g.x = min(g.save_x, g.cols - 1);
            g.y = min(g.save_y, g.rows - 1);
        case #char "J";
            if g.params[0] == 3 {
                g.history.count = 0;
                g.history_head = 0;
                return;
            }

            at := g.y * g.cols + g.x;
            start := ifx g.params[0] == 0 then at else 0;
            end := ifx g.params[0] == 1 then at else g.cells.count - 1;

            for i: start..end erase_cell(g, i % g.cols, i / g.cols);

        case #char "K";
            start := ifx g.params[0] == 0 then g.x else 0;
            end := ifx g.params[0] == 1 then g.x else g.cols - 1;

            for x: start..end erase_cell(g, x, g.y);
        case #char "X";
            for x: g.x..min(g.cols - 1, g.x + n - 1) erase_cell(g, x, g.y);
        case #char "@"; #through;
        case #char "P";
            n = min(n, g.cols - g.x);
            base := g.y * g.cols;

            if final == #char "@" {
                for < x: g.x + n..g.cols - 1 g.cells[base + x] = g.cells[base + x - n];
                for x: g.x..g.x + n - 1 g.cells[base + x] = .{ attr = g.attr };
            } else {
                for x: g.x..g.cols - n - 1 g.cells[base + x] = g.cells[base + x + n];
                for x: g.cols - n..g.cols - 1 g.cells[base + x] = .{ attr = g.attr };
            }

            repair_row(g, g.y);
        case #char "L"; #through;
        case #char "M";
            if g.y >= g.top && g.y <= g.bottom {
                top := g.top;

                g.top = g.y;
                if final == #char "L" scroll_down(g, n);
                else scroll_up(g, n, false);

                g.top = top;
            }
        case #char "S"; scroll_up(g, n);
        case #char "T"; scroll_down(g, n);
        case #char "r";
            top := n - 1;
            bottom := ifx g.params[1] == 0 then g.rows - 1 else g.params[1] - 1;

            if top < bottom && bottom < g.rows {
                g.top = top;
                g.bottom = bottom;
                g.x = 0;
                g.y = 0;
            }
        case #char "m"; sgr(g);
        case #char "n";
            if g.params[0] == 5 reply(g, "\x1b[0n");
            if g.params[0] == 6 reply(g, tprint("\x1b[%;%R", g.y + 1, g.x + 1));
        case #char "c"; reply(g, "\x1b[?1;2c");
        case;
    }
}

repair_row :: (g: *Grid, y: int) {
    for x: 0..g.cols - 1 {
        at := y * g.cols + x;
        c := g.cells[at];
        if c.width == 0 && (x == 0 || g.cells[at - 1].width != 2) g.cells[at] = .{ attr = g.attr };
        if c.width == 2 && (x == g.cols - 1 || g.cells[at + 1].width != 0) g.cells[at] = .{ attr = g.attr };
    }
}

palette :: (index: int) -> u32 {
    basic := u32.[0x161a22, 0xbf616a, 0xa3be8c, 0xebcb8b, 0x81a1c1, 0xb48ead, 0x88c0d0, 0xd8dee9, 0x4c566a, 0xff6b6b, 0xb5e48c, 0xffdf80, 0x99c1f1, 0xd0a5db, 0xa4e5e0, 0xffffff];
    i := clamp(index, 0, 255);

    if i < 16 return basic[i];

    if i >= 232 {
        v := cast(u32) (8 + (i - 232) * 10);
        return (v << 16) | (v << 8) | v;
    }

    i -= 16;
    r := i / 36;
    b := i % 6;
    green := (i / 6) % 6;

    return (cube(r) << 16) | (cube(green) << 8) | cube(b);
}

cube :: (n: int) -> u32 {
    return cast(u32) (ifx n == 0 then 0 else 55 + n * 40);
}

sgr :: (g: *Grid) {
    i := 0;
    while i < g.param_count {
        p := g.params[i];
        if p == 0 g.attr = .{};
        if p == 1 g.attr.bold = true;
        if p == 3 g.attr.italic = true;
        if p == 4 g.attr.underline = true;
        if p == 7 g.attr.inverse = true;
        if p == 22 g.attr.bold = false;
        if p == 23 g.attr.italic = false;
        if p == 24 g.attr.underline = false;
        if p == 27 g.attr.inverse = false;
        if p >= 30 && p <= 37 g.attr.fg = palette(p - 30);
        if p >= 40 && p <= 47 g.attr.bg = palette(p - 40);
        if p >= 90 && p <= 97 g.attr.fg = palette(p - 90 + 8);
        if p >= 100 && p <= 107 g.attr.bg = palette(p - 100 + 8);
        if p == 39 g.attr.fg = Attr.{}.fg;
        if p == 49 g.attr.bg = Attr.{}.bg;

        if (p == 38 || p == 48) && i + 2 < g.param_count {
            color: u32;
            valid := true;

            if g.params[i + 1] == 5 {
                color = palette(g.params[i + 2]);
                i += 2;
            } else if g.params[i + 1] == 2 && i + 4 < g.param_count {
                color = (cast(u32) clamp(g.params[i + 2], 0, 255) << 16) | (cast(u32) clamp(g.params[i + 3], 0, 255) << 8) | cast(u32) clamp(g.params[i + 4], 0, 255);
                i += 4;
            } else valid = false;

            if valid {
                if p == 38 g.attr.fg = color;
                else g.attr.bg = color;
            }
        }

        i += 1;
    }
}

reply :: (g: *Grid, text: string) {
    // A malicious query stream cannot grow the reply queue without bound.
    if g.replies.count + text.count > 8192 return;
    for text array_add(*g.replies, it);
}
