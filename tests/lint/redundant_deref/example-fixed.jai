#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

main :: () {
    p := Point.{ 1, 2 };
    q := *p;

    print("%\n", q.x + q.y);
}
