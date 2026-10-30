// Layout: one measure pass (children's natural sizes, bottom up) and one arrange pass (rectangles,
// top down). Rows and columns share leftover space by `grow`; scroll containers give their content
// room to be larger than the viewport and shift it by the scroll offset.

// Measure and arrange the whole tree for the window's size. `end_frame` calls it; call it again
// only after changing a node by hand.
layout_tree :: (ui: *UI_State) {
    root := ui.root;

    measure(ui, root);
    root.rect = .{ 0, 0, ui.size.x, ui.size.y };
    root.clip = root.rect;
    arrange_children(ui, root);
}

#scope_file

pad_x :: (style: Style) -> float {
    return 2 * (style.padding + style.padding_x);
}

pad_y :: (style: Style) -> float {
    return 2 * (style.padding + style.padding_y);
}

measure :: (ui: *UI_State, node: *Node) {
    style := node.style;
    size := Vector2.{ 0, 0 };

    if node.kind == .TEXT {
        metrics := font_metrics(*ui.text, node.text_style.font, node.text_style.size);
        size.x = text_width(*ui.text, node.text, node.text_style) + pad_x(style);
        size.y = metrics.line_height + pad_y(style);
    } else if node.kind == .LEAF {
        size = node.paint_size;
    } else {
        for node.children measure(ui, it);

        count := node.children.count;

        if node.axis == .STACK {
            for node.children {
                size.x = max(size.x, it.natural.x);
                size.y = max(size.y, it.natural.y);
            }
        } else if node.axis == .HORIZONTAL {
            for node.children {
                size.x += it.natural.x;
                size.y = max(size.y, it.natural.y);
            }

            if count > 1 size.x += style.gap * (count - 1);
        } else {
            for node.children {
                size.y += it.natural.y;
                size.x = max(size.x, it.natural.x);
            }

            if count > 1 size.y += style.gap * (count - 1);
        }

        // A scroll container is as large as its content, but never forces its parent to be.
        node.content = size;

        if node.scrolls {
            if node.axis == .HORIZONTAL size.x = 0;
            else size.y = 0;
        }

        size.x += pad_x(style);
        size.y += pad_y(style);
    }

    if style.width > 0 size.x = style.width;
    if style.height > 0 size.y = style.height;

    size.x = max(size.x, style.min_width);
    size.y = max(size.y, style.min_height);
    node.natural = size;
}

main_size :: (axis: Axis, v: Vector2) -> float {
    if axis == .HORIZONTAL return v.x;
    return v.y;
}

cross_size :: (axis: Axis, v: Vector2) -> float {
    if axis == .HORIZONTAL return v.y;
    return v.x;
}

// Place `node` at `rect`, clipped to `clip`, then its children.
arrange :: (ui: *UI_State, node: *Node, rect: Rect, clip: Rect) {
    node.rect = rect;
    node.clip = intersect(rect, clip);
    arrange_children(ui, node);
}

arrange_children :: (ui: *UI_State, node: *Node) {
    style := node.style;
    count := node.children.count;

    if !count return;

    pad_left := style.padding + style.padding_x;
    pad_top := style.padding + style.padding_y;
    inner := Rect.{ node.rect.x + pad_left, node.rect.y + pad_top, max(node.rect.w - pad_x(style), 0), max(node.rect.h - pad_y(style), 0) };

    child_clip := node.clip;

    if style.clip || node.scrolls child_clip = intersect(inner_clip(node), node.clip);

    if node.axis == .STACK {
        for child: node.children {
            rect := inner;
            if style.align != .STRETCH rect = aligned_in(inner, child.natural, style.align, style.align);
            arrange(ui, child, rect, child_clip);
        }

        return;
    }

    horizontal := node.axis == .HORIZONTAL;

    // A scroll container lays children out in a space as large as its content.
    space := inner;
    scroll_offset := 0.0;

    if node.scrolls {
        content := node.content;

        if horizontal {
            max_scroll := max(content.x - inner.w, 0);

            node.scroll_x = clamp(node.scroll_x, 0, max_scroll);
            scroll_offset = node.scroll_x;
            space.w = max(content.x, inner.w);
            space.x -= scroll_offset;
        } else {
            max_scroll := max(content.y - inner.h, 0);

            node.scroll_y = clamp(node.scroll_y, 0, max_scroll);
            scroll_offset = node.scroll_y;
            space.h = max(content.y, inner.h);
            space.y -= scroll_offset;
        }
    }

    total_natural := 0.0;
    total_grow := 0.0;

    for node.children {
        total_natural += main_size(node.axis, it.natural);
        total_grow += it.style.grow;
    }

    gap := style.gap;
    main_extent := ifx horizontal then space.w else space.h;
    cross_extent := ifx horizontal then space.h else space.w;
    free := main_extent - total_natural - gap * (count - 1);

    offset := 0.0;
    extra_gap := 0.0;

    if free > 0 && total_grow == 0 {
        if style.justify == .CENTER offset = free / 2;
        else if style.justify == .END offset = free;
        else if style.justify == .SPACE_BETWEEN && count > 1 extra_gap = free / (count - 1);
    }

    position := ifx horizontal then space.x + offset else space.y + offset;

    for child: node.children {
        main := main_size(node.axis, child.natural);

        if free > 0 && total_grow > 0 && child.style.grow > 0 {
            main += free * child.style.grow / total_grow;
        }

        // Overflowing content shrinks the children that asked to grow.
        if free < 0 && child.style.grow > 0 && total_grow > 0 {
            main = max(main + free * child.style.grow / total_grow, 0);
        }

        cross_natural := cross_size(node.axis, child.natural);
        fixed_cross := ifx horizontal then child.style.height > 0 else child.style.width > 0;
        cross := cross_natural;

        if style.align == .STRETCH && !fixed_cross cross = cross_extent;

        cross_start := ifx horizontal then space.y else space.x;

        if style.align == .CENTER cross_start += (cross_extent - cross) / 2;
        else if style.align == .END cross_start += cross_extent - cross;

        rect: Rect;

        if horizontal {
            rect = .{ position, cross_start, main, cross };
        } else {
            rect = .{ cross_start, position, cross, main };
        }

        arrange(ui, child, rect, child_clip);
        position += main + gap + extra_gap;
    }
}

inner_clip :: (node: *Node) -> Rect {
    return node.rect;
}

// A rectangle of `size` placed inside `outer` with the given alignments.
aligned_in :: (outer: Rect, size: Vector2, horizontal: Align, vertical: Align) -> Rect {
    rect := Rect.{ outer.x, outer.y, size.x, size.y };

    if horizontal == .CENTER rect.x += (outer.w - size.x) / 2;
    else if horizontal == .END rect.x += outer.w - size.x;
    else if horizontal == .STRETCH rect.w = outer.w;

    if vertical == .CENTER rect.y += (outer.h - size.y) / 2;
    else if vertical == .END rect.y += outer.h - size.y;
    else if vertical == .STRETCH rect.h = outer.h;

    return rect;
}
