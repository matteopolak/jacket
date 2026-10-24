#import "Basic";
#import "String";

describe :: (n: int) -> bool {
    // Compared and lost.
    if sprint("%", n) == "7" return true;
    return false;
}

width :: (a: string, b: string) -> int {
    // Measured and lost.
    return join(a, b).count;
}

first :: (n: int) -> u8 {
    // Indexed and lost.
    return sprint("%", n)[0];
}

main :: () {
    // Printed and lost.
    print("%\n", sprint("%", 42));
    print("%\n", describe(7));
    print("%\n", width("a", "b"));
    print("%\n", first(5));
}
