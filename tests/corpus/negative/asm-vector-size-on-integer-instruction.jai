// rules: asm.29
// `.x`/`.y`/`.z` are vector sizes; an integer instruction takes `.b/.w/.d/.q`.
main :: () {
    a: u64 = 1;
    b: u64 = 2;
    #asm { add.x a, b; }
}
