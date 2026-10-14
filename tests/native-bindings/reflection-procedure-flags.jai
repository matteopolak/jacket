// General compiler reproducer: erased procedure metadata must retain its flag.
Dispatch :: struct #type_info_procedures_are_void_pointers {
    raw: *void;
    invoke: (value: s32) -> s32 #c_call;
}

Ordinary :: struct {
    invoke: (value: s32) -> s32 #c_call;
}

libc :: #library,system "c";
abort :: () #foreign libc;
native_printf :: (fmt: *u8, args: ..Any) -> s32 #foreign libc "printf";
native_fflush :: (stream: *void) -> s32 #foreign libc "fflush";

check :: (value: bool) {
    if !value abort();
}

main :: () {
    found: s32;

    for member: type_info(Dispatch).members {
        if member.name == "raw" {
            check(member.type.type == .POINTER);
            check((member.flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) == 0);
            found += 1;
        } else if member.name == "invoke" {
            check(native_printf("erased procedure flags=%u\n", cast(u32) member.flags) >= 0);
            check(native_fflush(null) == 0);
            check(member.type.type == .POINTER);
            check((member.flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) != 0);
            found += 1;
        }
    }

    check(found == 2);
    found = 0;

    for member: type_info(Ordinary).members {
        if member.name == "invoke" {
            check(member.type.type == .PROCEDURE);
            check((member.flags & .PROCEDURE_WITH_VOID_POINTER_TYPE_INFO) == 0);
            found += 1;
        }
    }

    check(found == 1);
}
