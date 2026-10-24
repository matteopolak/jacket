#import "Basic";

Item :: struct {
    value: int;
}

Store :: struct {
    items: [..] Item;
}

element_then_add :: () -> int {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });
    first := *items[0];

    array_add(*items, .{ 2 });

    // `first` may point into the block `array_add` just freed.
    return first.value;
}

field_then_reserve :: (store: *Store) -> int {
    slot := *store.items[0].value;
    array_reserve(*store.items, 1000);

    return slot.*;
}

added_then_added :: () -> int {
    items: [..] Item;

    defer array_free(items);

    last := array_add(*items);

    array_add(*items, .{ 5 });

    last.value = 3;

    return items.count;
}

pointer_loop :: () -> int {
    items: [..] Item;

    defer array_free(items);

    array_add(*items, .{ 1 });

    for *items {
        array_add(*items, .{ 2 });
        it.value += 1;
        if items.count > 5 break;
    }

    return items.count;
}

main :: () {
    store: Store;
    defer array_free(store.items);
    array_add(*store.items, .{ 4 });

    print("% % % % %\n", element_then_add(), field_then_reserve(*store), added_then_added(), pointer_loop(), store.items.count);
}
