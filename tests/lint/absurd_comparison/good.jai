#import "Basic";

check :: (index: s64, size: u32, small: s8) -> bool {
    if index < 0 return false; // signed: can be negative
    if size > 0 return true; // unsigned, but not at the bottom
    if small <= -128 return false; // can be equal
    return 0 <= index;
}

main :: () {
    print("%\n", check(1, 2, 3));
}
