Config :: struct {
    line_spacing: s32 = 8;
    font_character_height: s32 = 8;
    draw_text: (x: float, y: float, str: string, color: Vector4) = dummy_draw_text;
    text_width: (s: string) -> float = dummy_text_width;
    draw_rectangle: (x0: float, y0: float, x1: float, y1: float, color: Vector4) = dummy_draw_rectangle;
    draw_line: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color: Vector4) = dummy_draw_line;
    graph_begin: (x: float, y: float, width: float, height: float) = dummy_graph_begin;
    graph_end: (x: float, y: float, width: float, height: float) = dummy_graph_end;
    limit_name_column_character_length := false;
    zone_time_precision_digits: s32 = 2;
    title_background_color := Vector4.{ .05, .05, .05, .97 };
    title_text_color := Vector4.{ .7, .7, .7, 1 };
    zone_text_color_cold := Vector4.{ .8, .15, .15, 1 };
    zone_text_color_hot := Vector4.{ 1, .9, .9, 1 };
    header_color_unhilit := Vector4.{ .6, .6, .6, 1 };
    header_color_hilit := Vector4.{ 1, 1, .7, 1 };
    bar_color_a := Vector4.{ .1, 0, .03, .93 };
    bar_color_b := Vector4.{ .14, .05, .06, .93 };
    selected_bar_whiten_amount := 0.35;
    graph_zone_under_cursor_text_color := Vector4.{ .8, 1, 1, 1 };
    graph_zone_under_cursor_background_color := Vector4.{ 0, 0, .1, .9 };
    graph_current_time_marker_color := Vector4.{ 1, 1, .9, 1 };
    graph_current_time_history_color := Vector4.{ 1, .3, .3, .75 };
    graph_time_cursor_color := Vector4.{ 1, 1, .9, .5 };
    selected_graph_color := Vector4.{ 1, 1, 1, 1 };
    text_y_offset: s32 = 0;
    graph_colors: [] Vector4;
    user_data: *void;
}

set_cursor_screen_coordinates :: (x: s32, y: s32) {
    profile_mouse_x = x;
    profile_mouse_y = y;
    profile_mouse_valid = true;
}

get_current_graph_scaling :: () -> *Graph_Scaling {
    if displayed_quantity == .SELF_TIME return *profile_self_scaling;
    return *profile_hier_scaling;
}

graph_select_sample :: () {
    if current_graph_window_selected_sample < 0 || current_graph_window_selected_sample >= NUM_FRAME_SLOTS return;

    offset := history_index - 1 - current_graph_window_selected_sample;
    if offset < 0 offset += NUM_FRAME_SLOTS;

    set_frame(offset);
    if zone_under_cursor displayed_quantity = .CALL_GRAPH;
}

draw :: (x: float, y: float, full_width: float, requested_height: float, config: *Config) {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    assert(config != null && full_width >= 0 && requested_height >= 0);
    report := create_report();
    step := cast(float) max(cast(s32) 1, config.font_character_height + config.line_spacing);

    config.draw_rectangle(x, y, x + full_width, y + min(step * 2, requested_height), config.title_background_color);
    config.draw_text(x, y + cast(float) config.text_y_offset, report.title, config.title_text_color);
    right_width := config.text_width(report.title_right);

    config.draw_text(x + max(cast(float) 0, full_width - right_width), y + cast(float) config.text_y_offset, report.title_right, config.title_text_color);
    columns: [5] float = .[x, x + full_width * .5, x + full_width * .65, x + full_width * .8, x + full_width * .9];

    for title, i: report.header if title {
        color := ifx i == report.hilit_header then config.header_color_hilit else config.header_color_unhilit;
        config.draw_text(columns[i], y + step + cast(float) config.text_y_offset, title, color);
    }

    for row, i: report.records {
        row_y := y + step * cast(float) (i + 2);
        if row_y + step > y + requested_height break;

        background := ifx (i & 1) == 0 then config.bar_color_a else config.bar_color_b;
        if i == report.hilight background = profile_whiten(background, config.selected_bar_whiten_amount);

        config.draw_rectangle(x, row_y, x + full_width, row_y + step, background);
        name := row.name;
        if config.limit_name_column_character_length name = profile_fit_name(name, full_width * .48, config);

        color := ifx row.heat >= .5 then config.zone_text_color_hot else config.zone_text_color_cold;

        config.draw_text(x + cast(float) row.indent * 8, row_y + cast(float) config.text_y_offset, name, color);
        values: [4] string = .[
            tprint("%", formatFloat(row.self_time, trailing_width = cast(s64) config.zone_time_precision_digits, zero_removal = .NO)),
            tprint("%", formatFloat(row.hier_time, trailing_width = cast(s64) config.zone_time_precision_digits, zero_removal = .NO)),
            tprint("%", formatFloat(row.calls, trailing_width = 0)),
            tprint("%", formatFloat(row.heat, trailing_width = 2))];
        for value, j: values config.draw_text(columns[j + 1], row_y + cast(float) config.text_y_offset, value, color);

        if profile_mouse_valid && profile_mouse_x >= x && profile_mouse_x <= x + full_width && profile_mouse_y >= row_y && profile_mouse_y < row_y + step {
            next_zone_under_cursor = row.zone;
            set_cursor(i);
        }
    }
}

draw_graph :: (sx: float, sy: float, requested_width: float, requested_height: float, config: *Config) {
    prior_runtime_guard := Profile_Context.set_active(true);

    defer Profile_Context.set_active(prior_runtime_guard);

    assert(config != null && requested_width >= 0 && requested_height >= 0);
    Prof_init_highlevel();
    now := profile_now();
    dt := ifx profile_graph_last_tick > 0 then cast(float64) (now - profile_graph_last_tick) * profile_nominal_seconds else 0.0;

    profile_graph_last_tick = now;
    scale := get_current_graph_scaling();

    profile_advance_scale(scale, dt);
    size_step := cast(float) min(cast(float64) 1, dt * 5);

    profile_graph_size += (graph_bigness_t_target - profile_graph_size) * size_step;
    width := requested_width * max(cast(float) 0, 1 + profile_graph_size);
    height := requested_height * max(cast(float) 0, 1 + profile_graph_size);

    config.graph_begin(sx, sy, width, height);
    defer config.graph_end(sx, sy, width, height);

    #if DO_HISTORY {
        samples := min(update_index, cast(s64) NUM_FRAME_SLOTS);
        if samples < 1 || width <= 0 || height <= 0 return;

        best_distance: float = 1.0e30;

        current_graph_window_selected_sample = -1;
        prior_zone_under_cursor := zone_under_cursor;

        zone_under_cursor = null;
        mouse_sample: s64 = -1;

        if profile_mouse_valid && profile_mouse_x >= sx && profile_mouse_x <= sx + width && profile_mouse_y >= sy && profile_mouse_y <= sy + height {
            mouse_sample = cast(s64) ((cast(float) profile_mouse_x - sx) / width * cast(float) max(cast(s64) 1, samples - 1));
            mouse_sample = min(mouse_sample, samples - 1);
            current_graph_window_selected_sample = (history_index + NUM_FRAME_SLOTS - samples + mouse_sample) % NUM_FRAME_SLOTS;
        }

        for node, n: zone_hash if node && node.zone && node.zone.initted {
            duplicate := false;

            for j: 0..n - 1 if zone_hash[j] && zone_hash[j].zone == node.zone {
                duplicate = true;
                break;
            }

            if duplicate continue;

            zone := node.zone;
            color := ifx config.graph_colors.count > 0 then config.graph_colors[n % config.graph_colors.count] else config.zone_text_color_cold;

            if zone == prior_zone_under_cursor color = config.selected_graph_color;

            previous: Vector2;

            for i: 0..samples - 1 {
                position := (history_index + NUM_FRAME_SLOTS - samples + i) % NUM_FRAME_SLOTS;
                value := ifx displayed_quantity == .SELF_TIME then zone.history_self.values[position] else zone.history_hier.values[position];
                point := Vector2.{ sx + width * cast(float) i / cast(float) max(cast(s64) 1, samples - 1),
                    sy + height - min(height, max(cast(float) 0, value * 1000 * cast(float) scale.output_factor)) };

                if i > 0 config.draw_line(previous, previous, point, point, color);

                if i == mouse_sample {
                    distance := Graph_Math.abs(point.y - cast(float) profile_mouse_y);
                    if distance < best_distance {
                        best_distance = distance;
                        zone_under_cursor = zone;
                    }
                }

                previous = point;
            }
        }

        previous_frame: Vector2;

        for i: 0..samples - 1 {
            position := (history_index + NUM_FRAME_SLOTS - samples + i) % NUM_FRAME_SLOTS;
            point := Vector2.{ sx + width * cast(float) i / cast(float) max(cast(s64) 1, samples - 1),
                sy + height - min(height, max(cast(float) 0, frame_time.history[position] * 1000 * cast(float) scale.output_factor)) };

            if i > 0 config.draw_line(previous_frame, previous_frame, point, point, config.graph_current_time_history_color);

            previous_frame = point;
        }

        marker_x := sx + width * cast(float) max(cast(s64) 0, samples - 1 - display_frame) / cast(float) max(cast(s64) 1, samples - 1);
        a := Vector2.{ marker_x, sy };
        b := Vector2.{ marker_x, sy + height };

        config.draw_line(a, a, b, b, config.graph_current_time_marker_color);

        if mouse_sample >= 0 {
            mouse_a := Vector2.{ cast(float) profile_mouse_x, sy };
            mouse_b := Vector2.{ cast(float) profile_mouse_x, sy + height };
            config.draw_line(mouse_a, mouse_a, mouse_b, mouse_b, config.graph_time_cursor_color);
        }

        if zone_under_cursor {
            name_width := config.text_width(zone_under_cursor.name);
            config.draw_rectangle(sx, sy, sx + name_width, sy + cast(float) config.font_character_height, config.graph_zone_under_cursor_background_color);
            config.draw_text(sx, sy, zone_under_cursor.name, config.graph_zone_under_cursor_text_color);
        }
    } else config.draw_text(sx, sy, "Iprof history disabled", config.title_text_color);
}

#scope_module
#import "Basic";
Graph_Math :: #import "Math";

Vector2 :: Graph_Math.Vector2;
Vector4 :: Graph_Math.Vector4;
profile_self_scaling: Graph_Scaling;
profile_hier_scaling: Graph_Scaling;
profile_mouse_x, profile_mouse_y: s32;
profile_mouse_valid: bool;
profile_graph_last_tick: s64;
profile_graph_size: float;

profile_advance_scale :: (scale: *Graph_Scaling, dt: float64) {
    target := cast(float) max(cast(s64) 0, min(scale.target, cast(s64) Graph_Scaling.SCALE_MAX));
    gap := target - scale.current;

    if gap == 0 scale.ease = 0;
    else {
        scale.ease = min(cast(float) 1, scale.ease + cast(float) dt * 5);
        speed: float = ifx Graph_Math.abs(gap) > 1 then 10 else 5;
        step := speed * cast(float) dt * Graph_Math.sqrt(max(cast(float) 0, scale.ease));

        if Graph_Math.abs(gap) <= step {
            scale.current = target;
            scale.ease = 0;
        } else if gap > 0 scale.current += step;
        else scale.current -= step;
    }

    scale.output_factor = Graph_Math.pow(cast(float64) .5, cast(float64) scale.current);
}

profile_whiten :: (color: Vector4, amount: float) -> Vector4 {
    factor := min(cast(float) 1, max(cast(float) 0, amount));
    return .{ color.x + (1 - color.x) * factor, color.y + (1 - color.y) * factor, color.z + (1 - color.z) * factor, color.w };
}

profile_fit_name :: (name: string, available: float, config: *Config) -> string {
    while name.count > 0 && config.text_width(name) > available {
        name.count -= 1;
        while name.count > 0 && (name.data[name.count] & 0xc0) == 0x80 name.count -= 1;
    }

    return name;
}

dummy_draw_text :: (x: float, y: float, str: string, color: Vector4) {
    log_error("Iprof: draw_text callback is unset\n");
}

dummy_text_width :: (s: string) -> float {
    log_error("Iprof: text_width callback is unset\n");
    return 0;
}

dummy_draw_rectangle :: (x0: float, y0: float, x1: float, y1: float, color: Vector4) {
    log_error("Iprof: draw_rectangle callback is unset\n");
}

dummy_draw_line :: (p0: Vector2, p1: Vector2, p2: Vector2, p3: Vector2, color: Vector4) {
    log_error("Iprof: draw_line callback is unset\n");
}

dummy_graph_begin :: (x: float, y: float, width: float, height: float) {
    log_error("Iprof: graph_begin callback is unset\n");
}

dummy_graph_end :: (x: float, y: float, width: float, height: float) {
    log_error("Iprof: graph_end callback is unset\n");
}
