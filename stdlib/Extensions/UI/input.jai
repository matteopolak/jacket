// Input events and how they reach widgets.
//
// The platform loop (or a test) queues `Input_Event`s with `send_*`. At the start of a frame they
// are applied against the previous frame's layout: the pointer hit-tests the tree to find the
// hovered and pressed nodes, scrolling reaches the scroll container under the pointer, and
// keyboard and text events are kept for the focused widget to read as `ui.input.keys`.

// Mouse buttons.
Mouse_Button :: enum u8 {
    LEFT;
    RIGHT;
    MIDDLE;
}

// Which modifier keys are held. `primary` is Cmd on macOS and Ctrl elsewhere.
Modifiers :: struct {
    shift: bool;
    ctrl: bool;
    alt: bool;
    super: bool; // Cmd on macOS, the Windows key elsewhere
}

// Whether the platform's main shortcut modifier is held.
primary :: (mods: Modifiers) -> bool {
    #if OS == .MACOS return mods.super;
    else return mods.ctrl;
}

// What an input event is.
Input_Kind :: enum u8 {
    MOUSE_MOVE;
    MOUSE_DOWN;
    MOUSE_UP;
    SCROLL;
    KEY; // a key press or auto-repeat, with the key in `key`
    TEXT; // typed text, as one code point in `character`
}

// A queued input event. Keys use the `Input` module's `Key_Code` (letters are their upper case ASCII).
Input_Event :: struct {
    kind: Input_Kind;
    x: float;
    y: float;
    button: Mouse_Button;
    key: Key_Code;
    repeat: bool;
    character: u32;
    mods: Modifiers;
    dx: float; // scroll amount in points
    dy: float;
}

// The pointer, modifiers and the keyboard events a frame sees.
Input_State :: struct {
    events: [..] Input_Event; // queued, not yet applied
    keys: [..] Input_Event; // this frame's KEY and TEXT events, for the focused widget
    mouse_x: float;
    mouse_y: float;
    mouse_down: bool;
    drag_start_x: float;
    drag_start_y: float;
    mods: Modifiers;
    clicked: u64; // the node a click ended on this frame
    double_clicked: u64;
    scroll_dx: float; // scrolling this frame that no scroll container took
    scroll_dy: float;
    cursor: Cursor_Kind;
}

// Queue a pointer move to (`x`, `y`) in points.
send_mouse_move :: (ui: *UI_State, x: float, y: float) {
    array_add(*ui.input.events, .{ kind = .MOUSE_MOVE, x = x, y = y });
}

// Queue a button press at the position of the last queued move.
send_mouse_down :: (ui: *UI_State, button: Mouse_Button = .LEFT) {
    array_add(*ui.input.events, .{ kind = .MOUSE_DOWN, button = button });
}

// Queue a button release at the position of the last queued move.
send_mouse_up :: (ui: *UI_State, button: Mouse_Button = .LEFT) {
    array_add(*ui.input.events, .{ kind = .MOUSE_UP, button = button });
}

// Queue wheel or trackpad scrolling; positive `dy` scrolls the content up (towards its end).
send_scroll :: (ui: *UI_State, dx: float, dy: float) {
    array_add(*ui.input.events, .{ kind = .SCROLL, dx = dx, dy = dy });
}

// Queue a key press.
send_key :: (ui: *UI_State, key: Key_Code, mods: Modifiers = .{}, repeat: bool = false) {
    array_add(*ui.input.events, .{ kind = .KEY, key = key, mods = mods, repeat = repeat });
}

// Queue typed text, one event per code point.
send_text :: (ui: *UI_State, text: string) {
    i := 0;

    while i < text.count {
        code, size := decode_at(text, i);
        array_add(*ui.input.events, .{ kind = .TEXT, character = code });
        i += size;
    }
}

#scope_module

free_input :: (input: *Input_State) {
    array_free(input.events);
    array_free(input.keys);
}

// Apply the queued events against the previous frame's layout.
process_input :: (ui: *UI_State) {
    input := *ui.input;

    ui.focus_previous = ui.focus;
    input.clicked = 0;
    input.double_clicked = 0;
    input.scroll_dx = 0;
    input.scroll_dy = 0;
    array_reset(*input.keys);

    for event: input.events {
        if event.kind == {
            case .MOUSE_MOVE;
                input.mouse_x = event.x;
                input.mouse_y = event.y;
                ui.hover = hit_test(ui, event.x, event.y);
                ui.wants_frame = true;
            case .MOUSE_DOWN;
                if event.button != .LEFT continue;

                input.mouse_down = true;
                input.drag_start_x = input.mouse_x;
                input.drag_start_y = input.mouse_y;
                ui.hover = hit_test(ui, input.mouse_x, input.mouse_y);
                ui.pressed = ui.hover;
                count_clicks(ui, input.mouse_x, input.mouse_y);

                // Focus goes to the nearest focusable node under the pointer, or nowhere.
                target := focusable_at(ui, ui.hover);

                ui.focus = ifx target then target.id else 0;
                ui.wants_frame = true;
            case .MOUSE_UP;
                if event.button != .LEFT continue;

                ui.hover = hit_test(ui, input.mouse_x, input.mouse_y);

                if ui.pressed != 0 && ui.pressed == ui.hover {
                    input.clicked = ui.pressed;
                    if ui.click_count >= 2 input.double_clicked = ui.pressed;
                }

                input.mouse_down = false;
                ui.pressed = 0;
                ui.wants_frame = true;
            case .SCROLL;
                scroll_at_pointer(ui, event.dx, event.dy);
                ui.wants_frame = true;
            case .KEY;
                input.mods = event.mods;
                array_add(*input.keys, event);
                ui.wants_frame = true;
            case .TEXT;
                array_add(*input.keys, event);
                ui.wants_frame = true;
        }
    }

    array_reset(*input.events);
}

finish_input :: (ui: *UI_State) {
    array_reset(*ui.input.keys);
    ui.input.clicked = 0;
    ui.input.double_clicked = 0;
}

count_clicks :: (ui: *UI_State, x: float, y: float) {
    close := abs(x - ui.last_click_x) < 4 && abs(y - ui.last_click_y) < 4;

    if close && ui.now - ui.last_click_time < DOUBLE_CLICK_SECONDS {
        ui.click_count += 1;
    } else {
        ui.click_count = 1;
    }

    ui.last_click_time = ui.now;
    ui.last_click_x = x;
    ui.last_click_y = y;
}

// The deepest node taking pointer input at a point, using the previous layout.
hit_test :: (ui: *UI_State, x: float, y: float) -> u64 {
    if !ui.root return 0;

    found := hit_node(ui.root, x, y);
    if found return found.id;
    return 0;
}

hit_node :: (node: *Node, x: float, y: float) -> *Node {
    if !contains(node.clip, x, y) return null;

    for < node.children {
        found := hit_node(it, x, y);
        if found return found;
    }

    if node.pointer && contains(node.rect, x, y) return node;

    return null;
}

focusable_at :: (ui: *UI_State, id: u64) -> *Node {
    if !id return null;

    found := table_find_pointer(*ui.nodes, id);
    if !found return null;

    node := found.*;

    while node {
        if node.focusable return node;
        node = node.parent;
    }

    return null;
}

// Scroll the innermost scroll container under the pointer; what it cannot take goes to the input.
scroll_at_pointer :: (ui: *UI_State, dx: float, dy: float) {
    node := ui.root;
    target: *Node;
    x := ui.input.mouse_x;
    y := ui.input.mouse_y;

    while node {
        if node.scrolls && contains(node.clip, x, y) target = node;

        next: *Node;

        for < node.children {
            if contains(it.clip, x, y) && contains(it.rect, x, y) {
                next = it;
                break;
            }
        }

        node = next;
    }

    if !target {
        ui.input.scroll_dx += dx;
        ui.input.scroll_dy += dy;
        return;
    }

    target.scroll_x += dx;
    target.scroll_y += dy;
}

// Two clicks this close in time count as a double click.
DOUBLE_CLICK_SECONDS :: 0.4;
