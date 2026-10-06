// rules: asm.23
// Jai #asm has no x87 instructions.
main :: () {
    f: float64 = 1;
    p := *f;
    #asm { fld [p]; }
}
