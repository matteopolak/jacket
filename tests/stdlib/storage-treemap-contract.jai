// Authored consumers: original records, weighted tiling, borders, multiple displays.
#import "Basic";
T :: #import "Treemap";
L :: #import "legacy/Bucket_Array";
M :: #import "Math";
#load "storage-allocator-tracker.jai";

Treemap_Layout :: struct {
    nodes: L.Bucket_Array(T.Treemap_Node, 1024);
    root: *T.Treemap_Node;
    dirty := true;
}
Treemap_Display_Layout :: struct {
    nodes: [..] T.Display_Node;
    treemap: *T.Treemap;
    outer_size: M.Vector2;
    dirty := true;
}
#assert size_of(T.Treemap) == size_of(Treemap_Layout);
#assert size_of(T.Treemap_Display) == size_of(Treemap_Display_Layout);

border_calls: int;
bad_parent: bool;

root_border :: (corner: M.Vector2, size: M.Vector2, node: *T.Treemap_Node, parent: *T.Display_Node, values: *[4]float) {
    border_calls += 1;
    if !node.parent { if parent bad_parent = true; <<values = .[2,2,2,2]; }
    else if !parent || parent.node != node.parent bad_parent = true;
}

no_border :: (corner: M.Vector2, size: M.Vector2, node: *T.Treemap_Node, parent: *T.Display_Node, values: *[4]float) {
    border_calls += 1;
}

node_area :: (node: T.Display_Node) -> float64 { return cast(float64)node.size.x * cast(float64)node.size.y; }

close_area :: (a: float64, b: float64) -> bool {
    difference := a - b;
    if difference < 0 difference = -difference;
    return difference < 0.01;
}

check_treemap_contract :: () -> int {
    if !storage_same_layout(T.Treemap, Treemap_Layout) || !storage_same_layout(T.Treemap_Display, Treemap_Display_Layout) return 1;
    border_calls = 0;
    bad_parent = false;
    map: T.Treemap;
    defer T.free_treemap(*map);
    root := T.add_node(*map, "root", null);
    large := T.add_node(*map, "large", root);
    small := T.add_node(*map, "small", root);
    zero := T.add_node(*map, "zero", root);
    large.size = 9;
    small.size = 3;
    zero.size = 0;
    first := T.init_treemap_display(*map);
    second := T.init_treemap_display(*map);
    defer T.free_treemap_display(*first);
    defer T.free_treemap_display(*second);
    T.recompute_if_dirty(*first, .{100,80}, root_border);
    if map.dirty || first.dirty || root.size != 12 || first.nodes.count != 4 || border_calls != 4 || bad_parent return 2;
    a := first.nodes[T.get_index(*map, large)];
    b := first.nodes[T.get_index(*map, small)];
    z := first.nodes[T.get_index(*map, zero)];
    if !close_area(node_area(a), 5472) || !close_area(node_area(b), 1824) || node_area(z) != 0 return 3;
    if a.corner.x < 2 || a.corner.y < 2 || a.corner.x + a.size.x > 98.01 || a.corner.y + a.size.y > 78.01 return 4;
    if b.corner.x < 2 || b.corner.y < 2 || b.corner.x + b.size.x > 98.01 || b.corner.y + b.size.y > 78.01 return 5;
    separated := a.corner.x + a.size.x <= b.corner.x + 0.01 || b.corner.x + b.size.x <= a.corner.x + 0.01 || a.corner.y + a.size.y <= b.corner.y + 0.01 || b.corner.y + b.size.y <= a.corner.y + 0.01;
    if !separated return 6;
    T.recompute_if_dirty(*first, .{100,80}, root_border);
    if border_calls != 4 return 7;
    T.recompute_if_dirty(*second, .{100,80}, root_border);
    if border_calls != 8 return 8;
    large.size = 6;
    small.size = 6;
    map.dirty = true;
    T.recompute_if_dirty(*first, .{100,80}, root_border);
    T.recompute_if_dirty(*second, .{100,80}, root_border);
    if border_calls != 16 || bad_parent return 9;
    a = second.nodes[T.get_index(*map, large)];
    b = second.nodes[T.get_index(*map, small)];
    if !close_area(node_area(a), 3648) || !close_area(node_area(b), 3648) return 10;
    T.recompute_if_dirty(*first, .{100,80}, root_border, true);
    if border_calls != 20 return 11;
    T.recompute_if_dirty(*first, .{100,80}, no_border);
    if border_calls != 24 || !close_area(node_area(first.nodes[T.get_index(*map, large)]), 4000) return 12;
    T.recompute_if_dirty(*first, .{0,80}, no_border);
    if border_calls != 28 || node_area(first.nodes[T.get_index(*map, large)]) != 0 return 13;
    T.free_treemap_display(*first);
    T.free_treemap_display(*second);
    T.free_treemap(*map);
    if map.root != null || map.nodes.count != 0 || first.nodes.count != 0 || second.nodes.count != 0 return 14;
    new_root := T.add_node(*map, "again", null);
    leaf := T.add_node(*map, "leaf", new_root);
    leaf.size = 1;
    first = T.init_treemap_display(*map);
    T.recompute_if_dirty(*first, .{10,10}, no_border);
    if first.nodes.count != 2 || !close_area(node_area(first.nodes[T.get_index(*map, leaf)]), 100) return 15;
    return 0;
}

#assert #run check_treemap_contract() == 0;
main :: () -> int { return check_treemap_contract(); }
