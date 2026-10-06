// Between float32 and float64 overloads an untyped literal picks float32.
#import "Basic";

lg :: (x: float64) -> float64 {
    return 64;
}
lg :: (x: float32) -> float32 {
    return 32;
}
pick :: (x: s32) -> int {
    return 1;
}
pick :: (x: float64) -> int {
    return 2;
}

main :: () {
    a := lg(2);
    b := lg(2.5);
    if type_of(a) != float32 || type_of(b) != float32 exit(1);
    if pick(3) != 1 exit(2);
    print("ok\n");
}
