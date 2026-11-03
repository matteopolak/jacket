#import "Basic";

widen :: (a: s32) -> s64 {
    return cast,force(s64) a;
}

main :: () {
    print("%\n", widen(7));
}
