// Several constants of one name can each be `#procedure_of_call` of a different instance of a
// polymorphic procedure, next to ordinary procedures of that name: they all join one overload set.
#import "Basic";

V2 :: struct {
    x: float;
    y: float;
}

Range :: struct($T: Type) {
    min: T;
    max: T;
}

MakeRange :: (max: $T) -> Range(T) {
    return Range(T).{ max = max };
}

MakeRange :: (min: $T, max: T) -> Range(T) {
    return Range(T).{ min, max };
}

Rect :: Range(V2);
MakeRect :: #procedure_of_call MakeRange(V2.{});
MakeRect :: #procedure_of_call MakeRange(V2.{}, V2.{});

MakeRect :: (max_x: float, max_y: float) -> Rect {
    return .{ max = .{ max_x, max_y } };
}

MakeRect :: (x: float, y: float, max_x: float, max_y: float) -> Rect {
    return .{ .{ x, y }, .{ max_x, max_y } };
}

main :: () {
    a := MakeRect(V2.{ 1, 2 });

    assert(a.min.x == 0 && a.max.x == 1 && a.max.y == 2);
    b := MakeRect(V2.{ 1, 2 }, V2.{ 3, 4 });

    assert(b.min.x == 1 && b.max.y == 4);
    c := MakeRect(5.0, 6.0);

    assert(c.min.y == 0 && c.max.x == 5 && c.max.y == 6);
    d := MakeRect(1.0, 2.0, 3.0, 4.0);

    assert(d.min.y == 2 && d.max.x == 3);
}
