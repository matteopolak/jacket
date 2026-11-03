#import "Basic";

Point :: struct {
    x, y: int;
}

origin :: () -> *Point {
    p := New(Point);
    return p;
}

main :: () {
    p := origin();
    print("%\n", p.x);
    free(p);
}
