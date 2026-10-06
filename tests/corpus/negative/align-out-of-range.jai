// Found by fuzzing: `#align -8` made a struct of 2^64 bytes.
S :: struct {
    a: int #align -8;
}

main :: () {
    s: S;
}
