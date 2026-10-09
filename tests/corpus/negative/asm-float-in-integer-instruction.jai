// rules: asm.29
// A float variable is a vector register in #asm, so an integer instruction cannot take it.
main :: () {
    x: float32 = 1;
    y: float32 = 2;
    #asm { add x, y; }
}
