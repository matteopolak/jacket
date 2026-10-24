#import "Basic";

parse :: (text: string) -> int {
    return text.count;
}

Point :: struct {
    x: int;
    y: int;
}

Mode :: enum {
    FAST;
    SLOW;
}

Bits :: union {
    a: int;
    b: float;
}

// A comment separated by a blank line is not a doc.

late :: () {}

main :: () {
    print("% %\n", parse("a"), Point.{ 1, 2 });
}
