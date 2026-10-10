#import "Basic";
Strings :: #import "String";

Mode :: enum {
    // Same as [Mode.OF], which is not a member.
    IDLE;
    BUSY;
}

Point :: struct {
    LIMIT :: 100;
    // Past [Point.LIMT] it clips; see also [Point.z].
    x: int;
    y: int; // Like [Point.xx].
}

// Scales a [Point] and prints it with [prnt], or [`print_it()`](print_it).
scale :: (p: Point) {}

// A [the-missing](gone) and a missing [Strings.joyn] and [Nowhere.thing].
other :: () {}

main :: () {
    scale(.{});
    other();
    print("done\n");
    _ := Strings.join("a", "b");
}
