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

main :: () {
    show(.["a", "b"]);
    number(.["x", "", "y"]);
}
