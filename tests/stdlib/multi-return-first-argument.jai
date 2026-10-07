#import "Basic";

two :: () -> int, string {
    return 7, "x";
}

one :: (a: int) -> int {
    return a * 2;
}

pair :: (a: int, b: string) -> int {
    return a + b.count;
}

main :: () {
    assert(one(two()) == 14);
    assert(pair(two()) == 8);
    print("ok\n");
}
