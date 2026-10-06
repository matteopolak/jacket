// Found by fuzzing (lsp): a struct of 2^48 bytes, all zero by default, built its default
// image in memory and aborted the compiler.
Big :: struct {
    a: [1 << 47] u8;
    b: [1 << 47] u8;
}

main :: () {
    b: Big;
}
