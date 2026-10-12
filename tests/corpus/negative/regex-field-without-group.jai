// Regex rejects this at compile time.
Regex :: #import "Extensions/Regex";

Point :: struct {
    x: int;
    y: int;
}

main :: () {
    point: Point;
    Regex.parse_into(Regex.Static("(?<x>\\d+)"), "12", *point);
}
