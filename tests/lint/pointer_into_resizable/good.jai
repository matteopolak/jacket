#import "Basic";

Item :: struct {
    value: int;
}

by_value_loop :: () -> int {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });

    // A by-value loop reads the count and data again each time round.
    for items {
        if items.count < 4 array_add(*items, .{ it.value + 1 });
    }

    return items.count;
}

pointer_before_only :: () -> int {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });
    first := *items[0];
    first.value = 9;
    array_add(*items, .{ 2 });

    return items[0].value;
}

taken_again :: () -> int {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });
    first := *items[0];

    array_add(*items, .{ 2 });
    first = *items[0];

    return first.value;
}

other_array :: () -> int {
    a: [..] Item;
    b: [..] Item;

    defer array_free(a);
    defer array_free(b);

    array_add(*a, .{ 1 });
    first := *a[0];

    array_add(*b, .{ 2 });

    return first.value;
}

leaves :: (flag: bool) -> int {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });
    first := *items[0];

    if flag {
        array_add(*items, .{ 2 });
        return items.count;
    }

    return first.value;
}

index_kept :: () -> int {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });
    index := 0;

    array_add(*items, .{ 2 });

    return items[index].value;
}

fixed_array :: () -> int {
    items: [4] Item;
    first := *items[0];
    first.value = 1;

    return items[0].value;
}

main :: () {
    print("% % % % % % % %\n", by_value_loop(), pointer_before_only(), taken_again(), other_array(), leaves(false), leaves(true), index_kept(), fixed_array());
}
