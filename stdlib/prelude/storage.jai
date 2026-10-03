// Canonical bootstrap storage declarations.
Newstring :: struct {
    count: s64;
    data: *u8;
}

Array_View_64 :: struct {
    count: s64;
    data: *u8;
}

Resizable_Array :: struct {
    count: s64;
    data: *void;
    allocated: s64;

    allocator: Allocator;
}

Any_Struct :: struct {
    type: *Type_Info;
    value_pointer: *void;
}

Procedure_With_Data :: struct {
    proc: *void;
    data: *void;
}

For_Flags :: enum_flags u8 {
    POINTER :: 0x1;
    REVERSE :: 0x2;
}
