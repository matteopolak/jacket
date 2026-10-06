#import "Basic";

// Taken as a value: the signature is fixed by its caller.
on_event :: (code: int, data: *void) {
    print("event %\n", code);
}

// Marked as deliberately unused.
draw :: (x: int, _y: int) {
    print("%\n", x);
}

// Overloads share a shape.
describe :: (value: int, width: int) {
    print("% (%)\n", value, width);
}

describe :: (value: string, width: int) {
    print("%\n", value);
}

// The argument settles `T`.
size_of_value :: (value: $T) -> s64 {
    return size_of(T);
}

// A stub.
later :: (count: int) {}

main :: () {
    handler := on_event;
    handler(1, null);
    draw(1, 2);
    describe(1, 4);
    describe("x", 4);
    later(3);
    print("%\n", size_of_value(1.0));
}
