byte_swap :: (input: $T) -> T #expand {
    result: T;
    source := cast(*u8) *input;
    dest := cast(*u8) *result;

    for i: 0..size_of(T) - 1 dest[i] = source[size_of(T) - 1 - i];

    return result;
}

byte_swap_in_place :: (input: *$T) #expand {
    input.* = byte_swap(input.*);
}

bit_scan_forward :: (input: $T) -> s32 #expand {
    value := get_u64_from_void_pointer(*input, size_of(T));
    if value == 0 return 0;

    index: s32 = 1;

    while (value & 1) == 0 {
        value >>= 1;
        index += 1;
    }

    return index;
}

bit_scan_reverse :: (input: $T) -> s32 #expand {
    value := get_u64_from_void_pointer(*input, size_of(T));
    index: s32;

    while value != 0 {
        value >>= 1;
        index += 1;
    }

    return index;
}

popcount :: (value: $T) -> s32 #expand {
    bits := get_u64_from_void_pointer(*value, size_of(T));
    count: s32;

    while bits != 0 {
        bits &= bits - 1;
        count += 1;
    }

    return count;
}

#scope_file
#import "Basic";
