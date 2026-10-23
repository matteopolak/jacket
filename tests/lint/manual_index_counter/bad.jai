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

// A `case` body and the arm of a `#if` are statement lists, not blocks.
by_case :: (names: [] string, mode: int) {
    if mode == {
        case 1;
            n := 0;

            for names {
                print("% %\n", n, it);
                n += 1;
            }
    }
}

by_static_if :: (names: [] string) {
    #if true {
        n := 0;

        for names {
            print("% %\n", n, it);
            n += 1;
        }
    }
}

main :: () {
    by_case(.["a"], 1);
    by_static_if(.["a"]);
    letters("abc");
    show(.["a", "b"]);
    number(.["x", "", "y"]);
}
