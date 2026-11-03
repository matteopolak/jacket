#import "Basic";

Point :: struct {
    x: int;
    y: int;
}

// Scales a [Point] and prints it with [print].
scale :: (p: Point) {
    print("%\n", p.x * 2);
}

main :: () {
    scale(.{ 1, 2 });
}
