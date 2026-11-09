// Run with jacket bench --path benchmarks/stdlib.
#import "Extensions/Testing";
#import "Basic";
#load "../../../stdlib/Extensions/Byte_Search/module.jai";

TEXT_BYTES :: 8 * 1024 * 1024;

scalar_find :: (text: string, byte: u8, start: int) -> int {
    for i: start..text.count - 1 if text[i] == byte return i;
    return -1;
}

byte_search_case :: (b: *Bench, operation: int) {
    text := alloc_string(TEXT_BYTES);

    defer free(text);
    state: u32 = 7;

    for *text {
        state = state * 1664525 + 1013904223;
        it.* = cast(u8) (#char "a" + (state >> 24) % 26);
    }

    finder := make_finder("xyzqzx");

    if operation == 2 || operation == 5 || operation == 7 || operation == 9 || operation == 11 {
        assert(CPU != .X64 || cpu_features().avx2, "wide benchmark requires AVX2");
    }

    set_bytes(b, text.count);
    reset_timer(b);

    for 0..b.iterations - 1 {
        input := black_box(text);
        offset := black_box(it % 7);
        result: int;

        if operation == {
            case 0; result = scalar_find(input, #char "Q", offset);
            case 1; result = find_byte_narrow(input, #char "Q", offset);
            case 2; result = find_byte_wide(input, #char "Q", offset);
            case 3; result = find(input, #char "Q", offset);
            case 4; result = find_three_narrow(input, #char "Q", #char "X", #char "Z", offset);
            case 5; result = find_three_wide(input, #char "Q", #char "X", #char "Z", offset);
            case 6; result = find_last_byte_narrow(input, #char "Q", text.count - offset);
            case 7; result = find_last_byte_wide(input, #char "Q", text.count - offset);
            case 8; result = count_byte_narrow(input, #char "a");
            case 9; result = count_byte_wide(input, #char "a");
            case 10; result = find_finder_narrow(*finder, input, offset);
            case 11; result = find_finder_wide(*finder, input, offset);
        }

        black_box(result);
    }
}

find_scalar :: (b: *Bench) {
    byte_search_case(b, 0);
} @bench

find_narrow :: (b: *Bench) {
    byte_search_case(b, 1);
} @bench

find_wide :: (b: *Bench) {
    byte_search_case(b, 2);
} @bench

find_picked :: (b: *Bench) {
    byte_search_case(b, 3);
} @bench

three_narrow :: (b: *Bench) {
    byte_search_case(b, 4);
} @bench

three_wide :: (b: *Bench) {
    byte_search_case(b, 5);
} @bench

last_narrow :: (b: *Bench) {
    byte_search_case(b, 6);
} @bench

last_wide :: (b: *Bench) {
    byte_search_case(b, 7);
} @bench

count_narrow :: (b: *Bench) {
    byte_search_case(b, 8);
} @bench

count_wide :: (b: *Bench) {
    byte_search_case(b, 9);
} @bench

substring_narrow :: (b: *Bench) {
    byte_search_case(b, 10);
} @bench

substring_wide :: (b: *Bench) {
    byte_search_case(b, 11);
} @bench
