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
    cap: int;
    depth: int;
}

draw :: (using style: Style) -> int {
    return count + color + width;
}

// Only the members that are meant come in.
pick :: (n: int, using,only(color) style: Style) -> int {
    return n + color;
}

// The member that would collide is left out.
trim :: (count: int, using,except(count) style: Style) -> int {
    return count + color + width;
}

stack :: (using frame: Frame) -> int {
    return count + color + width;
}

nest :: (n: int) -> int {
    f: Frame;

    using f;

    return n + color;
}

cap :: (l: Limits) -> int {
    using l;

    return limit + depth + cap;
}

main :: () {
    f: Frame;
    s: Style;
    print("%\n", draw(s) + pick(1, s) + trim(1, s) + stack(f) + nest(2) + cap(.{ 1, 2 }));
}
