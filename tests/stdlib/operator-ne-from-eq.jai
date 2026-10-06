// `a != b` falls back to `!(a == b)` when no `operator !=` matches, even when other
// `operator !=` overloads are visible.
// rules: opov.6
#import "Basic";

Id :: struct {
    v: int;
}
operator != :: (a: Id, b: Id) -> bool {
    return a.v != b.v;
}

P :: struct {
    x: int;
}
operator == :: (a: P, b: P) -> bool {
    return a.x == b.x;
}

main :: () {
    a := P.{ 1 };
    if !(a != .{ 2 }) exit(1);
    if a != .{ 1 } exit(2);
    t := current_time_monotonic();
    if t != t exit(3);
    print("ok\n");
}
