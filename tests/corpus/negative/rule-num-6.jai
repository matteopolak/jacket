// rules: num.6
// Implicit narrowing from u16 to u8 is a compile error.
main :: () {
    wide: u16 = 1000;
    narrow: u8 = wide;
}
