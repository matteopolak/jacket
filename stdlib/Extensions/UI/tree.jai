// The retained, keyed widget tree and the frame cycle.
//
// A frame is `begin_frame`, the app's `build` procedure declaring widgets, then `end_frame`
// (layout). Widgets are matched with the previous frame's by id, so what a node remembers (scroll
// offset, focus, a widget's typed state slot) survives. An id comes from the call site, the parent
// and a count of repeats of that call site under the parent, or from `key(...)`.

// How a node lays out and paints.
Node_Kind :: enum u8 {
    CONTAINER; // lays out children
    TEXT; // one line of text
    LEAF; // a size and a custom `paint`
}

// A procedure that paints a node's rectangle. It may use `ui.draw` freely.
Paint_Proc :: #type (ui: *UI_State, node: *Node);

// One widget in the tree. App code reads it through `widget`'s result and the test helpers; the
// library fills it in.
Node :: struct {
    id: u64;
    parent: *Node;
    children: [..] *Node;
    seen: int; // the frame that last declared this node

    // Declared every frame.
    kind: Node_Kind;
    role: Role;
    axis: Axis;
    style: Style;
    text: string;
    text_style: Text_Style;
    label: string; // names the widget to tests and a later accessibility export
    value: string; // the widget's current value, for the same
    focusable: bool;
    key_name: string; // the text of the `key` call that named this node this frame, for warnings (debug only)
    pointer: bool; // takes mouse input (hover, press)
    scrolls: bool; // a scroll container
    paint: Paint_Proc;
    paint_data: *void; // valid while the frame's build runs and until the next frame
    paint_size: Vector2; // natural size of a LEAF

    // Computed by layout.
    natural: Vector2;
    rect: Rect;
    clip: Rect; // the visible part, after the clips of ancestors
    content: Vector2; // size of a scroll container's content

    // Kept between frames.
    scroll_x: float;
    scroll_y: float;
    warned_zero_size: bool; // a zero-size focusable has been reported
    state: *void; // see `state`
    state_free: (data: *void);
    state_size: int;
    repeats: Table(u64, int); // call sites seen under this node this frame
}

// What happened to a node this frame, from the input events routed to it.
Node_Events :: struct {
    hovered: bool;
    pressed: bool; // the primary button is down on it
    clicked: bool; // the primary button went down and up on it
    double_clicked: bool;
    focused: bool;
    focus_gained: bool;
    focus_lost: bool;
}

// All the state of one UI: its tree, fonts, input and the frame being drawn. Create one with
// `headless` or let `run_app` make it.
UI_State :: struct {
    text: Text_System;
    theme: Theme;
    root: *Node;
    nodes: Table(u64, *Node);
    stack: [..] *Node;
    frame: int;
    size: Vector2; // logical window size
    frame_pool: Flat_Pool;
    key_pending: bool;
    key_value: u64;
    key_name: string;
    input: Input_State;
    draw: Draw_List;

    // Interaction.
    hover: u64;
    pressed: u64;
    focus: u64;
    focus_previous: u64;
    pressed_button: Mouse_Button;
    click_count: int;
    last_click_time: float64;
    last_click_x: float;
    last_click_y: float;
    now: float64; // seconds, set by the platform loop

    // Asks for another frame even without input (animation, blinking caret).
    wants_frame: bool;
    wake_time: float64; // the time of the next frame that is wanted without input (0: none)
    quit: bool;
    hooks: UI_Hooks;
    outer: *UI_State; // the UI that was current when this one was pushed
}

// Platform services the library needs but does not own, set by `run_app`. Tests leave them null.
UI_Hooks :: struct {
    clipboard_get: () -> string;
    clipboard_set: (text: string);
    set_cursor: (kind: Cursor_Kind);
}

// The mouse cursor shapes widgets ask for.
Cursor_Kind :: enum u8 {
    ARROW;
    TEXT;
    RESIZE_HORIZONTAL;
    RESIZE_VERTICAL;
    HAND;
}

#add_context ui: *UI_State;

// The UI being built (`context.ui`).
current_ui :: () -> *UI_State {
    return context.ui;
}

// Release everything a UI owns.
free_ui :: (ui: *UI_State) {
    for ui.nodes free_node(it);

    deinit(*ui.nodes);
    array_free(ui.stack);
    free_text_system(*ui.text);
    free_draw_list(*ui.draw);
    free_input(*ui.input);
    fini(*ui.frame_pool);
    ui.* = .{};
}

// Start a frame: reset the per-frame arena, clear the display list and open the root container.
begin_frame :: (ui: *UI_State) {
    ui.frame += 1;
    ui.wants_frame = false;
    ui.wake_time = 0;

    reset(*ui.frame_pool);
    reset_atlas_if_full(*ui.text);
    reset_draw_list(*ui.draw);
    array_reset(*ui.stack);

    root := claim_node(ui, 1, null);
    root.kind = .CONTAINER;
    root.axis = .STACK;
    root.role = .GROUP;
    root.style = .{};
    root.rect = .{ 0, 0, ui.size.x, ui.size.y };
    ui.root = root;
    array_add(*ui.stack, root);
}

// Finish the frame: drop nodes nobody declared, lay the tree out and fix focus.
end_frame :: (ui: *UI_State) {
    while ui.stack.count > 1 end_container(ui);

    sweep_unseen(ui);
    layout_tree(ui);
    #if DEBUG_WARNINGS warn_zero_size_focusables(ui);
    paint_tree(ui);
}

// Run one frame with the app's `build` procedure.
//
// Queued input is applied first, against the previous frame's layout, and `build` reacts to it
// while it runs (`button` returns `true`, a text editor takes the typed text). A widget declared
// above the one that reacted has already been drawn from the old state, so when the frame had input
// it builds once more with the input spent: what you queued shows in the tree and the layout when
// `frame` returns. `build` therefore runs twice on such a frame, and the second run sees no clicks,
// keys or focus changes. A frame without input builds once.
frame :: (ui: *UI_State, app: *$T, build: (app: *T)) {
    push_ui(ui);
    defer pop_ui();

    had_input := ui.input.events.count > 0;

    process_input(ui);
    begin_frame(ui);
    build(app);
    end_frame(ui);
    finish_input(ui);

    if !had_input return;

    ui.focus_previous = ui.focus;
    begin_frame(ui);
    build(app);
    end_frame(ui);
    finish_input(ui);
}

// Make `ui` the current UI for the code that follows (a second window, or tests).
push_ui :: (ui: *UI_State) {
    ui.outer = context.ui;
    context.ui = ui;
}

// Restore the UI that was current before the matching `push_ui`.
pop_ui :: () {
    context.ui = context.ui.outer;
}

// Give the next widget an explicit key, for items in a list whose order changes. Without it a
// widget in a loop is identified by its position in the loop.
key :: (value: $T) {
    ui := context.ui;
    ui.key_pending = true;
    ui.key_value = hash_any(value);
    #if DEBUG_WARNINGS ui.key_name = frame_string(ui, tprint("%", value));
}

// The retained value of type `T` stored on a node, created zeroed on first use and kept until the
// node leaves the tree.
state :: (node: *Node, $T: Type) -> *T {
    if node.state && node.state_size == size_of(T) return cast(*T) node.state;

    if node.state && node.state_free node.state_free(node.state);
    else if node.state free(node.state);

    node.state = New(T);
    node.state_size = size_of(T);

    return cast(*T) node.state;
}

// Declare a widget: find or create its node, make it the next child of the open container and
// return it. This is what the library's own widgets use; a custom widget calls it too.
widget :: (kind: Node_Kind, role: Role, style: Style, loc: Source_Code_Location) -> *Node {
    ui := context.ui;
    parent := ui.stack[ui.stack.count - 1];
    node := declare_node(ui, parent, hash_location(loc));

    node.kind = kind;
    node.role = role;
    node.style = style;
    node.axis = .VERTICAL;
    node.text = "";
    node.label = "";
    node.value = "";
    node.focusable = false;
    node.pointer = false;
    node.scrolls = false;
    node.paint = null;
    node.paint_data = null;
    node.paint_size = .{};

    return node;
}

// Open a container as the current parent; `end_container` closes it. Containers written with `for` do both.
begin_container :: (axis: Axis, role: Role, style: Style, loc: Source_Code_Location) -> *Node {
    node := widget(.CONTAINER, role, style, loc);
    node.axis = axis;
    array_add(*context.ui.stack, node);
    return node;
}

// Close the innermost open container.
end_container :: (ui: *UI_State) {
    if ui.stack.count > 1 pop(*ui.stack);
}

// The allocator of the per-frame arena: memory that lives until the next frame begins.
frame_allocator :: (ui: *UI_State) -> Allocator {
    return .{ flat_pool_allocator_proc, *ui.frame_pool };
}

// A copy of `text` that lives until the next frame, for strings a node keeps.
frame_string :: (ui: *UI_State, text: string) -> string {
    if !text.count return "";

    result: string;
    result.count = text.count;
    result.data = get(*ui.frame_pool, text.count);
    memcpy(result.data, text.data, text.count);

    return result;
}

// What happened to `node` this frame.
events_of :: (ui: *UI_State, node: *Node) -> Node_Events {
    events: Node_Events;
    events.hovered = ui.hover == node.id;
    events.pressed = ui.pressed == node.id && ui.input.mouse_down;
    events.clicked = ui.input.clicked == node.id;
    events.double_clicked = ui.input.double_clicked == node.id;
    events.focused = ui.focus == node.id;
    events.focus_gained = ui.focus == node.id && ui.focus_previous != node.id;
    events.focus_lost = ui.focus_previous == node.id && ui.focus != node.id;

    return events;
}

// Give keyboard focus to `node`.
focus_node :: (ui: *UI_State, node: *Node) {
    if ui.focus == node.id return;
    ui.focus = node.id;
    ui.wants_frame = true;
}

// Ask for another frame (animation, a blinking caret).
request_frame :: () {
    context.ui.wants_frame = true;
}

// The frame's current time in seconds.
ui_time :: () -> float64 {
    return context.ui.now;
}

#scope_module

hash_location :: (loc: Source_Code_Location) -> u64 {
    h: u64 = FNV_OFFSET;
    h = (h ^ cast(u64) loc.fully_pathed_filename.data) * FNV_PRIME;
    h = (h ^ cast(u64) loc.line_number) * FNV_PRIME;
    h = (h ^ cast(u64) loc.character_number) * FNV_PRIME;

    return h;
}

hash_any :: (value: $T) -> u64 {
    h: u64 = FNV_OFFSET;

    #if T == string {
        for 0..value.count - 1 h = (h ^ value.data[it]) * FNV_PRIME;
    } else {
        bytes := cast(*u8) *value;
        for 0..size_of(T) - 1 h = (h ^ bytes[it]) * FNV_PRIME;
    }

    return h;
}

mix :: (a: u64, b: u64) -> u64 {
    h := (a ^ (b + 0x9E3779B97F4A7C15 + (a << 6) + (a >> 2))) * FNV_PRIME;
    return h ^ (h >> 29);
}

declare_node :: (ui: *UI_State, parent: *Node, site: u64) -> *Node {
    id: u64;
    key_name: string;

    if ui.key_pending {
        id = mix(parent.id, ui.key_value);
        ui.key_pending = false;
        key_name = ui.key_name;
        ui.key_name = "";
    } else {
        count := table_find_pointer(*parent.repeats, site);

        if count {
            count.* += 1;
            id = mix(mix(parent.id, site), cast(u64) count.*);
        } else {
            table_set(*parent.repeats, site, 0);
            id = mix(parent.id, site);
        }
    }

    node := claim_node(ui, id, parent);
    node.key_name = key_name;

    array_add(*parent.children, node);

    return node;
}

claim_node :: (ui: *UI_State, id: u64, parent: *Node) -> *Node {
    existing := table_find_pointer(*ui.nodes, id);
    node: *Node;

    if existing {
        node = existing.*;
    } else {
        node = New(Node);
        node.id = id;
        table_set(*ui.nodes, id, node);
    }

    node.parent = parent;
    node.seen = ui.frame;
    array_reset(*node.children);
    table_reset_keeping_memory(*node.repeats);

    return node;
}

free_node :: (node: *Node) {
    if node.state && node.state_free node.state_free(node.state);
    else if node.state free(node.state);

    array_free(node.children);
    deinit(*node.repeats);
    free(node);
}

sweep_unseen :: (ui: *UI_State) {
    dead: [..] u64;
    dead.allocator = temp;

    for node, id: ui.nodes {
        if node.seen != ui.frame array_add(*dead, id);
    }

    for id: dead {
        found := table_find_pointer(*ui.nodes, id);

        if ui.focus == id ui.focus = 0;
        if ui.hover == id ui.hover = 0;
        if ui.pressed == id ui.pressed = 0;

        free_node(found.*);
        table_remove(*ui.nodes, id);
    }
}

FNV_OFFSET :: 1469598103934665603;
FNV_PRIME :: 1099511628211;
