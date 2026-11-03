// Containers and the basic widgets: label, button, checkbox, list item, tabs, scroll and split.
//
// A container is declared with `for`: `for row(.{gap = 8}) { ... }` opens the row, runs the body
// once and closes the row, also when the body `break`s. The body is ordinary code in the
// caller's scope, so it can use the caller's locals without closures.

// An open container, returned by `row`, `column` and the other containers so `for` can close it.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for row(.{ gap = 8 }) {
//         label("a");
//         label("b");
//     }
// });
// assert(find_label(*ui, "b") != null);
// ```
Container :: struct {
    node: *Node;
}

// Close the container when its `for` body is done.
//
// - container: the container to close afterwards.
// - body: the `for` body, run once.
// - flags: the `for` flags. Unused.
for_expansion :: (container: Container, body: Code, flags: For_Flags) #expand {
    defer end_container(context.ui);
    #insert body;
}

// A container that lays its children out left to right.
//
// - style: layout and paint options for the container.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for row(.{ gap = 10 }) {
//         label("left");
//         label("right");
//     }
// });
// assert(find_label(*ui, "right").rect.x > find_label(*ui, "left").rect.x);
// ```
row :: (style: Style = .{}, loc := #caller_location) -> Container {
    return .{ begin_container(.HORIZONTAL, .GROUP, style, loc) };
}

// A container that lays its children out top to bottom.
//
// - style: layout and paint options for the container.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for column(.{ gap = 10 }) {
//         label("top");
//         label("bottom");
//     }
// });
// assert(find_label(*ui, "bottom").rect.y > find_label(*ui, "top").rect.y);
// ```
column :: (style: Style = .{}, loc := #caller_location) -> Container {
    return .{ begin_container(.VERTICAL, .GROUP, style, loc) };
}

// A container whose children overlap, each filling it (or placed by `align`).
//
// - style: layout and paint options for the container.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for stack() {
//         label("under");
//         label("over");
//     }
// });
// assert(find_label(*ui, "under").rect.y == find_label(*ui, "over").rect.y);
// ```
stack :: (style: Style = .{}, loc := #caller_location) -> Container {
    return .{ begin_container(.STACK, .GROUP, style, loc) };
}

// A column on the theme's panel colour.
//
// - style: layout and paint options. A `background` set here replaces the panel colour.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for panel(.{ padding = 12 }) {
//         label("Settings");
//     }
// });
// assert(find_label(*ui, "Settings") != null);
// ```
panel :: (style: Style = .{}, loc := #caller_location) -> Container {
    s := style;
    if s.background.w == 0 s.background = context.ui.theme.surface;
    return .{ begin_container(.VERTICAL, .GROUP, s, loc) };
}

// A column whose content scrolls vertically (horizontally with `axis = .HORIZONTAL`) when it is
// larger than the space it is given. Give it `grow = 1` or a fixed size.
//
// - style: layout and paint options for the container.
// - axis: the scrolling direction.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless(.{ width = 200, height = 100 });
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for scroll(.{ height = 100 }) {
//         for 0..19 label(tprint("line %", it));
//     }
// });
// assert(find_role(*ui, .SCROLL) != null);
// ```
scroll :: (style: Style = .{}, axis: Axis = .VERTICAL, loc := #caller_location) -> Container {
    node := begin_container(axis, .SCROLL, style, loc);
    node.scrolls = true;
    node.pointer = true;
    return .{ node };
}

// Options for text: all optional.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     label("big and quiet", .{ size = 24, muted = true });
// });
// assert(find_role(*ui, .TEXT).text_style.size == 24);
// ```
Text_Options :: struct {
    size: float; // points; 0 is the theme's size
    color: Color; // alpha 0 is the theme's text colour
    font: Font_Slot;
    grow: float;
    muted: bool; // use the theme's muted text colour
    padding: float;
}

// One line of text.
//
// - text: the text to show. It is copied for the frame.
// - options: size, colour, font and spacing.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     label("Hello");
// });
// assert(find_label(*ui, "Hello") != null);
// ```
label :: (text: string, options: Text_Options = .{}, loc := #caller_location) -> *Node {
    ui := context.ui;
    style: Style;
    style.grow = options.grow;
    style.padding = options.padding;

    node := widget(.TEXT, .TEXT, style, loc);
    node.text = frame_string(ui, text);
    node.label = node.text;
    node.text_style = resolve_text(ui, options);

    return node;
}

// Looks of a button.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     button("Save", .{ variant = .PRIMARY });
//     button("Cancel", .{ variant = .SUBTLE });
// });
// ```
Button_Variant :: enum u8 {
    NORMAL;
    PRIMARY; // the accent colour
    SUBTLE; // no background until hovered
}

// Options for `button`.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     button("Delete", .{ enabled = false });
// });
// assert(find_label(*ui, "Delete") != null);
// ```
Button_Options :: struct {
    variant: Button_Variant;
    grow: float;
    width: float;
    enabled: bool = true;
}

// A push button. Returns true on the frame it was clicked (or Enter or Space pressed while focused).
//
// - text: the caption. It is copied for the frame.
// - options: look, size and whether it is enabled.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless(.{ width = 320, height = 200 });
// defer free_ui(*ui);
// count := 0;
// build := (n: *int) {
//     for column(.{ gap = 8, padding = 16 }) {
//         label(tprint("Clicked % times", n.*));
//         if button("Add") n.* += 1;
//     }
// };
// frame(*ui, *count, build);
// click(*ui, find_label(*ui, "Add"));
// frame(*ui, *count, build);
// assert(count == 1);
// ```
button :: (text: string, options: Button_Options = .{}, loc := #caller_location) -> clicked: bool {
    ui := context.ui;
    t := *ui.theme;

    style: Style;
    style.padding_x = t.padding + 2;
    style.padding_y = t.padding / 2;
    style.radius = t.radius;
    style.grow = options.grow;
    style.width = options.width;

    node := widget(.TEXT, .BUTTON, style, loc);
    node.text = frame_string(ui, text);
    node.label = node.text;
    node.focusable = options.enabled;
    node.pointer = options.enabled;
    node.text_style = .{ size = t.font_size, color = t.text };

    events := events_of(ui, node);
    base: Color;

    if options.variant == .PRIMARY {
        base = t.accent;
        node.text_style.color = t.accent_text;
    } else if options.variant == .NORMAL {
        base = t.surface_raised;
    }

    color := base;

    if !options.enabled {
        node.text_style.color = t.text_muted;
    } else if events.pressed {
        color = ifx options.variant == .PRIMARY then mix_color(base, hex(0x000000), 0.15) else t.surface_pressed;
    } else if events.hovered {
        color = ifx options.variant == .PRIMARY then mix_color(base, hex(0xFFFFFF), 0.12) else t.surface_hover;
    }

    node.style.background = color;

    if events.focused && options.enabled {
        node.style.border = 1;
        node.style.border_color = t.accent;
    }

    clicked := events.clicked && options.enabled;
    if events.focused && options.enabled && activated_by_keyboard(ui) clicked = true;

    return clicked;
}

// A checkbox with a caption. Returns true on the frame the value changed.
//
// - value: the flag to show and toggle. It is changed in place.
// - text: a caption next to the box. Empty for none.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// on := false;
// ui := headless();
// defer free_ui(*ui);
// build := (value: *bool) {
//     checkbox(value, "Dark mode");
// };
// frame(*ui, *on, build);
// click(*ui, find_role(*ui, .CHECKBOX));
// frame(*ui, *on, build);
// assert(on);
// ```
checkbox :: (value: *bool, text: string = "", loc := #caller_location) -> changed: bool {
    ui := context.ui;
    t := *ui.theme;

    style: Style;
    style.gap = t.gap / 2;
    style.align = .CENTER;
    node := begin_container(.HORIZONTAL, .CHECKBOX, style, loc);

    defer end_container(ui);

    node.focusable = true;
    node.pointer = true;
    node.label = frame_string(ui, text);
    node.value = ifx value.* then "checked" else "unchecked";

    events := events_of(ui, node);
    changed := events.clicked || (events.focused && activated_by_keyboard(ui));

    if changed value.* = !value.*;

    box := widget(.LEAF, .CUSTOM, .{}, #location());
    box.paint_size = .{ 16, 16 };
    box.paint = checkbox_paint;
    box.paint_data = value;

    if text.count label(text);

    return changed;
}

// A row in a list or tree: highlighted when `selected`, indented by `indent` points. Returns true
// when clicked.
//
// - text: the row's text. It is copied for the frame.
// - selected: whether to draw it highlighted.
// - indent: extra space on the left, in points.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// selected := -1;
// ui := headless();
// defer free_ui(*ui);
// build := (choice: *int) {
//     for column() {
//         for 0..2 {
//             if list_item(tprint("Item %", it), choice.* == it) choice.* = it;
//         }
//     }
// };
// frame(*ui, *selected, build);
// click(*ui, find_label(*ui, "Item 1"));
// frame(*ui, *selected, build);
// assert(selected == 1);
// ```
list_item :: (text: string, selected: bool = false, indent: float = 0, loc := #caller_location) -> clicked: bool {
    ui := context.ui;
    t := *ui.theme;

    style: Style;
    style.padding_x = t.padding;
    style.padding_y = 3;
    style.radius = 3;
    node := widget(.TEXT, .LIST_ITEM, style, loc);
    node.text = frame_string(ui, text);
    node.label = node.text;
    node.pointer = true;
    node.focusable = false;
    node.style.padding_x += indent;
    node.text_style = .{ size = t.font_size, color = t.text };

    events := events_of(ui, node);

    if selected {
        node.style.background = with_alpha(t.accent, 0.35);
        node.value = "selected";
    } else if events.hovered {
        node.style.background = t.surface_hover;
    }

    return events.clicked;
}

// A horizontal rule in a column (a vertical one in a row).
//
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for column() {
//         label("above");
//         separator();
//         label("below");
//     }
// });
// assert(find_role(*ui, .SEPARATOR) != null);
// ```
separator :: (loc := #caller_location) {
    ui := context.ui;
    parent := ui.stack[ui.stack.count - 1];

    style: Style;
    style.background = ui.theme.outline;
    if parent.axis == .HORIZONTAL style.width = 1;
    else style.height = 1;

    widget(.LEAF, .SEPARATOR, style, loc);
}

// Empty space that grows to fill what is left in its row or column.
//
// - grow: its share of the leftover space.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless(.{ width = 300, height = 40 });
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for row(.{ width = 300 }) {
//         label("left");
//         spacer(); // pushes what follows to the right edge
//         label("right");
//     }
// });
// assert(find_label(*ui, "right").rect.x > 200);
// ```
spacer :: (grow: float = 1, loc := #caller_location) {
    style: Style;
    style.grow = grow;
    widget(.LEAF, .GROUP, style, loc);
}

// A row of tabs.
//
// - style: layout and paint options for the container.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for tab_bar() {
//         tab("One", true);
//         tab("Two", false);
//     }
// });
// assert(find_role(*ui, .TAB_LIST) != null);
// ```
tab_bar :: (style: Style = .{}, loc := #caller_location) -> Container {
    s := style;
    if s.background.w == 0 s.background = context.ui.theme.surface;
    s.clip = true;
    return .{ begin_container(.HORIZONTAL, .TAB_LIST, s, loc) };
}

// What a click on a tab did.
//
// ```jai
// result := Tab_Result.NONE;
// assert(result != .SELECT && result != .CLOSE);
// ```
Tab_Result :: enum u8 {
    NONE;
    SELECT; // the tab was clicked
    CLOSE; // its close button was clicked
}

// Options for `tab`.
//
// ```jai
// ui := headless();
// defer free_ui(*ui);
// frame(*ui, *ui, (u: *UI_State) {
//     for tab_bar() {
//         tab("main.jai", true, .{ dirty = true, closable = true });
//     }
// });
// ```
Tab_Options :: struct {
    dirty: bool; // unsaved changes: shown as a dot
    closable: bool;
}

// One tab in a `tab_bar`, drawn selected when `active`.
//
// - text: the tab's title. It is copied for the frame.
// - active: whether this is the selected tab.
// - options: whether it shows an unsaved dot or a close button.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// current := 0;
// ui := headless();
// defer free_ui(*ui);
// build := (selected: *int) {
//     for tab_bar() {
//         if tab("One", selected.* == 0) == .SELECT selected.* = 0;
//         if tab("Two", selected.* == 1) == .SELECT selected.* = 1;
//     }
// };
// frame(*ui, *current, build);
// click(*ui, find_label(*ui, "Two"));
// frame(*ui, *current, build);
// assert(current == 1);
// ```
tab :: (text: string, active: bool, options: Tab_Options = .{}, loc := #caller_location) -> Tab_Result {
    ui := context.ui;
    t := *ui.theme;

    style: Style;
    style.padding_x = t.padding + 2;
    style.padding_y = TAB_PADDING;
    style.gap = TAB_PADDING;
    style.align = .CENTER;
    style.background = ifx active then t.background else t.surface;
    node := begin_container(.HORIZONTAL, .TAB, style, loc);

    defer end_container(ui);

    node.pointer = true;
    node.label = frame_string(ui, text);
    node.value = ifx active then "selected" else "";

    events := events_of(ui, node);
    if !active && events.hovered node.style.background = t.surface_hover;

    label(text, .{ muted = !active });

    result := Tab_Result.NONE;
    if events.clicked result = .SELECT;

    if options.closable {
        glyph := ifx options.dirty then "●" else "×";
        close := widget(.TEXT, .BUTTON, .{ padding_x = 3 }, #location());
        close.text = glyph;
        close.label = "Close";
        close.pointer = true;
        close.text_style = .{ size = t.font_size, color = ifx options.dirty then t.text else t.text_muted };

        close_events := events_of(ui, close);
        if close_events.hovered close.text_style.color = t.text;
        if close_events.clicked result = .CLOSE;
    } else if options.dirty {
        dot := widget(.TEXT, .TEXT, .{}, #location());
        dot.text = "●";
        dot.text_style = .{ size = t.font_size, color = t.text };
    }

    return result;
}

// An open split: the two panes are visited with `for pane: split(...)`.
//
// ```jai
// width := 150.0;
// ui := headless(.{ width = 400, height = 200 });
// defer free_ui(*ui);
// frame(*ui, *width, (w: *float) {
//     for pane: split(.HORIZONTAL, w) {
//         if pane == 0 label("left pane");
//         else label("right pane");
//     }
// });
// assert(find_label(*ui, "left pane") != null);
// ```
Split :: struct {
    node: *Node;
    position: *float;
    axis: Axis;
    min_size: float;
}

// Options for `split`.
//
// ```jai
// options := Split_Options.{ min = 100, max = 300 };
// assert(options.min == 100);
// ```
Split_Options :: struct {
    min: float = 80; // the smallest the first pane may be
    max: float; // the largest the first pane may be (0: no limit)
}

// Two panes with a draggable divider. `position` is the first pane's size along `axis`
// (HORIZONTAL puts the panes side by side); the app keeps it so it can be saved. Use
// `for pane: split(.HORIZONTAL, *width) { if pane == 0 { ... } else { ... } }`.
//
// - axis: the direction the panes sit in.
// - position: the first pane's size in points. It is clamped to the options' limits and updated
//   while the divider is dragged.
// - options: the smallest and largest size of the first pane.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai
// width := 150.0;
// ui := headless(.{ width = 400, height = 200 });
// defer free_ui(*ui);
// frame(*ui, *width, (w: *float) {
//     for pane: split(.HORIZONTAL, w, .{ min = 100 }) {
//         if pane == 0 label("files");
//         else label("editor");
//     }
// });
// assert(width == 150);
// ```
split :: (axis: Axis, position: *float, options: Split_Options = .{}, loc := #caller_location) -> Split {
    node := begin_container(axis, .SPLIT, .{}, loc);
    node.style.grow = 1;

    extent := ifx axis == .HORIZONTAL then node.rect.w else node.rect.h;
    limit := ifx options.max > 0 then options.max else (ifx extent > 0 then extent - options.min else 100000.0);

    position.* = clamp(position.*, options.min, max(limit, options.min));

    return .{ node, position, axis, options.min };
}

// Visit pane 0, the divider, then pane 1; `it` is the pane number.
//
// - s: the split being visited.
// - body: the `for` body. It runs once per pane, with `it` set to the pane number.
// - flags: the `for` flags. Unused.
for_expansion :: (s: Split, body: Code, flags: For_Flags) #expand {
    ui := context.ui;

    defer end_container(ui);

    horizontal := s.axis == .HORIZONTAL;

    first_style: Style;
    first_style.clip = true;
    if horizontal first_style.width = s.position.*;
    else first_style.height = s.position.*;

    begin_container(.VERTICAL, .GROUP, first_style, #location());
    `it := 0;
    #insert body;
    end_container(ui);

    divider(s);

    begin_container(.VERTICAL, .GROUP, .{ grow = 1, clip = true }, #location());
    `it = 1;
    #insert body;
    end_container(ui);
}

// Draws and drags the divider between split panes.
//
// - s: the split the divider belongs to.
// - loc: the call site. It identifies the widget from one frame to the next, so leave the default.
//
// ```jai,no_run
// // split() draws it for you; a custom layout calls it between its own panes
// width := 150.0;
// for pane: split(.HORIZONTAL, *width) {
//     label("pane");
// }
// ```
divider :: (s: Split, loc := #caller_location) {
    ui := context.ui;
    horizontal := s.axis == .HORIZONTAL;

    style: Style;

    if horizontal style.width = DIVIDER_WIDTH;
    else style.height = DIVIDER_WIDTH;

    node := widget(.LEAF, .SEPARATOR, style, loc);
    node.pointer = true;
    node.paint = divider_paint;
    node.paint_data = null;

    events := events_of(ui, node);

    if events.pressed {
        start := ifx horizontal then ui.input.mouse_x else ui.input.mouse_y;
        origin := ifx horizontal then s.node.rect.x else s.node.rect.y;
        s.position.* = max(start - origin - 2, s.min_size);
        ui.wants_frame = true;
    }

    ui.input.cursor = ifx events.hovered || events.pressed then (ifx horizontal then Cursor_Kind.RESIZE_HORIZONTAL else Cursor_Kind.RESIZE_VERTICAL) else ui.input.cursor;
}

#scope_module

resolve_text :: (ui: *UI_State, options: Text_Options) -> Text_Style {
    style: Text_Style;
    style.font = options.font;
    style.size = options.size;

    if style.size == 0 style.size = ifx options.font == .MONO then ui.theme.mono_size else ui.theme.font_size;

    if options.color.w > 0 {
        style.color = options.color;
    } else if options.muted {
        style.color = ui.theme.text_muted;
    } else {
        style.color = ui.theme.text;
    }

    return style;
}

// Enter or Space pressed this frame: how a focused button or checkbox is activated.
activated_by_keyboard :: (ui: *UI_State) -> bool {
    for ui.input.keys {
        if it.kind == .KEY && !it.repeat && (it.key == .ENTER || it.key == .SPACEBAR) return true;
    }

    return false;
}

checkbox_paint :: (ui: *UI_State, node: *Node) {
    t := *ui.theme;
    value := cast(*bool) node.paint_data;
    r := node.rect;

    if value.* {
        draw_rect(ui, r, t.accent, 3);
        mark := Text_Style.{ size = 13, color = t.accent_text };
        width := text_width(*ui.text, "✓", mark);
        draw_text(ui, r.x + (r.w - width) / 2, r.y + (r.h - font_metrics(*ui.text, .SANS, 13).line_height) / 2, "✓", mark);
    } else {
        draw_rect(ui, r, t.surface_raised, 3, 1, t.outline);
    }
}

divider_paint :: (ui: *UI_State, node: *Node) {
    r := node.rect;
    horizontal := r.w < r.h;
    line := ifx horizontal then Rect.{ r.x + r.w / 2, r.y, 1, r.h } else Rect.{ r.x, r.y + r.h / 2, r.w, 1 };

    draw_rect(ui, line, ui.theme.outline);

    if contains(r, ui.input.mouse_x, ui.input.mouse_y) || ui.pressed == node.id {
        draw_rect(ui, r, with_alpha(ui.theme.accent, 0.25));
    }
}

mix_color :: (a: Color, b: Color, t: float) -> Color {
    return .{ a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t, a.w + (b.w - a.w) * t };
}

// Padding inside a tab, and the grab width of a split's divider, in points.
TAB_PADDING :: 6;
DIVIDER_WIDTH :: 5;
