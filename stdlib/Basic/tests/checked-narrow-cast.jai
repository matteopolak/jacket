// Original ABI mistake must remain a checked-cast error outside the decoder.
#import "Basic";

main :: () {
    encoded: u64 = 65416;
    incorrectly_narrowed := cast(s16) encoded;
    print("%\n", incorrectly_narrowed);
}
