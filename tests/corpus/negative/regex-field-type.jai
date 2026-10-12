// Regex rejects this at compile time.
Regex :: #import "Extensions/Regex";

Point :: struct {
    x: [2] int;
}

main :: () {
    point: Point;
    Regex.parse_into(Regex.Static("(?<x>\\d+)"), "1", *point);
}
