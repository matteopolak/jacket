// A UI without a window or a GPU, and helpers to drive and inspect it from tests.

// Options for `headless`.
Headless_Options :: struct {
    width: float = 800;
    height: float = 600;
    scale: float = 1;
    system_fonts: bool; // load the platform's fonts instead of fixed-width synthetic ones
    theme: Theme_Mode;
}

// A UI that is not attached to a window. Fonts are synthetic (every character is 0.6 of the font
// size wide) unless `system_fonts` is set, so layouts are the same on every machine. Release it
// with `free_ui`.
headless :: (options: Headless_Options = .{}) -> UI_State {
    ui: UI_State;

    init(*ui.frame_pool, 256 * 1024 * 1024);
    ui.size = .{ options.width, options.height };
    ui.theme = default_theme(options.theme);
    set_scale(*ui.text, options.scale);

    if options.system_fonts {
        load_system_fonts(*ui.text);
    } else {
        add_synthetic_font(*ui.text);
        add_synthetic_font(*ui.text);
    }

    return ui;
}

// Change the logical size of the window a UI draws into.
resize :: (ui: *UI_State, width: float, height: float) {
    ui.size = .{ width, height };
    ui.wants_frame = true;
}

// How `find` picks a node: every field that is set must match.
Find_Query :: struct {
    label: string; // the node's label
    text: string; // the text it draws
    role: Role;
    match_role: bool; // `role` counts
    index: int; // the nth match, from 0
}

// The first node matching `query`, in tree order, or null.
find :: (ui: *UI_State, query: Find_Query) -> *Node {
    skipped := 0;
    return find_in(ui.root, query, *skipped);
}

// The first node with this label.
find_label :: (ui: *UI_State, label: string) -> *Node {
    return find(ui, .{ label = label });
}

// The nth node with this role.
find_role :: (ui: *UI_State, role: Role, index: int = 0) -> *Node {
    return find(ui, .{ role = role, match_role = true, index = index });
}

// Move the pointer to the centre of `node` and click it. The next `frame` applies it and shows the result. `node` must come
// from a frame that has already laid it out.
click :: (ui: *UI_State, node: *Node) {
    if !node return;

    send_mouse_move(ui, node.rect.x + node.rect.w / 2, node.rect.y + node.rect.h / 2);
    send_mouse_down(ui);
    send_mouse_up(ui);
}

// Click at a point in the window.
click_at :: (ui: *UI_State, x: float, y: float) {
    send_mouse_move(ui, x, y);
    send_mouse_down(ui);
    send_mouse_up(ui);
}

// The tree as indented text: role, label or text, and the rectangle of each node. Useful as a
// snapshot of a layout.
layout_dump :: (ui: *UI_State) -> string {
    builder: String_Builder;
    dump_node(*builder, ui.root, 0);
    return builder_to_string(*builder,, temp);
}

#scope_file

find_in :: (node: *Node, query: Find_Query, skipped: *int) -> *Node {
    matches := true;
    if query.label.count && node.label != query.label matches = false;
    if query.text.count && node.text != query.text matches = false;
    if query.match_role && node.role != query.role matches = false;
    if !query.label.count && !query.text.count && !query.match_role matches = false;

    if matches {
        if skipped.* == query.index return node;
        skipped.* += 1;
    }

    for node.children {
        found := find_in(it, query, skipped);
        if found return found;
    }

    return null;
}

dump_node :: (builder: *String_Builder, node: *Node, depth: int) {
    for 1..depth append(builder, "  ");

    print_to_builder(builder, "% ", node.role);

    if node.text.count print_to_builder(builder, "\"%\" ", node.text);
    else if node.label.count print_to_builder(builder, "\"%\" ", node.label);

    r := node.rect;

    print_to_builder(builder, "[% % % %]\n", cast(int) round(r.x), cast(int) round(r.y), cast(int) round(r.w), cast(int) round(r.h));

    for node.children dump_node(builder, it, depth + 1);
}
