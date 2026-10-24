#import "Basic";

// TODO: handle the empty list
first :: (items: [] int) -> int {
    return items[0];
}

/* FIXME the loop below is quadratic */
total :: (items: [] int) -> int {
    sum := 0;
    for items sum += it;
    return sum;
}

main :: () {
    values := int.[1, 2, 3];
    print("%\n", first(values)); // XXX
    print("%\n", total(values)); // (HACK) until the parser lands
}
