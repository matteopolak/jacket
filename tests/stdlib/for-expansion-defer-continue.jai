// `continue` / `break` in a for body inserted into a for_expansion run the defers the
// expansion registered inside its loop (`defer i += 1;` before `#insert body;`).
#import "Basic";

Bag :: struct { count: int; items: [6] int; }

for_expansion :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    i := 0;
    while i < bag.count {
        `it := bag.items[i];
        `it_index := i;
        defer i += 1;
        #insert body;
    }
}

main :: () {
    bag: Bag;
    bag.count = 6;
    bag.items = .[1, 0, 3, 0, 5, 9];

    seen: [..] int;
    for bag {
        if it == 0 continue;
        if it == 9 break;
        array_add(*seen, it_index);
    }
    assert(seen.count == 3 && seen[0] == 0 && seen[1] == 2 && seen[2] == 4);

    // The same rule in a plain loop.
    j := 0;
    total := 0;
    while j < 4 {
        defer j += 1;
        if j == 1 continue;
        total += j;
    }
    assert(total == 5);
    print("ok\n");
}
