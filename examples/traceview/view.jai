// UI declaration and custom, clipped timeline/flame widgets.
build :: (app: *App) {
    for column(.{ padding = 12, gap = 8 }) {
        label("Traceview / Iprof", .{ size = 22 });
        label(tprint("% frames · % zones · selected % · baseline %", app.trace.frames.count, app.trace.names.count, app.selected, app.baseline));
        timeline(app);

        for row(.{ gap = 8 }) {
            if button("Previous") app.selected = max(0, app.selected - 1);
            if button("Next") app.selected = min(app.trace.frames.count - 1, app.selected + 1);
            if button("Use selected as baseline") app.baseline = app.selected;
            if button("Swap frames") swap(*app.selected, *app.baseline);
        }

        selected_ns := app.trace.frames[app.selected].duration;
        baseline_ns := app.trace.frames[app.baseline].duration;

        label(tprint("Frame %: % ns · baseline %: % ns · Δ % ns — flame view", app.selected, selected_ns, app.baseline, baseline_ns, cast(s64) selected_ns - cast(s64) baseline_ns));
        for scroll(.{ height = 210 }) flame(app);

        for row(.{ gap = 8 }) {
            label("Search zones");
            text_field(*app.search, .{ placeholder = "case-sensitive name", grow = 1 });
        }

        label("Times in ns; Δ is selected minus baseline. Click a heading to sort.", .{ muted = true });

        for row(.{ gap = 8 }) {
            for name, index: string.["Zone", "Calls", "Self", "Total", "Avg", "Max"] {
                key(index);

                if button(name, .{ width = ifx index == 0 then 160.0 else 100.0 }) {
                    column := cast(Model.Sort_Column) index;
                    if app.column == column app.descending = !app.descending;
                    else {
                        app.column = column;
                        app.descending = column != .NAME;
                    }
                }
            }

            label("Δ calls / total");
        }

        for scroll(.{ grow = 1 }) zone_table(app);
    }
}

timeline :: (app: *App) {
    ui := current_ui();
    node := widget(.LEAF, .CUSTOM, .{ height = 90 }, #location());
    node.label = "Frame timeline";
    node.pointer = true;
    node.paint = paint_timeline;
    node.paint_data = app;
    if events_of(ui, node).clicked && node.rect.w > 0 {
        fraction := (ui.input.mouse_x - node.rect.x) / node.rect.w;
        app.selected = clamp(cast(s64) (fraction * cast(float) app.trace.frames.count), 0, app.trace.frames.count - 1);
    }
}

paint_timeline :: (ui: *UI_State, node: *Node) {
    app := cast(*App) node.paint_data;
    r := node.rect;

    push_clip(ui, r);
    defer pop_clip(ui);
    draw_rect(ui, r, rgb(24, 28, 36));
    width := r.w / cast(float) app.trace.frames.count;

    for index: 0..app.trace.frames.count - 1 {
        height := max(2.0, cast(float) Model.frame_height(*app.trace, index) * (r.h - 18));
        color := ifx index == app.selected then rgb(76, 181, 245) else rgb(85, 110, 145);

        if index == app.baseline && index != app.selected color = rgb(226, 174, 70);

        draw_rect(ui, .{ r.x + cast(float) index * width + 1, r.y + r.h - height, max(1.0, width - 2), height }, color);
        if width > 35 draw_text(ui, r.x + cast(float) index * width + 3, r.y, tprint("%", index), .{ size = 12, color = rgb(240, 240, 240) });
    }
}

flame :: (app: *App) {
    current := *app.trace.frames[app.selected];
    boxes := Model.flame_layout(current);

    defer array_free(boxes);
    depth: s64;
    for boxes depth = max(depth, it.depth);

    node := widget(.LEAF, .CUSTOM, .{ height = cast(float) (depth + 1) * 28 }, #location());
    node.label = "Flame view";
    node.paint = paint_flame;
    node.paint_data = app;
}

paint_flame :: (ui: *UI_State, node: *Node) {
    app := cast(*App) node.paint_data;
    current := *app.trace.frames[app.selected];
    boxes := Model.flame_layout(current);

    defer array_free(boxes);
    push_clip(ui, node.rect);
    defer pop_clip(ui);

    for box: boxes {
        span := current.spans[box.span];
        rect := Rect.{ node.rect.x + cast(float) box.x * node.rect.w, node.rect.y + cast(float) box.depth * 28, cast(float) box.width * node.rect.w, 26 };
        color := rgb(cast(u8) (100 + span.zone * 37 % 130), cast(u8) (80 + span.zone * 19 % 110), 65);

        draw_rect(ui, rect, color, 2);

        if rect.w > 20 {
            push_clip(ui, rect);
            draw_text(ui, rect.x + 4, rect.y + 4, app.trace.names[span.zone], .{ size = 12, color = rgb(255, 255, 255) });
            pop_clip(ui);
        }
    }
}

zone_table :: (app: *App) {
    selected := Model.statistics(*app.trace, app.selected, contents(*app.search), app.column, app.descending, include_empty = true);
    baseline := Model.statistics(*app.trace, app.baseline);

    defer array_free(selected);
    defer array_free(baseline);

    for stat: selected {
        base: Model.Stat;
        for baseline if it.zone == stat.zone base = it;
        if stat.calls == 0 && base.calls == 0 continue;

        for row(.{ gap = 8 }) {
            cell(app.trace.names[stat.zone], 160);
            cell(tprint("%", stat.calls), 100);
            cell(tprint("%", stat.self_ns), 100);
            cell(tprint("%", stat.total), 100);
            cell(tprint("%", cast(u64) stat.average), 100);
            cell(tprint("%", stat.maximum), 100);
            label(tprint("% / %", stat.calls - base.calls, cast(s64) stat.total - cast(s64) base.total));
        }
    }
}

cell :: (text: string, width: float) {
    node := label(text);
    node.style.width = width;
}
