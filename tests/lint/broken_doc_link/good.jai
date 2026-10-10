#import "Basic";
Strings :: #import "String";

Mode :: enum {
    // The first, like [Mode.BUSY].
    IDLE;
    BUSY;
}

Point :: struct {
    LIMIT :: 100;
    // Capped by [Point.LIMIT], next to [`Point.y`].
    x: int;
    y: int; // Like [Point.x].
}

// Scales a [Point] and prints it with [print] or [Strings.contains()], see [the text](scale).
// Not links: a[i], [0], [x][y], [x], [TODO], [ ], [^1], [some words], [text](https://example.com).
// Prose: matches any byte [sic], is not here [yet], quotes '[description]' and a [table.header].
// In code: `[nothing]` and ``[none]``.
//
// ```
// [absent]
// ```
//
// - [ ] a checkbox
// - [reference] is defined below, see [guide][ref].
// [reference]: https://example.com/reference
// [ref]: https://example.com/guide
scale :: (p: Point) {}

// Documents main.
main :: () {
    // The value of [x] is not documented here.
    x := 5;
    // jailint: allow(broken_doc_link)
    // This one is silenced: [alsogone].
    scale(.{});
    // Statements are not documented: [nowhere_at_all] is not looked at.
    print("%\n", x);
    _ := Strings.begins_with("a", "b");
}

// jailint: allow(broken_doc_link)
// [Point.nothing] is silenced by the marker above it.
quiet :: () {}
