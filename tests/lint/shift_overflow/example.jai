#import "Basic";

high_word :: (bits: u32) -> u32 {
    return bits << 32;
}

main :: () {
    print("%\n", high_word(1));
}
