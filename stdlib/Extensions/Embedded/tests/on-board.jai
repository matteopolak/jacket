// Tests that run on a board, not on the machine building them:
//
//     jacket test --target Qemu_Virt_Aarch64 --path stdlib/Extensions/Embedded
//
// They use only what exists with no operating system: memory, the heap and the format code.

#import "Basic";
#import "String";
#import "Extensions/Testing";
Embedded :: #import "Extensions/Embedded";

heap_blocks_are_aligned_and_distinct :: (t: *Test) {
    first := alloc(3);
    second := alloc(5);
    third := alloc(1);

    defer free(first);
    defer free(second);
    defer free(third);

    expect(first != null && second != null && third != null);
    expect(cast(u64) first % 16 == 0);
    expect(cast(u64) second % 16 == 0);
    expect(cast(u64) third % 16 == 0);
    expect(first != second && second != third);
} @test

dynamic_arrays_grow_on_the_heap :: (t: *Test) {
    numbers: [..] int;

    defer array_free(numbers);

    for 1..1000 array_add(*numbers, it * it);

    expect(numbers.count == 1000);
    expect(numbers[999] == 1000000);
} @test

strings_are_built_in_memory :: (t: *Test) {
    text := tprint("% and %", 42, "ok");

    expect(text == "42 and ok");
    expect(contains(text, "and"));
} @test

volatile_accesses_reach_memory :: (t: *Test) {
    cell: u32 = 7;

    Embedded.volatile_store(*cell, 0xc0ffee);
    expect(Embedded.volatile_load(*cell) == 0xc0ffee);
} @test

block_copies_and_compares_use_the_runtime_memory_functions :: (t: *Test) {
    Big :: struct {
        numbers: [64] s32;
    }

    left: Big;
    right: Big;

    for 0..63 left.numbers[it] = cast(s32) (it * 3);

    right = left;

    equal := true;
    for 0..63 if right.numbers[it] != left.numbers[it] equal = false;

    expect(equal);

    right.numbers[63] = 0;
    expect(right.numbers[63] != left.numbers[63]);
} @test

wide_arithmetic_and_floats_work :: (t: *Test) {
    big: s64 = 123456789012345;
    divisor: s64 = 1000;

    expect(big / divisor == 123456789012);
    expect(big % divisor == 345);

    value: float64 = 2.5;

    expect(value * 4 == 10);
    expect(cast(s64) (value * 1000) == 2500);
} @test
