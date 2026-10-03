# Code_Visit

## Public declaration inventory

Visible public declarations and module-conditional variants are listed below. Procedure bodies and aggregate initializers are omitted; complete public type members are listed separately.

- `maybe_array_add :: inline (array: *[..] $T, item: T) #no_abc`
- `get_subexpressions :: (node: *Code_Node, pointers: *[..] *Code_Node)`
- `get_recursive_nodes_breadth_first :: (node: *Code_Node, pointers: *[..] *Code_Node)`
- `get_recursive_nodes_depth_first :: (node: *Code_Node, pointers: *[..] *Code_Node)`
- `Dumb_Value :: struct`
- `get_values :: (node: *Code_Node, values: *[..] Dumb_Value)`
- `visit_pre_and_postorder :: (node: *Code_Node, code: Code) #expand`

## Public type members

#### `Dumb_Value` (struct)

- `node: *Code_Node`
- `preorder := false`

## Behavioral contract and constraints

The visitor traverses selected syntax-tree nodes: procedure nodes include their arguments, while headers do not descend into bodies. It intentionally omits ancillary structures such as overload records, parent links, and members, as described by the source. Callers should not treat it as a complete compiler AST walk.

For each yielded node, the macro evaluates the supplied `Code` with two caller-scope bindings: `it: *Code_Node` is the current node, and `preorder: bool` is `true` for the pre-order event and `false` for the post-order event. The same node is therefore presented twice when it has both events; code that declares conflicting caller-scope names may collide with these bindings.
