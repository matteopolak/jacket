#import "Basic";
T :: #import "Treemap";
B :: #import "Bucket_Array";
M :: #import "Math";

calls: s64;

border :: (corner: M.Vector2, outer_dimensions: M.Vector2, node: *T.Treemap_Node, parent: *T.Display_Node, border_values: *[4] float) {
    calls += 1;
    if !parent for *value: border_values.* value.* = 1;
    else assert(parent.node == node.parent);
}

near :: (a: float64, b: float64) -> bool {
    difference := a - b;
    if difference < 0 difference = -difference;
    return difference < 0.001;
}

area :: (display: *T.Treemap_Display, node: *T.Treemap_Node) -> float64 {
    item := display.nodes[T.get_index(display.treemap, node)];
    return cast(float64) item.size.x * cast(float64) item.size.y;
}

main :: () {
    #assert !#exists(T.layout_children);
    assert((cast(*Type_Info_Struct) type_info(T.Treemap_Node)).name == "Treemap_Node");
    tree: T.Treemap;
    label := u8.[114, 111, 111, 116];
    root := T.add_node(*tree, string.{ label.count, label.data }, null);

    label[0] = 88;
    assert(root.name == "root");
    small := T.add_node(*tree, "small", root);
    large := T.add_node(*tree, "large", root);
    medium := T.add_node(*tree, "medium", root);

    small.size = 1;
    large.size = 6;
    medium.size = 3;
    assert(tree.dirty && tree.nodes.count == 4);
    T.recompute_if_dirty(*tree);
    assert(!tree.dirty && root.size == 10 && root.children[0] == large && root.children[2] == small);
    display := T.init_treemap_display(*tree);

    T.recompute_if_dirty(*display, .{ 100, 80 });
    assert(display.nodes.count == 4 && near(area(*display, root), 8000));
    assert(near(area(*display, large), 4800) && near(area(*display, medium), 2400) && near(area(*display, small), 800));

    for first_index: 0..2 {
        a := display.nodes[T.get_index(*tree, root.children[first_index])];

        for second_index: first_index + 1..2 {
            b := display.nodes[T.get_index(*tree, root.children[second_index])];
            assert(a.corner.x + a.size.x <= b.corner.x || b.corner.x + b.size.x <= a.corner.x ||
                   a.corner.y + a.size.y <= b.corner.y || b.corner.y + b.size.y <= a.corner.y);
        }
    }

    T.recompute_if_dirty(*display, .{ 100, 80 }, border_proc = border, force = true);
    assert(calls == 4 && near(area(*display, large), 98 * 78 * 0.6));
    T.recompute_if_dirty(*display, .{ 100, 80 }, border_proc = border);
    assert(calls == 4);
    large.size = 1;
    tree.dirty = true;
    T.recompute_if_dirty(*tree);
    T.recompute_if_dirty(*display, .{ 100, 80 }, force = true);
    assert(root.children[0] == medium && near(area(*display, medium), 4800));
    first := T.add_node(*tree, "first", large);
    second := T.add_node(*tree, "second", large);

    first.size = 1;
    second.size = 2;
    T.recompute_if_dirty(*tree);
    T.recompute_if_dirty(*display, .{ 100, 80 }, force = true);
    assert(large.size == 3 && root.size == 7);
    assert(near(area(*display, first) + area(*display, second), area(*display, large)));
    assert(near(area(*display, second), 2 * area(*display, first)));

    for index: 0..1024 {
        node := T.add_node(*tree, "more", root);
        node.size = 1;
        assert(T.get_index(*tree, node) == index + 6);
    }

    assert(T.get_index(*tree, root) == 0 && root.name == "root");
    T.recompute_if_dirty(*tree);
    T.recompute_if_dirty(*display, .{ 100, 80 }, force = true);
    assert(display.nodes.count == tree.nodes.count);
    total: float64;
    for node: root.children total += area(*display, node);

    assert(near(total, 8000));
    array_free(display.nodes);

    for *node: tree.nodes {
        free(node.name);
        array_free(node.children);
    }

    B.bucket_array_reset(*tree.nodes);
    array_free(tree.nodes.all_buckets);
    array_free(tree.nodes.unfull_buckets);
    print("ok\n");
}
