#import "Basic";

show :: (names: [] string) {
    for names {
        print("% %\n", it_index, it);
    }
}

number :: (lines: [] string) {
    for lines {
        if !it continue;
        print("%: %\n", it_index, it);
    }
}

// A string's bytes are counted by `it_index` too.
letters :: (word: string) {
    for word {
        print("% %\n", it_index, it);
    }
}

main :: () {
    letters("abc");
    show(.["a", "b"]);
    number(.["x", "", "y"]);
}
