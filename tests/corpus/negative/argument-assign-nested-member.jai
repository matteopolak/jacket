#import "Basic";

Inner :: struct {
    y: int;
}

Outer :: struct {
    q: Inner;
}

set :: (p: Outer) {
    p.q.y = 1;
}

main :: () {
    set(.{});
}
