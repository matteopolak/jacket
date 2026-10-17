// Original general assembler reproducer: requested memory-order instructions.
main :: () {
    #asm {
        mfence;
        lfence;
        sfence;
    }
}
