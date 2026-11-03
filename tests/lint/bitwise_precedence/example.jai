#import "Basic";

low_bits :: (bits: int) -> int {
    return 1 << bits - 1;
}

main :: () {
    print("%\n", low_bits(4));
}
