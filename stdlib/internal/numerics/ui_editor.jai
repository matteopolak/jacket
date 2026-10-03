MAX_STASHED_COLORS :: 10;
EPSILON :: 0.0001;

get_desired_aspect_ratio :: (theme: *Color_Picker_Theme) -> float {
    return 1;
}

get_desired_aspect_ratio :: (theme: *Color_Animation_Editor_Theme) -> float {
    return 2;
}

set_current_color_hsv :: (state: *Color_Picker_State, hsv: Vector3, update_zoom_hue := true) {
    state.current_hsv_color = hsv;
    state.current_rgb_color = hsv_to_rgb(hsv);
    radians := hsv.x * PI / 180;

    state.ring_point = .{ cos(radians), sin(radians) };
    state.disc_point = backsolve_disc_point_from_hsv(state, hsv.x, hsv.y, hsv.z);
    if update_zoom_hue state.zoom_hue_center = hsv.x;
}

set_original_and_current_color_rgb :: (state: *Color_Picker_State, rgb: Vector3) {
    state.original_rgb_color = rgb;
    set_current_color_hsv(state, rgb_to_hsv(rgb));
}

set_current_color_rgb :: (state: *Color_Picker_State, theme: *Color_Picker_Theme, rgb: Vector3, update_hsv := true, maintain_h := FLOAT32_INFINITY, update_disc_point := true) {
    state.current_rgb_color = rgb;

    if update_hsv {
        hsv := rgb_to_hsv(rgb);
        if !isinf(maintain_h) hsv.x = maintain_h;

        saved := state.disc_point;

        set_current_color_hsv(state, hsv);
        if !update_disc_point state.disc_point = saved;
    }
}

backsolve_disc_point_from_rgb :: (state: *Color_Picker_State, rgb: Vector3) -> Vector2 {
    hsv := rgb_to_hsv(rgb);
    return backsolve_disc_point_from_hsv(state, hsv.x, hsv.y, hsv.z);
}

backsolve_disc_point_from_hsv_warp :: (h: float, s: float, v: float) -> Vector2 {
    a := h * PI / 180;
    radius := saturate(s) * saturate(v);
    return .{ radius * cos(a), radius * sin(a) };
}

backsolve_disc_point_from_hsv :: (state: *Color_Picker_State, h: float, s: float, v: float) -> Vector2 {
    if state.circle_picker_in_warped_mode return backsolve_disc_point_from_hsv_warp(h, s, v);
    return .{ 2 * s - 1, 2 * v - 1 };
}

hsv_to_argb :: (hsv: Vector3) -> Vector4 {
    return xyzw(hsv_to_rgb(hsv), 1);
}

color_picker :: (r: Rect, input_and_output_color_rgb: *Vector3, theme: *Color_Picker_Theme = null, identifier: s64 = 0, loc := #caller_location) -> (applied: bool, still_dragging: bool, state: *Color_Picker_State) {
    if theme == null theme = *default_overall_theme.color_picker_theme;

    created: bool;
    state, created = find_or_create_state(Color_Picker_State, get_hash(loc, identifier));
    if created {
        set_original_and_current_color_rgb(state, input_and_output_color_rgb.*);
        state.color_pick_mode = theme.starting_color_pick_mode;
        for c: theme.initial_stashed_colors array_add(*state.stashed_colors, c);
    }

    rounded_rectangle(r, theme.background.shape, theme.background.color, theme.background.frame_color);
    content := shrink(r, min(r.w, r.h) * theme.horizontal_padding);
    controls, body := cut_bottom(content, content.h * 0.15);
    modes, area := cut_top(body, body.h * 0.12);
    names: [4] string = .["Circle", "HSV", "RGB", "Text"];

    for name, i: names {
        choice := Rect.{ modes.x + modes.w * cast(float) i / 4, modes.y, modes.w / 4, modes.h };
        pressed := button(choice, name, *theme.mode_switch_button_theme, child_id(loc, identifier, cast(u64) (100 + i)), loc);
        if pressed state.color_pick_mode = cast(Color_Pick_Mode) i;
    }

    stash, main_area := cut_right(area, area.w * theme.stashed_color_width);

    area = main_area;
    stash_button, stash_colors := cut_top(stash, stash.h / 11);
    stash_pressed := button(stash_button, "+", *theme.stash_button_theme, child_id(loc, identifier, 120), loc);

    if stash_pressed {
        if state.stashed_colors.count >= MAX_STASHED_COLORS array_ordered_remove_by_index(*state.stashed_colors, 0);
        array_add(*state.stashed_colors, state.current_rgb_color);
    }

    for color, i: state.stashed_colors {
        cell := Rect.{ stash_colors.x, stash_colors.y + stash_colors.h * cast(float) i / MAX_STASHED_COLORS, stash_colors.w, stash_colors.h / MAX_STASHED_COLORS };
        t := theme.stash_button_theme;
        t.surface_color = xyzw(color, 1);
        pressed := button(cell, "", *t, child_id(loc, identifier, cast(u64) (130 + i)), loc);
        if pressed set_current_color_hsv(state, rgb_to_hsv(color));
    }

    if state.color_pick_mode != .HSV_CIRCLE {
        values := state.current_hsv_color;
        if state.color_pick_mode == .RGB_SLIDERS || state.color_pick_mode == .TYPE_INS values = state.current_rgb_color;

        changed := false;

        for i: 0..2 {
            row := Rect.{ area.x, area.y + area.h * cast(float) i / 3, area.w, area.h / 3 };
            high: float = 1;
            low: float = 0;

            if state.color_pick_mode == .HSV_SLIDERS && i == 0 {
                high = theme.slider_hue_max;
                low = theme.slider_hue_min;
            }

            if state.color_pick_mode == .TYPE_INS {
                input_theme: Number_Input_Theme;
                input_theme.text_input_theme = theme.type_ins_input_theme.text_input_theme;
                if number_input(row, "", *values.component[i], low, high, *input_theme, child_id(loc, identifier, cast(u64) (180 + i)), loc) changed = true;
            } else {
                modified := slider(row, *values.component[i], low, high, 0.01, *default_overall_theme.slider_theme, identifier = child_id(loc, identifier, cast(u64) (150 + i)), loc = loc);
                if modified changed = true;
            }
        }

        if changed {
            if state.color_pick_mode == .HSV_SLIDERS set_current_color_hsv(state, values);
            else set_current_color_rgb(state, theme, values);
        }
    } else {
        center := Vector2.{ area.x + area.w / 2, area.y + area.h / 2 };
        outer := min(area.w, area.h) * theme.outer_circle_diameter / 2;
        inner := outer * theme.inner_circle_diameter;
        status := get_status_flags(area, *state.widget);
        down := (ui_get_button_state(.MOUSE_BUTTON_LEFT) & .DOWN) != 0;
        radial := distance(mouse_position, center);

        if (status & .PRESSED) != 0 {
            if radial >= inner && radial <= outer state.dragging_in_ring = true;
            else if radial < inner state.dragging_in_disc = true;
            active_widget_add(*state.widget, true);
            click_consumed = true;
        }

        if state.dragging_in_ring {
            if !down {
                state.dragging_in_ring = false;
                active_widget_remove(*state.widget);
            } else {
                delta := mouse_position - center;
                hsv := state.current_hsv_color;
                hsv.x = fmod_cycling(atan2(delta.y, delta.x) * 180 / PI, 360.0);
                set_current_color_hsv(state, hsv);
            }
        }

        disc := Rect.{ center.x - inner * 0.7, center.y - inner * 0.7, inner * 1.4, inner * 1.4 };

        if state.dragging_in_disc {
            if !down {
                state.dragging_in_disc = false;
                active_widget_remove(*state.widget);
            } else {
                hsv := state.current_hsv_color;
                hsv.y = saturate((mouse_position.x - disc.x) / disc.w);
                hsv.z = saturate((mouse_position.y - disc.y) / disc.h);
                set_current_color_hsv(state, hsv, false);
            }
        }

        if draw_procs.immediate_quad != null {
            for segment: 0..63 {
                a := cast(float) segment * TAU / 64;
                b := cast(float) (segment + 1) * TAU / 64;
                p := center + xy(cos(a), sin(a)) * inner;
                q := center + xy(cos(a), sin(a)) * outer;
                r1 := center + xy(cos(b), sin(b)) * outer;
                s := center + xy(cos(b), sin(b)) * inner;
                color := hsv_to_argb(.{ a * 180 / PI, 1, 1 });

                #if LEFT_HANDED draw_procs.immediate_quad(p, s, r1, q, color);
                else draw_procs.immediate_quad(p, q, r1, s, color);
            }

            cells :: 16;
            for x: 0..cells - 1 for y : 0..cells - 1 {
                cell := Rect.{ disc.x + disc.w * cast(float) x / cells, disc.y + disc.h * cast(float) y / cells, disc.w / cells, disc.h / cells };
                a, b, c, d := get_quad(cell);
                color := hsv_to_argb(.{ state.current_hsv_color.x, cast(float) x / (cells - 1), cast(float) y / (cells - 1) });
                draw_procs.immediate_quad(a, b, c, d, color);
            }
        }
    }

    apply_rect, revert_rect := cut_left(controls, controls.w / 2);
    pressed := button(apply_rect, theme.apply_string, *theme.apply_and_revert_button_theme, child_id(loc, identifier, 1), loc);

    applied = pressed;

    if pressed {
        input_and_output_color_rgb.* = state.current_rgb_color;
        state.original_rgb_color = state.current_rgb_color;
    }

    revert := button(revert_rect, theme.revert_string, *theme.apply_and_revert_button_theme, child_id(loc, identifier, 2), loc);
    if revert {
        set_current_color_hsv(state, rgb_to_hsv(state.original_rgb_color));
        input_and_output_color_rgb.* = state.original_rgb_color;
    }

    state.dragging_in_ring_t = saturate(state.dragging_in_ring_t + state.cosmetic_dt * (ifx state.dragging_in_ring then 8.0 else -8.0));
    state.dragging_in_disc_t = saturate(state.dragging_in_disc_t + state.cosmetic_dt * (ifx state.dragging_in_disc then 8.0 else -8.0));
    still_dragging = state.dragging_in_disc || state.dragging_in_ring || state.dragging_in_h;

    return applied, still_dragging, state;
}

set_keyframes :: (state: *Color_Animation_Editor_State, frames: [] Color_Keyframe) {
    array_resize(*state.color_keyframes, frames.count);

    for frame, i: frames {
        state.color_keyframes[i] = .{};
        state.color_keyframes[i].base = frame;
    }
}

get_keyframes :: (state: *Color_Animation_Editor_State) -> [] Color_Keyframe {
    frames: [] Color_Keyframe = .{ state.color_keyframes.count, cast(*Color_Keyframe) alloc(state.color_keyframes.count * size_of(Color_Keyframe)) };
    for key, i: state.color_keyframes frames[i] = key.base;
    return frames;
}

get_interpolation :: (state: *Color_Animation_Editor_State) -> Interpolation {
    interp: Interpolation;
    interp.color_space = cast(type_of(interp.color_space)) state.interpolation_color_space_s32;
    interp.geometry = cast(type_of(interp.geometry)) state.interpolation_geometry_s32;
    interp.wrapping = state.interpolation_wrapping;

    return interp;
}

set_interpolation :: (using state: *Color_Animation_Editor_State, interp: Interpolation) {
    interpolation_color_space_s32 = cast(s32) interp.color_space;
    interpolation_geometry_s32 = cast(s32) interp.geometry;
    interpolation_wrapping = interp.wrapping;
}

get_sorted_keyframes :: (state: *Color_Animation_Editor_State, index_to_ignore := -1) -> [] *Augmented_Keyframe {
    keys: [..] *Augmented_Keyframe;
    for *key, i: state.color_keyframes if i != index_to_ignore array_add(*keys, key);

    for i: 1..keys.count - 1 {
        j := i;
        while j > 0 && keys[j].t < keys[j - 1].t {
            swap(*keys[j], *keys[j - 1]);
            j -= 1;
        }
    }

    return .{ keys.count, keys.data };
}

find_index_of_t :: (sorted: [] *Augmented_Keyframe, t: float) -> (index: int) {
    index = 0;
    while index + 1 < sorted.count && sorted[index + 1].t <= t index += 1;
    return index;
}

color_animation_editor :: (r: Rect, theme: *Color_Animation_Editor_Theme = null, identifier: s64 = 0, loc := #caller_location) -> (state: *Color_Animation_Editor_State) {
    if theme == null theme = *default_overall_theme.color_animation_editor_theme;

    created: bool;
    state, created = find_or_create_state(Color_Animation_Editor_State, get_hash(loc, identifier));
    state.theme = theme;

    if created {
        set_keyframes(state, theme.default_keyframes);
        set_interpolation(state, theme.default_interpolation);
        state.num_clipboard_samples = theme.default_num_clipboard_samples;
        state.time_view_period = theme.default_time_view_period;
        state.dragging_keyframe_index = -1;
        state.color_picker_keyframe_index = -1;
    }

    rounded_rectangle(r, theme.background.shape, theme.background.color, theme.background.frame_color);
    bar, controls := cut_top(shrink(r, min(r.w, r.h) * theme.horizontal_padding), r.h * 0.65);

    update_keyframes(bar, theme, state);
    draw_color_gradient_bar(bar, theme, state, 0);
    for i: 0..state.color_keyframes.count - 1 draw_color_keyframe(i, bar, theme, state);

    if (theme.draw_subwidgets_flags & .INTERPOLATION_CONTROLS) != 0 {
        row, rest := cut_top(controls, controls.h / 2);

        controls = rest;
        spaces: [3] string = .["RGB", "HSV", "OKLab"];
        geometries: [3] string = .["Linear", "Quadric", "Hermite"];
        first, last := cut_left(row, row.w / 3);
        second, third := cut_left(last, last.w / 2);

        dropdown(first, spaces, *state.interpolation_color_space_s32, *theme.dropdown_theme, child_id(loc, identifier, 500), loc = loc);
        dropdown(second, geometries, *state.interpolation_geometry_s32, *theme.dropdown_theme, child_id(loc, identifier, 501), loc = loc);
        toggle := button(third, "Wrap", *theme.button_theme, child_id(loc, identifier, 502), loc);

        if toggle state.interpolation_wrapping = !state.interpolation_wrapping;
    }

    if (theme.draw_subwidgets_flags & .TIME_VIEW) != 0 {
        control, rest := cut_left(controls, controls.w / 3);
        controls = rest;
        pressed := button(control, "Preview", *theme.button_theme, child_id(loc, identifier, 503), loc);
        if pressed open_time_view(theme, state, bar);
    }

    if (theme.draw_subwidgets_flags & .SAMPLING_CONTROLS) != 0 {
        control, rest := cut_left(controls, controls.w / 2);
        controls = rest;
        slider(control, *state.num_clipboard_samples, 0, 4096, 1, *default_overall_theme.slider_theme, identifier = child_id(loc, identifier, 504), loc = loc);
    }

    if (theme.draw_subwidgets_flags & .COPY_TO_CLIPBOARD) != 0 {
        pressed := button(controls, "Copy", *theme.button_theme, child_id(loc, identifier, 505), loc);
        if pressed copy_output_to_clipboard(state, cast(Output_Type) state.output_type_s32);
    }

    if state.color_picker_subwindow_info.open draw_color_picker(theme, state);
    if state.time_view_subwindow_info.open draw_time_view(theme, state);

    update_keyframe_color_due_to_open_color_picker(state);

    return state;
}

draw_color_gradient_bar :: (area: Rect, theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State, top_margin: float) {
    count := max(2, cast(int) ceil(area.w));
    count = min(count, 4096);
    array_resize(*state.color_samples, count);
    sample_editor(state, state.color_samples);
    assert(draw_procs.immediate_quad != null, "quad drawing callback required");

    for color, i: state.color_samples {
        r := Rect.{ area.x + area.w * cast(float) i / cast(float) count, area.y + top_margin, area.w / cast(float) count + 1, area.h - top_margin };
        a, b, c, d := get_quad(r);
        draw_procs.immediate_quad(a, b, c, d, color);
    }
}

update_keyframes :: (rect: Rect, theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State) {
    status := get_status_flags(rect, *state.widget);
    if (status & .PRESSED) != 0 && (ui_get_button_state(.CTRL) & .DOWN) != 0 {
        key: Augmented_Keyframe;
        key.t = saturate((mouse_position.x - rect.x) / rect.w);
        key.color_rgb = .{ 1, 1, 1, 1 };
        array_add(*state.color_keyframes, key);
        init_color_by_interpolation(state, state.color_keyframes.count - 1);
        click_consumed = true;

        return;
    }

    if (ui_get_button_state(.DELETE) & .START) != 0 && is_inside(mouse_position.x, mouse_position.y, rect) && state.color_keyframes.count > 0 {
        nearest := 0;
        best := FLOAT32_INFINITY;

        for key, i: state.color_keyframes {
            d := abs(mouse_position.x - (rect.x + rect.w * key.t));
            if d < best {
                best = d;
                nearest = i;
            }
        }

        array_ordered_remove_by_index(*state.color_keyframes, nearest);
        state.dragging_in_strip = false;
        state.color_picker_keyframe_index = -1;

        return;
    }

    if (status & .PRESSED) != 0 && state.color_keyframes.count > 0 {
        nearest := 0;
        best := FLOAT32_INFINITY;

        for key, i: state.color_keyframes {
            distance := abs(mouse_position.x - (rect.x + rect.w * key.t));
            if distance < best {
                best = distance;
                nearest = i;
            }
        }

        if best < rect.h * theme.keyframe_selection_size_relative_to_visual_size {
            state.dragging_keyframe_index = nearest;
            state.dragging_in_strip = true;
            active_widget_add(*state.widget, true);
            click_consumed = true;
        }
    }

    if state.dragging_in_strip {
        if (ui_get_button_state(.MOUSE_BUTTON_LEFT) & .DOWN) == 0 {
            state.dragging_in_strip = false;
            active_widget_remove(*state.widget);
        } else if state.dragging_keyframe_index >= 0 && state.dragging_keyframe_index < state.color_keyframes.count state.color_keyframes[state.dragging_keyframe_index].t = saturate((mouse_position.x - rect.x) / rect.w);
    }

    if (ui_get_button_state(.MOUSE_BUTTON_RIGHT) & .START) != 0 && is_inside(mouse_position.x, mouse_position.y, rect) {
        index := 0;
        best := FLOAT32_INFINITY;

        for key, i: state.color_keyframes {
            distance := abs(mouse_position.x - (rect.x + rect.w * key.t));
            if distance < best {
                best = distance;
                index = i;
            }
        }

        if state.color_keyframes.count > 0 open_color_picker(theme, state, index, rect);
    }
}

draw_color_keyframe :: (index: int, rect: Rect, theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State) {
    key := *state.color_keyframes[index];
    width := rect.h * theme.keyframe_width_relative_to_bar_height_nondragging;

    if state.dragging_in_strip && index == state.dragging_keyframe_index width *= theme.keyframe_scale_when_dragging;

    key.pos_x = rect.x + key.t * rect.w;
    shape: Rectangle_Shape;

    rounded_rectangle(.{ key.pos_x - width / 2, rect.y, width, rect.h }, shape, key.color_rgb, theme.keyframe_color_backing);
}

my_color_slider :: (rect: Rect, alpha: float, theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State) -> (changed: bool) {
    if state.dragging_keyframe_index < 0 || state.dragging_keyframe_index >= state.color_keyframes.count return false;
    color := *state.color_keyframes[state.dragging_keyframe_index].color_rgb;
    return slider(rect, *color.w, 0.0, 1.0, 0.01, *default_overall_theme.slider_theme);
}

init_color_by_interpolation :: (state: *Color_Animation_Editor_State, keyframe_index: int) {
    assert(keyframe_index >= 0 && keyframe_index < state.color_keyframes.count);
    sorted := get_sorted_keyframes(state, keyframe_index);

    defer array_free(sorted);
    if sorted.count == 0 return;

    frames: [..] Color_Keyframe;
    for key: sorted array_add(*frames, key.base);

    defer array_free(frames);
    pointers: [..] *Color_Keyframe;
    for *key: frames array_add(*pointers, key);

    defer array_free(pointers);
    interp := get_interpolation(state);
    colors := make_colors(pointers, interp.color_space);

    defer array_free(colors);
    t := state.color_keyframes[keyframe_index].t;
    index := 0;

    while index + 1 < pointers.count - 1 && pointers[index + 1].t <= t index += 1;

    state.color_keyframes[keyframe_index].color_rgb = interpolate_one_sample(index, t, interp, pointers, colors);
}

open_color_picker :: (theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State, keyframe_index: int, bar_rect: Rect) {
    assert(keyframe_index >= 0 && keyframe_index < state.color_keyframes.count);
    state.color_picker_keyframe_index = keyframe_index;
    info := *state.color_picker_subwindow_info;
    info.open = true;
    info.rect = .{ bar_rect.x, bar_rect.y, bar_rect.w / 2, bar_rect.w / 2 };
    info.user_data = state;
    info.draw = draw_callback_color_picker;
    state.color_picker_editing_color_applied = state.color_keyframes[keyframe_index].color_rgb.xyz;
}

open_time_view :: (theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State, bar_rect: Rect) {
    info := *state.time_view_subwindow_info;
    info.open = true;
    info.rect = bar_rect;
    info.draw = draw_callback_time_view;
    info.user_data = state;
}

draw_color_picker :: (theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State) {
    add_subwindow(*state.color_picker_subwindow_info, *theme.color_picker_subwindow_theme, cast(s64) state);
}

draw_time_view :: (theme: *Color_Animation_Editor_Theme, state: *Color_Animation_Editor_State) {
    add_subwindow(*state.time_view_subwindow_info, *theme.time_view_subwindow_theme, cast(s64) state + 1);
}

draw_callback_color_picker :: (state: *Subwindow_State, r: Rect, data: *void) {
    editor := cast(*Color_Animation_Editor_State) data;
    applied, _, picker := color_picker(r, *editor.color_picker_editing_color_applied, *editor.theme.color_picker_theme, cast(s64) editor);
    editor.last_color_picker_state = picker;
    if applied update_keyframe_color_due_to_open_color_picker(editor);
}

update_keyframe_color_due_to_open_color_picker :: (state: *Color_Animation_Editor_State) {
    i := state.color_picker_keyframe_index;
    if i >= 0 && i < state.color_keyframes.count state.color_keyframes[i].color_rgb.xyz = state.color_picker_editing_color_applied;
}

draw_callback_time_view :: (state: *Subwindow_State, r: Rect, data: *void) {
    editor := cast(*Color_Animation_Editor_State) data;
    if editor.time_view_playing editor.time_view_current_time += cast(float64) dt;

    sample: [256] Vector4;

    sample_editor(editor, sample);
    t: float = 0;
    if editor.time_view_period > 0 t = cast(float) fmod_cycling(editor.time_view_current_time / cast(float64) editor.time_view_period, 1.0);

    index := cast(int) (t * 255);
    shape: Rectangle_Shape;

    rounded_rectangle(r, shape, sample[index], .{ 0, 0, 0, 1 });
}

copy_output_to_clipboard :: (state: *Color_Animation_Editor_State, output_type: Output_Type) {
    builder: String_Builder;

    defer free_buffers(*builder);

    if output_type == .RESAMPLED_ARRAY {
        array_resize(*state.color_samples, max(0, state.num_clipboard_samples));
        sample_editor(state, state.color_samples);
        for color: state.color_samples append_rgba(*builder, color);
    } else {
        for key: state.color_keyframes {
            append_decimal(*builder, cast(float64) key.t, 6);
            append(*builder, " ");
            append_rgba(*builder, key.color_rgb);
        }
    }

    text := builder_to_string(*builder);

    defer free(text);
    clipboard_write(text);
}

get_title_bar_height :: (theme: *Subwindow_Theme, override_render_target_height: s32 = 0) -> float {
    h := render_height;
    if override_render_target_height > 0 h = override_render_target_height;
    if (theme.subwindow_flags & .TITLE_BAR) == 0 return 0;
    return theme.title_bar_size * cast(float) h;
}

subwindow_rect_from_content_rect :: (content_rect: Rect, theme: *Subwindow_Theme, override_render_target_height: s32 = 0) -> Rect {
    r := content_rect;
    title := get_title_bar_height(theme, override_render_target_height);

    #if LEFT_HANDED r.y -= title;
    r.h += title;

    return r;
}

add_subwindow :: (info: *Subwindow_Info, theme: *Subwindow_Theme = null, identifier: s64 = 0, loc := #caller_location,z: s32 = 0) -> (state: *Subwindow_State) {
    if theme == null theme = *default_overall_theme.subwindow_theme;

    created: bool;
    state, created = find_or_create_state(Subwindow_State, get_hash(loc, identifier));
    state.hash = get_hash(loc, identifier);
    state.info = info;
    state.theme = theme.*;
    state.wants_draw = info.open;
    state.render_target_width = render_width;
    state.render_target_height = render_height;
    if info.open array_add_if_unique(*subwindow_popups, state);

    return state;
}

draw_subwindow :: (state: *Subwindow_State) {
    if state.info == null || !state.info.open return;

    theme := *state.theme;
    full := state.info.rect;

    update_dragging(state, *full);
    state.info.rect = full;
    occluder_start(*state.widget, full);
    title, content := cut_top(full, get_title_bar_height(theme));

    state.title_bar_rect = title;
    state.content_rect = content;
    rounded_rectangle(full, theme.region_background.shape, theme.region_background.color, theme.region_background.frame_color);

    if title.h > 0 {
        pressed := button(title, state.info.title_text, *theme.title_bar, cast(s64) state.hash);
        if pressed {
            state.dragging = cast(type_of(state.dragging)) 1;
            state.dragging_last = mouse_position;
            active_widget_add(*state.widget, true);
        }
    }

    if (theme.subwindow_flags & .CLOSE_BUTTON) != 0 {
        close := cut_right(title, title.h * theme.close_button_width);
        pressed := button(close, "×", *theme.close_button, cast(s64) state.hash + 1);

        if pressed {
            state.info.open = false;
            active_widget_remove(*state.widget);
        }
    }

    if state.info.draw != null && state.info.open {
        push_scissor(content);
        state.info.draw(state, content, state.info.user_data);
        pop_scissor();
    }

    if (theme.subwindow_flags & .VISIBLE_RESIZE_CORNER) != 0 draw_resize_corner(theme, state, full);

    occluder_finish(*state.widget);
}

raise :: (state: *Subwindow_State) {
    state.wants_a_raise = true;
    state.info.wants_a_raise = true;
    array_ordered_remove_by_value(*subwindow_popups, state);
    array_add(*subwindow_popups, state);
}

update_dragging :: (state: *Subwindow_State, full: *Rect) {
    if cast(u8) state.dragging == 0 return;

    if (ui_get_button_state(.MOUSE_BUTTON_LEFT) & .DOWN) == 0 {
        state.dragging = cast(type_of(state.dragging)) 0;
        active_widget_remove(*state.widget);
        return;
    }

    delta := mouse_position - state.dragging_last;

    state.dragging_last = mouse_position;
    if cast(u8) state.dragging == 1 {
        full.x += delta.x;
        full.y += delta.y;
    } else {
        flags := state.dragging_resize_control;
        if (flags & .EAST) != 0 full.w += delta.x;

        if (flags & .WEST) != 0 {
            full.x += delta.x;
            full.w -= delta.x;
        }

        #if LEFT_HANDED {
            if (flags & .SOUTH) != 0 full.h += delta.y;

            if (flags & .NORTH) != 0 {
                full.y += delta.y;
                full.h -= delta.y;
            }
        } else {
            if (flags & .NORTH) != 0 full.h += delta.y;

            if (flags & .SOUTH) != 0 {
                full.y += delta.y;
                full.h -= delta.y;
            }
        }

        full.w = max(full.w, state.theme.resize_minimum_size.x * cast(float) render_width);
        full.h = max(full.h, state.theme.resize_minimum_size.y * cast(float) render_height);
        if state.info.fixed_aspect_ratio > 0 full.h = full.w / state.info.fixed_aspect_ratio;
    }
}

maybe_diagonalize :: (flag: Resize_Controls, x: float, w: float, margin: float, low: Resize_Controls, high: Resize_Controls, mouse: float) -> Resize_Controls {
    if mouse < x + margin return flag | low;
    if mouse > x + w - margin return flag | high;
    return flag;
}

adjust_resize_control :: (theme: *Subwindow_Theme, state: *Subwindow_State, f: Resize_Controls) -> Resize_Controls {
    return f & theme.resize_controls;
}

start_resizing :: (state: *Subwindow_State, f: Rect) {
    state.dragging = cast(type_of(state.dragging)) 2;
    state.dragging_start = mouse_position;
    state.dragging_last = mouse_position;
    if state.dragging_resize_control == 0 state.dragging_resize_control = .EAST | .SOUTH;

    active_widget_add(*state.widget, true);
}

get_pointer_from_resize_control :: (flags: Resize_Controls) -> Pointer_Image {
    if (flags & .EW) != 0 && (flags & .NS) != 0 {
        if (flags & .NE) == .NE || (flags & .SW) == .SW return .DRAGGING_NE_SW;
        return .DRAGGING_NW_SE;
    }

    if (flags & .EW) != 0 return .DRAGGING_HORIZONTAL;
    if (flags & .NS) != 0 return .DRAGGING_VERTICAL;

    return .NORMAL;
}

draw_resize_corner :: (theme: *Subwindow_Theme, state: *Subwindow_State, r: Rect) {
    side := theme.resize_rect_size * min(r.w, r.h);
    corner := Rect.{ r.x + r.w - side, r.y, side, side };

    #if LEFT_HANDED corner.y = r.y + r.h - side;
    status := get_status_flags(corner, *state.widget);
    if (status & .PRESSED) != 0 {
        state.dragging_resize_control = .EAST | .SOUTH;
        start_resizing(state, r);
        click_consumed = true;
    }

    if draw_procs.immediate_triangle != null emit_triangle(.{ corner.x + corner.w, corner.y, 0 }, .{ corner.x + corner.w, corner.y + corner.h, 0 }, .{ corner.x, corner.y, 0 }, theme.resize_corner_color);
}

#if !LEFT_HANDED {
    draw_proc_adjust_y1 :: (state: *Subwindow_State, new_y1: float) {
        state.draw_proc_adjusted_y = true;
        state.draw_proc_new_y1 = new_y1;
        state.content_rect.h = new_y1 - state.content_rect.y;
    }
}

#scope_module
sample_editor :: (state: *Color_Animation_Editor_State, samples: [] Vector4) {
    sorted := get_sorted_keyframes(state);

    defer array_free(sorted);
    frames: [..] Color_Keyframe;
    for key: sorted array_add(*frames, key.base);

    defer array_free(frames);
    pointers: [..] *Color_Keyframe;
    for *frame: frames array_add(*pointers, frame);

    defer array_free(pointers);
    generate_color_samples(pointers, get_interpolation(state), samples);
}

append_rgba :: (builder: *String_Builder, color: Vector4) {
    append(builder, "{");

    for value, i: color.component {
        if i > 0 append(builder, ", ");
        append_decimal(builder, cast(float64) value, 6);
    }

    append(builder, "}\n");
}
