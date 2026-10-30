// Editable text: the `Text_Edit` model (a buffer, a cursor, a selection and a scroll position) and
// the `text_editor` and `text_field` widgets that show and edit one.
//
// The model works without a UI, so editing is testable on its own; the widgets turn the frame's
// pointer and keyboard input into calls on it. Cursor movement is by grapheme cluster, by word and
// by line, and vertical movement keeps its column as a goal x position.

// A piece of text being edited. Zero is empty; release it with `free_text_edit`.
Text_Edit :: struct {
    buffer: Text_Buffer;
    cursor: int; // byte offset of the caret
    anchor: int; // the other end of the selection (equal to `cursor` when nothing is selected)
    goal_x: float = -1; // x position vertical movement tries to keep; -1 when not set
    scroll_x: float; // how far the view is scrolled, in points
    scroll_y: float;
    single_line: bool; // Enter is "submit" and line breaks are not accepted
    dragging: bool; // a mouse drag is selecting
    follow_version: int; // the buffer version and caret the view last scrolled to show
    follow_cursor: int = -1;
    caret_time: float64; // the last time the caret moved, so it is solid while moving
}

// A coloured range of one line, produced by an editor's highlight procedure.
Highlight_Span :: struct {
    start: int; // byte offsets within the line
    end: int;
    color: Color;
}

// A procedure that colours one line of text by appending spans (unlisted text uses the default colour).
Highlight_Proc :: #type (line: string, spans: *[..] Highlight_Span);

// A range of the text drawn with a background: search results, diagnostics, matching brackets.
Text_Mark :: struct {
    start: int; // byte offsets in the whole text
    end: int;
    color: Color;
    underline: bool; // a wavy-style underline instead of a background
}

// Options for `text_editor` and `text_field`.
Editor_Options :: struct {
    font: Font_Slot = .MONO;
    size: float; // 0: the theme's code size
    line_numbers: bool;
    highlight: Highlight_Proc;
    marks: [] Text_Mark;
    read_only: bool;
    placeholder: string; // shown in an empty field
    grow: float = 1;
    tab_width: int = 4;
}

// Release the text and its history.
//
// - edit: the text to release. It is reset to empty.
free_text_edit :: (edit: *Text_Edit) {
    free_buffer(*edit.buffer);
    edit.* = .{};
}

// Replace everything with `text` and put the caret at the start. History is cleared.
//
// - edit: the text being edited.
// - text: the new text.
set_contents :: (edit: *Text_Edit, text: string) {
    set_text(*edit.buffer, text);
    edit.cursor = 0;
    edit.anchor = 0;
    edit.goal_x = -1;
    edit.scroll_x = 0;
    edit.scroll_y = 0;
}

// The whole text, as a copy in temporary storage.
//
// - edit: the text being edited.
contents :: (edit: *Text_Edit) -> string {
    return buffer_to_string(*edit.buffer,, temp);
}

// Whether any text is selected.
//
// - edit: the text being edited.
has_selection :: (edit: *Text_Edit) -> bool {
    return edit.cursor != edit.anchor;
}

// The selected range as (start, end) byte offsets.
//
// - edit: the text being edited.
selection_range :: (edit: *Text_Edit) -> start: int, end: int {
    return min(edit.cursor, edit.anchor), max(edit.cursor, edit.anchor);
}

// The selected text, valid until the next edit.
//
// - edit: the text being edited.
selected_text :: (edit: *Text_Edit) -> string {
    start, end := selection_range(edit);
    return text_range(*edit.buffer, start, end);
}

// Where the caret is, as a zero-based line and a byte column within it.
//
// - edit: the text being edited.
caret_position :: (edit: *Text_Edit) -> line: int, column: int {
    line := line_of_offset(*edit.buffer, edit.cursor);
    return line, edit.cursor - line_start(*edit.buffer, line);
}

// Put the caret at `offset` (clamped), selecting from the old anchor when `extend` is set.
//
// - edit: the text being edited.
// - offset: the new byte offset.
// - extend: keep the anchor, so the selection grows or shrinks.
move_caret :: (edit: *Text_Edit, offset: int, extend: bool = false) {
    edit.cursor = clamp(offset, 0, edit.buffer.length);
    if !extend edit.anchor = edit.cursor;
    edit.goal_x = -1;
    seal_undo(*edit.buffer);
}

// Select all the text.
//
// - edit: the text being edited.
select_all :: (edit: *Text_Edit) {
    edit.anchor = 0;
    edit.cursor = edit.buffer.length;
    seal_undo(*edit.buffer);
}

// Replace the selection (or insert at the caret) with `text`.
//
// - edit: the text being edited.
// - text: the text to insert. Line breaks are removed in a single-line edit.
insert_text :: (edit: *Text_Edit, text: string) {
    if !text.count && !has_selection(edit) return;

    start, end := selection_range(edit);
    insertion := text;

    if edit.single_line insertion = without_breaks(text);

    if end > start {
        replace(*edit.buffer, start, end - start, insertion);
    } else {
        insert(*edit.buffer, start, insertion);
    }

    edit.cursor = start + insertion.count;
    edit.anchor = edit.cursor;
    edit.goal_x = -1;
    edit.caret_time = 0;
}

// Delete the selection. Returns whether there was one.
//
// - edit: the text being edited.
delete_selection :: (edit: *Text_Edit) -> bool {
    if !has_selection(edit) return false;

    start, end := selection_range(edit);

    erase(*edit.buffer, start, end - start);
    edit.cursor = start;
    edit.anchor = start;
    edit.goal_x = -1;

    return true;
}

// What a movement or deletion covers.
Unit :: enum u8 {
    CHARACTER; // one grapheme cluster
    WORD;
    LINE; // to the start or end of the line
    DOCUMENT;
}

// Offset one `unit` to the left (`direction` -1) or right (1) of `from`.
//
// - edit: the text being edited.
// - from: the byte offset to start from.
// - unit: how far to go.
// - direction: -1 for left, 1 for right.
offset_by :: (edit: *Text_Edit, from: int, unit: Unit, direction: int) -> int {
    buffer := *edit.buffer;

    if unit == .DOCUMENT return ifx direction < 0 then 0 else buffer.length;

    line := line_of_offset(buffer, from);
    start := line_start(buffer, line);
    end := line_end(buffer, line);

    if unit == .LINE {
        if direction < 0 {
            // First press goes to the first non-blank, a second to the true start.
            text := line_text(buffer, line);
            blank := 0;

            while blank < text.count && (text.data[blank] == #char " " || text.data[blank] == #char "\t") blank += 1;
            if from > start + blank return start + blank;

            return start;
        }

        return end;
    }

    if direction < 0 {
        if from <= start {
            if line == 0 return 0;
            return line_end(buffer, line - 1);
        }

        text := text_range(buffer, start, end);
        local := from - start;

        if unit == .WORD return start + previous_word(text, local);

        return start + previous_grapheme(text, local);
    }

    if from >= end {
        if line + 1 >= line_count(buffer) return buffer.length;
        return line_start(buffer, line + 1);
    }

    text := text_range(buffer, start, end);
    local := from - start;

    if unit == .WORD return start + next_word(text, local);

    return start + next_grapheme(text, local);
}

// Move the caret one `unit` left or right.
//
// - edit: the text being edited.
// - unit: how far to go.
// - direction: -1 for left, 1 for right.
// - extend: keep the anchor, so the selection grows or shrinks.
move_by :: (edit: *Text_Edit, unit: Unit, direction: int, extend: bool = false) {
    // With a selection, a plain left or right collapses to that side of it.
    if !extend && has_selection(edit) && unit == .CHARACTER {
        start, end := selection_range(edit);
        move_caret(edit, ifx direction < 0 then start else end);
        return;
    }

    move_caret(edit, offset_by(edit, edit.cursor, unit, direction), extend);
}

// Delete one `unit` before (`direction` -1) or after (1) the caret, or the selection if there is one.
//
// - edit: the text being edited.
// - unit: how much to delete.
// - direction: -1 for before the caret, 1 for after.
delete_by :: (edit: *Text_Edit, unit: Unit, direction: int) {
    if delete_selection(edit) return;

    target := offset_by(edit, edit.cursor, unit, direction);
    start := min(edit.cursor, target);
    end := max(edit.cursor, target);

    if end == start return;

    erase(*edit.buffer, start, end - start);
    edit.cursor = start;
    edit.anchor = start;
    edit.goal_x = -1;
}

// Move the caret `lines` lines up (negative) or down, keeping its horizontal position.
//
// - ui: the UI whose text system measures the lines.
// - edit: the text being edited.
// - lines: how many lines to move. Past either end the caret goes to the start or end of the text.
// - style: the style the lines are drawn in.
// - extend: keep the anchor, so the selection grows or shrinks.
move_vertically :: (ui: *UI_State, edit: *Text_Edit, lines: int, style: Text_Style, extend: bool = false) {
    buffer := *edit.buffer;
    line := line_of_offset(buffer, edit.cursor);
    start := line_start(buffer, line);

    if edit.goal_x < 0 {
        prefix := text_range(buffer, start, edit.cursor);
        edit.goal_x = text_width(*ui.text, prefix, style);
    }

    target := line + lines;

    if target < 0 {
        edit.cursor = 0;
    } else if target >= line_count(buffer) {
        edit.cursor = buffer.length;
    } else {
        text := line_text(buffer, target);
        edit.cursor = line_start(buffer, target) + index_at_x(*ui.text, text, style, edit.goal_x);
    }

    if !extend edit.anchor = edit.cursor;

    seal_undo(*edit.buffer);
}

// Select the word around `offset`.
//
// - edit: the text being edited.
// - offset: a byte offset in the word.
select_word_at :: (edit: *Text_Edit, offset: int) {
    buffer := *edit.buffer;
    line := line_of_offset(buffer, offset);
    start := line_start(buffer, line);
    text := text_range(buffer, start, line_end(buffer, line));
    local := offset - start;

    from := previous_word(text, min(next_word(text, local), text.count));
    to := next_word(text, from);

    edit.anchor = start + from;
    edit.cursor = start + to;
}

// Select the whole line containing `offset`, including its line break.
//
// - edit: the text being edited.
// - offset: a byte offset in the line.
select_line_at :: (edit: *Text_Edit, offset: int) {
    buffer := *edit.buffer;
    line := line_of_offset(buffer, offset);
    edit.anchor = line_start(buffer, line);
    edit.cursor = ifx line + 1 < line_count(buffer) then line_start(buffer, line + 1) else buffer.length;
}

// Undo the last step and put the caret where it belongs. Returns whether anything changed.
//
// - edit: the text being edited.
undo_edit :: (edit: *Text_Edit) -> bool {
    ok, cursor := undo(*edit.buffer);
    if ok {
        edit.cursor = min(cursor, edit.buffer.length);
        edit.anchor = edit.cursor;
    }

    return ok;
}

// Redo the last undone step. Returns whether anything changed.
//
// - edit: the text being edited.
redo_edit :: (edit: *Text_Edit) -> bool {
    ok, cursor := redo(*edit.buffer);
    if ok {
        edit.cursor = min(cursor, edit.buffer.length);
        edit.anchor = edit.cursor;
    }

    return ok;
}

// Insert a line break, repeating the current line's leading blanks.
//
// - edit: the text being edited.
insert_newline :: (edit: *Text_Edit) {
    line := line_of_offset(*edit.buffer, min(edit.cursor, edit.anchor));
    text := line_text(*edit.buffer, line);
    blank := 0;

    while blank < text.count && (text.data[blank] == #char " " || text.data[blank] == #char "\t") blank += 1;

    builder: String_Builder;

    append(*builder, "\n");
    append(*builder, string.{ blank, text.data });
    insert_text(edit, builder_to_string(*builder,, temp));
    free_buffers(*builder);
}

// Insert spaces up to the next multiple of `width` columns.
//
// - edit: the text being edited.
// - width: the tab stop spacing in columns.
insert_tab :: (edit: *Text_Edit, width: int) {
    _, column := caret_position(edit);
    count := width - column % max(width, 1);
    spaces := "        ";
    insert_text(edit, string.{ min(count, spaces.count), spaces.data });
}

// What `handle_key` did.
Key_Result :: struct {
    changed: bool; // the text changed
    submitted: bool; // Enter in a single-line edit
    handled: bool; // the key meant something to the editor
}

// Apply one KEY or TEXT input event to `edit`. `style` is the text style the lines are drawn in
// (vertical movement measures with it); `page_lines` is how many lines a page is.
//
// - ui: the UI, used for measuring and the clipboard.
// - edit: the text being edited.
// - event: the KEY or TEXT event.
// - style: the text style the lines are drawn in, used to measure vertical moves.
// - options: read-only mode and the tab width.
// - page_lines: how many lines Page Up and Page Down move.
handle_key :: (ui: *UI_State, edit: *Text_Edit, event: Input_Event, style: Text_Style, options: Editor_Options, page_lines: int) -> Key_Result {
    result: Key_Result;
    version := edit.buffer.version;

    if event.kind == .TEXT {
        if options.read_only return result;

        buffer: [4] u8;
        n := encode_utf8(event.character, buffer.data);

        insert_text(edit, string.{ n, buffer.data });
        result.handled = true;
        result.changed = edit.buffer.version != version;

        return result;
    }

    mods := event.mods;
    extend := mods.shift;
    mac := OS == .MACOS;
    word := ifx mac then mods.alt else mods.ctrl;
    line_unit := mac && mods.super;
    command := primary(mods);

    result.handled = true;

    if event.key == {
        case .ARROW_LEFT;
            move_by(edit, ifx line_unit then Unit.LINE else (ifx word then Unit.WORD else Unit.CHARACTER), -1, extend);
        case .ARROW_RIGHT;
            move_by(edit, ifx line_unit then Unit.LINE else (ifx word then Unit.WORD else Unit.CHARACTER), 1, extend);
        case .ARROW_UP;
            if mac && mods.super move_by(edit, .DOCUMENT, -1, extend);
            else if edit.single_line move_by(edit, .DOCUMENT, -1, extend);
            else move_vertically(ui, edit, -1, style, extend);
        case .ARROW_DOWN;
            if mac && mods.super move_by(edit, .DOCUMENT, 1, extend);
            else if edit.single_line move_by(edit, .DOCUMENT, 1, extend);
            else move_vertically(ui, edit, 1, style, extend);
        case .PAGE_UP;
            move_vertically(ui, edit, -page_lines, style, extend);
        case .PAGE_DOWN;
            move_vertically(ui, edit, page_lines, style, extend);
        case .HOME;
            move_by(edit, ifx mods.ctrl then Unit.DOCUMENT else Unit.LINE, -1, extend);
        case .END;
            move_by(edit, ifx mods.ctrl then Unit.DOCUMENT else Unit.LINE, 1, extend);
        case .BACKSPACE;
            if options.read_only return result;

            delete_by(edit, ifx line_unit then Unit.LINE else (ifx word then Unit.WORD else Unit.CHARACTER), -1);
        case .DELETE;
            if options.read_only return result;

            delete_by(edit, ifx word then Unit.WORD else Unit.CHARACTER, 1);
        case .ENTER;
            if edit.single_line {
                result.submitted = true;
            } else if !options.read_only {
                insert_newline(edit);
            }
        case .TAB;
            if edit.single_line || options.read_only {
                result.handled = false;
            } else {
                insert_tab(edit, options.tab_width);
            }
        case .ESCAPE;
            if has_selection(edit) move_caret(edit, edit.cursor);
            else result.handled = false;
        case;
            result.handled = false;

            if command {
                result.handled = true;

                if event.key == {
                    case cast(Key_Code) #char "A"; select_all(edit);
                    case cast(Key_Code) #char "C"; copy_selection(ui, edit);
                    case cast(Key_Code) #char "X"; if !options.read_only cut_selection(ui, edit);
                    case cast(Key_Code) #char "V"; if !options.read_only paste_clipboard(ui, edit);
                    case cast(Key_Code) #char "Z";
                        if mods.shift redo_edit(edit);
                        else undo_edit(edit);
                    case cast(Key_Code) #char "Y"; if !mac redo_edit(edit);
                    case; result.handled = false;
                }
            }
    }

    result.changed = edit.buffer.version != version;

    return result;
}

// Copy the selection to the clipboard.
//
// - ui: the UI whose clipboard hooks and text measuring to use.
// - edit: the text being edited.
copy_selection :: (ui: *UI_State, edit: *Text_Edit) {
    if !has_selection(edit) || !ui.hooks.clipboard_set return;
    ui.hooks.clipboard_set(selected_text(edit));
}

// Copy the selection to the clipboard and delete it.
//
// - ui: the UI whose clipboard hooks and text measuring to use.
// - edit: the text being edited.
cut_selection :: (ui: *UI_State, edit: *Text_Edit) {
    if !has_selection(edit) return;
    copy_selection(ui, edit);
    delete_selection(edit);
}

// Replace the selection with the clipboard's text.
//
// - ui: the UI whose clipboard hooks and text measuring to use.
// - edit: the text being edited.
paste_clipboard :: (ui: *UI_State, edit: *Text_Edit) {
    if !ui.hooks.clipboard_get return;
    text := ui.hooks.clipboard_get();
    if text.count insert_text(edit, text);
}

// A multi-line text editor over `edit`: line numbers, selection, scrolling, undo and the clipboard.
// Returns true on the frame the text changed. It fills the space it is given.
//
// - edit: the text being edited.
// - options: font, size, line numbers, highlighting, marks, read-only mode and tab width.
//
// ```jai
// if text_editor(*doc.edit, .{line_numbers = true, highlight = highlight_jai}) doc.dirty = true;
// ```
text_editor :: (edit: *Text_Edit, options: Editor_Options = .{}, loc := #caller_location) -> changed: bool {
    edit.single_line = false;
    changed, _ := edit_widget(edit, options, .TEXT_EDITOR, loc);
    return changed;
}

// A single-line text field over `edit`. Returns whether the text changed and whether Enter was
// pressed.
//
// - edit: the text being edited.
// - options: placeholder, size and read-only mode. The font is always the sans one, with no line
//   numbers and no growing.
text_field :: (edit: *Text_Edit, options: Editor_Options = .{}, loc := #caller_location) -> changed: bool, submitted: bool {
    edit.single_line = true;
    opts := options;
    opts.font = .SANS;
    opts.line_numbers = false;
    opts.grow = 0;

    return edit_widget(edit, opts, .TEXT_FIELD, loc);
}

// The visible rows of an editor as (first line, last line, line height), from its last layout.
//
// - ui: the UI whose text system measures the lines.
// - edit: the text being edited.
// - node: the editor's node, whose height says how much fits.
// - options: the editor's font and size.
visible_lines :: (ui: *UI_State, edit: *Text_Edit, node: *Node, options: Editor_Options) -> first: int, last: int, line_height: float {
    style := editor_text_style(ui, options);
    line_height := font_metrics(*ui.text, style.font, style.size).line_height;
    first := max(cast(int) (edit.scroll_y / line_height), 0);
    last := min(cast(int) ((edit.scroll_y + node.rect.h) / line_height), line_count(*edit.buffer) - 1);

    return first, last, line_height;
}

// Ask for a frame in `seconds` even if no input arrives (animation, a blinking caret).
//
// - ui: the UI to ask a frame on.
// - seconds: how long from now. An earlier wake time already asked for is kept.
wake_in :: (ui: *UI_State, seconds: float64) {
    at := ui.now + seconds;
    if ui.wake_time == 0 || at < ui.wake_time ui.wake_time = at;
}

#scope_module

Editor_Paint :: struct {
    edit: *Text_Edit;
    options: Editor_Options;
    focused: bool;
    field: bool;
}

editor_text_style :: (ui: *UI_State, options: Editor_Options) -> Text_Style {
    size := options.size;
    if size == 0 size = ifx options.font == .MONO then ui.theme.mono_size else ui.theme.font_size;
    return .{ font = options.font, size = size, color = ui.theme.text };
}

edit_widget :: (edit: *Text_Edit, options: Editor_Options, role: Role, loc: Source_Code_Location) -> changed: bool, submitted: bool {
    ui := context.ui;
    t := *ui.theme;
    field := role == .TEXT_FIELD;
    style := editor_text_style(ui, options);
    metrics := font_metrics(*ui.text, style.font, style.size);

    layout: Style;
    layout.grow = options.grow;
    layout.clip = true;

    if field {
        layout.padding_x = t.padding;
        layout.padding_y = FIELD_PADDING_Y;
        layout.height = metrics.line_height + 2 * FIELD_PADDING_Y;
        layout.min_width = FIELD_MIN_WIDTH;
        layout.radius = t.radius;
        layout.background = t.surface_raised;
        layout.border = 1;
        layout.border_color = t.outline;
    } else {
        layout.background = t.background;
    }

    node := widget(.LEAF, role, layout, loc);
    node.focusable = true;
    node.pointer = true;
    node.paint = editor_paint;

    paint := New(Editor_Paint,, frame_allocator(ui));
    paint.edit = edit;
    paint.options = options;
    node.paint_data = paint;

    events := events_of(ui, node);

    paint.focused = events.focused;
    paint.field = field;

    changed := false;
    submitted := false;
    inner := node.rect;

    if field {
        inner = .{ inner.x + t.padding, inner.y + FIELD_PADDING_Y, max(inner.w - 2 * t.padding, 0), max(inner.h - 2 * FIELD_PADDING_Y, 0) };
    }

    gutter := editor_gutter(ui, edit, options, style);
    page_lines := max(cast(int) (inner.h / metrics.line_height) - 1, 1);

    if events.focused {
        for event: ui.input.keys {
            result := handle_key(ui, edit, event, style, options, page_lines);
            changed = changed || result.changed;
            submitted = submitted || result.submitted;
            if result.handled edit.caret_time = ui.now;
        }

        // The caret blinks, so ask to be woken for the next change of phase.
        wake_in(ui, CARET_BLINK_SECONDS);
    }

    // The pointer places the caret and drags a selection.
    if ui.pressed == node.id && ui.input.mouse_down {
        text_x := ui.input.mouse_x - inner.x - gutter + edit.scroll_x;
        text_y := ui.input.mouse_y - inner.y + edit.scroll_y;
        line := clamp(cast(int) floor(text_y / metrics.line_height), 0, line_count(*edit.buffer) - 1);
        offset := line_start(*edit.buffer, line) + index_at_x(*ui.text, line_text(*edit.buffer, line), style, text_x);

        if !edit.dragging {
            edit.dragging = true;

            if ui.click_count >= 3 {
                select_line_at(edit, offset);
            } else if ui.click_count == 2 {
                select_word_at(edit, offset);
            } else {
                move_caret(edit, offset, ui.input.mods.shift);
            }
        } else if ui.click_count == 1 {
            edit.cursor = offset;
            edit.goal_x = -1;
        }

        edit.caret_time = ui.now;
        ui.wants_frame = true;
    } else {
        edit.dragging = false;
    }

    if events.hovered ui.input.cursor = .TEXT;

    // Scrolling by wheel when nothing else took it.
    if events.hovered && !field {
        edit.scroll_y = max(edit.scroll_y + ui.input.scroll_dy, 0);
        edit.scroll_x = max(edit.scroll_x + ui.input.scroll_dx, 0);
    }

    keep_caret_visible(ui, edit, inner, gutter, style, metrics, field);
    node.value = frame_string(ui, ifx field then contents(edit) else "");
    node.label = options.placeholder;

    return changed, submitted;
}

// Space for line numbers (0 when they are off).
editor_gutter :: (ui: *UI_State, edit: *Text_Edit, options: Editor_Options, style: Text_Style) -> float {
    if !options.line_numbers return 0;

    digits := 2;
    lines := line_count(*edit.buffer);

    while lines >= 100 {
        lines /= 10;
        digits += 1;
    }

    return digits * glyph_advance(*ui.text, #char "0", style) + GUTTER_PADDING;
}

keep_caret_visible :: (ui: *UI_State, edit: *Text_Edit, inner: Rect, gutter: float, style: Text_Style, metrics: Font_Metrics, field: bool) {
    if inner.w <= 0 || inner.h <= 0 return;

    line := line_of_offset(*edit.buffer, edit.cursor);
    top := line * metrics.line_height;
    view_h := inner.h;

    max_y := max(line_count(*edit.buffer) * metrics.line_height - view_h, 0);

    // Follow the caret only when it or the text moved, so wheel scrolling is not undone.
    if edit.follow_version == edit.buffer.version && edit.follow_cursor == edit.cursor {
        edit.scroll_y = clamp(edit.scroll_y, 0, max_y);
        return;
    }

    edit.follow_version = edit.buffer.version;
    edit.follow_cursor = edit.cursor;

    if top < edit.scroll_y edit.scroll_y = top;
    if top + metrics.line_height > edit.scroll_y + view_h edit.scroll_y = top + metrics.line_height - view_h;

    prefix := text_range(*edit.buffer, line_start(*edit.buffer, line), edit.cursor);
    x := text_width(*ui.text, prefix, style);
    view_w := inner.w - gutter - 8;

    if x < edit.scroll_x edit.scroll_x = max(x - SCROLL_MARGIN, 0);
    if x > edit.scroll_x + view_w edit.scroll_x = x - view_w + SCROLL_MARGIN;

    edit.scroll_y = clamp(edit.scroll_y, 0, max_y);
}

editor_paint :: (ui: *UI_State, node: *Node) {
    paint := cast(*Editor_Paint) node.paint_data;
    edit := paint.edit;
    options := paint.options;
    t := *ui.theme;
    style := editor_text_style(ui, options);
    metrics := font_metrics(*ui.text, style.font, style.size);
    rect := node.rect;
    inner := rect;

    if paint.field inner = .{ rect.x + t.padding, rect.y + FIELD_PADDING_Y, max(rect.w - 2 * t.padding, 0), max(rect.h - 2 * FIELD_PADDING_Y, 0) };

    gutter := editor_gutter(ui, edit, options, style);

    push_clip(ui, intersect(rect, node.clip));
    defer pop_clip(ui);

    first := max(cast(int) (edit.scroll_y / metrics.line_height), 0);
    last := min(cast(int) ((edit.scroll_y + inner.h) / metrics.line_height), line_count(*edit.buffer) - 1);
    sel_start, sel_end := selection_range(edit);
    text_left := inner.x + gutter - edit.scroll_x;
    spans: [..] Highlight_Span;
    spans.allocator = temp;

    caret_line := line_of_offset(*edit.buffer, edit.cursor);

    for line: first..last {
        y := inner.y + line * metrics.line_height - edit.scroll_y;
        ls := line_start(*edit.buffer, line);
        le := line_end(*edit.buffer, line);
        text := line_text(*edit.buffer, line);

        if !paint.field && line == caret_line && !has_selection(edit) && paint.focused {
            draw_rect(ui, .{ inner.x + gutter, y, inner.w - gutter, metrics.line_height }, with_alpha(t.text, 0.05));
        }

        // Marks (search results, diagnostics) behind the text.
        for mark: options.marks {
            if mark.end <= ls || mark.start > le continue;

            from := max(mark.start, ls) - ls;
            to := min(mark.end, le) - ls;
            x0 := text_left + text_width(*ui.text, string.{ from, text.data }, style);
            x1 := text_left + text_width(*ui.text, string.{ to, text.data }, style);

            if mark.underline {
                draw_rect(ui, .{ x0, y + metrics.line_height - 3, max(x1 - x0, 4), 2 }, mark.color);
            } else {
                draw_rect(ui, .{ x0, y, max(x1 - x0, 4), metrics.line_height }, mark.color);
            }
        }

        // The selection, extended a little at line ends it covers.
        if sel_end > sel_start && sel_end > ls && sel_start <= le {
            from := max(sel_start, ls) - ls;
            to := min(sel_end, le) - ls;
            x0 := text_left + text_width(*ui.text, string.{ from, text.data }, style);
            x1 := text_left + text_width(*ui.text, string.{ to, text.data }, style);

            if sel_end > le x1 += glyph_advance(*ui.text, #char " ", style) / 2;

            draw_rect(ui, .{ x0, y, max(x1 - x0, 2), metrics.line_height }, t.selection);
        }

        // The line's text, in runs of one colour.
        if options.highlight {
            array_reset(*spans);
            options.highlight(text, *spans);
        }

        x := text_left;
        position := 0;

        for span: spans {
            if span.start < position || span.end > text.count || span.end <= span.start continue;

            if span.start > position {
                x += draw_text(ui, x, y, string.{ span.start - position, text.data + position }, style);
            }

            coloured := style;
            coloured.color = span.color;
            x += draw_text(ui, x, y, string.{ span.end - span.start, text.data + span.start }, coloured);
            position = span.end;
        }

        if position < text.count {
            draw_text(ui, x, y, string.{ text.count - position, text.data + position }, style);
        }

        if options.line_numbers {
            number := tprint("%", line + 1);
            width := text_width(*ui.text, number, style);
            number_style := style;
            number_style.color = ifx line == caret_line then t.text else t.text_muted;
            draw_text(ui, inner.x + gutter - width - 10, y, number, number_style);
        }
    }

    // The placeholder of an empty field.
    if paint.field && edit.buffer.length == 0 && !paint.focused && options.placeholder.count {
        muted := style;
        muted.color = t.text_muted;
        draw_text(ui, inner.x, inner.y, options.placeholder, muted);
    }

    // The caret, blinking while the editor has focus.
    if paint.focused {
        phase := cast(int) ((ui.now - edit.caret_time) / CARET_BLINK_SECONDS);
        solid := ui.now - edit.caret_time < CARET_BLINK_SECONDS;

        if solid || phase % 2 == 0 {
            ls := line_start(*edit.buffer, caret_line);
            prefix := text_range(*edit.buffer, ls, edit.cursor);
            x := text_left + text_width(*ui.text, prefix, style);
            y := inner.y + caret_line * metrics.line_height - edit.scroll_y;

            draw_rect(ui, .{ x, y, 1.5, metrics.line_height }, t.caret);
        }
    }

    // The gutter's edge.
    if gutter > 0 {
        draw_rect(ui, .{ inner.x + gutter - 4, inner.y, 1, inner.h }, with_alpha(t.outline, 0.6));
    }

    if paint.focused && paint.field {
        draw_rect(ui, rect, .{}, t.radius, 1.5, t.accent);
    }
}

without_breaks :: (text: string) -> string {
    builder: String_Builder;

    for 0..text.count - 1 {
        c := text.data[it];
        if c == #char "\n" || c == #char "\r" append(*builder, " ");
        else append(*builder, string.{ 1, *c });
    }

    return builder_to_string(*builder,, temp);
}

// Write `code` as UTF-8 into `out` (at least 4 bytes); returns the number of bytes.
encode_utf8 :: (code: u32, out: *u8) -> int {
    if code < 0x80 {
        out[0] = cast(u8) code;
        return 1;
    }

    if code < 0x800 {
        out[0] = cast(u8) (0xC0 | (code >> 6));
        out[1] = cast(u8) (0x80 | (code & 0x3F));
        return 2;
    }

    if code < 0x10000 {
        out[0] = cast(u8) (0xE0 | (code >> 12));
        out[1] = cast(u8) (0x80 | ((code >> 6) & 0x3F));
        out[2] = cast(u8) (0x80 | (code & 0x3F));
        return 3;
    }

    out[0] = cast(u8) (0xF0 | (code >> 18));
    out[1] = cast(u8) (0x80 | ((code >> 12) & 0x3F));
    out[2] = cast(u8) (0x80 | ((code >> 6) & 0x3F));
    out[3] = cast(u8) (0x80 | (code & 0x3F));

    return 4;
}

// Layout constants of the editing widgets, in points and seconds.
FIELD_PADDING_Y :: 5;
FIELD_MIN_WIDTH :: 120;
GUTTER_PADDING :: 20;
SCROLL_MARGIN :: 20;
CARET_BLINK_SECONDS :: 0.53;
