// Containers and the basic widgets: label, button, checkbox, list item, tabs, scroll and split.
//
// A container is declared with `for`: `for row(.{gap = 8}) { ... }` opens the row, runs the body
// once and closes the row, also when the body `break`s. The body is ordinary code in the
// caller's scope, so it can use the caller's locals without closures.

// An open container, returned by `row`, `column` and the other containers so `for` can close it.
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
row :: (style: Style = .{}, loc := #caller_location) -> Container {
    return .{ begin_container(.HORIZONTAL, .GROUP, style, loc) };
}

// A container that lays its children out top to bottom.
//
// - style: layout and paint options for the container.
column :: (style: Style = .{}, loc := #caller_location) -> Container {
    return .{ begin_container(.VERTICAL, .GROUP, style, loc) };
}

// A container whose children overlap, each filling it (or placed by `align`).
//
// - style: layout and paint options for the container.
stack :: (style: Style = .{}, loc := #caller_location) -> Container {
    return .{ begin_container(.STACK, .GROUP, style, loc) };
}

// A column on the theme's panel colour.
//
// - style: layout and paint options. A `background` set here replaces the panel colour.
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
scroll :: (style: Style = .{}, axis: Axis = .VERTICAL, loc := #caller_location) -> Container {
    node := begin_container(axis, .SCROLL, style, loc);
    node.scrolls = true;
    node.pointer = true;
    return .{ node };
}

// Options for text: all optional.
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
Button_Variant :: enum u8 {
    NORMAL;
    PRIMARY; // the accent colour
    SUBTLE; // no background until hovered
}

// Options for `button`.
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
spacer :: (grow: float = 1, loc := #caller_location) {
    style: Style;
    style.grow = grow;
    widget(.LEAF, .GROUP, style, loc);
}

// A row of tabs.
//
// - style: layout and paint options for the container.
tab_bar :: (style: Style = .{}, loc := #caller_location) -> Container {
    s := style;
    if s.background.w == 0 s.background = context.ui.theme.surface;
    s.clip = true;
    return .{ begin_container(.HORIZONTAL, .TAB_LIST, s, loc) };
}

// What a click on a tab did.
Tab_Result :: enum u8 {
    NONE;
    SELECT; // the tab was clicked
    CLOSE; // its close button was clicked
}

// Options for `tab`.
Tab_Options :: struct {
    dirty: bool; // unsaved changes: shown as a dot
    closable: bool;
}

// One tab in a `tab_bar`, drawn selected when `active`.
//
// - text: the tab's title. It is copied for the frame.
// - active: whether this is the selected tab.
// - options: whether it shows an unsaved dot or a close button.
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
Split :: struct {
    node: *Node;
    position: *float;
    axis: Axis;
    min_size: float;
}

// Options for `split`.
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
