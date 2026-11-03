#import "Basic";

widen :: (a: s32) -> s64 {
    return cast(s64) a;
}

main :: () {
    print("%\n", widen(7));
}
