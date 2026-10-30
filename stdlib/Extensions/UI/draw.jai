// The display list and painting the tree into it. A frame's display list is a flat array of
// rectangles, glyph quads and clip changes in points; the renderer turns it into GPU draws.

// What a display list entry draws.
Draw_Kind :: enum u8 {
    RECT; // a filled, optionally rounded and bordered rectangle
    GLYPH; // a quad sampling the glyph atlas, tinted with `color`
    CLIP; // sets the clip rectangle for what follows
}

// One display list entry.
Draw_Cmd :: struct {
    kind: Draw_Kind;
    rect: Rect;
    color: Color;
    border_color: Color;
    radius: float;
    border: float;
    uv: Vector4; // u0, v0, u1, v1 for a glyph
}

// The display list of a frame plus the clip stack used while building it.
Draw_List :: struct {
    cmds: [..] Draw_Cmd;
    clips: [..] Rect;
}

// Release the display list.
free_draw_list :: (list: *Draw_List) {
    array_free(list.cmds);
    array_free(list.clips);
}

// Empty the display list for a new frame.
reset_draw_list :: (list: *Draw_List) {
    array_reset(*list.cmds);
    array_reset(*list.clips);
}

// Fill `rect` with `color`; `radius` rounds the corners and `border` draws an outline inside.
draw_rect :: (ui: *UI_State, rect: Rect, color: Color, radius: float = 0, border: float = 0, border_color: Color = .{}) {
    if rect.w <= 0 || rect.h <= 0 return;
    if color.w <= 0 && (border <= 0 || border_color.w <= 0) return;

    array_add(*ui.draw.cmds, .{ kind = .RECT, rect = rect, color = color, border_color = border_color, radius = radius, border = border });
}

// Draw one line of `text` with its top left at (`x`, `y`) and return its width in points.
draw_text :: (ui: *UI_State, x: float, y: float, text: string, style: Text_Style) -> float {
    metrics := font_metrics(*ui.text, style.font, style.size);
    return draw_text_baseline(ui, x, y + metrics.ascent, text, style);
}

// Draw one line of `text` with its baseline at `baseline`, returning its width in points.
draw_text_baseline :: (ui: *UI_State, x: float, baseline: float, text: string, style: Text_Style) -> float {
    scale := ui.text.scale;
    pen := x;
    i := 0;

    while i < text.count {
        code, size := decode_at(text, i);

        i += size;

        if code == #char "\t" {
            pen += 4 * glyph_advance(*ui.text, #char " ", style);
            continue;
        }

        if code < 32 continue;

        glyph := get_glyph(*ui.text, code, style.font, style.size);

        if glyph.visible {
            px := round(pen * scale + glyph.left);
            py := round(baseline * scale - glyph.top);
            rect := Rect.{ px / scale, py / scale, glyph.width / scale, glyph.height / scale };
            array_add(*ui.draw.cmds, .{ kind = .GLYPH, rect = rect, color = style.color, uv = .{ glyph.u0, glyph.v0, glyph.u1, glyph.v1 } });
        }

        pen += glyph_advance(*ui.text, code, style);
    }

    return pen - x;
}

// Restrict drawing to `rect` (intersected with the clip already in force) until `pop_clip`.
push_clip :: (ui: *UI_State, rect: Rect) {
    current := rect;
    if ui.draw.clips.count current = intersect(current, ui.draw.clips[ui.draw.clips.count - 1]);

    array_add(*ui.draw.clips, current);
    array_add(*ui.draw.cmds, .{ kind = .CLIP, rect = current });
}

// End the innermost `push_clip`.
pop_clip :: (ui: *UI_State) {
    if !ui.draw.clips.count return;

    pop(*ui.draw.clips);

    // With no clip left the rectangle covers the window.
    rect := Rect.{ 0, 0, ui.size.x, ui.size.y };
    if ui.draw.clips.count rect = ui.draw.clips[ui.draw.clips.count - 1];

    array_add(*ui.draw.cmds, .{ kind = .CLIP, rect = rect });
}

#scope_module

paint_tree :: (ui: *UI_State) {
    reset_draw_list(*ui.draw);
    array_add(*ui.draw.cmds, .{ kind = .CLIP, rect = .{ 0, 0, ui.size.x, ui.size.y } });

    paint_node(ui, ui.root);
}

paint_node :: (ui: *UI_State, node: *Node) {
    style := node.style;

    if style.background.w > 0 || style.border > 0 {
        draw_rect(ui, node.rect, style.background, style.radius, style.border, style.border_color);
    }

    if node.kind == .TEXT {
        pad := style.padding;
        draw_text(ui, node.rect.x + pad + style.padding_x, node.rect.y + pad + style.padding_y, node.text, node.text_style);
    } else if node.kind == .LEAF && node.paint {
        node.paint(ui, node);
    }

    clip := style.clip || node.scrolls;
    if clip push_clip(ui, node.rect);

    for node.children paint_node(ui, it);

    if clip pop_clip(ui);
}
