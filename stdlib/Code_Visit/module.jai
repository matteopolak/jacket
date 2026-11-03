// Walks the nodes of a `Compiler.Code_Node` tree.
//
// - [get_subexpressions] lists a node's direct children.
// - [get_recursive_nodes_breadth_first] and [get_recursive_nodes_depth_first] list a node and everything below it.
// - [get_values] gives enter and leave events, for tracking nesting.
//
// ```jai
// Compiler :: #import "Compiler";
// left := Compiler.make_integer_literal(13);
// right := Compiler.make_integer_literal(29);
// sum: Compiler.Code_Binary_Operator;
// sum.kind = .BINARY_OPERATOR;
// sum.left = left;
// sum.right = right;
//
// all: [..] *Compiler.Code_Node;
// get_recursive_nodes_depth_first(*sum, *all);
// assert(all.count == 3);
// ```

// One step of a [get_values] walk: the node, and whether the walk is entering it (`preorder`) or leaving it.
Dumb_Value :: struct {
    node: *C.Code_Node;
    preorder := false;
}

// Adds `item` to `array` unless it is zero.
maybe_array_add :: inline (array: *[..] $T, item: T) #no_abc {
    if item Basic.array_add(array, item);
}

get_subexpressions :: Impl.get_subexpressions;
get_recursive_nodes_breadth_first :: Impl.get_recursive_nodes_breadth_first;
get_recursive_nodes_depth_first :: Impl.get_recursive_nodes_depth_first;
get_values :: Impl.get_values;

#scope_file
C :: #import "Compiler";
Basic :: #import "Basic";
Impl :: #import "internal/code-visit";
