// `for: path x` picks a for_expansion named by a plain name, `Type.name`, or `value.name`.
#import "Basic";

Range :: struct {
    first: int;
    count: int;
}

Holder :: struct {
    Walk :: (range: *Range, body: Code, flags: For_Flags) #expand {
        #assert(flags == 0);

        for i: 0..range.count - 1 {
            `it_index := range.first + i;
            `it := `tree.values[`it_index];
            #insert (remove = #assert(false), break = break i) body;
        }
    }

    values: [..] int;
}

Holder_Value :: struct {
    holder: Holder;
}

Walk :: (range: *Range, body: Code, flags: For_Flags) #expand {
    #assert(flags == 0);

    for i: 0..range.count - 1 {
        `it_index := range.first + i;
        `it := 100 + range.first + i;
        #insert (remove = #assert(false), break = break i) body;
    }
}

sum_member :: (tree: *Holder, range: Range) -> int {
    total := 0;
    for: tree.Walk range total += it;
    return total;
}

main :: () {
    tree: Holder;
    for 1..5 array_add(*tree.values, it * 10);

    defer array_free(tree.values);
    range := Range.{ 1, 3 };

    assert(sum_member(*tree, range) == 20 + 30 + 40);

    total := 0;
    for: Holder.Walk range total += it;

    assert(total == 90);

    total = 0;
    for: Walk range total += it;

    assert(total == 101 + 102 + 103);

    total = 0;
    wrapper: Holder_Value;
    wrapper.holder = tree;
    for: wrapper.holder.Walk range total += it;

    assert(total == 90);
}
