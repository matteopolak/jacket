// rules: num.20
// A negative constant never converts to an unsigned type, even where its bits would fit.
main :: () {
    all_bits: u32 = -1;
}
