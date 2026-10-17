// Original reduction: explicitly valued enum members can depend on later members.
Flags :: enum_flags u32 {
    FIRST :: 1;
    MASK :: FIRST | LAST;
    LAST :: 4;
}

#assert Flags.MASK == xx 5;

main :: () -> s32 {
    return cast(s32) Flags.MASK;
}
