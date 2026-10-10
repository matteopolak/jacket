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

// Skips some bytes, so the counter is not the position.
nonblank :: (word: string) {
    n := 0;
    for word {
        if it == #char " " continue;
        print("% %\n", n, it);
        n += 1;
    }
}

main :: () {
    nonblank("a b");
    names := string.["ab", "abc"];
    print("%\n", count_long(names));
    show(names);
    total(names);
}
