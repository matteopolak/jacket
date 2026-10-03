Vector :: struct { x: int; y: int; }
operator + :: (a: Vector, b: Vector) -> Vector {
    return Vector.{x=a.x+b.x, y=a.y+b.y};
}
operator * :: (a: Vector, scale: int) -> Vector #symmetric {
    return Vector.{x=a.x*scale, y=a.y*scale};
}
operator == :: (a: Vector, b: Vector) -> bool {
    return a.x == b.x && a.y == b.y;
}
main :: () -> int {
    value := Vector.{x=2, y=3} + Vector.{x=5, y=7};
    value *= 3;
    reversed := 2 * Vector.{x=1, y=4};
    if value == Vector.{x=21, y=30} return value.x + reversed.y;
    return 0;
}
