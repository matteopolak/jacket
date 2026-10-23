FNV_64_PRIME: u64 : 0x100000001b3;
FNV_64_OFFSET_BIAS: u64 : 0xcbf29ce484222325;

sdbm_hash :: (data: *void, size: int, h := HASH_INIT) -> u32 #no_aoc {
    // Each step is value * 65599 + byte. Wrapping arithmetic lets four steps fold into one sum of
    // independent products, which takes fewer instructions than four dependent steps.
    M: u32 : 65599;
    M2: u32 : 0x7e0f81;
    M3: u32 : 0x2e86d0bf;
    M4: u32 : 0x43ec5f01; // the powers of M, wrapped to 32 bits
    value := h;
    bytes := cast(*u8) data;
    i := 0;

    while i + 4 <= size {
        value = value * M4 + cast(u32) bytes[i] * M3 + cast(u32) bytes[i + 1] * M2 + cast(u32) bytes[i + 2] * M + cast(u32) bytes[i + 3];
        i += 4;
    }

    // At most three bytes are left: two steps, then one, without a loop.
    if size - i >= 2 {
        value = value * M2 + cast(u32) bytes[i] * M + cast(u32) bytes[i + 1];
        i += 2;
    }

    if i < size value = value * M + cast(u32) bytes[i];

    return value;
}

sdbm_float_hash :: inline (f: *float, count: int, h := HASH_INIT) -> u32 {
    value := h;

    for i: 0..count - 1 {
        x := f[i];
        if x == 0 x = 0;
        value = sdbm_hash(*x, size_of(float), value);
    }

    return value;
}

djb2_hash :: (s: string) -> u32 #no_aoc {
    // Four steps of value * 33 + byte at a time, folded as in sdbm_hash.
    M: u32 : 33;
    value: u32 = 5381;
    i := 0;

    while i + 4 <= s.count {
        value = value * (M * M * M * M) + cast(u32) s.data[i] * (M * M * M) + cast(u32) s.data[i + 1] * (M * M) + cast(u32) s.data[i + 2] * M + cast(u32) s.data[i + 3];
        i += 4;
    }

    if s.count - i >= 2 {
        value = value * (M * M) + cast(u32) s.data[i] * M + cast(u32) s.data[i + 1];
        i += 2;
    }

    if i < s.count value = value * M + cast(u32) s.data[i];

    return value;
}

fnv1a_hash :: inline (val: u64, h := FNV_64_OFFSET_BIAS) -> u64 #no_aoc {
    return (val ^ h) * FNV_64_PRIME;
}

fnv1a_hash :: (data: *void, size: int, h := FNV_64_OFFSET_BIAS) -> u64 #no_aoc {
    value := h;
    bytes := cast(*u8) data;
    for i: 0..size - 1 value = fnv1a_hash(cast(u64) bytes[i], value);
    return value;
}

knuth_hash :: inline (x: u64) -> u64 #no_aoc {
    return x * 11400714819323198485;
}

get_hash :: (s: string, h := HASH_INIT) -> u32 {
    return cast,trunc(u32) fnv1a_hash(s.data, s.count, cast(u64) h);
}

get_hash :: (array: [] $T, h := HASH_INIT) -> u32 {
    return sdbm_hash(array.data, array.count * size_of(T), h);
}

get_hash :: (x: $T, h := HASH_INIT) -> u32 #modify {
    info := hash_scalar_base_info(type_info(T));
    return info.type == .FLOAT || info.type == .INTEGER || info.type == .BOOL || info.type == .ENUM || info.type == .POINTER, "get_hash requires an integer, boolean, enum, pointer, or float";
} {
    #if hash_scalar_base_info(type_info(T)).type == .FLOAT {
        // Equal keys hash alike: negative zero is folded into zero.
        key := x;
        if key == 0 key = 0;

        return sdbm_hash(*key, size_of(T), h);
    } else return cast(u32) (knuth_hash((cast,no_check(u64) x) ^ cast(u64) h) >> 32);
}

#scope_file
HASH_INIT: u32 : 5381;

hash_scalar_base_info :: (info: *Type_Info) -> *Type_Info {
    while info.type == .VARIANT info = (cast(*Type_Info_Variant) info).variant_of;
    return info;
}
