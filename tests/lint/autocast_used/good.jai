#import "Basic";

take :: (x: u8) -> u8 {
    return x;
}

narrow :: (big: s64) -> u8 {
    small: u8 = cast(u8) big;

    // Arguments belong to `autocast_in_arg`.
    return take(xx big) + small;
}

main :: () {
    print("%\n", narrow(7));
}
