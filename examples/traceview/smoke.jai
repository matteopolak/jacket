// The smoke uses the real UI declaration, paints both custom widgets, and clicks the controls.
smoke :: (app: *App, output: string) -> bool {
    ui := headless(.{ width = 1100, height = 800 });

    defer free_ui(*ui);
    frame(*ui, app, build);
    timeline_node := find_label(*ui, "Frame timeline");
    if !timeline_node || !find_label(*ui, "Flame view") return false;

    click_at(*ui, timeline_node.rect.x + timeline_node.rect.w - 1, timeline_node.rect.y + 30);
    frame(*ui, app, build);
    if app.selected != app.trace.frames.count - 1 return false;

    click(*ui, find_label(*ui, "Calls"));
    frame(*ui, app, build);
    if app.column != .CALLS return false;

    click(*ui, find_label(*ui, "Use selected as baseline"));
    frame(*ui, app, build);
    if app.baseline != app.selected return false;

    app.baseline = 0;
    set_contents(*app.search, "work");
    frame(*ui, app, build);
    stats := Model.statistics(*app.trace, app.selected, "work");

    defer array_free(stats);
    expected: s64;
    for app.trace.frames[app.selected].spans if contains(app.trace.names[it.zone], "work") expected += 1;
    if expected > 0 && stats.count == 0 return false;
    for stats if !contains(app.trace.names[it.zone], "work") return false;

    set_contents(*app.search, "");
    frame(*ui, app, build);
    renderer := create_offscreen_renderer(1100, 800);
    if renderer {
        defer destroy_renderer(renderer);
        image := render_to_image(renderer, *ui, ui.theme.background);

        defer array_free(image.pixels);
        if image.pixels.count == 0 return false;

        timeline_rect := find_label(*ui, "Frame timeline").rect;
        bar_x := timeline_rect.x + timeline_rect.w * (cast(float) app.selected + 0.5) / cast(float) app.trace.frames.count;

        if !pixel_matches(image, bar_x, timeline_rect.y + timeline_rect.h - 1, .[76, 181, 245]) return false;

        flame_rect := find_label(*ui, "Flame view").rect;
        boxes := Model.flame_layout(*app.trace.frames[app.selected]);

        defer array_free(boxes);

        if boxes.count > 0 && boxes[0].width * cast(float64) flame_rect.w > 4 {
            span := app.trace.frames[app.selected].spans[0];
            x := flame_rect.x + cast(float) (boxes[0].x + boxes[0].width / 2) * flame_rect.w;
            expected_color := u8.[cast(u8) (100 + span.zone * 37 % 130), cast(u8) (80 + span.zone * 19 % 110), 65];
            if !pixel_matches(image, x, flame_rect.y + 23, expected_color) return false;
        }

        picture := Images.Image.{ cast(s32) image.width, cast(s32) image.height, .RGBA8, image.pixels };
        png := Images.encode_png(picture);

        defer array_free(png);
        if !write_entire_file(output, png.data, png.count) return false;

        print("offscreen PNG ok\n");
    } else {
        print("skip offscreen PNG: no WebGPU adapter\n");
    }

    spans: s64;
    for app.trace.frames spans += it.spans.count;

    print("traceview frames=% zones=% spans=% selected=% baseline=%\n", app.trace.frames.count, app.trace.names.count, spans, app.selected, app.baseline);
    summary := Model.statistics(*app.trace, app.selected, "", .NAME, false);

    defer array_free(summary);
    for summary print("zone=% calls=% self_ns=% total_ns=% avg_ns=% max_ns=%\n", app.trace.names[it.zone], it.calls, it.self_ns, it.total, cast(u64) it.average, it.maximum);

    print("smoke: timeline selection, sorting, search, comparison ok\n");

    return true;
}

pixel_matches :: (image: Rendered_Image, x: float, y: float, expected: [3] u8) -> bool {
    px := cast(s64) x;
    py := cast(s64) y;

    if px < 0 || py < 0 || px >= image.width || py >= image.height return false;

    index := (py * image.width + px) * 4;

    for expected {
        delta := cast(s64) image.pixels[index + it_index] - cast(s64) it;
        if delta < -2 || delta > 2 return false;
    }

    return true;
}
