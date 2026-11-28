#import "Basic";

LIMIT :: 64;

Counter :: struct {
    value: int;
}

Mode :: enum {
    SLOW;
    FAST;
}

orphan_count: int;

// Only `report` calls it, and nothing calls `report`.
describe :: (n: int) -> string {
    return "number";
}

report :: (n: int) {
    print("%\n", describe(n));
}

ping :: (n: int) {
    if n > 0 pong(n - 1);
}

pong :: (n: int) {
    if n > 0 ping(n - 1);
}

main :: () {
    print("hello\n");
}
