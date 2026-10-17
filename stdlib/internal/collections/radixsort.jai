RadixSort :: struct {
    ranks: [] u32;
    ranks2: [] u32;
    valid_ranks: bool;
    allocator: Allocator;
}

reset :: (using radix: *RadixSort) {
    valid_ranks = false;
}

resize :: (using radix: *RadixSort, new_size: u32) {
    if ranks.count == new_size return;
    if !allocator.proc allocator = context.allocator;

    resize_rank_buffer(*ranks, new_size, allocator);
    resize_rank_buffer(*ranks2, new_size, allocator);
    valid_ranks = false;
}

free_resources :: (using radix: *RadixSort) {
    if ranks.data free_rank_buffer(ranks, allocator);
    if ranks2.data free_rank_buffer(ranks2, allocator);

    ranks = .{};
    ranks2 = .{};
    valid_ranks = false;
} @Releases

rank :: (using radix: *RadixSort, i: int) -> u32 {
    assert(valid_ranks && i >= 0 && i < ranks.count);
    return ranks[i];
}

operator [] :: (using radix: RadixSort, i: int) -> u32 {
    return rank(*radix, i);
}

sort :: (using radix: *RadixSort, input: *u32, count: u32) {
    sort_ranks(radix, input, count);
}

sort :: (using radix: *RadixSort, input: *u64, count: u32) {
    sort_ranks(radix, input, count);
}

sort :: (using radix: *RadixSort, input: *float, count: u32) {
    sort_ranks(radix, input, count);
}

sort :: (using radix: *RadixSort, input: [] u32) {
    sort(radix, input.data, cast(u32) input.count);
}

sort :: (using radix: *RadixSort, input: [] u64) {
    sort(radix, input.data, cast(u32) input.count);
}

sort :: (using radix: *RadixSort, input: [] float) {
    sort(radix, input.data, cast(u32) input.count);
}

reorder :: (input: *[..] $T, radix: RadixSort) {
    assert(radix.valid_ranks && radix.ranks.count == input.count);
    result: [..] T;
    result.allocator = input.allocator;
    array_resize(*result, input.count, false);
    for index, i: radix.ranks result[i] = input.data[index];

    array_free(input.*);
    input.* = result;
}

radix_tests :: () {
    sorter: RadixSort;

    defer free_resources(*sorter);
    input: [3] u32 = .[9, 1, 4];

    sort(*sorter, input);
    assert(sorter.ranks[0] == 1 && sorter.ranks[1] == 2 && sorter.ranks[2] == 0);
}

#scope_file
#import "Basic";

resize_rank_buffer :: (buffer: *[] u32, size: u32, allocator: Allocator) {
    capacity: s64;
    raw: *u8;

    if buffer.data {
        raw = cast(*u8) buffer.data - 8;
        capacity = (cast(*s64) raw).*;
    }

    if size > capacity {
        grown := max(cast(s64) size, max(cast(s64) 16, capacity * 2));
        memory := allocator.proc(.RESIZE, grown * 4 + 8, capacity * 4 + 8, raw, allocator.data);

        assert(memory != null, "rank allocation failed");
        (cast(*s64) memory).* = grown;
        buffer.data = cast(*u32) (cast(*u8) memory + 8);
    }

    buffer.count = size;
}

free_rank_buffer :: (buffer: [] u32, allocator: Allocator) {
    raw := cast(*u8) buffer.data - 8;
    capacity := (cast(*s64) raw).*;
    allocator.proc(.FREE, 0, capacity * 4 + 8, raw, allocator.data);
} @Releases

sort_ranks :: (radix: *RadixSort, input: *$T, count: u32) {
    if !input || count == 0 return;

    resize(radix, count);
    for i: 0..cast(s64) count - 1 radix.ranks[i] = cast(u32) i;

    for pass: 0..size_of(T) - 1 {
        histogram: [256] u32;

        for index: radix.ranks {
            key := sortable_key(input + index);
            byte := (key >> (pass * 8)) & 255;
            histogram[byte] += 1;
        }

        offset: u32;

        for *frequency: histogram {
            next := offset + frequency.*;
            frequency.* = offset;
            offset = next;
        }

        for index: radix.ranks {
            key := sortable_key(input + index);
            byte := (key >> (pass * 8)) & 255;
            radix.ranks2[histogram[byte]] = index;
            histogram[byte] += 1;
        }

        swap(*radix.ranks, *radix.ranks2);
    }

    radix.valid_ranks = true;
}

sortable_key :: (input: *$T) -> u64 {
    #if T == float {
        bits := (cast(*u32) input).*;
        if (bits & 0x80000000) != 0 return cast(u64) ~bits;
        return cast(u64) (bits ^ 0x80000000);
    } else return cast(u64) input.*;
}
