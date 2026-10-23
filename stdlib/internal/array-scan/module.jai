// Searches for one item in an array of integers, enums or pointers, a block at a time. A block
// is compared with a loop that has no early exit, which LLVM turns into vector compares, and
// only a block that holds the item is searched item by item. Basic uses it, so it imports nothing.

// True for the element types find compares by their bits: the same as ==.
is_scannable :: ($T: Type) -> bool {
    info := type_info(T);
    if info.type != .INTEGER && info.type != .ENUM && info.type != .POINTER return false;
    return size_of(T) == 1 || size_of(T) == 2 || size_of(T) == 4 || size_of(T) == 8;
}

// The index of the first element equal to `item`, or -1.
find :: no_inline (data: *$T, count: s64, item: T) -> s64 {
    #if size_of(T) == 1 return find_bits(cast(*u8) data, count, bits_of(u8, item));
    #if size_of(T) == 2 return find_bits(cast(*u16) data, count, bits_of(u16, item));
    #if size_of(T) == 4 return find_bits(cast(*u32) data, count, bits_of(u32, item));
    #if size_of(T) == 8 return find_bits(cast(*u64) data, count, bits_of(u64, item));

    return -1;
}

#scope_file

// Bytes compared per block.
BLOCK_BYTES :: 128;

bits_of :: inline ($U: Type, item: $T) -> U {
    result: U;
    memcpy(*result, *item, size_of(U));
    return result;
}

find_bits :: (data: *$U, count: s64, item: U) -> s64 {
    BLOCK :: BLOCK_BYTES / size_of(U);

    i := 0;

    while i + BLOCK <= count {
        hit: U = 0;
        for k: 0..BLOCK - 1 hit |= cast(U) (data[i + k] == item);

        if hit != 0 {
            for k: 0..BLOCK - 1 if data[i + k] == item return i + k;
        }

        i += BLOCK;
    }

    while i < count {
        if data[i] == item return i;
        i += 1;
    }

    return -1;
}
