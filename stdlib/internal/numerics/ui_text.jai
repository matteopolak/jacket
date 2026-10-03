em :: (x: float, font: *Font) -> float {
    if x == 0 return 0;
    assert(font != null, "em requires a font");
    return x * cast(float) font.em_width;
}

text_size :: (font: *Font, text: string) -> (width: float, height: float) {
    assert(draw_procs.prepare_text != null, "text preparation callback required");
    width = cast(float) draw_procs.prepare_text(font, text);
    assert(font != null, "text_size requires a font");
    height = cast(float) font.character_height;

    return width, height;
}

default_text_baseline :: (font: *Font, height: float) -> float {
    assert(font != null, "default_text_baseline requires a font");
    ascender := abs(cast(float) font.max_ascender);
    descender := abs(cast(float) font.max_descender);
    offset := (height - ascender - descender) / 2 + cast(float) font.y_offset_for_centering;

    #if LEFT_HANDED return offset + ascender;
    else return offset + descender;
}

text_origin :: (theme: *Label_Theme, font: *Font, r: Rect, text_width: float) -> (x: float, y: float) {
    x = r.x + (r.w - text_width) / 2;
    if theme.alignment == .Left x = r.x + em(theme.alignment_pad_ems, font);
    if theme.alignment == .Right x = r.x + r.w - text_width - em(theme.alignment_pad_ems, font);
    if !isinf(theme.text_baseline_vertical_position) y = r.y + r.h * theme.text_baseline_vertical_position;
    else y = r.y + default_text_baseline(font, r.h);

    return x, y;
}

label_from_prepared_text :: (r: Rect, font: *Font, theme: *Label_Theme, do_scissor := true) {
    assert(draw_procs.draw_prepared_text != null, "text drawing callback required");
    if do_scissor push_scissor(r);
    draw_procs.draw_prepared_text(font, cast(s64) pixel(r.x), cast(s64) pixel(r.y), theme.text_color, theme.effects);
    if do_scissor pop_scissor();
}

label :: (r: Rect, text: string, theme: *Label_Theme, do_scissor := true) -> desired_width: float {
    if text.count == 0 return 0;

    assert(draw_procs.prepare_text != null && draw_procs.draw_prepared_text != null, "text callbacks required");
    font := theme.font;
    if font == null font = default_overall_theme.font;

    desired_width = cast(float) draw_procs.prepare_text(font, text, theme.effects);
    x, y := text_origin(theme, font, r, desired_width);

    if do_scissor push_scissor(r);

    draw_procs.draw_prepared_text(font, cast(s64) pixel(x), cast(s64) pixel(y), theme.text_color, theme.effects);
    if do_scissor pop_scissor();

    return desired_width;
}

my_draw_text :: (font: *Font, x: s64, y: s64, text: string, color: Vector4, effects: Font_Effects = 0) {
    assert(draw_procs.prepare_text != null && draw_procs.draw_prepared_text != null);
    draw_procs.prepare_text(font, text, effects);
    draw_procs.draw_prepared_text(font, x, y, color, effects);
}

get_y_range :: (font: *Font, text_y: float) -> (y0: float, y1: float) {
    assert(font != null, "get_y_range requires a font");
    ascender := abs(cast(float) font.max_ascender);
    descender := abs(cast(float) font.max_descender);

    #if LEFT_HANDED return text_y - ascender, text_y + descender;
    else return text_y - descender, text_y + ascender;
}

get_selection_endpoints :: (font: *Font, text: string, effect: Font_Effects, text_x: float, start_character: int, end_character: int) -> (x0: float, x1: float) {
    assert(start_character >= 0 && end_character >= start_character && end_character <= text.count);
    assert(draw_procs.prepare_text != null);
    return text_x + cast(float) draw_procs.prepare_text(font, .{ start_character, text.data }, effect), text_x + cast(float) draw_procs.prepare_text(font, .{ end_character, text.data }, effect);
}

get_insert_point_for_cursor :: (state: *Text_Input_State, font: *Font, text: string, current_frame_text_x: float, cursor_x: int, effect: Font_Effects, glyphs_prepped := false) -> int {
    assert(draw_procs.prepare_text != null);
    i := 0;
    previous_width: float = 0;

    while i < text.count {
        next := seek_right_character(text, i);
        width := cast(float) draw_procs.prepare_text(font, .{ next, text.data }, effect);

        if cast(float) cursor_x - current_frame_text_x < (previous_width + width) / 2 return i;

        i = next;
        previous_width = width;
    }

    return text.count;
}

sanitize :: (s: string) -> string {
    // Replace malformed UTF-8 with U+FFFD and drop NUL; preserve valid byte sequences.
    builder: String_Builder;

    defer free_buffers(*builder);
    i := 0;
    while i < s.count {
        n := utf8_sequence_length(s, i);
        if n == 0 {
            append(*builder, "�");
            i += 1;
        } else {
            if s[i] != 0 append(*builder, s.data + i, n);
            i += n;
        }
    }

    return builder_to_string(*builder);
}

seek_left_one_utf8_character :: (s: string, _point: int) -> int {
    i := clamp(_point, 0, s.count);
    if i == 0 return 0;

    i -= 1;
    while i > 0 && (s[i] & 0xc0) == 0x80 i -= 1;

    return i;
}

seek_left_one_word :: (s: string, point: int) -> int {
    i := clamp(point, 0, s.count);
    while i > 0 && is_space(s[seek_left_one_utf8_character(s, i)]) i = seek_left_one_utf8_character(s, i);
    while i > 0 && !is_space(s[seek_left_one_utf8_character(s, i)]) i = seek_left_one_utf8_character(s, i);
    return i;
}

seek_right_one_word :: (s: string, point: int) -> int {
    i := clamp(point, 0, s.count);
    while i < s.count && !is_space(s[i]) i = seek_right_character(s, i);
    while i < s.count && is_space(s[i]) i = seek_right_character(s, i);
    return i;
}

break_into_lines :: (s: string, between_lines_string := "") -> [] string {
    result: [..] string;
    start := 0;

    for byte, i: s if byte == 10 {
        array_add(*result, string.{ i - start, s.data + start });
        if between_lines_string.count > 0 array_add(*result, between_lines_string);
        start = i + 1;
    }

    array_add(*result, string.{ s.count - start, s.data + start });

    return .{ result.count, result.data };
}

clamp_insert_point :: (state: *Text_Input_State) {
    state.insert_point = clamp(state.insert_point, 0, state.text.count);
    while state.insert_point > 0 && state.insert_point < state.text.count && (state.text[state.insert_point] & 0xc0) == 0x80 state.insert_point -= 1;
}

set_input :: (using state: *Text_Input_State, _s: string) {
    set_text(state, _s);
    insert_point = text.count;
}

set_text :: (state: *Text_Input_State, s: string, replace_pre_modification_text := true) {
    cleaned := sanitize(s);

    defer free(cleaned);
    assert(cleaned.count <= state.MAX_BUFFER_SIZE, "text input capacity exceeded");
    // cleaned is separately owned, so aliasing the input buffer is safe.
    memcpy(state.input_buffer.data, cleaned.data, cleaned.count);
    state.text = .{ cleaned.count, state.input_buffer.data };
    clamp_insert_point(state);

    if replace_pre_modification_text {
        free(state.pre_modification_text);
        state.pre_modification_text = copy_string(state.text);
    }
}

clear_text :: (using state: *Text_Input_State) {
    text = .{ 0, input_buffer.data };
    insert_point = 0;
    deselect(*selection);
}

append_text :: (using state: *Text_Input_State, s: string) {
    insert_point = text.count;
    inserted := insert_text(state, s);
    if !inserted {
        context.logger("text input capacity exceeded\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "text input capacity exceeded");
    }
}

insert_text :: (using state: *Text_Input_State, s: string) -> bool {
    cleaned := sanitize(s);

    defer free(cleaned);
    lo, hi := selection_range(state);
    if text.count - (hi - lo) + cleaned.count > MAX_BUFFER_SIZE return false;

    old_count := text.count;
    tail := old_count - hi;
    // Moving backward handles overlapping source and destination ranges.
    i := tail;

    while i > 0 {
        i -= 1;
        input_buffer[lo + cleaned.count + i] = input_buffer[hi + i];
    }

    memcpy(input_buffer.data + lo, cleaned.data, cleaned.count);
    text = .{ old_count - (hi - lo) + cleaned.count, input_buffer.data };
    insert_point = lo + cleaned.count;
    deselect(*selection);
    pending_result_actions |= .TEXT_MODIFIED;

    return true;
}

get_result :: (using state: *Text_Input_State) -> string {
    return text;
}

reset :: (using state: *Text_Input_State) {
    clear_text(state);
    pending_result_actions = 0;
    active = false;
    active_widget_remove(*widget);
}

activate :: (using state: *Text_Input_State) {
    active_widget_deactivate_all(*widget);
    active = true;
    free(last_saved_input);
    last_saved_input = copy_string(text);
    active_widget_add(*widget);
}

set_last_inert_text :: (state: *Text_Input_State, text: string) {
    free(state.last_inert_text);
    state.last_inert_text = copy_string(text);
}

start_selecting :: (using state: *Text_Input_State) {
    if selection.active return;

    selection.active = true;
    selection.start_line = 0;
    selection.end_line = 0;
    selection.start_character = insert_point;
    selection.end_character = insert_point;
}

deselect :: (using selection: *Selection) {
    active = false;
    start_line = -1;
    end_line = -1;
}

selection_was_modified :: (using selection: *Selection) {
    active = start_character != end_character || start_line != end_line;
}

select_all :: (using state: *Text_Input_State) {
    selection = .{ 0, 0, 0, text.count, true };
    insert_point = text.count;
}

copy_selection_to_clipboard :: (using state: *Text_Input_State, cut := false) {
    lo, hi := selection_range(state);
    if lo == hi return;

    clipboard_write(string.{ hi - lo, text.data + lo });

    if cut erase_selection(state);
}

insert_clipboard_at_cursor :: (using state: *Text_Input_State) {
    clipboard := clipboard_read();

    defer free(clipboard);
    if !insert_text(state, clipboard) report_native_failure("Clipboard text exceeds input capacity");
}

move_cursor_left :: (using state: *Text_Input_State, want_selection := false) {
    move_cursor(state, seek_left_one_utf8_character(text, insert_point), want_selection);
}

move_cursor_right :: (state: *Text_Input_State, want_selection := false) {
    move_cursor(state, seek_right_character(state.text, state.insert_point), want_selection);
}

move_cursor_left_by_word :: (using state: *Text_Input_State, want_selection := false) {
    move_cursor(state, seek_left_one_word(text, insert_point), want_selection);
}

move_cursor_right_by_word :: (using state: *Text_Input_State, want_selection := false) {
    move_cursor(state, seek_right_one_word(text, insert_point), want_selection);
}

move_cursor_to_beginning :: (state: *Text_Input_State, want_selection := false) {
    move_cursor(state, 0, want_selection);
}

move_cursor_to_end :: (state: *Text_Input_State, want_selection := false) {
    move_cursor(state, state.text.count, want_selection);
}

refresh_input_from_history :: (using state: *Text_Input_State) {
    if command_history_index >= 0 && command_history_index < command_history.count set_input(state, command_history[command_history_index]);
    else set_input(state, last_saved_input);
}

navigate_history_backward :: (using state: *Text_Input_State) {
    if command_history.count == 0 return;
    command_history_index = max(0, command_history_index - 1);
    refresh_input_from_history(state);
}

navigate_history_forward :: (using state: *Text_Input_State) {
    command_history_index = min(command_history.count, command_history_index + 1);
    refresh_input_from_history(state);
}

#if LEFT_HANDED {
    backspace_from_cursor :: (using state: *Text_Input_State, by_word := false) {
        delete_left(state, by_word);
    }

    delete_from_cursor :: (using state: *Text_Input_State, by_word := false) {
        delete_right(state, by_word);
    }
} else {
    backspace_from_cursor :: (using state: *Text_Input_State) {
        delete_left(state, false);
    }

    delete_from_cursor :: (using state: *Text_Input_State) {
        delete_right(state, false);
    }
}

press_enter :: (using state: *Text_Input_State, shift_pressed := false) {
    if shift_pressed {
        insert_user_text(state, "\n");
        return;
    }

    pending_result_actions |= .ENTERED;
    array_add(*command_history, copy_string(text));
    command_history_index = command_history.count;
    active = false;
    active_widget_remove(*widget);
}

press_escape :: (using state: *Text_Input_State) {
    pending_result_actions |= .ESCAPED;
    set_text(state, last_saved_input, false);
    active = false;
    active_widget_remove(*widget);
}

press_tab :: (using state: *Text_Input_State, shift_pressed := false) {
    tab_pressed = true;
    if auto_complete.proc == null return;

    matches, longest := auto_complete.proc(text, auto_complete.data);

    match_array = matches;
    match_length = matches.count;
    longest_match = longest;
    if matches.count == 0 return;
    if shift_pressed match_selection = (match_selection + matches.count - 1) % matches.count;
    else match_selection = (match_selection + 1) % matches.count;

    set_input(state, matches[match_selection]);
    completion_change_time = current_time;
    pending_result_actions |= .TEXT_MODIFIED;
}

text_input_deactivate :: (widget: *Active_Widget) {
    state := cast(*Text_Input_State) widget;
    state.active = false;
}

text_input_check_for_activation :: (widget: *Active_Widget, event: Event) -> bool {
    state := cast(*Text_Input_State) widget;
    return event.type == .KEYBOARD && event.key_code == .MOUSE_BUTTON_LEFT && event.key_pressed != 0 && has_focus(state.rect);
}

text_input_serial_events :: (widget: *Active_Widget, events: [] Event) {
    state := cast(*Text_Input_State) widget;
    for event: events handle_event(state, event);
}

handle_event :: (state: *Text_Input_State, event: Event) {
    if !state.active || state.disabled return;

    state.last_keypress_time = current_time;

    if event.type == .TEXT_INPUT {
        buffer: [4] u8;
        n := encode_utf8(event.utf32, *buffer);

        if n > 0 insert_user_text(state, string.{ n, buffer.data });
        else print("GetRect text input: invalid Unicode scalar rejected\n", to_standard_error = true);

        return;
    }

    if event.type != .KEYBOARD || event.key_pressed == 0 return;

    if event.key_code == {
        case .BACKSPACE; delete_left(state, event.ctrl_pressed);
        case .DELETE; delete_right(state, event.ctrl_pressed);
        case .ARROW_LEFT;
            if event.ctrl_pressed move_cursor_left_by_word(state, event.shift_pressed);
            else move_cursor_left(state, event.shift_pressed);
        case .ARROW_RIGHT;
            if event.ctrl_pressed move_cursor_right_by_word(state, event.shift_pressed);
            else move_cursor_right(state, event.shift_pressed);
        case .HOME; move_cursor_to_beginning(state, event.shift_pressed);
        case .END; move_cursor_to_end(state, event.shift_pressed);
        case .ARROW_UP; navigate_history_backward(state);
        case .ARROW_DOWN; navigate_history_forward(state);
        case .ENTER; press_enter(state, event.shift_pressed);
        case .ESCAPE; press_escape(state);
        case .TAB; press_tab(state, event.shift_pressed);
        case;
            if event.ctrl_pressed || event.cmd_meta_pressed {
                key := cast(int) event.key_code;
                if key == #char "A" select_all(state);
                if key == #char "C" copy_selection_to_clipboard(state);
                if key == #char "X" copy_selection_to_clipboard(state, true);
                if key == #char "V" insert_clipboard_at_cursor(state);
            }
    }
}

text_input :: (_r: Rect, inert_text: string, theme: *Text_Input_Theme, identifier: s64 = 0, loc := #caller_location,input_action: Text_Input_Action = 0, auto_complete: Auto_Complete = .{}) -> (result_action: Text_Result_Action, new_string: string, state: *Text_Input_State) {
    if theme == null theme = *default_overall_theme.text_input_theme;

    created: bool;
    state, created = find_or_create_state(Text_Input_State, get_hash(loc, identifier));
    state.rect = _r;
    state.auto_complete = auto_complete;
    state.deactivate_proc = text_input_deactivate;
    state.serial_events_proc = text_input_serial_events;
    state.check_for_activation_proc = text_input_check_for_activation;
    if created || (!state.active && state.last_inert_text != inert_text) {
        set_input(state, inert_text);
        set_last_inert_text(state, inert_text);
    }

    if (input_action & .ACTIVATE) != 0 || (get_status_flags(_r, *state.widget) & .PRESSED) != 0 {
        if !state.active activate(state);
        click_consumed = true;
    }

    if (input_action & .DEACTIVATE) != 0 {
        state.active = false;
        active_widget_remove(*state.widget);
    }

    result_action = state.pending_result_actions;
    state.pending_result_actions = 0;
    draw(state, _r, theme, identifier, loc);

    return result_action, state.text, state;
}

draw_text_input_background :: (r: Rect, theme: *Text_Input_Theme, state: *Text_Input_State, active := false, changed := false) {
    state.base.rect = r;
    if active state.base.pressed_factor = 1;
    else state.base.pressed_factor = 0;
    core_draw_button(r, "", *state.base, state.base.status, *theme.button_theme, null);
}

draw :: (using state: *Text_Input_State, r: Rect, theme: *Text_Input_Theme = null, identifier: s64 = 0, loc := #caller_location) {
    if theme == null theme = *default_overall_theme.text_input_theme;
    rect = r;
    draw_text_input_background(r, theme, state, active, (pending_result_actions & .TEXT_MODIFIED) != 0);
    if text.count > 0 label(r, text, *theme.label_theme);
}

#scope_module
insert_user_text :: (state: *Text_Input_State, text: string) {
    if !insert_text(state, text) {
        print("GetRect text input: rejected input because capacity was exceeded\n", to_standard_error = true);
    }
}

utf8_sequence_length :: (s: string, i: int) -> int {
    b := s[i];
    if b < 0x80 return 1;

    n := 0;
    if b >= 0xc2 && b <= 0xdf n = 2;
    else if b >= 0xe0 && b <= 0xef n = 3;
    else if b >= 0xf0 && b <= 0xf4 n = 4;
    else return 0;
    if i + n > s.count return 0;
    for k: 1..n - 1 if (s[i + k] & 0xc0) != 0x80 return 0;
    if n == 3 && ((b == 0xe0 && s[i + 1] < 0xa0) || (b == 0xed && s[i + 1] >= 0xa0)) return 0;
    if n == 4 && ((b == 0xf0 && s[i + 1] < 0x90) || (b == 0xf4 && s[i + 1] >= 0x90)) return 0;

    return n;
}

seek_right_character :: (s: string, i: int) -> int {
    if i >= s.count return s.count;
    n := utf8_sequence_length(s, max(0, i));
    return min(s.count, max(0, i) + max(1, n));
}

selection_range :: (state: *Text_Input_State) -> (int, int) {
    clamp_insert_point(state);
    if !state.selection.active return state.insert_point, state.insert_point;
    return clamp(min(state.selection.start_character, state.selection.end_character), 0, state.text.count), clamp(max(state.selection.start_character, state.selection.end_character), 0, state.text.count);
}

erase_selection :: (state: *Text_Input_State) {
    lo, hi := selection_range(state);
    if lo == hi return;

    i := hi;
    while i < state.text.count {
        state.input_buffer[lo + i - hi] = state.input_buffer[i];
        i += 1;
    }

    state.text.count -= hi - lo;
    state.insert_point = lo;
    deselect(*state.selection);
    state.pending_result_actions |= .TEXT_MODIFIED;
}

move_cursor :: (state: *Text_Input_State, point: int, want_selection: bool) {
    if want_selection start_selecting(state);
    else deselect(*state.selection);

    state.insert_point = clamp(point, 0, state.text.count);
    clamp_insert_point(state);
    if want_selection state.selection.end_character = state.insert_point;
}

delete_left :: (state: *Text_Input_State, word: bool) {
    if state.selection.active {
        erase_selection(state);
        return;
    }

    start_selecting(state);
    if word state.selection.start_character = seek_left_one_word(state.text, state.insert_point);
    else state.selection.start_character = seek_left_one_utf8_character(state.text, state.insert_point);

    erase_selection(state);
}

delete_right :: (state: *Text_Input_State, word: bool) {
    if state.selection.active {
        erase_selection(state);
        return;
    }

    start_selecting(state);
    if word state.selection.end_character = seek_right_one_word(state.text, state.insert_point);
    else state.selection.end_character = seek_right_character(state.text, state.insert_point);

    erase_selection(state);
}

encode_utf8 :: (cp: u32, buffer: *[4] u8) -> int {
    if cp == 0 || cp > 0x10ffff || (cp >= 0xd800 && cp <= 0xdfff) return 0;

    if cp < 0x80 {
        buffer.*[0] = cast(u8) cp;
        return 1;
    }

    if cp < 0x800 {
        buffer.*[0] = cast(u8) (0xc0 | (cp >> 6));
        buffer.*[1] = cast(u8) (0x80 | (cp & 63));
        return 2;
    }

    if cp < 0x10000 {
        buffer.*[0] = cast(u8) (0xe0 | (cp >> 12));
        buffer.*[1] = cast(u8) (0x80 | ((cp >> 6) & 63));
        buffer.*[2] = cast(u8) (0x80 | (cp & 63));
        return 3;
    }

    buffer.*[0] = cast(u8) (0xf0 | (cp >> 18));
    buffer.*[1] = cast(u8) (0x80 | ((cp >> 12) & 63));
    buffer.*[2] = cast(u8) (0x80 | ((cp >> 6) & 63));
    buffer.*[3] = cast(u8) (0x80 | (cp & 63));

    return 4;
}
