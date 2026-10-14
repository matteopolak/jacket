For_Flags :: u8;

Bag :: struct {
    count: int;
}

for_expansion :: (bag: *Bag, body: Code, flags: For_Flags) #expand {
    for index: 0..bag.count {
        `it := index;
        #insert body;
    }
}

walk :: (bag: $T, body: Code, flags: For_Flags) #expand {
    for index: 0..bag.count {
        `it := index;
        #insert body;
    }
}

aliased_walk :: walk;

main :: () {
    collection: Bag;
    for item: collection {
        result := item + 1;
    }
    for: aliased_walk item : collection {
        result := item * 2;
    }
}
