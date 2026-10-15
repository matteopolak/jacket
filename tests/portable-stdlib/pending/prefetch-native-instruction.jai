// Original general assembler reproducer; no stdlib module/symbol dependency.
main :: () {
    data: u8 = 73;
    pointer := *data;
    #asm { prefetcht0 [pointer]; }
}
