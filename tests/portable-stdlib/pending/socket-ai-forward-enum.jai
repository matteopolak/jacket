// Original general reduction: preserve enum declaration/reflection order.
Flags :: enum_flags u32 {
    FIRST :: 1;
    BOTH :: FIRST | LATER;
    LATER :: 2;
}

#assert cast(u32) Flags.BOTH == 3;
main :: () {}
