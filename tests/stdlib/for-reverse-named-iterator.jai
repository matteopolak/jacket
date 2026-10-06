// `for < :name x: c` combines reverse iteration with a named for_expansion (segcore/jai-protobuf
// writes `for < :one_ofs oneof: definition.subdefs`).
#import "Basic";

Bag :: struct {
    values: [3] int = .[1, 2, 3];
}

backwards_aware :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    reverse := flags & .REVERSE;
    for i: 0..2 {
        index := ifx reverse then 2 - i else i;
        `it := bag.values[index];
        `it_index := index;
        #insert body;
    }
}

main :: () {
    bag: Bag;
    seen: [..] int;
    for <: backwards_aware v : *bag array_add(*seen, v);
    assert(seen.count == 3 && seen[0] == 3 && seen[2] == 1);

    seen.count = 0;
    for: backwards_aware v : *bag array_add(*seen, v);
    assert(seen[0] == 1 && seen[2] == 3);
}
