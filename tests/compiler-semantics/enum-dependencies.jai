// Original forward enum values: dependency order differs from declaration order.
Flags :: enum_flags u32 {
    FIRST;
    MASK :: FIRST | LAST;
    #if true {
        LAST :: 4;
    }
    #insert "NEXT; ALIAS :: FIRST;";
    AFTER;
}

Number :: enum s32 {
    FIRST :: Number.LAST + 3;
    SECOND;
    LAST :: 7;
    AFTER;
}

Alias :: Flags;

Record :: struct {
    flags: Alias = .MASK;
    number: Number = .SECOND;
}

#program_export "enum_dependency_check"
enum_dependency_check :: () -> s32 #c_call {
    value: Record;
    if value.flags != .MASK || value.number != .SECOND return 1;
    if (value.flags & .LAST) != .LAST return 2;

    value.flags |= .NEXT;
    if (cast(u32) value.flags) != 13 return 3;
    if Number.FIRST != xx 10 || Number.SECOND != xx 11 return 4;
    if Number.AFTER != xx 8 || Flags.AFTER != xx 2 return 5;
    if Alias != Flags return 6;

    return 0;
}

main :: () -> s32 {
    return enum_dependency_check();
}
