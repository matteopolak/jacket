// to_json_value keeps its stack of arrays and objects being built inline: a shallow document
// allocates only the tree's own blocks, and one nested deeper than the inline frames still encodes
// the same tree as text.
//
//     jaic stdlib/Extensions/Json/tests/tree-frames.jai -interp

#import "Basic";
#import "Extensions/Json";

failures := 0;

check :: (condition: bool, what: string) {
    if condition return;
    print("FAIL: %\n", what);
    failures += 1;
}

#load "../../test-support/counting-allocator.jai";

Point :: struct {
    x: s32;
    y: s32;
}

Shape :: struct {
    origin: Point;
    corners: [3] Point;
    heights: [..] s32;
}

Chain :: struct {
    value: s32;
    next: *Chain; @json(default,skip_serializing_if=null)
}

// The blocks the trees themselves take: their keys and their arrays and objects as they grow.
// Nothing else is allocated, so a frame stack on the heap shows as one block too many.
POINT_BLOCKS :: 5;
SHAPE_BLOCKS :: 28;

// Encodes `source` into a tree under the tracking allocator and returns how many blocks it took.
encode_counted :: (source: $T, tree: *Json_Value) -> s64 {
    tally: Tally;

    {
        push_allocator(tally_proc, *tally);

        tree.* = to_json_value(source);
    }

    return tally.allocations;
}

shallow :: () {
    point_tree: Json_Value;
    point_blocks := encode_counted(Point.{ 3, 4 }, *point_tree);

    check(point_blocks == POINT_BLOCKS, tprint("a point allocated % blocks, not %", point_blocks, POINT_BLOCKS));
    free_json(*point_tree);

    shape: Shape;

    array_add(*shape.heights, 1);
    array_add(*shape.heights, 2);
    defer array_free(shape.heights);

    shape_tree: Json_Value;
    shape_blocks := encode_counted(shape, *shape_tree);

    check(shape_blocks == SHAPE_BLOCKS, tprint("a shape allocated % blocks, not %", shape_blocks, SHAPE_BLOCKS));
    check(to_json(shape_tree,, temp) == to_json(shape,, temp), "the shape's tree has the text's content");
    free_json(*shape_tree);
}

deep :: () {
    // More levels than the inline frames hold: the rest spill to the heap, and the tree is the same.
    for levels: int.[10, 16, 17, 100] {
        nodes := NewArray(levels, Chain);

        defer array_free(nodes);

        for i: 0..levels - 1 {
            nodes[i].value = cast(s32) i;

            if i + 1 < levels nodes[i].next = *nodes[i + 1];
        }

        tally: Tally;
        same: bool;

        {
            push_allocator(tally_proc, *tally);

            tree := to_json_value(nodes[0]);

            same = to_json(tree,, temp) == to_json(nodes[0],, temp);
            free_json(*tree);
        }

        check(same, tprint("a chain of % levels encodes to the text's tree", levels));
        check(tally.outstanding == 0, tprint("a chain of % levels left % blocks allocated", levels, tally.outstanding));
    }
}

main :: () {
    shallow();
    deep();

    if failures {
        print("% failures\n", failures);
        exit(1);
    }

    print("ok\n");
}
