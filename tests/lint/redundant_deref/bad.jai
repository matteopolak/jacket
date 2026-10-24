#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

Line :: struct {
    start: Point;
    end: *Point;
}

main :: () {
    p := Point.{ 1, 2 };
    q := *p;

    q.*.x = 5;

    sum := q.*.x + q.*.y;

    line := Line.{ end = q };
    l := *line;

    sum += l.*.start.x + l.*.end.*.y;

    print("%\n", sum);
}
