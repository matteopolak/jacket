# Treemap


## Public declaration inventory

Only names visible through the module’s exported scope are listed. Declarations from conditional branches retain their condition label; implementation-local declarations are omitted. Aggregate fields and enum members are recorded in the dedicated type-shape section.

- `Border_Direction :: enum u8`
- `Display_Node :: struct`
- `Treemap_Display :: struct`
- `Treemap_Node :: struct`
- `Treemap :: struct`
- `Border_Proc :: #type (corner: Vector2, outer_dimensions: Vector2, node: *Treemap_Node, parent: *Display_Node, border_values: *[4] float)`
- `add_node :: (treemap: *Treemap, name: string, parent: *Treemap_Node) -> *Treemap_Node`
- `recompute_if_dirty :: (treemap: *Treemap)`
- `init_treemap_display :: (treemap: *Treemap) -> Treemap_Display`
- `recompute_if_dirty :: (display: *Treemap_Display, outer_size: Vector2, border_proc: Border_Proc = null, force := false)`
- `size_children :: (treemap: *Treemap, root: *Treemap_Node, display: *Treemap_Display, border_proc: Border_Proc)`
- `flush_nodes :: (children: [] *Treemap_Node, unsized_cursor: int, child_index: int, display: *Treemap_Display, root_corner: Vector2, fill_weight_total: float64, horizontal: bool, size: Vector2) -> (flushed_node_weight: float64)`
- `get_index :: inline (treemap: *Treemap, node: *Treemap_Node) -> s64`


## Public aggregate shapes and enum values

Each block gives the exported type declaration with its member fields or enum values. Conditional declarations with distinct shapes are shown as separate variants; platform labels describe availability, not source organization.

### `Border_Direction`

```jai
Border_Direction :: enum u8 {
    EAST  :: 0;
    NORTH :: 1;
    WEST  :: 2;
    SOUTH :: 3;
}
```
### `Display_Node`

```jai
Display_Node :: struct {
    corner: Vector2;
    size:   Vector2;

    border: [4] float;

    node: *Treemap_Node;
}
```
### `Treemap`

```jai
Treemap :: struct {
    nodes: Bucket_Array(Treemap_Node, 1024);
    root: *Treemap_Node;

    dirty := true;
}
```
### `Treemap_Display`

```jai
Treemap_Display :: struct {
    nodes: [..] Display_Node;

    treemap: *Treemap;

    outer_size: Vector2;

    dirty := true;
}
```
### `Treemap_Node`

```jai
Treemap_Node :: struct {
    name: string;
    size: float64;

    parent:   *Treemap_Node;
    children: [..] *Treemap_Node;

    user_data_pointer: *void;
    user_data_s64:     s64;
    user_data_float64: float64;

    locator: Bucket_Locator;
}
```
## Behavioral contract

`add_node` creates the sole root when the tree is empty (parent must be null); every later node requires a non-null parent. It copies the name, links the node into the parent’s children, marks the tree dirty, and returns a stable node pointer until reset/destruction. There is no public node removal operation. `recompute_if_dirty(treemap)` recalculates non-leaf weights from descendants and sorts each child list from largest to smallest; this mutates child ordering. Call it after adding nodes or changing leaf weights before asking for display layout.

`init_treemap_display` binds a display object to a treemap. Display recomputation requires the tree already clean; it reuses cached geometry when the outer size is unchanged unless `force` is true or the display is marked dirty. It resizes `display.nodes` to match the node count, sets the root to the full outer rectangle, and computes descendant rectangles. `Border_Proc` defaults to null; a null callback leaves all border widths at zero. Before a non-null callback, the four widths are zeroed in `Border_Direction` order (east, north, west, south). The callback receives the current node's candidate corner and dimensions, that node, its parent's `Display_Node` or null for the root, and a writable pointer to the four widths. The returned widths inset the node's child-layout rectangle; negative remaining width or height is clamped to zero while moving the corresponding origin back to the inset boundary. `get_index` maps an owned node pointer to its bucket-array index. Display node positions are invalidated/recomputed when the tree/display changes and should be read only after recomputation. The public contract does not define useful layout guarantees for zero/negative weights, non-finite dimensions, or a node not belonging to the treemap. Depends on `Math` vectors and allocation helpers.

`flush_nodes` processes the half-open child range `[unsized_cursor, child_index)`. For each child it writes the corresponding display node's corner and size, dividing the supplied rectangle along `horizontal`; each child receives a fraction proportional to its weight divided by `fill_weight_total`, with denominator 1 when that total is zero. It returns the sum of the processed child weights. The procedure has no default arguments; callers must supply the range, display, corner, total, orientation, and rectangle explicitly. The public contract does not promise valid output for out-of-range cursors or indices.
