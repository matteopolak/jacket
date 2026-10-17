// Inserting a custom iteration body retains its enclosing array-loop removal capability.
#import "Basic";

Bag :: struct {
    items: [..] int;
}

Wrapper :: struct {
    bag: Bag;
}

for_expansion :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    for <= (flags & .REVERSE != 0) `it, `it_index: bag.items {
        #insert body;
    }
}

for_expansion :: (wrapper: *Wrapper, body: Code, flags: For_Flags) #expand {
    for_expansion(*wrapper.bag, body, flags);
}

main :: () {
    wrapper: Wrapper;
    for 1..6 array_add(*wrapper.bag.items, it);

    defer array_free(wrapper.bag.items);
    visited := 0;

    for value, slot: wrapper {
        visited += 1;
        assert(slot >= 0 && slot < wrapper.bag.items.count);
        if value % 2 == 0 remove value;
    }

    assert(visited == 6);
    assert(wrapper.bag.items.count == 3);
    assert(wrapper.bag.items[0] == 1 && wrapper.bag.items[1] == 5 && wrapper.bag.items[2] == 3);
    wrapper.bag.items.count = 0;
    for 1..6 array_add(*wrapper.bag.items, it);

    visited = 0;

    for < value: wrapper {
        visited += 1;
        if value % 2 == 0 remove value;
    }

    assert(visited == 6 && wrapper.bag.items.count == 3);
    assert(wrapper.bag.items[0] == 1 && wrapper.bag.items[1] == 5 && wrapper.bag.items[2] == 3);
}
