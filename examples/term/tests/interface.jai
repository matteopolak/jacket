#import "Basic";
#import "Extensions/UI";
#import "Extensions/Testing";
#import "Input";
#import,dir "../vt";
Pty :: #import "Extensions/Pty";
#load "../terminal.jai";

copied: string;

record_copy :: (text: string) {
    free(copied);
    copied = copy_string(text);
}

mouse_selection_clipboard_and_keyboard :: () {
    app: Terminal;
    app.headless = true;
    app.grid = make_grid(12, 3);
    defer close_terminal(*app);
    feed(*app.grid, "hello world");
    ui := headless(.{ width = 150, height = 90 });

    defer free_ui(*ui);
    ui.hooks.clipboard_set = record_copy;
    ui.hooks.clipboard_get = () -> string {
        return "paste\nline";
    };

    defer {
        free(copied);
        copied = "";
    }

    frame(*ui, *app, build_terminal);
    frame(*ui, *app, build_terminal);
    node := find_label(*ui, "terminal");

    require(node != null);
    send_mouse_move(*ui, node.rect.x + 1, node.rect.y + 2);
    send_mouse_down(*ui);
    frame(*ui, *app, build_terminal);
    send_mouse_move(*ui, node.rect.x + app.cell_w * 4 + 1, node.rect.y + 2);
    frame(*ui, *app, build_terminal);
    send_mouse_up(*ui);
    frame(*ui, *app, build_terminal);
    mods: Modifiers;
    #if OS == .MACOS mods.super = true;
    else {
        mods.ctrl = true;
        mods.shift = true;
    }

    send_key(*ui, cast(Key_Code) #char "C", mods);
    frame(*ui, *app, build_terminal);
    expect_equal(copied, "hello");
    app.grid.bracketed = true;
    send_key(*ui, cast(Key_Code) #char "V", mods);
    frame(*ui, *app, build_terminal);
    expect_equal(string.{ app.pending.count, app.pending.data }, "\x1b[200~paste\nline\x1b[201~");
    app.pending.count = 0;
    send_key(*ui, cast(Key_Code) #char "C", .{ ctrl = true });
    send_key(*ui, .ARROW_UP);
    send_key(*ui, .ENTER);
    frame(*ui, *app, build_terminal);
    expect_equal(string.{ app.pending.count, app.pending.data }, "\x03\x1b[A\r");
} @test

styled_cells_render_from_the_glyph_cache :: () {
    app: Terminal;
    app.headless = true;
    app.grid = make_grid(4, 2);
    defer close_terminal(*app);
    feed(*app.grid, "\x1b[1;3;4;7mA");
    ui := headless(.{ width = 120, height = 90, system_fonts = true });

    defer free_ui(*ui);
    frame(*ui, *app, build_terminal);
    glyphs := 0;
    for ui.draw.cmds if it.kind == .GLYPH glyphs += 1;

    expect(glyphs > 2, "synthetic italic and bold emit glyph strips");
    node := find_label(*ui, "terminal");

    require(node != null);
    // Cell backgrounds must precede every terminal glyph, including a wide glyph's trailing cell.
    terminal_glyph_seen := false;

    for ui.draw.cmds {
        if it.rect.y < node.rect.y continue;
        if it.kind == .GLYPH terminal_glyph_seen = true;
        if terminal_glyph_seen && it.kind == .RECT && it.rect.h == app.cell_h expect(false, "a cell background would erase an earlier glyph");
    }

    selected := selected_text(*app);

    defer free(selected);
    expect_equal(selected, "");
} @test
