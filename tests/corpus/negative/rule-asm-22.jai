// rules: asm.22
// An instruction #asm does not support is a compile error naming it.
main :: () {
    x: u64 = 1;
    #asm { push x; }
}
