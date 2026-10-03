Relative_Pointer :: struct(Storage_Type: Type, Pointer_To_Type: Type) {
    value: Storage_Type;
    #assert Storage_Type == s8 || Storage_Type == s16 || Storage_Type == s32 || Storage_Type == s64;
}

Relative_String :: struct(Storage_Type: Type) {
    count: Storage_Type;
    data: Storage_Type;
    #assert Storage_Type == s8 || Storage_Type == s16 || Storage_Type == s32 || Storage_Type == s64;
}

decode :: inline (p: *Relative_Pointer) -> (result: *p.Pointer_To_Type) {
    pointer := decode_relative_pointer(cast(s32) size_of(p.Storage_Type), *p.value);
    return cast(*p.Pointer_To_Type) pointer;
}

encode :: (p: *Relative_Pointer, absolute: *p.Pointer_To_Type) {
    write_relative_pointer(p.Storage_Type, *p.value, absolute);
}

decode :: (p: *Relative_String) -> string {
    pointer := decode_relative_pointer(cast(s32) size_of(p.Storage_Type), *p.data);
    return .{ cast(s64) p.count, cast(*u8) pointer };
}

decode_relative_pointer :: (relative_pointer_size: s32, address: *void) -> (pointer: *void, offset: s64) {
    return decode_relative_pointer(relative_pointer_size, address, address);
}

decode_relative_pointer :: (relative_pointer_size: s32, pointer_address: *void, base_address: *void) -> (pointer: *void, offset: s64) #no_aoc {
    value := get_s64_from_void_pointer(pointer_address, relative_pointer_size);
    if value == 0 return null, 0;

    mask := signed_mask(relative_pointer_size);
    offset := value ^ mask;

    return cast(*void) (cast(s64) base_address + offset), offset;
}

decode_relative_pointer_static :: ($type: Type, address: *void) -> (pointer: *void) {
    pointer := decode_relative_pointer(cast(s32) size_of(type), address);
    return pointer;
}

write_relative_pointer :: ($type: Type, pointer_address: *void, target_address: *void) {
    write_relative_pointer(type, pointer_address, pointer_address, target_address);
}

write_relative_pointer :: ($type: Type, pointer_address: *void, base_address: *void, target_address: *void) #no_aoc {
    if !target_address {
        (cast(*type) pointer_address).* = 0;
        return;
    }

    displacement := cast(s64) target_address - cast(s64) base_address;
    encoded := displacement ^ signed_mask(cast(s32) size_of(type));

    (cast(*type) pointer_address).* = cast,truncate(type) encoded;
}

#scope_file
#import "Basic";

signed_mask :: (size: s32) -> s64 {
    assert(size == 1 || size == 2 || size == 4 || size == 8);
    if size == 8 return -9223372036854775808;
    return -((cast(s64) 1) << (size * 8 - 1));
}
