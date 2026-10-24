#import "Basic";

narrow :: (big: s64) -> u8 {
    small: u8 = xx big;

    other: u8;
    other = xx big;

    if small > 3 return xx (big + 1);

    return small + other;
}

main :: () {
    print("%\n", narrow(7));
}
