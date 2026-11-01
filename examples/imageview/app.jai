#import "Basic";
#import "Input";
#import "String";
#import "File";
#import "File_Utilities";
#import "Extensions/UI";
#import,dir "viewer";

Viewer :: struct {
    cache: Cache;
    picture: Picture;
    selected: s64 = -1;
    page: s64;
    grid: bool = true;
    view: View;
    fitting: bool = true;
    open_path: Text_Edit;
    export_path: Text_Edit;
    status: string;
    dragging: bool;
    last_x: float;
    last_y: float;
    gif_frame: s64;
    gif_next: float64;
    paused: bool;
}

set_status :: (app: *Viewer, message: string) {
    free(app.status);
    app.status = copy_string(message);
}

close_viewer :: (app: *Viewer) {
    free_cache(*app.cache);
    release_picture(*app.picture);
    free_text_edit(*app.open_path);
    free_text_edit(*app.export_path);
    free(app.status);
}

open_target :: (app: *Viewer, target: string) -> bool {
    if !file_exists(target) && !is_directory(target) {
        set_status(app, "File or folder does not exist");
        return false;
    }

    next: Cache;
    next.cap = app.cache.cap;
    if !open_folder(*next, target) {
        free_cache(*next);
        set_status(app, "Cannot read folder");
        return false;
    }

    free_cache(*app.cache);
    release_picture(*app.picture);
    app.cache = next;
    app.selected = -1;
    app.page = 0;
    app.grid = true;

    set_status(app, tprint("% images", app.cache.entries.count));

    if !is_directory(target) {
        chosen: s64 = -1;

        for app.cache.entries if path_filename(it.path) == path_filename(target) {
            chosen = it_index;
            break;
        }

        if chosen < 0 {
            array_add(*app.cache.entries, .{ path = copy_string(target) });
            chosen = app.cache.entries.count - 1;
        }

        select_image(app, chosen);
    }

    return true;
}

select_image :: (app: *Viewer, index: s64) {
    if index < 0 || index >= app.cache.entries.count return;

    release_picture(*app.picture);
    app.selected = index;
    app.picture = decode_picture(app.cache.entries[index].path, true);
    app.grid = false;
    app.fitting = true;
    app.gif_frame = 0;
    app.gif_next = 0;
    app.paused = false;
    set_status(app, app.picture.error);
}

rotate_image :: (app: *Viewer) {
    if !app.picture.image.pixels.count return;

    turned := Image.rotate_90(app.picture.image);

    Image.deinit(*app.picture.image);
    app.picture.image = turned;
    // Rotation turns this into a still so animation cannot restore an unrotated frame.
    array_free(app.picture.frames);
    array_free(app.picture.delays);
    app.fitting = true;
}

animate :: (app: *Viewer, ui: *UI_State) {
    if app.paused || app.picture.delays.count <= 1 return;
    if app.gif_next == 0 app.gif_next = ui.now + max(10, app.picture.delays[0]) / 1000.0;

    if ui.now >= app.gif_next {
        app.gif_frame = (app.gif_frame + 1) % app.picture.delays.count;
        bytes := app.picture.image.pixels.count;
        memcpy(app.picture.image.pixels.data, app.picture.frames.data + app.gif_frame * bytes, bytes);
        app.gif_next = ui.now + max(10, app.picture.delays[app.gif_frame]) / 1000.0;
    }

    ui.wake_time = app.gif_next;
}

build :: (app: *Viewer) {
    ui := current_ui();
    if !app.grid animate(app, ui);

    for column(.{ grow = 1, gap = 6, padding = 8 }) {
        for row(.{ gap = 6, height = 32 }) {
            text_field(*app.open_path, .{ grow = 1 });
            if button("Open") open_target(app, contents(*app.open_path));

            if button("Grid") {
                app.grid = true;
                app.page = max(0, app.selected / 24);
            }

            if button("Previous") select_image(app, max(0, app.selected - 1));
            if button("Next") select_image(app, app.selected + 1);
            if button("Fit") app.fitting = true;

            if button("1:1") {
                app.fitting = false;
                app.view = .{};
            }

            if button("Rotate") rotate_image(app);
            if button("Pause GIF") app.paused = !app.paused;
        }

        for row(.{ gap = 6, height = 30 }) {
            text_field(*app.export_path, .{ grow = 1 });

            if button("Export as") {
                ok := export_picture(app.picture.image, contents(*app.export_path));
                set_status(app, ifx ok then "Exported" else "Export failed (choose .png or .jpg)");
            }

            if button("Page back") app.page = max(0, app.page - 1);
            if button("Page forward") app.page = min(max(0, (app.cache.entries.count - 1) / 24), app.page + 1);
        }

        if app.selected >= 0 {
            image := app.picture.image;
            label(tprint("% | % x % | % | % bits/channel -> RGBA8 | % bytes | % frames", path_filename(app.cache.entries[app.selected].path), image.width, image.height, app.picture.format, app.picture.bits, app.picture.file_bytes, max(1, app.picture.delays.count)));
        }

        label(app.status);
        if app.grid load_thumbnails(*app.cache, app.page * 24, 24);

        node := widget(.LEAF, .CUSTOM, .{ grow = 1, min_height = 100 }, #location());
        node.label = "Images";
        node.pointer = true;
        node.focusable = true;
        node.paint = paint_canvas;
        node.paint_data = app;
        canvas_input(app, ui, node);
    }
}

canvas_input :: (app: *Viewer, ui: *UI_State, node: *Node) {
    r := node.rect;
    hovered := ui.input.mouse_x >= r.x && ui.input.mouse_x < r.x + r.w && ui.input.mouse_y >= r.y && ui.input.mouse_y < r.y + r.h;

    if ui.input.clicked == node.id && app.grid {
        col := cast(s64) ((ui.input.mouse_x - r.x) / max(1.0, r.w / 6));
        row := cast(s64) ((ui.input.mouse_y - r.y) / max(1.0, r.h / 4));
        select_image(app, app.page * 24 + row * 6 + col);
    }

    if !app.grid && hovered && ui.input.scroll_dy != 0 {
        app.fitting = false;
        zoom_at(*app.view, ifx ui.input.scroll_dy < 0 then 1.2 else 1.0 / 1.2, ui.input.mouse_x - r.x, ui.input.mouse_y - r.y);
    }

    if ui.input.mouse_down && ui.pressed == node.id && !app.grid {
        if app.dragging {
            app.fitting = false;
            app.view.x += ui.input.mouse_x - app.last_x;
            app.view.y += ui.input.mouse_y - app.last_y;
        }

        app.dragging = true;
        app.last_x = ui.input.mouse_x;
        app.last_y = ui.input.mouse_y;
    } else app.dragging = false;

    if ui.focus != node.id return;

    for ui.input.keys {
        if it.key == .ARROW_RIGHT select_image(app, app.selected + 1);
        if it.key == .ARROW_LEFT select_image(app, app.selected - 1);
        if it.key == .ESCAPE app.grid = true;
        if it.key == cast(Key_Code) #char "R" rotate_image(app);
        if it.key == cast(Key_Code) #char "F" app.fitting = true;

        if it.key == cast(Key_Code) #char "1" {
            app.fitting = false;
            app.view = .{};
        }
    }
}

paint_canvas :: (ui: *UI_State, node: *Node) {
    app := cast(*Viewer) node.paint_data;
    r := node.rect;

    push_clip(ui, r);
    defer pop_clip(ui);
    draw_rect(ui, r, rgb(28, 30, 36));

    if app.grid {
        for i: 0..23 {
            index := app.page * 24 + i;
            if index >= app.cache.entries.count break;

            entry := *app.cache.entries[index];
            w := r.w / 6;
            h := r.h / 4;
            x := r.x + (i % 6) * w;
            tile_row := i / 6;
            y := r.y + tile_row * h;
            thumb := entry.thumb;

            if thumb.pixels.count {
                ratio := min(1.0, min((w - 12) / thumb.width, (h - 24) / thumb.height));
                draw_image(ui, .{ x + 6, y + 4, thumb.width * ratio, thumb.height * ratio }, thumb);
            }

            draw_text(ui, x + 6, y + h - 18, path_filename(entry.path), .{ size = 11, color = rgb(220, 220, 230) });
        }
    } else if app.picture.image.pixels.count {
        image := app.picture.image;
        if app.fitting app.view = fit_view(image.width, image.height, r.w, r.h);

        // Checkerboard makes alpha visible without flattening the source or exports.
        for y: 0..cast(int) (r.h / 16) for x : 0..cast(int) (r.w / 16) {
            c: u8 = ifx (x + y) % 2 == 0 then 60 else 85;
            draw_rect(ui, .{ r.x + x * 16, r.y + y * 16, 16, 16 }, rgb(c, c, c));
        }

        draw_image(ui, .{ r.x + app.view.x, r.y + app.view.y, image.width * app.view.zoom, image.height * app.view.zoom }, image);
    }
}
