#import "Basic";

high_word :: (bits: u64) -> u64 {
    return bits << 32;
}

main :: () {
    print("%\n", high_word(1));
}
