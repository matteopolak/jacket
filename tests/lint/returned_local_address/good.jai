#import "Basic";

Point :: struct {
    x, y: int;
}

g_count := 0;

global_counter :: () -> *int {
    return *g_count;
}

through_pointer :: (p: *Point) -> *int {
    q := p;
    return *q.y;
}

element :: (xs: [] int, i: int) -> *int {
    view := xs;
    return *view[i];
}

fresh :: () -> *Point {
    p := New(Point);
    return p;
}

copy :: () -> Point {
    p: Point;
    return p;
}

parameter :: (n: int) -> *int {
    return *n;
}

main :: () {
    print("%\n", global_counter());
    point: Point;

    print("%\n", through_pointer(*point));
    print("%\n", element(.[1, 2], 1));
    print("%\n", fresh());
    print("%\n", copy());
    print("%\n", parameter(1));
}
