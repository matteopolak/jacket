// An untyped struct literal as an operand of an overloaded binary operator takes the type the
// operator overloads ask for, the way a literal argument of a call does.
#import "Basic";
#import "Math";

Rect :: struct {
    x, y, w, h: float;
}

make_rect :: (x: float, y: float, w: float, h: float) -> Rect {
    return .{ x, y, w, h };
}

make_rect :: (position: Vector2, size: Vector2) -> Rect {
    return .{ position.x, position.y, size.x, size.y };
}

PARTICLE_WIDTH :: 4.0;
PARTICLE_HEIGHT :: 2.0;

main :: () {
    // The literal is the left operand and nothing else names its type.
    f := 1.0;
    size := .{ 6.0, 6.0 } * f;

    assert(type_of(size) == Vector2);
    assert(size.x == 6 && size.y == 6);

    // The literal is the right operand.
    sum := Vector2.{ 1, 1 } + .{ 2, 3 };

    assert(type_of(sum) == Vector2);
    assert(sum.x == 3 && sum.y == 4);

    // The literal is the left operand of an operator whose other operand is a struct.
    sum2 := .{ 2.0, 3.0 } + Vector2.{ 1, 1 };

    assert(type_of(sum2) == Vector2);
    assert(sum2.x == 3 && sum2.y == 4);

    // Compound assignment.
    v := Vector2.{ 1, 2 };
    v += .{ 10, 20 };

    assert(v.x == 11 && v.y == 22);

    // The issue's program: a literal operand next to overloaded constructors and constants.
    scale := 1.5;
    r := make_rect(.{ 1, 2 }, .{ PARTICLE_WIDTH, PARTICLE_HEIGHT } * scale);

    assert(r.x == 1 && r.y == 2 && r.w == 6 && r.h == 3);

    print("ok\n");
}
