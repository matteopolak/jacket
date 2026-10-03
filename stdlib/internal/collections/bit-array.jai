Bit_Array :: struct {
    slots: [] s64;
    count: int;
    allocator: Allocator;
}

init_bit_array :: (array: *Bit_Array, count: int) {
    assert(count >= 0);
    if !array.allocator.proc array.allocator = context.allocator;

    words := (count + 63) / 64;
    memory := array.allocator.proc(.ALLOCATE, words * 8, 0, null, array.allocator.data);

    assert(memory != null || words == 0);
    memset(memory, 0, words * 8);
    array.slots = .{ words, cast(*s64) memory };
    array.count = count;
}

make_bit_array :: (count: int) -> Bit_Array {
    result: Bit_Array;
    init_bit_array(*result, count);
    return result;
}

deinit :: (using array: *Bit_Array) {
    if slots.data allocator.proc(.FREE, 0, slots.count * 8, slots.data, allocator.data);
    slots.data = null;
}

set_bit :: (using array: *Bit_Array, i: int) {
    assert(i >= 0 && i < count);
    slots[i / 64] |= (cast(s64) 1) << (i % 64);
}

clear_bit :: (using array: *Bit_Array, i: int) {
    assert(i >= 0 && i < count);
    slots[i / 64] &= ~((cast(s64) 1) << (i % 64));
}

toggle_bit :: (using array: *Bit_Array, i: int) {
    assert(i >= 0 && i < count);
    slots[i / 64] ^= (cast(s64) 1) << (i % 64);
}

set_bit_to :: (using array: *Bit_Array, i: int, value: bool) {
    if value set_bit(array, i);
    else clear_bit(array, i);
}

clear_all_bits :: (using array: *Bit_Array) {
    memset(slots.data, 0, slots.count * 8);
}

set_all_bits :: (using array: *Bit_Array) {
    memset(slots.data, 255, slots.count * 8);
}

toggle_all_bits :: (using array: *Bit_Array) {
    for *word: slots word.* = ~word.*;
}

read_bit :: (array: *Bit_Array, i: int) -> bool {
    assert(i >= 0 && i < array.count);
    return (array.slots[i / 64] & ((cast(s64) 1) << (i % 64))) != 0;
}

for_expansion :: (array: *Bit_Array, body: Code, flags: For_Flags) #expand {
    assert(flags == 0);

    for i: 0..array.count - 1 {
        `it := read_bit(array, i);
        `it_index := i;
        #insert body;
    }
}

only_set :: #bake_arguments only_set_or_unset(target_value = true);
only_unset :: #bake_arguments only_set_or_unset(target_value = false);

operator [] :: (array: Bit_Array, i: int) -> bool {
    return read_bit(*array, i);
}

operator []= :: (array: *Bit_Array, i: int, value: bool) {
    set_bit_to(array, i, value);
}

#scope_file
#import "Basic";

only_set_or_unset :: (array: *Bit_Array, body: Code, flags: For_Flags, $target_value: bool) #expand {
    assert(flags == 0);

    for i: 0..array.count - 1 {
        value := read_bit(array, i);
        if value == target_value {
            `it := value;
            `it_index := i;
            #insert body;
        }
    }
}
