#import "Basic";

square :: (n: int) -> int {
    result := n * n;
    return result;
}

one :: () -> int {
    return 7;
}

called :: () -> int {
    value := one();
    return value;
}

copies :: (n: int) {
    raw := n * 3;
    total := raw;
    total += 1;
    print("%\n", total);

    // A literal copied, then changed.
    start := 10;
    count := start;
    count += n;
    print("%\n", count);

    // Multi-line initializer.
    wide := (n + 1) *
        (n + 2);
    area := wide;
    print("%\n", area);
}

// An explicit type on the first is kept.
typed :: () {
    small: u8 = 5;
    big := small;
    print("%\n", big);
}

in_loop :: () {
    for 0..2 {
        doubled := it * 2;
        shown := doubled;
        print("%\n", shown);
    }
}

main :: () {
    copies(4);
    typed();
    in_loop();
    print("%\n", square(3) + called());
}
