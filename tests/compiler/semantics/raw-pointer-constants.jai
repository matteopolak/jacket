// Original opaque scalar pointer values; never dereference these handles.
HANDLE :: *void;
HKEY :: HANDLE;
Handle_Alias :: HKEY;
ROOT :: cast(HKEY) 0x80000001;
OTHER :: cast(Handle_Alias) 0x80000060;

Record :: struct {
    handle: HKEY = ROOT;
}

global_handle: HKEY = ROOT;
storage: u32 = 71;

storage_address :: () -> *void #no_context {
    return cast(*void) *storage;
}

RELOCATED :: #run storage_address();

read_default :: (handle: HKEY = ROOT) -> u64 #no_context {
    return cast(u64) handle;
}

make_handle :: () -> HKEY #no_context {
    return cast(HKEY) 0x80000050;
}

make_record :: () -> Record #no_context {
    return .{ handle = OTHER };
}

RUN_HANDLE :: #run make_handle();

baked_bits :: ($value: $T) -> u64 #no_context {
    #assert T == HKEY;
    return cast(u64) value;
}

binding_kind :: ($value:) -> int #no_context {
    if type_of(value) == HKEY return 1;
    return 0;
}

RUN_RECORD :: #run make_record();

#assert type_of(ROOT) == HKEY && HKEY == Handle_Alias;
#assert ROOT != null && ROOT != OTHER;
#assert (cast(u64) ROOT) == 0x80000001;
#assert (cast(u64) RUN_HANDLE) == 0x80000050;
#assert (cast(u64) RUN_RECORD.handle) == 0x80000060;
#assert read_default() == 0x80000001;

exported_handle: HKEY = ROOT;

#program_export "opaque_handle_value"
opaque_handle_value :: () -> u64 #c_call {
    return cast(u64) exported_handle;
}

#program_export "raw_pointer_check"
check :: () -> s32 #c_call {
    value: Record;
    if value.handle != ROOT || global_handle != ROOT return 1;
    if read_default() != 0x80000001 return 2;
    if (cast(u64) RUN_HANDLE) != 0x80000050 return 3;
    if (cast(u64) RUN_RECORD.handle) != 0x80000060 return 4;

    integer: u64 = cast(u64) OTHER;
    handle := cast(HKEY) integer;

    if handle != OTHER || handle == ROOT || handle == null return 5;

    typed :: cast(*u32) 0x80000002;
    if (cast(u64) typed) != 0x80000002 return 6;
    if RELOCATED != cast(*void) *storage return 7;
    if (cast(*u32) RELOCATED).* != 71 return 8;
    if baked_bits(ROOT) != 0x80000001 return 9;
    if binding_kind(0x80000001) != 0 || binding_kind(ROOT) != 1 return 10;

    return 0;
}

main :: () -> s32 {
    return check();
}
