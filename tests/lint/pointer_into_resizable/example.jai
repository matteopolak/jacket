#import "Basic";

Item :: struct {
    value: int;
}

main :: () {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });
    items.count = items.allocated;
    first := *items[0];

    array_add(*items, .{ 2 });

    print("%\n", first.value);
}
