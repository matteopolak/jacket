// Returns `x` times the width of one em in `font`, in pixels.
//
// Returns 0 for `x == 0` without looking at `font`; any other `x` asserts that `font` is not null.
em :: (x: float, font: *Font) -> float {
    if x == 0 return 0;
    assert(font != null, "em requires a font");
    return x * cast(float) font.em_width;
}

// Returns the width and height in pixels that `text` takes in `font`.
//
// The width is measured by the draw procedures' `prepare_text` callback; the height is the font's character
// height whatever the text. Asserts that the callback and `font` are set.
text_size :: (font: *Font, text: string) -> (width: float, height: float) {
    assert(draw_procs.prepare_text != null, "text preparation callback required");
    width = cast(float) draw_procs.prepare_text(font, text);
    assert(font != null, "text_size requires a font");
    height = cast(float) font.character_height;

    return width, height;
}

// Returns the distance from the `y` edge of a rectangle `height` tall to the baseline of text in `font`, so
// that the text is centred vertically in it.
//
// Uses the font's largest ascender and descender plus its `y_offset_for_centering`.
default_text_baseline :: (font: *Font, height: float) -> float {
    assert(font != null, "default_text_baseline requires a font");
    ascender := abs(cast(float) font.max_ascender);
    descender := abs(cast(float) font.max_descender);
    offset := (height - ascender - descender) / 2 + cast(float) font.y_offset_for_centering;

    #if LEFT_HANDED return offset + ascender;
    else return offset + descender;
}

// Returns the `x` and `y` at which a label's text is drawn inside `r`.
//
// `x` follows `theme.alignment`: centred by default, or inset by `theme.alignment_pad_ems` from the left or
// right edge. `y` is `r.y` plus `theme.text_baseline_vertical_position` times `r.h` when that is finite, and
// `r.y` plus [default_text_baseline] otherwise.
text_origin :: (theme: *Label_Theme, font: *Font, r: Rect, text_width: float) -> (x: float, y: float) {
    x = r.x + (r.w - text_width) / 2;
    if theme.alignment == .Left x = r.x + em(theme.alignment_pad_ems, font);
    if theme.alignment == .Right x = r.x + r.w - text_width - em(theme.alignment_pad_ems, font);
    if !isinf(theme.text_baseline_vertical_position) y = r.y + r.h * theme.text_baseline_vertical_position;
    else y = r.y + default_text_baseline(font, r.h);

    return x, y;
}

// Draws the text last prepared for `font` with its origin at the `x` and `y` of `r`, in `theme.text_color`.
//
// No alignment is applied and `r` only supplies the position and the clip: the draw is clipped to `r` unless
// `do_scissor` is false. Asserts that the draw callback is set.
label_from_prepared_text :: (r: Rect, font: *Font, theme: *Label_Theme, do_scissor := true) {
    assert(draw_procs.draw_prepared_text != null, "text drawing callback required");
    if do_scissor push_scissor(r);
    draw_procs.draw_prepared_text(font, cast(s64) pixel(r.x), cast(s64) pixel(r.y), theme.text_color, theme.effects);
    if do_scissor pop_scissor();
}

// Draws `text` inside `r` using `theme` and returns the width in pixels the text needs.
//
// Draws nothing and returns 0 for empty text. The font is `theme.font`, else the default theme's font, else a
// bundled font sized to the height of `r`. The text is aligned by [text_origin] and clipped to `r` unless
// `do_scissor` is false.
label :: (r: Rect, text: string, theme: *Label_Theme, do_scissor := true) -> desired_width: float {
    if text.count == 0 return 0;

    assert(draw_procs.prepare_text != null && draw_procs.draw_prepared_text != null, "text callbacks required");
    font := theme.font;
    if font == null font = default_overall_theme.font;
    if font == null font = default_font_for_height(r.h);

    desired_width = cast(float) draw_procs.prepare_text(font, text, theme.effects);
    x, y := text_origin(theme, font, r, desired_width);

    if do_scissor push_scissor(r);

    draw_procs.draw_prepared_text(font, cast(s64) pixel(x), cast(s64) pixel(y), theme.text_color, theme.effects);
    if do_scissor pop_scissor();

    return desired_width;
}

// Draws `text` in `font` with its origin at pixel (`x`, `y`), without alignment or clipping.
my_draw_text :: (font: *Font, x: s64, y: s64, text: string, color: Vector4, effects: Font_Effects = 0) {
    assert(draw_procs.prepare_text != null && draw_procs.draw_prepared_text != null);
    draw_procs.prepare_text(font, text, effects);
    draw_procs.draw_prepared_text(font, x, y, color, effects);
}

// Returns the vertical extent of a line of text whose baseline is at `text_y`, as `y0 <= y1`.
//
// The extent runs from the font's largest descender to its largest ascender, so the baseline lies inside it.
get_y_range :: (font: *Font, text_y: float) -> (y0: float, y1: float) {
    assert(font != null, "get_y_range requires a font");
    ascender := abs(cast(float) font.max_ascender);
    descender := abs(cast(float) font.max_descender);

    #if LEFT_HANDED return text_y - ascender, text_y + descender;
    else return text_y - descender, text_y + ascender;
}

// Returns the `x` positions of the left and right edges of the part of `text` from byte `start_character` up
// to byte `end_character`, for text drawn from `text_x`.
//
// Both offsets are byte offsets, not character counts, and must be on character boundaries to give a sensible
// result. Asserts `0 <= start_character <= end_character <= text.count`.
get_selection_endpoints :: (font: *Font, text: string, effect: Font_Effects, text_x: float, start_character: int, end_character: int) -> (x0: float, x1: float) {
    assert(start_character >= 0 && end_character >= start_character && end_character <= text.count);
    assert(draw_procs.prepare_text != null);
    return text_x + cast(float) draw_procs.prepare_text(font, .{ start_character, text.data }, effect), text_x + cast(float) draw_procs.prepare_text(font, .{ end_character, text.data }, effect);
}

// Returns the byte offset of the character boundary nearest to the horizontal position `cursor_x`, for text
// drawn from `current_frame_text_x`.
//
// A position left of the middle of the first character gives 0, and one right of the middle of the last gives
// `text.count`. `state` and `glyphs_prepped` are not read.
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

// Returns a copy of `s` with every malformed UTF-8 byte replaced by U+FFFD and every NUL byte removed.
//
// Valid sequences are kept as they are. Overlong forms, surrogates and values above U+10FFFF count as
// malformed. The result is allocated with the context allocator and the caller frees it with `free`.
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

// Returns the byte offset of the start of the character before byte offset `_point`.
//
// `_point` is clamped to `0..s.count`, and 0 is returned at the start. Steps back over continuation bytes
// without validating the sequence.
seek_left_one_utf8_character :: (s: string, _point: int) -> int {
    i := clamp(_point, 0, s.count);
    if i == 0 return 0;

    i -= 1;
    while i > 0 && (s[i] & 0xc0) == 0x80 i -= 1;

    return i;
}

// Returns the byte offset of the start of the word before byte offset `point`.
//
// Skips whitespace to the left of `point`, then the non-whitespace run before it. Returns 0 when there is no
// earlier word.
seek_left_one_word :: (s: string, point: int) -> int {
    i := clamp(point, 0, s.count);
    while i > 0 && is_space(s[seek_left_one_utf8_character(s, i)]) i = seek_left_one_utf8_character(s, i);
    while i > 0 && !is_space(s[seek_left_one_utf8_character(s, i)]) i = seek_left_one_utf8_character(s, i);
    return i;
}

// Returns the byte offset of the start of the word after byte offset `point`.
//
// Skips the rest of the current word, then the whitespace after it. Returns `s.count` when no later word
// starts.
seek_right_one_word :: (s: string, point: int) -> int {
    i := clamp(point, 0, s.count);
    while i < s.count && !is_space(s[i]) i = seek_right_character(s, i);
    while i < s.count && is_space(s[i]) i = seek_right_character(s, i);
    return i;
}

// Splits `s` at every newline byte (10) and returns the lines without the newlines.
//
// Each line points into `s` and is not copied. A string with no newline gives one line, and a trailing
// newline gives a final empty line. A carriage return stays in its line. When `between_lines_string` is not
// empty it is inserted as its own element after every line except the last. The array is allocated with the
// context allocator and the caller frees its `data`.
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

// Moves the cursor of `state` into the text and onto a character boundary.
//
// Clamps `insert_point` to `0..text.count`, then steps back over continuation bytes to the start of the
// character it was inside.
clamp_insert_point :: (state: *Text_Input_State) {
    state.insert_point = clamp(state.insert_point, 0, state.text.count);
    while state.insert_point > 0 && state.insert_point < state.text.count && (state.text[state.insert_point] & 0xc0) == 0x80 state.insert_point -= 1;
}

// Replaces the text of `state` with `_s` and puts the cursor at the end.
//
// Sanitizes `_s` as [set_text] does, and records it as the text that [text_input] restores on Escape.
set_input :: (using state: *Text_Input_State, _s: string) {
    set_text(state, _s);
    insert_point = text.count;
}

// Replaces the text of `state` with a sanitized copy of `s`.
//
// Copies into the state's own buffer, so `s` may point into the old text. Asserts that the result fits in
// `Text_Input_State.MAX_BUFFER_SIZE` bytes. The cursor is kept but clamped to a character boundary; no
// [Text_Result_Action] is raised. When `replace_pre_modification_text` is true, `pre_modification_text`
// becomes a copy of the new text.
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

// Empties the text of `state`, puts the cursor at 0 and clears the selection.
//
// Raises no [Text_Result_Action] and leaves the history alone.
clear_text :: (using state: *Text_Input_State) {
    text = .{ 0, input_buffer.data };
    insert_point = 0;
    deselect(*selection);
}

// Inserts `s` at the end of the text of `state` with [insert_text].
//
// Logs an error and asserts when the text would exceed `Text_Input_State.MAX_BUFFER_SIZE` bytes.
append_text :: (using state: *Text_Input_State, s: string) {
    insert_point = text.count;
    inserted := insert_text(state, s);
    if !inserted {
        context.logger("text input capacity exceeded\n", context.logger_data, Log_Info.{ common_flags = .ERROR });
        assert(false, "text input capacity exceeded");
    }
}

// Inserts a sanitized copy of `s` at the cursor, replacing the selection if there is one, and returns whether
// it fit.
//
// Returns false and changes nothing when the result would exceed `Text_Input_State.MAX_BUFFER_SIZE` bytes.
// On success the cursor moves after the inserted text, the selection is cleared and `.TEXT_MODIFIED` is
// raised.
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

// Returns the text of `state`.
//
// The string points into the state's buffer and is valid until the text next changes; copy it to keep it.
get_result :: (using state: *Text_Input_State) -> string {
    return text;
}

// Empties `state`, discards its pending result actions and deactivates it.
reset :: (using state: *Text_Input_State) {
    clear_text(state);
    pending_result_actions = 0;
    active = false;
    active_widget_remove(*widget);
}

// Makes `state` the active input, deactivating other active widgets.
//
// Saves a copy of the current text as the text Escape restores and the history falls back to.
activate :: (using state: *Text_Input_State) {
    active_widget_deactivate_all(*widget);
    active = true;
    free(last_saved_input);
    last_saved_input = copy_string(text);
    active_widget_add(*widget);
}

// Records a copy of `text` as the inert text that [text_input] last saw.
//
// [text_input] compares the text it is given with this to notice that the program changed it while the input
// was inactive.
set_last_inert_text :: (state: *Text_Input_State, text: string) {
    free(state.last_inert_text);
    state.last_inert_text = copy_string(text);
}

// Starts a selection at the cursor if there is none.
//
// The new selection is empty, with both ends at the cursor's byte offset. Does nothing when a selection is
// already active.
start_selecting :: (using state: *Text_Input_State) {
    if selection.active return;

    selection.active = true;
    selection.start_line = 0;
    selection.end_line = 0;
    selection.start_character = insert_point;
    selection.end_character = insert_point;
}

// Marks `selection` as not active and resets its lines to -1; the character offsets are left as they were.
deselect :: (using selection: *Selection) {
    active = false;
    start_line = -1;
    end_line = -1;
}

// Sets `selection.active` to whether its start and end differ, in line or in character offset.
selection_was_modified :: (using selection: *Selection) {
    active = start_character != end_character || start_line != end_line;
}

// Selects the whole text of `state` and puts the cursor at the end.
select_all :: (using state: *Text_Input_State) {
    selection = .{ 0, 0, 0, text.count, true };
    insert_point = text.count;
}

// Writes the selected bytes to the clipboard, and removes them from the text when `cut` is true.
//
// Does nothing when nothing is selected.
copy_selection_to_clipboard :: (using state: *Text_Input_State, cut := false) {
    lo, hi := selection_range(state);
    if lo == hi return;

    clipboard_write(string.{ hi - lo, text.data + lo });

    if cut erase_selection(state);
}

// Inserts the clipboard text at the cursor with [insert_text], replacing the selection.
//
// Reports a native failure and leaves the text unchanged when the result would not fit.
insert_clipboard_at_cursor :: (using state: *Text_Input_State) {
    clipboard := clipboard_read();

    defer free(clipboard);
    if !insert_text(state, clipboard) report_native_failure("Clipboard text exceeds input capacity");
}

// Moves the cursor one character left, to the previous character boundary.
//
// With `want_selection` the selection is extended from where the cursor was; without it the selection is
// cleared. The same holds for the other `move_cursor_` procedures.
move_cursor_left :: (using state: *Text_Input_State, want_selection := false) {
    move_cursor(state, seek_left_one_utf8_character(text, insert_point), want_selection);
}

// Moves the cursor one character right, to the next character boundary.
//
// See [move_cursor_left] for `want_selection`.
move_cursor_right :: (state: *Text_Input_State, want_selection := false) {
    move_cursor(state, seek_right_character(state.text, state.insert_point), want_selection);
}

// Moves the cursor to the start of the previous word.
//
// See [move_cursor_left] for `want_selection`.
move_cursor_left_by_word :: (using state: *Text_Input_State, want_selection := false) {
    move_cursor(state, seek_left_one_word(text, insert_point), want_selection);
}

// Moves the cursor to the start of the next word, or to the end of the text after the last word.
//
// See [move_cursor_left] for `want_selection`.
move_cursor_right_by_word :: (using state: *Text_Input_State, want_selection := false) {
    move_cursor(state, seek_right_one_word(text, insert_point), want_selection);
}

// Moves the cursor to the start of the text.
//
// See [move_cursor_left] for `want_selection`.
move_cursor_to_beginning :: (state: *Text_Input_State, want_selection := false) {
    move_cursor(state, 0, want_selection);
}

// Moves the cursor to the end of the text.
//
// See [move_cursor_left] for `want_selection`.
move_cursor_to_end :: (state: *Text_Input_State, want_selection := false) {
    move_cursor(state, state.text.count, want_selection);
}

// Sets the text of `state` to the history entry at `command_history_index`, or to the text saved by
// [activate] when the index is outside the history.
refresh_input_from_history :: (using state: *Text_Input_State) {
    if command_history_index >= 0 && command_history_index < command_history.count set_input(state, command_history[command_history_index]);
    else set_input(state, last_saved_input);
}

// Shows the previous history entry, stopping at the oldest.
//
// Does nothing when the history is empty.
navigate_history_backward :: (using state: *Text_Input_State) {
    if command_history.count == 0 return;
    command_history_index = max(0, command_history_index - 1);
    refresh_input_from_history(state);
}

// Shows the next history entry; past the newest it restores the text saved by [activate].
navigate_history_forward :: (using state: *Text_Input_State) {
    command_history_index = min(command_history.count, command_history_index + 1);
    refresh_input_from_history(state);
}

#if LEFT_HANDED {
    // Deletes the selection, or the character before the cursor, or the word before it when `by_word` is true.
    backspace_from_cursor :: (using state: *Text_Input_State, by_word := false) {
        delete_left(state, by_word);
    }

    // Deletes the selection, or the character after the cursor, or the word after it when `by_word` is true.
    delete_from_cursor :: (using state: *Text_Input_State, by_word := false) {
        delete_right(state, by_word);
    }
} else {
    // Deletes the selection, or the character before the cursor.
    backspace_from_cursor :: (using state: *Text_Input_State) {
        delete_left(state, false);
    }

    // Deletes the selection, or the character after the cursor.
    delete_from_cursor :: (using state: *Text_Input_State) {
        delete_right(state, false);
    }
}

// Handles the Enter key: finishes the input, or inserts a newline when `shift_pressed` is true.
//
// Finishing raises `.ENTERED`, appends a copy of the text to the history, points the history index past its
// end and deactivates the input.
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

// Handles the Escape key: restores the text saved by [activate], raises `.ESCAPED` and deactivates the input.
press_escape :: (using state: *Text_Input_State) {
    pending_result_actions |= .ESCAPED;
    set_text(state, last_saved_input, false);
    active = false;
    active_widget_remove(*widget);
}

// Handles the Tab key by cycling through the completions of the state's `auto_complete`.
//
// Calls `auto_complete.proc` with the current text and replaces the text with the next match, or the previous
// one when `shift_pressed` is true, then raises `.TEXT_MODIFIED`. Leaves the text alone when there is no
// completion procedure or no match.
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

// Deactivates the [Text_Input_State] that `widget` belongs to; stored as its `deactivate_proc`.
text_input_deactivate :: (widget: *Active_Widget) {
    state := cast(*Text_Input_State) widget;
    state.active = false;
}

// Returns whether `event` is a left mouse button press while the pointer is over the input's rectangle;
// stored as the state's `check_for_activation_proc`.
text_input_check_for_activation :: (widget: *Active_Widget, event: Event) -> bool {
    state := cast(*Text_Input_State) widget;
    return event.type == .KEYBOARD && event.key_code == .MOUSE_BUTTON_LEFT && event.key_pressed != 0 && has_focus(state.rect);
}

// Passes each of `events` to [handle_event] in order; stored as the state's `serial_events_proc`.
text_input_serial_events :: (widget: *Active_Widget, events: [] Event) {
    state := cast(*Text_Input_State) widget;
    for event: events handle_event(state, event);
}

// Applies one input event to `state`; ignored while the input is inactive or disabled.
//
// A text event inserts its character as UTF-8, and one that is not a valid Unicode scalar is rejected with a
// message on standard error. Key presses act as follows:
// - Backspace and Delete: delete a character, or a word with Ctrl.
// - Left and Right: move the cursor, by word with Ctrl; Shift extends the selection.
// - Home and End: move to the start or end; Shift extends the selection.
// - Up and Down: step through the history.
// - Enter, Escape and Tab: see [press_enter], [press_escape] and [press_tab].
// - Ctrl or Cmd with A, C, X and V: select all, copy, cut and paste.
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

// Draws a single-line text input in `_r` and returns what the user did this frame.
//
// `inert_text` is the text shown while the input is inactive; when the program passes a different string
// later, the input takes it as its new text. The state is found from the call location `loc` and
// `identifier`, so inputs made from one call site need distinct identifiers. A null `theme` selects the
// default theme. `input_action` can activate or deactivate the input from code.
//
// Returns:
// - result_action: what happened since the last call: `.ENTERED`, `.ESCAPED` and `.TEXT_MODIFIED`.
// - new_string: the current text. It points into the state's buffer and is valid until the text next
//   changes.
// - state: the input's state.
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

// Draws the background of a text input in `r`, in its pressed look while `active` is true.
draw_text_input_background :: (r: Rect, theme: *Text_Input_Theme, state: *Text_Input_State, active := false, changed := false) {
    state.base.rect = r;
    if active state.base.pressed_factor = 1;
    else state.base.pressed_factor = 0;
    core_draw_button(r, "", *state.base, state.base.status, *theme.button_theme, null);
}

// Draws `state` in `r`: the background, then its text as a label.
//
// Does not read events or change the text. A null `theme` selects the default theme.
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

#scope_module

// Text whose theme names no font is drawn with the `default_font` given to `ui_init`, or with
// Noto Sans when none was given, at a pixel height that follows the rectangle it is drawn in.
DEFAULT_FONT_HEIGHT_FRACTION :: 0.6;

// The font file shipped beside the standard library (`internal/fonts`, under its own OFL licence).
BUNDLED_FONT :: #run read_bundled_font();

read_bundled_font :: () -> string {
    path := tprint("%../fonts/NotoSans-Regular.ttf", #filepath);
    data, ok := Font_File.read_entire_file(path);
    assert(ok, "GetRect could not read its default font at %", path);
    return data;
}

Font_File :: #import "File";

default_font_data: string;
default_fonts: [..] Default_Font;

Default_Font :: struct {
    pixel_height: int;
    font: *Font;
}

// The default font at the size for a rectangle `height` tall, loaded once per size. Null when
// `ui_init` has not run or the draw procedures cannot load fonts.
default_font_for_height :: (height: float) -> *Font {
    if default_font_data.count == 0 || draw_procs.get_font_at_size == null return null;

    pixel_height := max(1, cast(int) (height * DEFAULT_FONT_HEIGHT_FRACTION + 0.5));
    for default_fonts if it.pixel_height == pixel_height return it.font;

    font := draw_procs.get_font_at_size(cast([] u8) default_font_data, pixel_height);

    array_add(*default_fonts, .{ pixel_height, font });

    return font;
}
