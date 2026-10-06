// Found by fuzzing: field offsets past 2^64 overflowed during layout.
Big :: struct {
    a: [1 << 47] u8;
    b: [1 << 47] u8;
    c: [1 << 47] u8;
}

main :: () {
    b: Big;
}
