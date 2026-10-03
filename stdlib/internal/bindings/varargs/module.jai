// Runtime Any values require dynamic C ABI marshalling, rather than passing a
// Jai Any-array descriptor as a native variadic argument.
Argument :: struct {
    kind: u32;
    reserved: u32;
    value: u64;
}

invoke_int :: (function: *void, fixed: [] Argument, dynamic: [] Any) -> s32 {
    return cast,trunc(s32) invoke(function, 0, fixed, dynamic);
}

invoke_pointer :: (function: *void, fixed: [] Argument, dynamic: [] Any) -> *u8 {
    return cast(*u8) invoke(function, 1, fixed, dynamic);
}

invoke_void :: (function: *void, fixed: [] Argument, dynamic: [] Any) {
    invoke(function, 2, fixed, dynamic);
}

invoke_bool :: (function: *void, fixed: [] Argument, dynamic: [] Any) -> bool {
    return invoke(function, 3, fixed, dynamic) != 0;
}

#scope_file
Native :: #import "internal/bindings/native";

bridge :: #library,system "binding_ffi";
#library,system,link_always "ffi";
libc :: #library,system "c";
abort :: () #foreign libc;

call_native :: (function: *void, result_kind: s32, fixed_count: u32, arguments: *Argument,
                count: u32, result: *u64) -> s32 #foreign bridge "binding_ffi_call";

describe :: (arg: Any) -> Argument {
    result: Argument;
    info := arg.type;

    while info.type == .VARIANT info = (cast(*Type_Info_Variant) info).variant_of;
    if info.type == .ENUM info = cast(*Type_Info) (cast(*Type_Info_Enum) info).internal_type;

    if info.type == .INTEGER {
        integer := cast(*Type_Info_Integer) info;

        result.kind = ifx info.runtime_size < 4 || integer.signed then 1 else 2;
        if info.runtime_size == 8 result.kind += 2;

        if integer.signed {
            if info.runtime_size == 1 result.value = cast,trunc(u64) <<cast(*s8) arg.value_pointer;
            else if info.runtime_size == 2 result.value = cast,trunc(u64) <<cast(*s16) arg.value_pointer;
            else if info.runtime_size == 4 result.value = cast,trunc(u64) <<cast(*s32) arg.value_pointer;
            else if info.runtime_size == 8 result.value = cast,trunc(u64) <<cast(*s64) arg.value_pointer;
            else abort();
        } else {
            if info.runtime_size == 1 result.value = <<cast(*u8) arg.value_pointer;
            else if info.runtime_size == 2 result.value = <<cast(*u16) arg.value_pointer;
            else if info.runtime_size == 4 result.value = <<cast(*u32) arg.value_pointer;
            else if info.runtime_size == 8 result.value = <<cast(*u64) arg.value_pointer;
            else abort();
        }
    } else if info.type == .FLOAT {
        number: float64;
        if info.runtime_size == 4 number = cast(float64) <<cast(*float32) arg.value_pointer;
        else if info.runtime_size == 8 number = <<cast(*float64) arg.value_pointer;
        else abort();

        result.kind = 5;
        result.value = <<cast(*u64) *number;
    } else if info.type == .BOOL {
        result.kind = 1;
        result.value = cast(u64) <<cast(*bool) arg.value_pointer;
    } else if info.type == .POINTER || info.type == .PROCEDURE {
        result.kind = 6;
        result.value = <<cast(*u64) arg.value_pointer;
    } else if info.type == .STRING {
        result.kind = 6 | 256;
        result.value = cast(u64) Native.terminated_copy(<<cast(*string) arg.value_pointer);
    } else {
        abort();
    }

    return result;
}

invoke :: (function: *void, result_kind: s32, fixed: [] Argument, dynamic: [] Any) -> u64 {
    count := fixed.count + dynamic.count;
    if count > 0xffff_ffff abort();

    arguments := cast(*Argument) Native.allocate(count * size_of(Argument));

    defer Native.release(arguments);
    for i: 0..fixed.count - 1 arguments[i] = fixed[i];
    for i: 0..dynamic.count - 1 arguments[fixed.count + i] = describe(dynamic[i]);

    defer {
        for i: fixed.count..count - 1 if (arguments[i].kind & 256) != 0 Native.release(cast(*void) arguments[i].value);
    }

    result: u64;
    status := call_native(function, result_kind, cast(u32) fixed.count,
                          arguments, cast(u32) count, *result);

    if status != 0 abort();

    return result;
}
