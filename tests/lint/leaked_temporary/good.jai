#import "Basic";

named :: (n: int) -> string {
    // Handed on to the caller.
    return sprint("%", n);
}

describe :: (n: int) -> bool {
    // Kept in a local and freed.
    text := sprint("%", n);
    defer free(text);
    return text == "7";
}

temporary :: (n: int) -> bool {
    // Temporary storage is reset by the program, not freed.
    return tprint("%", n) == "7";
}

scoped :: (n: int) -> bool {
    // Under another allocator the memory is not the caller's to free.
    push_allocator(temp);
    return sprint("%", n) == "7";
}

changed :: (n: int) -> bool {
    return sprint("%", n,, temp) == "7";
}

main :: () {
    print("%\n", describe(7));
    print("%\n", temporary(7));
    print("%\n", scoped(7));
    print("%\n", changed(7));
    value := named(3);

    defer free(value);
    print("%\n", value);
}
