// One rectangle's worth of data: a name and a size, with children inside it. A leaf's `size` is set
// by the caller; a parent's is the sum of its children's, filled in by `recompute_if_dirty`.
// The `user_data_` fields are free for the program.
Treemap_Node :: struct {
    name: string;
    size: float64;
    parent: *Treemap_Node;
    children: [..] *Treemap_Node;
    user_data_pointer: *void;
    user_data_s64: s64;
    user_data_float64: float64;
    locator: Bucket_Locator;
}

// A tree of sized nodes to be drawn as nested rectangles. Build it with `add_node`.
Treemap :: struct {
    nodes: Bucket_Array(Treemap_Node, 1024);
    root: *Treemap_Node;
    dirty := true;
}

// Where one node landed: its top-left `corner`, its `size`, and the `border` taken from its edges.
Display_Node :: struct {
    corner: Vector2;
    size: Vector2;
    border: [4] float;
    node: *Treemap_Node;
}

// The rectangles a `Treemap` lays out to in a given area, one `Display_Node` per node, indexed with `get_index`.
Treemap_Display :: struct {
    nodes: [..] Display_Node;
    treemap: *Treemap;
    outer_size: Vector2;
    dirty := true;
}

// Indexes into a node's four `border` widths.
Border_Direction :: enum u8 {
    EAST :: 0;
    NORTH :: 1;
    WEST :: 2;
    SOUTH :: 3;
}

// Called while laying out to say how much room a node keeps free along each edge for its
// own frame or label; the children are laid out inside what is left. Fill `border_values`
// (see `Border_Direction`); the default is zero.
Border_Proc :: #type (corner: Vector2, outer_dimensions: Vector2, node: *Treemap_Node, parent: *Display_Node, border_values: *[4] float);

#scope_module
#import "Math";
#import "Bucket_Array";
