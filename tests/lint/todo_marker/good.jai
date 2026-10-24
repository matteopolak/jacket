#import "Basic";

// Returns the first item; the list must not be empty.
first :: (items: [] int) -> int {
    return items[0];
}

// The marker words are only markers when they stand alone in capitals:
// todo lists, TODOS, AUTODOC, XXXL, HACKS and "to do" are prose.
total :: (items: [] int) -> int {
    sum := 0;
    for items sum += it;
    return sum;
}

main :: () {
    values := int.[1, 2, 3];
    print("%\n", first(values));
    print("TODO is text in a string\n");
    print("%\n", total(values));
}
