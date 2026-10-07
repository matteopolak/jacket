#import "Basic";

place :: (x: s64, y: s64) -> s64 {
    return x * 1000 + y;
}

place :: (x: float, y: float) -> s64 {
    return -1;
}

place :: (s: string) -> s64 {
    return -2;
}

main :: () {
    h: float = 7.5;
    assert(place(xx (h * 2), xx (h / 2) + 1) == 15004);
    print("ok\n");
}
