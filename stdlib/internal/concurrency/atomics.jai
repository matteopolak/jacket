IsAtomicScalar :: (T: Type) -> bool #compile_time {
    info := cast(*Type_Info) T;
    size := info.runtime_size;

    if size != 1 && size != 2 && size != 4 && size != 8 return false;

    return info.type == .INTEGER || info.type == .BOOL || info.type == .FLOAT ||
           info.type == .POINTER || info.type == .ENUM || info.type == .PROCEDURE;
}

atomic_add :: (dest: *$T, to_add: $I) -> T #no_aoc #modify {
    info := cast(*Type_Info) T;
    increment := cast(*Type_Info) I;
    return IsAtomicScalar(T) && (info.type == .INTEGER || info.type == .POINTER) &&
           increment.type == .INTEGER, "atomic_add requires an integer/pointer target and integer increment";
} {
    U :: #run word_type(T);
    pointer := cast(*U) dest;

    require_alignment(pointer);
    _, old := compare_and_swap(pointer, 0, 0);
    success := false;

    while !success {
        next := old + cast,no_check(U) to_add;
        observed: U;
        success, observed = compare_and_swap(pointer, old, next);
        if !success old = observed;
    }

    return from_word(T, old);
}

atomic_increment :: (dest: *$T) #modify {
    info := cast(*Type_Info) T;
    return IsAtomicScalar(T) && (info.type == .INTEGER || info.type == .POINTER), "atomic_increment requires an integer/pointer target";
} {
    atomic_add(dest, 1);
}

atomic_decrement :: (dest: *$T) -> bool #modify {
    info := cast(*Type_Info) T;
    return IsAtomicScalar(T) && (info.type == .INTEGER || info.type == .POINTER), "atomic_decrement requires an integer/pointer target";
} {
    old := atomic_add(dest, -1);
    return to_word(old) == 1;
}

atomic_and :: (dest: *$T, value: T) #modify {
    return bit_target(T), "atomic_and requires an integer or enum scalar";
} {
    update_bits(dest, to_word(value), .AND);
}

atomic_or :: (dest: *$T, value: T) #modify {
    return bit_target(T), "atomic_or requires an integer or enum scalar";
} {
    update_bits(dest, to_word(value), .OR);
}

atomic_xor :: (dest: *$T, value: $I) #modify {
    info := cast(*Type_Info) I;
    return bit_target(T) && (info.type == .INTEGER || info.type == .ENUM), "atomic_xor requires integer/enum scalars";
} {
    U :: #run word_type(T);
    update_bits(dest, cast,no_check(U) value, .XOR);
}

atomic_bit_test_and_set :: (dest: *$T, $$i: s64) -> bool {
    #assert bit_target(T);
    #assert size_of(T) == 2 || size_of(T) == 4 || size_of(T) == 8, "atomic bit test requires a 2-, 4-, or 8-byte target";
    bit := i;
    if bit < 0 bit += size_of(T) * 8;

    assert(bit >= 0 && bit < size_of(T) * 8, "atomic bit index out of range");
    U :: #run word_type(T);
    mask := (cast(U) 1) << bit;
    prior := update_bits(dest, mask, .OR);

    return (prior & mask) != 0;
}

#scope_file
#import "Basic";

Bit_Operation :: enum {
    AND;
    OR;
    XOR;
}

bit_target :: (T: Type) -> bool #compile_time {
    info := cast(*Type_Info) T;
    return IsAtomicScalar(T) && (info.type == .INTEGER || info.type == .ENUM);
}

word_type :: (T: Type) -> Type #compile_time {
    size := (cast(*Type_Info) T).runtime_size;
    if size == 1 return u8;
    if size == 2 return u16;
    if size == 4 return u32;

    assert(size == 8, "unsupported atomic scalar size");

    return u64;
}

require_alignment :: (pointer: *$U) {
    assert(pointer != null && cast(u64) pointer % size_of(U) == 0, "atomic target must be nonnull and naturally aligned");
}

from_word :: ($T: Type, word: $U) -> T {
    #assert size_of(T) == size_of(U);
    result: T = ---;
    memcpy(*result, *word, size_of(T));
    return result;
}

to_word :: (value: $T) -> #run word_type(T) {
    U :: #run word_type(T);
    result: U;
    memcpy(*result, *value, size_of(T));
    return result;
}

update_bits :: (dest: *$T, value: $U, $operation: Bit_Operation) -> U {
    #assert size_of(T) == size_of(U);
    pointer := cast(*U) dest;

    require_alignment(pointer);
    _, old := compare_and_swap(pointer, 0, 0);
    success := false;

    while !success {
        next: U;

        #if operation == .AND next = old & value;
        #if operation == .OR next = old | value;
        #if operation == .XOR next = old ^ value;
        observed: U;
        success, observed = compare_and_swap(pointer, old, next);
        if !success old = observed;
    }

    return old;
}
