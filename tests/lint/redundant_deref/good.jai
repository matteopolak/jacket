#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

main :: () {
    p := Point.{ 1, 2 };
    q := *p;
    qq := *q;

    q.x = 5;

    sum := q.x + q.y;

    // A pointer to a pointer needs its dereference.
    sum += qq.*.x;

    // A copy of the whole value is a different thing from a member.
    copy := q.*;

    sum += copy.x;

    print("%\n", sum);
}
