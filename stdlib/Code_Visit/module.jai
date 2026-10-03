Dumb_Value :: struct {
    node: *C.Code_Node;
    preorder := false;
}

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
