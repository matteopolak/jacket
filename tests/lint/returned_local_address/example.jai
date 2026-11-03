#import "Basic";

Point :: struct {
    x, y: int;
}

origin :: () -> *Point {
    p: Point;
    return *p;
}

main :: () {
    p := origin();
    print("%\n", p.x);
}
