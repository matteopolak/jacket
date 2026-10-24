#import "Basic";

// The length of `text`.
parse :: (text: string) -> int {
    return text.count;
}

/* A point on the grid. */
Point :: struct {
    x: int;
    y: int;
}

// How fast it goes.
Mode :: enum {
    FAST;
    SLOW;
}

LIMIT :: 10;

alias :: Point;

#scope_file

helper :: () {}

Hidden :: struct {
    z: int;
}

#scope_export

// Back in the exported scope.
visible :: () {
    helper();
}

main :: () {
    print("% %\n", parse("a"), Point.{ 1, 2 });
}
