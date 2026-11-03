#import "Basic";

scale :: (a: int) -> int {
    // jailint: allow(unused_variable)
    total := a * 2;

    return total;
}

main :: () {
    print("%\n", scale(4));
}
