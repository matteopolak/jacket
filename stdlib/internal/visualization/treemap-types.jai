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

Treemap :: struct {
    nodes: Bucket_Array(Treemap_Node, 1024);
    root: *Treemap_Node;
    dirty := true;
}

Display_Node :: struct {
    corner: Vector2;
    size: Vector2;
    border: [4] float;
    node: *Treemap_Node;
}

Treemap_Display :: struct {
    nodes: [..] Display_Node;
    treemap: *Treemap;
    outer_size: Vector2;
    dirty := true;
}

Border_Direction :: enum u8 {
    EAST :: 0;
    NORTH :: 1;
    WEST :: 2;
    SOUTH :: 3;
}

Border_Proc :: #type (corner: Vector2, outer_dimensions: Vector2, node: *Treemap_Node, parent: *Display_Node, border_values: *[4] float);

#scope_module
#import "Math";
#import "Bucket_Array";
