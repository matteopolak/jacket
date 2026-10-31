// A pointer to a `Pointer_To_Type`, stored as a signed `Storage_Type` (`s8` to `s64`) offset from its own address. An offset of zero means null, so a pointer to its own address cannot be stored.
Relative_Pointer :: struct(Storage_Type: Type, Pointer_To_Type: Type) {
    value: Storage_Type;
    #assert Storage_Type == s8 || Storage_Type == s16 || Storage_Type == s32 || Storage_Type == s64;
}

// A string stored as a length and a relative pointer to its bytes, each a `Storage_Type`.
Relative_String :: struct(Storage_Type: Type) {
    count: Storage_Type;
    data: Storage_Type;
    #assert Storage_Type == s8 || Storage_Type == s16 || Storage_Type == s32 || Storage_Type == s64;
}

// Returns the pointer a [Relative_Pointer] holds, or null if it is unset.
//
// ```jai
// target: u8 = 5;
// holder: struct { pointer: Relative_Pointer(s32, u8); };
// encode(*holder.pointer, *target);
// assert(decode(*holder.pointer) == *target);
// ```
decode :: inline (p: *Relative_Pointer) -> (result: *p.Pointer_To_Type) {
    pointer := decode_relative_pointer(cast(s32) size_of(p.Storage_Type), *p.value);
    return cast(*p.Pointer_To_Type) pointer;
}

// Points `p` at `absolute`, or clears it when `absolute` is null. The target must be within range of what `Storage_Type` can reach.
encode :: (p: *Relative_Pointer, absolute: *p.Pointer_To_Type) {
    write_relative_pointer(p.Storage_Type, *p.value, absolute);
}

// Returns the string a [Relative_String] holds.
//
// ```jai
// blob: struct { text: Relative_String(s16); bytes: [3] u8; };
// blob.bytes = .[104, 105, 33];
// blob.text.count = 3;
// write_relative_pointer(s16, *blob.text.data, blob.bytes.data);
// assert(decode(*blob.text) == "hi!");
// ```
decode :: (p: *Relative_String) -> string {
    pointer := decode_relative_pointer(cast(s32) size_of(p.Storage_Type), *p.data);
    return .{ cast(s64) p.count, cast(*u8) pointer };
}

// Reads the relative pointer of `relative_pointer_size` bytes (1, 2, 4 or 8) at `address`. Returns the pointer, or null if it is unset, and its offset from `address`.
decode_relative_pointer :: (relative_pointer_size: s32, address: *void) -> (pointer: *void, offset: s64) {
    return decode_relative_pointer(relative_pointer_size, address, address);
}

// Like the two-argument form, but the offset counts from `base_address` instead of from where the pointer is stored.
decode_relative_pointer :: (relative_pointer_size: s32, pointer_address: *void, base_address: *void) -> (pointer: *void, offset: s64) #no_aoc {
    value := get_s64_from_void_pointer(pointer_address, relative_pointer_size);
    if value == 0 return null, 0;

    mask := signed_mask(relative_pointer_size);
    offset := value ^ mask;

    return cast(*void) (cast(s64) base_address + offset), offset;
}

// Reads the relative pointer at `address`, taking its size from the integer type `type`.
decode_relative_pointer_static :: ($type: Type, address: *void) -> (pointer: *void) {
    pointer := decode_relative_pointer(cast(s32) size_of(type), address);
    return pointer;
}

// Stores a relative pointer of integer type `type` at `pointer_address` that leads to `target_address`; null clears it.
//
// ```jai
// slot: s16;
// value: u8;
// write_relative_pointer(s16, *slot, *value);
// assert(decode_relative_pointer_static(s16, *slot) == *value);
// ```
write_relative_pointer :: ($type: Type, pointer_address: *void, target_address: *void) {
    write_relative_pointer(type, pointer_address, pointer_address, target_address);
}

// Like the three-argument form, but the offset counts from `base_address` instead of from where the pointer is stored.
write_relative_pointer :: ($type: Type, pointer_address: *void, base_address: *void, target_address: *void) #no_aoc {
    if !target_address {
        (cast(*type) pointer_address).* = 0;
        return;
    }

    displacement := cast(s64) target_address - cast(s64) base_address;
    encoded := displacement ^ signed_mask(cast(s32) size_of(type));

    (cast(*type) pointer_address).* = cast,trunc(type) encoded;
}

#scope_file
#import "Basic";

signed_mask :: (size: s32) -> s64 {
    assert(size == 1 || size == 2 || size == 4 || size == 8);
    if size == 8 return -9223372036854775808;
    return -((cast(s64) 1) << (size * 8 - 1));
}
