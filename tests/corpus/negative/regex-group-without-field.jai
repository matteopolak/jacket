// Regex rejects this at compile time.
Regex :: #import "Extensions/Regex";

Point :: struct {
    x: int;
}

main :: () {
    point: Point;
    Regex.parse_into(Regex.Static("(?<x>\\d+),(?<y>\\d+)"), "1,2", *point);
}
