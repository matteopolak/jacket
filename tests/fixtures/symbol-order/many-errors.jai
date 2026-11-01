// Many independent errors. Their text and order must not depend on how the compiler numbers
// the spellings.
Point :: struct {
    zulu: int;
    mike: int;
    alpha: int;
    quebec: int;
}

first :: (p: Point) -> int {
    return p.zlu + p.mik + p.alpa;
}

second :: () -> int {
    return unknown_one + unknown_two + unknown_three;
}

third :: () {
    a: Point;
    b: string = a;
    c: int = "text";
    d: float = a.quebec.nothing;
}

fourth :: (x: int) -> int {
    return x;
}

fourth :: (y: int) -> int {
    return y;
}

main :: () {
    first(1, 2);
    second(3);
    third();
}
