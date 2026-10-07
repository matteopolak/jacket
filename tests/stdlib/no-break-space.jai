// U+00A0 (no-break space) separates tokens like a space.
#import "Basic";

main :: () {
    x: [..][..] int;
    array_add(*x, .{});
    assert(x.count == 1);
}
