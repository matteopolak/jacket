add_node :: (treemap: *Treemap, name: string, parent: *Treemap_Node) -> *Treemap_Node {
    assert(treemap != null);

    if !treemap.root assert(parent == null);
    else {
        assert(parent != null);
        get_index(treemap, parent);
    }

    pointer, locator := B.find_and_occupy_empty_slot(*treemap.nodes);
    pointer.name = copy_string(name);
    pointer.parent = parent;
    pointer.locator = locator;
    if parent array_add(*parent.children, pointer);
    else treemap.root = pointer;

    treemap.dirty = true;

    return pointer;
}

get_index :: inline (treemap: *Treemap, node: *Treemap_Node) -> s64 {
    assert(treemap != null && node != null);
    assert(B.bucket_array_find_pointer(*treemap.nodes, node.locator) == node);
    return cast(s64) node.locator.bucket_index * 1024 + cast(s64) node.locator.slot_index;
}

recompute_if_dirty :: (treemap: *Treemap) {
    assert(treemap != null);
    if !treemap.dirty return;
    if treemap.root weigh(treemap.root);
    treemap.dirty = false;
}

init_treemap_display :: (treemap: *Treemap) -> Treemap_Display {
    assert(treemap != null);
    return .{ treemap = treemap };
}

recompute_if_dirty :: (display: *Treemap_Display, outer_size: Vector2, border_proc: Border_Proc = null, force := false) {
    assert(display != null && display.treemap != null && !display.treemap.dirty, "Treemap display requires clean weights");
    assert(outer_size.x >= 0 && outer_size.y >= 0 && Math.is_finite(outer_size.x) && Math.is_finite(outer_size.y));
    if !force && !display.dirty && display.outer_size.x == outer_size.x && display.outer_size.y == outer_size.y && display.nodes.count == display.treemap.nodes.count return;

    array_resize(*display.nodes, display.treemap.nodes.count);
    for *node: display.nodes node.* = .{};

    display.outer_size = outer_size;
    root := display.treemap.root;
    if root {
        item := *display.nodes[get_index(display.treemap, root)];
        item.node = root;
        item.size = outer_size;
        prepare_border(display, root, border_proc);
        size_children(display.treemap, root, display, border_proc);
    }

    display.dirty = false;
}

size_children :: (treemap: *Treemap, root: *Treemap_Node, display: *Treemap_Display, border_proc: Border_Proc) {
    assert(display.treemap == treemap && !treemap.dirty);
    item := display.nodes[get_index(treemap, root)];
    corner := Vector2.{ item.corner.x + item.border[2], item.corner.y + item.border[3] };
    size := Vector2.{ max(cast(float) 0, item.size.x - item.border[0] - item.border[2]), max(cast(float) 0, item.size.y - item.border[1] - item.border[3]) };

    layout_children(root.children, display, corner, size, root.size);

    for child: root.children {
        prepare_border(display, child, border_proc);
        if child.children.count > 0 size_children(treemap, child, display, border_proc);
    }
}

flush_nodes :: (children: [] *Treemap_Node, unsized_cursor: int, child_index: int, display: *Treemap_Display, root_corner: Vector2, fill_weight_total: float64, horizontal: bool, size: Vector2) -> (flushed_node_weight: float64) {
    assert(unsized_cursor >= 0 && child_index >= unsized_cursor && child_index <= children.count);
    weight: float64;
    for index: unsized_cursor..child_index - 1 weight += children[index].size;
    if child_index == unsized_cursor return 0;

    fraction: float64;
    if fill_weight_total > 0 fraction = min(cast(float64) 1, weight / fill_weight_total);

    cursor := root_corner;
    row_right := root_corner.x + size.x;
    row_top := root_corner.y + size.y;
    thickness: float;

    if horizontal thickness = cast(float) (cast(float64) root_corner.y + cast(float64) size.y * fraction) - root_corner.y;
    else thickness = cast(float) (cast(float64) root_corner.x + cast(float64) size.x * fraction) - root_corner.x;

    for index: unsized_cursor..child_index - 1 {
        node := children[index];
        item := *display.nodes[get_index(display.treemap, node)];
        item.node = node;
        item.corner = cursor;
        share: float64;
        if weight > 0 share = node.size / weight;

        if horizontal {
            item.size.y = thickness;
            next := cast(float) (cast(float64) cursor.x + cast(float64) size.x * share);
            if index == child_index - 1 next = row_right;

            item.size.x = max(cast(float) 0, next - cursor.x);
            cursor.x = next;
        } else {
            item.size.x = thickness;
            next := cast(float) (cast(float64) cursor.y + cast(float64) size.y * share);
            if index == child_index - 1 next = row_top;

            item.size.y = max(cast(float) 0, next - cursor.y);
            cursor.y = next;
        }
    }

    return weight;
}

#scope_file
#import "Basic";
B :: #import "Bucket_Array";
Sorting :: #import "Sort";
Math :: #import "Math";

descending :: (a: *Treemap_Node, b: *Treemap_Node) -> int {
    if a.size > b.size return -1;
    if a.size < b.size return 1;
    return 0;
}

weigh :: (node: *Treemap_Node) -> float64 {
    if node.children.count > 0 {
        node.size = 0;
        for child: node.children node.size += weigh(child);
        Sorting.quick_sort(node.children, descending);
    }

    assert(node.size >= 0 && Math.is_finite(node.size), "Treemap weights must be finite and nonnegative");

    return node.size;
}

prepare_border :: (display: *Treemap_Display, node: *Treemap_Node, proc: Border_Proc) {
    item := *display.nodes[get_index(display.treemap, node)];
    for *width: item.border width.* = 0;

    if proc {
        parent: *Display_Node;
        if node.parent parent = *display.nodes[get_index(display.treemap, node.parent)];
        proc(item.corner, item.size, node, parent, *item.border);
    }

    for width: item.border assert(width >= 0);

    assert(item.border[0] + item.border[2] <= item.size.x && item.border[1] + item.border[3] <= item.size.y, "Treemap borders exceed rectangle");
}

worst_ratio :: (sum: float64, smallest: float64, largest: float64, total: float64, short_side: float64, long_side: float64) -> float64 {
    if smallest <= 0 || short_side <= 0 || long_side <= 0 || total <= 0 return 1.7976931348623157e308;

    share := sum / total;
    side_ratio := short_side / long_side;
    square := share * share;
    lower := side_ratio * (smallest / total);

    if square == 0 || lower == 0 return 1.7976931348623157e308;

    return max(side_ratio * (largest / total) / square, square / lower);
}

layout_children :: (children: [] *Treemap_Node, display: *Treemap_Display, corner: Vector2, size: Vector2, weight: float64) {
    start: s64;
    right := corner.x + size.x;
    top := corner.y + size.y;

    while start < children.count {
        end := start + 1;
        sum := children[start].size;
        minimum := sum;
        maximum := sum;
        short_side := cast(float64) min(size.x, size.y);
        long_side := cast(float64) max(size.x, size.y);
        score := worst_ratio(sum, minimum, maximum, weight, short_side, long_side);

        while end < children.count && children[end].size > 0 {
            next := children[end].size;
            next_min := min(minimum, next);
            next_max := max(maximum, next);
            next_score := worst_ratio(sum + next, next_min, next_max, weight, short_side, long_side);

            if next_score > score break;

            sum += next;
            minimum = next_min;
            maximum = next_max;
            score = next_score;
            end += 1;
        }

        horizontal := size.x < size.y;
        flushed := flush_nodes(children, start, end, display, corner, weight, horizontal, size);
        first := display.nodes[get_index(display.treemap, children[start])];

        if horizontal {
            corner.y += first.size.y;
            size.y = max(cast(float) 0, top - corner.y);
        } else {
            corner.x += first.size.x;
            size.x = max(cast(float) 0, right - corner.x);
        }

        weight = max(cast(float64) 0, weight - flushed);
        start = end;
    }
}
