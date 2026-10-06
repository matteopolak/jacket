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

main :: () {
    show(.["a", "b"]);
    number(.["x", "", "y"]);
}
