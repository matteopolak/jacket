#import "Basic";

// Counts only some elements.
count_long :: (names: [] string) -> int {
    n := 0;
    for names {
        if it.count < 3 continue;
        n += 1;
    }
    return n;
}

// Starts at one.
show :: (names: [] string) {
    n := 1;
    for names {
        print("% %\n", n, it);
        n += 1;
    }
}

// Used after the loop.
total :: (names: [] string) {
    n := 0;
    for names {
        print("%\n", it);
        n += 1;
    }
    print("% names\n", n);
}

main :: () {
    names := string.["ab", "abc"];
    print("%\n", count_long(names));
    show(names);
    total(names);
}
