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

// Encodes `source` into a tree under the tracking allocator, compares it with the text encoding
// and frees it there, since free_json releases through context.allocator.
encode_counted :: (source: $T) -> blocks: s64, same: bool, outstanding: s64 {
    tally: Tally;
    same: bool;

    {
        push_allocator(tally_proc, *tally);

        tree := to_json_value(source);

        same = to_json(tree,, temp) == to_json(source,, temp);
        free_json(*tree);
    }

    return tally.allocations, same, tally.outstanding;
}

shallow :: () {
    point_blocks, point_same, point_left := encode_counted(Point.{ 3, 4 });

    check(point_blocks == POINT_BLOCKS, tprint("a point allocated % blocks, not %", point_blocks, POINT_BLOCKS));
    check(point_same, "the point's tree has the text's content");
    check(point_left == 0, tprint("a point's tree left % blocks allocated", point_left));

    shape: Shape;

    array_add(*shape.heights, 1);
    array_add(*shape.heights, 2);
    defer array_free(shape.heights);

    shape_blocks, shape_same, shape_left := encode_counted(shape);

    check(shape_blocks == SHAPE_BLOCKS, tprint("a shape allocated % blocks, not %", shape_blocks, SHAPE_BLOCKS));
    check(shape_same, "the shape's tree has the text's content");
    check(shape_left == 0, tprint("a shape's tree left % blocks allocated", shape_left));
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
