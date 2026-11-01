check :: (ok: bool, message: string) {
    if !ok {
        print("term smoke failed: %\n", message);
        exit(1);
    }
}

smoke :: (frame_path: string) {
    #if OS == .MACOS || OS == .LINUX {
        app: Terminal;
        app.grid = make_grid(40, 8);
        app.headless = true;
        ok: bool;
        app.session, ok = Pty.open_session("/bin/sh", "stty size; printf '\\033[2J\\033[Hfirst\\r\\n\\033[31mred\\033[0m\\r\\n'; printf '\\033[?1049hhidden\\033[?1049l'; printf '\\033]2;PTY smoke\\007'; printf '\\033[4;1Hdone'", 40, 8);
        check(ok, "real PTY opens");
        defer close_terminal(*app);

        for 0..999 {
            pump(*app);
            if app.eof && app.session.exited break;
            sleep_milliseconds(2);
        }

        check(app.eof && app.session.exited && app.session.status == 0, "child exits and output drains");
        first := row_text(*app.grid, 0);
        last := row_text(*app.grid, 3);

        defer free(first);
        defer free(last);
        check(begins_with(first, "first") && begins_with(last, "done"), "PTY final grid snapshot");
        check(app.grid.cells[40].attr.fg == palette(1), "SGR survives PTY reads");
        check(app.grid.title == "PTY smoke" && !app.grid.alternate, "OSC and alternate screen");
        smoke_input_and_resize();
        ui := headless(.{ width = 500, height = 240, system_fonts = true });

        defer free_ui(*ui);
        frame(*ui, *app, build_terminal);
        frame(*ui, *app, build_terminal);
        check(ui.draw.cmds.count > 320, "offscreen display list includes backgrounds and glyphs");

        glyphs := 0;
        for ui.draw.cmds if it.kind == .GLYPH glyphs += 1;

        check(glyphs > 0, "FreeType rasterized visible glyphs");

        if frame_path.count {
            renderer := create_offscreen_renderer(500, 240);
            if !renderer {
                check(environment_variable("REQUIRE_GPU") != "1", "GPU offscreen PNG");
                print("skip GPU offscreen PNG: no graphics adapter available\n");
            } else {
                image := render_to_image(renderer, *ui, rgb(22, 26, 34));

                defer array_free(image.pixels);
                defer destroy_renderer(renderer);
                check(image.pixels.count == 500 * 240 * 4, "GPU readback size");
                at := (170 * 500 + 300) * 4;

                check(image.pixels[at] == 22 && image.pixels[at + 1] == 26 && image.pixels[at + 2] == 34, "GPU blank cell background pixel");
                check(write_screenshot(*ui, frame_path), "write offscreen PNG");
            }
        }

        app.selection_start = 0;
        app.selection_end = 4;
        selected := selected_text(*app);

        defer free(selected);
        check(selected == "first", "selection copies grid text");
        app.grid.bracketed = true;
        paste(*app, "a\nb");
        check(string.{ app.pending.count, app.pending.data } == "\x1b[200~a\nb\x1b[201~", "bracketed paste delimiters");
        print("term smoke: PTY, input, resize, grid, attributes, alternate, title, FreeType frame ok\n");
    } else print("skip term smoke: PTY requires macOS or Linux\n");
}

smoke_input_and_resize :: () {
    #if OS == .MACOS || OS == .LINUX {
        app: Terminal;
        app.headless = true;
        app.grid = make_grid(40, 8);
        ok: bool;
        app.session, ok = Pty.open_session("/bin/sh", "stty -echo; printf ready; read line; printf '\\033[2J\\033[H'; stty size; printf '%s' \"$line\"", 40, 8);
        check(ok, "input PTY opens");
        defer close_terminal(*app);
        ready := false;

        for 0..999 {
            pump(*app);

            if app.grid.cells[4].code == #char "y" {
                ready = true;
                break;
            }

            sleep_milliseconds(2);
        }

        check(ready, "child is ready for input");
        check(Pty.resize_session(*app.session, 37, 9), "resize PTY");
        queue_input(*app, "hello world\n");

        for 0..999 {
            pump(*app);
            if app.eof && app.session.exited break;
            sleep_milliseconds(2);
        }

        check(app.session.exited && app.session.status == 0, "input child exits");
        size := row_text(*app.grid, 0);
        input := row_text(*app.grid, 1);

        defer free(size);
        defer free(input);
        check(begins_with(size, "9 37"), "child sees resized dimensions");
        check(begins_with(input, "hello world"), "queued input reaches the child once");
    }
}
