#import "Basic";
#import "String";
#import "Extensions/Toml";

Stats :: struct {
    allocations: int;
}

count_numbers :: (mode: Allocator_Mode, size: s64, old_size: s64, memory: *void, data: *void) -> *void {
    stats := cast(*Stats) data;
    if mode == .ALLOCATE stats.allocations += 1;
    backing := context.default_allocator;
    return backing.proc(mode, size, old_size, memory, backing.data);
}

rejected_allocations :: (text: string) -> int {
    stats: Stats;

    {
        push_allocator(count_numbers, *stats);
        doc, error := parse_toml(text, .{ max_number_bytes = 16384 });

        assert(error.kind == .RANGE && error.span.start == 2);
        assert(toml_error_path(*error) == "$[\"x\"]");
        free_toml(*doc);
    }

    return stats.allocations;
}

main :: () {
    digits: [8192] u8;
    for *digits it.* = 57;

    short := tprint("x=%", slice(string.{ digits.count, digits.data }, 0, 32));
    long := tprint("x=%", string.{ digits.count, digits.data });
    short_count := rejected_allocations(short);
    long_count := rejected_allocations(long);

    // Integer grammar/range checks need no decoded staging buffer, even beyond the
    // string builder's inline capacity. Both inputs retain only normal parser state.
    assert(short_count == long_count, tprint("short=% long=%", short_count, long_count));
    print("ok\n");
}
