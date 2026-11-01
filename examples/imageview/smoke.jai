#load "fixtures.jai";

smoke :: (root: string) {
    generate_fixtures(root);
    app: Viewer;

    defer close_viewer(*app);
    assert(open_target(*app, root));
    assert(app.cache.entries.count == 9);
    ui := headless(.{ width = 1000, height = 700 });

    defer free_ui(*ui);
    frame(*ui, *app, build);
    frame(*ui, *app, build);
    assert(app.cache.used > 0 && app.cache.used <= app.cache.cap);
    canvas := find_label(*ui, "Images");

    assert(canvas != null);
    // Every supported generated format decoded, a malformed file is marked as failed.
    good := 0;
    for app.cache.entries if it.thumb.pixels.count > 0 good += 1;

    assert(good == 8);
    animated := decode_picture(tprint("%/animated.gif", root), true);

    assert(animated.delays.count == 2 && animated.frames.count == 16);
    release_picture(*animated);
    assert(open_target(*app, tprint("%/animated.gif", root)));
    ui.now = 0;
    frame(*ui, *app, build);
    ui.now = 0.11;
    frame(*ui, *app, build);
    assert(app.gif_frame == 1 && app.picture.image.pixels[2] == 255);
    app.paused = true;
    ui.now = 1;
    frame(*ui, *app, build);
    assert(app.gif_frame == 1);
    app.grid = true;
    frame(*ui, *app, build);
    // Grid click uses the same input path as the window.
    click_at(*ui, canvas.rect.x + 12, canvas.rect.y + 12);
    frame(*ui, *app, build);
    assert(!app.grid && app.selected == 0);
    assert(open_target(*app, tprint("%/gradient.png", root)));
    frame(*ui, *app, build);
    assert(app.picture.image.width == 64);
    app.grid = true;
    frame(*ui, *app, build);
    renderer := create_offscreen_renderer(1000, 700);
    if !renderer {
        print("skip: GPU grid/zoom PNG readback and pixel checks (no adapter)\n");
    } else {
        defer destroy_renderer(renderer);
        grid := render_to_image(renderer, *ui, rgb(0, 0, 0));

        defer array_free(grid.pixels);
        assert(grid.pixels.count == 1000 * 700 * 4);
        // Locate gradient.png's tile and test its opaque red top-left patch.
        tile: s64 = -1;
        for app.cache.entries if path_filename(it.path) == "gradient.png" tile = it_index;

        canvas = find_label(*ui, "Images");
        px := cast(s64) (canvas.rect.x + (tile % 6) * canvas.rect.w / 6 + 8);
        tile_row := tile / 6;
        py := cast(s64) (canvas.rect.y + tile_row * canvas.rect.h / 4 + 6);
        at := (py * 1000 + px) * 4;

        assert(grid.pixels[at] > 240 && grid.pixels[at + 1] < 10);
        save_render(grid, tprint("%/grid.png", root));
        app.grid = false;
        app.fitting = false;
        app.view = .{ zoom = 4, x = 0, y = 0 };
        frame(*ui, *app, build);
        zoom := render_to_image(renderer, *ui, rgb(0, 0, 0));

        defer array_free(zoom.pixels);
        canvas = find_label(*ui, "Images");
        at = (cast(s64) (canvas.rect.y + 8) * 1000 + cast(s64) (canvas.rect.x + 8)) * 4;
        assert(zoom.pixels[at] > 240 && zoom.pixels[at + 1] < 10);
        save_render(zoom, tprint("%/zoom.png", root));
    }

    app.grid = false;
    frame(*ui, *app, build);
    canvas = find_label(*ui, "Images");
    old := app.view.zoom;

    send_mouse_move(*ui, canvas.rect.x + 40, canvas.rect.y + 40);
    send_scroll(*ui, 0, -1);
    frame(*ui, *app, build);
    assert(app.view.zoom > old);
    before_x := app.view.x;

    send_mouse_down(*ui);
    frame(*ui, *app, build);
    send_mouse_move(*ui, canvas.rect.x + 60, canvas.rect.y + 40);
    frame(*ui, *app, build);
    assert(app.view.x == before_x + 20);
    send_mouse_up(*ui);
    frame(*ui, *app, build);
    click(*ui, find_label(*ui, "Rotate"));
    frame(*ui, *app, build);
    assert(app.picture.image.width == 32 && app.picture.image.height == 64);
    assert(export_picture(app.picture.image, tprint("%/export.png", root)));
    assert(export_picture(app.picture.image, tprint("%/export.jpg", root)));

    for string.["export.png", "export.jpg"] {
        decoded := decode_picture(tprint("%/%", root, it));
        assert(decoded.image.width == 32 && decoded.image.height == 64);
        release_picture(*decoded);
    }

    source, read_ok := read_entire_file(tprint("%/gradient.png", root));

    assert(read_ok);
    defer free(source);
    assert(write_entire_file(tprint("%/image.asset", root), source));
    assert(open_target(*app, tprint("%/image.asset", root)));
    assert(app.picture.image.width == 64);
    assert(open_target(*app, tprint("%/broken.png", root)));
    assert(app.picture.error.count > 0 && app.status == app.picture.error);
    print("imageview smoke: decode, workers, grid click, animation, wheel, pan, rotate, export passed\n");
}

save_render :: (render: Rendered_Image, path: string) {
    image := Image.Image.{ cast(s32) render.width, cast(s32) render.height, .RGBA8, render.pixels };
    png := Image.encode_png(image);
    defer array_free(png);
    assert(write_entire_file(path, string.{ png.count, png.data }));
}
