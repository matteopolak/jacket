Terminal :: struct {
    grid: Grid;
    session: Pty.Session;
    pending: [..] u8;
    sent: int;
    selection_start: int = -1;
    selection_end: int = -1;
    selecting: bool;
    view_back: int;
    eof: bool;
    headless: bool;
    cell_w: float = 9;
    cell_h: float = 20;
}

close_terminal :: (app: *Terminal) {
    Pty.close_session(*app.session);
    free_grid(*app.grid);
    array_free(app.pending);
}

queue_input :: (app: *Terminal, text: string) {
    for text array_add(*app.pending, it);
    app.view_back = 0;
}

paste :: (app: *Terminal, text: string) {
    if app.grid.bracketed queue_input(app, "\x1b[200~");
    queue_input(app, text);
    if app.grid.bracketed queue_input(app, "\x1b[201~");
}

pump :: (app: *Terminal) {
    if app.session.fd < 0 return;

    if app.sent < app.pending.count {
        text := string.{ app.pending.count - app.sent, app.pending.data + app.sent };
        app.sent += Pty.write_session(*app.session, text);
    }

    if app.sent == app.pending.count {
        app.pending.count = 0;
        app.sent = 0;
    }

    buffer: [8192] u8;

    // Bound work per frame so a noisy child cannot starve input or drawing.
    for 0..15 {
        n := Pty.read_session(*app.session, buffer);
        if n < 0 {
            app.eof = true;
            break;
        }

        if n == 0 break;

        feed(*app.grid, .{ n, buffer.data });
    }

    if app.grid.replies.count {
        queue_input(app, .{ app.grid.replies.count, app.grid.replies.data });
        app.grid.replies.count = 0;
    }

    if app.grid.alternate app.view_back = 0;
    else app.view_back = min(app.view_back, app.grid.history.count / app.grid.cols);

    Pty.poll_exit(*app.session);
}

build_terminal :: (app: *Terminal) {
    ui := current_ui();

    // UI may rebuild after input; draining nonblocking I/O is safe on either build.
    if !app.headless pump(app);

    ui.wake_time = ui.now + 0.016;
    if app.eof && app.session.exited && !app.headless ui.quit = true;

    for column(.{ grow = 1, background = rgb(22, 26, 34) }) {
        label(ifx app.grid.title.count then app.grid.title else "Jacket terminal", .{ padding = 4 });
        node := widget(.LEAF, .CUSTOM, .{ grow = 1, clip = true }, #location());
        node.label = "terminal";
        node.pointer = true;
        node.focusable = true;
        node.paint = paint_terminal;
        node.paint_data = app;
        if ui.focus == 0 focus_node(ui, node);

        app.cell_w = max(1.0, glyph_advance(*ui.text, #char "M", .{ font = .MONO, size = 15 }));
        cols := max(1, cast(int) (node.rect.w / app.cell_w));
        rows := max(1, cast(int) (node.rect.h / app.cell_h));

        if node.rect.w > 0 && node.rect.h > 0 && !app.headless && (cols != app.grid.cols || rows != app.grid.rows) {
            if Pty.resize_session(*app.session, cols, rows) {
                resize_grid(*app.grid, cols, rows);
                app.selection_start = -1;
                app.selection_end = -1;
                app.view_back = 0;
            }
        }

        terminal_input(app, ui, node);
    }
}

terminal_input :: (app: *Terminal, ui: *UI_State, node: *Node) {
    if ui.hover == node.id {
        app.view_back = clamp(app.view_back - cast(int) (ui.input.scroll_dy / app.cell_h), 0, ifx app.grid.alternate then 0 else app.grid.history.count / app.grid.cols);
        ui.input.scroll_dy = 0;
    }

    if ui.pressed == node.id && ui.input.mouse_down {
        x := clamp(cast(int) ((ui.input.mouse_x - node.rect.x) / app.cell_w), 0, app.grid.cols - 1);
        y := clamp(cast(int) ((ui.input.mouse_y - node.rect.y) / app.cell_h), 0, app.grid.rows - 1);
        at := y * app.grid.cols + x;

        if !app.selecting {
            app.selection_start = at;
            app.selecting = true;
        }

        app.selection_end = at;
    } else app.selecting = false;

    if ui.focus != node.id return;

    for *event: ui.input.keys {
        if event.kind == .TEXT {
            text := U.character_utf32_to_utf8(event.character);
            queue_input(app, text);
            free(text);
        } else if event.kind == .KEY {
            shortcut := primary(event.mods);

            #if OS != .MACOS shortcut = shortcut && event.mods.shift;

            if shortcut && event.key == cast(Key_Code) #char "C" {
                text := selected_text(app);
                defer free(text);
                if ui.hooks.clipboard_set ui.hooks.clipboard_set(text);
            } else if shortcut && event.key == cast(Key_Code) #char "V" {
                if ui.hooks.clipboard_get paste(app, ui.hooks.clipboard_get());
            } else if event.mods.ctrl && cast(int) event.key >= 64 && cast(int) event.key <= 95 {
                byte := cast(u8) (cast(int) event.key & 31);
                queue_input(app, .{ 1, *byte });
            } else {
                text := special_key(event.key);
                if app.grid.application_cursor && text.count == 3 && text[1] == #char "[" {
                    // Arrow keys and Home/End use SS3 in application cursor mode.
                    bytes: [3] u8;

                    memcpy(bytes.data, text.data, 3);
                    bytes[1] = #char "O";
                    queue_input(app, .{ 3, bytes.data });
                    event.kind = .COMMAND;
                    continue;
                }

                if text.count queue_input(app, text);
            }
        }

        event.kind = .COMMAND; // consume: a second UI build must not send the same key twice
    }
}

special_key :: (key: Key_Code) -> string {
    if key == {
        case .ENTER; return "\r";
        case .BACKSPACE; return "\x7f";
        case .TAB; return "\t";
        case .ESCAPE; return "\x1b";
        case .ARROW_UP; return "\x1b[A";
        case .ARROW_DOWN; return "\x1b[B";
        case .ARROW_RIGHT; return "\x1b[C";
        case .ARROW_LEFT; return "\x1b[D";
        case .HOME; return "\x1b[H";
        case .END; return "\x1b[F";
        case .DELETE; return "\x1b[3~";
        case .INSERT; return "\x1b[2~";
        case .PAGE_UP; return "\x1b[5~";
        case .PAGE_DOWN; return "\x1b[6~";
        case; return "";
    }
}

visible_cell :: (app: *Terminal, at: int) -> Cell {
    g := *app.grid;
    index := g.history.count - app.view_back * g.cols + at;
    if index < g.history.count return history_cell(g, index);
    return g.cells[index - g.history.count];
}

selected_text :: (app: *Terminal) -> string {
    b: String_Builder;
    if app.selection_start < 0 return builder_to_string(*b);

    low := min(app.selection_start, app.selection_end);
    high := max(app.selection_start, app.selection_end);

    for i: low..high {
        c := visible_cell(app, i);
        if c.width != 0 {
            buffer: [4] u8;
            text := string.{ 4, buffer.data };
            U.character_utf32_to_utf8(c.code, *text);
            append(*b, text);
        }

        if i % app.grid.cols == app.grid.cols - 1 && i != high append(*b, "\n");
    }

    return builder_to_string(*b);
}

color32 :: (value: u32) -> Color {
    return rgb(cast(int) ((value >> 16) & 255), cast(int) ((value >> 8) & 255), cast(int) (value & 255));
}

paint_terminal :: (ui: *UI_State, node: *Node) {
    app := cast(*Terminal) node.paint_data;

    push_clip(ui, node.rect);
    defer pop_clip(ui);

    // Paint every background first so trailing wide cells cannot cover a glyph's right half.
    for y: 0..app.grid.rows - 1 {
        for x: 0..app.grid.cols - 1 {
            at := y * app.grid.cols + x;
            c := visible_cell(app, at);
            bg := color32(ifx c.attr.inverse then c.attr.fg else c.attr.bg);

            if app.selection_start >= 0 && at >= min(app.selection_start, app.selection_end) && at <= max(app.selection_start, app.selection_end) bg = rgb(65, 85, 120);

            draw_rect(ui, .{ node.rect.x + x * app.cell_w, node.rect.y + y * app.cell_h, app.cell_w, app.cell_h }, bg);
        }
    }

    for y: 0..app.grid.rows - 1 {
        for x: 0..app.grid.cols - 1 {
            at := y * app.grid.cols + x;
            c := visible_cell(app, at);
            rect := Rect.{ node.rect.x + x * app.cell_w, node.rect.y + y * app.cell_h, app.cell_w, app.cell_h };
            fg := color32(ifx c.attr.inverse then c.attr.bg else c.attr.fg);

            if c.width == 0 continue;

            buffer: [4] u8;
            text := string.{ 4, buffer.data };

            U.character_utf32_to_utf8(c.code, *text);
            style := Text_Style.{ font = .MONO, size = 15, color = fg };

            draw_cell_text(ui, rect.x, rect.y, text, style, c.attr.italic);
            // Synthetic bold shares the same cached glyph and preserves terminal advances.
            if c.attr.bold draw_cell_text(ui, rect.x + 0.6, rect.y, text, style, c.attr.italic);
            if c.attr.underline draw_rect(ui, .{ rect.x, rect.y + 17, app.cell_w * c.width, 1 }, fg);
        }
    }

    if app.grid.cursor_visible && app.view_back == 0 {
        draw_rect(ui, .{ node.rect.x + app.grid.x * app.cell_w, node.rect.y + app.grid.y * app.cell_h + 18, app.cell_w, 2 }, rgb(220, 220, 220));
    }
}

// Shear cached glyphs by emitting horizontal strips; no extra font faces or atlas entries.
draw_cell_text :: (ui: *UI_State, x: float, y: float, text: string, style: Text_Style, italic: bool) {
    start := ui.draw.cmds.count;

    draw_text(ui, x, y, text, style);
    if !italic || ui.draw.cmds.count == start return;

    glyph := ui.draw.cmds[ui.draw.cmds.count - 1];

    ui.draw.cmds.count -= 1;
    rows := max(1, cast(int) (glyph.rect.h * ui.text.scale));
    for row: 0..rows - 1 {
        strip := glyph;
        fraction := cast(float) row / rows;
        next := cast(float) (row + 1) / rows;

        strip.rect.y = glyph.rect.y + fraction * glyph.rect.h;
        strip.rect.h = glyph.rect.h / rows;
        strip.rect.x += (1 - fraction) * glyph.rect.h * 0.2;
        strip.uv.y = glyph.uv.y + fraction * (glyph.uv.w - glyph.uv.y);
        strip.uv.w = glyph.uv.y + next * (glyph.uv.w - glyph.uv.y);
        array_add(*ui.draw.cmds, strip);
    }
}
