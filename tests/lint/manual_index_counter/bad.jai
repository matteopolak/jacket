#import "Basic";

show :: (names: [] string) {
    n := 0;
    for names {
        print("% %\n", n, it);
        n += 1;
    }
}

number :: (lines: [] string) {
    line_number := 0;
    for lines {
        defer line_number += 1;
        if !it continue;
        print("%: %\n", line_number, it);
    }
}

// A string's bytes are counted by `it_index` too.
letters :: (word: string) {
    n := 0;
    for word {
        print("% %\n", n, it);
        n += 1;
    }
}

main :: () {
    letters("abc");
    show(.["a", "b"]);
    number(.["x", "", "y"]);
}
