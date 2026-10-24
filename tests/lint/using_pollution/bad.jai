#import "Basic";

Style :: struct {
    count: int;
    color: int;
    width: int;
}

Frame :: struct {
    using style: Style;
    title: string;
}

limit := 10;

Limits :: struct {
    limit: int;
    depth: int;
}

draw :: (count: int, using style: Style) -> int {
    return count + color + width;
}

stack :: (count: int, using frame: Frame) -> int {
    return count + color + width;
}

check :: () -> int {
    count := 3;
    style: Style;

    using style;

    return width + color + count;
}

nest :: (width: int) -> int {
    f: Frame;

    using f;

    return width + color;
}

cap :: (l: Limits) -> int {
    using l;

    return limit + depth;
}

main :: () {
    f: Frame;
    print("%\n", draw(1, f.style) + stack(1, f) + check() + nest(2) + cap(.{ 1, 2 }));
}
