// Loaded by both entries: its names mean the same in each.
Counter :: struct { value: int; }

bump :: (counter: *Counter) {
    counter.value += 1;
}
