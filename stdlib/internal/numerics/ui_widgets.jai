update_action_durations :: (state: *Button_State, theme: *Button_Theme, begin: bool, dt: float) -> (pressed_flash_factor: float) {
    if begin state.action_duration = 0;
    else if state.action_duration >= 0 state.action_duration += dt;
    if state.action_duration < 0 || theme.press_duration <= 0 return 0;
    return saturate(1 - state.action_duration / theme.press_duration);
}

update_over_state :: (state: *Button_State, theme: *Button_Theme, status: Status_Flags, dt: float) {
    over := (status & .OVER) != 0;
    down := (status & .DOWN) != 0;

    if over state.over_duration += dt;
    else state.over_duration = 0;
    if down state.down_duration += dt;
    else state.down_duration = 0;
    if over state.over_factor = saturate(state.over_factor + dt * theme.over_fade_in_speed);
    else state.over_factor = saturate(state.over_factor - dt * theme.over_fade_out_speed);
    if down state.pressed_factor = saturate(state.pressed_factor + dt * theme.down_fade_in_speed);
    else state.pressed_factor = saturate(state.pressed_factor - dt * theme.down_fade_out_speed);

    state.over_effect_t = state.over_factor;
    state.down_effect_t = state.pressed_factor;
}

update_production_value_button :: (r: Rect, changed: bool, state: *Button_State, status: Status_Flags, theme: *Button_Theme) {
    state.rect = r;
    state.status = status;
    update_over_state(state, theme, status, state.cosmetic_dt);
    update_action_durations(state, theme, changed, state.cosmetic_dt);
}

get_colors_and_thickness_for_button :: (theme: *Button_Theme, state: *Button_State) -> (surface_color: Vector4, frame_color: Vector4, frame_thickness: float) {
    surface_color = lerp(theme.surface_color, theme.surface_color_over, state.over_factor);
    surface_color = lerp(surface_color, theme.surface_color_down, state.pressed_factor);
    frame_color = lerp(theme.frame_color, theme.frame_color_over, state.over_factor);
    frame_color = lerp(frame_color, theme.frame_color_down, state.pressed_factor);
    flash: float = 0;
    if state.action_duration >= 0 && theme.press_duration > 0 flash = saturate(1 - state.action_duration / theme.press_duration);

    surface_color = lerp(surface_color, theme.surface_color_flash, flash);
    frame_color = lerp(frame_color, theme.frame_color_flash, flash);
    surface_color.w *= theme.alpha_scale;
    frame_color.w *= theme.alpha_scale;
    frame_thickness = get_roundedness_for_rect(state.rect, theme.rectangle_shape.frame_thickness_type, theme.rectangle_shape.frame_thickness, frame_pixels_global);

    if theme.enable_variable_frame_thickness {
        frame_thickness = lerp(frame_thickness, theme.frame_thickness_over * state.rect.h, state.over_factor);
        frame_thickness = lerp(frame_thickness, theme.frame_thickness_down * state.rect.h, state.pressed_factor);
    }

    return surface_color, frame_color, frame_thickness;
}

core_draw_button_label :: (r: Rect, text: string, state: *Button_State, theme: *Button_Theme) {
    t := theme.label_theme;
    t.text_color = lerp(t.text_color, theme.text_color_over, state.over_factor);
    t.text_color = lerp(t.text_color, theme.text_color_pressed, state.pressed_factor);
    label(r, text, *t);
}

core_draw_button :: (r: Rect, text: string, state: *Button_State, status: Status_Flags, theme: *Button_Theme, texture: *Texture) {
    if (theme.properties & .DO_NOT_DRAW) != 0 return;

    if theme.draw_callback != null {
        theme.draw_callback(r, text, theme, texture, state);
        return;
    }

    color, frame, thickness := get_colors_and_thickness_for_button(theme, state);
    frame_result := rounded_rectangle(r, theme.rectangle_shape, color, frame, thickness);

    state.frame_pixels = frame_result;

    if texture != null {
        assert(draw_procs.set_shader_for_images != null && draw_procs.immediate_quad != null, "image callbacks required");
        draw_procs.set_shader_for_images(texture);
        a, b, c, d := get_quad(r);
        draw_procs.immediate_quad(a, b, c, d, color);
    }

    if text.count > 0 core_draw_button_label(r, text, state, theme);
}

button :: (r: Rect, text: string, theme: *Button_Theme = null, identifier: s64 = 0, loc := #caller_location,texture: *Texture = null, disable_over := false, disable_press := false) -> (was_just_pressed: bool, state: *Button_State, was_just_released: bool) {
    if theme == null theme = *default_overall_theme.button_theme;

    hash := get_hash(loc, identifier);
    created: bool;
    state, created = find_or_create_state(Button_State, hash);
    state.hash = hash;
    state.rect = r;
    status := get_status_flags(r, *state.widget);
    if disable_over status &= ~ .OVER;
    if disable_press status &= ~(.PRESSED | .DOWN);

    was_just_pressed = (status & .PRESSED) != 0;
    was_just_released = state.is_on_state_stack && (ui_get_button_state(.MOUSE_BUTTON_LEFT) & .END) != 0;

    if was_just_pressed {
        click_consumed = true;
        if (theme.properties & .TOGGLE) != 0 state.pressed = !state.pressed;
        else state.pressed = true;
        active_widget_add(*state.widget, true);
    }

    if was_just_released {
        if (theme.properties & .TOGGLE) == 0 state.pressed = false;
        active_widget_remove(*state.widget);
    }

    if state.pressed status |= .DOWN;

    state.status = status;
    update_over_state(state, theme, status, state.cosmetic_dt);
    update_action_durations(state, theme, was_just_pressed, state.cosmetic_dt);
    core_draw_button(r, text, state, status, theme, texture);

    return was_just_pressed, state, was_just_released;
}

base_checkbox :: (_r: Rect, text: string, selected: bool, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State {
    if theme == null theme = *default_overall_theme.checkbox_theme;

    t := theme.button_theme;

    if selected {
        t.surface_color = theme.button_color_selected;
        t.text_color = theme.text_color_selected;
    }

    clicked, base := button(_r, text, *t, identifier, loc);

    checkbox_state.base = base.*;
    checkbox_state.widget = base.widget;
    checkbox_state.selected_t = 0;
    if selected checkbox_state.selected_t = 1;

    return clicked, checkbox_state;
}

radio_button :: (r: Rect, text: string, current_value: *$Enum, button_value: Enum, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State {
    clicked, state := base_checkbox(r, text, current_value.* == button_value, theme, identifier, loc);
    changed = clicked && current_value.* != button_value;
    if clicked current_value.* = button_value;
    return changed, state;
}

radio_button :: (r: Rect, current_value: *$Enum, button_value: Enum, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State {
    return radio_button(r, "", current_value, button_value, theme, identifier, loc);
}

checkbox_flags :: (r: Rect, text: string, current_value: *$Enum, button_value: Enum, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State {
    clicked, state := base_checkbox(r, text, (current_value.* & button_value) != 0, theme, identifier, loc);
    if clicked current_value.* ^= button_value;
    return clicked, state;
}

checkbox_flags :: (r: Rect, current_value: *$Enum, button_value: Enum, theme: *Checkbox_Theme = null, identifier: s64 = 0, loc := #caller_location) -> changed: bool, checkbox_state: Checkbox_State {
    return checkbox_flags(r, "", current_value, button_value, theme, identifier, loc);
}

dropdown :: (r: Rect, choices: [] string, current_value_pointer: *s32, theme: *Dropdown_Theme = null, identifier: s64 = 0, z: s32 = 100, loc := #caller_location) -> changed: bool, Dropdown_State {
    if theme == null theme = *default_overall_theme.dropdown_theme;

    hash := get_hash(loc, identifier);
    state, created := find_or_create_state(Dropdown_State, combine_hashes(hash, 1));
    state.hash = hash;
    state.base_rect = r;
    state.choices = choices;
    state.theme = theme.*;
    state.loc = loc;
    state.z = z;
    state.current_value_pointer = current_value_pointer;
    state.deactivate_proc = dropdown_deactivate;
    #if LEFT_HANDED if created state.original_value = current_value_pointer.*;
    text := "";
    if current_value_pointer.* >= 0 && current_value_pointer.* < choices.count text = choices[current_value_pointer.*];

    pressed, base := button(r, text, *theme.theme_for_current_value, identifier, loc);

    state.current_value_state = base;

    if pressed {
        state.open = !state.open;
        if state.open {
            active_widget_add(*state.widget);
            array_add_if_unique(*dropdown_popups, state);
        } else active_widget_remove(*state.widget);
    }

    changed := false;

    for *pending: dropdown_changes if pending.state == state {
        changed = pending.changed;
        pending.changed = false;
    }

    return changed, state.*;
}

draw_dropdown_popup :: (_state: *void) {
    state := cast(*Dropdown_State) _state;
    if !state.open return;

    r := state.base_rect;

    occluder_start(*state.widget, .{ r.x, r.y, r.w, r.h * cast(float) (state.choices.count + 1) });

    for text, i: state.choices {
        #if LEFT_HANDED r.y += r.h;
        else r.y -= r.h;
        t := *state.theme.theme_for_each_choice;
        if i == state.current_value_pointer.* t = *state.theme.theme_for_current_choice;

        pressed := button(r, text, t, cast(s64) combine_hashes(state.hash, cast(u64) i + 2), state.loc);
        if pressed {
            if state.current_value_pointer.* != cast(s32) i {
                found := false;

                for *pending: dropdown_changes if pending.state == state {
                    pending.changed = true;
                    found = true;
                }

                if !found array_add(*dropdown_changes, .{ state, true });
            }

            state.current_value_pointer.* = cast(s32) i;
            state.open = false;
            active_widget_remove(*state.widget);
        }
    }

    occluder_finish(*state.widget);
}

dropdown_deactivate :: (widget: *Active_Widget) {
    state := cast(*Dropdown_State) widget;
    state.open = false;
}

draw_popups :: () {
    for p: dropdown_popups if p.open draw_dropdown_popup(p);
    dropdown_popups.count = 0;
    for w: subwindow_popups if w.wants_draw draw_subwindow(w);
    subwindow_popups.count = 0;
}

begin_scrollable_region :: (r: Rect, theme: *Scrollable_Region_Theme = null, identifier: s64 = 0, loc := #caller_location) -> (state: *Scrollable_Region_State, inside: Rect) {
    if theme == null theme = *default_overall_theme.scrollable_region_theme;

    created: bool;
    state, created = find_or_create_state(Scrollable_Region_State, get_hash(loc, identifier));
    state.outer_rect = r;
    state.theme = theme.*;
    inside = r;
    if (theme.margin_flags & .Horizontal) != 0 inside = shrink_x(inside, theme.horizontal_margin_size * r.w);
    if (theme.margin_flags & .Vertical) != 0 inside = shrink_y(inside, theme.vertical_margin_size * r.h);

    if (theme.direction & .VERTICAL) != 0 {
        bar, rest := cut_right(inside, theme.scrollbar_size * r.w);
        state.bar_rect = bar;
        inside = rest;
    } else if (theme.direction & .HORIZONTAL) != 0 {
        bar, rest := cut_bottom(inside, theme.scrollbar_size * r.h);
        state.bar_rect = bar;
        inside = rest;
    }

    state.inner_rect = inside;
    rounded_rectangle(r, theme.region_background.shape, theme.region_background.color, theme.region_background.frame_color);
    push_scissor(inside);

    return state, inside;
}

#if LEFT_HANDED {
    end_scrollable_region :: (state: *Scrollable_Region_State, max_x: float, max_y: float, scroll_value: *float) {
        finish_scroll(state, max_x, max_y - state.inner_rect.y, scroll_value);
    }
} else {
    end_scrollable_region :: (state: *Scrollable_Region_State, max_x: float, min_y: float, scroll_value: *float) {
        finish_scroll(state, max_x, state.inner_rect.y + state.inner_rect.h - min_y, scroll_value);
    }
}

slidable_region :: (r: Rect, theme: *Slidable_Region_Theme = null, identifier: s64 = 0, loc := #caller_location) -> (state: *Slidable_Region_State, left_or_top_rect: Rect, right_or_bottom_rect: Rect, divider_rect: Rect) {
    if theme == null theme = *default_overall_theme.slidable_region_theme;

    created: bool;
    state, created = find_or_create_state(Slidable_Region_State, get_hash(loc, identifier));
    state.theme = theme;

    if created {
        position := theme.initial_divider_position;
        if theme.initial_divider_position_type == .RELATIVE_TO_HEIGHT position *= r.h;
        if theme.initial_divider_position_type == .RELATIVE_TO_WIDTH position *= r.w;
        if theme.initial_divider_position_type == .ABSOLUTE_FROM_GLOBAL position = divider_initial_pixels;
        if theme.orientation == .HORIZONTAL state.divider_t = position / r.w;
        else state.divider_t = position / r.h;
    }

    thick := get_roundedness_for_rect(r, theme.divider_thickness_type, theme.divider_thickness, divider_pixels, 1.0);

    if theme.orientation == .HORIZONTAL {
        left_or_top_rect, right_or_bottom_rect = cut_left(r, r.w * state.divider_t, thick);
        divider_rect = .{ left_or_top_rect.x + left_or_top_rect.w, r.y, thick, r.h };
    } else {
        left_or_top_rect, right_or_bottom_rect = cut_top(r, r.h * state.divider_t, thick);
        divider_rect = .{ r.x, min(left_or_top_rect.y + left_or_top_rect.h, right_or_bottom_rect.y + right_or_bottom_rect.h), r.w, thick };
    }

    pressed, b, released := button(divider_rect, "", *theme.divider_theme, child_id(loc, identifier, 1), loc);

    state.divider_state = b.*;
    if pressed state.dragging = true;
    if released state.dragging = false;

    if state.dragging {
        if theme.orientation == .HORIZONTAL state.divider_t = saturate((mouse_position.x - r.x) / r.w);
        else {
            #if LEFT_HANDED state.divider_t = saturate((mouse_position.y - r.y) / r.h);
            else state.divider_t = saturate((r.y + r.h - mouse_position.y) / r.h);
        }
    }

    state.left_or_top_rect = left_or_top_rect;
    state.right_or_bottom_rect = right_or_bottom_rect;
    state.divider_rect = divider_rect;
    rounded_rectangle(left_or_top_rect, theme.first_background.shape, theme.first_background.color, theme.first_background.frame_color);
    push_scissor(left_or_top_rect);

    return state, left_or_top_rect, right_or_bottom_rect, divider_rect;
}

switch_to_second_rect :: (state: *Slidable_Region_State) {
    pop_scissor();
    rounded_rectangle(state.right_or_bottom_rect, state.theme.second_background.shape, state.theme.second_background.color, state.theme.second_background.frame_color);
    push_scissor(state.right_or_bottom_rect);
}

switch_to_first_rect :: (state: *Slidable_Region_State) {
    pop_scissor();
    push_scissor(state.left_or_top_rect);
}

switch_to_bottom_rect :: switch_to_second_rect;
switch_to_right_rect :: switch_to_second_rect;
switch_to_top_rect :: switch_to_first_rect;
switch_to_left_rect :: switch_to_first_rect;

finish :: (state: *Slidable_Region_State) {
    pop_scissor();
}

set_initial_value :: (state: *Slider_State, value: $T) {
    #if T == float32 state.initial_value_float32 = value;
    else #if T == float64 state.initial_value_float64 = value;
    else state.initial_value_int = cast(int) value;
}

get_initial_value :: (state: *Slider_State, $T: Type) -> T {
    #if T == float32 return state.initial_value_float32;
    else #if T == float64 return state.initial_value_float64;
    else return cast(T) state.initial_value_int;
}

set_and_maybe_clamp :: (new_value: $ValueType, pointer: *$T, min_value: T, max_value: T, clamp_low: bool, clamp_high: bool) {
    if clamp_low && new_value <= cast(ValueType) min_value {
        pointer.* = min_value;
        return;
    }

    if clamp_high && new_value >= cast(ValueType) max_value {
        pointer.* = max_value;
        return;
    }

    pointer.* = cast(T) new_value;
}

slider :: (_r: Rect, value: *$T, min_value: T, max_value: T, spinbox_step: T, theme: *Slider_Theme, prefix := "", suffix := "", identifier: s64 = 0, loc := #caller_location) -> changed: bool, entered: bool {
    if theme == null theme = *default_overall_theme.slider_theme;

    state := find_or_create_state(Slider_State, get_hash(loc, identifier));
    state.value_type = type_info(T);
    state.changed_this_frame = false;
    state.deactivate_proc = slider_deactivate;
    old := value.*;
    r := _r;
    range := cast(float64) max_value - cast(float64) min_value;

    if theme.use_spinboxes {
        plus, rest := cut_right(r, r.h * theme.spinbox_button_width);
        minus, track := cut_right(rest, r.h * theme.spinbox_button_width);

        r = track;
        plus_pressed := button(plus, "+", *theme.spinbox_theme, child_id(loc, identifier, 2), loc);
        minus_pressed := button(minus, "-", *theme.spinbox_theme, child_id(loc, identifier, 3), loc);

        if plus_pressed set_and_maybe_clamp(cast(float64) value.* + cast(float64) spinbox_step, value, min_value, max_value, true, true);
        if minus_pressed set_and_maybe_clamp(cast(float64) value.* - cast(float64) spinbox_step, value, min_value, max_value, true, true);
    }

    status := get_status_flags(r, *state.widget);
    if (status & .PRESSED) != 0 {
        state.sliding = true;
        set_initial_value(state, value.*);
        active_widget_add(*state.widget, true);
        click_consumed = true;
        state.current_slide_setting = mouse_position.x;
        state.sliding_is_relative = theme.sliding_is_relative || (theme.sliding_becomes_relative_by_pressing_shift && (ui_get_button_state(.SHIFT) & .DOWN) != 0);
        state.accumulated_delta_for_int_slider = 0;
    }

    if state.sliding {
        f := saturate((mouse_position.x - r.x) / r.w);
        new_value := cast(float64) min_value + range * cast(float64) f;

        if state.sliding_is_relative {
            factor := theme.relative_slide_factor;
            if (ui_get_button_state(.SHIFT) & .DOWN) != 0 factor = theme.relative_slide_factor_modified;

            delta := cast(float64) ((mouse_position.x - state.current_slide_setting) / r.w * factor) * range;

            state.current_slide_setting = mouse_position.x;

            #if T == float32 || T == float64 new_value = cast(float64) value.* + delta;
            else {
                delta += cast(float64) state.accumulated_delta_for_int_slider;
                whole := floor(delta);
                if delta < 0 whole = -floor(-delta);

                state.accumulated_delta_for_int_slider = cast(float) (delta - whole);
                new_value = cast(float64) value.* + whole;
            }
        }

        #if T == float32 || T == float64 set_and_maybe_clamp(new_value, value, min_value, max_value, true, true);
        else set_and_maybe_clamp(floor(new_value + 0.5), value, min_value, max_value, true, true);

        if (ui_get_button_state(.MOUSE_BUTTON_LEFT) & .END) != 0 {
            state.sliding = false;
            active_widget_remove(*state.widget);
        }
    }

    foreground := r;
    fraction: float = 0;

    if range != 0 fraction = cast(float) ((cast(float64) value.* - cast(float64) min_value) / range);

    if theme.surface_style == .NUB {
        width := get_roundedness_for_rect(r, theme.nub_thickness_type, theme.nub_thickness, nub_pixels, 1);
        foreground.w = width;
        foreground.x = r.x + (r.w - width) * fraction;
    } else foreground.w *= fraction;

    state.background.rect = r;
    state.foreground.rect = foreground;
    theme.draw_background(r, theme, state);
    theme.draw_foreground(r, foreground, theme, state);
    text: string;

    #if T == float32 || T == float64 text = theme.format_text_float(prefix, suffix, cast(float64) value.*, theme, state, .DISPLAY);
    else text = theme.format_text_int(prefix, suffix, cast(s64) value.*, theme, state, .DISPLAY);
    defer free(text);
    if text.count > 0 && draw_procs.prepare_text != null label(r, text, *theme.foreground.label_theme);

    entered := false;

    if theme.text_editable && (ui_get_button_state(.MOUSE_BUTTON_RIGHT) & .START) != 0 && is_inside(mouse_position.x, mouse_position.y, r) state.inputting_text = true;

    if state.inputting_text {
        edit_text: string;

        #if T == float32 || T == float64 edit_text = theme.format_text_float("", "", cast(float64) value.*, theme, state, .EDITING);
        else edit_text = theme.format_text_int("", "", cast(s64) value.*, theme, state, .EDITING);
        defer free(edit_text);
        action, result := text_input(r, edit_text, *theme.text_input_theme, child_id(loc, identifier, 4), loc, input_action = .ACTIVATE);
        if (action & .ENTERED) != 0 {
            #if T == float32 || T == float64 {
                ok, parsed := theme.parse_text_float(result, theme, state);
                if ok {
                    set_and_maybe_clamp(parsed, value, min_value, max_value, theme.clamp_text_input_low, theme.clamp_text_input_high);
                    entered = true;
                }
            } else {
                ok, parsed := theme.parse_text_int(result, theme, state);
                if ok {
                    set_and_maybe_clamp(parsed, value, min_value, max_value, theme.clamp_text_input_low, theme.clamp_text_input_high);
                    entered = true;
                }
            }

            state.inputting_text = false;
        }

        if (action & .ESCAPED) != 0 state.inputting_text = false;
    }

    state.changed_this_frame = old != value.*;

    return state.changed_this_frame, entered;
}

to_vec3 :: (v2: Vector2) -> Vector3 {
    return xyz(v2, 0);
}

draw_arrow :: (state: *Button_State, p4: Vector3, p5: Vector3, p6: Vector3, theme: *Button_Theme) {
    assert(draw_procs.immediate_triangle != null);
    emit_triangle(p4, p5, p6, theme.text_color);
}

slider_draw_background_default :: (r: Rect, theme: *Slider_Theme, state: *Slider_State) {
    core_draw_button(r, "", *state.background, state.background.status, *theme.background, null);
}

slider_draw_foreground_default :: (r_background: Rect, r_foreground: Rect, theme: *Slider_Theme, state: *Slider_State) {
    core_draw_button(r_foreground, "", *state.foreground, state.foreground.status, *theme.foreground, null);
}

slider_format_text_float_default :: (prefix: string, suffix: string, value: float64, theme: *Slider_Theme, state: *Slider_State, mode: Slider_Format_Text_Mode) -> string {
    builder: String_Builder;

    defer free_buffers(*builder);
    append(*builder, prefix);
    append_decimal(*builder, value, theme.decimals);
    append(*builder, suffix);

    return builder_to_string(*builder);
}

slider_format_text_int_default :: (prefix: string, suffix: string, value: s64, theme: *Slider_Theme, state: *Slider_State, mode: Slider_Format_Text_Mode) -> string {
    return sprint("%%%", prefix, value, suffix);
}

slider_parse_text_float_default :: (text: string, theme: *Slider_Theme, state: *Slider_State) -> (success: bool, value: float64) {
    return parse_decimal(text);
}

slider_parse_text_int_default :: (text: string, theme: *Slider_Theme, state: *Slider_State) -> (success: bool, value: s64) {
    v, ok, rest := string_to_int(text);
    return ok && eat_spaces(rest).count == 0, v;
}

number_input :: (_r: Rect, text: string, value: *$T, min_value: T, max_value: T, theme: *Number_Input_Theme, identifier: s64 = 0, loc := #caller_location,parse_proc: (string) -> (T, bool) = null) -> bool {
    if theme == null theme = *number_default;

    find_or_create_state(Number_Input_State, get_hash(loc, identifier));
    initial: string;

    #if T == float32 || T == float64 {
        builder: String_Builder;
        append_decimal(*builder, cast(float64) value.*, 6);
        initial = builder_to_string(*builder);
        free_buffers(*builder);
    } else initial = sprint("%", value.*);

    defer free(initial);
    action, result := text_input(_r, initial, *theme.text_input_theme, child_id(loc, identifier, 1), loc);
    if (action & .ENTERED) == 0 return false;

    v: T;
    ok: bool;

    if parse_proc != null v, ok = parse_proc(result);
    else {
        #if T == float32 || T == float64 {
            success, f := parse_decimal(result);
            v = cast(T) f;
            ok = success;
        } else {
            f, success, rest := string_to_int(result, T = T);
            v = f;
            ok = success && eat_spaces(rest).count == 0;
        }
    }

    if !ok return false;

    old := value.*;

    set_and_maybe_clamp(v, value, min_value, max_value, theme.clamp_low, theme.clamp_high);

    return old != value.*;
}

#scope_module
dropdown_popups: [..] *Dropdown_State;

Dropdown_Change :: struct {
    state: *Dropdown_State;
    changed: bool;
}

dropdown_changes: [..] Dropdown_Change;
subwindow_popups: [..] *Subwindow_State;

finish_scroll :: (state: *Scrollable_Region_State, max_x: float, extent: float, scroll_value: *float) {
    pop_scissor();
    horizontal := (state.theme.direction & .VERTICAL) == 0 && (state.theme.direction & .HORIZONTAL) != 0;
    visible := state.inner_rect.h;
    dimension := cast(float) render_height;

    if horizontal {
        visible = state.inner_rect.w;
        extent = max_x - state.inner_rect.x;
        dimension = cast(float) render_width;
    }

    max_scroll := max(0.0, extent - visible);

    scroll_value.* = clamp(scroll_value.*, 0.0, max_scroll);

    if (get_status_flags(state.outer_rect, *state.widget) & .OVER) != 0 {
        notches := take_wheel_notches();
        movement := notches * cast(float64) state.theme.mouse_wheel_increment * cast(float64) dimension;
        scroll_value.* = clamp(cast(float) (cast(float64) scroll_value.* - movement), 0.0, max_scroll);
    }

    r := state.bar_rect;
    if r.w > 0 {
        theme := state.theme;

        rounded_rectangle(r, theme.scrollbar_background.shape, theme.scrollbar_background.color, theme.scrollbar_background.frame_color);
        ratio := 1.0;
        if extent > 0 ratio = min(1.0, visible / extent);

        nib := r;

        if horizontal {
            nib.w = min(r.w, max(theme.minimum_nib_size, r.w * ratio));
            if max_scroll > 0 nib.x += scroll_value.* / max_scroll * (r.w - nib.w);
            nib = shrink_y(nib, r.h * theme.nib_margin);
        } else {
            nib.h = min(r.h, max(theme.minimum_nib_size, r.h * ratio));
            if max_scroll > 0 nib.y += scroll_value.* / max_scroll * (r.h - nib.h);
            nib = shrink_x(nib, r.w * theme.nib_margin);
        }

        pressed, b, released := button(nib, "", *theme.scrollbar_nib_theme, cast(s64) state);

        state.nib_state = b.*;
        if pressed state.dragging = true;
        if released state.dragging = false;

        state.teleporting = false;
        if (get_status_flags(r, *state.widget) & .PRESSED) != 0 {
            state.dragging = true;
            state.teleporting = true;
            click_consumed = true;
        }

        if state.dragging {
            if horizontal && r.w > nib.w scroll_value.* = saturate((mouse_position.x - r.x - nib.w / 2) / (r.w - nib.w)) * max_scroll;
            if !horizontal && r.h > nib.h scroll_value.* = saturate((mouse_position.y - r.y - nib.h / 2) / (r.h - nib.h)) * max_scroll;
            if (ui_get_button_state(.MOUSE_BUTTON_LEFT) & .DOWN) == 0 state.dragging = false;
        }
    }
}

append_decimal :: (builder: *String_Builder, value: float64, decimals: s32) {
    Num_Text.append_number(builder, value, decimals);
}

parse_decimal :: (text: string) -> (success: bool, value: float64) {
    return Num_Text.parse_number(text);
}

number_default: Number_Input_Theme;

Num_Text :: #import "internal/numerics/number_text";

slider_deactivate :: (widget: *Active_Widget) {
    state := cast(*Slider_State) widget;
    state.sliding = false;
    state.inputting_text = false;
}
