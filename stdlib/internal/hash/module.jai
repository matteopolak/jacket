FNV_64_PRIME: u64 : 0x100000001b3;
FNV_64_OFFSET_BIAS: u64 : 0xcbf29ce484222325;

// A 32-bit hash of `size` bytes at `data`. Pass the previous result as `h` to hash several pieces as one.
//
// ```jai
// whole := "hello world";
// one := sdbm_hash(whole.data, whole.count);
// part := sdbm_hash(whole.data, 5);
// two := sdbm_hash(whole.data + 5, whole.count - 5, part);
// assert(one == two);
// ```
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

// A 32-bit hash of `count` floats starting at `f`. Negative zero hashes like zero.
//
// ```jai
// values := float.[1.5, 2.5];
// assert(sdbm_float_hash(values.data, 2) == sdbm_float_hash(values.data, 2));
// ```
sdbm_float_hash :: inline (f: *float, count: int, h := HASH_INIT) -> u32 {
    value := h;

    for i: 0..count - 1 {
        x := f[i];
        if x == 0 x = 0;
        value = sdbm_hash(*x, size_of(float), value);
    }

    return value;
}

// A 32-bit hash of a string.
//
// ```jai
// assert(djb2_hash("abc") == djb2_hash("abc"));
// assert(djb2_hash("abc") != djb2_hash("abd"));
// ```
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

// One step of 64-bit FNV-1a: mixes `val` into the hash `h`.
fnv1a_hash :: inline (val: u64, h := FNV_64_OFFSET_BIAS) -> u64 #no_aoc {
    return (val ^ h) * FNV_64_PRIME;
}

// A 64-bit FNV-1a hash of `size` bytes at `data`. Pass a previous result as `h` to continue it.
//
// ```jai
// text := "jacket";
// assert(fnv1a_hash(text.data, text.count) == fnv1a_hash(text.data, text.count));
// ```
fnv1a_hash :: (data: *void, size: int, h := FNV_64_OFFSET_BIAS) -> u64 #no_aoc {
    value := h;
    bytes := cast(*u8) data;
    for i: 0..size - 1 value = fnv1a_hash(cast(u64) bytes[i], value);
    return value;
}

// Scrambles an integer by multiplying it with a large odd constant. The high bits of the result are the well mixed ones.
//
// ```jai
// assert(knuth_hash(1) != knuth_hash(2));
// ```
knuth_hash :: inline (x: u64) -> u64 #no_aoc {
    KNUTH_MULTIPLIER: u64 : 11400714819323198485;

    return x * KNUTH_MULTIPLIER;
}

// A 32-bit hash of a string. This is the hash `Table` uses for string keys.
//
// ```jai
// assert(get_hash("key") == get_hash("key"));
// assert(get_hash("key") != get_hash("other"));
// ```
get_hash :: (s: string, h := HASH_INIT) -> u32 {
    return cast,trunc(u32) fnv1a_hash(s.data, s.count, cast(u64) h);
}

// A 32-bit hash of the bytes of an array.
//
// ```jai
// values := int.[1, 2, 3];
// assert(get_hash(values) == get_hash(int.[1, 2, 3]));
// ```
get_hash :: (array: [] $T, h := HASH_INIT) -> u32 {
    return sdbm_hash(array.data, array.count * size_of(T), h);
}

// A 32-bit hash of an integer, boolean, enum, pointer or float. Pass a previous result as `h` to combine several values.
//
// ```jai
// combined := get_hash(7, get_hash(42));
// assert(combined == get_hash(7, get_hash(42)));
// ```
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
