// Found by fuzzing: building the default value of a huge struct aborted the compiler.
Big :: struct {
    a: [1 << 40] u8;
    x := 5;
}

main :: () {
    b: Big;
}
